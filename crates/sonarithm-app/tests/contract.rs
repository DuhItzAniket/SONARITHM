// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The IPC contract (ADR-0011, ADR-0017): every command answers with the reply kind the
//! generated `REPLY_KINDS` table promises (`ipc::tests` checks `EVENT_CHANNELS`). The
//! TypeScript client relies on both tables.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use sonarithm_app::ipc::binary::View;
use sonarithm_app::ipc::{BINARY_COMMANDS, COMMANDS, Request};
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

const CSV: &str = "day,temp\n1,8.5\n2,9.25\n3,7.0\n4,10.5\n";

/// One request per command, in an order where each can succeed.
fn requests(dir: &TempDir) -> Vec<Value> {
    let csv = dir.path().join("t.csv");
    std::fs::write(&csv, CSV).unwrap();
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "dataset", "id": "d1" } },
        "substrate": { "time": { "field": "day" } },
        "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
    })
    .to_string();
    let live = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "stream", "transport": "tail", "path": csv } },
        "substrate": { "time": { "field": "day", "pace": { "secondsPerUnit": 1 } } },
        "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
    })
    .to_string();
    let wav = dir.path().join("t.wav");
    let project = dir.path().join("t.sonarithm");
    let exported = dir.path().join("contract.json");
    // A chart for the correction commands (C02-T06).
    let chart = dir.path().join("rain.json");
    std::fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../engines/sonarithm-plotsona/tests/corpus/chartjson/vl-line.json"),
        &chart,
    )
    .unwrap();
    let corrections = dir.path().join("rain.corrections.json");
    vec![
        json!({ "command": "hello", "args": { "protocol": 1 } }),
        json!({ "command": "dataset_open", "args": { "path": csv } }),
        json!({ "command": "dataset_list" }),
        json!({ "command": "spec_set", "args": { "text": spec } }),
        json!({ "command": "spec_get" }),
        json!({ "command": "spec_apply", "args": { "patch": [
            { "op": "replace", "path": "/tracks/0/id", "value": "t" }
        ], "label": "Rename track" } }),
        json!({ "command": "spec_validate", "args": { "text": "{}" } }),
        json!({ "command": "compile" }),
        json!({ "command": "transport_state" }),
        json!({ "command": "transport_seek", "args": { "seconds": 0.5 } }),
        json!({ "command": "transport_set_speed", "args": { "speed": 1.5 } }),
        json!({ "command": "transport_set_loop", "args": { "range": null } }),
        json!({ "command": "transport_play" }),
        json!({ "command": "transport_pause" }),
        json!({ "command": "announce_repeat" }),
        json!({ "command": "nav_move", "args": { "verb": { "verb": "into" } } }),
        json!({ "command": "nav_where" }),
        json!({ "command": "stream_offer", "args": { "transport": "tail", "path": csv } }),
        // The first offer's token.
        json!({ "command": "stream_connect", "args": { "token": 1 } }),
        json!({ "command": "stream_list" }),
        json!({ "command": "spec_set", "args": { "text": live } }),
        json!({ "command": "live_start", "args": { "stream": "s1" } }),
        json!({ "command": "live_stop" }),
        json!({ "command": "spec_set", "args": { "text": spec } }),
        json!({ "command": "stream_hide", "args": { "id": "s1" } }),
        json!({ "command": "stream_pause", "args": { "id": "s1" } }),
        json!({ "command": "stream_resume", "args": { "id": "s1" } }),
        json!({ "command": "stream_stop", "args": { "id": "s1" } }),
        json!({ "command": "stop_all" }),
        json!({ "command": "transport_stop" }),
        json!({ "command": "export_render", "args": { "options": { "path": wav, "cues": false, "preserveLevel": false } } }),
        json!({ "command": "job_status", "args": { "id": 1 } }),
        json!({ "command": "job_cancel", "args": { "id": 1 } }),
        json!({ "command": "job_list" }),
        json!({ "command": "history_undo" }),
        json!({ "command": "history_redo" }),
        json!({ "command": "settings_get" }),
        json!({ "command": "devices_list" }),
        json!({ "command": "device_select", "args": { "id": null } }),
        // The first edit of the session wrote autosave 1; saving discards autosaves.
        json!({ "command": "project_recovery" }),
        json!({ "command": "project_recover", "args": { "id": 1 } }),
        json!({ "command": "project_discard_recovery" }),
        json!({ "command": "project_save_as", "args": { "path": project, "embedData": true } }),
        json!({ "command": "project_save" }),
        json!({ "command": "project_recent" }),
        json!({ "command": "project_open_recent", "args": { "index": 0 } }),
        json!({ "command": "project_open", "args": { "path": project } }),
        json!({ "command": "profile_list" }),
        json!({ "command": "profile_get", "args": { "id": "preset:analyst" } }),
        json!({ "command": "profile_save", "args": { "profile": {
            "format": "sonarithm-profile", "formatVersion": "1.0", "name": "Contract" } } }),
        json!({ "command": "profile_activate", "args": { "id": "contract" } }),
        json!({ "command": "profile_export", "args": { "id": "contract", "path": exported } }),
        json!({ "command": "profile_import", "args": { "path": exported } }),
        json!({ "command": "profile_flip_polarity", "args": { "field": "temp" } }),
        json!({ "command": "settings_effective" }),
        json!({ "command": "modality_toggle", "args": { "modality": "visual" } }),
        json!({ "command": "choices_reset" }),
        json!({ "command": "spec_quick", "args": { "dataset": "d1", "x": "day", "y": "temp" } }),
        json!({ "command": "cursor_set", "args": { "dataset": "d1", "row": 0, "origin": "table" } }),
        json!({ "command": "cursor_get" }),
        json!({ "command": "dataset_open", "args": { "path": chart } }),
        json!({ "command": "corrections_get", "args": { "dataset": "d2" } }),
        json!({ "command": "corrections_apply", "args": { "dataset": "d2", "patch": [{ "op": "add", "path": "/edits/-", "value": {
            "op": "move", "id": "e1", "series": "north", "point": 0, "was": { "y": 99.2 }, "to": { "y": 100 }, "before": { "class": "exact" } } }] } }),
        json!({ "command": "corrections_export", "args": { "dataset": "d2", "path": corrections } }),
        json!({ "command": "corrections_import", "args": { "dataset": "d2", "path": corrections } }),
        json!({ "command": "dataset_close", "args": { "id": "d1" } }),
        json!({ "command": "project_new" }),
    ]
}

