// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! B03-T02: the cursor, its verbs at every level (keyboard-and-navigation §4.3) and
//! "where am I" (§6); A11Y-04, A11Y-07, A11Y-08, A11Y-27.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use std::fmt::Write as _;

use proptest::prelude::*;
use serde_json::{Value, json};
use sonarithm_audaxis::earcons::EarconId;
use sonarithm_audaxis::nav::{Cursor, Edge, Invalid, Landmark, Outcome, Verb, XTarget};
use sonarithm_core::engine::Annotation;
use sonarithm_core::ids::{AnalyzerId, RowRange};
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree};
use support::{inline_bundle, tree_with};

/// Two series (temperature and humidity) over 20 days, so each has two segments of ten
/// points; humidity has no value on day 5.
fn spec() -> Value {
    let values: Vec<Value> = (0..20)
        .map(|day| {
            let humidity = if day == 5 {
                Value::Null
            } else {
                json!(40 + (day * 7) % 30)
            };
            json!({"day": day, "temp": 10 + (day * 3) % 11, "humid": humidity})
        })
        .collect();
    json!({
        "specVersion": "0.1",
        "name": "Weather",
        "data": {"source": {"kind": "inline", "values": values},
                 "fields": {"day": {"role": "x"}}},
        "tracks": [
            {"id": "temp", "mark": "tone", "channels": {"pitch": {"field": "temp"}}},
            {"id": "humid", "mark": "tone", "channels": {"pitch": {"field": "humid"}}}
        ]
    })
}

/// The fixture tree with two extrema insights on temperature (days 3 and 7).
fn fixture() -> NavTree {
    let spec = spec();
    let rows = inline_bundle(&spec).primary().unwrap().rows();
    let at = |row, label| {
        Annotation::new(
            AnalyzerId::from_static("extrema"),
            label,
            RowRange {
                start: row,
                end: row + 1,
                ..rows
            },
        )
    };
    tree_with(&spec, &[at(7, "maximum"), at(3, "minimum")])
}

/// A short name for a node, for the verb matrix.
fn name(tree: &NavTree, id: NavNodeId) -> String {
    let node = tree.node(id).unwrap();
    let label = node.label.clone().unwrap_or_default();
    match &node.kind {
        NavKind::Dataset(_) => "dataset".to_owned(),
        NavKind::Point(p) => format!(
            "point {} of {label}",
            p.index,
            label = series_label(tree, id)
        ),
        NavKind::Segment(_) => {
            let parent = tree.parent(id).unwrap();
            let index = tree.children(parent).iter().position(|&c| c == id).unwrap();
            format!("segment {} of {}", index + 1, series_label(tree, id))
        }
        other => format!("{} {label}", other.name()).trim_end().to_owned(),
    }
}

fn series_label(tree: &NavTree, id: NavNodeId) -> String {
    let mut current = Some(id);
    while let Some(n) = current {
        let node = tree.node(n).unwrap();
        if matches!(node.kind, NavKind::Series(_)) {
            return node.label.clone().unwrap_or_default();
        }
        current = tree.parent(n);
    }
    String::new()
}

fn describe(tree: &NavTree, outcome: &Outcome) -> String {
    match outcome {
        Outcome::Moved(m) => {
            let mut text = format!("→ {}", name(tree, m.to));
            if m.nearest {
                text.push_str(" (nearest)");
            }
            if let Some(segment) = m.new_segment {
                let _ = write!(text, " [enters {}]", name(tree, segment));
            }
            if m.landmark {
                text.push_str(" [landmark]");
            }
            text
        }
        Outcome::Boundary { edge, .. } => format!("boundary {edge:?}"),
        Outcome::Invalid { reason, .. } => format!("invalid {reason:?}"),
        Outcome::Describe { .. } => "describe".to_owned(),
        Outcome::Toggled { included, .. } => format!("toggle (included: {included})"),
        other => format!("{other:?}"),
    }
}

fn verbs() -> Vec<(&'static str, Verb)> {
    vec![
        ("Right", Verb::Next),
        ("Left", Verb::Previous),
        ("Jump+Right", Verb::JumpForward),
        ("Jump+Left", Verb::JumpBack),
        ("Home", Verb::First),
        ("End", Verb::Last),
        ("Down", Verb::Into),
        ("Up", Verb::Out),
        ("Enter", Verb::Activate),
        ("PageDown", Verb::NextSeries),
        ("PageUp", Verb::PreviousSeries),
        ("Jump+Up", Verb::Maximum),
        ("Jump+Down", Verb::Minimum),
        ("N", Verb::NextInsight),
        ("Shift+N", Verb::PreviousInsight),
    ]
}

