// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fmt::Write as _;

use sonarithm_core::nav::{DatasetNode, NavNode, PointNode, SegmentNode, SeriesNode, XValue};
use sonarithm_core::{Generation, Provenance, RowRef, TableId, TrackId};

use super::*;

fn field(part: Part, text: &str) -> ReviewField {
    ReviewField {
        part,
        text: text.to_owned(),
    }
}

/// The default terse fields of a point (keyboard §8.1), in symbol form.
fn point_fields() -> Vec<ReviewField> {
    vec![
        field(Part::Value, "28.7 °C"),
        field(Part::X, "Mar 14"),
        field(Part::Position, "73/365"),
        field(Part::Provenance, "est."),
    ]
}

/// A11Y-21, keyboard §10: the review line at several cell counts. Whole fields only, and
/// the "more" mark when some are left out.
#[test]
fn a11y_21_review_line_at_several_cell_counts() {
    let fields = point_fields();
    let mut out = String::new();
    for cells in [None, Some(4), Some(8), Some(14), Some(20), Some(40)] {
        let line = review_line(&fields, cells);
        let label = cells.map_or("unset".to_owned(), |c| c.to_string());
        let _ = writeln!(out, "{label:>5} | {line} | {}", line.chars().count());
        if let Some(cells) = cells {
            let first = fields[0].text.chars().count();
            assert!(line.chars().count() <= cells.max(first), "{cells}: {line}");
        }
    }
    insta::assert_snapshot!(out);
}

/// A number is never cut: the first field is kept whole even on a display that is too
/// short, and a later field is either whole or left out.
#[test]
fn a11y_21_review_line_never_cuts_a_field() {
    let fields = point_fields();
    let texts: Vec<&str> = fields.iter().map(|f| f.text.as_str()).collect();
    for cells in 0..60 {
        let line = review_line(&fields, Some(cells));
        let body = line.trim_end_matches(MORE);
        let whole = (1..=texts.len()).any(|n| texts[..n].join(" ") == body);
        assert!(whole, "{cells}: {line} is not a run of whole fields");
        assert!(line.starts_with("28.7 °C"), "{cells}: {line}");
    }
    assert_eq!(
        review_line(&fields, None),
        "28.7 °C Mar 14 73/365 est.",
        "no cell count: every field, the screen reader pans"
    );
}

/// MAIDR's four heights (notes 03 §1): dots 7–8, 3–6, 2–5, 1–4, as Unicode 8-dot cells.
#[test]
fn a11y_21_sparkline_levels_are_maidr_dot_chords() {
    let cells: String = (0..4).map(cell).collect();
    assert_eq!(cells, "\u{28C0}\u{2824}\u{2812}\u{2809}");
    assert_eq!(level(0.0, 0.0, 1.0), 0);
    assert_eq!(level(0.24, 0.0, 1.0), 0);
    assert_eq!(level(0.25, 0.0, 1.0), 1);
    assert_eq!(level(0.5, 0.0, 1.0), 2);
    assert_eq!(level(1.0, 0.0, 1.0), 3);
    assert_eq!(
        level(5.0, 5.0, 5.0),
        2,
        "an even segment sits at the third height"
    );
}

fn point(row: u64, value: Option<f64>) -> NavNode {
    #[expect(clippy::cast_precision_loss, reason = "small test rows")]
    let x = row as f64;
    NavNode::new(NavKind::Point(PointNode {
        row: RowRef {
            table: TableId(0),
            generation: Generation(0),
            row,
        },
        index: row,
        t: None,
        x: Some(XValue::Number(x)),
        value,
        provenance: Provenance::EXACT,
    }))
}

/// One series with one segment of `values`; returns the tree and the point IDs.
fn tree_of(values: &[Option<f64>]) -> (NavTree, Vec<NavNodeId>) {
    let mut tree = NavTree::new(DatasetNode::default());
    let n = u64::try_from(values.len()).unwrap();
    let series = tree.push(
        NavNodeId::ROOT,
        NavNode::new(NavKind::Series(SeriesNode {
            track: TrackId::new("y"),
            field: None,
            unit: None,
            ucum: None,
            points: n,
            range: None,
            pitch_hz: None,
            polarity: None,
        })),
    );
    let segment = tree.push(
        series,
        NavNode::new(NavKind::Segment(SegmentNode {
            semantic: false,
            points: n,
            range: None,
        })),
    );
    let ids = values
        .iter()
        .enumerate()
        .map(|(row, &value)| tree.push(segment, point(u64::try_from(row).unwrap(), value)))
        .collect();
    (tree, ids)
}

/// A11Y-21: one cell per point at its height in the segment's range; a missing value is a
/// blank cell; the caret is on the cursor's cell.
#[test]
fn a11y_21_sparkline_encodes_the_cursor_segment() {
    let (tree, ids) = tree_of(&[Some(0.0), Some(1.0), None, Some(2.0), Some(4.0)]);
    let line = sparkline(&tree, ids[3], None).unwrap();
    assert_eq!(line.text, "\u{28C0}\u{2824}\u{2800}\u{2812}\u{2809}");
    assert_eq!(line.caret, 3);
    assert_eq!(line.points, ids);
    assert_eq!(
        sparkline(&tree, NavNodeId::ROOT, None),
        None,
        "not on a point"
    );
}

/// With fewer cells than points, the window holds the cursor, centred where the ends allow.
#[test]
fn a11y_21_sparkline_window_keeps_the_cursor_visible() {
    let values: Vec<Option<f64>> = (0..20).map(|v| Some(f64::from(v))).collect();
    let (tree, ids) = tree_of(&values);
    for (at, start) in [(0, 0), (3, 0), (10, 6), (19, 12)] {
        let line = sparkline(&tree, ids[at], Some(8)).unwrap();
        assert_eq!(line.text.chars().count(), 8);
        assert_eq!(line.points[0], ids[start], "cursor at {at}");
        assert_eq!(line.points[line.caret], ids[at]);
    }
}

/// A11Y-27, keyboard §14: a routing key on a cell selects the point under it.
#[test]
fn a11y_27_routing_selects_the_point_under_the_cell() {
    let values: Vec<Option<f64>> = (0..20).map(|v| Some(f64::from(v))).collect();
    let (tree, ids) = tree_of(&values);
    let line = sparkline(&tree, ids[10], Some(8)).unwrap();
    assert_eq!(route(&line, 0), Some(ids[6]));
    assert_eq!(route(&line, 7), Some(ids[13]));
    assert_eq!(route(&line, 8), None, "past the last cell");
}
