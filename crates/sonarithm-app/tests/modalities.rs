// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The explorer's modality and verbosity toggles (B03-T09; keyboard-and-navigation §5.5;
//! A11Y-10, A11Y-12): `modality_toggle` flips the effective value in the explicit-UI-action
//! layer, which is saved over the profile, and every output follows it; "Speech off" is
//! heard before speech goes quiet and "Speech on" after it returns; speech, sonification
//! and braille are never all off (`SNR-APP-094`); `choices_reset` returns to the profile.
//! Headless, on the silent null device.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sonarithm_app::ipc::{AnnouncementInfo, Event, IpcError};
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n4,10.5\n";

struct Harness {
    app: App,
    sink: CollectingSink,
    config: PathBuf,
    dir: TempDir,
}

impl Harness {
    fn new(label: &str) -> Self {
        let dir = TempDir::new(label);
        let config = dir.path().join("config");
        let (app, sink) = Self::open(&config);
        Self {
            app,
            sink,
            config,
            dir,
        }
    }

    fn open(config: &std::path::Path) -> (App, CollectingSink) {
        let sink = CollectingSink::new();
        let app = App::new(
            AppConfig::new(config.to_path_buf()).with_audio(AudioOutput::null()),
            Arc::new(sink.clone()),
        )
        .unwrap();
        (app, sink)
    }

    /// Closes the app and starts it again on the same settings, as a restart does.
    fn restart(&mut self) {
        let (app, sink) = Self::open(&self.config);
        self.app = app;
        self.sink = sink;
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.send(request)
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }

    fn toggle(&mut self, modality: &str) -> Result<Value, IpcError> {
        self.send(&json!({ "command": "modality_toggle", "args": { "modality": modality } }))
    }

    /// An open dataset and a compiled Score, so the explorer's verbs announce.
    fn compile(&mut self) {
        let csv = self.dir.path().join("temps.csv");
        std::fs::write(&csv, CSV).unwrap();
        let spec = json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "dataset", "id": "d1" } },
            "substrate": { "time": { "field": "day" } },
            "tracks": [{ "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
        })
        .to_string();
        self.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
        self.ok(&json!({ "command": "spec_set", "args": { "text": spec } }));
        let report = self.ok(&json!({ "command": "compile" }));
        assert_eq!(report["ok"], true, "{report}");
    }

    fn step(&mut self, verb: &str) {
        self.ok(&json!({ "command": "nav_move", "args": { "verb": { "verb": verb } } }));
    }

    /// The announcements so far, waiting up to 5 s for at least `count`.
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
            if all.len() >= count {
                return all;
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "{count} announcements: {all:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The effective value of `key` and its layer.
    fn effective(&mut self, key: &str) -> (Value, String) {
        let reply = self.ok(&json!({ "command": "settings_effective" }));
        effective_in(&reply, key)
    }
}

