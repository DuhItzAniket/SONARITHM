// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC's detectors in a compile (C05-T01, C05-T02; RFC-0009 §1): they analyze the
//! fields the spec's tracks play, record the field and their parameters in every annotation,
//! and AUDAXIS links each insight to the series of its field.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use serde_json::{Value, json};
use sonarithm_audaxis::Audaxis;
use sonarithm_compiler::{Compiled, Compiler, Platform};
use sonarithm_core::engine::{AnnotationParam, CancelToken};
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree};
use sonarithm_core::provenance::Confidence;
use sonarithm_data::DataBundle;
use sonarithm_echometric::Echometric;
use sonarithm_numisona::Numisona;

/// Two series over 20 rows: a steady temperature, and a humidity with one spike at row 12.
fn two_series(detect: &[&str]) -> Value {
    let rows: Vec<Value> = (0..20_u32)
        .map(|i| {
            let temp = 10.0 + f64::from(i % 3) * 0.1;
            let humidity = if i == 12 {
                95.0
            } else {
                40.0 + f64::from(i % 4) * 0.5
            };
            json!({ "t": i, "temp": temp, "humidity": humidity })
        })
        .collect();
    json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "inline", "values": rows } },
        "substrate": { "time": { "field": "t" } },
        "tracks": [
            { "id": "temp", "mark": "tone", "channels": { "pitch": { "field": "temp" } } },
            { "id": "humidity", "mark": "tone", "channels": { "pitch": { "field": "humidity" } } }
        ],
        "analysis": { "detect": detect }
    })
}

fn compile(spec: &Value) -> Compiled {
    let platform = Platform::new();
    let numisona = Numisona::new();
    let echometric = Echometric;
    let audaxis = Audaxis::new();
    let compiler: Compiler<DataBundle> =
        Compiler::new(&[&platform, &numisona, &echometric, &audaxis]).unwrap();
    compiler
        .compile(&spec.to_string(), &CancelToken::new())
        .unwrap_or_else(|failure| panic!("{failure:?}"))
}

/// The label of the series node above `node`.
fn series_of(tree: &NavTree, mut node: NavNodeId) -> Option<String> {
    loop {
        let found = tree.node(node)?;
        if matches!(found.kind, NavKind::Series(_)) {
            return found.label.clone();
        }
        node = found.parent?;
    }
}

/// RFC-0009 §1's two-series test: the humidity outlier is about `humidity`, and its insight
/// links to the humidity series, not to the first series that covers row 12.
#[test]
fn a11y_23_an_outlier_in_the_second_series_links_to_that_series() {
    let compiled = compile(&two_series(&["outliers"]));
    let outliers: Vec<_> = compiled
        .annotations
        .iter()
        .filter(|a| a.label == "outlier")
        .collect();
    assert_eq!(outliers.len(), 1, "{:?}", compiled.annotations);
    let outlier = outliers[0];
    assert_eq!(outlier.field.as_deref(), Some("humidity"));
    assert_eq!((outlier.range.start, outlier.range.end), (12, 13));
    assert_eq!(
        outlier.params.get("method"),
        Some(&AnnotationParam::Text("moving-median-robust-z".to_owned()))
    );
    let tree = compiled.nav.as_ref().expect("AUDAXIS builds the tree");
    let target = tree
        .iter()
        .find_map(|(_, node)| match &node.kind {
            NavKind::Insight(insight) if node.label.as_deref() == Some("outlier") => insight.target,
            _ => None,
        })
        .expect("the outlier insight has a target");
    assert_eq!(series_of(tree, target).as_deref(), Some("humidity"));
}

/// Every annotation records its field and parameters (C05-T01's snapshot), the same way on
/// every compile (ADR-0013).
#[test]
fn c05_t01_annotations_carry_their_field_and_parameters() {
    let spec = two_series(&["extrema", "outliers", "trend-segments"]);
    let compiled = compile(&spec);
    let json = serde_json::to_string_pretty(&compiled.annotations).unwrap();
    // #520 review: on humidity's sawtooth with one spike, the segments rise and fall; with
    // the full range's band, the spike made every one "flat".
    let humidity: Vec<&str> = compiled
        .annotations
        .iter()
        .filter(|a| a.source.as_str() == "trend-segments" && a.field.as_deref() == Some("humidity"))
        .map(|a| a.label.as_str())
        .collect();
    assert!(
        humidity.contains(&"rising") && humidity.contains(&"falling"),
        "{humidity:?}"
    );
    assert_eq!(
        json,
        serde_json::to_string_pretty(&compile(&spec).annotations).unwrap(),
        "deterministic"
    );
    for annotation in &compiled.annotations {
        assert!(annotation.field.is_some(), "{annotation:?}");
        assert!(annotation.params.contains_key("method"), "{annotation:?}");
    }
    insta::assert_snapshot!("two_series_annotations", json);
}

