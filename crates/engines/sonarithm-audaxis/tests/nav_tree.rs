// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! B03-T01: the navigation tree (08 §2) for each NUMISONA template, its invariants (every
//! node reachable, every point a `RowRef`), segments and onsets (A11Y-01, A11Y-04).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use proptest::prelude::*;
use serde_json::{Value, json};
use sonarithm_audaxis::nav::tree::{MAX_SEGMENTS, MIN_SEGMENT_POINTS, TREND_SEGMENTS, build};
use sonarithm_core::engine::{Annotation, Budget, CancelToken, CompileContext};
use sonarithm_core::ids::{AnalyzerId, RowRange};
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree, XType};
use sonarithm_core::score::{ScoreEvent, TrackRole};
use sonarithm_data::DataBundle;
use sonarithm_numisona::examples;
use sonarithm_numisona::templates::{TEMPLATES, template};
use support::{compile, outline, tree};

fn source(name: &str) -> Value {
    json!({ "kind": "file", "path": format!("{name}.json") })
}

fn dataset(name: &str) -> DataBundle {
    examples::all()
        .unwrap()
        .into_iter()
        .find(|(n, _)| *n == name)
        .unwrap()
        .1
}

/// The six templates, each on the dataset its reviewed spec uses (A06-T07).
const TEMPLATE_CASES: [(&str, &str); 6] = [
    ("line-overview", "temperature"),
    ("bar-discrete", "fruit"),
    ("scatter-xy", "scatter"),
    ("histogram", "waiting_times"),
    ("box-five-number", "reaction_times"),
    ("heatmap-rowscan", "grid"),
];

/// The data a compile reads: the spec's inline values (templates that summarize their data
/// write it inline), else the bundle.
fn compiled_data(spec: &Value, bundle: &DataBundle) -> DataBundle {
    match spec.pointer("/data/source/values") {
        Some(values) => {
            let bytes = serde_json::to_vec(values).unwrap();
            sonarithm_data::read_bytes(
                &bytes,
                sonarithm_data::Format::Json,
                "inline.json",
                &sonarithm_data::ReadOptions::new(),
            )
            .unwrap()
            .bundle
        }
        None => bundle.clone(),
    }
}

fn template_tree(id: &str, name: &str) -> (NavTree, DataBundle) {
    let bundle = dataset(name);
    let template = template(id).unwrap();
    let bindings = template.bind(bundle.primary().unwrap()).unwrap();
    let spec = template.apply(&bundle, &bindings, &source(name)).unwrap();
    let compiled = compile(&spec.to_string(), Some(bundle.clone()));
    (tree(&compiled).clone(), compiled_data(&spec, &bundle))
}

/// Every point is a row of the data, and the tree's own invariants hold.
fn assert_sound(tree: &NavTree, bundle: &DataBundle) {
    tree.check().unwrap();
    for (_, node) in tree.iter() {
        if let NavKind::Point(point) = &node.kind {
            let table = bundle.table(point.row.table).unwrap();
            assert!(point.row.row < table.num_rows(), "{point:?}");
        }
    }
}

/// B03-T01 acceptance: a navigation-tree snapshot for each of the six NUMISONA templates.
#[test]
fn a11y_04_nav_tree_snapshots_for_the_six_templates() {
    for (id, name) in TEMPLATE_CASES {
        let (tree, bundle) = template_tree(id, name);
        assert_sound(&tree, &bundle);
        insta::assert_snapshot!(id, outline(&tree));
    }
}

/// A11Y-01, A11Y-04: every template on every compatible example dataset gives a sound tree
/// whose root is the dataset, with series, axes, insights and filters under it.
#[test]
fn a11y_04_every_compatible_template_gives_a_sound_tree() {
    let datasets = examples::all().unwrap();
    for template in &TEMPLATES {
        for (name, bundle) in &datasets {
            let Ok(bindings) = template.bind(bundle.primary().unwrap()) else {
                continue;
            };
            let spec = template.apply(bundle, &bindings, &source(name)).unwrap();
            let compiled = compile(&spec.to_string(), Some(bundle.clone()));
            let tree = tree(&compiled);
            assert_sound(tree, &compiled_data(&spec, bundle));
            let data_tracks = compiled
                .score
                .tracks
                .iter()
                .filter(|t| t.role == TrackRole::Data)
                .count();
            assert_eq!(
                tree.series().count(),
                data_tracks,
                "{} × {name}",
                template.id
            );
            for landmark in ["axes", "insights", "filters"] {
                assert!(tree.landmark(landmark).is_some(), "{landmark}");
            }
        }
    }
}

