// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A06-T04: the `tone` and `glide` encoders through the real compiler, on the example
//! datasets: snapshots of the review form, and the rules the encoders enforce.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::unnecessary_wraps,
    reason = "integration test; exact times and pitch-range ends are part of the contract, and row numbers are small"
)]

mod support;

use serde_json::json;
use sonarithm_core::score::ScoreEvent;
use sonarithm_data::DataBundle;
use sonarithm_numisona::Numisona;
use sonarithm_numisona::examples;
use sonarithm_numisona::polarity::PolarityProfile;
use support::{compile, compile_with, has_code, notes, review};

fn spec(tracks: &serde_json::Value) -> String {
    json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "file", "path": "example.json" } },
        "tracks": tracks,
    })
    .to_string()
}

fn spec_with(extra: &serde_json::Value, tracks: &serde_json::Value) -> String {
    let mut value = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "file", "path": "example.json" } },
        "tracks": tracks,
    });
    for (key, item) in extra.as_object().unwrap() {
        value[key] = item.clone();
    }
    value.to_string()
}

fn tone(channels: &serde_json::Value) -> serde_json::Value {
    json!([{ "id": "data", "mark": "tone", "channels": channels }])
}

fn temperature() -> Option<DataBundle> {
    Some(examples::temperature().unwrap())
}

/// A06-T04 acceptance: snapshots of the encoders on the example datasets.
#[test]
fn the_example_datasets_encode_as_reviewed() {
    let cases: Vec<(&str, String, DataBundle)> = vec![
        (
            "temperature-tone",
            spec(&tone(&json!({ "pitch": { "field": "temp" } }))),
            examples::temperature().unwrap(),
        ),
        (
            "temperature-glide",
            spec(
                &json!([{ "id": "data", "mark": "glide", "channels": { "pitch": { "field": "temp" } } }]),
            ),
            examples::temperature().unwrap(),
        ),
        (
            "prices-redundant-taps",
            spec(
                &json!([{ "id": "price", "mark": "tone", "channels": { "pitch": { "field": "price", "polarity": "positive" } }, "redundant": ["taps"] }]),
            ),
            examples::prices().unwrap(),
        ),
        (
            "regions-north",
            spec_with(
                &json!({ "substrate": { "time": { "field": "month", "pace": { "pointsPerSecond": 2 } } } }),
                &json!([{ "id": "north", "mark": "tone", "channels": { "pitch": { "field": "rain" }, "pan": { "value": -0.5 } }, "filter": { "expr": "region == \"north\"" } }]),
            ),
            examples::regions().unwrap(),
        ),
        (
            "fruit-bars",
            spec(&tone(
                &json!({ "pitch": { "field": "sold", "scale": { "domain": "zero-based" } }, "timbre": { "field": "fruit" } }),
            )),
            examples::fruit().unwrap(),
        ),
        (
            "estimated",
            spec(&tone(&json!({ "pitch": { "field": "y" } }))),
            examples::estimated().unwrap(),
        ),
    ];
    for (name, spec, data) in cases {
        let compiled = compile(&spec, Some(data));
        insta::assert_snapshot!(name, review(&compiled));
    }
}

/// SDR-07 (DEFAULT) and the minimal spec of 04 §6.1: pitch carries the value; higher
/// temperatures sound higher (positive concept, SDR-12).
#[test]
fn sdr_07_pitch_rises_with_the_value() {
    let compiled = compile(
        &spec(&tone(&json!({ "pitch": { "field": "temp" } }))),
        temperature(),
    );
    let notes = notes(&compiled.score);
    let bundle = examples::temperature().unwrap();
    let table = bundle.primary().unwrap();
    let temps = table.column("temp").unwrap();
    let temps = temps
        .as_any()
        .downcast_ref::<arrow_array::Float64Array>()
        .unwrap();
    for pair in notes.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let (ta, tb) = (
            temps.value(a.src.unwrap().row as usize),
            temps.value(b.src.unwrap().row as usize),
        );
        if ta < tb {
            assert!(a.pitch.0 < b.pitch.0);
        }
    }
    assert_eq!(notes.len(), 120);
}

