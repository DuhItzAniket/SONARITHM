// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The study runner library: protocol checks, deterministic trial order, scoring against
//! ground truth, the JSONL log, pause, and resumption from the log. Responses are scripted;
//! no participant data is involved (OD-04).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    reason = "integration test; clippy treats its helpers as non-test code, and scores are compared exactly"
)]

use sonarithm_echometric::harness::Truth;
use sonarithm_echometric::harness::study::{
    AudioContext, Event, LogRecord, Prepared, Protocol, Schedule, Session, SessionError,
    SessionInfo, State, parse_log,
};

const PILOT: &str = r#"
format = "sonarithm-study/0"
exp-id = "PILOT-001"
protocol-version = "1"
title = "Pilot: point estimation with Smith & Walker context conditions"
seed = 20261019
instructions = ["You will hear a stock price over one trading day.", "The opening price is 50 dollars."]
break-every = 4

[task]
kind = "point-estimation"
prompt = "What was the price at {x} o'clock?"
response = { kind = "number", min = 0, max = 120, unit = "dollars" }

[[conditions]]
id = "none"
label = "No context"
spec = "conditions/none.json"

[[conditions]]
id = "dynamic"
label = "Dynamic reference tone"
spec = "conditions/dynamic.json"

[[blocks]]
id = "practice"
practice = true
feedback = true
conditions = ["none"]
repeats = 2
order = "fixed"
stimulus = { generator = "GEN-002" }

[[blocks]]
id = "main"
conditions = ["none", "dynamic"]
repeats = 3
stimulus = { generator = "GEN-002" }
"#;

fn info() -> SessionInfo {
    SessionInfo {
        session_id: "P001-s1".to_owned(),
        participant_code: "P001".to_owned(),
        engine_version: "0.0.0".to_owned(),
        protocol_sha256: "0".repeat(64),
        context: AudioContext::default(),
    }
}

fn pilot() -> Protocol {
    Protocol::from_toml(PILOT).unwrap()
}

fn codes(text: &str) -> Vec<String> {
    match Protocol::from_toml(text) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .iter()
            .map(|d| d.code.as_str().to_owned())
            .collect(),
    }
}

/// The true answer of the current trial's first query.
fn truth_of(session: &Session) -> f64 {
    match &session.stimulus().unwrap().truth {
        Truth::Prices(truth) => truth.queries[0].value,
        Truth::Reversals(truth) => truth.queries[0].value,
        other => panic!("{other:?}"),
    }
}

/// Runs one trial: start, play, optionally replay, answer `answer`.
fn run_trial(
    session: &mut Session,
    t: &mut u64,
    answer: &str,
    replay: bool,
    log: &mut Vec<LogRecord>,
) -> bool {
    *t += 100;
    log.push(session.start_trial(*t, &Prepared::default()).unwrap());
    *t += 50;
    log.push(session.play(*t).unwrap());
    if replay {
        *t += 10_000;
        log.push(session.play(*t).unwrap());
    }
    *t += 2_000;
    let scored = session.respond(*t, answer).unwrap();
    log.extend(scored.records);
    scored.break_due
}

#[test]
fn the_pilot_protocol_reads_and_checks() {
    let protocol = pilot();
    assert_eq!(protocol.exp_id, "PILOT-001");
    assert!(protocol.check().is_empty());
    let schedule = Schedule::build(&protocol, protocol.seed);
    assert_eq!(schedule.len(), 2 + 6);
    // The practice block keeps its fixed order and comes first.
    assert!(
        schedule.trials[..2]
            .iter()
            .all(|trial| trial.practice && trial.condition == "none")
    );
    assert_eq!(
        schedule
            .trials
            .iter()
            .filter(|trial| trial.condition == "dynamic")
            .count(),
        3
    );
}

#[test]
fn trial_order_follows_the_seeds() {
    let protocol = pilot();
    let a = Schedule::build(&protocol, 1);
    let b = Schedule::build(&protocol, 1);
    assert_eq!(a, b);
    // Another order seed counterbalances: different stimulus seeds, and over a few seeds
    // a different condition order in the shuffled block.
    let orders: Vec<Vec<String>> = (2..8)
        .map(|seed| {
            Schedule::build(&protocol, seed)
                .trials
                .iter()
                .map(|t| t.condition.clone())
                .collect()
        })
        .collect();
    assert!(orders.iter().any(|order| *order != orders[0]));
    assert_ne!(
        a.trials[0].stimulus_seed,
        Schedule::build(&protocol, 2).trials[0].stimulus_seed
    );
}

