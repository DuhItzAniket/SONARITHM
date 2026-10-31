// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chart corrections through the IPC protocol (C02-T06; RFC-0005 §9–§12): every correction
//! is undoable, export and import round-trip, import renumbers and rewrites references,
//! history survives a save and reopen, and no path is ever written.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::path::{Path, PathBuf};
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
    fn new(label: &str) -> Self {
        let dir = TempDir::new(label);
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let app = App::new(config, Arc::new(CollectingSink::new())).unwrap();
        Self { app, dir }
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        match self.send(request) {
            Ok(reply) => reply,
            Err(err) => panic!("{request} failed: {err}"),
        }
    }

    /// Opens the corpus Vega-Lite line chart (north, south, east; Jan…Dec) as `d1`.
    fn open_chart(&mut self) -> PathBuf {
        let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../engines/sonarithm-plotsona/tests/corpus/chartjson/vl-line.json");
        let chart = self.dir.path().join("rain.json");
        std::fs::copy(corpus, &chart).unwrap();
        let opened = self.ok(&json!({ "command": "dataset_open", "args": { "path": chart } }));
        assert_eq!(opened["id"], "d1", "{opened}");
        chart
    }

    fn apply(&mut self, edit: &Value) -> Value {
        self.ok(&json!({ "command": "corrections_apply", "args": {
            "dataset": "d1", "label": "Correct", "patch": [{ "op": "add", "path": "/edits/-", "value": edit }]
        }}))
    }

    fn corrections(&mut self) -> Value {
        self.ok(&json!({ "command": "corrections_get", "args": { "dataset": "d1" } }))
    }
}

fn edits(state: &Value) -> Vec<Value> {
    let file: Value = serde_json::from_str(state["text"].as_str().unwrap()).unwrap();
    file["edits"].as_array().unwrap().clone()
}

fn move_north_jan(id: &str, to: f64) -> Value {
    json!({ "op": "move", "id": id, "series": "north", "point": 0, "was": { "x": "Jan", "y": 99.2 }, "to": { "y": to },
            "before": { "class": "exact" } })
}

/// RFC-0005 §9, FR-111: each correction is one undoable edit; undo puts the dataset back.
#[test]
fn every_correction_is_undoable() {
    let mut h = Harness::new("corr-undo");
    h.open_chart();
    let empty = h.corrections();
    assert_eq!(empty["edits"], 0);
    assert_eq!(empty["editable"], true);
    let rows = empty["summary"]["rows"].clone();
    let moved = h.apply(&move_north_jan("e1", 100.0));
    assert_eq!(moved["edits"], 1);
    assert!(moved["undoToken"].as_u64().is_some());
    assert_eq!(moved["notApplied"], json!([]));
    let deleted = h.apply(
        &json!({ "op": "delete", "id": "e2", "series": "south", "point": 1,
        "was": { "x": "Feb" }, "before": { "class": "exact" } }),
    );
    assert_eq!(
        deleted["summary"]["rows"].as_u64(),
        rows.as_u64().map(|r| r - 1)
    );
    let undone = h.ok(&json!({ "command": "history_undo" }));
    assert_eq!(undone["corrections"]["edits"], 1, "{undone}");
    assert_eq!(undone["corrections"]["summary"]["rows"], rows);
    let undone = h.ok(&json!({ "command": "history_undo" }));
    assert_eq!(undone["corrections"]["edits"], 0);
    assert_eq!(undone["corrections"]["text"], empty["text"]);
    let redone = h.ok(&json!({ "command": "history_redo" }));
    assert_eq!(redone["corrections"]["edits"], 1);
    // A patch that breaks the file changes nothing.
    let err = h
        .send(&json!({ "command": "corrections_apply", "args": { "dataset": "d1",
            "patch": [{ "op": "add", "path": "/edits/-", "value": { "op": "move", "id": "e1" } }] } }))
        .unwrap_err();
    // The reader says exactly what is wrong (RFC-0005 §8).
    assert_eq!(err.code, "SNR-SPEC-091");
    assert_eq!(h.corrections()["edits"], 1);
}

