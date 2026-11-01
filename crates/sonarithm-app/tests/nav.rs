// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The chart explorer's verbs on the one cursor (B03-T09; ADR-0017 "Navigation"; A11Y-07,
//! A11Y-08, A11Y-10): `nav_move` moves the cursor of `cursor_set` and its row, sends the
//! `cursor` event with origin `nav` and announces the outcome; `nav_where` announces
//! without moving; a filter takes its series out of playback and view, and the exclusion
//! survives a recompile and ends with the session's data.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sonarithm_app::ipc::{CursorOrigin, CursorState, Event, IpcError};
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

const CSV: &str = "day,temp,rain\n1,8.5,3\n2,9.25,0\n3,7.0,1.5\n";

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
        let mut h = Self { app, sink, dir };
        let csv = h.dir.path().join("weather.csv");
        std::fs::write(&csv, CSV).unwrap();
        h.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
        h
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.send(request)
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }

    /// Compiles one track per field.
    fn compile(&mut self, fields: &[&str]) {
        let tracks: Vec<Value> = fields
            .iter()
            .map(|field| json!({ "id": field, "mark": "tone", "channels": { "pitch": { "field": field } } }))
            .collect();
        let spec = json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "dataset", "id": "d1" } },
            "substrate": { "time": { "field": "day" } },
            "tracks": tracks
        })
        .to_string();
        self.ok(&json!({ "command": "spec_set", "args": { "text": spec } }));
        let report = self.ok(&json!({ "command": "compile" }));
        assert_eq!(report["ok"], true, "{report}");
    }

    fn nav(&mut self, verb: &Value) -> Value {
        self.ok(&json!({ "command": "nav_move", "args": { "verb": verb } }))
    }

    fn step(&mut self, verb: &str) -> Value {
        self.nav(&json!({ "verb": verb }))
    }

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

    fn announcements(&self) -> usize {
        self.sink
            .events()
            .iter()
            .filter(|event| matches!(event, Event::Announcement(_)))
            .count()
    }

    /// Waits for the transport thread to deliver `count` announcements.
    fn wait_for_announcements(&self, count: usize) {
        let start = Instant::now();
        while self.announcements() < count {
            assert!(start.elapsed() < Duration::from_secs(5), "no announcement");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn levels(state: &Value) -> Vec<String> {
    state["path"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["level"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn b03_t09_a_verb_without_a_compiled_score_is_snr_app_090() {
    let mut h = Harness::new("nav-none");
    let err = h
        .send(&json!({ "command": "nav_move", "args": { "verb": { "verb": "next" } } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-090");
    let err = h.send(&json!({ "command": "nav_where" })).unwrap_err();
    assert_eq!(err.code, "SNR-APP-090");
    assert_eq!(h.cursors(), Vec::new());
}

/// A11Y-10: the explorer moves the one cursor; the row follows the node, every move is a
/// `cursor` event with origin `nav`, and the move is announced.
#[test]
fn a11y_10_explorer_verbs_move_the_one_cursor_and_its_row() {
    let mut h = Harness::new("nav-move");
    h.compile(&["temp"]);
    // From the root, Down goes into the series, then its segment, then a point.
    let series = h.step("into");
    assert_eq!(series["reply"], "nav");
    assert_eq!(series["outcome"], "moved");
    assert_eq!(levels(&series["cursor"]), ["dataset", "series"]);
    assert_eq!(
        series["cursor"]["row"], 0,
        "a series stands for its point nearest x"
    );
    h.step("into");
    let point = h.step("into");
    assert_eq!(levels(&point["cursor"]).last().unwrap(), "point");
    let next = h.step("next");
    assert_eq!(next["cursor"]["row"], 1);
    assert_eq!(next["cursor"]["x"], 2.0);
    assert!(next["highlight"].is_object(), "{next}");
    assert_eq!(next["announced"], true);
    let event = h.cursors().pop().unwrap();
    assert_eq!(event.origin, Some(CursorOrigin::Nav));
    assert_eq!(event.row, Some(1));
    assert_eq!(
        serde_json::to_value(&event).unwrap()["seq"],
        next["cursor"]["seq"]
    );
    assert_eq!(
        h.ok(&json!({ "command": "cursor_get" }))["node"],
        next["cursor"]["node"]
    );
    h.wait_for_announcements(4);
}

/// A11Y-10: a table move puts the explorer on that row's point, so the next verb starts
/// there; an explorer move moves the table's row (both read the one cursor).
#[test]
fn a11y_10_the_explorer_starts_from_the_table_row() {
    let mut h = Harness::new("nav-from-table");
    h.compile(&["temp"]);
    h.ok(&json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": 2, "origin": "table" } }));
    let previous = h.step("previous");
    assert_eq!(previous["outcome"], "moved");
    assert_eq!(previous["cursor"]["row"], 1);
    let first = h.step("first");
    assert_eq!(first["cursor"]["row"], 0);
}

/// A11Y-08: a move past the edge stays put, names the edge and plays `E-BOUNDARY`; a verb
/// with no meaning plays `E-INVALID`.
#[test]
fn a11y_08_boundaries_and_invalid_verbs_keep_the_cursor() {
    let mut h = Harness::new("nav-edges");
    h.compile(&["temp"]);
    h.ok(&json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": 2, "origin": "table" } }));
    let edge = h.step("next");
    assert_eq!(edge["outcome"], "boundary");
    assert_eq!(edge["edge"], "last");
    assert_eq!(edge["earcon"], "E-BOUNDARY");
    assert_eq!(edge["cursor"]["row"], 2);
    let lateral = h.step("nextSeries");
    assert_eq!(lateral["outcome"], "invalid");
    assert_eq!(lateral["reason"], "oneSeries");
    assert_eq!(lateral["earcon"], "E-INVALID");
    let go = h.nav(&json!({ "verb": "goTo", "x": { "number": 2.2 } }));
    assert_eq!(go["cursor"]["row"], 1);
    let landmark = h.nav(&json!({ "verb": "landmark", "landmark": "xAxis" }));
    assert_eq!(levels(&landmark["cursor"]).last().unwrap(), "axis");
    assert_eq!(
        landmark["cursor"]["row"], 1,
        "an axis leaves the row where it was"
    );
}

/// A11Y-07: "where am I" is announced and moves nothing.
#[test]
fn a11y_07_where_am_i_announces_without_moving() {
    let mut h = Harness::new("nav-where");
    h.compile(&["temp"]);
    h.ok(&json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": 1, "origin": "table" } }));
    let events = h.cursors().len();
    let spoken = h.announcements();
    let reply = h.ok(&json!({ "command": "nav_where" }));
    assert_eq!(reply, json!({ "reply": "announce", "queued": true }));
    h.wait_for_announcements(spoken + 1);
    assert_eq!(h.cursors().len(), events);
}

/// Keyboard §3 (Filters): Enter on a filter takes its series out of playback and view.
/// The exclusion is on the cursor, survives a recompile by the series' name, and ends when
/// a new project replaces the data.
#[test]
fn b03_t09_a_filter_excludes_its_series_until_the_data_is_replaced() {
    let mut h = Harness::new("nav-filter");
    h.compile(&["temp", "rain"]);
    h.nav(&json!({ "verb": "landmark", "landmark": "filters" }));
    h.step("into");
    let toggled = h.step("activate");
    assert_eq!(toggled["outcome"], "toggled");
    assert_eq!(toggled["included"], false);
    let excluded = toggled["cursor"]["excluded"].as_array().unwrap().clone();
    assert_eq!(excluded.len(), 1, "{toggled}");
    assert_eq!(levels(&toggled["cursor"]).last().unwrap(), "filter");

    // A recompile renumbers the nodes; the same series stays out.
    h.compile(&["temp", "rain"]);
    let after = h.ok(&json!({ "command": "cursor_get" }));
    let labels = |list: &Value| -> Vec<String> {
        list.as_array()
            .unwrap()
            .iter()
            .map(|s| s["label"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(labels(&after["excluded"]), labels(&Value::Array(excluded)));

    // Enter again puts it back.
    h.nav(&json!({ "verb": "landmark", "landmark": "filters" }));
    h.step("into");
    assert_eq!(h.step("activate")["included"], true);
    h.step("activate");
    h.ok(&json!({ "command": "project_new" }));
    assert_eq!(
        h.ok(&json!({ "command": "cursor_get" }))["excluded"],
        json!([])
    );
}

/// ADR-0017 "Navigation": an export renders every series, so while a filter excludes one,
/// starting an export says so; with nothing excluded it says nothing about it.
#[test]
fn b03_t09_an_export_while_a_series_is_excluded_says_it_includes_all_series() {
    let mut h = Harness::new("nav-export");
    h.compile(&["temp", "rain"]);
    let note = "Export includes all series";
    let spoken = |h: &Harness| -> Vec<String> {
        h.sink
            .events()
            .into_iter()
            .filter_map(|event| match event {
                Event::Announcement(info) => Some(info.speech),
                _ => None,
            })
            .collect()
    };
    let wav = h.dir.path().join("all.wav");
    h.ok(&json!({ "command": "export_render", "args": { "options": { "path": wav } } }));
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !spoken(&h).iter().any(|s| s.contains(note)),
        "{:?}",
        spoken(&h)
    );

    h.nav(&json!({ "verb": "landmark", "landmark": "filters" }));
    h.step("into");
    assert_eq!(h.step("activate")["included"], false);
    let wav = h.dir.path().join("filtered.wav");
    h.ok(&json!({ "command": "export_render", "args": { "options": { "path": wav } } }));
    let start = Instant::now();
    while !spoken(&h).iter().any(|s| s.contains(note)) {
        assert!(start.elapsed() < Duration::from_secs(5), "{:?}", spoken(&h));
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// ADR-0017 "Cursor": closing the dataset clears the row, and the explorer leaves its node.
#[test]
fn b03_t09_closing_the_dataset_clears_the_explorer_node() {
    let mut h = Harness::new("nav-close");
    h.compile(&["temp"]);
    h.ok(&json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": 1, "origin": "table" } }));
    h.step("next");
    h.ok(&json!({ "command": "dataset_close", "args": { "id": "d1" } }));
    let cleared = h.cursors().pop().unwrap();
    assert_eq!(
        (cleared.dataset, cleared.row, cleared.node),
        (None, None, None)
    );
}
