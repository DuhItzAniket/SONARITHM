// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A06-T06: the experimental `auto` mark switches from tones to a glide above about 10
//! events per second (SDR-20, INFER), and every use says it is experimental.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use serde_json::json;
use sonarithm_core::score::ScoreEvent;
use sonarithm_numisona::examples;
use support::{compile, has_code};

fn auto(pace: &serde_json::Value, channels: &serde_json::Value) -> String {
    json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "file", "path": "example.json" } },
        "substrate": { "time": { "pace": pace } },
        "tracks": [{ "id": "data", "mark": "auto", "channels": channels }],
    })
    .to_string()
}

fn kinds(compiled: &sonarithm_compiler::Compiled) -> (usize, usize) {
    compiled.score.tracks[0]
        .events
        .iter()
        .fold((0, 0), |(notes, glides), event| match event {
            ScoreEvent::Note(_) => (notes + 1, glides),
            ScoreEvent::Glide(_) => (notes, glides + 1),
            _ => (notes, glides),
        })
}

/// SDR-20 (INFER, experimental): `auto` keeps tones at overview rates, glides above about
/// 10 events per second, and reports itself as experimental every time.
#[test]
fn sdr_20_auto_mark_switches_to_a_glide_on_dense_data_and_is_flagged_experimental() {
    let pitch = json!({ "pitch": { "field": "temp" } });
    let slow = compile(
        &auto(&json!({ "pointsPerSecond": 3 }), &pitch),
        Some(examples::temperature().unwrap()),
    );
    assert_eq!(kinds(&slow), (120, 0));
    assert!(has_code(&slow, "SNR-ENG-011"));
    let fast = compile(
        &auto(&json!({ "pointsPerSecond": 20 }), &pitch),
        Some(examples::temperature().unwrap()),
    );
    assert_eq!(kinds(&fast), (0, 1));
    let flag = fast
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "SNR-ENG-011")
        .unwrap();
    assert!(flag.message.contains("experimental") && flag.rules[0].as_str() == "SDR-20");
    // No SDR-19 rate warning for the glide: it is not a discrete mark.
    assert!(!has_code(&fast, "SNR-ENG-049"));

    // The DS-005-like dense signal: 2000 samples in 20 s is 100 events per second.
    let signal = compile(
        &auto(
            &json!({ "duration": { "s": 20 } }),
            &json!({ "pitch": { "field": "amplitude" }, "duration": { "value": { "s": 0.1 } } }),
        ),
        Some(examples::signal().unwrap()),
    );
    assert_eq!(kinds(&signal), (0, 1));
    // Tone-only channels do not apply once `auto` glides, and the encoder says so.
    assert!(has_code(&signal, "SNR-ENG-055"));
}
