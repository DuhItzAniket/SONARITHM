// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C02-T09b: `experimental.lowConfidenceTimbre` (RFC-0006 §3, SDR-43, `INFER`). Estimated
//! notes play darker, more so at lower confidence; exact notes, pitch, timing and loudness
//! do not change; with the switch off or absent the Score is byte-identical.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use sonarithm_core::score::{Note, ScoreEvent};
use sonarithm_numisona::encode::{
    DEFAULT_BRIGHTNESS, ESTIMATE_DARKENING, UNCERTAINTY_DARKENING, low_confidence_brightness,
};
use support::{compile, has_code};

const ROWS: &str = r#"[
  { "x": 1, "y": 3.0, "__snr_prov": "exact" },
  { "x": 2, "y": 5.0, "__snr_prov": "cv_estimate", "__snr_conf": 1.0 },
  { "x": 3, "y": 4.0, "__snr_prov": "cv_estimate", "__snr_conf": 0.5 },
  { "x": 4, "y": 6.0, "__snr_prov": "ml_estimate", "__snr_conf": 0.0 },
  { "x": 5, "y": 2.0, "__snr_prov": "user_edited" }
]"#;

/// A 0.4 spec over [`ROWS`] with `experimental` (or none) and extra channels.
fn spec(experimental: Option<&str>, channels: &str) -> String {
    spec_of("tone", experimental, channels)
}

/// [`spec`] with another mark.
fn spec_of(mark: &str, experimental: Option<&str>, channels: &str) -> String {
    let block = experimental.map_or_else(String::new, |b| format!(r#""experimental": {b},"#));
    format!(
        r#"{{ "specVersion": "0.4", {block}
  "data": {{ "source": {{ "kind": "inline", "values": {ROWS} }} }},
  "substrate": {{ "time": {{ "field": "x" }} }},
  "tracks": [{{ "id": "y", "mark": "{mark}", "channels": {{ "pitch": {{ "field": "y" }} {channels} }} }}] }}"#
    )
}

fn notes(spec: &str) -> Vec<Note> {
    let compiled = compile(spec, None);
    compiled.score.tracks[0]
        .events
        .iter()
        .filter_map(|event| match event {
            ScoreEvent::Note(note) => Some(note.clone()),
            _ => None,
        })
        .collect()
}

/// SDR-43: one octave of cutoff darker at full confidence, up to two at none, never
/// below 0.
#[test]
fn sdr_43_the_darkening_steps_with_confidence() {
    let b = DEFAULT_BRIGHTNESS;
    assert!((low_confidence_brightness(b, Some(1.0)) - (b - ESTIMATE_DARKENING)).abs() < 1e-12);
    let half = b - ESTIMATE_DARKENING - UNCERTAINTY_DARKENING * 0.5;
    assert!((low_confidence_brightness(b, Some(0.5)) - half).abs() < 1e-12);
    assert!(low_confidence_brightness(b, Some(0.0)).abs() < 1e-12);
    assert!(low_confidence_brightness(b, None).abs() < 1e-12);
    assert!(low_confidence_brightness(0.1, Some(1.0)).abs() < 1e-12);
    assert!((low_confidence_brightness(1.0, Some(1.0)) - 0.75).abs() < 1e-12);
}

