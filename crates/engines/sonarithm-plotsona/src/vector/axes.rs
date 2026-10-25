// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Finding a chart's axes in its scene.
//!
//! Tick labels of the x axis share a baseline: they form the fullest **row** of two or
//! more texts (the lowest on a tie). Tick labels of the y axis share an alignment edge: they form the leftmost
//! **column** of two or more texts. A label is drawn beside its tick, offset by a constant
//! (a baseline shift, a rotation, a gap), so each label is snapped to the nearest tick or
//! grid line after removing that offset: the lines give exact positions, the text only
//! approximate ones. Labels that read as numbers or dates make a continuous axis,
//! calibrated linear, log or date; other labels make a category axis.

use std::collections::BTreeMap;

use sonarithm_core::{Confidence, Diagnostic};

use super::{Pt, Scene, Text};
use crate::calibrate::ticks::{Tick, date_anchors, looks_like_dates, number_anchors};
use crate::calibrate::{Anchor, Calibration, CategoryAxis, ScaleKind, check_monotonic};

/// Which way an axis runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Dir {
    /// Positions are x coordinates.
    Horizontal,
    /// Positions are y coordinates.
    Vertical,
}

impl Dir {
    pub(crate) const fn of(self, p: Pt) -> f64 {
        match self {
            Self::Horizontal => p.0,
            Self::Vertical => p.1,
        }
    }

    const fn across(self, p: Pt) -> f64 {
        match self {
            Self::Horizontal => p.1,
            Self::Vertical => p.0,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "x",
            Self::Vertical => "y",
        }
    }
}

/// A found axis.
#[derive(Clone, Debug)]
pub(crate) enum Axis {
    /// Numbers or dates through a fitted scale.
    Continuous {
        calibration: Calibration,
        dir: Dir,
        /// The lowest and highest tick positions.
        span: (f64, f64),
    },
    /// Labels at positions.
    Category {
        axis: CategoryAxis,
        dir: Dir,
        span: (f64, f64),
    },
}

impl Axis {
    pub(crate) const fn dir(&self) -> Dir {
        match self {
            Self::Continuous { dir, .. } | Self::Category { dir, .. } => *dir,
        }
    }

    pub(crate) const fn span(&self) -> (f64, f64) {
        match self {
            Self::Continuous { span, .. } | Self::Category { span, .. } => *span,
        }
    }

    pub(crate) fn confidence(&self) -> Confidence {
        match self {
            Self::Continuous { calibration, .. } => calibration.confidence(),
            Self::Category { .. } => Confidence::CERTAIN,
        }
    }

    pub(crate) fn residual(&self) -> Option<f64> {
        match self {
            Self::Continuous { calibration, .. } => Some(calibration.residual()),
            Self::Category { .. } => None,
        }
    }

    pub(crate) const fn is_category(&self) -> bool {
        matches!(self, Self::Category { .. })
    }
}

/// The two axes of a chart, when found.
#[derive(Debug, Default)]
pub(crate) struct Axes {
    pub(crate) x: Option<Axis>,
    pub(crate) y: Option<Axis>,
    /// Indices of the texts used as tick labels.
    pub(crate) labels: Vec<usize>,
}

/// Rounds a coordinate into a 1.5-unit bucket for grouping aligned texts.
fn bucket(v: f64) -> i64 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "drawing coordinates are small"
    )]
    let b = (v / 1.5).round() as i64;
    b
}

/// Finds the axes; calibration problems become diagnostics.
pub(crate) fn find(scene: &Scene, diagnostics: &mut Vec<Diagnostic>) -> Axes {
    let mut axes = Axes::default();
    let candidates: Vec<usize> = (0..scene.texts.len())
        .filter(|i| scene.texts.get(*i).is_some_and(|t| !is_title(t)))
        .collect();
    // x axis: a row of at least two labels spread along x.
    // Rotated labels line up by one end: the anchor, or (for text measured from its
    // glyphs, as in PDF) the end, so rows are formed by both.
    let mut rows: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    for i in &candidates {
        if let Some(t) = scene.texts.get(*i) {
            rows.entry(bucket(t.at.1)).or_default().push(*i);
        }
    }
    let mut ends: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    for i in &candidates {
        if let Some(t) = scene.texts.get(*i).filter(|t| t.rotated) {
            ends.entry(bucket(t.end.1)).or_default().push(*i);
        }
    }
    // The x tick labels are the fullest row, the lowest one on a tie.
    let row = rows
        .iter()
        .chain(&ends)
        .filter(|(_, m)| m.len() >= 2 && spread(scene, m, Dir::Horizontal) > 1.0)
        .max_by_key(|(k, m)| (m.len(), **k))
        .map(|(_, m)| m.clone());
    let row = row.as_ref();
    if let Some(row) = row {
        axes.x = build(scene, row, Dir::Horizontal, diagnostics);
        if axes.x.is_some() {
            axes.labels.extend(row);
        }
    }
    // y axis: the leftmost column of at least two labels spread along y, not used above.
    // Labels align by their end (right-aligned), anchor or start (left-aligned).
    let mut columns: Vec<BTreeMap<i64, Vec<usize>>> = vec![BTreeMap::new(); 3];
    for i in &candidates {
        if axes.labels.contains(i) {
            continue;
        }
        if let Some(t) = scene.texts.get(*i) {
            for (map, x) in columns.iter_mut().zip([t.end.0, t.at.0, t.start.0]) {
                map.entry(bucket(x)).or_default().push(*i);
            }
        }
    }
    let column = columns
        .iter()
        .flat_map(BTreeMap::values)
        .filter(|m| m.len() >= 2 && spread(scene, m, Dir::Vertical) > 1.0)
        // Prefer a numeric column (a legend's labels form a column too), then the fullest,
        // then the leftmost.
        .max_by_key(|m| {
            let numeric = m
                .iter()
                .filter(|i| {
                    scene.texts.get(**i).is_some_and(|t| {
                        crate::calibrate::ticks::parse_number(&t.content).is_some()
                    })
                })
                .count();
            (
                numeric * 2 >= m.len(),
                m.len(),
                std::cmp::Reverse(bucket_x(scene, m)),
            )
        });
    if let Some(column) = column {
        axes.y = build(scene, column, Dir::Vertical, diagnostics);
        if axes.y.is_some() {
            axes.labels.extend(column);
        }
    }
    axes
}

