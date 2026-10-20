// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The session state machine: trials in order, plays and replays, responses scored
//! against the stimulus's ground truth, pauses, and resumption from the log.
//!
//! The session does no I/O and reads no clock. The caller passes the monotonic time of
//! each action and writes each returned [`LogRecord`] as one line as soon as it gets it,
//! so a log is complete up to the last action even if the process stops. To resume, the
//! caller reads the log back and calls [`Session::recover`]: trials with a `trial_end`
//! record are done, and the session continues at the first trial without one.

use super::log::{Event, LOG_SCHEMA, LogRecord};
use super::protocol::{Protocol, ResponseSpec, TaskKind, is_code};
use super::schedule::{Schedule, Trial, stimulus_of};
use crate::harness::generators::GeneratorError;
use crate::harness::stimulus::{Query, Stimulus, Truth};

/// Who and what a session is: everything in the `session_start` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionInfo {
    /// The session ID.
    pub session_id: String,
    /// The pseudonymous participant code.
    pub participant_code: String,
    /// The SONARITHM version.
    pub engine_version: String,
    /// SHA-256 of the protocol file, as 64 hexadecimal digits.
    pub protocol_sha256: String,
    /// Audio context entered by the researcher.
    pub context: AudioContext,
}

/// Audio context, recorded only as entered (evaluation-plan "Calibration").
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioContext {
    /// The playback level as set or calibrated.
    pub playback_level: Option<String>,
    /// The kind of output device.
    pub output_device: Option<String>,
    /// The headphone model.
    pub headphone_model: Option<String>,
    /// Assistive technology as reported.
    pub assistive_tech: Option<String>,
}

/// Where a prepared stimulus came from, logged with `trial_start`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Prepared {
    /// The hash of the bound spec.
    pub spec_hash: Option<String>,
    /// The data fingerprint of the bound stimulus.
    pub data_fingerprint: Option<String>,
}

/// What the caller should do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Between trials: start the next one.
    Ready,
    /// In a trial, before its first play.
    InTrial,
    /// In a trial after a play: replay or respond.
    Listening,
    /// Paused; resume to continue.
    Paused,
    /// Every trial is done.
    Done,
}

/// The result of a response.
#[derive(Clone, Debug, PartialEq)]
pub struct Scored {
    /// The records to write: `response`, then `trial_end`.
    pub records: Vec<LogRecord>,
    /// The correct answer, to read when the block gives feedback.
    pub feedback: Option<String>,
    /// Whether a break is due before the next trial.
    pub break_due: bool,
}

