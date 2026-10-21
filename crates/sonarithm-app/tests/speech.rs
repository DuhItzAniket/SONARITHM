// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Speech sequenced with sonification through the app core (B03-T07): the value on pause
//! (A11Y-09), self-voicing (A11Y-19), repeat, stop all, and the `WebVTT` cue sidecar timed by
//! AUDAXIS's announcements (FR-101, A11Y-20). Headless, on the silent null device.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sonarithm::{Sonarithm, Spec};
use sonarithm_app::ipc::{AnnouncementClass, AnnouncementInfo, Event};
use sonarithm_app::speech::SpeakerSource;
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};
use sonarithm_audaxis::announce::{Composer, Settings};
use sonarithm_core::nav::NavKind;
use sonarithm_core::{EngineId, InstrumentRef, ScoreEvent, ScoreTrack, Speech, TrackId, TrackRole};
use sonarithm_i18n::Localizer;
use sonarithm_speech::{Recorder, Speaker, WordsPerMinute};

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n4,10.5\n5,11.0\n6,9.0\n7,8.0\n8,12.5\n";
const VALUES: [&str; 8] = ["8.5", "9.25", "7", "10.5", "11", "9", "8", "12.5"];

fn spec() -> String {
    json!({
        "specVersion": "0.1",
        "name": "Temperatures",
        "data": { "source": { "kind": "dataset", "id": "d1" } },
        "substrate": { "time": { "field": "day" } },
        "tracks": [{ "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
    })
    .to_string()
}

struct Harness {
    app: App,
    sink: CollectingSink,
    _dir: TempDir,
}

impl Harness {
    fn new(label: &str, speaker: Option<Recorder>) -> Self {
        let dir = TempDir::new(label);
        let sink = CollectingSink::new();
        let mut config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        if let Some(recorder) = speaker {
            config = config.with_speaker(SpeakerSource::new(move || {
                Ok(Box::new(recorder.clone()) as Box<dyn Speaker>)
            }));
        }
        let mut app = App::new(config, Arc::new(sink.clone())).unwrap();
        let csv = dir.path().join("temps.csv");
        std::fs::write(&csv, CSV).unwrap();
        for request in [
            json!({ "command": "dataset_open", "args": { "path": csv } }),
            json!({ "command": "spec_set", "args": { "text": spec() } }),
            json!({ "command": "compile" }),
        ] {
            app.dispatch_json(&request.to_string()).unwrap();
        }
        Self {
            app,
            sink,
            _dir: dir,
        }
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.app
            .dispatch_json(&request.to_string())
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }

    fn self_voicing(&mut self) {
        let mut settings = self.ok(&json!({ "command": "settings_get" }));
        let settings = settings.as_object_mut().unwrap();
        settings.remove("reply");
        settings["speech"]["route"] = json!("selfVoicing");
        self.ok(&json!({ "command": "settings_set", "args": { "settings": settings } }));
    }

    /// Plays for a moment, then pauses.
    fn play_then_pause(&mut self) {
        self.ok(&json!({ "command": "transport_play" }));
        let start = Instant::now();
        while self.ok(&json!({ "command": "transport_state" }))["positionS"]
            .as_f64()
            .unwrap_or(0.0)
            < 0.6
            && start.elapsed() < Duration::from_secs(5)
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        self.ok(&json!({ "command": "transport_pause" }));
    }

    /// The announcements so far, waiting up to 2 s for at least `count`.
    fn announcements(&self, count: usize) -> Vec<AnnouncementInfo> {
        let start = Instant::now();
        loop {
            let all: Vec<AnnouncementInfo> = self
                .sink
                .events()
                .into_iter()
                .filter_map(|event| match event {
                    Event::Announcement(info) => Some(info),
                    _ => None,
                })
                .collect();
            if all.len() >= count || start.elapsed() > Duration::from_secs(2) {
                return all;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[test]
fn a11y_09_the_current_value_is_announced_on_pause() {
    let mut h = Harness::new("a11y09", None);
    h.play_then_pause();
    let announced = h.announcements(1);
    let [info] = announced.as_slice() else {
        panic!("one announcement: {announced:?}");
    };
    assert!(info.speech.starts_with("Paused. "), "{info:?}");
    assert!(
        VALUES.iter().any(|value| info.speech.contains(value)),
        "a data value: {info:?}"
    );
    assert_eq!(info.class, AnnouncementClass::Response);
    assert!(!info.assertive && !info.self_voiced);
    assert!(info.node.is_some());
}

#[test]
fn a11y_19_self_voicing_speaks_through_the_speaker() {
    let recorder = Recorder::new();
    let mut h = Harness::new("a11y19", Some(recorder.clone()));
    h.self_voicing();
    h.play_then_pause();
    let announced = h.announcements(1);
    assert!(announced[0].self_voiced, "{announced:?}");
    let said = recorder.said();
    assert_eq!(said.len(), 1);
    assert_eq!(said[0].text, announced[0].speech);
    // Stop all audio stops speech too (A11Y-15).
    h.ok(&json!({ "command": "stop_all" }));
    assert!(recorder.stops() >= 1);
}

#[test]
fn without_a_speaker_self_voicing_falls_back_to_the_screen_reader() {
    let mut h = Harness::new("fallback", None);
    h.self_voicing();
    h.play_then_pause();
    let announced = h.announcements(1);
    assert!(!announced[0].self_voiced, "{announced:?}");
}

#[test]
fn repeat_speaks_the_last_announcement_again() {
    let mut h = Harness::new("repeat", None);
    let nothing = h.ok(&json!({ "command": "announce_repeat" }));
    assert_eq!(nothing["queued"], false);
    h.play_then_pause();
    let first = h.announcements(1);
    let again = h.ok(&json!({ "command": "announce_repeat" }));
    assert_eq!(again["reply"], "announce");
    assert_eq!(again["queued"], true);
    let both = h.announcements(2);
    assert_eq!(both.len(), 2, "{both:?}");
    assert_eq!(both[1].speech, first[0].speech);
    assert!(both[1].seq > both[0].seq);
}

/// FR-101 and A11Y-20: the cue sidecar of a narrated Score (each point announced by
/// AUDAXIS when it sounds) is valid `WebVTT` whose cues never overlap, each lasting until the
/// next or until its speech ends.
#[test]
fn a11y_20_the_cue_sidecar_sequences_audaxis_announcements() {
    let snr = Sonarithm::builder()
        .with_default_engines()
        .with_navigation()
        .build()
        .unwrap();
    let values: Vec<Value> = CSV
        .lines()
        .skip(1)
        .map(|line| {
            let (day, temp) = line.split_once(',').unwrap();
            json!({ "day": day.parse::<f64>().unwrap(), "temp": temp.parse::<f64>().unwrap() })
        })
        .collect();
    let inline = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "inline", "values": values } },
        "substrate": { "time": { "field": "day" } },
        "tracks": [{ "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
    })
    .to_string();
    let compiled = snr.compile(&Spec::from_json_str(&inline).unwrap()).unwrap();
    let tree = compiled.nav().unwrap();
    let localizer = Localizer::new("en-US").unwrap();
    let mut composer = Composer::new(&localizer, Settings::default());
    let mut narration = ScoreTrack::new(
        TrackId::new("narration"),
        TrackRole::Speech,
        EngineId::from_static("audaxis"),
        InstrumentRef("speech".to_owned()),
    );
    for (id, node) in tree.iter() {
        if let NavKind::Point(point) = &node.kind
            && let Some(t) = point.t
        {
            let text = composer.node(tree, id).unwrap().speech;
            narration
                .events
                .push(ScoreEvent::Speech(Speech::new(t, text)));
        }
    }
    assert_eq!(narration.events.len(), 8);
    let mut score = compiled.score().clone();
    score.tracks.push(narration);
    let cues = sonarithm_export::cues::cues_at(&score, WordsPerMinute::DEFAULT);
    let vtt = sonarithm_export::cues::webvtt(&cues);
    assert!(vtt.starts_with("WEBVTT\n\n"));
    let blocks: Vec<&str> = vtt.trim_end().split("\n\n").skip(1).collect();
    assert_eq!(blocks.len(), cues.len());
    for (index, block) in blocks.iter().enumerate() {
        let lines: Vec<&str> = block.lines().collect();
        assert_eq!(lines[0], (index + 1).to_string());
        let (start, end) = lines[1].split_once(" --> ").unwrap();
        assert!(end > start, "{block}");
        assert!(lines.len() > 2, "{block}");
    }
    for pair in cues.windows(2) {
        assert!(pair[0].end <= pair[1].start, "sequenced: {pair:?}");
    }
}