/// One representative node per level and kind (keyboard §4.3 rows).
fn representatives(tree: &NavTree) -> Vec<NavNodeId> {
    let root = tree.root();
    let series: Vec<NavNodeId> = tree.series().collect();
    let temp = series[0];
    let segment = tree.children(temp)[1];
    let points = tree.points(temp);
    let axes = tree.landmark("axes").unwrap();
    let insights = tree.landmark("insights").unwrap();
    let group = tree.children(insights)[0];
    let filters = tree.landmark("filters").unwrap();
    vec![
        root,
        temp,
        axes,
        insights,
        filters,
        segment,
        tree.children(axes)[0],
        group,
        tree.children(filters)[1],
        points[12],
        tree.children(group)[0],
    ]
}

/// A11Y-04, keyboard §4.3: a unit test per verb and level. The matrix is reviewed as a
/// snapshot: every row is a starting node, every entry what one verb does from there.
#[test]
fn a11y_04_every_verb_at_every_level() {
    let tree = fixture();
    let mut table = String::new();
    for start in representatives(&tree) {
        let _ = writeln!(table, "{}:", name(&tree, start));
        for (key, verb) in verbs() {
            let mut cursor = Cursor::at(&tree, start).unwrap();
            let outcome = cursor.apply(&tree, &verb);
            let _ = writeln!(table, "  {key:<10} {}", describe(&tree, &outcome));
        }
    }
    insta::assert_snapshot!(table);
}

/// A11Y-08: a boundary move leaves the cursor in place and plays the boundary earcon.
#[test]
fn a11y_08_boundary_moves_keep_the_cursor_and_bonk() {
    let tree = fixture();
    let temp = tree.series().next().unwrap();
    let last = *tree.points(temp).last().unwrap();
    let mut cursor = Cursor::at(&tree, last).unwrap();
    let outcome = cursor.apply(&tree, &Verb::Next);
    assert_eq!(
        outcome,
        Outcome::Boundary {
            at: last,
            edge: Edge::Last
        }
    );
    assert_eq!(outcome.earcon(), Some(EarconId::Boundary));
    assert_eq!(cursor.node(), last);

    let mut top = Cursor::new();
    assert_eq!(
        top.apply(&tree, &Verb::Out),
        Outcome::Boundary {
            at: tree.root(),
            edge: Edge::Top
        }
    );
    let mut cursor = Cursor::at(&tree, last).unwrap();
    assert!(matches!(
        cursor.apply(&tree, &Verb::Into),
        Outcome::Boundary {
            edge: Edge::Bottom,
            ..
        }
    ));
}

/// A11Y-08: a verb that does not apply plays the invalid earcon with a reason.
#[test]
fn a11y_08_invalid_verbs_say_why() {
    let one_series = json!({
        "specVersion": "0.1",
        "data": {"source": {"kind": "inline", "values": [{"x": 1, "y": 2}, {"x": 2, "y": 3}]},
                 "fields": {"x": {"role": "x"}}},
        "tracks": [{"id": "y", "mark": "tone", "channels": {"pitch": {"field": "y"}}}]
    });
    let tree = tree_with(&one_series, &[]);
    let point = tree.points(tree.series().next().unwrap())[0];
    let mut cursor = Cursor::at(&tree, point).unwrap();
    let outcome = cursor.apply(&tree, &Verb::NextSeries);
    assert_eq!(
        outcome,
        Outcome::Invalid {
            at: point,
            reason: Invalid::OneSeries
        }
    );
    assert_eq!(outcome.earcon(), Some(EarconId::Invalid));
    assert_eq!(cursor.node(), point);
}

/// Keyboard §4.3: entering a child and going back up keep the x position.
#[test]
fn entering_and_leaving_keep_the_x_position() {
    let tree = fixture();
    let temp = tree.series().next().unwrap();
    let point = tree.points(temp)[14];
    let mut cursor = Cursor::at(&tree, point).unwrap();
    cursor.apply(&tree, &Verb::Out);
    cursor.apply(&tree, &Verb::Out);
    assert_eq!(cursor.node(), temp);
    cursor.apply(&tree, &Verb::Into);
    cursor.apply(&tree, &Verb::Into);
    assert_eq!(cursor.node(), point, "back on the same point");
}

