// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `sonarithm study check` and `sonarithm study run`: the front end of ECHOMETRIC's study
//! runner (A08-T03; 07, ECHOMETRIC, study runner).
//!
//! The runner library decides the trials, scores the answers and writes the log records;
//! this module reads files, compiles each trial's sound through the facade, plays it, and
//! talks to the listener.
//!
//! **Accessible by construction.** Every prompt is one plain line on standard output and
//! every answer is a line typed and sent with Enter: no raw keyboard mode, no redraws, no
//! colour, nothing that moves. Screen readers read each line as it arrives, in order, and
//! braille displays show the current line. At any prompt the same short commands work:
//! `r` listens again, `p` pauses, `q` saves and quits, `h` lists them.
//!
//! **Research use (OD-04).** The project has no ethics review route, so this command must
//! not collect data from participants as research. The log holds a pseudonymous code,
//! never a name, and no paths (ADR-0012).

use std::collections::BTreeMap;
use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use sonarithm::units::{Frames, SampleRate, Seconds};
use sonarithm::{Compiled, Datasets, Diagnostics, Sonarithm, Spec};
use sonarithm_audio::{AudioBackend, CpalBackend, NullBackend, NullConfig, Player, StreamConfig};
use sonarithm_echometric::harness::Stimulus;
use sonarithm_echometric::harness::study::schedule::stimulus_of;
use sonarithm_echometric::harness::study::{
    AudioContext, LogRecord, Prepared, Protocol, STIMULUS, Schedule, Session, SessionError,
    SessionInfo, State, bind_spec, digest, is_code, parse_log,
};
use sonarithm_render::PlanConfig;
use sonarithm_render::graph::Command as Transport;

use crate::cli::{StudyAudio, StudyCheck, StudyRun};
use crate::output::{Exit, Out, error, json_value, line};

/// How often the playback loop checks the transport.
const TICK: Duration = Duration::from_millis(10);

/// What the listener can type at any prompt.
const HELP: &str = "Type your answer and press Enter. Other commands: r listens again, p pauses, q saves and quits, h repeats this help.";

/// A protocol with its condition specs, read from disk.
struct Loaded {
    name: String,
    text: String,
    dir: PathBuf,
    protocol: Protocol,
    /// Condition ID → (spec file name relative to the protocol, spec text).
    specs: BTreeMap<String, (String, String)>,
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.to_string_lossy().into_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// Reads and checks a protocol and its condition specs. Problems are printed.
fn load(path: &Path, out: Out) -> Option<Loaded> {
    let name = file_name(path);
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            error(&format!("`{name}` cannot be read: {err}"));
            return None;
        }
    };
    let protocol = match Protocol::from_toml(&text) {
        Ok(protocol) => protocol,
        Err(diagnostics) => {
            out.diagnostics(&name, &text, &diagnostics, false);
            error(&format!(
                "`{name}` is not a valid study protocol; the problems are listed above."
            ));
            return None;
        }
    };
    let dir = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let mut specs = BTreeMap::new();
    for condition in &protocol.conditions {
        match std::fs::read_to_string(dir.join(&condition.spec)) {
            Ok(spec) => {
                specs.insert(condition.id.clone(), (condition.spec.clone(), spec));
            }
            Err(err) => {
                error(&format!(
                    "condition `{}`: its spec `{}` cannot be read: {err}",
                    condition.id, condition.spec
                ));
                return None;
            }
        }
    }
    Some(Loaded {
        name,
        text,
        dir,
        protocol,
        specs,
    })
}

/// Compiles a condition's spec over a stimulus.
fn compile(
    snr: &Sonarithm,
    loaded: &Loaded,
    condition: &str,
    stimulus: &Stimulus,
) -> Result<Compiled, (String, String, Diagnostics)> {
    let Some((file, text)) = loaded.specs.get(condition) else {
        return Err((condition.to_owned(), String::new(), Diagnostics::new()));
    };
    let fail = |diagnostics: Diagnostics| (file.clone(), text.clone(), diagnostics);
    let bound = bind_spec(text).map_err(|err| {
        error(&format!("`{file}`: {err}"));
        fail(Diagnostics::new())
    })?;
    let spec = Spec::from_json_str(bound)
        .map(|spec| spec.with_name(file).with_base_dir(&loaded.dir))
        .map_err(|err| fail(err.diagnostics()))?;
    let bundle = stimulus.to_bundle(STIMULUS).map_err(|err| {
        error(&format!("the stimulus cannot be loaded: {err}"));
        fail(Diagnostics::new())
    })?;
    snr.compile_with(&spec, &Datasets::new().with(STIMULUS, bundle))
        .map_err(|err| fail(err.diagnostics()))
}