/// Every played note's row is a point of its series with the note's onset; points follow
/// x order.
#[test]
fn points_carry_their_onsets_and_follow_x_order() {
    let (tree, _) = template_tree("line-overview", "temperature");
    let series = tree.series().next().unwrap();
    let points = tree.points(series);
    assert!(points.len() > 1);
    let mut last = f64::NEG_INFINITY;
    for id in &points {
        let node = tree.node(*id).unwrap();
        let NavKind::Point(point) = &node.kind else {
            panic!("points() returns points");
        };
        assert!(point.t.is_some(), "every line-overview point is played");
        let x = node.span.unwrap().start;
        assert!(x >= last);
        last = x;
    }
}

/// Glide tracks give each point its breakpoint onset.
#[test]
fn glide_points_get_breakpoint_onsets() {
    let spec = json!({
        "specVersion": "0.1",
        "data": {"source": {"kind": "inline", "values": [
            {"x": 3, "y": 1.0}, {"x": 1, "y": 2.0}, {"x": 2, "y": 4.0}, {"x": 4, "y": 3.0}
        ]}, "fields": {"x": {"role": "x"}}},
        "tracks": [{"id": "y", "mark": "glide", "channels": {"pitch": {"field": "y"}}}]
    });
    let compiled = compile(&spec.to_string(), None);
    let tree = tree(&compiled);
    let series = tree.series().next().unwrap();
    let onsets: Vec<f64> = tree
        .points(series)
        .iter()
        .filter_map(|&id| match &tree.node(id).unwrap().kind {
            NavKind::Point(p) => p.t.map(|t| t.0),
            _ => None,
        })
        .collect();
    let ScoreEvent::Glide(glide) = &compiled.score.tracks[0].events[0] else {
        panic!("a glide track has glides");
    };
    let expected: Vec<f64> = glide.curve[..glide.curve.len() - 1]
        .iter()
        .map(|b| b.t.0)
        .collect();
    assert_eq!(onsets, expected);
}

/// Rows with a missing value stay in the tree as points without a value; rows the track
/// filter drops do not.
#[test]
fn missing_values_stay_and_filtered_rows_go() {
    let spec = json!({
        "specVersion": "0.1",
        "data": {"source": {"kind": "inline", "values": [
            {"x": 1, "y": 1.0}, {"x": 2, "y": null}, {"x": 3, "y": 3.0}, {"x": 4, "y": 9.0}
        ]}, "fields": {"x": {"role": "x"}}},
        "tracks": [{"id": "y", "mark": "tone", "channels": {"pitch": {"field": "y"}},
                    "filter": {"expr": "x < 4"}}]
    });
    let compiled = compile(&spec.to_string(), None);
    let tree = tree(&compiled);
    let series = tree.series().next().unwrap();
    let values: Vec<Option<f64>> = tree
        .points(series)
        .iter()
        .map(|&id| match &tree.node(id).unwrap().kind {
            NavKind::Point(p) => p.value,
            _ => panic!("points() returns points"),
        })
        .collect();
    assert_eq!(values, [Some(1.0), None, Some(3.0)]);
}

/// Even segments: at least `MIN_SEGMENT_POINTS` points, at most `MAX_SEGMENTS` (OD-18).
#[test]
fn even_segments_respect_their_bounds() {
    for n in [1_usize, 7, 8, 16, 50, 96, 97, 365, 1000] {
        let values: Vec<Value> = (0..n).map(|i| json!({"x": i, "y": i % 7})).collect();
        let spec = json!({
            "specVersion": "0.1",
            "data": {"source": {"kind": "inline", "values": values},
                     "fields": {"x": {"role": "x"}}},
            "tracks": [{"id": "y", "mark": "tone", "channels": {"pitch": {"field": "y"}}}]
        });
        let compiled = compile(&spec.to_string(), None);
        let tree = tree(&compiled);
        let series = tree.series().next().unwrap();
        let sizes: Vec<usize> = tree
            .children(series)
            .iter()
            .map(|&s| tree.children(s).len())
            .collect();
        assert!(sizes.len() <= MAX_SEGMENTS, "{n}: {sizes:?}");
        assert_eq!(sizes.iter().sum::<usize>(), n);
        if n >= MIN_SEGMENT_POINTS {
            assert!(
                sizes.iter().all(|&s| s >= MIN_SEGMENT_POINTS),
                "{n}: {sizes:?}"
            );
        }
        let (lo, hi) = (sizes.iter().min().unwrap(), sizes.iter().max().unwrap());
        assert!(
            hi - lo <= 1,
            "even segments differ by at most one point: {sizes:?}"
        );
    }
}

