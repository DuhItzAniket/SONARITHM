// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration scenarios through the IPC protocol, headless, on the silent null device
//! (testing-strategy §10; B01-T01). They drive `App::dispatch_json` with the JSON a host
//! would forward, so they test the protocol a Tauri or Electron host exposes (OD-15).

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
use sonarithm_app::settings::{Fault, Settings, SettingsStore, Theme, Verbosity};
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};
use sonarithm_audio::NullConfig;

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n4,10.5\n5,11.0\n6,9.0\n7,8.0\n8,12.5\n";

fn spec(dataset: &str) -> String {
    json!({
        "specVersion": "0.1",
        "name": "Temperatures",
        "data": { "source": { "kind": "dataset", "id": dataset } },
        "substrate": { "time": { "field": "day" } },
        "tracks": [{ "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
    })
    .to_string()
}

struct Harness {
    app: App,
    sink: CollectingSink,
    dir: TempDir,
}

impl Harness {
    fn new(label: &str) -> Self {
        Self::with_audio(label, AudioOutput::null())
    }

    fn with_audio(label: &str, audio: AudioOutput) -> Self {
        let dir = TempDir::new(label);
        Self::in_dir(dir, audio)
    }

    fn in_dir(dir: TempDir, audio: AudioOutput) -> Self {
        let sink = CollectingSink::new();
        let config = AppConfig::new(dir.path().join("config")).with_audio(audio);
        let app = App::new(config, Arc::new(sink.clone())).unwrap();
        Self { app, sink, dir }
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

    fn state(&mut self) -> Value {
        self.ok(&json!({ "command": "transport_state" }))
    }

    /// Polls the transport until `done` holds, at most 5 s.
    fn wait_for(&mut self, done: impl Fn(&Value) -> bool) -> Value {
        let start = Instant::now();
        loop {
            let state = self.state();
            if done(&state) || start.elapsed() > Duration::from_secs(5) {
                return state;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Opens the CSV fixture and compiles a spec over it.
    fn open_and_compile(&mut self) -> Value {
        let csv = self.dir.path().join("temps.csv");
        std::fs::write(&csv, CSV).unwrap();
        let opened = self.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
        assert_eq!(opened["reply"], "dataset");
        assert_eq!(opened["id"], "d1");
        assert_eq!(opened["name"], "temps.csv");
        assert_eq!(opened["rows"], 8);
        self.ok(&json!({ "command": "spec_set", "args": { "text": spec("d1") } }));
        let report = self.ok(&json!({ "command": "compile" }));
        assert_eq!(report["reply"], "compile");
        assert_eq!(report["ok"], true, "{report}");
        report
    }
}

fn status(state: &Value) -> &str {
    state["status"].as_str().unwrap_or_default()
}

fn position(state: &Value) -> f64 {
    state["positionS"].as_f64().unwrap_or(-1.0)
}

#[test]
fn scenario_open_compile_play_seek_stop_export() {
    let mut h = Harness::new("scenario");
    let hello = h.ok(&json!({ "command": "hello", "args": { "protocol": 1 } }));
    assert_eq!(hello["protocol"], 1);

    let report = h.open_and_compile();
    let duration = report["score"]["durationS"].as_f64().unwrap();
    assert!(duration > 1.5, "the Score lasts {duration} s");

    let playing = h.ok(&json!({ "command": "transport_play" }));
    assert_eq!(status(&playing), "playing");
    assert_eq!(playing["device"], "null");
    let moved = h.wait_for(|s| position(s) > 0.1);
    assert!(position(&moved) > 0.1, "{moved}");

    let sought = h.ok(&json!({ "command": "transport_seek", "args": { "seconds": 1.0 } }));
    assert!((position(&sought) - 1.0).abs() < 1e-9);
    let after = h.wait_for(|s| position(s) >= 1.0);
    assert!(
        position(&after) >= 1.0 && position(&after) < duration,
        "{after}"
    );

    let stopped = h.ok(&json!({ "command": "transport_stop" }));
    assert_eq!(status(&stopped), "stopped");
    assert!(position(&stopped).abs() < 1e-9);

    let wav = h.dir.path().join("temps.wav");
    let job = h.ok(&json!({ "command": "export_render", "args": { "options": { "path": wav } } }));
    assert_eq!(job["reply"], "job");
    assert!(h.app.wait_for_jobs(Duration::from_secs(60)));
    let id = job["id"].clone();
    let done = h.ok(&json!({ "command": "job_status", "args": { "id": id } }));
    assert_eq!(done["state"], "finished", "{done}");
    assert_eq!(done["export"]["sampleRate"], 48_000);
    // Files are named alone, never with their folder (ADR-0017 §7).
    assert_eq!(done["export"]["audio"], "temps.wav");
    assert_eq!(done["export"]["manifest"], "temps.render.json");
    assert!(wav.is_file());
    assert!(h.dir.path().join("temps.render.json").is_file());
    assert!(h.dir.path().join("temps.cues.vtt").is_file());

    // Clock anchors reached the sink, in order.
    let seqs: Vec<u64> = h
        .sink
        .events()
        .iter()
        .filter_map(|event| match event {
            Event::Clock(anchor) => Some(anchor.seq),
            _ => None,
        })
        .collect();
    assert!(seqs.len() >= 3, "{seqs:?}");
    assert!(seqs.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn a11y_15_playback_starts_only_on_a_play_command() {
    let mut h = Harness::new("a11y15");
    h.open_and_compile();
    std::thread::sleep(Duration::from_millis(100));
    let state = h.state();
    assert_eq!(status(&state), "stopped");
    // The device opens on the first play (09 §7), so nothing could have sounded.
    assert_eq!(state["device"], Value::Null);

    // Seeking, speed and loop changes do not start playback either.
    h.ok(&json!({ "command": "transport_seek", "args": { "seconds": 0.5 } }));
    h.ok(&json!({ "command": "transport_set_speed", "args": { "speed": 2.0 } }));
    let state = h.ok(&json!({ "command": "transport_set_loop", "args": { "range": { "startS": 0.0, "endS": 1.0 } } }));
    assert_eq!(status(&state), "paused");
    assert_eq!(state["device"], Value::Null);

    // A recompile while playing pauses; it never continues into the new Score.
    h.ok(&json!({ "command": "transport_play" }));
    h.ok(&json!({ "command": "compile" }));
    let state = h.wait_for(|s| status(s) != "playing");
    assert_eq!(status(&state), "paused");
}

#[test]
fn stop_all_pauses_playback() {
    let mut h = Harness::new("stopall");
    h.open_and_compile();
    h.ok(&json!({ "command": "transport_play" }));
    let state = h.ok(&json!({ "command": "stop_all" }));
    assert_eq!(status(&state), "paused");
}

#[test]
fn playback_stops_at_the_end_and_reports_it() {
    let mut fast = NullConfig::new(48_000, 256);
    fast.paced = false;
    let mut h = Harness::with_audio("finish", AudioOutput::Null(fast));
    h.open_and_compile();
    h.ok(&json!({ "command": "transport_play" }));
    let state = h.wait_for(|s| status(s) == "stopped");
    assert_eq!(status(&state), "stopped");
    assert!(
        h.sink
            .events()
            .iter()
            .any(|event| matches!(event, Event::PlaybackFinished { .. }))
    );
    // Playing again starts from the beginning.
    let again = h.ok(&json!({ "command": "transport_play" }));
    assert_eq!(status(&again), "playing");
}

#[test]
fn device_loss_pauses_at_the_same_position() {
    let mut h = Harness::new("unplug");
    h.open_and_compile();
    h.ok(&json!({ "command": "transport_play" }));
    h.wait_for(|s| position(s) > 0.2);
    h.app.unplug_audio();
    let state = h.wait_for(|s| status(s) == "paused");
    assert_eq!(status(&state), "paused", "{state}");
    assert!(position(&state) > 0.2);
    assert!(
        h.sink
            .events()
            .iter()
            .any(|event| matches!(event, Event::DeviceChanged { .. }))
    );
    // HS-05, B01-T08: the change is said in words, assertively, for the log and the screen
    // reader. The sequencer delivers it on its next tick.
    let start = Instant::now();
    let said = loop {
        let found = h.sink.events().into_iter().find_map(|event| match event {
            Event::Announcement(info) if info.speech.starts_with("Audio device changed to") => {
                Some(info)
            }
            _ => None,
        });
        if let Some(info) = found {
            break info;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "no device announcement"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(said.assertive, "{said:?}");
    assert!(
        said.speech.ends_with("Playback is paused."),
        "{}",
        said.speech
    );
}

#[test]
fn transport_values_out_of_range_are_refused() {
    let mut h = Harness::new("ranges");
    let err = h.send(&json!({ "command": "transport_play" })).unwrap_err();
    assert_eq!(err.code, "SNR-APP-004");
    h.open_and_compile();
    for request in [
        json!({ "command": "transport_set_speed", "args": { "speed": 8.0 } }),
        json!({ "command": "transport_seek", "args": { "seconds": -1.0 } }),
        json!({ "command": "transport_set_loop", "args": { "range": { "startS": 2.0, "endS": 1.0 } } }),
    ] {
        assert_eq!(h.send(&request).unwrap_err().code, "SNR-APP-005");
    }
    // A seek past the end lands on the end.
    let state = h.ok(&json!({ "command": "transport_seek", "args": { "seconds": 1.0e6 } }));
    assert!((position(&state) - state["durationS"].as_f64().unwrap()).abs() < 1e-9);
}

#[test]
fn spec_edits_undo_and_redo_through_the_bus() {
    let mut h = Harness::new("undo");
    let first = h.ok(&json!({ "command": "spec_set", "args": { "text": spec("d1") } }));
    assert_eq!(first["undoToken"], 1);
    let second = h.ok(&json!({ "command": "spec_set", "args": { "text": spec("d2") } }));
    assert_eq!(second["undoToken"], 2);
    let undone = h.ok(&json!({ "command": "history_undo" }));
    assert_eq!(undone["token"], 2);
    assert_eq!(undone["spec"]["text"], spec("d1"));
    assert_eq!(undone["canRedo"], true);
    let redone = h.ok(&json!({ "command": "history_redo" }));
    assert_eq!(redone["spec"]["text"], spec("d2"));
    h.ok(&json!({ "command": "history_undo" }));
    let empty = h.ok(&json!({ "command": "history_undo" }));
    assert_eq!(empty["spec"]["text"], Value::Null);
    assert_eq!(empty["canUndo"], false);
    let nothing = h.ok(&json!({ "command": "history_undo" }));
    assert_eq!(nothing["token"], Value::Null);
}

#[test]
fn spec_problems_are_diagnostics_not_errors() {
    let mut h = Harness::new("diagnostics");
    let err = h.send(&json!({ "command": "compile" })).unwrap_err();
    assert_eq!(err.code, "SNR-APP-003");
    let err = h
        .send(&json!({ "command": "spec_set", "args": { "text": "{ not json" } }))
        .unwrap_err();
    assert!(err.code.starts_with("SNR-SPEC-"), "{err}");
    // The dataset is not open: a compile diagnostic, and no Score.
    h.ok(&json!({ "command": "spec_set", "args": { "text": spec("d9") } }));
    let report = h.ok(&json!({ "command": "compile" }));
    assert_eq!(report["ok"], false);
    assert!(report["counts"]["errors"].as_u64().unwrap() >= 1);
    assert_eq!(report["score"], Value::Null);
}

#[test]
fn datasets_open_list_and_close() {
    let mut h = Harness::new("datasets");
    h.open_and_compile();
    let csv = h.dir.path().join("temps.csv");
    let second = h.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
    assert_eq!(second["id"], "d2");
    let fields: Vec<&str> = second["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert_eq!(fields, ["day", "temp"]);
    let listed = h.ok(&json!({ "command": "dataset_close", "args": { "id": "d1" } }));
    assert_eq!(listed["datasets"].as_array().map(Vec::len), Some(1));
    let err = h
        .send(&json!({ "command": "dataset_close", "args": { "id": "d1" } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-002");
    let missing = h.dir.path().join("missing.csv");
    let err = h
        .send(&json!({ "command": "dataset_open", "args": { "path": missing } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-060", "{err}");
    // Only the file name reaches the UI; never the folder (P8).
    assert!(
        !err.message.contains(&*h.dir.path().to_string_lossy()),
        "{err}"
    );
    // Directories and files that are not data are refused unread.
    let notes = h.dir.path().join("notes.txt");
    std::fs::write(&notes, "hello").unwrap();
    let folder = h.dir.path().join("folder.csv");
    std::fs::create_dir_all(&folder).unwrap();
    for path in [notes, folder] {
        let err = h
            .send(&json!({ "command": "dataset_open", "args": { "path": path } }))
            .unwrap_err();
        assert_eq!(err.code, "SNR-APP-060", "{err}");
    }
}

/// C01-T12: the open-dataset path accepts a chart file. The reply carries the chart's long
/// table and the summary sentence of 10 §5 (`SNR-ENG-385`); a spec with a `dataset` source
/// plays it through its chart-type template; a chart file with no chart is refused.
#[test]
fn a_chart_opens_as_a_dataset_headless() {
    let mut h = Harness::new("chart");
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engines/sonarithm-plotsona/tests/corpus/svg/vl-line.svg");
    let svg = h.dir.path().join("rain.svg");
    std::fs::copy(corpus, &svg).unwrap();
    let opened = h.ok(&json!({ "command": "dataset_open", "args": { "path": svg } }));
    assert_eq!(opened["name"], "rain.svg");
    assert_eq!(opened["rows"], 36);
    let fields: Vec<&str> = opened["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert_eq!(fields, ["series", "x", "y"]);
    let summary = opened["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "SNR-ENG-385")
        .unwrap_or_else(|| panic!("{opened}"));
    assert!(
        summary["message"]
            .as_str()
            .unwrap()
            .starts_with("rain.svg: 36 points: 0 exact, 36 calibrated, 0 estimated"),
        "{summary}"
    );
    let spec = json!({
        "specVersion": "0.2",
        "data": { "source": { "kind": "dataset", "id": "d1" } },
        "template": { "id": "chart-line" }
    });
    h.ok(&json!({ "command": "spec_set", "args": { "text": spec.to_string() } }));
    let report = h.ok(&json!({ "command": "compile" }));
    assert_eq!(report["ok"], true, "{report}");
    let broken = h.dir.path().join("broken.svg");
    std::fs::write(&broken, "not a chart").unwrap();
    let err = h
        .send(&json!({ "command": "dataset_open", "args": { "path": broken } }))
        .unwrap_err();
    assert!(err.code.starts_with("SNR-ENG-3"), "{err}");
}

#[test]
fn malformed_requests_and_versions_fail_fast() {
    let mut h = Harness::new("protocol");
    let err = h
        .app
        .dispatch_json("{\"command\": \"warp_drive\"}")
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-009");
    let err = h
        .send(&json!({ "command": "hello", "args": { "protocol": 2 } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-001");
    assert!(err.message.contains("protocol 2") && err.message.contains("protocol 1"));
}

#[test]
fn export_paths_are_checked() {
    let mut h = Harness::new("export-path");
    h.open_and_compile();
    let folder = h.dir.path().to_path_buf();
    for path in [
        "relative.wav".to_owned(),
        folder.join("x.mp3").to_string_lossy().into_owned(),
        folder
            .join("nope")
            .join("x.wav")
            .to_string_lossy()
            .into_owned(),
    ] {
        let err = h
            .send(&json!({ "command": "export_render", "args": { "options": { "path": path } } }))
            .unwrap_err();
        assert_eq!(err.code, "SNR-APP-021", "{path}");
    }
}

#[test]
fn settings_round_trip_and_device_selection() {
    let mut h = Harness::new("settings");
    let mut settings: Value = h.ok(&json!({ "command": "settings_get" }));
    assert_eq!(settings["settingsVersion"], 1);
    settings.as_object_mut().unwrap().remove("reply");
    settings["theme"] = json!("dark");
    settings["audio"]["volumeDb"] = json!(-6.0);
    let saved =
        h.ok(&json!({ "command": "settings_set", "args": { "settings": settings.clone() } }));
    assert_eq!(saved["theme"], "dark");
    settings["audio"]["volumeDb"] = json!(3.0);
    let err = h
        .send(&json!({ "command": "settings_set", "args": { "settings": settings } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-013");

    let devices = h.ok(&json!({ "command": "devices_list" }));
    assert_eq!(devices["devices"][0]["id"], "null");
    assert_eq!(devices["devices"][0]["selected"], true);
    h.ok(&json!({ "command": "device_select", "args": { "id": "null" } }));
    let err = h
        .send(&json!({ "command": "device_select", "args": { "id": "speakers" } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-006");

    // A new app over the same folder reads what was saved.
    let Harness { app, dir, .. } = h;
    drop(app);
    let mut again = Harness::in_dir(dir, AudioOutput::null());
    let settings = again.ok(&json!({ "command": "settings_get" }));
    assert_eq!(settings["theme"], "dark");
    assert_eq!(settings["audio"]["device"], "null");
    assert_eq!(settings["audio"]["volumeDb"], -6.0);
}

#[test]
fn settings_migrate_from_the_v0_fixture() {
    let dir = TempDir::new("migrate");
    let config = dir.path().join("config");
    std::fs::create_dir_all(&config).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/settings-v0.json");
    std::fs::copy(fixture, config.join("settings.json")).unwrap();

    let mut store = SettingsStore::new(&config);
    let loaded = store.load();
    assert_eq!(loaded.migrated_from, Some(0));
    assert_eq!(loaded.notices, Vec::new());
    let settings = loaded.settings;
    assert_eq!(settings.settings_version, 1);
    assert_eq!(settings.theme, Theme::HighContrastDark);
    assert_eq!(settings.verbosity, Verbosity::Verbose);
    assert_eq!(settings.audio.device.as_deref(), Some("null"));
    assert_eq!(settings.audio.buffer_frames, Some(512));
    assert!((settings.audio.volume_db + 12.5).abs() < 1e-12);
    assert_eq!(
        settings.keymap.get("transport.play").map(String::as_str),
        Some("Ctrl+Space")
    );

    // The file is rewritten in the current version, so the next start reads it as is.
    let again = SettingsStore::new(&config).load();
    assert_eq!(again.migrated_from, Some(1));
    assert_eq!(again.settings, settings);
}

#[test]
fn an_interrupted_write_leaves_the_previous_settings_readable() {
    for fault in [Fault::PartialTemp, Fault::BeforeRename] {
        let dir = TempDir::new("fault");
        let mut store = SettingsStore::new(dir.path());
        let mut first = Settings::default();
        first.theme = Theme::Light;
        store.save(&first).unwrap();

        let mut second = first.clone();
        second.theme = Theme::Dark;
        store.inject_fault(Some(fault));
        assert_eq!(store.save(&second).unwrap_err().code, "SNR-APP-010");

        // "Restart": a fresh store reads the previous settings, without a notice.
        let loaded = SettingsStore::new(dir.path()).load();
        assert_eq!(loaded.settings.theme, Theme::Light, "{fault:?}");
        assert_eq!(loaded.notices, Vec::new());

        // The next good write replaces the stray temporary file.
        store.inject_fault(None);
        store.save(&second).unwrap();
        assert_eq!(
            SettingsStore::new(dir.path()).load().settings.theme,
            Theme::Dark
        );
    }
}

#[test]
fn a_damaged_settings_file_falls_back_to_the_last_good_copy() {
    let dir = TempDir::new("damaged");
    let store = SettingsStore::new(dir.path());
    let mut good = Settings::default();
    good.theme = Theme::Light;
    store.save(&good).unwrap();
    let mut newer = good.clone();
    newer.theme = Theme::Dark;
    store.save(&newer).unwrap(); // the light copy is now the backup
    std::fs::write(store.path(), b"{ \"settingsVersion\": 1, \"aud").unwrap();

    let sink = CollectingSink::new();
    let app = App::new(
        AppConfig::new(dir.path()).with_audio(AudioOutput::null()),
        Arc::new(sink.clone()),
    )
    .unwrap();
    drop(app);
    let notices: Vec<String> = sink
        .events()
        .into_iter()
        .filter_map(|event| match event {
            Event::SettingsNotice { error } => Some(error.code),
            _ => None,
        })
        .collect();
    assert_eq!(notices, ["SNR-APP-011"]);
    // The backup was restored as the main file.
    assert_eq!(
        SettingsStore::new(dir.path()).load().settings.theme,
        Theme::Light
    );
}

/// ADR-0017 §7: no reply, error or event names the folder of a file the user chose.
#[test]
fn adr_0017_no_reply_error_or_event_holds_an_absolute_path() {
    let mut h = Harness::new("privacy");
    let folder = h
        .dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let mut seen: Vec<String> = Vec::new();
    seen.push(h.open_and_compile().to_string());
    let wav = h.dir.path().join("out.wav");
    let missing = h.dir.path().join("gone").join("x.csv");
    let bad_folder = h.dir.path().join("gone").join("x.wav");
    for request in [
        json!({ "command": "dataset_list" }),
        json!({ "command": "export_render", "args": { "options": { "path": wav } } }),
        json!({ "command": "dataset_open", "args": { "path": missing } }),
        json!({ "command": "export_render", "args": { "options": { "path": bad_folder } } }),
        json!({ "command": "transport_play" }),
        json!({ "command": "transport_stop" }),
    ] {
        match h.send(&request) {
            Ok(reply) => seen.push(reply.to_string()),
            Err(err) => seen.push(serde_json::to_string(&err).unwrap()),
        }
    }
    assert!(h.app.wait_for_jobs(Duration::from_secs(60)));
    seen.push(h.ok(&json!({ "command": "job_list" })).to_string());
    for event in h.sink.events() {
        seen.push(serde_json::to_string(&event).unwrap());
    }
    for text in &seen {
        assert!(!text.contains(&folder), "a folder leaked: {text}");
    }
}

fn anchors(h: &Harness) -> usize {
    h.sink
        .events()
        .iter()
        .filter(|event| matches!(event, Event::Clock(_)))
        .count()
}

#[test]
fn b01_t04_anchors_come_on_changes_not_on_every_tick() {
    let mut h = Harness::new("anchors");
    h.open_and_compile();
    h.ok(&json!({ "command": "transport_set_loop", "args": { "range": { "startS": 0.0, "endS": 1.0 } } }));
    h.ok(&json!({ "command": "transport_play" }));
    let after_play = anchors(&h);
    // Nothing changes for half a second, well inside the 2 s period: no anchor.
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(anchors(&h), after_play);
    // A seek is a change: exactly one anchor.
    h.ok(&json!({ "command": "transport_seek", "args": { "seconds": 0.25 } }));
    assert_eq!(anchors(&h), after_play + 1);
    // B04-T04: the null device reports no output latency, so the anchors say so (null),
    // and the playhead shows the position uncorrected.
    let json = serde_json::to_value(h.sink.events().iter().find_map(|event| match event {
        Event::Clock(anchor) => Some(anchor.clone()),
        _ => None,
    }))
    .unwrap();
    assert_eq!(json["outputLatencyS"], Value::Null, "{json}");
    assert!(
        json.as_object()
            .is_some_and(|o| o.contains_key("outputLatencyS"))
    );
}

#[test]
fn b04_t04_a_stored_latency_replaces_the_backends_on_the_anchors() {
    let mut h = Harness::new("calibrated");
    h.open_and_compile();
    // ADR-0016: a stored latency for the open device (the null device, the system default
    // here) comes before the backend's report, which the null device does not give.
    let mut settings: Value = h.ok(&json!({ "command": "settings_get" }));
    settings.as_object_mut().unwrap().remove("reply");
    settings["audio"]["calibratedLatencyS"] = json!({ "null": 0.071 });
    h.ok(&json!({ "command": "settings_set", "args": { "settings": settings.clone() } }));
    h.ok(&json!({ "command": "transport_set_loop", "args": { "range": { "startS": 0.0, "endS": 1.0 } } }));
    h.ok(&json!({ "command": "transport_play" }));
    let latest = |h: &Harness| {
        h.sink.events().iter().rev().find_map(|event| match event {
            Event::Clock(anchor) => Some(anchor.output_latency_s),
            _ => None,
        })
    };
    assert_eq!(latest(&h), Some(Some(0.071)));
    // Removing it while playing sends a new anchor at once, back to no correction.
    settings["audio"]["calibratedLatencyS"] = json!({});
    let before = anchors(&h);
    h.ok(&json!({ "command": "settings_set", "args": { "settings": settings } }));
    assert!(anchors(&h) > before);
    assert_eq!(latest(&h), Some(None));
}

#[test]
fn b01_t04_meters_are_binary_frames_at_most_30_a_second_while_playing() {
    use sonarithm_app::ipc::meters::MeterFrame;
    let mut h = Harness::new("meters");
    h.open_and_compile();
    h.ok(&json!({ "command": "transport_set_loop", "args": { "range": { "startS": 0.0, "endS": 1.0 } } }));
    let start = Instant::now();
    h.ok(&json!({ "command": "transport_play" }));
    std::thread::sleep(Duration::from_millis(700));
    h.ok(&json!({ "command": "transport_pause" }));
    let elapsed = start.elapsed().as_secs_f64();
    // Let the control thread send the last frame after the pause, then nothing more.
    std::thread::sleep(Duration::from_millis(200));
    let frames = h.sink.meter_frames();
    let Some((last, playing)) = frames.split_last() else {
        panic!("no meter frames");
    };
    #[expect(clippy::cast_precision_loss, reason = "a handful of frames")]
    let count = playing.len() as f64;
    assert!(
        count <= elapsed * 30.0 + 1.0,
        "{count} frames in {elapsed} s"
    );
    assert!(playing.len() >= 5, "{} frames", playing.len());
    assert!(frames.windows(2).all(|w| w[0].seq < w[1].seq));
    assert!(
        playing.iter().any(|frame| frame.peak[0] > 0.0),
        "the tones are metered"
    );
    assert!(playing.iter().all(|frame| !frame.last));
    // The pause sent one last frame with zero peaks, so the meters fall to silence.
    assert!(last.last && last.peak == [0.0, 0.0], "{last:?}");
    // Every frame survives its binary encoding; none travels as a JSON event (09 §2).
    for frame in &frames {
        assert_eq!(MeterFrame::decode(&frame.encode()).ok(), Some(*frame));
    }
    assert!(
        h.sink
            .events()
            .iter()
            .all(|event| event.channel() != sonarithm_app::ipc::Channel::Meters)
    );
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(
        h.sink.meter_frames().len(),
        frames.len(),
        "paused: no more frames"
    );
}

#[test]
fn b01_t07_start_up_never_waits_for_an_audio_driver() {
    // A driver that takes 2 s for every call (09 §7): the app core starts, loads its
    // settings and answers without touching it, and only the first play opens the device.
    let delay = Duration::from_secs(2);
    let mut slow = NullConfig::new(48_000, 256);
    slow.driver_delay = delay;
    let started = Instant::now();
    let mut h = Harness::with_audio("slowdriver", AudioOutput::Null(slow));
    h.ok(&json!({ "command": "hello", "args": { "protocol": 1 } }));
    h.ok(&json!({ "command": "settings_get" }));
    let state = h.state();
    let start_up = started.elapsed();
    assert!(start_up < delay, "start-up took {start_up:?}");
    assert_eq!(
        state["device"],
        Value::Null,
        "no device is open before a play"
    );

    h.open_and_compile();
    let play = Instant::now();
    let state = h.ok(&json!({ "command": "transport_play" }));
    assert_eq!(status(&state), "playing");
    assert!(play.elapsed() >= delay, "the first play opens the device");
}
