// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Functions in the app core (C03-T08; ADR-0017 "Functions"): `function_check` for live
//! diagnostics, `function_open` as a dataset the cursor and the cues work on, the SDR-31
//! calculator, and projects that refuse to save a function rather than drop it.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::sync::Arc;

use serde_json::{Value, json};
use sonarithm_app::ipc::IpcError;
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

struct Harness {
    app: App,
    dir: TempDir,
}

impl Harness {
    fn new(name: &str) -> Self {
        let dir = TempDir::new(name);
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let app = App::new(config, Arc::new(CollectingSink::new())).unwrap();
        Self { app, dir }
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.send(request)
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }
}

fn cubic() -> Value {
    json!({ "command": "function_open", "args": { "expr": "x^3 - 3*x", "domain": [-3, 3] } })
}

#[test]
fn function_check_reads_an_expression_without_opening_anything() {
    let mut h = Harness::new("function-check");
    let good = h.ok(&json!({ "command": "function_check", "args": { "expr": "sin(x) / x" } }));
    assert_eq!(good["reply"], "function_check");
    assert_eq!(good["ok"], true);
    assert_eq!(good["canonical"], "sin(x)/x");
    assert_eq!(good["diagnostics"], json!([]));
    let bad = h.ok(&json!({ "command": "function_check", "args": { "expr": "2x + 1" } }));
    assert_eq!(bad["ok"], false);
    assert_eq!(bad["canonical"], Value::Null);
    assert_eq!(bad["diagnostics"][0]["code"], "SNR-ENG-400");
    assert!(bad["span"].is_array(), "{bad}");
    let variables = h.ok(&json!({ "command": "function_check",
        "args": { "expr": "a*x", "variables": { "a": 2 } } }));
    assert_eq!(variables["canonical"], "a*x");
    let datasets = h.ok(&json!({ "command": "dataset_list" }));
    assert_eq!(datasets["datasets"], json!([]), "nothing was opened");
}