/// Compiles every condition with the first stimulus of each block; prints problems.
fn check_conditions(snr: &Sonarithm, loaded: &Loaded, schedule: &Schedule, out: Out) -> bool {
    let mut ok = true;
    let mut checked = std::collections::BTreeSet::new();
    for trial in &schedule.trials {
        if !checked.insert((trial.block.clone(), trial.condition.clone())) {
            continue;
        }
        let stimulus = match stimulus_of(&loaded.protocol, trial) {
            Ok(Some(stimulus)) => stimulus,
            Ok(None) => continue,
            Err(err) => {
                error(&format!("block `{}`: {err}", trial.block));
                ok = false;
                continue;
            }
        };
        if let Err((file, text, diagnostics)) = compile(snr, loaded, &trial.condition, &stimulus) {
            out.diagnostics(&file, &text, &diagnostics, false);
            error(&format!(
                "condition `{}` does not compile with block `{}`'s stimulus.",
                trial.condition, trial.block
            ));
            ok = false;
        }
    }
    ok
}

#[derive(Serialize)]
struct Checked {
    protocol: String,
    ok: bool,
    exp_id: String,
    order_seed: u64,
    trials: usize,
    practice_trials: usize,
    blocks: Vec<BlockSummary>,
}

#[derive(Serialize)]
struct BlockSummary {
    id: String,
    trials: usize,
    conditions: Vec<String>,
    generator: String,
}

/// `sonarithm study check`.
pub(crate) fn check(args: &StudyCheck, out: Out) -> Exit {
    let Some(loaded) = load(&args.protocol, out) else {
        return Exit::Errors;
    };
    let order_seed = args.order_seed.unwrap_or(loaded.protocol.seed);
    let schedule = Schedule::build(&loaded.protocol, order_seed);
    let snr = Sonarithm::new();
    let ok = check_conditions(&snr, &loaded, &schedule, out);
    let blocks: Vec<BlockSummary> = loaded
        .protocol
        .blocks
        .iter()
        .map(|block| BlockSummary {
            id: block.id.clone(),
            trials: schedule
                .trials
                .iter()
                .filter(|t| t.block == block.id)
                .count(),
            conditions: block.conditions.clone(),
            generator: block.stimulus.generator().as_str().to_owned(),
        })
        .collect();
    let summary = Checked {
        protocol: loaded.name.clone(),
        ok,
        exp_id: loaded.protocol.exp_id.clone(),
        order_seed,
        trials: schedule.len(),
        practice_trials: schedule.trials.iter().filter(|t| t.practice).count(),
        blocks,
    };
    if out.json() {
        json_value(&summary);
    } else {
        line(format!(
            "{}: {} ({}), {} trials, {} of them practice, order seed {}.",
            summary.protocol,
            summary.exp_id,
            loaded.protocol.title,
            summary.trials,
            summary.practice_trials,
            summary.order_seed
        ));
        for block in &summary.blocks {
            line(format!(
                "Block {}: {} trials over {}, stimuli from {}.",
                block.id,
                block.trials,
                block.conditions.join(" and "),
                block.generator
            ));
        }
        line(if ok {
            "Every condition compiles with its block's stimuli."
        } else {
            "Some conditions do not compile; see the errors above."
        });
    }
    if ok { Exit::Success } else { Exit::Errors }
}

/// Reads one line of input; `None` at the end of input.
fn read_line() -> Option<String> {
    let mut text = String::new();
    match std::io::stdin().lock().read_line(&mut text) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(text.trim().to_owned()),
    }
}

/// Says one line to the listener: on standard output, or on standard error with `--json`,
/// whose standard output is the one JSON summary.
fn say(out: Out, text: impl AsRef<str>) {
    if out.json() {
        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(stderr, "{}", text.as_ref());
    } else {
        line(text);
        let _ = std::io::stdout().flush();
    }
}

/// Says a prompt and waits for a line.
fn ask(out: Out, prompt: &str) -> Option<String> {
    say(out, prompt);
    read_line()
}

/// The log file, opened for appending; every record is written and flushed at once.
struct Log {
    file: std::fs::File,
}

impl Log {
    fn write(&mut self, records: &[LogRecord]) -> Result<(), String> {
        for record in records {
            writeln!(self.file, "{}", record.to_line())
                .and_then(|()| self.file.flush())
                .map_err(|err| format!("the log cannot be written: {err}"))?;
        }
        Ok(())
    }
}

