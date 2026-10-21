// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! B03-T05: context cues. The tick track (SDR-16), the dynamic reference tone (SDR-17)
//! and the spoken legend (SDR-18), with SDR-22 ramps.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    reason = "integration test; clippy treats its helpers as non-test code, and the test values are exact"
)]

mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use sonarithm_audaxis::context::{CROSSFADE, LEGEND_TRACK, REFERENCE_TRACK, TICK_TRACK};
use sonarithm_compiler::Compiled;
use sonarithm_core::score::{ScoreEvent, ScoreTrack, TrackRole};
use sonarithm_numisona::examples;
use sonarithm_numisona::templates::TEMPLATES;
use support::compile;

/// A spec over `ys` at two points per second, x = 0, 1, 2…
fn spec(ys: &[f64], mark: &str, context: Option<Value>) -> Value {
    let values: Vec<Value> = ys
        .iter()
        .enumerate()
        .map(|(x, y)| json!({"day": x, "temp": y}))
        .collect();
    let mut spec = json!({
        "specVersion": "0.1",
        "name": "Weather",
        "data": {"source": {"kind": "inline", "values": values},
                 "fields": {"day": {"role": "x"},
                            "temp": {"unit": "°C", "label": "Temperature"}}},
        "substrate": {"time": {"pace": {"pointsPerSecond": 2}}},
        "tracks": [{"id": "temp", "mark": mark, "channels": {"pitch": {"field": "temp"}}}]
    });
    if let Some(context) = context {
        spec["context"] = context;
    }
    spec
}

fn track<'a>(compiled: &'a Compiled, id: &str) -> Option<&'a ScoreTrack> {
    compiled.score.tracks.iter().find(|t| t.id.as_str() == id)
}

fn notes(track: &ScoreTrack) -> Vec<(f64, f64, f64)> {
    track
        .events
        .iter()
        .filter_map(|e| match e {
            ScoreEvent::Note(n) => Some((n.t.0, n.dur.0, n.pitch.0)),
            _ => None,
        })
        .collect()
}

fn codes(compiled: &Compiled) -> Vec<String> {
    compiled
        .diagnostics
        .iter()
        .map(|d| d.code.as_str().to_owned())
        .collect()
}

fn ramp(n: usize) -> Vec<f64> {
    (0..n).map(|k| k as f64).collect()
}

/// SDR-16: discrete marks get ~10 ms centred clicks at the major x ticks, on the context
/// bus; the interval is a nice step for about five intervals (19 days → every 5 days).
#[test]
fn sdr_16_ticks_play_at_major_ticks_for_discrete_marks() {
    let compiled = compile(&spec(&ramp(20), "tone", None).to_string(), None);
    let ticks = track(&compiled, TICK_TRACK).expect("a tick track");
    assert_eq!(ticks.role, TrackRole::Context);
    assert_eq!(ticks.instrument.0, "click");
    let found = notes(ticks);
    let onsets: Vec<f64> = found.iter().map(|n| n.0).collect();
    assert_eq!(
        onsets,
        [0.0, 2.5, 5.0, 7.5],
        "days 0, 5, 10 and 15 at two points per second"
    );
    for (_, dur, _) in &found {
        assert!((dur - 0.010).abs() < 1e-12, "about 10 ms");
    }
    for event in &ticks.events {
        let ScoreEvent::Note(n) = event else {
            panic!("ticks are notes")
        };
        assert!(n.pan.0.abs() < 1e-12, "centred, never panned");
    }
    assert!(
        codes(&compiled).contains(&"SNR-ENG-107".to_owned()),
        "the automatic choice is reported"
    );
}

/// SDR-16: ticks are off for continuous glides.
#[test]
fn sdr_16_ticks_are_off_for_glides() {
    let compiled = compile(&spec(&ramp(20), "glide", None).to_string(), None);
    assert!(track(&compiled, TICK_TRACK).is_none());
    assert!(codes(&compiled).contains(&"SNR-ENG-107".to_owned()));
    let forced = spec(
        &ramp(20),
        "glide",
        Some(json!({"ticks": {"enabled": true}})),
    );
    assert!(
        track(&compile(&forced.to_string(), None), TICK_TRACK).is_some(),
        "on when asked"
    );
}

