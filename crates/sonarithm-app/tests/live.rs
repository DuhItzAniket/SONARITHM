// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Live playback through the app core (B06-T07; FR-005): a stream's newest window is
//! compiled about once a second and swapped in while playing; playback trails the newest
//! data and never stops at a window's end. The stream follows a local file written by a
//! synthetic feed, so nothing touches the network, and the output is the silent device.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sonarithm_app::ipc::{Event, IpcError};
use sonarithm_app::testing::{FileFeed, TempDir};
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

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

    fn state(&mut self) -> Value {
        self.ok(&json!({ "command": "transport_state" }))
    }

    /// Follows `file` as stream `s1` and sets a live spec over it that maps `pitch`.
    fn live(&mut self, file: &Path, pitch: &str) {
        let offer = self.ok(&json!({ "command": "stream_offer",
            "args": { "transport": "tail", "path": file } }));
        self.ok(&json!({ "command": "stream_connect", "args": { "token": offer["token"] } }));
        let spec = json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "stream", "transport": "tail", "path": file,
                "window": { "s": 5 } } },
            "substrate": { "time": { "field": "t", "pace": { "secondsPerUnit": 1 } } },
            "tracks": [{ "id": "v", "mark": "glide", "channels": { "pitch": { "field": pitch } } }]
        });
        self.ok(&json!({ "command": "spec_set", "args": { "text": spec.to_string() } }));
    }

    /// Waits up to `limit` for the transport state to satisfy `done`.
    fn wait(&mut self, limit: Duration, done: impl Fn(&Value) -> bool) -> Value {
        let start = Instant::now();
        loop {
            let state = self.state();
            if done(&state) || start.elapsed() > limit {
                return state;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn number(state: &Value, key: &str) -> f64 {
    state[key].as_f64().unwrap()
}

#[test]
fn fr_005_a_stream_plays_live_past_every_window_end() {
    let mut h = Harness::new("live");
    let file = h.dir.path().join("live.ndjson");
    std::fs::write(&file, "").unwrap();
    h.live(&file, "v");
    let feed = FileFeed::start(&file, 200.0);
    let started = h.ok(&json!({ "command": "live_start", "args": { "stream": "s1" } }));
    assert_eq!(started["live"], true);
    // The first window arrives within a compile period or two.
    let first = h.wait(Duration::from_secs(5), |s| number(s, "durationS") > 0.5);
    assert_eq!(first["loaded"], true, "{first}");
    // Nothing sounds before a play command (A11Y-15).
    assert_ne!(first["status"], "playing");
    h.ok(&json!({ "command": "transport_play" }));
    std::thread::sleep(Duration::from_secs(4));
    let state = h.state();
    assert_eq!(state["status"], "playing", "{state}");
    assert_eq!(state["live"], true);
    // The Score is the stream's timeline: it grows with the data, about 1 s per second.
    let (duration, position) = (number(&state, "durationS"), number(&state, "positionS"));
    assert!(
        duration > number(&first, "durationS") + 2.5,
        "{first} then {state}"
    );
    // The playhead trails the newest data by about 2 s (the band is 0.5 s to 4 s, plus
    // up to a compile period of new data not yet swapped in).
    let behind = duration - position;
    assert!((0.0..=5.0).contains(&behind), "{state}");
    // The feed ran throughout: about 200 rows a second.
    assert!(feed.rows() > 500, "{} rows", feed.rows());
    // No window end stopped playback.
    let finished = h
        .sink
        .events()
        .into_iter()
        .filter(|e| matches!(e, Event::PlaybackFinished { .. }))
        .count();
    assert_eq!(finished, 0);
    // Leaving live mode pauses on the last window.
    let stopped = h.ok(&json!({ "command": "live_stop" }));
    assert_eq!(
        (stopped["live"].clone(), stopped["status"].clone()),
        (json!(false), json!("paused"))
    );
    assert!(stopped["xruns"].is_u64(), "{stopped}");
}

#[test]
fn live_playback_needs_a_stream_spec_and_an_open_stream() {
    let mut h = Harness::new("live-refused");
    let start = json!({ "command": "live_start", "args": { "stream": "s1" } });
    assert_eq!(h.send(&start).unwrap_err().code, "SNR-APP-003");
    let inline = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "inline", "values": [{ "t": 0, "v": 1 }] } },
        "tracks": [{ "id": "v", "mark": "tone", "channels": { "pitch": { "field": "v" } } }]
    });
    h.ok(&json!({ "command": "spec_set", "args": { "text": inline.to_string() } }));
    assert_eq!(h.send(&start).unwrap_err().code, "SNR-APP-073");
    let file = h.dir.path().join("live.ndjson");
    std::fs::write(&file, "").unwrap();
    h.live(&file, "v");
    let elsewhere = json!({ "command": "live_start", "args": { "stream": "s9" } });
    assert_eq!(h.send(&elsewhere).unwrap_err().code, "SNR-APP-070");
    assert_eq!(h.state()["live"], false);
}

#[test]
fn a_window_that_does_not_compile_is_reported_once() {
    let mut h = Harness::new("live-notice");
    let file = h.dir.path().join("live.ndjson");
    std::fs::write(&file, "").unwrap();
    h.live(&file, "missing");
    let _feed = FileFeed::start(&file, 200.0);
    h.ok(&json!({ "command": "live_start", "args": { "stream": "s1" } }));
    std::thread::sleep(Duration::from_millis(3_500));
    let notices: Vec<IpcError> = h
        .sink
        .events()
        .into_iter()
        .filter_map(|e| match e {
            Event::LiveNotice { error } => Some(error),
            _ => None,
        })
        .collect();
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert_eq!(notices[0].code, "SNR-APP-074");
    assert_eq!(h.state()["loaded"], false);
    h.ok(&json!({ "command": "live_stop" }));
    assert_eq!(h.state()["live"], false);
}
