// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `sonarithm study check` and `sonarithm study run` end to end on the pilot protocol
//! (examples/studies/pilot-001), with scripted answers on standard input and the silent
//! audio output. No participant data is involved (OD-04). The dialogue's accessibility
//! rules are checked on its text: one plain line per prompt, no escape sequences.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn pilot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/studies/pilot-001/protocol.toml")
}

/// A fresh folder for one test's log.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("study")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn study(args: &[&str], stdin: &str) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_sonarithm"))
        .env("NO_COLOR", "1")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap();
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn run_args<'a>(log: &'a str, protocol: &'a str) -> Vec<&'a str> {
    vec![
        "study",
        "run",
        protocol,
        "--participant",
        "P001",
        "--audio",
        "null",
        "--log",
        log,
    ]
}

/// The JSON records of a log.
fn records(path: &Path) -> Vec<serde_json::Value> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn events(records: &[serde_json::Value]) -> Vec<String> {
    records
        .iter()
        .map(|r| r["event"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn check_summarizes_the_pilot() {
    let protocol = pilot();
    let run = study(&["study", "check", protocol.to_str().unwrap()], "");
    assert_eq!(run.code, 0, "{}", run.stderr);
    insta::assert_snapshot!(run.stdout);
}

#[test]
fn check_reports_an_invalid_protocol() {
    let dir = scratch("invalid");
    let path = dir.join("protocol.toml");
    std::fs::write(&path, "format = \"sonarithm-study/9\"\n").unwrap();
    let run = study(&["study", "check", path.to_str().unwrap()], "");
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("SNR-ENG-600"), "{}", run.stderr);
}

#[test]
fn the_pilot_runs_end_to_end() {
    let dir = scratch("full");
    let log = dir.join("pilot.jsonl");
    let protocol = pilot();
    // For each trial: Enter to listen, then an answer. The fourth answer is followed by the
    // break prompt's Enter.
    let script = "\n50\n\n61\n\n70\n\n44\n\n\n55\n\n60\n\n65\n\n70\n";
    let run = study(
        &run_args(log.to_str().unwrap(), protocol.to_str().unwrap()),
        script,
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("There are 8 trials."));
    assert!(
        run.stdout.contains("The answer was"),
        "practice trials give feedback"
    );
    assert!(
        run.stdout
            .ends_with("The session is complete. Thank you.\n"),
        "{}",
        run.stdout
    );
    let records = records(&log);
    let ends = records.iter().filter(|r| r["event"] == "trial_end").count();
    assert_eq!(ends, 8);
    assert_eq!(records.last().unwrap()["event"], "session_end");
    assert_eq!(records[0]["log_schema"], "sonarithm-study-log/0");
    assert_eq!(records[0]["participant_code"], "P001");
    // Every response is scored against the stimulus's truth.
    for response in records.iter().filter(|r| r["event"] == "response") {
        let error = response["error"].as_f64().unwrap();
        let truth = response["true_value"].as_f64().unwrap();
        let given: f64 = response["response"].as_str().unwrap().parse().unwrap();
        assert!((given - truth - error).abs() < 1e-9);
    }
    // Each trial start carries the stimulus provenance and the bound spec's hashes.
    for start in records.iter().filter(|r| r["event"] == "trial_start") {
        assert_eq!(start["stimulus_id"], "GEN-002");
        assert_eq!(start["spec_hash"].as_str().unwrap().len(), 64);
        assert_eq!(start["data_fingerprint"].as_str().unwrap().len(), 64);
        assert!(["none.json", "dynamic.json"].contains(&start["manifest_ref"].as_str().unwrap()));
    }
    // No path reaches the log.
    let text = std::fs::read_to_string(&log).unwrap();
    assert!(!text.contains(":\\\\") && !text.contains("/Users/") && !text.contains("\\\\Users"));
}

#[test]
fn a_session_pauses_quits_and_resumes() {
    let dir = scratch("resume");
    let log = dir.join("pilot.jsonl");
    let protocol = pilot();
    let args = run_args(log.to_str().unwrap(), protocol.to_str().unwrap());
    // Trial 1 answered; trial 2 listened to, paused, then quit at the pause prompt.
    let first = study(&args, "\n50\n\np\nq\n");
    assert_eq!(first.code, 0, "{}", first.stderr);
    assert!(
        first.stdout.contains("Saved after 1 of 8 trials."),
        "{}",
        first.stdout
    );
    let after_first = events(&records(&log));
    assert_eq!(
        after_first[after_first.len() - 2..],
        ["trial_abort".to_owned(), "pause".to_owned()]
    );
    // A second run without --resume refuses to overwrite the log.
    let again = study(&args, "");
    assert_eq!(again.code, 1);
    assert!(again.stderr.contains("--resume"), "{}", again.stderr);
    // Resuming repeats trial 2 and finishes in run 2. The break comes after the fourth
    // trial of the run (trial 5): a resume is itself a break, so the count restarts.
    let mut resume = args.clone();
    resume.push("--resume");
    let script = "\n61\n\n70\n\n44\n\n55\n\n\n60\n\n65\n\n70\n";
    let second = study(&resume, script);
    assert_eq!(second.code, 0, "{}", second.stderr);
    assert!(
        second
            .stdout
            .contains("Resuming session P001-s1 at trial 2 of 8."),
        "{}",
        second.stdout
    );
    let all = records(&log);
    let ends: Vec<u64> = all
        .iter()
        .filter(|r| r["event"] == "trial_end")
        .map(|r| r["trial"].as_u64().unwrap())
        .collect();
    assert_eq!(ends, (0..8).collect::<Vec<_>>());
    assert!(all.iter().any(|r| r["run"] == 2));
}

#[test]
fn a_name_is_refused_as_a_participant_code() {
    let dir = scratch("name");
    let protocol = pilot();
    let log = dir.join("x.jsonl");
    let run = study(
        &[
            "study",
            "run",
            protocol.to_str().unwrap(),
            "--participant",
            "Jane Doe",
            "--audio",
            "null",
            "--log",
            log.to_str().unwrap(),
        ],
        "",
    );
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("never a name"), "{}", run.stderr);
    assert!(!log.exists());
}

#[test]
fn the_dialogue_is_plain_lines_for_screen_readers() {
    let dir = scratch("plain");
    let log = dir.join("pilot.jsonl");
    let protocol = pilot();
    let run = study(
        &run_args(log.to_str().unwrap(), protocol.to_str().unwrap()),
        "\nh\nr\n50\n\nq\n",
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(!run.stdout.contains('\u{1b}'), "no escape sequences");
    assert!(!run.stdout.contains('\r'), "no carriage returns or redraws");
    for line in run.stdout.lines() {
        assert!(!line.trim().is_empty(), "no blank lines");
        assert!(
            line.ends_with('.') || line.ends_with('?'),
            "every line is a full sentence: {line}"
        );
    }
    assert!(
        run.stdout
            .contains("r listens again, p pauses, q saves and quits")
    );
    // A replay is logged as such.
    let events = events(&records(&log));
    assert!(events.contains(&"replay".to_owned()));
}

#[test]
fn json_mode_keeps_standard_output_for_the_summary() {
    let dir = scratch("json");
    let log = dir.join("pilot.jsonl");
    let protocol = pilot();
    let mut args = vec!["--json"];
    args.extend(run_args(log.to_str().unwrap(), protocol.to_str().unwrap()));
    let run = study(&args, "\nq\n");
    assert_eq!(run.code, 0, "{}", run.stderr);
    let summary: serde_json::Value = serde_json::from_str(&run.stdout).unwrap();
    assert_eq!(summary["done"], 0);
    assert_eq!(summary["finished"], false);
    assert_eq!(summary["log"], "pilot.jsonl");
    assert!(run.stderr.contains("Trial 1 of 8 (practice)."));
}