#[test]
fn invalid_protocols_get_coded_diagnostics() {
    assert_eq!(codes("format = 1"), vec!["SNR-ENG-600"]);
    assert_eq!(
        codes(&PILOT.replace("sonarithm-study/0", "sonarithm-study/9")),
        vec!["SNR-ENG-601"]
    );
    assert_eq!(
        codes(&PILOT.replace("id = \"dynamic\"", "id = \"none\"")),
        vec!["SNR-ENG-602", "SNR-ENG-605"]
    );
    assert_eq!(
        codes(&PILOT.replace("conditions/none.json", "C:/Users/someone/none.json")),
        vec!["SNR-ENG-603"]
    );
    assert_eq!(
        codes(&PILOT.replace("conditions/none.json", "../none.json")),
        vec!["SNR-ENG-603"]
    );
    assert_eq!(
        codes(&PILOT.replace("repeats = 3", "repeats = 0")),
        vec!["SNR-ENG-604"]
    );
    assert_eq!(
        codes(&PILOT.replace("[\"none\", \"dynamic\"]", "[\"none\", \"loud\"]")),
        vec!["SNR-ENG-605"]
    );
    // A count task cannot be scored from price series.
    let count = PILOT.replace("kind = \"point-estimation\"", "kind = \"count\"");
    assert_eq!(codes(&count), vec!["SNR-ENG-606", "SNR-ENG-606"]);
    // Invalid generator parameters are reported at the block's stimulus.
    let bad_stimulus = PILOT.replace(
        "repeats = 3\nstimulus = { generator = \"GEN-002\" }",
        "repeats = 3\nstimulus = { generator = \"GEN-002\", queries = 40 }",
    );
    assert_eq!(codes(&bad_stimulus), vec!["SNR-ENG-606"]);
    let choice = PILOT.replace(
        "{ kind = \"number\", min = 0, max = 120, unit = \"dollars\" }",
        "{ kind = \"choice\", options = [\"a\", \"b\"] }",
    );
    assert!(codes(&choice).contains(&"SNR-ENG-607".to_owned()));
}

#[test]
fn a_parse_error_names_the_line() {
    let Err(diagnostics) = Protocol::from_toml("format = \"sonarithm-study/0\"\nexp-id = PILOT\n")
    else {
        panic!()
    };
    let message = &diagnostics.iter().next().unwrap().message;
    assert!(message.contains("line 2"), "{message}");
}

#[test]
fn participant_codes_must_not_be_names() {
    let mut named = info();
    named.participant_code = "Jane Doe".to_owned();
    assert!(matches!(
        Session::start(pilot(), 1, named, 0),
        Err(SessionError::Code("participant code"))
    ));
}

#[test]
fn a_pilot_session_runs_end_to_end_and_logs_every_event() {
    let protocol = pilot();
    let (mut session, start) = Session::start(protocol.clone(), protocol.seed, info(), 0).unwrap();
    let mut log = vec![start];
    let mut t = 0;
    let mut answers = Vec::new();
    let mut breaks = 0;
    while session.state() == State::Ready {
        // Scripted listener: answers $5 above the truth on even trials, $3 below on odd ones.
        log.push(
            session
                .start_trial(
                    t,
                    &Prepared {
                        spec_hash: Some("ab".repeat(32)),
                        data_fingerprint: Some("cd".repeat(32)),
                    },
                )
                .unwrap(),
        );
        let truth = truth_of(&session);
        assert!(session.prompt().ends_with("o'clock?"));
        let index = session.trial().unwrap().index;
        let answer = if index % 2 == 0 {
            truth + 5.0
        } else {
            truth - 3.0
        };
        answers.push((answer, truth));
        t += 100;
        log.push(session.play(t).unwrap());
        t += 1_500;
        let scored = session.respond(t, &format!("{answer}")).unwrap();
        // Feedback only in the practice block.
        assert_eq!(scored.feedback.is_some(), index < 2);
        breaks += usize::from(scored.break_due);
        log.extend(scored.records);
        t += 500;
    }
    assert_eq!(session.state(), State::Done);
    log.extend(session.end(t));
    assert_eq!(breaks, 1, "break-every = 4 offers one break in 8 trials");

    let responses: Vec<&LogRecord> = log.iter().filter(|r| r.event == Event::Response).collect();
    assert_eq!(responses.len(), 8);
    for (record, (answer, truth)) in responses.iter().zip(&answers) {
        assert_eq!(record.true_value, Some(*truth));
        assert_eq!(record.error, Some(answer - truth));
        assert_eq!(record.rt_ms, Some(1_500));
        assert_eq!(record.plays, Some(1));
    }
    // RMS error over the scored (non-practice) trials is computable from the log alone.
    let errors: Vec<f64> = responses
        .iter()
        .filter(|r| r.practice == Some(false))
        .filter_map(|r| r.error)
        .collect();
    assert_eq!(errors.len(), 6);

    // Every trial start names its stimulus and where it came from; no path is absolute.
    for record in log.iter().filter(|r| r.event == Event::TrialStart) {
        assert_eq!(record.stimulus_id.as_deref(), Some("GEN-002"));
        assert_eq!(record.stimulus_sha256.as_ref().map(String::len), Some(64));
        assert!(
            record.seed.is_some()
                && record.spec_hash.is_some()
                && record.data_fingerprint.is_some()
        );
        let spec = record.manifest_ref.as_deref().unwrap();
        assert!(spec.starts_with("conditions/") && !spec.contains(':'));
    }

    // The JSONL text round-trips.
    let text: String = log.iter().map(|record| record.to_line() + "\n").collect();
    assert_eq!(parse_log(&text).unwrap(), log);
    assert_eq!(
        log.first().unwrap().log_schema.as_deref(),
        Some("sonarithm-study-log/0")
    );
    assert_eq!(log.last().unwrap().event, Event::SessionEnd);
}

