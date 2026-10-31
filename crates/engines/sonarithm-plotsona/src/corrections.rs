// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Applying chart corrections (C02-T06; RFC-0004 §6, RFC-0005 amendment 3).
//!
//! Edits apply in order to the chart as extracted, after extraction and before any
//! `transform`. A point is found by its stable ID: `<series>#<index>` in the extractor's
//! order, or `+<edit>` for a point an `add` created. Every edit that refers to a point is
//! guarded by its `was` value; an edit whose point is gone or whose guard fails is not
//! applied and is reported (`SNR-ENG-386`), so no edit ever lands on a point other than the
//! one the person saw. Moved, added and retyped points become `user_edited` with no
//! confidence; deleted points are removed. The report keeps every edit with the point's
//! value and provenance before it: the history 03 §4 asks for.

use std::collections::BTreeMap;

use serde::Serialize;
use sonarithm_core::{Confidence, Diagnostic, Provenance, ProvenanceClass, Sha256Digest};
use sonarithm_spec::correction_file::{CorrectionFile, FileEdit, PointRef};
use sonarithm_spec::v02::{AnchorValue, Corrections, Edit, PointValue};

use crate::calibrate::ticks::parse_date;
use crate::codes;
use crate::model::{ChartData, Point, Series, XValue};

/// The corrections to apply to an extraction: inline edits from a spec (RFC-0004 §6) or a
/// correction file (RFC-0005).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CorrectionSet {
    /// `data.source.corrections.edits`.
    Inline(Corrections),
    /// A correction file, bound to its chart by SHA-256.
    File(Box<CorrectionFile>),
}

/// One edit as it was applied, or not: the extraction report's correction history.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[non_exhaustive]
pub struct CorrectionRecord {
    /// The edit's ID: `e<n>` in a file, `#<n>` (its 0-based position) inline.
    pub edit: String,
    /// `add`, `move`, `delete` or `retype`.
    pub op: &'static str,
    /// The point's stable ID.
    pub point: String,
    /// Whether the edit was applied.
    pub applied: bool,
    /// The point's value before the edit; `None` for an `add`.
    pub was: Option<Point>,
    /// The point's provenance before the edit; `None` for an `add`.
    pub before: Option<Provenance>,
}

/// One edit, from either source, in one shape.
struct Step<'a> {
    edit: String,
    op: &'static str,
    series: &'a str,
    /// The point's ID, for edits on a point.
    target: Option<String>,
    was: Option<&'a PointValue>,
    to: Option<&'a PointValue>,
    retype_to: Option<&'a str>,
}

impl CorrectionSet {
    fn steps(&self) -> Vec<Step<'_>> {
        let id = |series: &str, point: &PointRef| match point {
            PointRef::Index(i) => format!("{series}#{i}"),
            PointRef::Added(added) => added.clone(),
        };
        match self {
            Self::Inline(corrections) => corrections
                .edits
                .iter()
                .enumerate()
                .map(|(n, edit)| {
                    let mut step = Step {
                        edit: format!("#{n}"),
                        op: "add",
                        series: "",
                        target: None,
                        was: None,
                        to: None,
                        retype_to: None,
                    };
                    match edit {
                        Edit::Add { series, to } => {
                            step.series = series;
                            step.to = Some(to);
                        }
                        Edit::Move {
                            series,
                            point,
                            was,
                            to,
                        } => {
                            (step.op, step.series, step.was, step.to) =
                                ("move", series, Some(was), Some(to));
                            step.target = Some(format!("{series}#{point}"));
                        }
                        Edit::Delete { series, point, was } => {
                            (step.op, step.series, step.was) = ("delete", series, Some(was));
                            step.target = Some(format!("{series}#{point}"));
                        }
                        Edit::Retype {
                            series,
                            point,
                            was,
                            to,
                        } => {
                            (step.op, step.series, step.was) = ("retype", series, Some(was));
                            step.target = Some(format!("{series}#{point}"));
                            step.retype_to = Some(to.series.as_str());
                        }
                        _ => step.op = "unknown",
                    }
                    step
                })
                .collect(),
            Self::File(file) => file
                .edits
                .iter()
                .map(|edit| {
                    let mut step = Step {
                        edit: edit.id().to_owned(),
                        op: "add",
                        series: edit.series(),
                        target: edit.point().map(|p| id(edit.series(), p)),
                        was: None,
                        to: None,
                        retype_to: None,
                    };
                    match edit {
                        FileEdit::Add { to, .. } => step.to = Some(to),
                        FileEdit::Move { was, to, .. } => {
                            (step.op, step.was, step.to) = ("move", Some(was), Some(to));
                        }
                        FileEdit::Delete { was, .. } => (step.op, step.was) = ("delete", Some(was)),
                        FileEdit::Retype { was, to, .. } => {
                            (step.op, step.was, step.retype_to) =
                                ("retype", Some(was), Some(to.series.as_str()));
                        }
                        _ => step.op = "unknown",
                    }
                    step
                })
                .collect(),
        }
    }
}

