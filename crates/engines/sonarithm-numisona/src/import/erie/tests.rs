// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Unit tests of the Erie importer's conversions and error cases. The gallery subset is
//! tested end to end in `tests/erie_gallery.rs`.

use serde_json::json;

use super::*;

fn one_spec(erie: &Value) -> (Value, ImportReport) {
    let import = import_value(erie).unwrap();
    assert_eq!(import.specs.len(), 1);
    (import.specs[0].clone(), import.report)
}

fn stream(encoding: &Value) -> Value {
    json!({
        "data": { "values": [ { "x": 1, "y": 2 }, { "x": 3, "y": 5 } ] },
        "tone": { "type": "default" },
        "encoding": encoding,
    })
}

#[test]
fn note_names_convert_to_hertz() {
    let hz = |name: &str| pitch_hz(&json!(name)).unwrap();
    assert!((hz("A4") - 440.0).abs() < 1e-9);
    assert!((hz("C4") - 261.625_565).abs() < 1e-5);
    assert!((hz("F#3") - 184.997_211).abs() < 1e-5);
    assert!((hz("Bb2") - 116.540_940).abs() < 1e-5);
    assert_eq!(pitch_hz(&json!(523.25)), Some(523.25));
    assert_eq!(pitch_hz(&json!("H4")), None);
    assert_eq!(pitch_hz(&json!("C")), None);
}

#[test]
fn track_ids_follow_the_spec_syntax() {
    assert_eq!(track_id(Some("stream-1"), 0), "stream-1");
    assert_eq!(track_id(Some("Body Mass (g)"), 0), "Body-Mass--g");
    assert_eq!(track_id(Some("2nd"), 0), "s2nd");
    assert_eq!(track_id(None, 2), "stream3");
    assert_eq!(track_id(Some(&"a".repeat(80)), 0).len(), 64);
}

#[test]
fn javascript_expressions_are_translated_and_reported() {
    let mut report = ImportReport::default();
    let translate = |expr: &str, report: &mut ImportReport| {
        expression(expr, &Ptr::root(), "transform.filter", report)
    };
    assert_eq!(
        translate("datum.cost > 30 && datum.kind === 'a'", &mut report),
        "cost > 30 && kind == \"a\""
    );
    assert_eq!(
        translate("Math.abs(datum['net cost']) !== 0", &mut report),
        "abs(col(\"net cost\")) != 0"
    );
    assert_eq!(report.entries().len(), 2);
    assert_eq!(report.with(Outcome::Approximated).count(), 2);
}

#[test]
fn reversed_ranges_and_domains_become_polarity() {
    let (spec, _) = one_spec(&stream(&json!({
        "pitch": { "field": "y", "scale": { "range": [800, 200] } },
        "pan": { "field": "x", "scale": { "domain": [3, 1], "polarity": "negative" } },
    })));
    let channels = &spec["tracks"][0]["channels"];
    assert_eq!(channels["pitch"]["range"], json!({ "hz": [200.0, 800.0] }));
    assert_eq!(channels["pitch"]["polarity"], json!("negative"));
    // A reversed domain and an explicit negative polarity cancel out.
    assert_eq!(channels["pan"]["scale"]["domain"], json!([1.0, 3.0]));
    assert_eq!(channels["pan"]["polarity"], json!("positive"));
}

#[test]
fn gains_become_decibels_within_minus_60_and_0() {
    let (spec, report) = one_spec(&stream(&json!({
        "loudness": { "field": "y", "scale": { "range": [0, 2] } },
    })));
    assert_eq!(
        spec["tracks"][0]["channels"]["loudness"]["range"],
        json!({ "db": [-60.0, 0.0] })
    );
    assert_eq!(report.entries()[0].construct, "encoding.loudness");
    let (spec, report) = one_spec(&stream(&json!({ "loudness": { "value": 0.25 } })));
    assert_eq!(
        spec["tracks"][0]["channels"]["loudness"]["value"],
        json!({ "db": -12.04 })
    );
    assert!(report.is_empty());
}

