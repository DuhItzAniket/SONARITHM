// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Profiles through the app core (ADR-0020; B03-T08): presets, import and export, the
//! layers of 12 §3, the one-key flip (SDR-15), every personalization field (SDR-38), the
//! project profile, and a Score that re-renders from its manifest without the profile.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use sonarithm::{Datasets, Sonarithm, Spec};
use sonarithm_app::ipc::IpcError;
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, NullSink};
use sonarithm_core::TableId;
use sonarithm_data::{BundleId, ReadOptions, read_file};

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n4,10.5\n5,11.0\n6,9.0\n7,8.0\n8,12.5\n";

fn spec(extra_data: &Value) -> String {
    let mut data = json!({ "source": { "kind": "dataset", "id": "d1" } });
    if let (Some(data), Some(extra)) = (data.as_object_mut(), extra_data.as_object()) {
        for (k, v) in extra {
            data.insert(k.clone(), v.clone());
        }
    }
    json!({
        "specVersion": "0.1",
        "data": data,
        "substrate": { "time": { "field": "day" } },
        "tracks": [{ "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
    })
    .to_string()
}

fn profile(sound: &Value, interaction: &Value) -> Value {
    json!({
        "format": "sonarithm-profile", "formatVersion": "1.0", "name": "Test listener",
        "sound": sound, "interaction": interaction
    })
}

struct Harness {
    app: App,
    dir: Arc<TempDir>,
}

impl Harness {
    fn new(label: &str) -> Self {
        Self::in_dir(Arc::new(TempDir::new(label)))
    }

    fn in_dir(dir: Arc<TempDir>) -> Self {
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let app = App::new(config, Arc::new(NullSink)).unwrap();
        Self { app, dir }
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.send(request)
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }

    fn csv(&self) -> std::path::PathBuf {
        let csv = self.dir.path().join("temps.csv");
        std::fs::write(&csv, CSV).unwrap();
        csv
    }

    /// Opens the data, sets `text` as the spec and compiles.
    fn compile(&mut self, text: &str) -> Value {
        let csv = self.csv();
        if self.ok(&json!({ "command": "dataset_list" }))["datasets"]
            .as_array()
            .is_none_or(Vec::is_empty)
        {
            self.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
        }
        self.ok(&json!({ "command": "spec_set", "args": { "text": text } }));
        let report = self.ok(&json!({ "command": "compile" }));
        assert_eq!(report["ok"], true, "{report}");
        report
    }

    /// Saves `profile` as a new user profile and activates it; its ID.
    fn activate(&mut self, profile: &Value) -> String {
        let before: Vec<String> = ids(&self.ok(&json!({ "command": "profile_list" })));
        let after = self.ok(&json!({ "command": "profile_save", "args": { "profile": profile } }));
        let id = ids(&after)
            .into_iter()
            .find(|id| !before.contains(id))
            .unwrap();
        self.ok(&json!({ "command": "profile_activate", "args": { "id": id } }));
        id
    }

    fn effective(&mut self, key: &str) -> (Value, String) {
        let reply = self.ok(&json!({ "command": "settings_effective" }));
        reply["settings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["key"] == key)
            .map_or((Value::Null, "default".to_owned()), |s| {
                (s["value"].clone(), s["layer"].as_str().unwrap().to_owned())
            })
    }

    /// Exports the compiled Score; the render manifest.
    fn render(&mut self) -> Value {
        let wav = self.dir.path().join("render.wav");
        self.ok(&json!({ "command": "export_render", "args": { "options": { "path": wav } } }));
        assert!(self.app.wait_for_jobs(Duration::from_secs(60)));
        let manifest = std::fs::read(self.dir.path().join("render.render.json")).unwrap();
        serde_json::from_slice(&manifest).unwrap()
    }
}

fn ids(list: &Value) -> Vec<String> {
    list["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn presets_are_listed_read_only_and_proposals() {
    let mut h = Harness::new("presets");
    let list = h.ok(&json!({ "command": "profile_list" }));
    let presets: Vec<&Value> = list["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["preset"] == true)
        .collect();
    assert_eq!(presets.len(), 5);
    assert_eq!(presets[0]["name"], "BLV chart reading");
    let changed = h.send(&json!({ "command": "profile_save", "args": {
        "id": "preset:analyst", "profile": profile(&json!({}), &json!({})) } }));
    assert_eq!(changed.unwrap_err().code, "SNR-APP-083");
    let unknown = h.send(&json!({ "command": "profile_activate", "args": { "id": "nobody" } }));
    assert_eq!(unknown.unwrap_err().code, "SNR-APP-083");
}

#[test]
fn a_profile_imports_and_exports_byte_for_byte() {
    let mut h = Harness::new("import");
    let file = h.dir.path().join("mine.json");
    let mine = profile(
        &json!({ "timbre": "square" }),
        &json!({ "verbosity": "verbose" }),
    );
    std::fs::write(&file, serde_json::to_vec_pretty(&mine).unwrap()).unwrap();
    let imported = h.ok(&json!({ "command": "profile_import", "args": { "path": file } }));
    assert!(ids(&imported).contains(&"test-listener".to_owned()));
    let out = h.dir.path().join("out.json");
    h.ok(&json!({ "command": "profile_export",
        "args": { "id": "test-listener", "path": out } }));
    let exported = std::fs::read(&out).unwrap();
    // Importing the export and exporting again gives the same bytes (canonical form).
    h.ok(&json!({ "command": "profile_import", "args": { "path": out } }));
    let again = h.dir.path().join("again.json");
    h.ok(&json!({ "command": "profile_export",
        "args": { "id": "test-listener-2", "path": again } }));
    assert_eq!(std::fs::read(&again).unwrap(), exported);

    let bad = h.dir.path().join("bad.json");
    std::fs::write(&bad, br#"{"format":"sonarithm-profile","formatVersion":"1.0","name":"x","sound":{"timbre":"/etc"}}"#).unwrap();
    let refused = h.send(&json!({ "command": "profile_import", "args": { "path": bad } }));
    assert_eq!(refused.unwrap_err().code, "SNR-APP-080");
}

/// SDR-15: the one-key flip is saved in the active profile and survives a restart.
#[test]
fn sdr_15_the_polarity_flip_persists_across_a_restart() {
    let dir = Arc::new(TempDir::new("flip"));
    let mut h = Harness::in_dir(Arc::clone(&dir));
    h.compile(&spec(&json!({})));
    assert_eq!(h.effective("sound.polarity.temp").1, "default");
    let report = h.ok(&json!({ "command": "profile_flip_polarity", "args": { "field": "temp" } }));
    assert_eq!(report["reply"], "compile");
    assert_eq!(report["ok"], true);
    assert_eq!(
        h.effective("sound.polarity.temp"),
        (json!("negative"), "profile".to_owned())
    );
    drop(h);

    let mut restarted = Harness::in_dir(dir);
    restarted.compile(&spec(&json!({})));
    assert_eq!(
        restarted.effective("sound.polarity.temp"),
        (json!("negative"), "profile".to_owned()),
        "the flip survived the restart"
    );
    // Flipping again restores the spec's polarity.
    restarted.ok(&json!({ "command": "profile_flip_polarity", "args": { "field": "temp" } }));
    assert_eq!(restarted.effective("sound.polarity.temp").1, "default");
}

#[test]
fn sdr_38_the_pitch_range_comes_from_the_profile() {
    let mut h = Harness::new("range");
    h.activate(&profile(
        &json!({ "pitchRange": { "hz": [300, 1200] } }),
        &json!({}),
    ));
    h.compile(&spec(&json!({})));
    let (range, layer) = h.effective("sound.pitchRange.temp");
    assert_eq!(layer, "profile");
    let hz: Vec<f64> = range["hz"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_f64)
        .collect();
    assert_eq!(hz, [300.0, 1200.0]);
    let manifest = h.render();
    assert_eq!(
        manifest["spec"]["embedded"]["tracks"][0]["channels"]["pitch"]["range"],
        json!({ "hz": [300, 1200] })
    );
}

#[test]
fn sdr_38_the_timbre_comes_from_the_profile() {
    let mut h = Harness::new("timbre");
    h.activate(&profile(&json!({ "timbre": "square" }), &json!({})));
    let report = h.compile(&spec(&json!({})));
    assert_eq!(report["score"]["tracks"][0]["instrument"], "square");
}

#[test]
fn sdr_38_the_rate_comes_from_the_profile() {
    let mut h = Harness::new("rate");
    let default = h.compile(&spec(&json!({})))["score"]["durationS"]
        .as_f64()
        .unwrap();
    h.activate(&profile(
        &json!({ "pace": { "pointsPerSecond": 1 } }),
        &json!({}),
    ));
    let slower = h.compile(&spec(&json!({})))["score"]["durationS"]
        .as_f64()
        .unwrap();
    assert!(slower > 2.0 * default, "{slower} s against {default} s");
}

#[test]
fn sdr_38_marks_follow_the_profile_in_quick_specs() {
    let mut h = Harness::new("marks");
    let csv = h.csv();
    h.ok(&json!({ "command": "dataset_open", "args": { "path": csv } }));
    for (marks, mark) in [("continuous", "glide"), ("discrete", "tone")] {
        h.activate(&profile(&json!({ "marks": marks }), &json!({})));
        let quick = h.ok(&json!({ "command": "spec_quick",
            "args": { "dataset": "d1", "x": "day", "y": "temp" } }));
        let text: Value = serde_json::from_str(quick["text"].as_str().unwrap()).unwrap();
        assert_eq!(text["tracks"][0]["mark"], mark, "{marks}");
        assert_eq!(
            text["data"]["source"],
            json!({ "kind": "dataset", "id": "d1" })
        );
    }
}

#[test]
fn sdr_38_polarity_by_concept_comes_from_the_profile() {
    let mut h = Harness::new("concept");
    h.activate(&profile(
        &json!({ "polarity": { "concepts": { "cost": "negative" } } }),
        &json!({}),
    ));
    h.compile(&spec(
        &json!({ "fields": { "temp": { "concept": "cost" } } }),
    ));
    assert_eq!(
        h.effective("sound.polarity.temp"),
        (json!("negative"), "profile".to_owned())
    );
}

/// 12 §3 through the app: settings < profile < project.
#[test]
fn interaction_settings_follow_the_layers() {
    let mut h = Harness::new("layers");
    let mut settings = h.ok(&json!({ "command": "settings_get" }));
    let settings = settings.as_object_mut().unwrap();
    settings.remove("reply");
    settings["verbosity"] = json!("verbose");
    h.ok(&json!({ "command": "settings_set", "args": { "settings": settings } }));
    assert_eq!(
        h.effective("interaction.verbosity"),
        (json!("verbose"), "settings".to_owned())
    );
    h.ok(&json!({ "command": "profile_activate", "args": { "id": "preset:analyst" } }));
    assert_eq!(
        h.effective("interaction.verbosity"),
        (json!("terse"), "profile".to_owned())
    );
    h.ok(
        &json!({ "command": "profile_save", "args": { "id": "project",
        "profile": profile(&json!({}), &json!({ "verbosity": "verbose" })) } }),
    );
    assert_eq!(
        h.effective("interaction.verbosity"),
        (json!("verbose"), "project".to_owned())
    );
}

#[test]
fn the_project_profile_travels_in_the_bundle() {
    let dir = Arc::new(TempDir::new("projectprofile"));
    let mut h = Harness::in_dir(Arc::clone(&dir));
    h.compile(&spec(&json!({})));
    h.ok(
        &json!({ "command": "profile_save", "args": { "id": "project",
        "profile": profile(&json!({ "timbre": "sine" }), &json!({})) } }),
    );
    let bundle = dir.path().join("p.sonarithm");
    h.ok(&json!({ "command": "project_save_as", "args": { "path": bundle, "embedData": true } }));
    drop(h);
    let mut other = Harness::in_dir(dir);
    other.ok(&json!({ "command": "project_open", "args": { "path": bundle } }));
    assert!(ids(&other.ok(&json!({ "command": "profile_list" }))).contains(&"project".to_owned()));
    let report = other.ok(&json!({ "command": "compile" }));
    assert_eq!(report["score"]["tracks"][0]["instrument"], "sine");
    assert_eq!(other.effective("sound.timbre.temp").1, "project");
}

/// Keyboard §13 (B03-T09): keys in a project's profile are never applied, and opening the
/// project says so (`SNR-APP-092`), while the rest of that profile still applies.
#[test]
fn keyboard_13_a_projects_keys_are_ignored_with_a_notice() {
    let dir = Arc::new(TempDir::new("projectkeys"));
    let mut h = Harness::in_dir(Arc::clone(&dir));
    h.compile(&spec(&json!({})));
    h.ok(
        &json!({ "command": "profile_save", "args": { "id": "project",
        "profile": profile(&json!({}), &json!({ "verbosity": "verbose", "keymap": { "nav.next": "L" } })) } }),
    );
    let bundle = dir.path().join("keys.sonarithm");
    h.ok(&json!({ "command": "project_save_as", "args": { "path": bundle, "embedData": true } }));
    drop(h);
    let sink = sonarithm_app::CollectingSink::new();
    let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
    let mut app = App::new(config, Arc::new(sink.clone())).unwrap();
    let open = json!({ "command": "project_open", "args": { "path": bundle } });
    app.dispatch_json(&open.to_string()).unwrap();
    let notices: Vec<String> = sink
        .events()
        .into_iter()
        .filter_map(|event| match event {
            sonarithm_app::ipc::Event::SettingsNotice { error } => Some(error.code),
            _ => None,
        })
        .collect();
    assert_eq!(notices, ["SNR-APP-092"]);
    let effective = app
        .dispatch_json(&json!({ "command": "settings_effective" }).to_string())
        .unwrap();
    let keys: Vec<&Value> = effective["settings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| {
            s["key"]
                .as_str()
                .is_some_and(|k| k.starts_with("interaction.keymap."))
        })
        .collect();
    assert!(keys.is_empty(), "{keys:?}");
    let verbosity = effective["settings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["key"] == "interaction.verbosity")
        .cloned();
    assert_eq!(
        verbosity.map(|s| s["layer"].clone()),
        Some(json!("project"))
    );
}

/// ADR-0020's acceptance condition: a Score compiled with a profile re-renders from the
/// spec its manifest records, with no profile at all, to the same Score hash.
#[test]
fn adr_0020_a_score_re_renders_from_its_manifest_without_the_profile() {
    let mut h = Harness::new("reproduce");
    h.activate(&profile(
        &json!({
            "pitchRange": { "hz": [300, 1200] },
            "timbre": "square",
            "pace": { "pointsPerSecond": 2 },
            "polarity": { "concepts": { "cost": "negative" }, "flippedFields": ["temp"] },
            "context": { "ticks": true, "referenceTone": false, "spokenLegend": false }
        }),
        &json!({}),
    ));
    let report = h.compile(&spec(
        &json!({ "fields": { "temp": { "concept": "cost" } } }),
    ));
    let manifest = h.render();
    let embedded = manifest["spec"]["embedded"].clone();
    // The flip inverted the profile's negative: explicit in the recorded spec.
    assert_eq!(
        embedded["tracks"][0]["channels"]["pitch"]["polarity"],
        "positive"
    );

    // The same data, read as the app reads it (dataset d1 is table 1), and no profile.
    let options = ReadOptions::new().with_ids(BundleId(1), TableId(1));
    let csv = std::fs::canonicalize(h.dir.path().join("temps.csv")).unwrap();
    let bundle = read_file(&csv, None, &options).unwrap().bundle;
    let snr = Sonarithm::builder()
        .with_default_engines()
        .with_navigation()
        .build()
        .unwrap();
    let spec = Spec::from_json_str(embedded.to_string()).unwrap();
    let again = snr
        .compile_with(&spec, &Datasets::new().with("d1", bundle))
        .unwrap();
    assert_eq!(
        Some(again.score_hash().unwrap()),
        report["score"]["scoreHash"].as_str().map(str::to_owned)
    );
}
