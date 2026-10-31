// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! proptest generators of valid specs, shared with other crates through the non-default
//! `test-support` feature (testing-strategy §3). They also seed the fuzz corpus.

use proptest::collection::vec;
use proptest::prelude::*;
use serde_json::{Map, Value, json};

use crate::parse::parse_value;
use crate::spec::Spec;

/// The marks the generator uses, with the channels each accepts (04 §5.3).
const MARKS: &[(&str, &[&str])] = &[
    (
        "tone",
        &[
            "pitch",
            "duration",
            "loudness",
            "pan",
            "timbre",
            "brightness",
            "taps",
        ],
    ),
    (
        "glide",
        &["pitch", "loudness", "pan", "timbre", "brightness", "rate"],
    ),
    (
        "grain",
        &["density", "pitch", "duration", "pan", "brightness"],
    ),
    ("audify", &["loudness"]),
    ("sample", &["loudness", "pan", "rate"]),
];

/// A valid constant or mapping for `channel`.
fn channel(channel: &'static str) -> BoxedStrategy<Value> {
    match channel {
        "pitch" => prop_oneof![
            (20.0_f64..8000.0, 1.0_f64..4000.0, prop::bool::ANY).prop_map(
                |(low, width, negative)| {
                    json!({
                        "field": "v",
                        "range": { "hz": [low, (low + width).min(20_000.0)] },
                        "polarity": if negative { "negative" } else { "positive" }
                    })
                }
            ),
            (20.0_f64..20_000.0).prop_map(|hz| json!({ "value": { "hz": hz } })),
            (-48.0_f64..48.0).prop_map(|midi| json!({ "value": { "cents": midi * 100.0 } })),
        ]
        .boxed(),
        "duration" => (0.001_f64..4.0)
            .prop_map(|s| json!({ "value": { "s": s } }))
            .boxed(),
        "loudness" => (-60.0_f64..=0.0)
            .prop_map(|db| json!({ "value": { "db": db } }))
            .boxed(),
        "pan" => (-1.0_f64..=1.0)
            .prop_map(|pan| json!({ "value": pan }))
            .boxed(),
        "brightness" => (0.0_f64..=1.0).prop_map(|b| json!({ "value": b })).boxed(),
        "taps" => (1_u8..=8).prop_map(|taps| json!({ "value": taps })).boxed(),
        "timbre" => prop_oneof![
            Just(json!({ "value": "mallet" })),
            Just(json!({ "field": "g" })),
        ]
        .boxed(),
        _ => (0.1_f64..100.0).prop_map(|x| json!({ "value": x })).boxed(),
    }
}

fn track(index: usize) -> impl Strategy<Value = Value> {
    prop::sample::select(MARKS).prop_flat_map(move |(mark, accepted)| {
        let channels = prop::sample::subsequence(accepted.to_vec(), 0..=accepted.len());
        channels.prop_flat_map(move |names| {
            let strategies: Vec<_> = names.iter().map(|name| channel(name)).collect();
            (Just(names), strategies).prop_map(move |(names, values)| {
                let channels: Map<String, Value> = names
                    .iter()
                    .map(|name| (*name).to_owned())
                    .zip(values)
                    .collect();
                json!({ "id": format!("t{index}"), "mark": mark, "channels": channels })
            })
        })
    })
}