fn bucket_x(scene: &Scene, members: &[usize]) -> i64 {
    members
        .first()
        .and_then(|i| scene.texts.get(*i))
        .map_or(0, |t| bucket(t.end.0))
}

fn spread(scene: &Scene, members: &[usize], dir: Dir) -> f64 {
    let values: Vec<f64> = members
        .iter()
        .filter_map(|i| scene.texts.get(*i))
        .map(|t| dir.of(t.at))
        .collect();
    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    hi - lo
}

/// Whether a text is a chart or axis title rather than a tick label.
pub(crate) fn is_title(text: &Text) -> bool {
    let c = text.classes.to_ascii_lowercase();
    c.contains("title") || c.contains("legend")
}

/// Positions of tick and grid lines across `dir`: straight two-point segments
/// perpendicular to the axis direction.
fn line_positions(scene: &Scene, dir: Dir) -> Vec<f64> {
    let mut found: Vec<f64> = scene
        .shapes
        .iter()
        .filter(|s| s.points.len() == 2 && s.stroke.is_some())
        .filter_map(|s| {
            let (a, b) = (s.points.first()?, s.points.get(1)?);
            let along = (dir.of(*a) - dir.of(*b)).abs();
            let across = (dir.across(*a) - dir.across(*b)).abs();
            (along < 1e-6 && across > 1e-6).then_some(dir.of(*a))
        })
        .collect();
    found.sort_by(f64::total_cmp);
    found.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    found
}

/// Builds an axis from label texts.
fn build(
    scene: &Scene,
    members: &[usize],
    dir: Dir,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Axis> {
    let texts: Vec<&Text> = members.iter().filter_map(|i| scene.texts.get(*i)).collect();
    let positions = snap(&texts, &line_positions(scene, dir), dir);
    let ticks: Vec<Tick> = texts
        .iter()
        .zip(&positions)
        .map(|(t, p)| Tick::new(*p, t.content.clone()))
        .collect();
    let span = positions
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(*p), hi.max(*p))
        });
    let numeric = ticks
        .iter()
        .filter(|t| crate::calibrate::ticks::parse_number(&t.text).is_some())
        .count();
    let continuous =
        |anchors: Vec<Anchor>, kind: Option<ScaleKind>, diagnostics: &mut Vec<Diagnostic>| {
            if let Err(d) = check_monotonic(&anchors, dir.name()) {
                diagnostics.push(*d);
                return None;
            }
            let fit = match kind {
                Some(kind) => Calibration::fit(kind, &anchors),
                None => Calibration::fit_auto(&anchors),
            };
            match fit {
                Ok(calibration) => {
                    diagnostics.extend(calibration.check(&anchors, dir.name()));
                    Some(Axis::Continuous {
                        calibration,
                        dir,
                        span,
                    })
                }
                Err(d) => {
                    diagnostics.push(*d);
                    None
                }
            }
        };
    if looks_like_dates(&ticks) {
        let (anchors, unread) = date_anchors(&ticks, dir.name());
        diagnostics.extend(unread);
        return continuous(anchors, Some(ScaleKind::Date), diagnostics);
    }
    if numeric * 2 > ticks.len() && numeric >= 2 {
        let (anchors, unread) = number_anchors(&ticks, dir.name());
        diagnostics.extend(unread);
        return continuous(anchors, None, diagnostics);
    }
    Some(Axis::Category {
        axis: CategoryAxis::new(ticks.into_iter().map(|t| (t.position, t.text)).collect()),
        dir,
        span,
    })
}

/// Label positions snapped to the nearest line after removing the labels' common offset.
fn snap(texts: &[&Text], lines: &[f64], dir: Dir) -> Vec<f64> {
    let raw: Vec<f64> = texts.iter().map(|t| dir.of(t.at)).collect();
    if lines.is_empty() {
        return raw;
    }
    let nearest = |v: f64| {
        lines
            .iter()
            .copied()
            .min_by(|a, b| (a - v).abs().total_cmp(&(b - v).abs()))
            .unwrap_or(v)
    };
    let mut offsets: Vec<f64> = raw.iter().map(|v| v - nearest(*v)).collect();
    offsets.sort_by(f64::total_cmp);
    let offset = offsets.get(offsets.len() / 2).copied().unwrap_or(0.0);
    let mut sorted = raw.clone();
    sorted.sort_by(f64::total_cmp);
    let gap = sorted
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|g| *g > 1e-6)
        .fold(f64::INFINITY, f64::min);
    let tolerance = if gap.is_finite() { gap / 4.0 } else { 2.0 };
    raw.iter()
        .map(|v| {
            let shifted = v - offset;
            let line = nearest(shifted);
            if (line - shifted).abs() <= tolerance {
                line
            } else {
                shifted
            }
        })
        .collect()
}