/// SDR-16: `every` sets the interval in units of the time field.
#[test]
fn sdr_16_every_sets_the_tick_interval() {
    let spec = spec(&ramp(20), "tone", Some(json!({"ticks": {"every": 4}})));
    let compiled = compile(&spec.to_string(), None);
    let onsets: Vec<f64> = notes(track(&compiled, TICK_TRACK).unwrap())
        .iter()
        .map(|n| n.0)
        .collect();
    assert_eq!(onsets, [0.0, 2.0, 4.0, 6.0, 8.0]);
}

/// SDR-17: the reference sounds the maximum while values rise and the minimum while they
/// fall, in a timbre distinct from the data, and switches overlap by at least 40 ms.
#[test]
fn sdr_17_dynamic_reference_follows_the_trend() {
    let ys = [
        5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0,
    ];
    let compiled = compile(&spec(&ys, "tone", None).to_string(), None);
    let data = notes(&compiled.score.tracks[0]);
    let pitch_of = |value: f64| data[ys.iter().position(|y| *y == value).unwrap()].2;
    let (max, min) = (pitch_of(10.0), pitch_of(3.0));
    let reference = track(&compiled, REFERENCE_TRACK).expect("a reference track");
    assert_eq!(reference.role, TrackRole::Context);
    assert_eq!(
        reference.instrument.0, "triangle",
        "distinct from the mallet data"
    );
    let tones = notes(reference);
    let pitches: Vec<f64> = tones.iter().map(|n| n.2).collect();
    assert_eq!(pitches, [max, min], "rising, then falling");
    let overlap = tones[0].0 + tones[0].1 - tones[1].0;
    assert!(overlap >= CROSSFADE.0 - 1e-9, "crossfade {overlap} s");
}

/// SDR-17: a static first-value reference is never built: with the defaults, on every
/// template and example dataset, the reference only ever sounds the series maximum or
/// minimum.
#[test]
fn sdr_17_no_static_first_value_reference_anywhere() {
    let ys = [5.0, 7.0, 9.0, 4.0, 2.0, 6.0, 8.0];
    let compiled = compile(&spec(&ys, "tone", None).to_string(), None);
    let data = notes(&compiled.score.tracks[0]);
    let first = data[0].2;
    for (_, _, pitch) in notes(track(&compiled, REFERENCE_TRACK).unwrap()) {
        assert!(
            (pitch - first).abs() > 1e-9,
            "the first value is never the reference"
        );
    }
    let datasets = examples::all().unwrap();
    for template in &TEMPLATES {
        for (name, bundle) in &datasets {
            let Ok(bindings) = template.bind(bundle.primary().unwrap()) else {
                continue;
            };
            let source = json!({"kind": "file", "path": format!("{name}.json")});
            let spec = template.apply(bundle, &bindings, &source).unwrap();
            let compiled = compile(&spec.to_string(), Some(bundle.clone()));
            let Some(reference) = track(&compiled, REFERENCE_TRACK) else {
                continue;
            };
            let pitches: Vec<f64> = compiled.score.tracks[0]
                .events
                .iter()
                .filter_map(|e| match e {
                    ScoreEvent::Note(n) => Some(n.pitch.0),
                    ScoreEvent::Glide(g) => g.curve.first().map(|b| b.pitch.0),
                    _ => None,
                })
                .collect();
            let (low, high) = pitches
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                    (lo.min(*p), hi.max(*p))
                });
            for (_, _, pitch) in notes(reference) {
                assert!(
                    (pitch - low).abs() < 1e-6 || (pitch - high).abs() < 1e-6,
                    "{} × {name}: reference {pitch} is not an extreme ({low}, {high})",
                    template.id
                );
            }
        }
    }
}