/// RFC-0005 §11, §12: export then import into an empty session is the identity; the file
/// holds no path.
#[test]
fn export_then_import_round_trips_without_paths() {
    let mut h = Harness::new("corr-export");
    h.open_chart();
    h.apply(&move_north_jan("e1", 100.0));
    h.apply(&json!({ "op": "add", "id": "e2", "series": "east", "to": { "x": "Jan", "y": 50.0 }, "note": "missed" }));
    h.apply(&json!({ "op": "move", "id": "e3", "series": "east", "point": "+e2", "was": { "y": 50.0 }, "to": { "y": 51.0 },
        "before": { "class": "user_edited" } }));
    let file = h.dir.path().join("rain.corrections.json");
    let exported = h
        .ok(&json!({ "command": "corrections_export", "args": { "dataset": "d1", "path": file } }));
    let text = std::fs::read_to_string(&file).unwrap();
    assert_eq!(text, exported["text"].as_str().unwrap());
    let folder = h.dir.path().to_string_lossy().into_owned();
    assert!(!text.contains(&folder) && !text.contains('\\'), "{text}");
    let file_json: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(file_json["chart"]["name"], "rain.json");
    // Without notes.
    let bare = h.dir.path().join("bare.json");
    h.ok(&json!({ "command": "corrections_export", "args": { "dataset": "d1", "path": bare, "notes": false } }));
    assert!(!std::fs::read_to_string(&bare).unwrap().contains("missed"));
    // A fresh session takes the file back byte for byte.
    let mut fresh = Harness::new("corr-import");
    fresh.open_chart();
    let preview = fresh
        .ok(&json!({ "command": "corrections_import", "args": { "dataset": "d1", "path": file } }));
    assert_eq!(preview["applies"], json!(["e1", "e2", "e3"]));
    assert_eq!(preview["applied"], false);
    assert_eq!(fresh.corrections()["edits"], 0, "a preview changes nothing");
    let done = fresh.ok(&json!({ "command": "corrections_import", "args": { "dataset": "d1", "path": file, "confirm": true } }));
    assert_eq!(done["state"]["text"].as_str().unwrap(), text);
    // One undo takes the whole import back.
    let undone = fresh.ok(&json!({ "command": "history_undo" }));
    assert_eq!(undone["corrections"]["edits"], 0);
}

