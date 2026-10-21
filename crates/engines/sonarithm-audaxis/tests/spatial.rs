// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! B03-T11: stereo placement and the never-alone guard (A11Y-14, SDR-27, SDR-30).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "integration test; clippy treats its helpers as non-test code, and sample positions are small"
)]

mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use sonarithm_audaxis::spatial::Spatial;
use sonarithm_core::engine::{Budget, CancelToken, CompileContext, Shaper};
use sonarithm_core::score::{Score, ScoreEvent, ShaperStage};
use sonarithm_core::units::{SampleRate, Seconds};
use sonarithm_render::{OfflineRender, PlanConfig, RenderPlan};
use support::compile;

const RATE: u32 = 48_000;

fn inline(values: &[Value], pitch: &Value, pan: Option<&Value>) -> Value {
    let mut channels = json!({ "pitch": pitch });
    if let Some(pan) = pan {
        channels["pan"] = pan.clone();
    }
    json!({
        "specVersion": "0.1",
        "data": {"source": {"kind": "inline", "values": values},
                 "fields": {"x": {"role": "x"}}},
        "substrate": {"time": {"pace": {"pointsPerSecond": 2}}},
        "tracks": [{"id": "y", "mark": "tone", "channels": channels}],
        // Context cues off: these tests are about stereo placement alone.
        "context": {"ticks": {"enabled": false}, "reference": {"kind": "none"},
                    "legend": {"speak": "off"}}
    })
}

fn codes(compiled: &sonarithm_compiler::Compiled) -> Vec<String> {
    compiled
        .diagnostics
        .iter()
        .map(|d| d.code.as_str().to_owned())
        .collect()
}

fn shape(score: &mut Score) -> Vec<String> {
    let mut cx = CompileContext::new(0, Budget::default(), CancelToken::new());
    Spatial.shape(score, &mut cx).unwrap();
    cx.diagnostics()
        .iter()
        .map(|d| d.code.as_str().to_owned())
        .collect()
}

/// A Score with one data track of mallet notes; `notes` are (pitch in cents, pan, preset).
fn score_of(notes: &[(f64, f64, Option<&str>)]) -> Score {
    let events: Vec<Value> = notes
        .iter()
        .enumerate()
        .map(|(k, (pitch, pan, preset))| {
            json!({"type": "note", "t": k as f64 * 0.5, "dur": 0.25, "pitch": pitch,
                   "gain": 0.0, "pan": pan,
                   "timbre": {"preset": preset, "brightness": 0.5}, "taps": 1,
                   "src": null, "conf": null})
        })
        .collect();
    serde_json::from_value(json!({
        "header": {"ir_version": 1, "spec_version": "0.1",
                   "spec_hash": "0".repeat(64), "data_fingerprint": "0".repeat(64),
                   "seed": 0, "mode": "analysis", "duration": notes.len() as f64 * 0.5,
                   "engines": {}, "shapers": []},
        "tracks": [{"id": "y", "role": "data", "engine": "test", "instrument": "mallet",
                    "gain": 0.0, "pan": 0.0, "pitch_range": null, "events": events}],
        "markers": []
    }))
    .unwrap()
}

/// The power of `frequency` in `signal` (Goertzel).
fn power(signal: &[f32], frequency: f64) -> f64 {
    let omega = 2.0 * std::f64::consts::PI * frequency / f64::from(RATE);
    let coefficient = 2.0 * omega.cos();
    let (mut s1, mut s2) = (0.0_f64, 0.0_f64);
    for &x in signal {
        let s = f64::from(x) + coefficient * s1 - s2;
        s2 = s1;
        s1 = s;
    }
    s1 * s1 + s2 * s2 - coefficient * s1 * s2
}

/// A11Y-14: with pitch and pan on the same field, the mono downmix still tells every
/// value apart: inside each note, the note's own pitch dominates every other note's pitch,
/// and no note loses more than the 3 dB a constant-power panner allows.
#[test]
fn a11y_14_information_survives_a_mono_downmix() {
    let values: Vec<Value> = (0..5).map(|x| json!({"x": x, "y": x})).collect();
    let spec = inline(
        &values,
        &json!({"field": "y"}),
        Some(&json!({"field": "y"})),
    );
    let compiled = compile(&spec.to_string(), None);
    let score = &compiled.score;
    assert!(score.header.shapers.contains(&ShaperStage::Spatial));
    let plan = RenderPlan::build(score, &PlanConfig::new(SampleRate::new(RATE).unwrap()))
        .unwrap()
        .plan;
    let stereo = OfflineRender::new(plan, 256).render_all();
    let mono: Vec<f32> = stereo
        .chunks(2)
        .map(|f| f32::midpoint(f[0], f[1]))
        .collect();
    let notes: Vec<(Seconds, Seconds, f64, f64)> = score.tracks[0]
        .events
        .iter()
        .filter_map(|e| match e {
            ScoreEvent::Note(n) => Some((n.t, n.dur, n.pitch.to_hz().0, n.pan.0)),
            _ => None,
        })
        .collect();
    assert_eq!(notes.len(), 5);
    let pans: Vec<f64> = notes.iter().map(|n| n.3).collect();
    assert!(
        pans.first() < pans.last(),
        "pan follows the field: {pans:?}"
    );
    let mut levels = Vec::new();
    for (k, &(t, dur, hz, _)) in notes.iter().enumerate() {
        let start = ((t.0 + 0.03) * f64::from(RATE)) as usize;
        let end = ((t.0 + dur.0 - 0.03) * f64::from(RATE)) as usize;
        let window = &mono[start..end];
        let own = power(window, hz);
        for (j, other) in notes.iter().enumerate() {
            if j != k {
                assert!(
                    own > 10.0 * power(window, other.2),
                    "note {k} at {hz} Hz vs {} Hz",
                    other.2
                );
            }
        }
        let rms = (window.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / window.len() as f64)
            .sqrt();
        levels.push(20.0 * rms.log10());
    }
    let (low, high) = levels
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), l| {
            (lo.min(*l), hi.max(*l))
        });
    assert!(low > -50.0, "every note is audible in mono: {levels:?}");
    assert!(high - low < 6.0, "mono levels stay close: {levels:?}");
}