/// Keyboard §4.3: at point level, Right crosses into the next segment and names it.
#[test]
fn stepping_past_a_segment_continues_into_the_next() {
    let tree = fixture();
    let temp = tree.series().next().unwrap();
    let points = tree.points(temp);
    let mut cursor = Cursor::at(&tree, points[9]).unwrap();
    let Outcome::Moved(step) = cursor.apply(&tree, &Verb::Next) else {
        panic!("Right moves");
    };
    assert_eq!(step.to, points[10]);
    assert_eq!(step.new_segment, Some(tree.children(temp)[1]));
}

/// Keyboard §4.2: lateral moves keep x, and say "nearest" when the other series has no
/// point there.
#[test]
fn lateral_moves_keep_x_or_land_on_the_nearest() {
    let tree = fixture();
    let series: Vec<NavNodeId> = tree.series().collect();
    let temp_points = tree.points(series[0]);
    let humid_points = tree.points(series[1]);
    let mut cursor = Cursor::at(&tree, temp_points[12]).unwrap();
    let Outcome::Moved(lateral) = cursor.apply(&tree, &Verb::NextSeries) else {
        panic!("PageDown moves");
    };
    assert_eq!(lateral.to, humid_points[12]);
    assert!(!lateral.nearest);
    assert_eq!(
        cursor.apply(&tree, &Verb::NextSeries),
        Outcome::Boundary {
            at: humid_points[12],
            edge: Edge::Last
        }
    );
}

/// Targeted verbs: extremes, insights, landmarks, go to x and find value.
#[test]
fn targeted_verbs_land_where_they_say() {
    let tree = fixture();
    let temp = tree.series().next().unwrap();
    let points = tree.points(temp);
    let value = |id: NavNodeId| match &tree.node(id).unwrap().kind {
        NavKind::Point(p) => p.value.unwrap(),
        _ => panic!("a point"),
    };
    let mut cursor = Cursor::at(&tree, points[0]).unwrap();
    cursor.apply(&tree, &Verb::Maximum);
    assert_eq!(value(cursor.node()), 20.0);
    cursor.apply(&tree, &Verb::Minimum);
    assert_eq!(value(cursor.node()), 10.0);

    let mut cursor = Cursor::at(&tree, points[0]).unwrap();
    let Outcome::Moved(first) = cursor.apply(&tree, &Verb::NextInsight) else {
        panic!("N moves");
    };
    assert_eq!(first.to, points[3], "insights in x order: day 3 first");
    assert!(first.landmark);
    assert_eq!(cursor.apply(&tree, &Verb::NextInsight).cursor(), points[7]);
    assert!(matches!(
        cursor.apply(&tree, &Verb::NextInsight),
        Outcome::Boundary {
            edge: Edge::Last,
            ..
        }
    ));

    let mut cursor = Cursor::at(&tree, points[0]).unwrap();
    cursor.apply(&tree, &Verb::GoTo(XTarget::Number(16.4)));
    assert_eq!(cursor.node(), points[16]);
    cursor.apply(&tree, &Verb::Landmark(Landmark::XAxis));
    assert_eq!(
        cursor.node(),
        tree.children(tree.landmark("axes").unwrap())[0]
    );
    cursor.apply(&tree, &Verb::Landmark(Landmark::Dataset));
    assert_eq!(cursor.node(), tree.root());

    // "Find value": the next point equal to or crossing 15.5 after day 0 (10, 13, 16).
    let mut cursor = Cursor::at(&tree, points[0]).unwrap();
    cursor.apply(&tree, &Verb::FindValue(15.5));
    assert_eq!(cursor.node(), points[2]);
}

/// Enter on a filter toggles its series; on a point it asks for the full description.
#[test]
fn enter_toggles_filters_and_describes_points() {
    let tree = fixture();
    let filters = tree.landmark("filters").unwrap();
    let filter = tree.children(filters)[0];
    let mut cursor = Cursor::at(&tree, filter).unwrap();
    let Outcome::Toggled {
        series, included, ..
    } = cursor.apply(&tree, &Verb::Activate)
    else {
        panic!("Enter toggles");
    };
    assert!(!included);
    assert!(cursor.is_excluded(series));
    assert!(matches!(
        cursor.apply(&tree, &Verb::Activate),
        Outcome::Toggled { included: true, .. }
    ));
    let point = tree.points(tree.series().next().unwrap())[0];
    let mut cursor = Cursor::at(&tree, point).unwrap();
    assert_eq!(
        cursor.apply(&tree, &Verb::Activate),
        Outcome::Describe { node: point }
    );
}