#[test]
fn the_log_of_a_scripted_session_is_pinned() {
    let protocol = pilot();
    let (mut session, start) = Session::start(protocol, 7, info(), 0).unwrap();
    let mut log = vec![start];
    let mut t = 0;
    for answer in ["50", "61.5", "abc"] {
        if answer == "abc" {
            // An unreadable answer is refused with a hint and logs nothing.
            log.push(session.start_trial(t, &Prepared::default()).unwrap());
            log.push(session.play(t + 10).unwrap());
            let Err(SessionError::Response(hint)) = session.respond(t + 20, answer) else {
                panic!()
            };
            assert_eq!(
                hint,
                "Type a number from 0 to 120 dollars, then press Enter."
            );
            let scored = session.respond(t + 30, " 42 ").unwrap();
            log.extend(scored.records);
        } else {
            run_trial(&mut session, &mut t, answer, true, &mut log);
        }
    }
    log.extend(session.end(t + 1_000));
    let text: String = log.iter().map(|record| record.to_line() + "\n").collect();
    insta::assert_snapshot!(text);
}

#[test]
fn pausing_aborts_the_trial_and_resuming_repeats_it() {
    let protocol = pilot();
    let (mut session, start) = Session::start(protocol, 3, info(), 0).unwrap();
    let mut log = vec![start];
    log.push(session.start_trial(10, &Prepared::default()).unwrap());
    log.push(session.play(20).unwrap());
    let first = session.trial().unwrap().index;
    log.extend(session.pause(30).unwrap());
    assert_eq!(session.state(), State::Paused);
    assert!(session.play(40).is_err());
    log.push(session.resume(5_000).unwrap());
    assert_eq!(session.state(), State::Ready);
    assert_eq!(session.trial().unwrap().index, first);
    let events: Vec<Event> = log.iter().map(|r| r.event).collect();
    assert_eq!(
        events,
        vec![
            Event::SessionStart,
            Event::TrialStart,
            Event::Play,
            Event::TrialAbort,
            Event::Pause,
            Event::Resume
        ]
    );
}

#[test]
fn a_session_resumes_from_its_log_in_a_new_run() {
    let protocol = pilot();
    let (mut session, start) = Session::start(protocol.clone(), 11, info(), 0).unwrap();
    let mut log = vec![start];
    let mut t = 0;
    for _ in 0..3 {
        run_trial(&mut session, &mut t, "40", false, &mut log);
    }
    // The process stops in the middle of the fourth trial.
    log.push(session.start_trial(t + 100, &Prepared::default()).unwrap());
    log.push(session.play(t + 200).unwrap());
    let text: String = log.iter().map(|record| record.to_line() + "\n").collect();
    drop(session);

    let records = parse_log(&text).unwrap();
    let (mut resumed, record) = Session::recover(protocol.clone(), info(), &records, 0).unwrap();
    assert_eq!(record.event, Event::Resume);
    assert_eq!(record.run, 2);
    assert_eq!(record.trial, Some(3));
    assert_eq!(resumed.trial().unwrap().index, 3);
    // The same order seed comes back from the log, so the remaining trials are the same.
    assert_eq!(resumed.schedule(), &Schedule::build(&protocol, 11));
    let mut log2 = vec![record];
    let mut t2 = 0;
    while resumed.state() == State::Ready {
        run_trial(&mut resumed, &mut t2, "40", false, &mut log2);
    }
    let done = log
        .iter()
        .chain(&log2)
        .filter(|r| r.event == Event::TrialEnd)
        .count();
    assert_eq!(done, 8);
    assert!(log2.iter().all(|r| r.run == 2));

    // A changed protocol file cannot resume the session.
    let mut changed = info();
    changed.protocol_sha256 = "1".repeat(64);
    assert!(matches!(
        Session::recover(protocol.clone(), changed, &records, 0),
        Err(SessionError::Log(_))
    ));
    // Nor can another session's ID.
    let mut other = info();
    other.session_id = "P002-s1".to_owned();
    assert!(matches!(
        Session::recover(protocol, other, &records, 0),
        Err(SessionError::Log(_))
    ));
}