/// SDR-10 (MUST): Analysis mode keeps pitch continuous; NUMISONA never rounds to notes.
#[test]
fn sdr_10_analysis_mode_pitch_is_continuous() {
    let compiled = compile(
        &spec(&tone(&json!({ "pitch": { "field": "temp" } }))),
        temperature(),
    );
    let off_grid = notes(&compiled.score)
        .iter()
        .filter(|note| (note.pitch.0 / 100.0).fract().abs() > 1e-6)
        .count();
    assert!(
        off_grid > 100,
        "{off_grid} of 120 pitches are off the semitone grid"
    );
    assert_eq!(compiled.score.header.shapers.len(), 0);
}

/// A11Y-13 (SHOULD): `redundant` adds tap-count and duration encodings of the pitch field.
/// They rise with the value whatever the pitch polarity, within 1–8 taps and 0.1–0.3 s.
#[test]
fn a11y_13_redundant_taps_and_duration_encode_the_pitch_field() {
    for polarity in ["positive", "negative"] {
        let compiled = compile(
            &spec(&json!([{
                "id": "price", "mark": "tone",
                "channels": { "pitch": { "field": "price", "polarity": polarity } },
                "redundant": ["taps", "duration"],
            }])),
            Some(examples::prices().unwrap()),
        );
        let mut notes = notes(&compiled.score);
        assert!(has_code(&compiled, "SNR-ENG-007"));
        let bundle = examples::prices().unwrap();
        let prices = bundle.primary().unwrap().column("price").unwrap();
        let prices = prices
            .as_any()
            .downcast_ref::<arrow_array::Float64Array>()
            .unwrap();
        notes.sort_by(|a, b| {
            prices
                .value(a.src.unwrap().row as usize)
                .total_cmp(&prices.value(b.src.unwrap().row as usize))
        });
        for pair in notes.windows(2) {
            assert!(pair[0].taps <= pair[1].taps, "{polarity}");
            assert!(pair[0].dur.0 <= pair[1].dur.0, "{polarity}");
        }
        let (low, high) = (&notes[0], &notes[notes.len() - 1]);
        assert_eq!((low.taps, high.taps), (1, 8), "{polarity}");
        assert!((low.dur.0 - 0.1).abs() < 1e-12 && (high.dur.0 - 0.3).abs() < 1e-12);
    }
    // An explicit taps channel wins over the redundant one (RFC-0001 Q21).
    let compiled = compile(
        &spec(
            &json!([{ "id": "price", "mark": "tone", "channels": { "pitch": { "field": "price", "polarity": "positive" }, "taps": { "value": 2 } }, "redundant": ["taps"] }]),
        ),
        Some(examples::prices().unwrap()),
    );
    assert!(notes(&compiled.score).iter().all(|note| note.taps == 2));
    assert!(has_code(&compiled, "SNR-ENG-056"));
}

/// 05 §4.2 invariant 4 and SDR-15 (MUST): negative polarity, and the profile's one-key
/// flip, mirror every pitch within the range and change nothing else.
#[test]
fn sdr_15_polarity_flip_symmetry_in_the_score() {
    let encode = |polarity: &str, profile: PolarityProfile| {
        let spec = spec(&tone(
            &json!({ "pitch": { "field": "temp", "polarity": polarity } }),
        ));
        compile_with(&spec, temperature(), &Numisona::with_profile(profile)).unwrap()
    };
    let positive = encode("positive", PolarityProfile::default());
    let negative = encode("negative", PolarityProfile::default());
    let mut flips = PolarityProfile::default();
    flips.flipped_fields.insert("temp".to_owned());
    let flipped = encode("positive", flips);
    let range = positive.score.tracks[0].pitch_range.unwrap();
    assert_eq!(range, negative.score.tracks[0].pitch_range.unwrap());
    let (p, n, f) = (
        notes(&positive.score),
        notes(&negative.score),
        notes(&flipped.score),
    );
    for ((p, n), f) in p.iter().zip(&n).zip(&f) {
        assert!((n.pitch.0 - range.mirror(p.pitch).0).abs() < 1e-9);
        assert_eq!(n.pitch, f.pitch);
        let mut same = n.clone();
        same.pitch = p.pitch;
        assert_eq!(&same, p);
    }
}

