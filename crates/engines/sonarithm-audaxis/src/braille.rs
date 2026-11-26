// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Braille output (B03-T10; keyboard-and-navigation §10, §14; A11Y-21, A11Y-27).
//!
//! Two forms follow the one cursor (A11Y-10):
//!
//! - **The review line**: the cursor's fields (value with its unit, x, position…) in the
//!   listener's part order (A11Y-12), already formatted by the composer in symbol form.
//!   [`review_line`] only fits them to the display.
//! - **The sparkline** (Shift+B): one 8-dot cell per point of the cursor's segment, at one
//!   of four heights. [`sparkline`] builds it from the navigation tree, and [`route`] maps a
//!   routing-key press on a cell back to its point.
//!
//! **No invented sizes.** A web page cannot read a display's cell count, so it is the
//! listener's setting, as in MAIDR. Without one nothing is fitted and the screen reader
//! pans the line (keyboard §14). Widths are counted in characters, one cell each, which is
//! exact for computer (8-dot) braille; contracted braille tables can shorten text further
//! (checked on a display in B07, HP-10).
//!
//! Pure functions: no I/O, no state, the same output for the same input (ADR-0013).

use sonarithm_core::nav::{NavKind, NavNodeId, NavTree};

use crate::announce::Part;

/// The four heights of a sparkline cell, lowest first, as 8-dot patterns: dots 7–8, 3–6,
/// 2–5 and 1–4, MAIDR's bar encodings (notes 03 §1). Bit `n − 1` of a pattern is dot `n`.
pub const LEVELS: [u8; 4] = [0b1100_0000, 0b0010_0100, 0b0001_0010, 0b0000_1001];

/// The blank cell: a missing value (03 §5).
pub const BLANK: char = '\u{2800}';

/// Marks a review line that left out fields for lack of cells.
pub const MORE: char = '…';

/// One formatted field of the review line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewField {
    /// Which part it is (keyboard §8.1).
    pub part: Part,
    /// Its text in symbol form, such as "28.7 °C".
    pub text: String,
}

/// The review line from `fields`, in their order, fitted to `cells`.
///
/// Fields are joined by one space. With no cell count, every field is kept and the screen
/// reader pans. With one, and when not every field fits:
///
/// - the first field (the value) and every provenance qualifier are always kept, so an
///   estimate is never shown as if it were exact (AGENTS rule 7, A11Y-02);
/// - the other fields are added in their order while they fit, with one cell kept back
///   for [`MORE`], which ends the line whenever a field was left out;
/// - kept fields stay in the listener's order (A11Y-12).
///
/// A field is never cut, so no number is ever shortened. When the value and its qualifier
/// alone are longer than the display, they are shown whole and the screen reader pans.
#[must_use]
pub fn review_line(fields: &[ReviewField], cells: Option<usize>) -> String {
    let kept: Vec<(Part, &str)> = fields
        .iter()
        .map(|field| (field.part, field.text.trim()))
        .filter(|(_, text)| !text.is_empty())
        .collect();
    let all = kept
        .iter()
        .map(|(_, text)| text.chars().count())
        .sum::<usize>()
        + kept.len().saturating_sub(1);
    let Some(cells) = cells.filter(|&cells| all > cells) else {
        return kept
            .iter()
            .map(|(_, text)| *text)
            .collect::<Vec<_>>()
            .join(" ");
    };
    // A field will be left out, so one cell is kept back for the mark.
    let budget = cells.saturating_sub(1);
    let must = |index: usize, part: Part| index == 0 || part == Part::Provenance;
    let width_of = |text: &str| text.chars().count();
    let mut chosen: Vec<bool> = kept
        .iter()
        .enumerate()
        .map(|(i, (p, _))| must(i, *p))
        .collect();
    let mut width = kept
        .iter()
        .zip(&chosen)
        .filter(|(_, keep)| **keep)
        .map(|((_, text), _)| width_of(text))
        .sum::<usize>()
        + chosen
            .iter()
            .filter(|keep| **keep)
            .count()
            .saturating_sub(1);
    for (index, (_, text)) in kept.iter().enumerate() {
        if chosen[index] {
            continue;
        }
        let needed = width + 1 + width_of(text);
        if needed <= budget {
            chosen[index] = true;
            width = needed;
        }
    }
    let mut line = kept
        .iter()
        .zip(&chosen)
        .filter(|(_, keep)| **keep)
        .map(|((_, text), _)| *text)
        .collect::<Vec<_>>()
        .join(" ");
    if chosen.iter().any(|keep| !keep) && width < cells {
        line.push(MORE);
    }
    line
}