fn effective_in(reply: &Value, key: &str) -> (Value, String) {
    let setting = reply["settings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["key"] == key)
        .unwrap_or_else(|| panic!("{key} in {reply}"));
    (
        setting["value"].clone(),
        setting["layer"].as_str().unwrap().to_owned(),
    )
}

#[test]
fn b03_t09_speech_off_is_said_before_announcements_go_quiet() {
    let mut h = Harness::new("speech-off");
    h.compile();
    let reply = h.toggle("speech").unwrap();
    assert_eq!(
        effective_in(&reply, "interaction.modalities.speech"),
        (json!(false), "session".to_owned())
    );
    // "Speech off" is spoken: it was queued while speech was still on.
    let said = h.announcements(1);
    assert_eq!(said[0].speech, "Speech off");
    assert!(!said[0].muted, "{said:?}");
    // A move after it is shown and logged, but not spoken.
    h.step("next");
    let said = h.announcements(2);
    assert!(said[1].muted, "{said:?}");
    assert!(said[1].seq > said[0].seq);
    // "Speech on" is spoken: speech was on again before it was queued.
    h.toggle("speech").unwrap();
    let said = h.announcements(3);
    assert_eq!(said[2].speech, "Speech on");
    assert!(!said[2].muted, "{said:?}");
    h.step("next");
    let said = h.announcements(4);
    assert!(!said[3].muted, "{said:?}");
    assert!(said[3].seq > said[2].seq);
}

#[test]
fn b03_t09_sonification_off_silences_the_transport() {
    let mut h = Harness::new("sonification-off");
    let state = h.ok(&json!({ "command": "transport_state" }));
    assert_eq!(state["muted"], false);
    h.toggle("sonification").unwrap();
    assert_eq!(h.ok(&json!({ "command": "transport_state" }))["muted"], true);
    assert_eq!(h.announcements(1)[0].speech, "Sonification off");
    h.toggle("sonification").unwrap();
    assert_eq!(h.ok(&json!({ "command": "transport_state" }))["muted"], false);
}

#[test]
fn a11y_12_shift_t_switches_the_verbosity_of_announcements() {
    let mut h = Harness::new("verbosity");
    h.compile();
    h.step("next");
    let terse = h.announcements(1)[0].speech.clone();
    h.toggle("verbosity").unwrap();
    assert_eq!(
        h.effective("interaction.verbosity"),
        (json!("verbose"), "session".to_owned())
    );
    assert_eq!(h.announcements(2)[1].speech, "Verbose announcements");
    // Back and forth to the same point: now said verbosely.
    h.step("previous");
    h.announcements(3);
    h.step("next");
    let verbose = h.announcements(4)[3].speech.clone();
    assert_ne!(verbose, terse);
    assert!(verbose.len() > terse.len(), "{terse:?} then {verbose:?}");
}

#[test]
fn a11y_10_braille_and_visual_toggles_flip_the_effective_modality() {
    let mut h = Harness::new("braille-visual");
    let (braille, _) = h.effective("interaction.modalities.braille");
    let (visual, _) = h.effective("interaction.modalities.visual");
    h.toggle("braille").unwrap();
    h.announcements(1);
    h.toggle("visual").unwrap();
    assert_eq!(
        h.effective("interaction.modalities.braille").0,
        json!(!braille.as_bool().unwrap())
    );
    assert_eq!(
        h.effective("interaction.modalities.visual").0,
        json!(!visual.as_bool().unwrap())
    );
    let said: Vec<String> = h.announcements(2).into_iter().map(|a| a.speech).collect();
    assert_eq!(said, ["Braille off", "Visual highlight off"]);
}

#[test]
fn a11y_10_toggles_persist_over_the_profile_without_touching_the_base_settings() {
    let mut h = Harness::new("persist");
    let before = h.ok(&json!({ "command": "settings_get" }));
    h.toggle("visual").unwrap();
    h.announcements(1);
    h.toggle("verbosity").unwrap();
    h.announcements(2);
    h.restart();
    assert_eq!(
        h.effective("interaction.modalities.visual"),
        (json!(false), "session".to_owned())
    );
    assert_eq!(h.effective("interaction.verbosity").0, json!("verbose"));
    // Only the choices changed: a profile cannot override them through the base settings.
    let after = h.ok(&json!({ "command": "settings_get" }));
    assert_eq!(after["modalities"], before["modalities"]);
    assert_eq!(after["verbosity"], before["verbosity"]);
    assert_eq!(after["choices"], json!({ "visual": false, "verbosity": "verbose" }));
}

#[test]
fn b03_t09_choices_reset_returns_to_the_profile_defaults() {
    let mut h = Harness::new("reset");
    let (visual, layer) = h.effective("interaction.modalities.visual");
    h.toggle("visual").unwrap();
    h.announcements(1);
    h.toggle("sonification").unwrap();
    h.announcements(2);
    let reply = h.ok(&json!({ "command": "choices_reset" }));
    assert_eq!(
        effective_in(&reply, "interaction.modalities.visual"),
        (visual, layer)
    );
    assert_eq!(h.ok(&json!({ "command": "transport_state" }))["muted"], false);
    assert_eq!(h.ok(&json!({ "command": "settings_get" }))["choices"], json!({}));
    assert_eq!(
        h.announcements(3)[2].speech,
        "Toggles reset to your profile's defaults."
    );
    h.restart();
    assert_eq!(h.ok(&json!({ "command": "settings_get" }))["choices"], json!({}));
}

/// INFER (OD-30): a blind listener must not silence the app by accident.
#[test]
fn b03_t09_the_last_non_visual_output_cannot_be_turned_off() {
    let mut h = Harness::new("lockout");
    // With braille and speech off, sonification is the last non-visual output.
    assert_eq!(h.effective("interaction.modalities.braille").0, json!(true));
    h.toggle("braille").unwrap();
    h.announcements(1);
    h.toggle("speech").unwrap();
    h.announcements(2);
    let err = h.toggle("sonification").unwrap_err();
    assert_eq!(err.code, "SNR-APP-094");
    // Nothing changed, and the refusal is spoken although speech is off.
    assert_eq!(h.effective("interaction.modalities.sonification").0, json!(true));
    assert_eq!(h.ok(&json!({ "command": "transport_state" }))["muted"], false);
    let said = h.announcements(3);
    assert_eq!(
        said[2].speech,
        "Speech, sonification and braille cannot all be off. Turn another one on first."
    );
    assert!(said[2].assertive && !said[2].muted, "{said:?}");
    // With braille on, sonification can go.
    h.toggle("braille").unwrap();
    h.toggle("sonification").unwrap();
    assert_eq!(h.ok(&json!({ "command": "transport_state" }))["muted"], true);
}
