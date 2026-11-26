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
        Some(&AnnotationParam::Text("robust-z".to_owned()))
    );
    let tree = compiled.nav.as_ref().expect("AUDAXIS builds the tree");
    let target = tree
        .iter()
        .find_map(|(_, node)| match &node.kind {
            NavKind::Insight(insight) if node.label.as_deref() == Some("outlier") => {
                insight.target
            }
            _ => None,
        })
        .expect("the outlier insight has a target");
    assert_eq!(series_of(tree, target).as_deref(), Some("humidity"));
}

/// Every annotation records its field and parameters (C05-T01's snapshot), the same way on
/// every compile (ADR-0013).
#[test]
fn c05_t01_annotations_carry_their_field_and_parameters() {
    let spec = two_series(&["extrema", "outliers"]);
    let compiled = compile(&spec);
    let json = serde_json::to_string_pretty(&compiled.annotations).unwrap();
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
    assert!(with.annotations.is_empty());
    let platform = Platform::new();
    let numisona = Numisona::new();
    let audaxis = Audaxis::new();
    let without: Compiler<DataBundle> =
        Compiler::new(&[&platform, &numisona, &audaxis]).unwrap();
    let without = without
        .compile(&spec.to_string(), &CancelToken::new())
        .unwrap();
    assert_eq!(
        serde_json::to_string(&with.score).unwrap(),
        serde_json::to_string(&without.score).unwrap()
    );
}
