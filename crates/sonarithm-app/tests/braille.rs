// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The braille line on the one cursor (B03-T10; keyboard-and-navigation §10, §14; A11Y-10,
//! A11Y-21, A11Y-27): `cursor` events carry the review line or the sparkline, the cell
//! count and mode are app settings, and a routing key on the sparkline moves the one cursor
//! with origin `braille`, without an announcement (INFER, OD-30).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use sonarithm_app::ipc::{BrailleMode, CursorOrigin, CursorState, Event, IpcError};
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n";

struct Harness {
    app: App,
    sink: CollectingSink,
    _dir: TempDir,
}

impl Harness {
    fn new(label: &str) -> Self {
        let dir = TempDir::new(label);
        let sink = CollectingSink::new();
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let app = App::new(config, Arc::new(sink.clone())).unwrap();
        let csv = dir.path().join("weather.csv");
        std::fs::write(&csv, CSV).unwrap();
        let mut h = Self {
            app,
            sink,
            _dir: dir,
        };
        h.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
        let spec = json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "dataset", "id": "d1" } },
            "substrate": { "time": { "field": "day" } },
            "tracks": [{ "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
        })
        .to_string();
        h.ok(&json!({ "command": "spec_set", "args": { "text": spec } }));
        let report = h.ok(&json!({ "command": "compile" }));
        assert_eq!(report["ok"], true, "{report}");
        h
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.send(request)
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }

    /// Changes the settings with `change` (on the JSON object) and sets them.
    fn settings(
        &mut self,
        change: impl FnOnce(&mut serde_json::Map<String, Value>),
    ) -> Result<Value, IpcError> {
        let mut settings = self.ok(&json!({ "command": "settings_get" }));
        let settings = settings.as_object_mut().unwrap();
        settings.remove("reply");
        change(settings);
        self.send(&json!({ "command": "settings_set", "args": { "settings": settings } }))
    }

    fn set_row(&mut self, row: u64) -> Value {
        self.ok(&json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": row, "origin": "plot" } }))
    }

    fn last_cursor(&self) -> CursorState {
        self.sink
            .events()
            .into_iter()
            .rev()
            .find_map(|event| match event {
                Event::Cursor(state) => Some(state),
                _ => None,
            })
            .unwrap()
    }

    fn announcements(&self) -> usize {
        self.sink
            .events()
            .iter()
            .filter(|event| matches!(event, Event::Announcement(_)))
            .count()
    }
}

/// A11Y-21, keyboard §10: with braille on (the default), a cursor event carries the review
/// line of the cursor's point; nothing is fitted without a cell count.
#[test]
fn a11y_21_cursor_events_carry_the_review_line() {
    let mut h = Harness::new("braille-review");
    h.set_row(1);
    let braille = h.last_cursor().braille.unwrap();
    assert_eq!(braille.mode, BrailleMode::Review);
    assert!(braille.text.contains("9.25"), "{}", braille.text);
    assert_eq!((braille.caret, braille.cells), (0, None));
}

/// A11Y-21: in sparkline mode the line is one 8-dot cell per point of the segment, with the
/// caret on the cursor's point, and the cell count travels with it.
#[test]
fn a11y_21_sparkline_mode_draws_the_segment() {
    let mut h = Harness::new("braille-spark");
    h.settings(|s| s["braille"] = json!({ "mode": "sparkline", "cells": 20 }))
        .unwrap();
    h.set_row(1);
    let braille = h.last_cursor().braille.unwrap();
    assert_eq!(braille.mode, BrailleMode::Sparkline);
    assert_eq!(braille.text.chars().count(), 3);
    assert!(
        braille
            .text
            .chars()
            .all(|c| ('\u{2800}'..='\u{28FF}').contains(&c))
    );
    assert_eq!(braille.caret, 1);
    assert_eq!(braille.cells, Some(20));
}

/// A11Y-27, keyboard §14, OD-30: a routing key on a sparkline cell moves the one cursor to
/// that point with origin `braille`, and nothing is announced.
#[test]
fn a11y_27_a_routing_key_moves_the_one_cursor_silently() {
    let mut h = Harness::new("braille-route");
    h.settings(|s| s["braille"] = json!({ "mode": "sparkline" }))
        .unwrap();
    h.set_row(1);
    std::thread::sleep(Duration::from_millis(200));
    let before = h.announcements();
    let reply = h.ok(&json!({ "command": "braille_route", "args": { "cell": 2 } }));
    assert_eq!(reply["row"], 2, "{reply}");
    let state = h.last_cursor();
    assert_eq!(state.origin, Some(CursorOrigin::Braille));
    assert_eq!(state.row, Some(2));
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(h.announcements(), before, "a braille move is not announced");
}

/// Keyboard §14: in review mode, and past the last cell, a routing key moves nothing.
#[test]
fn a11y_27_routing_moves_nothing_in_review_mode_or_past_the_line() {
    let mut h = Harness::new("braille-review-route");
    h.set_row(1);
    let reply = h.ok(&json!({ "command": "braille_route", "args": { "cell": 0 } }));
    assert_eq!(reply["row"], 1);
    h.settings(|s| s["braille"] = json!({ "mode": "sparkline" }))
        .unwrap();
    let reply = h.ok(&json!({ "command": "braille_route", "args": { "cell": 99 } }));
    assert_eq!(reply["row"], 1);
}

/// A11Y-10: with braille off the cursor carries no line; a zero cell count is refused.
#[test]
fn a11y_10_braille_off_sends_no_line_and_zero_cells_are_refused() {
    let mut h = Harness::new("braille-off");
    h.settings(|s| s["modalities"]["braille"] = json!(false))
        .unwrap();
    h.set_row(0);
    assert_eq!(h.last_cursor().braille, None);
    let err = h
        .settings(|s| s["braille"] = json!({ "mode": "review", "cells": 0 }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-013");
}