/// Every note points back at its row and keeps its confidence (03 §4, AGENTS.md rule 7).
#[test]
fn notes_keep_their_rows_and_confidence() {
    let compiled = compile(
        &spec(&tone(&json!({ "pitch": { "field": "y" } }))),
        Some(examples::estimated().unwrap()),
    );
    for (i, note) in notes(&compiled.score).iter().enumerate() {
        let src = note.src.unwrap();
        assert_eq!(src.row, i as u64);
        let estimated = matches!(i, 3 | 4 | 7);
        assert_eq!(
            note.conf.map(sonarithm_core::Confidence::get),
            estimated.then_some(0.75),
            "row {i}"
        );
    }
}

/// A track filter keeps the original table's row references.
#[test]
fn track_filters_keep_original_row_references() {
    let compiled = compile(
        &spec(
            &json!([{ "id": "south", "mark": "tone", "channels": { "pitch": { "field": "rain" } }, "filter": { "expr": "region == \"south\"" } }]),
        ),
        Some(examples::regions().unwrap()),
    );
    let notes = notes(&compiled.score);
    assert_eq!(notes.len(), 12);
    assert!(notes.iter().all(|note| note.src.unwrap().row % 3 == 1));
    let bad = compile_with(
        &spec(
            &json!([{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "rain" } }, "filter": { "expr": "nonsense((" } }]),
        ),
        Some(examples::regions().unwrap()),
        &Numisona::new(),
    );
    assert!(bad.is_err());
}

/// The three paces: even spacing by rate or total duration, and spacing in proportion to
/// the time field.
#[test]
fn paces_space_points_evenly_or_by_value() {
    let onsets = |pace: serde_json::Value| {
        let spec = spec_with(
            &json!({ "substrate": { "time": { "field": "day", "pace": pace } } }),
            &tone(&json!({ "pitch": { "field": "temp" } })),
        );
        notes(&compile(&spec, temperature()).score)
            .iter()
            .map(|n| n.t.0)
            .collect::<Vec<f64>>()
    };
    let by_rate = onsets(json!({ "pointsPerSecond": 2 }));
    assert_eq!(by_rate[3], 1.5);
    let by_total = onsets(json!({ "duration": { "s": 60 } }));
    assert_eq!(by_total[1], 0.5);
    assert_eq!(*by_total.last().unwrap(), 59.5);
    let by_value = onsets(json!({ "secondsPerUnit": 0.25 }));
    assert_eq!(by_value[8], 2.0);
}

