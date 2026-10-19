// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `sonarithm` binary end to end: every command's text, plain and JSON output, its
//! exit code (11 §2), and the screen-reader rules for `--plain`.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn sonarithm() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sonarithm"));
    command.current_dir(fixtures()).env("NO_COLOR", "1");
    command
}

/// The exit code, standard output and standard error of one run.
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_in(dir: &Path, args: &[&str], stdin: Option<&str>) -> Run {
    let mut command = sonarithm();
    command.current_dir(dir).args(args);
    if let Some(stdin) = stdin {
        command.write_stdin(stdin);
    }
    let output = command.output().unwrap();
    Run {
        code: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn run(args: &[&str]) -> Run {
    run_in(&fixtures(), args, None)
}

impl Run {
    fn transcript(&self) -> String {
        format!(
            "exit code: {}\n--- stdout\n{}--- stderr\n{}",
            self.code, self.stdout, self.stderr
        )
    }

    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.stdout).unwrap()
    }
}

/// Snapshots a transcript with the release-dependent parts replaced: the crate version
/// and the Score hash (which covers the engine versions).
macro_rules! snapshot {
    ($name:expr, $run:expr) => {
        insta::with_settings!({
            filters => vec![
                (r"(sonarithm|numisona|platform)( |\x22: \x22)\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?", "$1$2[version]"),
                (r"(Score hash: |\x22score_hash\x22: \x22)[0-9a-f]{64}", "$1[score hash]"),
            ],
        }, {
            insta::assert_snapshot!($name, $run.transcript());
        });
    };
}

/// Box drawing and ANSI escapes have no place in `--plain` output (11 §2).
fn assert_linear(run: &Run) {
    for text in [&run.stdout, &run.stderr] {
        assert!(
            !text
                .chars()
                .any(|c| ('\u{2500}'..='\u{257F}').contains(&c) || c == '\u{1b}'),
            "box drawing or escapes in plain output:\n{text}"
        );
    }
}

#[test]
fn the_version_and_help_succeed_and_usage_errors_exit_with_2() {
    let version = run(&["--version"]);
    assert_eq!(version.code, 0);
    assert_eq!(
        version.stdout.trim_end(),
        format!("sonarithm {}", env!("CARGO_PKG_VERSION"))
    );
    snapshot!("help", run(&["--help"]));
    let bare = run(&[]);
    assert_eq!(bare.code, 2, "{}", bare.transcript());
    let unknown = run(&["validate", "--colour", "temps.spec.json"]);
    assert_eq!(unknown.code, 2, "{}", unknown.transcript());
}

#[test]
fn validate_reports_validity_and_exits_by_severity() {
    let valid = run(&["--plain", "validate", "temps.spec.json"]);
    assert_eq!(valid.code, 0);
    snapshot!("validate_valid", valid);
    let invalid = run(&["--plain", "validate", "invalid.spec.json"]);
    assert_eq!(invalid.code, 1);
    assert_linear(&invalid);
    snapshot!("validate_invalid", invalid);
    let json = run(&["--json", "validate", "invalid.spec.json"]);
    assert_eq!(json.code, 1);
    assert!(json.stderr.is_empty(), "{}", json.stderr);
    snapshot!("validate_invalid_json", json);
    // A flagged concept with automatic polarity warns (SDR-14).
    let warned = run(&["--plain", "validate", "prices.spec.json"]);
    assert_eq!(warned.code, 0);
    assert!(warned.stderr.contains("SNR-SPEC-042"), "{}", warned.stderr);
    let denied = run(&["--plain", "validate", "--deny-warnings", "prices.spec.json"]);
    assert_eq!(denied.code, 3);
}

#[test]
fn verbose_lists_the_applied_defaults_with_their_rules() {
    let verbose = run(&["--plain", "validate", "--verbose", "temps.spec.json"]);
    assert_eq!(verbose.code, 0);
    assert!(
        verbose.stderr.contains("SNR-SPEC-080"),
        "{}",
        verbose.stderr
    );
    assert!(verbose.stderr.contains("SDR-08"), "{}", verbose.stderr);
    assert!(!verbose.stderr.contains("--verbose lists"));
}

#[test]
fn specs_can_come_from_standard_input() {
    let text = std::fs::read_to_string(fixtures().join("prices.spec.json")).unwrap();
    let piped = run_in(&fixtures(), &["--json", "validate", "-"], Some(&text));
    assert_eq!(piped.code, 0);
    assert_eq!(piped.json()["spec"], "stdin.json");
}

#[test]
fn unreadable_files_exit_with_1_and_name_no_directories() {
    let missing = run(&["--plain", "validate", "../fixtures/absent.spec.json"]);
    assert_eq!(missing.code, 1);
    assert!(
        missing.stderr.contains("`absent.spec.json` cannot be read"),
        "{}",
        missing.stderr
    );
    assert!(!missing.stderr.contains("fixtures"), "{}", missing.stderr);
    let json = run(&["--json", "inspect", "absent.spec.json"]);
    assert_eq!(json.code, 1);
    assert_eq!(json.json()["ok"], false);
}