/// A point while edits apply.
#[derive(Clone, Debug)]
struct Slot {
    series: String,
    point: Point,
    provenance: Provenance,
    deleted: bool,
    /// Its place among the series' points: original points first, in order; added and
    /// retyped points after them, in the order the edits made them.
    order: (usize, usize),
}

/// How far a guard value may be from the point's value: 0.1 % of the chart's range on
/// that axis, at least 1e-9. Values are typed from announcements, which round (A11Y-29).
#[derive(Clone, Copy)]
struct Tolerance {
    x: f64,
    y: f64,
}

fn tolerance(chart: &ChartData) -> Tolerance {
    let span = |values: &mut dyn Iterator<Item = f64>| {
        let (lo, hi) = values.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        });
        if hi > lo { (hi - lo) * 1e-3 } else { 1e-9 }.max(1e-9)
    };
    let points = || chart.series.iter().flat_map(|s| s.points.iter());
    #[expect(
        clippy::cast_precision_loss,
        reason = "milliseconds since 1970 fit f64"
    )]
    let x = span(&mut points().filter_map(|p| match p.x {
        XValue::Number(x) => Some(x),
        XValue::Time(t) => Some(t as f64),
        XValue::Category(_) => None,
    }));
    Tolerance {
        x,
        y: span(&mut points().filter_map(|p| p.y)),
    }
}

/// The x value an anchor value stands for, in the kind of `like`.
fn x_of(value: &AnchorValue, like: &XValue) -> Option<XValue> {
    Some(match (value, like) {
        (AnchorValue::Text(text), XValue::Time(_)) => XValue::Time(parse_date(text)?),
        #[expect(clippy::cast_possible_truncation, reason = "a time in milliseconds")]
        (AnchorValue::Number(n), XValue::Time(_)) => XValue::Time(n.round() as i64),
        (AnchorValue::Number(n), XValue::Number(_)) => XValue::Number(*n),
        (AnchorValue::Text(text), XValue::Category(_)) => XValue::Category(text.clone()),
        (AnchorValue::Number(n), XValue::Category(_)) => XValue::Category(n.to_string()),
        (AnchorValue::Text(text), XValue::Number(_)) => XValue::Number(text.trim().parse().ok()?),
    })
}

/// Whether `point` matches the guard `was`.
fn matches(point: &Point, was: &PointValue, tol: Tolerance) -> bool {
    let x_ok = was
        .x
        .as_ref()
        .is_none_or(|x| match (x_of(x, &point.x), &point.x) {
            (Some(XValue::Number(a)), XValue::Number(b)) => (a - b).abs() <= tol.x,
            #[expect(
                clippy::cast_precision_loss,
                reason = "milliseconds since 1970 fit f64"
            )]
            (Some(XValue::Time(a)), XValue::Time(b)) => ((a - b) as f64).abs() <= tol.x,
            (Some(XValue::Category(a)), XValue::Category(b)) => a == *b,
            _ => false,
        });
    let y_ok = was
        .y
        .is_none_or(|y| point.y.is_some_and(|v| (v - y).abs() <= tol.y));
    x_ok && y_ok && (was.x.is_some() || was.y.is_some())
}

/// The point at the edit's values, starting from `from`.
fn moved(from: &Point, to: &PointValue) -> Option<Point> {
    let mut point = from.clone();
    if let Some(x) = &to.x {
        point.x = x_of(x, &from.x)?;
    }
    if let Some(y) = to.y {
        point.y = Some(y);
    }
    Some(point)
}

