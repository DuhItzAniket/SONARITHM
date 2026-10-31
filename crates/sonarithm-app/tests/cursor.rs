// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The one cursor (B04-T06; A11Y-10, A11Y-11, ADR-0017 "Cursor"): every move goes out as a
//! `cursor` event; a move from the table is not spoken, a plot move is; the cursor follows
//! the data under it when a dataset closes, is replaced, corrected or undone.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sonarithm_app::ipc::{CursorState, Event, IpcError};
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n";

struct Harness {
    app: App,
    sink: CollectingSink,
    dir: TempDir,
}

impl Harness {
    fn new(label: &str) -> Self {
        let dir = TempDir::new(label);
        let sink = CollectingSink::new();
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let app = App::new(config, Arc::new(sink.clone())).unwrap();
        Self { app, sink, dir }
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.send(request)
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }

    fn open_csv(&mut self) {
        let csv = self.dir.path().join("temps.csv");
        std::fs::write(&csv, CSV).unwrap();
        self.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
    }

    fn compile(&mut self) {
        let spec = json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "dataset", "id": "d1" } },
            "substrate": { "time": { "field": "day" } },
            "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
        })
        .to_string();
        self.ok(&json!({ "command": "spec_set", "args": { "text": spec } }));
        let report = self.ok(&json!({ "command": "compile" }));
        assert_eq!(report["ok"], true, "{report}");
    }

    fn set(&mut self, row: u64, origin: &str) -> Value {
        self.ok(&json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": row, "origin": origin } }))
    }

    /// The `cursor` events so far.
    fn cursors(&self) -> Vec<CursorState> {
        self.sink
            .events()
            .into_iter()
            .filter_map(|event| match event {
                Event::Cursor(state) => Some(state),
                _ => None,
            })
            .collect()
    }

    fn last_cursor(&self) -> CursorState {
        self.cursors()
            .pop()
            .unwrap_or_else(|| panic!("no cursor event"))
    }

    fn announcements(&self) -> usize {
        self.sink
            .events()
            .iter()
            .filter(|event| matches!(event, Event::Announcement(_)))
            .count()
    }
}

#[test]
fn b04_t06_a_table_move_sends_the_cursor_and_is_not_spoken() {
    let mut h = Harness::new("cursor-table");
    h.open_csv();
    let reply = h.set(1, "table");
    assert_eq!(reply["reply"], "cursor");
    assert_eq!(reply["dataset"], "d1");
    assert_eq!(reply["row"], 1);
    assert_eq!(reply["origin"], "table");
    // No Score yet: the path is the dataset alone, and there is no point node.
    assert_eq!(
        reply["path"],
        json!([{ "level": "dataset", "label": "temps.csv" }])
    );
    assert_eq!(reply["node"], Value::Null);
    let event = h.last_cursor();
    assert_eq!(serde_json::to_value(&event).unwrap()["seq"], reply["seq"]);
    assert_eq!(event.row, Some(1));
    let got = h.ok(&json!({ "command": "cursor_get" }));
    assert_eq!(got["row"], 1);
    assert_eq!(got["seq"], reply["seq"]);
    // The screen reader reads the focused row; SONARITHM says nothing more (OD-30).
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(h.announcements(), 0);
}