/// A sparkline of the cursor's segment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sparkline {
    /// The cells, one per point shown.
    pub text: String,
    /// The cell of the cursor's point, where the caret goes.
    pub caret: usize,
    /// The point under each cell, for routing keys.
    pub points: Vec<NavNodeId>,
}

/// The sparkline of the segment holding the point `cursor` (its parent segment, or its
/// series when it has none), or `None` when the cursor is not on a point.
///
/// Each point's value is placed in the segment's range, from its smallest to its largest
/// value, divided into four equal bands; a segment whose values are all equal is drawn at
/// the third height (band 2 of 0–3). A missing value is a blank cell. With a cell count
/// smaller than the segment, the cells shown are a window around the cursor, so its point
/// is always visible.
#[must_use]
pub fn sparkline(tree: &NavTree, cursor: NavNodeId, cells: Option<usize>) -> Option<Sparkline> {
    let node = tree.node(cursor)?;
    if !matches!(node.kind, NavKind::Point(_)) {
        return None;
    }
    let parent = node.parent?;
    let points: Vec<NavNodeId> = match tree.node(parent).map(|n| &n.kind) {
        Some(NavKind::Segment(_)) => tree
            .children(parent)
            .iter()
            .copied()
            .filter(|&id| matches!(tree.node(id).map(|n| &n.kind), Some(NavKind::Point(_))))
            .collect(),
        _ => tree.points(parent),
    };
    let at = points.iter().position(|&id| id == cursor)?;
    let value = |id: NavNodeId| match tree.node(id).map(|n| &n.kind) {
        Some(NavKind::Point(point)) => point.value.filter(|v| v.is_finite()),
        _ => None,
    };
    let values: Vec<Option<f64>> = points.iter().map(|&id| value(id)).collect();
    let (low, high) = values
        .iter()
        .flatten()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let (start, end) = window(points.len(), at, cells);
    let text = values[start..end]
        .iter()
        .map(|v| v.map_or(BLANK, |v| cell(level(v, low, high))))
        .collect();
    Some(Sparkline {
        text,
        caret: at - start,
        points: points[start..end].to_vec(),
    })
}

/// The point under cell `cell` of a sparkline: where a routing-key press moves the cursor
/// (keyboard §14). In review mode a routing key moves nothing, so it has no counterpart.
#[must_use]
pub fn route(sparkline: &Sparkline, cell: usize) -> Option<NavNodeId> {
    sparkline.points.get(cell).copied()
}

/// The band of `value` in `low..=high`, from 0 (lowest) to 3.
fn level(value: f64, low: f64, high: f64) -> usize {
    let share = if high > low {
        (value - low) / (high - low)
    } else {
        0.5
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "the share is clamped to [0, 1] and there are four levels, so the band is 0 to 4"
    )]
    let band = (share.clamp(0.0, 1.0) * LEVELS.len() as f64).floor() as usize;
    band.min(LEVELS.len() - 1)
}

/// The braille character of a height.
fn cell(level: usize) -> char {
    let dots = LEVELS.get(level).copied().unwrap_or(0);
    char::from_u32(0x2800 + u32::from(dots)).unwrap_or(BLANK)
}

/// The cells shown, `start..end`: everything without a cell count, else at most `cells`
/// around `at`, the cursor in the middle where the ends allow.
fn window(len: usize, at: usize, cells: Option<usize>) -> (usize, usize) {
    match cells {
        Some(cells) if cells > 0 && cells < len => {
            let start = at.saturating_sub(cells / 2).min(len - cells);
            (start, start + cells)
        }
        _ => (0, len),
    }
}

#[cfg(test)]
mod tests;