#[test]
fn count_choice_and_trend_tasks_are_scored_against_the_truth() {
    let count = r#"
format = "sonarithm-study/0"
exp-id = "PILOT-002"
protocol-version = "1"
title = "Count transits"
seed = 5
[task]
kind = "count"
prompt = "How many dips did you hear?"
response = { kind = "number", min = 0, max = 4 }
[[conditions]]
id = "tones"
label = "Tones"
spec = "tones.json"
[[blocks]]
id = "main"
conditions = ["tones"]
repeats = 3
stimulus = { generator = "GEN-003", snr = 30, transits = 2 }
"#;
    let protocol = Protocol::from_toml(count).unwrap();
    let (mut session, _) = Session::start(protocol, 1, info(), 0).unwrap();
    session.start_trial(0, &Prepared::default()).unwrap();
    session.play(1).unwrap();
    let scored = session.respond(2, "2").unwrap();
    assert_eq!(scored.records[0].correct, Some(true));
    assert_eq!(scored.records[0].true_value, Some(2.0));

    let choice = count
        .replace("kind = \"count\"", "kind = \"choice\"")
        .replace(
            "{ kind = \"number\", min = 0, max = 4 }",
            "{ kind = \"choice\", options = [\"B\", \"C\", \"D\"] }",
        )
        .replace(
            "{ generator = \"GEN-003\", snr = 30, transits = 2 }",
            "{ generator = \"GEN-004\", delta = 0.1 }",
        );
    let protocol = Protocol::from_toml(&choice).unwrap();
    let (mut session, _) = Session::start(protocol, 1, info(), 0).unwrap();
    session.start_trial(0, &Prepared::default()).unwrap();
    let Truth::Correlations(truth) = &session.stimulus().unwrap().truth else {
        panic!()
    };
    let correct = ["B", "C", "D"][truth.strongest];
    session.play(1).unwrap();
    let scored = session.respond(2, &correct.to_lowercase()).unwrap();
    assert_eq!(scored.records[0].correct, Some(true));
    assert_eq!(scored.records[0].true_answer.as_deref(), Some(correct));

    let trend = count
        .replace("kind = \"count\"", "kind = \"trend-identification\"")
        .replace(
            "How many dips did you hear?",
            "From {from} to {to} o'clock, did the price increase, decrease or stay the same?",
        )
        .replace(
            "{ kind = \"number\", min = 0, max = 4 }",
            "{ kind = \"choice\", options = [\"increased\", \"decreased\", \"stayed the same\"] }",
        )
        .replace(
            "{ generator = \"GEN-003\", snr = 30, transits = 2 }",
            "{ generator = \"GEN-001\", rate = 2, reversals = 2 }",
        );
    let protocol = Protocol::from_toml(&trend).unwrap();
    let (mut session, _) = Session::start(protocol, 1, info(), 0).unwrap();
    session.start_trial(0, &Prepared::default()).unwrap();
    let Truth::Reversals(truth) = &session.stimulus().unwrap().truth else {
        panic!()
    };
    let query = truth.trend_queries[0];
    let prompt = session.prompt();
    assert!(
        prompt.starts_with(&format!("From {} to {} o'clock", query.from, query.to)),
        "{prompt}"
    );
    session.play(1).unwrap();
    let scored = session
        .respond(2, &(query.answer.option() + 1).to_string())
        .unwrap();
    assert_eq!(scored.records[0].correct, Some(true));
}

#[test]
fn actions_out_of_order_are_refused() {
    let (mut session, _) = Session::start(pilot(), 1, info(), 0).unwrap();
    assert!(matches!(session.play(0), Err(SessionError::State(_))));
    assert!(matches!(
        session.respond(0, "50"),
        Err(SessionError::State(_))
    ));
    session.start_trial(0, &Prepared::default()).unwrap();
    assert!(
        matches!(session.respond(1, "50"), Err(SessionError::State(_))),
        "respond only after the sound"
    );
    assert!(matches!(
        session.start_trial(2, &Prepared::default()),
        Err(SessionError::State(_))
    ));
}
