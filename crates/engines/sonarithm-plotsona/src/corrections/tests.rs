// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "tests"
)]

use serde_json::{Value, json};
use sonarithm_core::{Confidence, Provenance, ProvenanceClass, Sha256Digest};
use sonarithm_data::RowProvenance;
use sonarithm_spec::correction_file::{self, ChartRef, CorrectionFile};

use super::*;
use crate::corpus::folder;
use crate::model::{ChartData, Point, Series, XValue};
use crate::{ExtractOptions, extract};

/// Two estimated series of three category points each, with per-point confidences.
fn chart() -> ChartData {
    let series = |name: &str, ys: [f64; 3]| {
        let mut s = Series::exact(
            name,
            ["Jan", "Feb", "Mar"]
                .iter()
                .zip(ys)
                .map(|(x, y)| Point::category(*x, Some(y)))
                .collect(),
        );
        s.provenance = Provenance {
            class: ProvenanceClass::CvEstimate,
            confidence: Confidence::new(0.8),
        };
        s.confidences = [0.9, 0.4, 0.8]
            .iter()
            .map(|c| Confidence::new(*c).unwrap())
            .collect();
        s
    };
    ChartData {
        series: vec![
            series("rain", [10.0, 20.0, 30.0]),
            series("snow", [1.0, 2.0, 3.0]),
        ],
        ..ChartData::default()
    }
}

fn inline(edits: &Value) -> CorrectionSet {
    CorrectionSet::Inline(serde_json::from_value(json!({ "edits": edits })).unwrap())
}

fn run(set: &CorrectionSet) -> (ChartData, Vec<CorrectionRecord>, Vec<String>) {
    let mut chart = chart();
    let mut diagnostics = Vec::new();
    let history = apply(&mut chart, set, b"", "test", &mut diagnostics);
    (
        chart,
        history,
        diagnostics
            .iter()
            .map(|d| d.code.as_str().to_owned())
            .collect(),
    )
}

fn ys(chart: &ChartData, series: usize) -> Vec<f64> {
    chart.series[series]
        .points
        .iter()
        .filter_map(|p| p.y)
        .collect()
}

/// RFC-0004 §6: each op does what it says, and every changed point becomes `user_edited`.
#[test]
fn moves_deletes_adds_and_retypes_apply_in_order() {
    let (chart, history, codes) = run(&inline(&json!([
        { "op": "move", "series": "rain", "point": 1, "was": { "x": "Feb", "y": 20.0 }, "to": { "y": 25.0 } },
        { "op": "delete", "series": "snow", "point": 0, "was": { "x": "Jan", "y": 1.0 } },
        { "op": "add", "series": "snow", "to": { "x": "Apr", "y": 4.0 } },
        { "op": "retype", "series": "rain", "point": 2, "was": { "y": 30.0 }, "to": { "series": "hail" } },
    ])));
    assert!(codes.is_empty(), "{codes:?}");
    assert!(history.iter().all(|r| r.applied));
    assert_eq!(ys(&chart, 0), [10.0, 25.0]);
    assert_eq!(chart.series[0].user_edited, [1]);
    assert_eq!(ys(&chart, 1), [2.0, 3.0, 4.0]);
    assert_eq!(chart.series[1].user_edited, [2]);
    assert_eq!(chart.series[2].name, "hail");
    assert_eq!(ys(&chart, 2), [30.0]);
    assert_eq!(chart.series[2].user_edited, [0]);
    // The history keeps each point's value and provenance before the edit.
    assert_eq!(history[0].was, Some(Point::category("Feb", Some(20.0))));
    assert_eq!(
        history[0].before,
        Some(Provenance {
            class: ProvenanceClass::CvEstimate,
            confidence: Confidence::new(0.4)
        })
    );
    assert_eq!(history[0].point, "rain#1");
}

/// RFC-0004 §6: a guard that no longer matches, or a point that is gone, stops the edit
/// and is reported; nothing else changes.
#[test]
fn an_edit_is_never_applied_to_another_point() {
    let (chart, history, codes) = run(&inline(&json!([
        { "op": "move", "series": "rain", "point": 1, "was": { "y": 99.0 }, "to": { "y": 25.0 } },
        { "op": "move", "series": "rain", "point": 7, "was": { "y": 20.0 }, "to": { "y": 25.0 } },
        { "op": "delete", "series": "rain", "point": 0, "was": { "x": "Jan", "y": 10.0 } },
        { "op": "delete", "series": "rain", "point": 0, "was": { "x": "Jan", "y": 10.0 } },
    ])));
    assert_eq!(codes, ["SNR-ENG-386", "SNR-ENG-386", "SNR-ENG-386"]);
    assert_eq!(
        history.iter().map(|r| r.applied).collect::<Vec<_>>(),
        [false, false, true, false]
    );
    assert_eq!(ys(&chart, 0), [20.0, 30.0]);
    assert_eq!(chart.series[0].user_edited.len(), 0);
}