#[test]
fn every_command_answers_with_its_declared_reply_kind() {
    let dir = TempDir::new("contract");
    let sink = CollectingSink::new();
    let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
    let mut app = App::new(config, Arc::new(sink)).unwrap();
    let table: std::collections::BTreeMap<&str, &str> = COMMANDS.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut requests = requests(&dir);
    // settings_set needs the settings as they are.
    let settings = app
        .dispatch_json(&json!({ "command": "settings_get" }).to_string())
        .unwrap();
    let mut settings = settings.as_object().unwrap().clone();
    settings.remove("reply");
    requests.push(json!({ "command": "settings_set", "args": { "settings": settings } }));
    for request in requests {
        let parsed: Request = serde_json::from_value(request.clone()).unwrap();
        let command = parsed.name();
        let reply = app
            .dispatch_json(&request.to_string())
            .unwrap_or_else(|err| panic!("{command} failed: {err}"));
        assert_eq!(
            reply["reply"].as_str(),
            table.get(command).copied(),
            "{command}"
        );
        if command == "export_render" {
            assert!(app.wait_for_jobs(Duration::from_secs(60)));
        }
        seen.insert(command);
    }
    let all: BTreeSet<&str> = table.keys().copied().collect();
    assert_eq!(seen, all, "every command in COMMANDS is exercised");
    assert_eq!(table.len(), COMMANDS.len(), "no command is listed twice");

    // The binary commands answer with views through their own entry point.
    let csv = dir.path().join("t.csv");
    let reopened = app
        .dispatch_json(&json!({ "command": "dataset_open", "args": { "path": csv } }).to_string())
        .unwrap();
    let id = reopened["id"].as_str().unwrap();
    let mut binary = BTreeSet::new();
    for request in [
        json!({ "command": "view_get", "args": { "dataset": id, "x": "day", "y": "temp",
                "range": null, "pxWidth": 8, "method": "m4" } }),
        json!({ "command": "table_page", "args": { "dataset": id, "offset": 0, "limit": 2,
                "sort": null } }),
    ] {
        let parsed: Request = serde_json::from_value(request).unwrap();
        let command = parsed.name();
        assert!(
            !table.contains_key(command),
            "{command} is not a JSON command"
        );
        let bytes = app.dispatch_binary(parsed).unwrap();
        assert!(View::decode(&bytes).is_ok(), "{command}");
        binary.insert(command);
    }
    assert_eq!(
        binary,
        BINARY_COMMANDS.iter().copied().collect::<BTreeSet<_>>()
    );
}