/// Applies `set` to `chart`, read from `bytes` by `extractor`. Returns the history; pushes
/// a diagnostic for every edit not applied (`SNR-ENG-386`), or one for a correction file
/// made on another chart or by another extractor (`SNR-ENG-387`, nothing applied).
pub(crate) fn apply(
    chart: &mut ChartData,
    set: &CorrectionSet,
    bytes: &[u8],
    extractor: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<CorrectionRecord> {
    if let CorrectionSet::File(file) = set {
        let sha = Sha256Digest::of(bytes).to_string();
        if file.chart.sha256 != sha || file.chart.extractor != extractor {
            diagnostics.push(Diagnostic::new(
                &codes::CORRECTIONS_OTHER_CHART,
                format!(
                    "These corrections were made on {} read by `{}`, not on this chart; none was applied.",
                    file.chart.name.as_deref().map_or_else(|| "another chart".to_owned(), |n| format!("`{n}`")),
                    file.chart.extractor
                ),
            ));
            return Vec::new();
        }
    }
    let mut working = Working::of(chart);
    let mut history = Vec::new();
    for step in set.steps() {
        let mut record = CorrectionRecord {
            edit: step.edit.clone(),
            op: step.op,
            point: step
                .target
                .clone()
                .unwrap_or_else(|| format!("+{}", step.edit.trim_start_matches('#'))),
            applied: false,
            was: None,
            before: None,
        };
        record.applied = working.step(&step, &mut record).is_some();
        if !record.applied {
            diagnostics.push(Diagnostic::new(
                &codes::CORRECTION_NOT_APPLIED,
                format!(
                    "The {} edit {} on {} was not applied: the point is not there or no longer has the value the edit was made on.",
                    step.op, step.edit, record.point
                ),
            ));
            record.was = None;
            record.before = None;
        }
        history.push(record);
    }
    rebuild(chart, &working.series_order, working.slots);
    history
}

/// The chart's points by stable ID while edits apply.
struct Working {
    slots: BTreeMap<String, Slot>,
    series_order: Vec<String>,
    next: usize,
    tol: Tolerance,
}

const EDITED: Provenance = Provenance {
    class: ProvenanceClass::UserEdited,
    confidence: None,
};

impl Working {
    fn of(chart: &ChartData) -> Self {
        let mut slots: BTreeMap<String, Slot> = BTreeMap::new();
        let mut series_order: Vec<String> = Vec::new();
        for (s, series) in chart.series.iter().enumerate() {
            series_order.push(series.name.clone());
            for (i, point) in series.points.iter().enumerate() {
                let mut provenance = series.provenance;
                if let Some(confidence) = series.confidences.get(i) {
                    provenance.confidence = Some(*confidence);
                }
                if series.user_edited.binary_search(&i).is_ok() {
                    provenance = EDITED;
                }
                slots.insert(
                    format!("{}#{i}", series.name),
                    Slot {
                        series: series.name.clone(),
                        point: point.clone(),
                        provenance,
                        deleted: false,
                        order: (s, i),
                    },
                );
            }
        }
        Self {
            slots,
            series_order,
            next: 0,
            tol: tolerance(chart),
        }
    }

    fn note_series(&mut self, name: &str) {
        if !self.series_order.iter().any(|s| s == name) {
            self.series_order.push(name.to_owned());
        }
    }

    /// Applies one edit; `None` when it cannot apply. Records the point's value and
    /// provenance before it.
    fn step(&mut self, step: &Step<'_>, record: &mut CorrectionRecord) -> Option<()> {
        if step.op == "add" {
            let to = step.to?;
            let like = self
                .slots
                .values()
                .find(|s| s.series == step.series)
                .map(|s| s.point.x.clone());
            let x = match (&to.x, like) {
                (Some(x), Some(like)) => x_of(x, &like)?,
                (Some(AnchorValue::Number(n)), None) => XValue::Number(*n),
                (Some(AnchorValue::Text(t)), None) => XValue::Category(t.clone()),
                (None, _) => return None,
            };
            self.next += 1;
            self.note_series(step.series);
            self.slots.insert(
                record.point.clone(),
                Slot {
                    series: step.series.to_owned(),
                    point: Point { x, y: to.y },
                    provenance: EDITED,
                    deleted: false,
                    order: (usize::MAX, self.next),
                },
            );
            return Some(());
        }
        let tol = self.tol;
        let slot = self.slots.get(step.target.as_deref()?)?;
        if slot.deleted || !matches(&slot.point, step.was?, tol) {
            return None;
        }
        record.was = Some(slot.point.clone());
        record.before = Some(slot.provenance);
        if let ("retype", Some(to)) = (step.op, step.retype_to) {
            self.note_series(to);
        }
        self.next += 1;
        let next = self.next;
        let slot = self.slots.get_mut(step.target.as_deref()?)?;
        match step.op {
            "move" => slot.point = moved(&slot.point, step.to?)?,
            "delete" => slot.deleted = true,
            "retype" => {
                step.retype_to?.clone_into(&mut slot.series);
                slot.order = (usize::MAX, next);
            }
            _ => return None,
        }
        slot.provenance = EDITED;
        Some(())
    }
}

/// The chart from the slots: series in their first order, new ones after; each series'
/// original points in order, then added and retyped ones in edit order. A series left
/// with no points is dropped.
fn rebuild(chart: &mut ChartData, series_order: &[String], slots: BTreeMap<String, Slot>) {
    let template: BTreeMap<String, Series> = chart
        .series
        .drain(..)
        .map(|s| (s.name.clone(), s))
        .collect();
    let mut live: Vec<Slot> = slots.into_values().filter(|s| !s.deleted).collect();
    live.sort_by_key(|s| s.order);
    for name in series_order {
        let points: Vec<&Slot> = live.iter().filter(|s| &s.series == name).collect();
        if points.is_empty() {
            continue;
        }
        let mut series = template.get(name).cloned().unwrap_or_else(|| {
            let mut s = Series::exact(name.clone(), Vec::new());
            s.provenance = Provenance {
                class: ProvenanceClass::UserEdited,
                confidence: None,
            };
            s
        });
        let keeps_confidences = !series.confidences.is_empty();
        series.points = points.iter().map(|s| s.point.clone()).collect();
        series.user_edited = points
            .iter()
            .enumerate()
            .filter(|(_, s)| s.provenance.class == ProvenanceClass::UserEdited)
            .map(|(i, _)| i)
            .collect();
        if keeps_confidences {
            series.confidences = points
                .iter()
                .map(|s| s.provenance.confidence.unwrap_or(Confidence::CERTAIN))
                .collect();
        }
        chart.series.push(series);
    }
}

#[cfg(test)]
mod tests;