/// Guards match within 0.1 % of the axis, so a value typed from a rounded announcement
/// still finds its point (A11Y-29).
#[test]
fn guards_match_rounded_values() {
    let (_, history, codes) = run(&inline(&json!([
        { "op": "move", "series": "rain", "point": 2, "was": { "y": 30.01 }, "to": { "y": 31.0 } },
    ])));
    assert_eq!(codes.len(), 0);
    assert!(history[0].applied);
    let (_, _, codes) = run(&inline(&json!([
        { "op": "move", "series": "rain", "point": 2, "was": { "y": 30.5 }, "to": { "y": 31.0 } },
    ])));
    assert_eq!(codes, ["SNR-ENG-386"]);
}

fn file(edits: Value, sha: &str, extractor: &str) -> CorrectionSet {
    let mut file = CorrectionFile::new(ChartRef::new(sha, extractor, "0.0.0"));
    file.edits = serde_json::from_value(edits).unwrap();
    CorrectionSet::File(Box::new(file))
}

/// RFC-0005 §4: a later edit can refer to a point an `add` created.
#[test]
fn file_edits_can_refer_to_added_points() {
    let sha = Sha256Digest::of(b"").to_string();
    let set = file(
        json!([
            { "op": "add", "id": "e1", "series": "rain", "to": { "x": "Apr", "y": 40.0 } },
            { "op": "move", "id": "e2", "series": "rain", "point": "+e1", "was": { "y": 40.0 }, "to": { "y": 41.0 },
              "before": { "class": "user_edited" } },
        ]),
        &sha,
        "test",
    );
    let (chart, history, codes) = run(&set);
    assert!(codes.is_empty(), "{codes:?}");
    assert_eq!(ys(&chart, 0), [10.0, 20.0, 30.0, 41.0]);
    assert_eq!(history[1].point, "+e1");
}

/// RFC-0005 §3: a file made on another chart, or by another extractor, applies nothing.
#[test]
fn a_file_for_another_chart_applies_nothing() {
    let edits = json!([
        { "op": "delete", "id": "e1", "series": "rain", "point": 0, "was": { "y": 10.0 }, "before": { "class": "cv_estimate" } }
    ]);
    let other = "0".repeat(64);
    let (chart, history, codes) = run(&file(edits.clone(), &other, "test"));
    assert_eq!(codes, ["SNR-ENG-387"]);
    assert_eq!(history.len(), 0);
    assert_eq!(chart, super::tests::chart());
    let sha = Sha256Digest::of(b"").to_string();
    let (_, _, codes) = run(&file(edits, &sha, "another-extractor"));
    assert_eq!(codes, ["SNR-ENG-387"]);
}

/// Through `extract`: corrected rows are `user_edited` with no confidence in the table,
/// the report counts them as exact and keeps the history, and a correction file written
/// and read back applies the same way.
#[test]
fn corrected_rows_are_user_edited_in_the_table() {
    let bytes = std::fs::read(folder("chartjson").join("vl-line.json")).unwrap();
    let edits = json!([
        { "op": "move", "series": "north", "point": 0, "was": { "x": "Jan", "y": 99.2 }, "to": { "y": 100.0 } },
        { "op": "delete", "series": "south", "point": 1, "was": { "x": "Feb" } },
    ]);
    let plain = extract(&bytes, "vl-line.json", &ExtractOptions::new()).unwrap();
    let options = ExtractOptions::new().with_corrections(inline(&edits));
    let read = extract(&bytes, "vl-line.json", &options).unwrap();
    assert!(
        read.report.diagnostics.is_empty(),
        "{:?}",
        read.report.diagnostics
    );
    assert_eq!(read.report.points, plain.report.points - 1);
    assert_eq!(read.report.corrections.len(), 2);
    assert_eq!(read.report.provenance.get("user_edited"), Some(&1));
    let rows = RowProvenance::read(read.bundle.primary().unwrap()).unwrap();
    assert_eq!(
        rows.get(0),
        Provenance {
            class: ProvenanceClass::UserEdited,
            confidence: None
        }
    );
    assert_eq!(rows.get(1).class, ProvenanceClass::Exact);
    let counts = read.report.counts();
    assert_eq!(counts.exact, read.report.points);
    // The same edits in a correction file bound to this chart, after a write and a read.
    let mut file = CorrectionFile::new(ChartRef::new(
        Sha256Digest::of(&bytes).to_string(),
        read.report.extractor.clone(),
        "0.0.0",
    ));
    file.edits = serde_json::from_value(json!([
        { "op": "move", "id": "e1", "series": "north", "point": 0, "was": { "x": "Jan", "y": 99.2 }, "to": { "y": 100.0 },
          "before": { "class": "exact" } },
        { "op": "delete", "id": "e2", "series": "south", "point": 1, "was": { "x": "Feb" }, "before": { "class": "exact" } },
    ]))
    .unwrap();
    let (file, _) = correction_file::read(correction_file::write(&file).as_bytes()).unwrap();
    let from_file = extract(
        &bytes,
        "vl-line.json",
        &ExtractOptions::new().with_corrections(CorrectionSet::File(Box::new(file))),
    )
    .unwrap();
    assert_eq!(from_file.chart, read.chart);
    assert_eq!(from_file.report.diagnostics.len(), 0);
    let _ = XValue::Number(0.0);
}