/// A11Y-14: a track that differs only in pan is reported, by the encoder for its own spec
/// (`SNR-ENG-052`) and by AUDAXIS for any other Score, never twice.
#[test]
fn a11y_14_pan_alone_is_reported_once() {
    let values: Vec<Value> = (0..4).map(|x| json!({"x": x, "y": x})).collect();
    let spec = inline(
        &values,
        &json!({"value": {"hz": 600}}),
        Some(&json!({"field": "y"})),
    );
    let found = codes(&compile(&spec.to_string(), None));
    assert!(found.contains(&"SNR-ENG-052".to_owned()), "{found:?}");
    assert!(!found.contains(&"SNR-ENG-100".to_owned()), "{found:?}");

    let mut score = score_of(&[(0.0, -1.0, None), (0.0, 0.0, None), (0.0, 1.0, None)]);
    assert_eq!(shape(&mut score), ["SNR-ENG-100"]);
}

/// A11Y-14: timbre alone is reported; timbre with pitch is not.
#[test]
fn a11y_14_timbre_alone_is_reported() {
    let mut alone = score_of(&[(0.0, 0.0, Some("mallet")), (0.0, 0.0, Some("pluck"))]);
    assert_eq!(shape(&mut alone), ["SNR-ENG-101"]);
    let mut with_pitch = score_of(&[(0.0, 0.0, Some("mallet")), (300.0, 0.0, Some("pluck"))]);
    assert_eq!(shape(&mut with_pitch), Vec::<String>::new());
}

/// A11Y-14, SDR-08: data pitches outside 65–1480 Hz are reported; a constant pitch is not
/// data, so it is not.
#[test]
fn a11y_14_sdr_08_pitch_extremes_are_reported() {
    // 0 cents is A4 (440 Hz); 2400 cents is 1760 Hz.
    let mut high = score_of(&[(0.0, 0.0, None), (2400.0, 0.0, None)]);
    assert_eq!(shape(&mut high), ["SNR-ENG-102"]);
    let mut inside = score_of(&[(-900.0, 0.0, None), (1200.0, 0.0, None)]);
    assert!(
        shape(&mut inside).is_empty(),
        "260 to 880 Hz is inside the range"
    );
    let mut constant = score_of(&[(2400.0, -0.5, None), (2400.0, 0.5, None)]);
    let found = shape(&mut constant);
    assert!(!found.contains(&"SNR-ENG-102".to_owned()), "{found:?}");
}

/// Stereo placement keeps every position finite and within [−1, 1].
#[test]
fn positions_are_kept_within_the_stereo_field() {
    let mut score = score_of(&[(0.0, 5.0, None), (300.0, 0.0, None), (600.0, -2.0, None)]);
    if let ScoreEvent::Note(note) = &mut score.tracks[0].events[1] {
        note.pan = sonarithm_core::units::Pan(f64::NAN);
    }
    shape(&mut score);
    let pans: Vec<f64> = score.tracks[0]
        .events
        .iter()
        .filter_map(|e| match e {
            ScoreEvent::Note(n) => Some(n.pan.0),
            _ => None,
        })
        .collect();
    assert_eq!(pans, [1.0, 0.0, -1.0]);
}

/// SDR-30: binaural and multichannel are reported as not yet available; the Score plays in
/// stereo.
#[test]
fn sdr_30_binaural_and_multichannel_fall_back_to_stereo() {
    let values: Vec<Value> = (0..3).map(|x| json!({"x": x, "y": x})).collect();
    let mut spec = inline(&values, &json!({"field": "y"}), None);
    spec["context"] = json!({"spatial": {"mode": "binaural"}});
    let found = codes(&compile(&spec.to_string(), None));
    assert!(found.contains(&"SNR-ENG-103".to_owned()), "{found:?}");
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tests/golden/stereo-placement")
}

/// The `stereo-placement` golden case (`cargo xtask golden`) renders this Score: x on pan
/// (a positional variable, SDR-30) and y on pitch, compiled with NUMISONA and AUDAXIS.
/// Engine versions are written as `*`, since they change with every release. Set
/// `UPDATE_GOLDEN_SCORE=1` to rewrite it, then update the golden reference.
#[test]
fn stereo_golden_score_is_current() {
    let ys = [3, 1, 4, 1, 5, 9, 2, 6];
    let values: Vec<Value> = ys
        .iter()
        .enumerate()
        .map(|(x, y)| json!({"x": x, "y": y}))
        .collect();
    let spec = inline(
        &values,
        &json!({"field": "y"}),
        Some(&json!({"field": "x"})),
    );
    let mut score = compile(&spec.to_string(), None).score;
    for version in score.header.engines.values_mut() {
        *version = sonarithm_core::score::EngineVersion("*".to_owned());
    }
    let text = serde_json::to_string_pretty(&score).unwrap() + "\n";
    let path = golden_dir().join("score.json");
    if std::env::var_os("UPDATE_GOLDEN_SCORE").is_some() {
        std::fs::write(&path, &text).unwrap();
    }
    let stored = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(
        stored, text,
        "run with UPDATE_GOLDEN_SCORE=1, then `cargo xtask golden --update`"
    );
}
