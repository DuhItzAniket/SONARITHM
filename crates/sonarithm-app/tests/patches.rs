// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spec patches with undo and redo through the command bus (B05-T01, FR-111).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::sync::Arc;

use proptest::prelude::*;
use serde_json::{Value, json};
use sonarithm_app::ipc::IpcError;
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, NullSink};
use sonarithm_spec::canonical::canonicalize;

const SPEC: &str = r#"{
  "specVersion": "0.1",
  "name": "Temperatures",
  "data": { "source": { "kind": "inline", "values": [ { "day": 1, "temp": 8.5 }, { "day": 2, "temp": 9.25 } ] } },
  "substrate": { "time": { "field": "day" } },
  "tracks": [ { "id": "temps", "mark": "tone", "channels": { "pitch": { "field": "temp" } } } ]
}"#;

fn app(dir: &TempDir) -> App {
    let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
    let mut app = App::new(config, Arc::new(NullSink)).unwrap();
    send(
        &mut app,
        &json!({ "command": "spec_set", "args": { "text": SPEC } }),
    )
    .unwrap();
    app
}

fn send(app: &mut App, request: &Value) -> Result<Value, IpcError> {
    app.dispatch_json(&request.to_string())
}

fn spec(app: &mut App) -> String {
    send(app, &json!({ "command": "spec_get" })).unwrap()["text"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn apply(app: &mut App, ops: &Value) -> Result<Value, IpcError> {
    send(
        app,
        &json!({ "command": "spec_apply", "args": { "patch": ops, "label": null } }),
    )
}

#[test]
fn every_patch_kind_is_undoable_and_redoable() {
    let dir = TempDir::new("kinds");
    for ops in [
        json!([{ "op": "add", "path": "/tracks/0/channels/pitch/range", "value": { "hz": [300, 900] } }]),
        json!([{ "op": "remove", "path": "/substrate" }]),
        json!([{ "op": "replace", "path": "/name", "value": "Daily temperatures" }]),
        json!([{ "op": "move", "from": "/name", "path": "/description" }]),
        json!([{ "op": "copy", "from": "/tracks/0", "path": "/tracks/-" }]),
        json!([{ "op": "test", "path": "/specVersion", "value": "0.1" }]),
    ] {
        let mut app = app(&dir);
        let before = spec(&mut app);
        let applied = apply(&mut app, &ops).unwrap();
        assert_eq!(applied["reply"], "spec_patch");
        let after = applied["text"].as_str().unwrap().to_owned();
        let token = applied["undoToken"].clone();
        let undone = send(&mut app, &json!({ "command": "history_undo" })).unwrap();
        assert_eq!(undone["token"], token, "{ops}");
        assert_eq!(spec(&mut app), before, "{ops}");
        send(&mut app, &json!({ "command": "history_redo" })).unwrap();
        assert_eq!(spec(&mut app), after, "{ops}");
    }
}

#[test]
fn a_patch_that_does_not_apply_changes_nothing() {
    let dir = TempDir::new("fail");
    let mut app = app(&dir);
    let before = spec(&mut app);
    let err = apply(
        &mut app,
        &json!([
            { "op": "replace", "path": "/name", "value": "changed" },
            { "op": "remove", "path": "/tracks/3" }
        ]),
    )
    .unwrap_err();
    assert_eq!(err.code, "SNR-APP-040");
    assert!(err.message.contains("operation 1"), "{err}");
    assert_eq!(spec(&mut app), before);
    // Nothing was recorded: the only edit to undo is the initial spec_set.
    let undone = send(&mut app, &json!({ "command": "history_undo" })).unwrap();
    assert_eq!(undone["label"], "Set spec");
}

#[test]
fn every_patch_reports_the_validation_of_its_result() {
    let dir = TempDir::new("validate");
    let mut app = app(&dir);
    let ok = apply(
        &mut app,
        &json!([{ "op": "replace", "path": "/name", "value": "T" }]),
    )
    .unwrap();
    assert_eq!(ok["counts"]["errors"], 0);
    // An unknown mark is an error, but the edit is kept so editing can continue.
    let bad = apply(
        &mut app,
        &json!([{ "op": "replace", "path": "/tracks/0/mark", "value": "kazoo" }]),
    )
    .unwrap();
    assert!(bad["counts"]["errors"].as_u64().unwrap() >= 1);
    let pointer = bad["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["severity"] == "error")
        .and_then(|d| d["pointer"].as_str())
        .unwrap_or_default()
        .to_owned();
    assert!(pointer.starts_with("/tracks/0"), "{pointer}");
    assert!(spec(&mut app).contains("kazoo"));
}

#[test]
fn spec_validate_leaves_the_session_alone() {
    let dir = TempDir::new("check");
    let mut app = app(&dir);
    let before = spec(&mut app);
    let reply = send(
        &mut app,
        &json!({ "command": "spec_validate", "args": { "text": "{ \"specVersion\": \"9\" }" } }),
    )
    .unwrap();
    assert_eq!(reply["reply"], "validation");
    assert!(reply["counts"]["errors"].as_u64().unwrap() >= 1);
    assert_eq!(spec(&mut app), before);
}

/// Candidate operations; some fail on some documents, which must leave no trace.
fn candidates() -> Vec<Value> {
    vec![
        json!({ "op": "replace", "path": "/name", "value": "A" }),
        json!({ "op": "add", "path": "/description", "value": "x" }),
        json!({ "op": "remove", "path": "/description" }),
        json!({ "op": "add", "path": "/tracks/-", "value": { "id": "b", "mark": "tone" } }),
        json!({ "op": "remove", "path": "/tracks/1" }),
        json!({ "op": "move", "from": "/tracks/0", "path": "/tracks/-" }),
        json!({ "op": "copy", "from": "/substrate", "path": "/substrateCopy" }),
        json!({ "op": "remove", "path": "/substrateCopy" }),
        json!({ "op": "test", "path": "/name", "value": "A" }),
        json!({ "op": "add", "path": "/data/source/values/0/temp", "value": -40.0 }),
        json!({ "op": "replace", "path": "/tracks/0/channels", "value": {} }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Apply any sequence of patches, then undo every applied one: the spec is back to its
    /// first text, so its canonical bytes are too.
    #[test]
    fn applying_then_undoing_returns_the_canonical_spec(
        picks in proptest::collection::vec(proptest::collection::vec(0_usize..11, 1..4), 1..12),
    ) {
        let dir = TempDir::new("prop");
        let mut app = app(&dir);
        let first = spec(&mut app);
        let candidates = candidates();
        let mut applied = 0;
        for pick in picks {
            let ops: Vec<Value> = pick.into_iter().map(|i| candidates[i].clone()).collect();
            if apply(&mut app, &Value::Array(ops)).is_ok() {
                applied += 1;
            }
        }
        for _ in 0..applied {
            let undone = send(&mut app, &json!({ "command": "history_undo" })).unwrap();
            prop_assert!(!undone["token"].is_null());
        }
        let last = spec(&mut app);
        prop_assert_eq!(&last, &first);
        prop_assert_eq!(canonicalize(&last).unwrap(), canonicalize(&first).unwrap());
    }
}
