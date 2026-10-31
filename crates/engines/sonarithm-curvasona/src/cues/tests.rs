// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_json::{Value, json};
use sonarithm_audaxis::Audaxis;
use sonarithm_audaxis::earcons::CATALOGUE;
use sonarithm_compiler::{Compiled, Compiler, Platform};
use sonarithm_core::engine::{CancelToken, Engine};
use sonarithm_core::score::ScoreTrack;

use super::*;
use crate::Curvasona;

fn spec(expr: &str, domain: [f64; 2], pitch: &Value, analysis: &Value) -> Value {
    json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "function", "expr": expr, "domain": domain, "samples": 401 } },
        "substrate": { "time": { "pace": { "duration": { "s": 10 } } } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": pitch } }],
        "analysis": analysis
    })
}

fn cued(expr: &str, domain: [f64; 2]) -> Value {
    spec(
        expr,
        domain,
        &json!({ "field": "y" }),
        &json!({ "detect": ["critical-points"] }),
    )
}

fn compile_with(spec: &Value, navigation: bool) -> Compiled {
    let platform = Platform::new();
    let curvasona = Curvasona::new();
    let numisona = sonarithm_numisona::Numisona::new();
    let audaxis = Audaxis::new();
    let mut engines: Vec<&dyn Engine<DataBundle>> = vec![&platform, &curvasona, &numisona];
    if navigation {
        engines.push(&audaxis);
    }
    let compiler: Compiler<DataBundle> = Compiler::new(&engines).unwrap();
    compiler
        .compile(&spec.to_string(), &CancelToken::new())
        .unwrap()
}

fn compile(spec: &Value) -> Compiled {
    compile_with(spec, false)
}

fn cue_track(compiled: &Compiled) -> Option<&ScoreTrack> {
    compiled
        .score
        .tracks
        .iter()
        .find(|t| t.id.as_str() == CUE_TRACK)
}