#[test]
fn input_too_large_to_read_is_refused_with_its_code() {
    let mut h = Harness::new("function-too-large");
    let long = "x+".repeat(3000) + "x";
    let err = h
        .send(&json!({ "command": "function_check", "args": { "expr": long } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-088", "{err}");
    let variables: serde_json::Map<String, Value> =
        (0..65).map(|i| (format!("v{i}"), json!(1.0))).collect();
    let err = h
        .send(&json!({ "command": "function_open",
            "args": { "expr": "x", "domain": [0, 1], "variables": variables } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-088", "{err}");
}

/// The function opens as a dataset whose critical points name their rows; the cursor goes
/// to a point's row; a spec reading the dataset plays the cues (A11Y-10, A11Y-24).
#[test]
fn a_function_opens_as_a_dataset_the_cursor_and_the_cues_use() {
    let mut h = Harness::new("function-open");
    let opened = h.ok(&cubic());
    assert_eq!(opened["reply"], "function");
    let dataset = &opened["dataset"];
    assert_eq!(dataset["id"], "d1");
    assert_eq!(dataset["name"], "f(x) = x^3 - 3*x");
    let fields: Vec<&str> = dataset["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert_eq!(&fields[..3], ["x", "y", "dy"]);
    let function = &opened["function"];
    assert_eq!(function["canonical"], "x^3 - 3*x");
    let points = function["criticalPoints"].as_array().unwrap();
    let kinds: Vec<&str> = points.iter().filter_map(|p| p["kind"].as_str()).collect();
    assert!(kinds.contains(&"local maximum") && kinds.contains(&"zero"), "{function}");
    let maximum = points.iter().find(|p| p["kind"] == "local maximum").unwrap();
    assert_eq!(maximum["text"], "local maximum at x = -1, y = 2");
    let row = maximum["row"].as_u64().unwrap();
    let cursor = h.ok(&json!({ "command": "cursor_set",
        "args": { "dataset": "d1", "row": row, "origin": "nav" } }));
    assert_eq!(cursor["row"], row);
    let spec = json!({
        "specVersion": "0.4",
        "data": { "source": { "kind": "dataset", "id": "d1" } },
        "substrate": { "time": { "pace": { "duration": { "s": 8 } } } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" } } }],
        "analysis": { "detect": ["critical-points"], "cue": "earcon" }
    });
    h.ok(&json!({ "command": "spec_set", "args": { "text": spec.to_string() } }));
    let report = h.ok(&json!({ "command": "compile" }));
    assert_eq!(report["ok"], true, "{report}");
    let cues = report["score"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "curvasona.cues")
        .unwrap_or_else(|| panic!("{report}"));
    assert_eq!(cues["events"], 5, "three zeros, two extrema; the inflection merges");
}

#[test]
fn a_function_that_cannot_be_sampled_is_an_error_with_its_code() {
    let mut h = Harness::new("function-bad");
    let err = h
        .send(&json!({ "command": "function_open", "args": { "expr": "2x", "domain": [0, 1] } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-ENG-400", "{err}");
    let err = h
        .send(&json!({ "command": "function_open",
            "args": { "expr": "x", "domain": [0, 1], "samples": 2_000_000 } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-ENG-404", "{err}");
}

/// SDR-31: the calculator gives the numbers a compile reports, on the same grid.
#[test]
fn sdr_31_the_calculator_agrees_with_the_compile() {
    let mut h = Harness::new("function-audify");
    h.ok(&json!({ "command": "function_open",
        "args": { "expr": "sin(x)", "domain": [0, 100], "samples": 2000 } }));
    let readout = h.ok(&json!({ "command": "audify_calculate",
        "args": { "dataset": "d1", "k": 100 } }));
    assert_eq!(readout["reply"], "audify_readout");
    assert_eq!(readout["k"], 100.0);
    assert_eq!(readout["series"], "y");
    assert_eq!(readout["codes"], json!(["SNR-ENG-461"]), "{readout}");
    let spec = json!({
        "specVersion": "0.4",
        "data": { "source": { "kind": "dataset", "id": "d1" } },
        "tracks": [{ "id": "a", "mark": "audify", "audify": { "k": 100 } }]
    });
    h.ok(&json!({ "command": "spec_set", "args": { "text": spec.to_string() } }));
    let report = h.ok(&json!({ "command": "compile" }));
    let message = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "SNR-ENG-460")
        .and_then(|d| d["message"].as_str())
        .unwrap_or_else(|| panic!("{report}"))
        .to_owned();
    let duration = readout["durationS"].as_f64().unwrap();
    assert!(
        message.contains(&format!("{duration:.3} s long")),
        "{message} / {readout}"
    );
    let default = h.ok(&json!({ "command": "audify_calculate", "args": { "dataset": "d1" } }));
    let top = default["bandHz"][1].as_f64().unwrap();
    assert!((top - 4000.0).abs() < 1e-6, "SDR-44's default top: {default}");
    let err = h
        .send(&json!({ "command": "audify_calculate", "args": { "dataset": "d1", "k": 0 } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-ENG-463", "{err}");
}

/// Projects cannot store a function yet (a project-format amendment will add it): saving one
/// fails with a coded error naming the dataset, and nothing is written or dropped.
#[test]
fn saving_a_project_with_a_function_fails_and_names_it() {
    let mut h = Harness::new("function-save");
    h.ok(&cubic());
    let spec = json!({
        "specVersion": "0.4",
        "data": { "source": { "kind": "dataset", "id": "d1" } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" } } }]
    });
    h.ok(&json!({ "command": "spec_set", "args": { "text": spec.to_string() } }));
    let path = h.dir.path().join("cubic.sonarithm");
    let err = h
        .send(&json!({ "command": "project_save_as", "args": { "path": path, "embedData": true } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-089", "{err}");
    assert!(err.message.contains("d1"), "{err}");
    assert!(err.message.contains("f(x) = x^3 - 3*x"), "{err}");
    assert!(!path.exists(), "nothing was written");
    h.ok(&json!({ "command": "dataset_close", "args": { "id": "d1" } }));
    h.ok(&json!({ "command": "project_save_as", "args": { "path": path, "embedData": true } }));
    assert!(path.exists());
}