/// Plays a Score to its end on the chosen audio output.
fn play(score: &sonarithm::Score, audio: StudyAudio, device: Option<&str>) -> Result<(), String> {
    match audio {
        StudyAudio::Device => play_on(CpalBackend::new(), score, device),
        StudyAudio::Null => {
            // The silent output runs as fast as it can, for scripted pilots and tests.
            let mut config = NullConfig::new(48_000, 256);
            config.paced = false;
            play_on(NullBackend::new(config), score, device)
        }
    }
}

fn play_on<B: AudioBackend>(
    backend: B,
    score: &sonarithm::Score,
    device: Option<&str>,
) -> Result<(), String> {
    let mut config = StreamConfig::default();
    if let Some(wanted) = device {
        let devices = backend.devices().map_err(|err| err.to_string())?;
        let found = devices
            .iter()
            .find(|d| d.id == wanted || d.name.eq_ignore_ascii_case(wanted))
            .ok_or_else(|| format!("no output device is called `{wanted}`"))?;
        config.device = Some(found.id.clone());
    }
    let mut player = Player::open(
        backend,
        score.clone(),
        PlanConfig::new(SampleRate::HZ_48000),
        config,
        256,
    )
    .map_err(|err| format!("the sound cannot play: {err}"))?;
    let rate = player
        .stream()
        .map(sonarithm_audio::Stream::sample_rate)
        .ok_or_else(|| "the audio stream did not open".to_owned())?;
    let end = Seconds(score.header.duration.0)
        .to_frames(SampleRate::new(rate).unwrap_or(SampleRate::HZ_48000))
        .unwrap_or(Frames(0));
    player.send(Transport::Play);
    loop {
        player
            .tick()
            .map_err(|err| format!("playback stopped: {err}"))?;
        if player.state().graph.position >= end {
            player.send(Transport::Stop);
            let _ = player.tick();
            return Ok(());
        }
        std::thread::sleep(TICK);
    }
}