/// SDR-17: an explicit static reference is built but warned about; a zero line needs zero
/// inside the data.
#[test]
fn sdr_17_static_and_zero_line_references() {
    let ys = [1.0, 3.0, 5.0, 2.0];
    let fixed = spec(
        &ys,
        "tone",
        Some(json!({"reference": {"kind": "static-value", "value": 3}})),
    );
    let compiled = compile(&fixed.to_string(), None);
    assert!(codes(&compiled).contains(&"SNR-ENG-104".to_owned()));
    let data = notes(&compiled.score.tracks[0]);
    let tones = notes(track(&compiled, REFERENCE_TRACK).unwrap());
    assert_eq!(tones.len(), 1);
    assert!(
        (tones[0].2 - data[1].2).abs() < 1e-6,
        "the pitch the data gives 3"
    );

    let zero = spec(
        &ys,
        "tone",
        Some(json!({"reference": {"kind": "zero-line"}})),
    );
    let compiled = compile(&zero.to_string(), None);
    assert!(track(&compiled, REFERENCE_TRACK).is_none());
    assert!(codes(&compiled).contains(&"SNR-ENG-105".to_owned()));
}

/// SDR-18: the legend is spoken before playback (a speech track at the start) with the
/// title, the ranges, the units, the polarity, the mode and the cues.
#[test]
fn sdr_18_the_legend_is_spoken_before_playback() {
    let compiled = compile(&spec(&ramp(20), "tone", None).to_string(), None);
    let legend = track(&compiled, LEGEND_TRACK).expect("a legend track");
    assert_eq!(legend.role, TrackRole::Speech);
    let [ScoreEvent::Speech(speech)] = legend.events.as_slice() else {
        panic!("one speech event");
    };
    assert!(speech.t.0.abs() < 1e-12);
    insta::assert_snapshot!(speech.text.clone());
    for part in [
        "Weather",
        "X axis day, 0 to 19",
        "degrees Celsius",
        "higher values sound higher",
        "Analysis mode",
        "Ticks and reference tone on",
    ] {
        assert!(speech.text.contains(part), "{part}: {}", speech.text);
    }
    let on_demand = spec(
        &ramp(20),
        "tone",
        Some(json!({"legend": {"speak": "on-demand"}})),
    );
    assert!(track(&compile(&on_demand.to_string(), None), LEGEND_TRACK).is_none());
}

/// SDR-22: every context note is long enough for its onset and offset ramps (two
/// minimum ramps of 5 ms).
#[test]
fn sdr_22_context_notes_have_room_for_their_ramps() {
    let compiled = compile(&spec(&ramp(20), "tone", None).to_string(), None);
    for track in compiled
        .score
        .tracks
        .iter()
        .filter(|t| t.role == TrackRole::Context)
    {
        for (_, dur, _) in notes(track) {
            assert!(dur >= 0.010 - 1e-12, "{}: {dur} s", track.id);
        }
    }
}

/// The context tracks of a compiled Score, for review.
#[test]
fn context_tracks_snapshot() {
    let ys = [5.0, 6.0, 8.0, 7.0, 4.0, 3.0, 5.0, 9.0, 10.0, 8.0];
    let compiled = compile(&spec(&ys, "tone", None).to_string(), None);
    let context: Vec<&ScoreTrack> = compiled
        .score
        .tracks
        .iter()
        .filter(|t| t.role != TrackRole::Data)
        .collect();
    insta::assert_json_snapshot!(context);
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tests/golden/context-cues")
}

/// The `context-cues` golden case (`cargo xtask golden`) renders this Score: a rising and
/// falling series with the default ticks, reference tone and legend. Engine versions are
/// written as `*`. Set `UPDATE_GOLDEN_SCORE=1` to rewrite it, then update the reference.
#[test]
fn context_golden_score_is_current() {
    let ys = [5.0, 6.0, 8.0, 7.0, 4.0, 3.0, 5.0, 9.0, 10.0, 8.0, 6.0, 7.0];
    let mut score = compile(&spec(&ys, "tone", None).to_string(), None).score;
    for version in score.header.engines.values_mut() {
        *version = sonarithm_core::score::EngineVersion("*".to_owned());
    }
    let text = serde_json::to_string_pretty(&score).unwrap() + "\n";
    let path = golden_dir().join("score.json");
    if std::env::var_os("UPDATE_GOLDEN_SCORE").is_some() {
        std::fs::create_dir_all(golden_dir()).unwrap();
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
