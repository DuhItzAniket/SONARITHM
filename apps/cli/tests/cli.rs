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
    // Tests never play through a real device (hearing safety, and CI has none): playback
    // uses the silent null backend unless a test asks otherwise.
    command
        .current_dir(fixtures())
        .env("NO_COLOR", "1")
        .env("SONARITHM_BACKEND", "null");
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
                // Semver pre-release identifiers are dot-separated, so a sentence's full stop
                // after `0.1.0-alpha` stays out of the match.
                (r"(sonarithm|numisona|platform|plotsona|curvasona)( |\x22: \x22)\d+\.\d+\.\d+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?", "$1$2[version]"),
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

/// C01-T12: `inspect` on a chart file prints its extraction report, the summary sentence
/// of 10 §5 (from the catalogue) and the Score of its chart-type template; `--json` gives
/// the report; a file with a chart extension that is no chart fails with PLOTSONA's code.
#[test]
fn inspect_reads_a_chart_file() {
    let corpus = "../../../../crates/engines/sonarithm-plotsona/tests/corpus";
    let plain = run(&["--plain", "inspect", &format!("{corpus}/svg/vl-line.svg")]);
    assert_eq!(plain.code, 0, "{}", plain.transcript());
    assert_linear(&plain);
    assert!(
        plain
            .stdout
            .contains("36 points: 0 exact, 36 calibrated, 0 estimated, median confidence")
    );
    snapshot!("inspect_chart", plain);
    let json = run(&[
        "--json",
        "inspect",
        &format!("{corpus}/office/xlsx-pie.xlsx"),
    ]);
    assert_eq!(json.code, 0, "{}", json.transcript());
    let value = json.json();
    assert_eq!(
        value["summary"],
        "5 points: 5 exact, 0 calibrated, 0 estimated"
    );
    assert_eq!(value["template"], "chart-pie");
    assert_eq!(value["report"]["provenance"]["exact"], 5);
    snapshot!("inspect_chart_json", json);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-chart");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("broken.svg"), "not a chart").unwrap();
    let broken = run_in(&dir, &["--plain", "inspect", "broken.svg"], None);
    assert_eq!(broken.code, 1, "{}", broken.transcript());
    assert!(
        broken.stderr.contains("cannot be read as a chart"),
        "{}",
        broken.transcript()
    );
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
    assert_eq!(value["played"]["finished"], true);
    let bad = run(&["--plain", "quick", "temps.csv", "--y", "nope"]);
    assert_eq!(bad.code, 1);
    assert!(bad.stderr.contains("nope"), "{}", bad.stderr);
    let range = run(&["quick", "temps.csv", "--range", "900,300"]);
    assert_eq!(range.code, 2, "{}", range.transcript());
}

/// C03-T07: `quick --function` plays a function as a glide with its critical-point
/// earcons, and prints the critical points as text first (A11Y-24, A11Y-28).
#[test]
fn quick_plays_a_function_with_its_critical_points_as_text() {
    let quick = run(&[
        "--plain",
        "quick",
        "--function",
        "x^3 - 3*x",
        "--domain=-3,3",
    ]);
    assert_eq!(quick.code, 0, "{}", quick.transcript());
    assert_linear(&quick);
    snapshot!("quick_function", quick);
    let json = run(&[
        "--json",
        "quick",
        "--function",
        "-x^2",
        "--domain",
        "-1,1",
        "--samples",
        "101",
        "--duration",
        "2",
    ]);
    assert_eq!(json.code, 0, "{}", json.transcript());
    let value = json.json();
    assert_eq!(value["function"]["expr"], "-x^2");
    let mut kinds: Vec<&str> = value["function"]["critical_points"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["kind"].as_str())
        .collect();
    kinds.sort_unstable();
    assert_eq!(kinds, ["local maximum", "zero"], "{value}");
    assert!(value["summary"]["engines"]["curvasona"].is_string(), "{value}");
    assert_eq!(value["played"]["finished"], true);
    let bad = run(&["--plain", "quick", "--function", "2x", "--domain", "0,1"]);
    assert_eq!(bad.code, 1, "{}", bad.transcript());
    assert!(bad.stderr.contains("SNR-ENG-400"), "{}", bad.stderr);
    let both = run(&["quick", "temps.csv", "--function", "x", "--domain", "0,1"]);
    assert_eq!(both.code, 2, "{}", both.transcript());
    let no_domain = run(&["quick", "--function", "x"]);
    assert_eq!(no_domain.code, 2, "{}", no_domain.transcript());
}