/// RFC-0005 §11 (amendments 1 and 2): imported edits are renumbered after the session's,
/// references to added points follow, duplicates are skipped and failed guards are
/// conflicts.
#[test]
fn import_renumbers_and_reports_duplicates_and_conflicts() {
    let mut a = Harness::new("corr-a");
    a.open_chart();
    a.apply(&move_north_jan("e1", 100.0));
    a.apply(&json!({ "op": "add", "id": "e2", "series": "east", "to": { "x": "Jan", "y": 50.0 } }));
    a.apply(&json!({ "op": "move", "id": "e3", "series": "east", "point": "+e2", "was": { "y": 50.0 }, "to": { "y": 51.0 },
        "before": { "class": "user_edited" } }));
    let file = a.dir.path().join("a.json");
    a.ok(&json!({ "command": "corrections_export", "args": { "dataset": "d1", "path": file } }));
    let mut b = Harness::new("corr-b");
    b.open_chart();
    // The same move (a duplicate), and another move of the same point (so a's e1 conflicts
    // with nothing: it is a duplicate), plus a delete that makes nothing of a's.
    b.apply(&move_north_jan("e1", 100.0));
    b.apply(
        &json!({ "op": "delete", "id": "e2", "series": "south", "point": 0, "was": { "x": "Jan" },
        "before": { "class": "exact" } }),
    );
    let preview = b.ok(&json!({ "command": "corrections_import", "args": { "dataset": "d1", "path": file, "confirm": true } }));
    assert_eq!(preview["duplicates"], json!(["e1"]));
    assert_eq!(preview["applies"], json!(["e2", "e3"]));
    assert_eq!(preview["conflicts"], json!([]));
    let after = edits(&preview["state"]);
    let ids: Vec<&str> = after.iter().map(|e| e["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["e1", "e2", "e3", "e4"]);
    assert_eq!(
        after[3]["point"], "+e3",
        "the reference follows the renumbered add"
    );
    assert_eq!(preview["state"]["notApplied"], json!([]));
    // A file whose move no longer matches the session conflicts and is not applied.
    let mut c = Harness::new("corr-c");
    c.open_chart();
    c.apply(&move_north_jan("e1", 120.0));
    let conflicting = c.dir.path().join("c.json");
    let mut other: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    other["edits"] = json!([move_north_jan("e1", 101.0)]);
    std::fs::write(&conflicting, other.to_string()).unwrap();
    let preview = c.ok(&json!({ "command": "corrections_import", "args": { "dataset": "d1", "path": conflicting, "confirm": true } }));
    assert_eq!(preview["conflicts"], json!(["e1"]));
    assert_eq!(preview["applies"], json!([]));
}

/// RFC-0005 §3: a file made on another chart imports nothing.
#[test]
fn a_file_for_another_chart_is_refused() {
    let mut h = Harness::new("corr-other");
    h.open_chart();
    let file = h.dir.path().join("other.json");
    let mut text: Value = serde_json::from_str(h.corrections()["text"].as_str().unwrap()).unwrap();
    text["chart"]["sha256"] = json!("0".repeat(64));
    std::fs::write(&file, text.to_string()).unwrap();
    let err = h
        .send(
            &json!({ "command": "corrections_import", "args": { "dataset": "d1", "path": file } }),
        )
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-086");
}

/// RFC-0005 §7, C02-T06: the corrections and each edit's history survive a save and a
/// reopen, in `annotations/corrections-d1.json`; they are read-only until the chart is
/// opened again, and still export.
#[test]
fn corrections_survive_a_save_and_reopen() {
    let mut h = Harness::new("corr-project");
    h.open_chart();
    h.ok(&json!({ "command": "spec_set", "args": { "text": json!({
        "specVersion": "0.3", "data": { "source": { "kind": "dataset", "id": "d1" } }, "template": { "id": "chart-line" }
    }).to_string() } }));
    let moved = h.apply(&move_north_jan("e1", 100.0));
    let project = h.dir.path().join("rain.sonarithm");
    h.ok(&json!({ "command": "project_save_as", "args": { "path": project, "embedData": true } }));
    let mut again = Harness::new("corr-reopen");
    again.ok(&json!({ "command": "project_open", "args": { "path": project } }));
    let kept = again.corrections();
    assert_eq!(kept["text"], moved["text"]);
    assert_eq!(kept["editable"], false);
    assert_eq!(edits(&kept)[0]["before"]["class"], "exact");
    let err = again
        .send(
            &json!({ "command": "corrections_apply", "args": { "dataset": "d1",
            "patch": [{ "op": "remove", "path": "/edits/0" }] } }),
        )
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-085");
    let out = again.dir.path().join("kept.json");
    again.ok(&json!({ "command": "corrections_export", "args": { "dataset": "d1", "path": out } }));
    assert_eq!(
        std::fs::read_to_string(out).unwrap(),
        moved["text"].as_str().unwrap()
    );
}

/// Only a dataset opened from a chart has corrections.
#[test]
fn data_files_have_no_corrections() {
    let mut h = Harness::new("corr-data");
    let csv = h.dir.path().join("t.csv");
    std::fs::write(&csv, "day,temp\n1,8.5\n2,9.25\n").unwrap();
    h.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
    let err = h
        .send(&json!({ "command": "corrections_get", "args": { "dataset": "d1" } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-086");
}