/// Trend-segment annotations that partition a series become its segments.
#[test]
fn trend_segments_become_semantic_segments() {
    let values: Vec<Value> = (0..20).map(|i| json!({"x": i, "y": i})).collect();
    let spec = json!({
        "specVersion": "0.1",
        "data": {"source": {"kind": "inline", "values": values}, "fields": {"x": {"role": "x"}}},
        "tracks": [{"id": "y", "mark": "tone", "channels": {"pitch": {"field": "y"}}}]
    });
    let compiled = compile(&spec.to_string(), None);
    let prepared = sonarithm_spec::prepare(&spec.to_string(), None).unwrap();
    let bundle = {
        let bytes = serde_json::to_vec(&spec["data"]["source"]["values"]).unwrap();
        sonarithm_data::read_bytes(
            &bytes,
            sonarithm_data::Format::Json,
            "inline.json",
            &sonarithm_data::ReadOptions::new(),
        )
        .unwrap()
        .bundle
    };
    let all = bundle.primary().unwrap().rows();
    let segment = |start, end, label| {
        Annotation::new(
            AnalyzerId::from_static(TREND_SEGMENTS),
            label,
            RowRange { start, end, ..all },
        )
    };
    let annotations = [segment(0, 5, "rising"), segment(5, 20, "flat")];
    let mut cx = CompileContext::new(0, Budget::default(), CancelToken::new());
    let tree = build(
        &prepared.resolved,
        &compiled.score,
        &bundle,
        &annotations,
        &mut cx,
    )
    .unwrap();
    tree.check().unwrap();
    let series = tree.series().next().unwrap();
    let segments: Vec<(bool, usize)> = tree
        .children(series)
        .iter()
        .map(|&s| match &tree.node(s).unwrap().kind {
            NavKind::Segment(seg) => (seg.semantic, tree.children(s).len()),
            _ => panic!("series children are segments"),
        })
        .collect();
    assert_eq!(segments, [(true, 5), (true, 15)]);
    // The annotations are insights too, and point at their first point.
    let insights = tree.landmark("insights").unwrap();
    let group = tree.children(insights)[0];
    let first = tree.children(group)[0];
    let NavKind::Insight(insight) = &tree.node(first).unwrap().kind else {
        panic!("insight groups hold insights");
    };
    assert_eq!(insight.target, Some(tree.points(series)[0]));
}

/// The x axis says what its values are.
#[test]
fn the_x_axis_names_its_value_type() {
    let x_type = |id, name| {
        let (tree, _) = template_tree(id, name);
        let axes = tree.landmark("axes").unwrap();
        match &tree.node(tree.children(axes)[0]).unwrap().kind {
            NavKind::Axis(found) => found.values,
            _ => panic!("the first axis is x"),
        }
    };
    assert_eq!(x_type("bar-discrete", "fruit"), XType::Category);
    assert_eq!(x_type("line-overview", "temperature"), XType::Number);
}

/// Rows of (x, y) with at least one value, so the pitch field is never all missing.
fn arbitrary_rows() -> impl Strategy<Value = Vec<(i32, Option<f64>)>> {
    (
        -1.0e3_f64..1.0e3,
        prop::collection::vec((-50_i32..50, prop::option::of(-1.0e3_f64..1.0e3)), 1..80),
    )
        .prop_map(|(first, mut rows)| {
            rows[0].1 = Some(first);
            rows
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// A11Y-04: for any data, every node is reachable from the root, every point is a row
    /// of the table, points follow x order, and the segments partition the points.
    #[test]
    fn a11y_04_every_node_is_reachable_and_every_point_is_a_row(
        rows in arbitrary_rows(),
        glide in any::<bool>(),
    ) {
        let values: Vec<Value> = rows.iter().map(|(x, y)| json!({"x": x, "y": y})).collect();
        let spec = json!({
            "specVersion": "0.1",
            "data": {"source": {"kind": "inline", "values": values},
                     "fields": {"x": {"role": "x"}}},
            "tracks": [{"id": "y", "mark": if glide { "glide" } else { "tone" },
                        "channels": {"pitch": {"field": "y"}}}]
        });
        let compiled = compile(&spec.to_string(), None);
        let tree = tree(&compiled);
        prop_assert!(tree.check().is_ok());
        let series = tree.series().next().unwrap();
        let points = tree.points(series);
        prop_assert_eq!(points.len(), rows.len());
        let mut last = f64::NEG_INFINITY;
        let mut seen = std::collections::BTreeSet::new();
        for (k, id) in points.iter().enumerate() {
            let node = tree.node(*id).unwrap();
            let NavKind::Point(point) = &node.kind else {
                return Err(TestCaseError::fail("points() returned a non-point"));
            };
            prop_assert_eq!(point.index, k as u64);
            prop_assert!(usize::try_from(point.row.row).unwrap() < rows.len());
            prop_assert!(seen.insert(point.row.row), "each row is one point");
            let x = node.span.unwrap().start;
            prop_assert!(x >= last);
            last = x;
        }
        let in_segments: usize = tree.children(series).iter().map(|&s| tree.children(s).len()).sum();
        prop_assert_eq!(in_segments, points.len());
        for (id, node) in tree.iter() {
            if id != NavNodeId::ROOT {
                prop_assert!(tree.depth(id) <= 3);
            }
            prop_assert!(node.children.iter().all(|c| tree.parent(*c) == Some(id)));
        }
    }
}