/// Milliseconds since the start of this run.
fn elapsed(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// What a finished or stopped run reports with `--json`.
#[derive(Serialize)]
struct Ran {
    exp_id: String,
    session_id: String,
    trials: usize,
    done: usize,
    finished: bool,
    log: String,
}

/// `sonarithm study run`.
pub(crate) fn run(args: &StudyRun, out: Out) -> Exit {
    match run_session(args, out) {
        Ok(exit) => exit,
        Err(message) => {
            error(&message);
            Exit::Errors
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one linear dialogue with the listener reads best in one place"
)]
fn run_session(args: &StudyRun, out: Out) -> Result<Exit, String> {
    if !is_code(&args.participant) {
        return Err(
            "use a pseudonymous code of 1 to 32 letters, digits and hyphens for --participant, such as P001, never a name"
                .to_owned(),
        );
    }
    let Some(loaded) = load(&args.protocol, out) else {
        return Ok(Exit::Errors);
    };
    let session_id = args
        .session
        .clone()
        .unwrap_or_else(|| format!("{}-s1", args.participant));
    let log_path = args
        .log
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("{}-{session_id}.jsonl", loaded.protocol.exp_id)));
    let info = SessionInfo {
        session_id: session_id.clone(),
        participant_code: args.participant.clone(),
        engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        protocol_sha256: digest(&loaded.text),
        context: AudioContext {
            playback_level: args.playback_level.clone(),
            output_device: args.output_device.clone(),
            headphone_model: args.headphones.clone(),
            assistive_tech: args.assistive_tech.clone(),
        },
    };
    let snr = Sonarithm::new();
    let order_seed = args.order_seed.unwrap_or(loaded.protocol.seed);
    if !check_conditions(
        &snr,
        &loaded,
        &Schedule::build(&loaded.protocol, order_seed),
        out,
    ) {
        return Ok(Exit::Errors);
    }
    let start = Instant::now();
    let (mut session, first) = if args.resume {
        let text = std::fs::read_to_string(&log_path)
            .map_err(|err| format!("the log `{}` cannot be read: {err}", file_name(&log_path)))?;
        let records = parse_log(&text)
            .map_err(|(row, err)| format!("the log's line {row} cannot be read: {err}"))?;
        Session::recover(loaded.protocol.clone(), info, &records, 0)
            .map_err(|err| err.to_string())?
    } else {
        if log_path.exists() {
            return Err(format!(
                "the log `{}` already exists; add --resume to continue its session, or choose another --log or --session",
                file_name(&log_path)
            ));
        }
        Session::start(loaded.protocol.clone(), order_seed, info, 0)
            .map_err(|err| err.to_string())?
    };
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|err| format!("the log `{}` cannot be opened: {err}", file_name(&log_path)))?;
    let mut log = Log { file };
    log.write(&[first])?;

    let total = session.schedule().len();
    say(out, format!("{}.", loaded.protocol.title));
    if args.resume {
        say(
            out,
            format!(
                "Resuming session {session_id} at trial {} of {total}.",
                session.trial().map_or(total, |t| t.index + 1)
            ),
        );
    } else {
        for paragraph in &loaded.protocol.instructions {
            say(out, paragraph);
        }
        say(out, format!("There are {total} trials. {HELP}"));
    }
    let mut finished = false;
    'trials: while session.state() == State::Ready {
        let Some(trial) = session.trial().cloned() else {
            break;
        };
        let stimulus = stimulus_of(&loaded.protocol, &trial)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| "the trial has no stimulus".to_owned())?;
        let compiled = compile(&snr, &loaded, &trial.condition, &stimulus).map_err(
            |(file, text, diagnostics)| {
                out.diagnostics(&file, &text, &diagnostics, false);
                format!("condition `{}` does not compile", trial.condition)
            },
        )?;
        let header = &compiled.score().header;
        let prepared = Prepared {
            spec_hash: Some(header.spec_hash.to_string()),
            data_fingerprint: Some(header.data_fingerprint.to_string()),
        };
        log.write(&[session
            .start_trial(elapsed(start), &prepared)
            .map_err(|err| err.to_string())?])?;
        let kind = if trial.practice { " (practice)" } else { "" };
        say(out, format!("Trial {} of {total}{kind}.", trial.index + 1));
        let question = session.prompt();
        // The question comes first, as in the published studies, then the sound.
        match ask(out, &format!("{question} Press Enter to listen.")).as_deref() {
            None | Some("q") => {
                log.write(&session.pause(elapsed(start)).map_err(|e| e.to_string())?)?;
                break 'trials;
            }
            _ => {}
        }
        log.write(&[session.play(elapsed(start)).map_err(|e| e.to_string())?])?;
        play(compiled.score(), args.audio, args.device.as_deref())?;
        loop {
            let Some(input) = ask(
                out,
                &format!("{question} Type your answer and press Enter."),
            ) else {
                log.write(&session.pause(elapsed(start)).map_err(|e| e.to_string())?)?;
                break 'trials;
            };
            match input.to_ascii_lowercase().as_str() {
                "h" | "help" => say(out, HELP),
                "r" => {
                    log.write(&[session.play(elapsed(start)).map_err(|e| e.to_string())?])?;
                    play(compiled.score(), args.audio, args.device.as_deref())?;
                }
                "p" => {
                    log.write(&session.pause(elapsed(start)).map_err(|e| e.to_string())?)?;
                    if ask(
                        out,
                        "Paused. Press Enter to continue, or type q to save and quit.",
                    )
                    .is_none_or(|answer| answer == "q")
                    {
                        break 'trials;
                    }
                    log.write(&[session.resume(elapsed(start)).map_err(|e| e.to_string())?])?;
                    // A pause ends the trial; it starts again with the same sound.
                    continue 'trials;
                }
                "q" => {
                    log.write(&session.pause(elapsed(start)).map_err(|e| e.to_string())?)?;
                    break 'trials;
                }
                answer => match session.respond(elapsed(start), answer) {
                    Ok(scored) => {
                        log.write(&scored.records)?;
                        if let Some(feedback) = scored.feedback {
                            say(out, format!("The answer was {feedback}."));
                        }
                        if scored.break_due
                            && ask(out, "You can take a break now. Press Enter when you are ready to go on.")
                                .is_none()
                        {
                            break 'trials;
                        }
                        continue 'trials;
                    }
                    Err(SessionError::Response(hint)) => say(out, hint),
                    Err(err) => return Err(err.to_string()),
                },
            }
        }
    }
    let done = session.trial().map_or(total, |trial| trial.index);
    if session.state() == State::Done {
        log.write(&session.end(elapsed(start)))?;
        finished = true;
        say(out, "The session is complete. Thank you.");
    } else {
        say(
            out,
            format!(
                "Saved after {done} of {total} trials. To continue, run the same command with --resume."
            ),
        );
    }
    if out.json() {
        json_value(&Ran {
            exp_id: loaded.protocol.exp_id.clone(),
            session_id,
            trials: total,
            done,
            finished,
            log: file_name(&log_path),
        });
    }
    Ok(Exit::Success)
}