/// C03-T07: `inspect` on a function spec lists its critical points, the off-screen spans
/// of its pitch window included, and on an audification prints the calculator's readout
/// (SDR-31) as a result line, not only as an Info diagnostic.
#[test]
fn inspect_lists_a_function_s_critical_points_and_the_audification_readout() {
    let inspect = run(&["--plain", "inspect", "function.spec.json"]);
    assert_eq!(inspect.code, 0, "{}", inspect.transcript());
    assert_linear(&inspect);
    snapshot!("inspect_function", inspect);
    let json = run(&["--json", "inspect", "function.spec.json"]);
    let value = json.json();
    let points = value["function"]["critical_points"].as_array().unwrap();
    assert!(
        points.iter().any(|p| p["kind"] == "above the window"),
        "{value}"
    );
    let audify = run(&["--plain", "inspect", "audify.spec.json"]);
    assert_eq!(audify.code, 0, "{}", audify.transcript());
    assert!(
        audify
            .stdout
            .lines()
            .any(|l| l.starts_with("Audification: k = 100,")),
        "{}",
        audify.stdout
    );
    assert!(audify.stdout.contains("1 audified series"), "{}", audify.stdout);
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
            "--backend",
            "null",
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
            .join("../../schemas/sonarithm-spec-0.4.schema.json"),
    )
    .unwrap();
    assert_eq!(schema.stdout.trim_end(), published.trim_end());
}

#[test]
fn spec_migrate_reports_what_it_did() {
    let current = run(&[
        "--plain",
        "spec",
        "migrate",
        "--to",
        "0.1",
        "temps.spec.json",
    ]);
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
    let json = run(&[
        "--json",
        "spec",
        "migrate",
        "--to",
        "0.1",
        "--in-place",
        "temps.spec.json",
    ]);
    assert_eq!(json.code, 0);
    assert_eq!(json.json()["changed"], false);
    assert_eq!(json.json()["written"], false);
    // RFC-0004 §9: to the current version, only `specVersion` changes.
    let migrated = run(&["--plain", "spec", "migrate", "temps.spec.json"]);
    assert_eq!(migrated.code, 0);
    let mut expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixtures().join("temps.spec.json")).unwrap())
            .unwrap();
    expected["specVersion"] = serde_json::json!("0.4");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&migrated.stdout).unwrap(),
        expected
    );
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

fn render_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cli-render")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn render_writes_audio_manifest_and_cues() {
    let dir = render_dir("plain");
    let wav = dir.join("temps.wav");
    let rendered = run(&[
        "--plain",
        "render",
        "temps.spec.json",
        "-o",
        wav.to_str().unwrap(),
    ]);
    assert_eq!(rendered.code, 0, "{}", rendered.transcript());
    assert_linear(&rendered);
    for file in ["temps.wav", "temps.render.json", "temps.cues.vtt"] {
        assert!(dir.join(file).is_file(), "{file}");
    }
    assert!(!rendered.stdout.contains("cli-render"), "file names only");
    snapshot!("render_plain", rendered);
    let manifest = std::fs::read_to_string(dir.join("temps.render.json")).unwrap();
    assert!(
        !manifest.contains("cli-render") && !manifest.contains("fixtures"),
        "no user paths"
    );
}

#[test]
fn render_json_carries_the_manifest_and_the_bytes_match_the_facade() {
    let dir = render_dir("json");
    let flac = dir.join("temps.flac");
    let json = run(&[
        "--json",
        "render",
        "temps.spec.json",
        "-o",
        flac.to_str().unwrap(),
    ]);
    assert_eq!(json.code, 0, "{}", json.transcript());
    assert!(json.stderr.is_empty(), "{}", json.stderr);
    let value = json.json();
    assert_eq!(value["ok"], true);
    assert_eq!(value["rendered"]["audio"], "temps.flac");
    assert_eq!(value["rendered"]["manifest_file"], "temps.render.json");
    assert_eq!(value["rendered"]["manifest"]["output"]["format"], "flac");

    let wav = dir.join("t.wav");
    assert_eq!(
        run(&["render", "temps.spec.json", "-o", wav.to_str().unwrap()]).code,
        0
    );
    let spec = sonarithm::Spec::from_file(fixtures().join("temps.spec.json")).unwrap();
    let snr = sonarithm::Sonarithm::new();
    let compiled = snr.compile(&spec).unwrap();
    let (bytes, _) = snr
        .render_wav(&compiled, &sonarithm::RenderOptions::default())
        .unwrap();
    assert_eq!(
        std::fs::read(&wav).unwrap(),
        bytes,
        "the CLI writes the facade's bytes"
    );
}

