// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Live streams through the app core (B06-T05; A11Y-16, ADR-0012): a stream opens only
//! with an approved offer, its notices are spoken, rate-limited, paused, hidden and
//! stopped on request, and errors are spoken whatever the stream's state. The streams
//! follow local files, so nothing touches the network (B01-T06's harness checks).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sonarithm_app::ipc::{AnnouncementClass, AnnouncementInfo, Event, IpcError};
use sonarithm_app::testing::TempDir;
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

    /// Offers a stream that follows `path` and connects it; the stream list after.
    fn follow(&mut self, path: &Path) -> Value {
        let offer = self.ok(&json!({ "command": "stream_offer",
            "args": { "transport": "tail", "path": path } }));
        self.ok(&json!({ "command": "stream_connect", "args": { "token": offer["token"] } }))
    }

    /// Waits up to 3 s for an announcement that `matches`.
    fn announcement(
        &self,
        matches: impl Fn(&AnnouncementInfo) -> bool,
    ) -> Option<AnnouncementInfo> {
        let start = Instant::now();
        loop {
            let found = self
                .sink
                .events()
                .into_iter()
                .find_map(|event| match event {
                    Event::Announcement(info) if matches(&info) => Some(info),
                    _ => None,
                });
            if found.is_some() || start.elapsed() > Duration::from_secs(3) {
                return found;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[test]
fn a11y_16_a_stream_opens_only_with_an_approved_offer() {
    let mut h = Harness::new("offer");
    let none = h.send(&json!({ "command": "stream_connect", "args": { "token": 1 } }));
    assert_eq!(none.unwrap_err().code, "SNR-APP-072");
    let file = h.dir.path().join("sensor.csv");
    std::fs::write(&file, "").unwrap();
    let offer = h.ok(&json!({ "command": "stream_offer",
        "args": { "transport": "tail", "path": file } }));
    assert_eq!(offer["reply"], "stream_offer");
    assert_eq!(offer["transport"], "tail");
    assert_eq!(
        offer["target"], "sensor.csv",
        "the file alone (ADR-0017 §7)"
    );
    let listed = h.ok(&json!({ "command": "stream_list" }));
    assert_eq!(listed["streams"], json!([]), "an offer opens nothing");
    let token = offer["token"].clone();
    let opened = h.ok(&json!({ "command": "stream_connect", "args": { "token": token } }));
    assert_eq!(opened["streams"][0]["id"], "s1");
    assert_eq!(opened["streams"][0]["target"], "sensor.csv");
    let again = h.send(&json!({ "command": "stream_connect", "args": { "token": token } }));
    assert_eq!(
        again.unwrap_err().code,
        "SNR-APP-072",
        "one approval, one connection"
    );
    let bad = h.send(&json!({ "command": "stream_offer",
        "args": { "transport": "tail", "path": "relative.csv" } }));
    assert_eq!(bad.unwrap_err().code, "SNR-APP-071");
}

#[test]
fn a11y_16_stream_notices_can_be_hidden_paused_resumed_and_stopped() {
    let mut h = Harness::new("controls");
    let file = h.dir.path().join("sensor.csv");
    std::fs::write(&file, "").unwrap();
    h.follow(&file);
    let connected = h
        .announcement(|info| info.class == AnnouncementClass::Stream)
        .unwrap();
    assert_eq!(connected.speech, "sensor.csv connected");
    assert!(!connected.assertive);

    let hidden = h.ok(&json!({ "command": "stream_hide", "args": { "id": "s1" } }));
    assert_eq!(hidden["streams"][0]["notices"], "hidden");
    let paused = h.ok(&json!({ "command": "stream_pause", "args": { "id": "s1" } }));
    assert_eq!(paused["streams"][0]["notices"], "paused");
    let resumed = h.ok(&json!({ "command": "stream_resume", "args": { "id": "s1" } }));
    assert_eq!(resumed["streams"][0]["notices"], "active");
    let stopped = h.ok(&json!({ "command": "stream_stop", "args": { "id": "s1" } }));
    assert_eq!(stopped["streams"], json!([]));
    let gone = h.send(&json!({ "command": "stream_pause", "args": { "id": "s1" } }));
    assert_eq!(gone.unwrap_err().code, "SNR-APP-070");
}

#[test]
fn a11y_16_errors_are_spoken_even_from_a_hidden_stream() {
    let mut h = Harness::new("errors");
    let missing = h.dir.path().join("not-yet.csv");
    h.follow(&missing);
    h.ok(&json!({ "command": "stream_hide", "args": { "id": "s1" } }));
    let error = h
        .announcement(|info| info.assertive)
        .unwrap_or_else(|| panic!("no error announced: {:?}", h.sink.events()));
    assert!(error.speech.starts_with("not-yet.csv error: "), "{error:?}");
    assert!(
        !error.speech.contains(&*h.dir.path().to_string_lossy()),
        "no folder in the text: {error:?}"
    );
}