/// SDR-43, RFC-0006 §3: with the switch on, estimated notes are darker in order of
/// confidence; exact and user-edited notes keep their brightness; pitch, onset, duration
/// and gain are unchanged.
#[test]
fn sdr_43_estimated_notes_play_darker_when_the_experiment_is_on() {
    let plain = notes(&spec(None, ""));
    let darker = notes(&spec(Some(r#"{ "lowConfidenceTimbre": true }"#), ""));
    assert_eq!(plain.len(), 5);
    assert_eq!(darker.len(), 5);
    let brightness: Vec<f64> = darker.iter().map(|n| n.timbre.brightness).collect();
    let expected = [
        DEFAULT_BRIGHTNESS,
        low_confidence_brightness(DEFAULT_BRIGHTNESS, Some(1.0)),
        low_confidence_brightness(DEFAULT_BRIGHTNESS, Some(0.5)),
        low_confidence_brightness(DEFAULT_BRIGHTNESS, Some(0.0)),
        DEFAULT_BRIGHTNESS,
    ];
    for (got, want) in brightness.iter().zip(expected) {
        assert!((got - want).abs() < 1e-6, "{brightness:?}");
    }
    for (a, b) in plain.iter().zip(&darker) {
        assert_eq!(
            (a.t, a.dur, a.pitch, a.gain, a.conf),
            (b.t, b.dur, b.pitch, b.gain, b.conf)
        );
    }
}

/// RFC-0006 §6: with the switch off, `{}` or no block, the Score is byte-identical.
#[test]
fn sdr_43_the_experiment_off_leaves_the_score_identical() {
    let hash = |spec: &str| compile(spec, None).score.canonical_hash().unwrap();
    let none = hash(&spec(None, ""));
    assert_eq!(hash(&spec(Some("{}"), "")), none);
    assert_eq!(
        hash(&spec(Some(r#"{ "lowConfidenceTimbre": false }"#), "")),
        none
    );
    assert_ne!(
        hash(&spec(Some(r#"{ "lowConfidenceTimbre": true }"#), "")),
        none
    );
}

/// RFC-0006 §4: a switch that is on is reported once (`SNR-COMP-037`), and NUMISONA warns
/// when brightness already carries data (`SNR-ENG-057`).
#[test]
fn sdr_43_the_experiment_is_reported_and_a_brightness_channel_warns() {
    let on = Some(r#"{ "lowConfidenceTimbre": true }"#);
    let compiled = compile(&spec(on, ""), None);
    let notes: Vec<_> = compiled
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str() == "SNR-COMP-037")
        .collect();
    assert_eq!(notes.len(), 1);
    assert!(notes[0].rules.iter().any(|r| r.as_str() == "SDR-43"));
    assert_eq!(compiled.experiments, ["lowConfidenceTimbre"]);
    assert!(!has_code(&compiled, "SNR-ENG-057"));
    let mapped = compile(&spec(on, r#", "brightness": { "field": "x" }"#), None);
    assert!(has_code(&mapped, "SNR-ENG-057"));
    let off = compile(&spec(None, r#", "brightness": { "field": "x" }"#), None);
    assert!(!has_code(&off, "SNR-COMP-037") && !has_code(&off, "SNR-ENG-057"));
    assert_eq!(off.experiments, Vec::<String>::new());
}

/// SDR-43: a glide's breakpoints over estimated rows are darker in the same way.
#[test]
fn sdr_43_glide_points_over_estimates_play_darker() {
    let compiled = compile(
        &spec_of("glide", Some(r#"{ "lowConfidenceTimbre": true }"#), ""),
        None,
    );
    let points: Vec<f64> = compiled.score.tracks[0]
        .events
        .iter()
        .filter_map(|event| match event {
            ScoreEvent::Glide(glide) => Some(glide.curve.iter().map(|p| p.brightness)),
            _ => None,
        })
        .flatten()
        .collect();
    // Five rows plus the held last value.
    assert_eq!(points.len(), 6, "{points:?}");
    let expected = [
        DEFAULT_BRIGHTNESS,
        low_confidence_brightness(DEFAULT_BRIGHTNESS, Some(1.0)),
        low_confidence_brightness(DEFAULT_BRIGHTNESS, Some(0.5)),
        low_confidence_brightness(DEFAULT_BRIGHTNESS, Some(0.0)),
        DEFAULT_BRIGHTNESS,
        DEFAULT_BRIGHTNESS,
    ];
    for (got, want) in points.iter().zip(expected) {
        assert!((got - want).abs() < 1e-6, "{points:?}");
    }
}