/// Why an action was refused.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SessionError {
    /// The action does not fit the current state.
    #[error("not possible now: {0}")]
    State(&'static str),
    /// The response could not be read; the message says what to type.
    #[error("{0}")]
    Response(String),
    /// The trial's stimulus could not be generated.
    #[error(transparent)]
    Stimulus(#[from] GeneratorError),
    /// The log does not belong to this protocol and session.
    #[error("the log does not match: {0}")]
    Log(String),
    /// A session ID or participant code is not a short code.
    #[error("use 1 to 32 letters, digits and hyphens for the {0}, never a name")]
    Code(&'static str),
}

fn check_codes(info: &SessionInfo) -> Result<(), SessionError> {
    if !is_code(&info.session_id) {
        return Err(SessionError::Code("session ID"));
    }
    if !is_code(&info.participant_code) {
        return Err(SessionError::Code("participant code"));
    }
    Ok(())
}

/// A running session.
#[derive(Debug)]
pub struct Session {
    protocol: Protocol,
    schedule: Schedule,
    info: SessionInfo,
    run: u32,
    next: usize,
    state: State,
    paused_from: State,
    plays: u32,
    last_play_ms: Option<u64>,
    stimulus: Option<Stimulus>,
    done_since_break: u32,
}

impl Session {
    /// Starts a new session at time `t_ms`.
    ///
    /// # Errors
    ///
    /// [`SessionError::Code`] unless the session ID and participant code are short codes
    /// of letters, digits and hyphens: logs hold pseudonymous codes, never names
    /// (ADR-0012).
    pub fn start(
        protocol: Protocol,
        order_seed: u64,
        info: SessionInfo,
        t_ms: u64,
    ) -> Result<(Self, LogRecord), SessionError> {
        check_codes(&info)?;
        let schedule = Schedule::build(&protocol, order_seed);
        let mut session = Self {
            protocol,
            schedule,
            info,
            run: 1,
            next: 0,
            state: State::Ready,
            paused_from: State::Ready,
            plays: 0,
            last_play_ms: None,
            stimulus: None,
            done_since_break: 0,
        };
        if session.schedule.is_empty() {
            session.state = State::Done;
        }
        let record = session.session_record(Event::SessionStart, t_ms);
        Ok((session, record))
    }

    /// Resumes a session from its log at time `t_ms` of a new run. Trials that have a
    /// `trial_end` record are done; the session continues at the first one without.
    ///
    /// # Errors
    ///
    /// [`SessionError::Log`] if the log has no `session_start` for `info.session_id`, or
    /// if its protocol hash differs from `info.protocol_sha256`.
    pub fn recover(
        protocol: Protocol,
        info: SessionInfo,
        records: &[LogRecord],
        t_ms: u64,
    ) -> Result<(Self, LogRecord), SessionError> {
        check_codes(&info)?;
        let ours: Vec<&LogRecord> = records
            .iter()
            .filter(|record| record.session_id == info.session_id)
            .collect();
        let start = ours
            .iter()
            .find(|record| record.event == Event::SessionStart && record.run == 1)
            .ok_or_else(|| {
                SessionError::Log(format!("no session_start for session {}", info.session_id))
            })?;
        if start.protocol_sha256.as_deref() != Some(info.protocol_sha256.as_str()) {
            return Err(SessionError::Log(
                "the protocol file changed since the session started; resume with the original file".to_owned(),
            ));
        }
        let order_seed = start.order_seed.unwrap_or(protocol.seed);
        let schedule = Schedule::build(&protocol, order_seed);
        let done: std::collections::BTreeSet<usize> = ours
            .iter()
            .filter(|record| record.event == Event::TrialEnd)
            .filter_map(|record| record.trial)
            .collect();
        let next = (0..schedule.len())
            .find(|index| !done.contains(index))
            .unwrap_or(schedule.len());
        let run = ours.iter().map(|record| record.run).max().unwrap_or(1) + 1;
        let mut session = Self {
            protocol,
            schedule,
            info,
            run,
            next,
            state: State::Ready,
            paused_from: State::Ready,
            plays: 0,
            last_play_ms: None,
            stimulus: None,
            done_since_break: 0,
        };
        if session.next >= session.schedule.len() {
            session.state = State::Done;
        }
        let mut record = session.session_record(Event::Resume, t_ms);
        record.trial = Some(session.next);
        Ok((session, record))
    }

    /// The current state.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// The protocol.
    #[must_use]
    pub const fn protocol(&self) -> &Protocol {
        &self.protocol
    }

    /// The schedule.
    #[must_use]
    pub const fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    /// The trial in progress, or the next one to start.
    #[must_use]
    pub fn trial(&self) -> Option<&Trial> {
        self.schedule.trials.get(self.next)
    }

    /// The stimulus of the trial in progress.
    #[must_use]
    pub const fn stimulus(&self) -> Option<&Stimulus> {
        self.stimulus.as_ref()
    }

    /// The question for the trial in progress, with `{x}` (the point-estimation query's
    /// clock hour), `{from}` and `{to}` (the trend query's interval) filled in.
    #[must_use]
    pub fn prompt(&self) -> String {
        let truth = self.stimulus.as_ref().map(|stimulus| &stimulus.truth);
        let x = truth
            .and_then(first_query)
            .map_or_else(String::new, |query| number(query.x));
        let (from, to) = match truth {
            Some(Truth::Reversals(truth)) => truth
                .trend_queries
                .first()
                .map_or_else(Default::default, |query| {
                    (number(query.from), number(query.to))
                }),
            _ => Default::default(),
        };
        self.protocol
            .task
            .prompt
            .replace("{x}", &x)
            .replace("{from}", &from)
            .replace("{to}", &to)
    }

    /// Starts the next trial: generates its stimulus. The caller prepares the sound
    /// (binds the stimulus into the condition's spec and renders it) and passes where it
    /// came from in `prepared`.
    ///
    /// # Errors
    ///
    /// [`SessionError::State`] outside `Ready`; [`SessionError::Stimulus`] if generation
    /// fails.
    pub fn start_trial(
        &mut self,
        t_ms: u64,
        prepared: &Prepared,
    ) -> Result<LogRecord, SessionError> {
        if self.state != State::Ready {
            return Err(SessionError::State("start a trial only between trials"));
        }
        let trial = self
            .schedule
            .trials
            .get(self.next)
            .ok_or(SessionError::State("no trials left"))?;
        let stimulus = stimulus_of(&self.protocol, trial)?;
        let mut record = self.trial_record(Event::TrialStart, t_ms);
        if let Some(stimulus) = &stimulus {
            record.stimulus_id = Some(stimulus.generator.as_str().to_owned());
            record.generator_version = Some(stimulus.generator.version());
            record.seed = Some(stimulus.seed);
            record.stimulus_sha256 = Some(stimulus.digest().to_string());
        }
        record.spec_hash.clone_from(&prepared.spec_hash);
        record
            .data_fingerprint
            .clone_from(&prepared.data_fingerprint);
        record.manifest_ref = self
            .protocol
            .condition(&trial.condition)
            .map(|condition| condition.spec.clone());
        self.stimulus = stimulus;
        self.plays = 0;
        self.last_play_ms = None;
        self.state = State::InTrial;
        Ok(record)
    }

    /// Records that the stimulus started playing: `play` the first time, `replay` after.
    ///
    /// # Errors
    ///
    /// [`SessionError::State`] outside a trial.
    pub fn play(&mut self, t_ms: u64) -> Result<LogRecord, SessionError> {
        if !matches!(self.state, State::InTrial | State::Listening) {
            return Err(SessionError::State("play only during a trial"));
        }
        let event = if self.plays == 0 {
            Event::Play
        } else {
            Event::Replay
        };
        self.plays += 1;
        self.last_play_ms = Some(t_ms);
        self.state = State::Listening;
        let mut record = self.trial_record(event, t_ms);
        record.plays = Some(self.plays);
        Ok(record)
    }

    /// Scores a typed response and ends the trial.
    ///
    /// # Errors
    ///
    /// [`SessionError::State`] before the first play; [`SessionError::Response`] with
    /// what to type if the text cannot be read. Nothing is logged for an unreadable
    /// response, so the listener can simply type again.
    pub fn respond(&mut self, t_ms: u64, text: &str) -> Result<Scored, SessionError> {
        if self.state != State::Listening {
            return Err(SessionError::State("respond after the sound has played"));
        }
        let answer = read_response(&self.protocol.task.response, text)?;
        let truth = self.stimulus.as_ref().map(|stimulus| &stimulus.truth);
        let score = score(
            self.protocol.task.kind,
            &self.protocol.task.response,
            &answer,
            truth,
        );
        let mut response = self.trial_record(Event::Response, t_ms);
        response.response = Some(answer.text.clone());
        response.correct = score.correct;
        response.true_value = score.true_value;
        response.true_answer.clone_from(&score.true_answer);
        response.error = score.error;
        response.rt_ms = self.last_play_ms.map(|start| t_ms.saturating_sub(start));
        response.plays = Some(self.plays);
        let mut end = self.trial_record(Event::TrialEnd, t_ms);
        end.plays = Some(self.plays);
        let feedback = self.trial().filter(|trial| trial.feedback).and_then(|_| {
            score
                .true_answer
                .clone()
                .or_else(|| score.true_value.map(number))
        });
        self.next += 1;
        self.stimulus = None;
        self.done_since_break += 1;
        let every = self.protocol.break_every;
        let break_due =
            every > 0 && self.done_since_break >= every && self.next < self.schedule.len();
        if break_due {
            self.done_since_break = 0;
        }
        self.state = if self.next >= self.schedule.len() {
            State::Done
        } else {
            State::Ready
        };
        Ok(Scored {
            records: vec![response, end],
            feedback,
            break_due,
        })
    }

    /// Pauses. A trial in progress is aborted and runs again from its start on resume.
    ///
    /// # Errors
    ///
    /// [`SessionError::State`] when already paused or done.
    pub fn pause(&mut self, t_ms: u64) -> Result<Vec<LogRecord>, SessionError> {
        if matches!(self.state, State::Paused | State::Done) {
            return Err(SessionError::State("pause only while running"));
        }
        let mut records = Vec::new();
        if matches!(self.state, State::InTrial | State::Listening) {
            records.push(self.trial_record(Event::TrialAbort, t_ms));
            self.stimulus = None;
        }
        self.paused_from = State::Ready;
        self.state = State::Paused;
        let mut record = self.session_base(Event::Pause, t_ms);
        record.trial = Some(self.next);
        records.push(record);
        Ok(records)
    }

    /// Resumes after [`pause`](Self::pause) in the same run.
    ///
    /// # Errors
    ///
    /// [`SessionError::State`] unless paused.
    pub fn resume(&mut self, t_ms: u64) -> Result<LogRecord, SessionError> {
        if self.state != State::Paused {
            return Err(SessionError::State("resume only when paused"));
        }
        self.state = self.paused_from;
        let mut record = self.session_base(Event::Resume, t_ms);
        record.trial = Some(self.next);
        Ok(record)
    }

    /// A researcher's note.
    #[must_use]
    pub fn note(&self, t_ms: u64, text: &str) -> LogRecord {
        let mut record = self.session_base(Event::Note, t_ms);
        record.trial = self.trial().map(|trial| trial.index);
        record.note = Some(text.trim().to_owned());
        record
    }

    /// Ends the session, early or after the last trial.
    #[must_use]
    pub fn end(&mut self, t_ms: u64) -> Vec<LogRecord> {
        let mut records = Vec::new();
        if matches!(self.state, State::InTrial | State::Listening) {
            records.push(self.trial_record(Event::TrialAbort, t_ms));
        }
        let mut record = self.session_base(Event::SessionEnd, t_ms);
        record.trials = Some(self.next);
        records.push(record);
        self.state = State::Done;
        records
    }

    fn session_base(&self, event: Event, t_ms: u64) -> LogRecord {
        LogRecord {
            event,
            t_mono_ms: t_ms,
            run: self.run,
            exp_id: self.protocol.exp_id.clone(),
            session_id: self.info.session_id.clone(),
            participant_code: self.info.participant_code.clone(),
            ..LogRecord::default()
        }
    }

    fn session_record(&self, event: Event, t_ms: u64) -> LogRecord {
        let context = &self.info.context;
        LogRecord {
            log_schema: Some(LOG_SCHEMA.to_owned()),
            protocol_version: Some(self.protocol.protocol_version.clone()),
            protocol_sha256: Some(self.info.protocol_sha256.clone()),
            engine_version: Some(self.info.engine_version.clone()),
            order_seed: Some(self.schedule.order_seed),
            trials: Some(self.schedule.len()),
            playback_level: context.playback_level.clone(),
            output_device: context.output_device.clone(),
            headphone_model: context.headphone_model.clone(),
            assistive_tech: context.assistive_tech.clone(),
            ..self.session_base(event, t_ms)
        }
    }

    fn trial_record(&self, event: Event, t_ms: u64) -> LogRecord {
        let trial = self.trial();
        LogRecord {
            trial: trial.map(|trial| trial.index),
            block: trial.map(|trial| trial.block.clone()),
            condition: trial.map(|trial| trial.condition.clone()),
            practice: trial.map(|trial| trial.practice),
            ..self.session_base(event, t_ms)
        }
    }
}

/// A response read from text.
#[derive(Clone, Debug, PartialEq)]
struct Answer {
    /// The normalized text that is logged.
    text: String,
    /// The number, for number responses.
    number: Option<f64>,
    /// The option index from 0, for choice responses.
    choice: Option<usize>,
}

fn read_response(spec: &ResponseSpec, text: &str) -> Result<Answer, SessionError> {
    let text = text.trim();
    match spec {
        ResponseSpec::Number { min, max, unit } => {
            let unit = unit
                .as_deref()
                .map_or(String::new(), |unit| format!(" {unit}"));
            let ask = || {
                SessionError::Response(format!(
                    "Type a number from {min} to {max}{unit}, then press Enter."
                ))
            };
            let number: f64 = text.parse().map_err(|_| ask())?;
            if !number.is_finite() || number < *min || number > *max {
                return Err(ask());
            }
            Ok(Answer {
                text: format!("{}", number + 0.0),
                number: Some(number),
                choice: None,
            })
        }
        ResponseSpec::Choice { options } => {
            let ask = || {
                SessionError::Response(format!(
                    "Type a number from 1 to {}, or the option's name, then press Enter.",
                    options.len()
                ))
            };
            let index = match text.parse::<usize>() {
                Ok(number) if (1..=options.len()).contains(&number) => number - 1,
                Ok(_) => return Err(ask()),
                Err(_) => options
                    .iter()
                    .position(|option| option.eq_ignore_ascii_case(text))
                    .ok_or_else(ask)?,
            };
            Ok(Answer {
                text: options.get(index).cloned().unwrap_or_default(),
                number: None,
                choice: Some(index),
            })
        }
    }
}

#[derive(Default)]
struct Score {
    correct: Option<bool>,
    true_value: Option<f64>,
    true_answer: Option<String>,
    error: Option<f64>,
}

fn score(kind: TaskKind, spec: &ResponseSpec, answer: &Answer, truth: Option<&Truth>) -> Score {
    let option = |index: usize| match spec {
        ResponseSpec::Choice { options } => options.get(index).cloned(),
        ResponseSpec::Number { .. } => None,
    };
    match (kind, truth) {
        (TaskKind::PointEstimation, Some(truth)) => {
            let true_value = first_query(truth).map(|query| query.value);
            Score {
                true_value,
                error: true_value
                    .zip(answer.number)
                    .map(|(truth, given)| given - truth),
                ..Score::default()
            }
        }
        (TaskKind::Count, Some(Truth::Transits(truth))) => {
            let count = f64::from(truth.transits);
            Score {
                correct: answer.number.map(|given| given.total_cmp(&count).is_eq()),
                true_value: Some(count),
                ..Score::default()
            }
        }
        (TaskKind::Choice, Some(Truth::Correlations(truth))) => Score {
            correct: answer.choice.map(|given| given == truth.strongest),
            true_answer: option(truth.strongest),
            ..Score::default()
        },
        (TaskKind::TrendIdentification, Some(Truth::Reversals(truth))) => {
            let expected = truth
                .trend_queries
                .first()
                .map(|query| query.answer.option());
            Score {
                correct: expected
                    .zip(answer.choice)
                    .map(|(expected, given)| expected == given),
                true_answer: expected.and_then(option),
                ..Score::default()
            }
        }
        _ => Score::default(),
    }
}

fn first_query(truth: &Truth) -> Option<&Query> {
    match truth {
        Truth::Reversals(truth) => truth.queries.first(),
        Truth::Prices(truth) => truth.queries.first(),
        Truth::Transits(_) | Truth::Correlations(_) => None,
    }
}

/// Formats a number for a prompt or feedback: shortest form, no `-0`.
fn number(value: f64) -> String {
    format!("{}", value + 0.0)
}