/// A11Y-07: "where am I" is defined at every node, from the root to the cursor, with
/// positions counted among peers ("point 13 of 20", "series 1 of 2").
#[test]
fn a11y_07_where_am_i_is_defined_at_every_node() {
    let tree = fixture();
    for (id, _) in tree.iter() {
        let report = Cursor::at(&tree, id).unwrap().where_am_i(&tree);
        assert_eq!(report.path.first().unwrap().node, tree.root());
        assert_eq!(report.path.last().unwrap().node, id);
        for step in &report.path {
            assert!(step.position >= 1 && step.position <= step.of, "{step:?}");
        }
    }
    let temp = tree.series().next().unwrap();
    let point = tree.points(temp)[12];
    let report = Cursor::at(&tree, point).unwrap().where_am_i(&tree);
    let positions: Vec<(u64, u64)> = report.path.iter().map(|s| (s.position, s.of)).collect();
    assert_eq!(positions, [(1, 1), (1, 2), (2, 2), (13, 20)]);
    assert_eq!(report.previous, Some(tree.points(temp)[11]));
    assert_eq!(report.next, Some(tree.points(temp)[13]));
}

fn any_verb() -> impl Strategy<Value = Verb> {
    prop_oneof![
        Just(Verb::Next),
        Just(Verb::Previous),
        Just(Verb::JumpForward),
        Just(Verb::JumpBack),
        Just(Verb::First),
        Just(Verb::Last),
        Just(Verb::Into),
        Just(Verb::Out),
        Just(Verb::Activate),
        Just(Verb::NextSeries),
        Just(Verb::PreviousSeries),
        Just(Verb::Maximum),
        Just(Verb::Minimum),
        Just(Verb::NextInsight),
        Just(Verb::PreviousInsight),
        Just(Verb::Landmark(Landmark::XAxis)),
        Just(Verb::Landmark(Landmark::YAxis)),
        Just(Verb::Landmark(Landmark::Insights)),
        Just(Verb::Landmark(Landmark::Filters)),
        Just(Verb::Landmark(Landmark::Dataset)),
        (-5.0_f64..30.0).prop_map(|x| Verb::GoTo(XTarget::Number(x))),
        (0.0_f64..60.0).prop_map(Verb::FindValue),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A11Y-04, A11Y-27: over any verb sequence, the cursor is never undefined, boundary
    /// and invalid moves leave it in place, and a moved cursor is where the outcome says.
    #[test]
    fn a11y_04_verb_sequences_never_leave_the_tree(
        verbs in prop::collection::vec(any_verb(), 1..60),
    ) {
        let tree = fixture();
        let mut cursor = Cursor::new();
        for verb in &verbs {
            let before = cursor.node();
            let outcome = cursor.apply(&tree, verb);
            prop_assert!(tree.node(cursor.node()).is_some());
            prop_assert_eq!(outcome.cursor(), cursor.node());
            match outcome {
                Outcome::Boundary { .. } | Outcome::Invalid { .. }
                | Outcome::Describe { .. } | Outcome::Toggled { .. } => {
                    prop_assert_eq!(cursor.node(), before);
                }
                Outcome::Moved(m) => {
                    prop_assert_eq!(m.from, before);
                    prop_assert_eq!(m.to, cursor.node());
                }
                _ => {}
            }
        }
    }
}

/// A11Y-04: no dead end. From every node, Up leads back to the root and some verb moves
/// the cursor; from the root, Down and Right reach every node.
#[test]
fn a11y_04_no_dead_end() {
    let tree = fixture();
    for (id, _) in tree.iter() {
        let mut cursor = Cursor::at(&tree, id).unwrap();
        for _ in 0..4 {
            cursor.apply(&tree, &Verb::Out);
        }
        assert_eq!(cursor.node(), tree.root(), "Up from {id:?}");
        let moves = verbs().into_iter().any(|(_, verb)| {
            let mut probe = Cursor::at(&tree, id).unwrap();
            matches!(probe.apply(&tree, &verb), Outcome::Moved(_))
        });
        assert!(moves, "a verb moves from {id:?}");
    }
    // Reachability by keys: depth-first over Down and Right from the root.
    let mut reached = std::collections::BTreeSet::new();
    let mut stack = vec![Cursor::new()];
    while let Some(cursor) = stack.pop() {
        if !reached.insert(cursor.node()) {
            continue;
        }
        for verb in [Verb::Into, Verb::Next] {
            let mut next = cursor.clone();
            if let Outcome::Moved(_) = next.apply(&tree, &verb) {
                stack.push(next);
            }
        }
        // Down lands on the child at the remembered x; Home reaches the first child too.
        let mut first = cursor.clone();
        if let Outcome::Moved(_) = first.apply(&tree, &Verb::First) {
            stack.push(first);
        }
    }
    assert_eq!(reached.len(), tree.len(), "every node is reachable by keys");
}