/// A spec without `analysis` gets no annotations, and its Score does not change when
/// ECHOMETRIC is registered (RFC-0009 §5).
#[test]
fn rfc_0009_a_spec_without_detectors_is_unchanged() {
    let mut spec = two_series(&[]);
    spec.as_object_mut().unwrap().remove("analysis");
    let with = compile(&spec);
    assert_eq!(with.annotations, []);
    let platform = Platform::new();
    let numisona = Numisona::new();
    let audaxis = Audaxis::new();
    let without: Compiler<DataBundle> = Compiler::new(&[&platform, &numisona, &audaxis]).unwrap();
    let without = without
        .compile(&spec.to_string(), &CancelToken::new())
        .unwrap();
    assert_eq!(
        serde_json::to_string(&with.score).unwrap(),
        serde_json::to_string(&without.score).unwrap()
    );
}

/// DS-003's month-major table: three regions' rainfall, one row per region and month.
const DS_003: &str = include_str!("../../../../examples/data/ds-003-regions.csv");

/// The DS-003 spec (three tracks, each filtering one region) with `analysis.detect`.
fn ds_003(detect: &[&str]) -> (Value, std::path::PathBuf) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/specs");
    let text = std::fs::read_to_string(dir.join("ds-003-regions.spec.json")).unwrap();
    let mut spec: Value = serde_json::from_str(&text).unwrap();
    spec["analysis"] = json!({ "detect": detect });
    (spec, dir)
}

fn compile_in(spec: &Value, dir: &std::path::Path) -> Compiled {
    let platform = Platform::with_base_dir(dir);
    let numisona = Numisona::new();
    let echometric = Echometric;
    let audaxis = Audaxis::new();
    let compiler: Compiler<DataBundle> =
        Compiler::new(&[&platform, &numisona, &echometric, &audaxis]).unwrap();
    compiler
        .compile(&spec.to_string(), &CancelToken::new())
        .unwrap_or_else(|failure| panic!("{failure:?}"))
}

/// (region, rain) by row, from the CSV.
fn ds_003_rows() -> Vec<(String, f64)> {
    DS_003
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            (parts[1].to_owned(), parts[2].trim().parse().unwrap())
        })
        .collect()
}

/// #506's review: each track's detectors read that track's own rows in the order it plays
/// them, so each region's maximum is its own, linked to its own series (RFC-0009 §1).
#[test]
fn a11y_23_each_region_has_its_own_maximum_on_its_own_track() {
    let (spec, dir) = ds_003(&["extrema", "outliers"]);
    let compiled = compile_in(&spec, &dir);
    let rows = ds_003_rows();
    for region in ["north", "south", "east"] {
        let own_max = rows
            .iter()
            .filter(|(r, _)| r == region)
            .map(|(_, rain)| *rain)
            .fold(f64::NEG_INFINITY, f64::max);
        let maxima: Vec<_> = compiled
            .annotations
            .iter()
            .filter(|a| {
                a.label == "maximum" && a.track.as_ref().is_some_and(|t| t.as_str() == region)
            })
            .collect();
        assert_eq!(maxima.len(), 1, "{region}: {:?}", compiled.annotations);
        let maximum = maxima[0];
        assert_eq!(maximum.value, Some(own_max), "{region}");
        let row = usize::try_from(maximum.range.start).unwrap();
        assert_eq!(rows[row].0, region, "the maximum's row is the region's own");
        // Every annotation of the track is about its own rows.
        for annotation in compiled.annotations.iter().filter(|a| {
            a.track.as_ref().is_some_and(|t| t.as_str() == region) && a.range.len() == 1
        }) {
            let row = usize::try_from(annotation.range.start).unwrap();
            assert_eq!(rows[row].0, region, "{annotation:?}");
        }
    }
    // The tree links each region's maximum to that region's series.
    let tree = compiled.nav.as_ref().unwrap();
    let linked: Vec<String> = tree
        .iter()
        .filter_map(|(_, node)| match &node.kind {
            NavKind::Insight(insight) if node.label.as_deref() == Some("maximum") => {
                insight.target.and_then(|t| series_of(tree, t))
            }
            _ => None,
        })
        .collect();
    assert_eq!(linked.len(), 3, "{linked:?}");
}