#[test]
fn render_options_reach_the_manifest() {
    let dir = render_dir("options");
    std::fs::copy(fixtures().join("temps.csv"), dir.join("other.csv")).unwrap();
    let wav = dir.join("t.wav");
    let other = dir.join("other.csv");
    let json = run(&[
        "--json",
        "render",
        "temps.spec.json",
        "-o",
        wav.to_str().unwrap(),
        "--seed",
        "7",
        "--sample-rate",
        "44100",
        "--data",
        other.to_str().unwrap(),
        "--no-cues",
        "--preserve-level",
    ]);
    assert_eq!(json.code, 0, "{}", json.transcript());
    let manifest = &json.json()["rendered"]["manifest"];
    assert_eq!(manifest["render"]["seed"], 7);
    assert_eq!(manifest["render"]["sampleRate"], 44_100);
    assert_eq!(manifest["data"]["source"]["name"], "other.csv");
    assert_eq!(manifest["output"]["gainDb"], 0.0, "the level is kept");
    assert!(manifest["cues"].is_null());
    assert!(!dir.join("t.cues.vtt").exists());
}

#[test]
fn render_with_denied_warnings_writes_nothing() {
    let dir = render_dir("denied");
    let wav = dir.join("t.wav");
    let denied = run(&[
        "--plain",
        "render",
        "--deny-warnings",
        "temps.spec.json",
        "-o",
        wav.to_str().unwrap(),
    ]);
    assert_eq!(denied.code, 3, "{}", denied.transcript());
    assert!(
        denied.stdout.contains("nothing was written"),
        "{}",
        denied.stdout
    );
    assert!(!wav.exists());
    let mp3 = run(&[
        "--plain",
        "render",
        "temps.spec.json",
        "-o",
        dir.join("t.mp3").to_str().unwrap(),
    ]);
    assert_eq!(mp3.code, 1);
    assert!(mp3.stderr.contains(".wav or .flac"), "{}", mp3.stderr);
    let invalid = run(&[
        "--plain",
        "render",
        "invalid.spec.json",
        "-o",
        wav.to_str().unwrap(),
    ]);
    assert_eq!(invalid.code, 1);
}

#[test]
fn quick_renders_with_output() {
    let dir = render_dir("quick");
    let flac = dir.join("anomaly.flac");
    let quick = run(&[
        "--plain",
        "quick",
        "temps.csv",
        "-o",
        flac.to_str().unwrap(),
    ]);
    assert_eq!(quick.code, 0, "{}", quick.transcript());
    assert!(
        quick.stdout.contains("Wrote anomaly.flac"),
        "{}",
        quick.stdout
    );
    assert!(dir.join("anomaly.render.json").is_file());
    let json = run(&["--json", "quick", "temps.csv", "-o", flac.to_str().unwrap()]);
    assert_eq!(json.json()["rendered"]["audio"], "anomaly.flac");
}

#[test]
fn play_announces_each_change_as_a_line() {
    let played = run(&["--plain", "play", "temps.spec.json", "--backend", "null"]);
    assert_eq!(played.code, 0, "{}", played.transcript());
    assert_linear(&played);
    snapshot!("play_null", played);
    let json = run(&[
        "--json",
        "play",
        "estimated.spec.json",
        "--backend",
        "null",
        "--speed",
        "4",
    ]);
    assert_eq!(json.code, 0, "{}", json.transcript());
    let value = json.json();
    assert_eq!(value["played"]["finished"], true);
    assert_eq!(value["played"]["device"], "null");
    assert_eq!(value["played"]["events"][1], "Finished at 0.92 seconds.");
    assert!(json.stderr.is_empty(), "{}", json.stderr);
}

#[test]
fn play_refuses_what_it_cannot_play() {
    let denied = run(&[
        "--plain",
        "play",
        "--deny-warnings",
        "temps.spec.json",
        "--backend",
        "null",
    ]);
    assert_eq!(denied.code, 3, "{}", denied.transcript());
    assert!(
        denied.stdout.contains("nothing was played"),
        "{}",
        denied.stdout
    );
    let device = run(&[
        "--plain",
        "play",
        "temps.spec.json",
        "--backend",
        "null",
        "--device",
        "Nowhere",
    ]);
    assert_eq!(device.code, 1, "{}", device.transcript());
    assert!(
        device.stderr.contains("`sonarithm devices` lists them"),
        "{}",
        device.stderr
    );
    assert_eq!(run(&["play", "temps.spec.json", "--speed", "9"]).code, 2);
    assert_eq!(
        run(&["--plain", "play", "invalid.spec.json", "--backend", "null"]).code,
        1
    );
}

#[test]
fn devices_lists_the_outputs() {
    let plain = run(&["--plain", "devices", "--backend", "null"]);
    assert_eq!(plain.code, 0, "{}", plain.transcript());
    assert_eq!(
        plain.stdout,
        "1 output device:\nnull: default, 48000 Hz, 2 channels.\n"
    );
    let json = run(&["--json", "devices", "--backend", "null"]);
    assert_eq!(json.json()["devices"][0]["is_default"], true);
}