/// A JSON document that parses and validates without errors.
pub fn valid_spec_json() -> impl Strategy<Value = Value> {
    (
        (1_usize..4, any::<bool>(), 0_u8..=127),
        prop::sample::select(&["analysis", "presentation", "monitoring"][..]),
        0_u64..(1 << 53),
        0.5_f64..8.0,
        prop::sample::select(&[8_000_u32, 44_100, 48_000, 96_000][..]),
        4_u32..=13,
        vec((-1e6_f64..1e6, "[a-z]{1,6}"), 1..8),
    )
        .prop_flat_map(|((tracks, v0_2, program), mode, seed, pace, rate, block, rows)| {
            let tracks: Vec<_> = (0..tracks).map(track).collect();
            (tracks, Just((mode, seed, pace, rate, block, rows, v0_2, program)))
        })
        .prop_map(|(tracks, (mode, seed, pace, rate, block, rows, v0_2, program))| {
            let values: Vec<Value> = rows
                .into_iter()
                .map(|(v, g)| json!({ "v": v, "g": g }))
                .collect();
            let mut spec = json!({
                "specVersion": "0.1",
                "mode": mode,
                "seed": seed,
                "data": { "source": { "kind": "inline", "values": values } },
                "substrate": { "time": { "pace": { "pointsPerSecond": pace } } },
                "tracks": tracks,
                "output": { "sampleRate": rate, "blockSize": 1_u32 << block },
                "x-generator": { "name": "proptest" }
            });
            if mode == "presentation" {
                spec["music"] = json!({ "tempo": { "bpm": 96 } });
            }
            // Half the specs are v0.2 with its sections (RFC-0004 §11).
            if v0_2 {
                spec["specVersion"] = json!("0.2");
                spec["instruments"] = json!({ "piano": { "kind": "soundfont", "path": "sounds/piano.sf2", "program": program } });
                spec["lint"] = json!({ "rules": { "SDR-17": "off" }, "suppress": [{ "rule": "SDR-19", "reason": "generated" }] });
                if mode == "presentation" {
                    spec["music"]["quantize"] = json!({ "scale": "D minor pentatonic" });
                }
            }
            spec
        })
}

/// A valid spec value.
pub fn valid_spec() -> impl Strategy<Value = Spec> {
    valid_spec_json().prop_filter_map("generated spec must parse", |value| parse_value(value).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::to_canonical;
    use crate::migrate::migrate;
    use crate::prepare::prepare;
    use crate::validate::{FieldCatalog, validate};

    proptest! {
        /// Generated specs validate without errors (the generator stays honest).
        #[test]
        fn generated_specs_are_valid(spec in valid_spec()) {
            let catalog = FieldCatalog::from_spec(&spec);
            let errors: Vec<_> = validate(&spec, catalog.as_ref()).into_iter().filter(sonarithm_core::Diagnostic::is_error).collect();
            prop_assert!(errors.is_empty(), "{errors:#?}");
        }

        /// testing-strategy §3: parse → serialize → parse gives an equal spec.
        #[test]
        fn specs_round_trip_through_json(spec in valid_spec()) {
            let text = serde_json::to_string(&spec).unwrap();
            prop_assert_eq!(crate::parse(&text).unwrap(), spec);
        }

        /// The canonical form prepares to itself: defaults and canonicalization are
        /// idempotent, so the spec hash is a fixed point.
        #[test]
        fn the_canonical_form_is_a_fixed_point(spec in valid_spec()) {
            let text = serde_json::to_string(&spec).unwrap();
            let prepared = prepare(&text, None).unwrap();
            let again = prepare(&prepared.canonical, None).unwrap();
            prop_assert_eq!(&again.canonical, &prepared.canonical);
            prop_assert_eq!(again.hash, prepared.hash);
            prop_assert_eq!(to_canonical(&again.resolved).unwrap(), prepared.canonical);
        }

        /// testing-strategy §3 and RFC-0004 §9: migration changes `specVersion` only, and
        /// migrating twice changes nothing more.
        #[test]
        fn migration_changes_only_the_version_and_is_idempotent(value in valid_spec_json()) {
            let migrated = migrate(value.clone()).unwrap();
            let mut expected = value;
            expected["specVersion"] = json!("0.4");
            prop_assert_eq!(&migrated.value, &expected);
            prop_assert_eq!(migrate(migrated.value.clone()).unwrap().value, migrated.value);
        }

        /// RFC-0004 §9: a v0.1 document is read as it is, so its spec hash does not change.
        #[test]
        fn reading_v0_1_keeps_its_version_and_hash(value in valid_spec_json()) {
            let text = value.to_string();
            let prepared = prepare(&text, None).unwrap();
            prop_assert_eq!(prepared.spec.spec_version.to_string(), value["specVersion"].as_str().unwrap());
            prop_assert!(!prepared.diagnostics.iter().any(|d| d.code.as_str() == "SNR-SPEC-006"));
        }
    }
}