/// The rows the detectors read for a track are the rows NUMISONA plays, in the same order:
/// `sonarithm_data::play_rows` against every note's source row in the encoded Score.
#[test]
fn rfc_0009_analyzed_rows_are_the_rows_each_track_plays() {
    let (spec, dir) = ds_003(&["extrema"]);
    let compiled = compile_in(&spec, &dir);
    let loaded = sonarithm_data::read_bytes(
        DS_003.as_bytes(),
        sonarithm_data::Format::Csv,
        "ds-003-regions.csv",
        &sonarithm_data::ReadOptions::default(),
    )
    .unwrap();
    let table = loaded.bundle.primary().unwrap();
    for track in spec["tracks"].as_array().unwrap() {
        let id = track["id"].as_str().unwrap();
        let filter = track["filter"]["expr"].as_str();
        let expected = sonarithm_data::play_rows(table, filter, Some("month")).unwrap();
        let played: Vec<u64> = compiled
            .score
            .tracks
            .iter()
            .find(|t| t.id.as_str() == id)
            .unwrap()
            .events
            .iter()
            .filter_map(|event| match event {
                sonarithm_core::score::ScoreEvent::Note(note) => note.src.map(|src| src.row),
                _ => None,
            })
            .collect();
        assert_eq!(played, expected, "track {id}");
    }
}

/// AGENTS rule 7 (#506 review): an annotation's confidence is the lowest among the rows it
/// covers, so an insight on an estimate is not reported as certain.
#[test]
fn a11y_23_an_insight_on_an_estimate_carries_its_confidence() {
    let rows: Vec<Value> = (0..20_u32)
        .map(|i| {
            let estimated = i == 12;
            let y = if estimated {
                95.0
            } else {
                40.0 + f64::from(i % 4) * 0.5
            };
            json!({
                "t": i,
                "y": y,
                "__snr_prov": if estimated { "cv_estimate" } else { "exact" },
                "__snr_conf": if estimated { json!(0.75) } else { Value::Null },
            })
        })
        .collect();
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "inline", "values": rows } },
        "substrate": { "time": { "field": "t" } },
        "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "y" } } }],
        "analysis": { "detect": ["extrema", "outliers"] }
    });
    let compiled = compile(&spec);
    let at = |label: &str| {
        compiled
            .annotations
            .iter()
            .find(|a| a.label == label)
            .unwrap_or_else(|| panic!("no {label}"))
    };
    let estimate = Confidence::new(0.75).unwrap();
    for label in ["maximum", "outlier"] {
        assert_eq!(at(label).range.start, 12, "{label}");
        assert_eq!(at(label).confidence, estimate, "{label}");
    }
    assert_eq!(at("minimum").confidence, Confidence::CERTAIN);
}

/// SDR-43's rule for an insight's confidence (#506 review): an estimate without a stated
/// confidence counts as zero, so the insight is never reported as certain.
#[test]
fn sdr_43_an_insight_on_an_unstated_estimate_is_not_certain() {
    let rows: Vec<Value> = (0..20_u32)
        .map(|i| {
            let estimated = i == 12;
            let y = if estimated {
                95.0
            } else {
                40.0 + f64::from(i % 4) * 0.5
            };
            json!({
                "t": i,
                "y": y,
                // No `__snr_conf` column at all: the estimate states no confidence.
                "__snr_prov": if estimated { "cv_estimate" } else { "exact" },
            })
        })
        .collect();
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "inline", "values": rows } },
        "substrate": { "time": { "field": "t" } },
        "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "y" } } }],
        "analysis": { "detect": ["extrema"] }
    });
    let compiled = compile(&spec);
    let maximum = compiled
        .annotations
        .iter()
        .find(|a| a.label == "maximum")
        .unwrap();
    assert_eq!(maximum.range.start, 12);
    assert_eq!(maximum.confidence, Confidence::ZERO);
}