#[test]
fn inspect_prints_the_summary_description_and_outline() {
    let inspected = run(&["--plain", "inspect", "--describe", "temps.spec.json"]);
    assert_eq!(inspected.code, 0);
    assert_linear(&inspected);
    snapshot!("inspect_describe", inspected);
    let tree = run(&["--plain", "inspect", "--tree", "estimated.spec.json"]);
    assert_eq!(tree.code, 0);
    assert_linear(&tree);
    snapshot!("inspect_tree", tree);
    let json = run(&["--json", "inspect", "--tree", "temps.spec.json"]);
    assert_eq!(json.code, 0);
    let value = json.json();
    assert_eq!(value["ok"], true);
    assert_eq!(value["summary"]["tracks"][0]["notes"], 10);
    assert!(value["description"]["l1"].is_array());
    assert_eq!(
        value["score"]["tracks"][0]["events"]
            .as_array()
            .unwrap()
            .len(),
        10
    );
    let failed = run(&["--plain", "inspect", "invalid.spec.json"]);
    assert_eq!(failed.code, 1);
    assert!(
        failed.stdout.contains("did not compile"),
        "{}",
        failed.stdout
    );
}

#[test]
fn the_cli_compiles_the_same_score_as_the_facade() {
    let json = run(&["--json", "inspect", "temps.spec.json"]);
    let spec = sonarithm::Spec::from_file(fixtures().join("temps.spec.json")).unwrap();
    let compiled = sonarithm::Sonarithm::new().compile(&spec).unwrap();
    assert_eq!(
        json.json()["summary"]["score_hash"],
        compiled.score_hash().unwrap(),
        "same spec, same Score bytes"
    );
}

#[test]
fn lint_is_a_stub_that_lists_warnings() {
    let linted = run(&["--plain", "lint", "temps.spec.json"]);
    assert_eq!(linted.code, 0);
    assert!(linted.stderr.contains("SNR-COMP-023"), "{}", linted.stderr);
    assert!(linted.stderr.contains("ECHOMETRIC"), "{}", linted.stderr);
    assert_eq!(run(&["lint", "--deny-warnings", "temps.spec.json"]).code, 3);
    let json = run(&["--json", "lint", "temps.spec.json"]);
    assert_eq!(json.json()["stub"], true);
    assert_eq!(json.json()["counts"]["warnings"], 1);
}

#[test]
fn quick_sonifies_a_data_file_with_the_defaults() {
    let quick = run(&[
        "--plain",
        "quick",
        "temps.csv",
        "--x",
        "year",
        "--y",
        "anomaly",
    ]);
    assert_eq!(quick.code, 0, "{}", quick.transcript());
    assert_linear(&quick);
    snapshot!("quick", quick);
    let json = run(&[
        "--json",
        "quick",
        "temps.csv",
        "--mark",
        "glide",
        "--range",
        "300,900",
        "--concept",
        "temperature",
    ]);
    assert_eq!(json.code, 0, "{}", json.transcript());
    let value = json.json();
    assert_eq!(value["summary"]["tracks"][0]["glides"], 1);
    assert_eq!(
        value["summary"]["tracks"][0]["pitch_range_hz"],
        serde_json::json!([300.0, 900.0])
    );
    let bad = run(&["--plain", "quick", "temps.csv", "--y", "nope"]);
    assert_eq!(bad.code, 1);
    assert!(bad.stderr.contains("nope"), "{}", bad.stderr);
    let range = run(&["quick", "temps.csv", "--range", "900,300"]);
    assert_eq!(range.code, 2, "{}", range.transcript());
}

#[test]
fn quick_writes_a_spec_with_a_relative_data_path() {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-quick");
    std::fs::create_dir_all(&out).unwrap();
    let spec_path = out.join("temps.spec.json");
    let data = fixtures().join("temps.csv");
    let written = run_in(
        &out,
        &[
            "--plain",
            "quick",
            data.to_str().unwrap(),
            "--spec-out",
            "temps.spec.json",
        ],
        None,
    );
    assert_eq!(written.code, 0, "{}", written.transcript());
    assert!(
        written
            .stdout
            .contains("Wrote the spec to temps.spec.json.")
    );
    let text = std::fs::read_to_string(&spec_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let path = value["data"]["source"]["path"].as_str().unwrap();
    assert!(path.ends_with("fixtures/temps.csv"), "{path}");
    assert!(!Path::new(path).is_absolute(), "relative (P8): {path}");
    // The written spec compiles where it was saved.
    let reread = run_in(&out, &["--json", "inspect", "temps.spec.json"], None);
    assert_eq!(reread.code, 0, "{}", reread.transcript());
}

#[test]
fn schema_prints_the_published_spec_schema() {
    let schema = run(&["schema"]);
    assert_eq!(schema.code, 0);
    let published = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../schemas/sonarithm-spec-0.1.schema.json"),
    )
    .unwrap();
    assert_eq!(schema.stdout.trim_end(), published.trim_end());
}

#[test]
fn spec_migrate_reports_what_it_did() {
    let current = run(&["--plain", "spec", "migrate", "temps.spec.json"]);
    assert_eq!(current.code, 0);
    assert_eq!(
        current.stdout,
        std::fs::read_to_string(fixtures().join("temps.spec.json")).unwrap()
    );
    assert!(
        current.stderr.contains("already at spec version 0.1"),
        "{}",
        current.stderr
    );
    let json = run(&["--json", "spec", "migrate", "--in-place", "temps.spec.json"]);
    assert_eq!(json.code, 0);
    assert_eq!(json.json()["changed"], false);
    assert_eq!(json.json()["written"], false);
    let unversioned = run(&["--plain", "spec", "migrate", "unversioned.spec.json"]);
    assert_eq!(unversioned.code, 1);
    snapshot!("migrate_unversioned", unversioned);
    assert_eq!(
        run(&["spec", "migrate", "--to", "zero", "temps.spec.json"]).code,
        2
    );
    assert_eq!(
        run(&["spec", "migrate", "--to", "9.0", "temps.spec.json"]).code,
        1
    );
    assert_eq!(run(&["spec", "migrate", "--in-place", "-"]).code, 2);
}