/// 03 §5: missing values are silent by default, or interpolated; glides split at gaps and
/// hold their last value for one step.
#[test]
fn missing_values_are_silent_interpolated_or_split_glides() {
    let data = |missing: &str, mark: &str| {
        json!({
            "specVersion": "0.1",
            "data": {
                "source": { "kind": "inline", "values": [
                    { "x": 0, "y": 1.0 }, { "x": 1, "y": 2.0 }, { "x": 2, "y": null },
                    { "x": 3, "y": 4.0 }, { "x": 4, "y": 3.0 }
                ] },
                "missing": missing,
            },
            "substrate": { "time": { "field": "x", "pace": { "pointsPerSecond": 4 } } },
            "tracks": [{ "id": "y", "mark": mark, "channels": { "pitch": { "field": "y" } } }],
        })
        .to_string()
    };
    let silent = compile(&data("silence", "tone"), None);
    assert_eq!(notes(&silent.score).len(), 4);
    assert!(has_code(&silent, "SNR-ENG-010"));
    let filled = compile(&data("interpolate", "tone"), None);
    let filled_notes = notes(&filled.score);
    assert_eq!(filled_notes.len(), 5);
    let mid = f64::midpoint(filled_notes[1].pitch.0, filled_notes[3].pitch.0);
    assert!((filled_notes[2].pitch.0 - mid).abs() < 1e-9);
    let glides = compile(&data("silence", "glide"), None);
    let contours: Vec<_> = glides.score.tracks[0]
        .events
        .iter()
        .filter_map(|event| match event {
            ScoreEvent::Glide(glide) => Some(glide.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(contours.len(), 2);
    assert_eq!((contours[0].t0.0, contours[0].t1.0), (0.0, 0.5));
    assert_eq!((contours[1].t0.0, contours[1].t1.0), (0.75, 1.25));
    assert_eq!(contours[1].curve.len(), 3);
}

/// SDR-25 (DEFAULT): a nominal field on timbre gives each category its own preset, the
/// track instrument first.
#[test]
fn sdr_25_categories_map_to_timbre() {
    let compiled = compile(
        &spec(&tone(
            &json!({ "pitch": { "field": "sold" }, "timbre": { "field": "fruit" } }),
        )),
        Some(examples::fruit().unwrap()),
    );
    let presets: Vec<Option<String>> = notes(&compiled.score)
        .iter()
        .map(|note| note.timbre.preset.as_ref().map(|p| p.0.clone()))
        .collect();
    assert_eq!(
        presets,
        [
            None,
            Some("square".into()),
            Some("triangle".into()),
            Some("saw".into()),
            Some("sine".into())
        ]
    );
    assert!(has_code(&compiled, "SNR-ENG-008"));
    // A categorical field on a continuous scale is refused with a pointer to timbre.
    let refused = compile_with(
        &spec(&tone(&json!({ "pitch": { "field": "fruit" } }))),
        Some(examples::fruit().unwrap()),
        &Numisona::new(),
    );
    let Err(sonarithm_compiler::Failure::Invalid(diagnostics)) = refused else {
        panic!("expected an error")
    };
    assert!(diagnostics.iter().any(|d| d.code.as_str() == "SNR-ENG-043"));
    // With a band scale, categories become pitches.
    let banded = compile(
        &spec(&tone(
            &json!({ "pitch": { "field": "fruit", "scale": { "type": "band" } } }),
        )),
        Some(examples::fruit().unwrap()),
    );
    assert_eq!(notes(&banded.score).len(), 5);
}

/// SDR-23 (MUST): loudness is constant by default; mapping data to it warns.
#[test]
fn sdr_23_loudness_is_constant_unless_mapped_and_then_warns() {
    let constant = compile(
        &spec(&tone(&json!({ "pitch": { "field": "temp" } }))),
        temperature(),
    );
    assert!(
        notes(&constant.score)
            .iter()
            .all(|note| note.gain.0 == -18.0)
    );
    assert!(has_code(&constant, "SNR-ENG-004"));
    let mapped = compile(
        &spec(&tone(
            &json!({ "pitch": { "field": "temp" }, "loudness": { "field": "temp" } }),
        )),
        temperature(),
    );
    assert!(has_code(&mapped, "SNR-ENG-051"));
    assert!(
        notes(&mapped.score)
            .iter()
            .all(|n| (-24.0..=-12.0).contains(&n.gain.0))
    );
}

/// A11Y-14 (MUST): pan must never be the only carrier of a field.
#[test]
fn a11y_14_pan_alone_warns() {
    let compiled = compile(
        &spec(&tone(
            &json!({ "pitch": { "field": "temp" }, "pan": { "field": "day" } }),
        )),
        temperature(),
    );
    assert!(has_code(&compiled, "SNR-ENG-052"));
    let constant = compile(
        &spec(&tone(
            &json!({ "pitch": { "field": "temp" }, "pan": { "value": 0.5 } }),
        )),
        temperature(),
    );
    assert!(!has_code(&constant, "SNR-ENG-052"));
}

/// SDR-27 (SHOULD): in Analysis mode, pan should not repeat the pitch field.
#[test]
fn sdr_27_pan_repeating_pitch_warns_in_analysis_mode() {
    let both = |mode: &str| {
        compile(
            &spec_with(
                &json!({ "mode": mode }),
                &tone(&json!({ "pitch": { "field": "temp" }, "pan": { "field": "temp" } })),
            ),
            temperature(),
        )
    };
    assert!(has_code(&both("analysis"), "SNR-ENG-053"));
    assert!(!has_code(&both("presentation"), "SNR-ENG-053"));
}

/// SDR-19 (DEFAULT): discrete tones faster than about 8 per second warn.
#[test]
fn sdr_19_fast_tone_paces_warn() {
    let paced = |rate: f64| {
        compile(
            &spec_with(
                &json!({ "substrate": { "time": { "pace": { "pointsPerSecond": rate } } } }),
                &tone(&json!({ "pitch": { "field": "temp" } })),
            ),
            temperature(),
        )
    };
    assert!(!has_code(&paced(3.0), "SNR-ENG-049"));
    assert!(has_code(&paced(12.0), "SNR-ENG-049"));
}

/// SDR-20 (INFER, experimental): taps shorter than about 100 ms may fuse, and the encoder
/// says so.
#[test]
fn sdr_20_taps_that_may_fuse_warn() {
    let compiled = compile(
        &spec(
            &json!([{ "id": "price", "mark": "tone", "channels": { "pitch": { "field": "price" } }, "redundant": ["taps"] }]),
        ),
        Some(examples::prices().unwrap()),
    );
    assert!(has_code(&compiled, "SNR-ENG-050"));
}

/// FR-026: every default the encoder applies is reported as `Info` with its rule.
#[test]
fn every_encoder_default_is_reported() {
    let compiled = compile(
        &spec(&tone(&json!({ "pitch": { "field": "temp" } }))),
        temperature(),
    );
    for code in [
        "SNR-ENG-001",
        "SNR-ENG-002",
        "SNR-ENG-003",
        "SNR-ENG-004",
        "SNR-ENG-020",
    ] {
        assert!(has_code(&compiled, code), "{code}");
    }
    let instrument = compiled
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "SNR-ENG-002")
        .unwrap();
    assert_eq!(instrument.rules[0].as_str(), "SDR-29");
}

/// Unknown fields and unreadable scales stop the compile with a located error.
#[test]
fn unknown_fields_are_located_errors() {
    let result = compile_with(
        &spec(&tone(&json!({ "pitch": { "field": "tmep" } }))),
        temperature(),
        &Numisona::new(),
    );
    let Err(sonarithm_compiler::Failure::Invalid(diagnostics)) = result else {
        panic!("expected an error")
    };
    let error = diagnostics
        .iter()
        .find(|d| d.code.as_str() == "SNR-ENG-041")
        .unwrap();
    assert_eq!(
        error.location.as_ref().unwrap().as_str(),
        "/tracks/0/channels/pitch/field"
    );
}

/// ADR-0013: the same input gives a byte-identical Score, and the canonical hash is pinned,
/// so it is checked on every CI platform.
#[test]
fn encoded_scores_hash_identically_on_every_platform() {
    let spec = spec(
        &json!([{ "id": "price", "mark": "tone", "channels": { "pitch": { "field": "price", "space": "erb" } }, "redundant": ["duration"] }]),
    );
    let first = compile(&spec, Some(examples::prices().unwrap()));
    let second = compile(&spec, Some(examples::prices().unwrap()));
    let hash = first.score.canonical_hash().unwrap();
    assert_eq!(hash, second.score.canonical_hash().unwrap());
    insta::assert_snapshot!("prices-erb-hash", hash.to_string());
}