#[test]
fn detune_without_pitch_becomes_pitch_in_cents() {
    let (spec, report) = one_spec(&stream(&json!({
        "detune": { "field": "y", "scale": { "range": [0, 1200] } },
    })));
    assert_eq!(
        spec["tracks"][0]["channels"]["pitch"]["range"],
        json!({ "cents": [0.0, 1200.0] })
    );
    assert_eq!(report.entries()[0].construct, "encoding.detune");
}

#[test]
fn continued_tones_become_glides_without_durations_or_taps() {
    let erie = json!({
        "data": { "values": [ { "x": 1, "y": 2 } ] },
        "tone": { "type": "sawtooth", "continued": true },
        "encoding": {
            "pitch": { "field": "y" },
            "duration": { "field": "y" },
            "tapCount": { "field": "y" },
        },
    });
    let (spec, report) = one_spec(&erie);
    let track = &spec["tracks"][0];
    assert_eq!(track["mark"], json!("glide"));
    assert_eq!(track["instrument"], json!("saw"));
    assert!(track["channels"].get("duration").is_none());
    let unmapped: Vec<&str> = report
        .with(Outcome::Unmapped)
        .map(|e| e.construct.as_str())
        .collect();
    assert_eq!(unmapped, ["encoding.duration", "encoding.tapCount"]);
}

#[test]
fn network_urls_are_not_downloaded() {
    let erie = json!({
        "data": { "url": "https://example.org/data/cars.json" },
        "tone": { "type": "default" },
        "encoding": { "pitch": { "field": "mpg" } },
    });
    let (spec, report) = one_spec(&erie);
    assert_eq!(
        spec["data"]["source"],
        json!({ "kind": "file", "path": "cars.json" })
    );
    assert_eq!(report.entries()[0].construct, "data.url");
}

#[test]
fn an_overlay_over_different_datasets_imports_one_spec_per_stream() {
    let erie = json!({
        "overlay": [
            { "name": "a", "data": { "values": [ { "y": 1 } ] }, "tone": {}, "encoding": { "pitch": { "field": "y" } } },
            { "name": "b", "data": { "values": [ { "y": 2 } ] }, "tone": {}, "encoding": { "pitch": { "field": "y" } } },
        ],
    });
    let import = import_value(&erie).unwrap();
    assert_eq!(
        (import.composition, import.specs.len()),
        (Composition::Overlay, 2)
    );
    assert_eq!(import.report.entries()[0].construct, "overlay");
}

#[test]
fn unusable_specs_are_errors() {
    assert!(matches!(import("{"), Err(ImportError::Json(_))));
    assert_eq!(import("[1]"), Err(ImportError::NotAnObject));
    assert_eq!(
        import(r#"{"sequence": []}"#),
        Err(ImportError::NoStreams("sequence"))
    );
    assert!(matches!(
        import(r#"{"tone": {}, "encoding": {}}"#),
        Err(ImportError::NoData { .. })
    ));
    assert!(matches!(
        import(r#"{"data": {"name": "missing"}, "encoding": {}}"#),
        Err(ImportError::NoData { .. })
    ));
    // Streaming data without test data has nothing to import yet (SONOTRACE, B06).
    assert!(matches!(
        import(r#"{"data": {"stream": true}, "encoding": {}}"#),
        Err(ImportError::NoData { .. })
    ));
}

#[test]
fn unknown_keys_are_reported_not_dropped() {
    let erie = json!({
        "data": { "values": [ { "y": 1 } ], "format": { "type": "csv" } },
        "tone": { "type": "default", "hasBaseTone": true },
        "encoding": { "pitch": { "field": "y", "condition": { "test": "d.y > 0", "value": 300 } }, "modulation": { "field": "y" } },
        "x-vendor": 1,
    });
    let (_, report) = one_spec(&erie);
    let constructs: Vec<&str> = report
        .entries()
        .iter()
        .map(|e| e.construct.as_str())
        .collect();
    for expected in [
        "x-vendor",
        "data.format",
        "tone.hasBaseTone",
        "encoding.pitch.condition",
        "encoding.modulation",
    ] {
        assert!(
            constructs.contains(&expected),
            "{expected} not in {constructs:?}"
        );
    }
}
