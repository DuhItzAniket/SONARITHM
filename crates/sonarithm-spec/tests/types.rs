// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The serde types accept the RFC-0001 shapes and reject unknown keys. The test that reads
//! every example straight from docs/architecture/04 is `doc_examples.rs` (A03-T11).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use sonarithm_spec::sections::Pace;
use sonarithm_spec::values::{ChannelRange, ChannelValue, DurationValue, UnitRange, UnitValue};
use sonarithm_spec::{JsonValue, Mark, Source, Spec};

fn parse(json: &str) -> Spec {
    serde_json::from_str(json).unwrap()
}

const TWO_SERIES: &str = r#"{
  "specVersion": "0.1", "mode": "analysis", "seed": 7,
  "data": { "source": { "kind": "file", "path": "co2_temp.csv" } },
  "substrate": { "time": { "field": "year", "pace": { "duration": { "s": 24 } } } },
  "tracks": [
    { "id": "co2",  "mark": "tone", "instrument": "mallet",
      "channels": { "pitch": { "field": "co2_ppm" }, "pan": { "value": -0.5 } } },
    { "id": "temp", "mark": "tone", "instrument": "clarinet-like",
      "channels": { "pitch": { "field": "anomaly", "polarity": "positive" }, "pan": { "value": 0.5 } },
      "redundant": ["taps"] }
  ],
  "context": { "ticks": { "axis": "x", "every": 10 }, "reference": { "kind": "dynamic-extreme", "track": "temp" } }
}"#;

#[test]
fn a_full_spec_round_trips_through_serde() {
    let spec = parse(TWO_SERIES);
    assert_eq!(spec.tracks.len(), 2);
    assert_eq!(spec.tracks[1].mark, Mark::Tone);
    assert!(matches!(spec.data.source, Source::File { .. }));
    let pace = spec
        .substrate
        .as_ref()
        .and_then(|s| s.time.as_ref())
        .and_then(|t| t.pace);
    assert_eq!(pace, Some(Pace::Duration(DurationValue::S(24.0))));
    let pan = spec.tracks[0]
        .channels
        .pan
        .as_ref()
        .and_then(|c| c.value.clone());
    assert_eq!(pan, Some(ChannelValue::Number(-0.5)));
    let again = parse(&serde_json::to_string(&spec).unwrap());
    assert_eq!(again, spec);
}

#[test]
fn unit_values_and_ranges_take_exactly_one_unit_key() {
    let spec = parse(
        r#"{ "specVersion": "0.1", "data": { "source": { "kind": "inline", "values": [{ "x": 1, "y": "a" }] } },
             "tracks": [{ "id": "t", "mark": "tone", "channels": {
               "pitch": { "field": "x", "range": { "hz": [400, 1400] } },
               "duration": { "value": { "ms": 250 } },
               "timbre": { "value": "mallet" } } }] }"#,
    );
    let channels = &spec.tracks[0].channels;
    let range = channels.pitch.as_ref().and_then(|c| c.range);
    assert_eq!(
        range,
        Some(ChannelRange::Unit(UnitRange::Hz([400.0, 1400.0])))
    );
    let duration = channels.duration.as_ref().and_then(|c| c.value.clone());
    assert_eq!(duration, Some(ChannelValue::Unit(UnitValue::Ms(250.0))));
    let timbre = channels.timbre.as_ref().and_then(|c| c.value.clone());
    assert_eq!(timbre, Some(ChannelValue::Name("mallet".to_owned())));
    let Source::Inline { values } = &spec.data.source else {
        panic!("inline source expected");
    };
    assert_eq!(values[0]["y"], JsonValue::String("a".to_owned()));

    let two_units = r#"{ "specVersion": "0.1", "data": { "source": { "kind": "file", "path": "a.csv" } },
        "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "value": { "hz": 1, "midi": 2 } } } }] }"#;
    assert!(serde_json::from_str::<Spec>(two_units).is_err());
}

#[test]
fn unknown_keys_are_rejected_below_the_top_level() {
    let typo = r#"{ "specVersion": "0.1", "data": { "source": { "kind": "file", "path": "a.csv" } },
        "tracks": [{ "id": "t", "mark": "tone", "channels": { "pich": { "field": "y" } } }] }"#;
    let err = serde_json::from_str::<Spec>(typo).unwrap_err().to_string();
    assert!(err.contains("unknown field `pich`"), "{err}");
    let source_typo = r#"{ "specVersion": "0.1", "data": { "source": { "kind": "file", "pth": "a.csv" } },
        "tracks": [] }"#;
    assert!(serde_json::from_str::<Spec>(source_typo).is_err());
}

#[test]
fn top_level_extensions_are_kept_for_validation() {
    let spec = parse(
        r#"{ "specVersion": "0.1", "x-tool": { "v": 2 }, "colour": 1,
             "data": { "source": { "kind": "file", "path": "a.csv" } }, "tracks": [] }"#,
    );
    let keys: Vec<&str> = spec.extensions.keys().map(String::as_str).collect();
    assert_eq!(keys, ["colour", "x-tool"]);
}

#[test]
fn spec_version_must_be_major_dot_minor() {
    let bad = r#"{ "specVersion": "0.1.0", "data": { "source": { "kind": "file", "path": "a.csv" } }, "tracks": [] }"#;
    assert!(serde_json::from_str::<Spec>(bad).is_err());
}