#[test]
fn b04_t06_a_plot_move_on_a_point_has_its_path_and_is_announced() {
    let mut h = Harness::new("cursor-plot");
    h.open_csv();
    h.set(2, "table");
    // A compile makes the row a point: the cursor event follows, with no origin.
    h.compile();
    let after_compile = h.last_cursor();
    assert!(after_compile.node.is_some(), "{after_compile:?}");
    assert_eq!(after_compile.origin, None);
    assert_eq!(after_compile.x, Some(3.0));
    let path = serde_json::to_value(&after_compile.path).unwrap();
    let levels: Vec<&str> = path
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["level"].as_str().unwrap())
        .collect();
    assert_eq!(levels.first(), Some(&"dataset"));
    assert_eq!(levels.last(), Some(&"point"));
    assert!(levels.contains(&"series"), "{levels:?}");
    assert_eq!(path.as_array().unwrap().last().unwrap()["label"], "3");

    let before = h.announcements();
    let reply = h.set(0, "plot");
    assert_eq!(reply["x"], 1.0);
    let start = Instant::now();
    while h.announcements() == before {
        assert!(start.elapsed() < Duration::from_secs(5), "no announcement");
        std::thread::sleep(Duration::from_millis(10));
    }
    // A table move on a point stays silent.
    let spoken = h.announcements();
    h.set(1, "table");
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(h.announcements(), spoken);
}

#[test]
fn b04_t06_a_row_outside_the_dataset_is_refused() {
    let mut h = Harness::new("cursor-range");
    h.open_csv();
    let err = h
        .send(&json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": 3, "origin": "table" } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-087");
    let err = h
        .send(&json!({ "command": "cursor_set", "args": { "dataset": "d9", "row": 0, "origin": "plot" } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-002");
    assert_eq!(h.cursors(), Vec::new());
}

#[test]
fn b04_t06_closing_or_replacing_the_dataset_clears_the_cursor() {
    let mut h = Harness::new("cursor-clear");
    h.open_csv();
    h.set(2, "table");
    h.ok(&json!({ "command": "dataset_close", "args": { "id": "d1" } }));
    let cleared = h.last_cursor();
    assert_eq!((cleared.dataset, cleared.row), (None, None));
    assert_eq!(cleared.origin, None);

    // A new project replaces every dataset, even one with the same ID.
    h.open_csv();
    let opened = h.ok(&json!({ "command": "dataset_list" }));
    let id = opened["datasets"][0]["id"].as_str().unwrap().to_owned();
    h.ok(
        &json!({ "command": "cursor_set", "args": { "dataset": id, "row": 1, "origin": "table" } }),
    );
    h.ok(&json!({ "command": "project_new" }));
    let cleared = h.last_cursor();
    assert_eq!((cleared.dataset, cleared.row), (None, None));
}

#[test]
fn b04_t06_corrections_and_undo_clamp_the_cursor() {
    let mut h = Harness::new("cursor-clamp");
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engines/sonarithm-plotsona/tests/corpus/chartjson/vl-line.json");
    let chart = h.dir.path().join("rain.json");
    std::fs::copy(corpus, &chart).unwrap();
    h.ok(&json!({ "command": "dataset_open", "args": { "path": chart } }));
    let apply = |h: &mut Harness, edit: Value| {
        h.ok(&json!({ "command": "corrections_apply", "args": {
            "dataset": "d1", "label": "Correct", "patch": [{ "op": "add", "path": "/edits/-", "value": edit }]
        }}))
    };
    let rows = h.ok(&json!({ "command": "corrections_get", "args": { "dataset": "d1" } }))["summary"]["rows"]
        .as_u64()
        .unwrap();

    // Corrected: a deleted point leaves one row fewer, so the cursor on the last row clamps.
    h.set(rows - 1, "table");
    apply(
        &mut h,
        json!({ "op": "delete", "id": "e1", "series": "south", "point": 1,
                "was": { "x": "Feb" }, "before": { "class": "exact" } }),
    );
    let clamped = h.last_cursor();
    assert_eq!(clamped.row, Some(rows - 2), "{clamped:?}");
    assert_eq!(clamped.origin, None);

    // Undone: an added point is taken away again, so the cursor on it clamps too.
    apply(
        &mut h,
        json!({ "op": "add", "id": "e2", "series": "east", "to": { "x": "Jan", "y": 50.0 } }),
    );
    h.set(rows - 1, "table");
    h.ok(&json!({ "command": "history_undo" }));
    let clamped = h.last_cursor();
    assert_eq!(clamped.row, Some(rows - 2), "{clamped:?}");
}