fn earcons(compiled: &Compiled) -> Vec<(f64, EarconKind)> {
    cue_track(compiled)
        .map(|track| {
            track
                .events
                .iter()
                .filter_map(|e| match e {
                    ScoreEvent::Earcon(earcon) => Some((earcon.t.0, earcon.kind)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.03
}

/// A11Y-28: a function's poles play `E-ASYMPTOTE` and its maximum `E-LANDMARK`, at the times
/// the glide reaches them (x = -2, 0, 2 over 10 s from -5).
#[test]
fn a11y_28_poles_and_extrema_are_cued_where_they_play() {
    let compiled = compile(&cued("1/(x^2 - 4)", [-5.0, 5.0]));
    let found = earcons(&compiled);
    let kinds: Vec<EarconKind> = found.iter().map(|(_, k)| *k).collect();
    assert_eq!(
        kinds,
        [
            EarconKind::Asymptote,
            EarconKind::Landmark,
            EarconKind::Asymptote
        ]
    );
    for ((t, _), want) in found.iter().zip([3.0, 5.0, 7.0]) {
        assert!(near(*t, want), "{t} vs {want}");
    }
    assert_eq!(cue_track(&compiled).unwrap().role, TrackRole::Earcon);
}

/// A11Y-28: zeros play `E-ZERO`; a zero and an inflection at the same point merge into one
/// cue whose kind is the zero (RFC-0007 amendment 1).
#[test]
fn a11y_28_zeros_play_e_zero_and_coincident_features_merge() {
    let compiled = compile(&cued("x^3 - 3*x", [-3.0, 3.0]));
    let kinds: Vec<EarconKind> = earcons(&compiled).iter().map(|(_, k)| *k).collect();
    assert_eq!(
        kinds,
        [
            EarconKind::Zero,
            EarconKind::Landmark,
            EarconKind::Zero,
            EarconKind::Landmark,
            EarconKind::Zero
        ]
    );
}

/// RFC-0007 amendment 1 and ADR-0029: features closer than the merge window play one
/// earcon of the highest-priority kind; its `src` is that feature's row, and the cue log
/// names every merged feature.
#[test]
fn a11y_24_merged_cues_keep_every_feature_in_the_log() {
    let compiled = compile(&cued("(x - 0.02)*x/(x - 0.01)", [-1.0, 1.0]));
    let track = cue_track(&compiled).unwrap();
    let merged = track
        .events
        .iter()
        .find_map(|e| match e {
            ScoreEvent::Earcon(earcon) if earcon.kind == EarconKind::Asymptote => Some(earcon),
            _ => None,
        })
        .unwrap();
    assert!(
        earcons(&compiled)
            .iter()
            .all(|(t, k)| *k == EarconKind::Asymptote || (t - merged.t.0).abs() >= 0.15),
        "{:?}",
        earcons(&compiled)
    );
    // The same function sampled outside the compile: the same table, rows and annotations.
    let sampled = crate::sample_function(
        &crate::FunctionSpec::new("(x - 0.02)*x/(x - 0.01)", [-1.0, 1.0]).with_samples(401),
        &Budgets::DEFAULT,
        (sonarithm_data::BundleId(0), crate::FUNCTION_TABLE),
    )
    .unwrap();
    let annotations = crate::function_features(&sampled.bundle, &Budgets::DEFAULT).unwrap();
    let found = cues(&compiled.score, &sampled.bundle, &annotations);
    let cue = found
        .iter()
        .find(|c| c.kind == EarconKind::Asymptote)
        .unwrap();
    for label in ["zero", "vertical asymptote"] {
        assert!(cue.labels.iter().any(|l| l == label), "{:?}", cue.labels);
    }
    assert_eq!(cue.labels.len(), cue.rows.len());
    let log = cue_log(&found);
    assert_eq!(log.len(), found.len());
    assert!(
        log.iter()
            .any(|line| line.contains("vertical asymptote") && line.contains("zero"))
    );
    let pole_row = cue
        .labels
        .iter()
        .zip(&cue.rows)
        .find(|(l, _)| *l == "vertical asymptote")
        .map(|(_, r)| *r);
    assert_eq!(merged.src, pole_row);
}

/// ADR-0029: a merged earcon's `src` is the row of its highest-priority feature, even when
/// a landmark comes first, and every feature stays in the cue (and so in `cue_log`).
#[test]
fn adr_0029_merged_src_is_the_highest_priority_features_row() {
    let row = |r: u64| RowRef {
        table: sonarithm_core::TableId(0),
        generation: sonarithm_core::Generation(0),
        row: r,
    };
    let landmark_then_asymptote = Cue {
        t: Seconds(1.0),
        kind: EarconKind::Asymptote,
        labels: vec![
            "local maximum".to_owned(),
            "zero".to_owned(),
            "vertical asymptote".to_owned(),
        ],
        rows: vec![row(10), row(11), row(12)],
    };
    assert_eq!(first_of_kind(&landmark_then_asymptote), Some(row(12)));
    let landmark_then_zero = Cue {
        t: Seconds(2.0),
        kind: EarconKind::Zero,
        labels: vec!["inflection point".to_owned(), "zero".to_owned()],
        rows: vec![row(20), row(21)],
    };
    assert_eq!(first_of_kind(&landmark_then_zero), Some(row(21)));
    let landmarks = Cue {
        t: Seconds(3.0),
        kind: EarconKind::Landmark,
        labels: vec!["local minimum".to_owned(), "inflection point".to_owned()],
        rows: vec![row(30), row(31)],
    };
    assert_eq!(first_of_kind(&landmarks), Some(row(30)));
    let log = cue_log(&[landmark_then_asymptote]);
    assert_eq!(log, ["1.000 s: local maximum, zero, vertical asymptote"]);
    assert!(priority(EarconKind::Asymptote) > priority(EarconKind::Undefined));
    assert!(priority(EarconKind::Undefined) > priority(EarconKind::Offscreen));
    assert!(priority(EarconKind::Offscreen) > priority(EarconKind::Zero));
    assert!(priority(EarconKind::Zero) > priority(EarconKind::Landmark));
}

/// `analysis.cue: "speech"` speaks each feature's text equivalent instead; `"none"` and a
/// spec without `critical-points` add no cue track.
#[test]
fn cue_delivery_follows_analysis_cue() {
    let speech = compile(&spec(
        "1/(x^2 - 4)",
        [-5.0, 5.0],
        &json!({ "field": "y" }),
        &json!({ "detect": ["critical-points"], "cue": "speech" }),
    ));
    let track = cue_track(&speech).unwrap();
    assert_eq!(track.role, TrackRole::Speech);
    let texts: Vec<&str> = track
        .events
        .iter()
        .filter_map(|e| match e {
            ScoreEvent::Speech(s) => Some(s.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        texts,
        ["vertical asymptote", "local maximum", "vertical asymptote"]
    );
    let none = compile(&spec(
        "1/(x^2 - 4)",
        [-5.0, 5.0],
        &json!({ "field": "y" }),
        &json!({ "detect": ["critical-points"], "cue": "none" }),
    ));
    assert!(cue_track(&none).is_none());
    let plain = compile(&spec(
        "1/(x^2 - 4)",
        [-5.0, 5.0],
        &json!({ "field": "y" }),
        &json!({ "detect": [] }),
    ));
    assert!(cue_track(&plain).is_none());
}

/// RFC-0007 Q5: with an explicit pitch `scale.domain`, leaving it plays `E-OFFSCREEN`
/// (x^2 leaves [0, 4] on [-3, -2] and [2, 3]).
#[test]
fn a11y_28_leaving_the_window_plays_e_offscreen() {
    let compiled = compile(&spec(
        "x^2",
        [-3.0, 3.0],
        &json!({ "field": "y", "scale": { "domain": [0, 4] } }),
        &json!({ "detect": ["critical-points"] }),
    ));
    let offscreen: Vec<f64> = earcons(&compiled)
        .into_iter()
        .filter(|(_, k)| *k == EarconKind::Offscreen)
        .map(|(t, _)| t)
        .collect();
    assert_eq!(offscreen.len(), 2, "{:?}", earcons(&compiled));
    assert!(
        near(offscreen[0], 0.0) && near(offscreen[1], 25.0 / 3.0),
        "{offscreen:?}"
    );
    let unwindowed = compile(&cued("x^2", [-3.0, 3.0]));
    assert!(
        earcons(&unwindowed)
            .iter()
            .all(|(_, k)| *k != EarconKind::Offscreen)
    );
}

/// A11Y-08, A11Y-24, ED-05: every cue a compile plays has a catalogue entry with a text
/// equivalent, and every cue names its features.
#[test]
fn a11y_08_every_cue_has_a_text_equivalent() {
    for (expr, domain) in [
        ("1/(x^2 - 4)", [-5.0, 5.0]),
        ("x^3 - 3*x", [-3.0, 3.0]),
        ("sqrt(x)", [-4.0, 4.0]),
        ("tan(x)", [-5.0, 5.0]),
    ] {
        let compiled = compile(&cued(expr, domain));
        let found = earcons(&compiled);
        assert!(!found.is_empty(), "{expr}");
        for (_, kind) in found {
            let entry = CATALOGUE.iter().find(|info| info.compiled == Some(kind));
            assert!(
                entry.is_some_and(|info| !info.text.is_empty()),
                "{expr}: {kind:?}"
            );
        }
    }
    let undefined = earcons(&compile(&cued("sqrt(x)", [-4.0, 4.0])));
    assert_eq!(
        undefined.first().map(|(_, k)| *k),
        Some(EarconKind::Undefined)
    );
}

/// A11Y-14: with AUDAXIS's spatial shaper on, the cue track carries nothing in pan, and no
/// pan-only or timbre-only warning names it.
#[test]
fn a11y_14_cues_carry_nothing_in_pan() {
    let compiled = compile_with(&cued("1/(x^2 - 4)", [-5.0, 5.0]), true);
    let track = cue_track(&compiled).unwrap();
    assert!(track.pan.0.abs() < f64::EPSILON);
    assert!(
        compiled
            .diagnostics
            .iter()
            .all(
                |d| !(d.code.as_str() == "SNR-ENG-100" || d.code.as_str() == "SNR-ENG-101")
                    || !d.message.contains(CUE_TRACK)
            ),
        "{:?}",
        compiled.diagnostics
    );
}

/// RFC-0007 §4: a Score with the new earcons records IR version 2; one without keeps 1.
#[test]
fn rfc_0007_scores_record_the_lowest_ir_version_that_reads_them() {
    let cued_score = compile(&cued("1/(x^2 - 4)", [-5.0, 5.0])).score;
    assert_eq!(cued_score.header.ir_version, 2);
    let landmarks_only = compile(&cued("x^2 + 1", [-1.0, 1.0])).score;
    assert_eq!(landmarks_only.header.ir_version, 1);
}
