// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Builds the navigation tree (08 §2, keyboard-and-navigation §4.1) from the Score, the
//! data and the annotations.
//!
//! - **Series.** One per data track, in Score order. Its points are the rows the track
//!   reads: the track's `filter` applies, and rows are ordered by the time field as the
//!   encoder orders them (the substrate's field, else the field with role `x`, else row
//!   order; numeric and temporal fields are sorted and rows without a time value are left
//!   out). Without a time field, a field with role `category` or `label` names the points
//!   (the bars of a bar chart), in row order. Each point keeps its `RowRef`, its value, its
//!   provenance and, if it is played, its onset in the Score. Rows with a missing value stay in the tree as points without
//!   a value, so navigation can say so.
//! - **Segments.** Semantic segments when trend-segment annotations partition a series;
//!   otherwise even segments of at least [`MIN_SEGMENT_POINTS`] points, at most
//!   [`MAX_SEGMENTS`] per series. Both numbers are provisional (OD-18), not research
//!   defaults: segments only group points, and moving along points crosses them.
//! - **Axes.** The x axis, then one value axis per distinct value field, with units and
//!   extents.
//! - **Insights.** One group per analyzer, in analyzer-ID order; each insight points at
//!   the first point it covers.
//! - **Filters.** One per series.

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, Int64Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use sonarithm_core::diagnostic::Diagnostics;
use sonarithm_core::engine::{Annotation, CompileContext, CompileError};
use sonarithm_core::ids::{AnalyzerId, RowRef, TrackId};
use sonarithm_core::nav::{
    AxisName, AxisNode, DatasetNode, FilterNode, InsightGroupNode, InsightNode, NavKind, NavNode,
    NavNodeId, NavTree, PointNode, Polarity, SegmentNode, SeriesNode, ValueRange, XSpan, XType,
    XValue,
};
use sonarithm_core::provenance::Provenance;
use sonarithm_core::score::{Score, ScoreEvent, ScoreTrack, TrackRole};
use sonarithm_core::units::{Cents, Seconds};
use sonarithm_data::{DataBundle, RowProvenance, Table};
use sonarithm_spec::data::Role;
use sonarithm_spec::spec::Spec;
use sonarithm_spec::track::{Filter, TrackSpec};

use crate::ENGINE;
use crate::columns::{self, Values};

/// The fewest points in an even segment. Provisional (OD-18): a design choice to settle in
/// co-design round 1 (B07), not a research default.
pub const MIN_SEGMENT_POINTS: usize = 8;

/// The most even segments in one series. Provisional (OD-18), as [`MIN_SEGMENT_POINTS`].
pub const MAX_SEGMENTS: usize = 12;

/// The analyzer whose annotations become semantic segments (ECHOMETRIC, C05).
pub const TREND_SEGMENTS: &str = "trend-segments";

/// A hidden column that carries each row's index through the track filter.
const ROW_COLUMN: &str = "__snr_audaxis_row";

/// One point of a series, before it becomes a node.
#[derive(Clone, Debug)]
pub(crate) struct Point {
    pub(crate) row: RowRef,
    pub(crate) position: f64,
    pub(crate) x: Option<XValue>,
    pub(crate) value: Option<f64>,
    pub(crate) provenance: Provenance,
    /// The onset in the Score, if the point is played.
    pub(crate) t: Option<Seconds>,
    /// How long it plays: the note's duration, or the time to the next breakpoint.
    pub(crate) dur: Option<Seconds>,
    /// The pitch it plays.
    pub(crate) pitch: Option<Cents>,
}

/// A field's name, spoken label and unit.
#[derive(Clone, Debug, Default)]
struct FieldInfo {
    name: Option<String>,
    label: Option<String>,
    unit: Option<String>,
    ucum: Option<String>,
}

/// One data track as a series.
#[derive(Clone, Debug)]
pub(crate) struct Series {
    pub(crate) track: TrackId,
    value_field: FieldInfo,
    x_field: FieldInfo,
    pub(crate) x_type: XType,
    pub(crate) points: Vec<Point>,
    /// The track's instrument.
    pub(crate) instrument: String,
    /// Whether the track plays glides (continuous marks).
    pub(crate) glides: bool,
}

impl Series {
    pub(crate) fn label(&self) -> String {
        self.value_field
            .label
            .clone()
            .or_else(|| self.value_field.name.clone())
            .unwrap_or_else(|| self.track.as_str().to_owned())
    }

    fn span(&self) -> Option<XSpan> {
        span_of(&self.points)
    }

    fn range(&self) -> Option<ValueRange> {
        ValueRange::of(self.points.iter().filter_map(|p| p.value))
    }

    /// The pitch range in Hz of the played points.
    fn pitch_hz(&self) -> Option<ValueRange> {
        ValueRange::of(
            self.points
                .iter()
                .filter_map(|p| p.pitch.map(|c| c.to_hz().0)),
        )
    }

    /// The polarity heard: the pitch of the largest value against that of the smallest.
    pub(crate) fn polarity(&self) -> Option<Polarity> {
        let (low, high) = self.extremes()?;
        let (low, high) = (low.pitch?, high.pitch?);
        if high.0 > low.0 {
            Some(Polarity::Positive)
        } else if high.0 < low.0 {
            Some(Polarity::Negative)
        } else {
            None
        }
    }

    /// The played points with the smallest and the largest value (the first of equals).
    pub(crate) fn extremes(&self) -> Option<(&Point, &Point)> {
        let played = || {
            self.points
                .iter()
                .filter(|p| p.value.is_some() && p.pitch.is_some())
        };
        let value = |p: &Point| p.value.unwrap_or(f64::NAN);
        let low = played().reduce(|a, b| if value(b) < value(a) { b } else { a })?;
        let high = played().reduce(|a, b| if value(b) > value(a) { b } else { a })?;
        Some((low, high))
    }
}

/// Builds the navigation tree of `score`.
///
/// # Errors
///
/// [`CompileError::Cancelled`] when the compile is cancelled, and
/// [`CompileError::Engine`] when a track filter cannot be applied.
pub fn build(
    spec: &Spec,
    score: &Score,
    data: &DataBundle,
    annotations: &[Annotation],
    cx: &mut CompileContext,
) -> Result<NavTree, CompileError> {
    let all = read(spec, score, data, cx)?;
    Ok(assemble(spec, &all, annotations))
}

/// Reads every data track of `score` as a series, in Score order.
pub(crate) fn read(
    spec: &Spec,
    score: &Score,
    data: &DataBundle,
    cx: &mut CompileContext,
) -> Result<Vec<Series>, CompileError> {
    let mut all = Vec::new();
    for track in score.tracks.iter().filter(|t| t.role == TrackRole::Data) {
        cx.check_cancelled()?;
        all.push(series(spec, track, data)?);
    }
    Ok(all)
}

/// Reads one data track's points.
fn series(spec: &Spec, track: &ScoreTrack, data: &DataBundle) -> Result<Series, CompileError> {
    let track_spec = spec.tracks.iter().find(|t| t.id == track.id.as_str());
    let mut series = Series {
        track: track.id.clone(),
        value_field: FieldInfo::default(),
        x_field: FieldInfo::default(),
        x_type: XType::RowOrder,
        points: Vec::new(),
        instrument: track.instrument.0.clone(),
        glides: track
            .events
            .iter()
            .any(|e| matches!(e, ScoreEvent::Glide(_))),
    };
    let Some(table) = table_of(track, data) else {
        return Ok(series);
    };
    if let Some(field) = track_spec.and_then(value_field) {
        series.value_field = field_info(spec, table, &field);
    }
    let mut rows = filtered_rows(table, track_spec.and_then(|t| t.filter.as_ref()))?;
    let x_field = time_field(spec, table).or_else(|| label_field(spec, table));
    let x_values = x_field.and_then(|field| {
        let values = table.column(&field).ok().and_then(|c| columns::read(&c));
        series.x_field = field_info(spec, table, &field);
        values
    });
    let positions: Vec<(f64, Option<XValue>)> = match &x_values {
        Some(Values::Numbers(xs, kind)) => {
            series.x_type = if *kind == XType::Number
                && series
                    .x_field
                    .name
                    .as_deref()
                    .is_some_and(|f| is_temporal(spec, table, f))
            {
                XType::Year
            } else {
                *kind
            };
            rows.retain(|&row| xs.get(row).is_some_and(|x| !x.is_nan()));
            rows.sort_by(|&a, &b| xs[a].total_cmp(&xs[b]));
            rows.iter()
                .map(|&row| (xs[row], Some(XValue::Number(xs[row]))))
                .collect()
        }
        Some(Values::Categories(labels)) => {
            series.x_type = XType::Category;
            rows.iter()
                .enumerate()
                .map(|(k, &row)| {
                    let label = labels.get(row).cloned().flatten();
                    (position(k), label.map(XValue::Category))
                })
                .collect()
        }
        None => (0..rows.len()).map(|k| (position(k), None)).collect(),
    };
    let values = series
        .value_field
        .name
        .as_deref()
        .and_then(|field| table.column(field).ok())
        .and_then(|column| columns::read(&column));
    let provenance = RowProvenance::read(table).ok();
    series.points = rows
        .iter()
        .zip(positions)
        .filter_map(|(&row, (position, x))| {
            let row_ref = table.row_ref(u64::try_from(row).ok()?)?;
            let value = match &values {
                Some(Values::Numbers(ys, _)) => ys.get(row).copied().filter(|y| y.is_finite()),
                _ => None,
            };
            Some(Point {
                row: row_ref,
                position,
                x,
                value,
                provenance: provenance
                    .as_ref()
                    .map_or(Provenance::EXACT, |p| p.get(row)),
                t: None,
                dur: None,
                pitch: None,
            })
        })
        .collect();
    assign_onsets(&mut series.points, track);
    Ok(series)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "point positions are row counts, far below 2^52"
)]
fn position(k: usize) -> f64 {
    k as f64
}

/// The table a track reads: the table of its first source row, else the primary table.
fn table_of<'d>(track: &ScoreTrack, data: &'d DataBundle) -> Option<&'d Table> {
    let first = track.events.iter().find_map(|event| match event {
        ScoreEvent::Note(note) => note.src.map(|src| src.table),
        ScoreEvent::Glide(glide) => glide.src.map(|src| src.table),
        _ => None,
    });
    first
        .and_then(|id| data.table(id))
        .or_else(|| data.primary())
}

/// The field whose values a track's points carry: the pitch field, else the first channel
/// with a field.
fn value_field(track: &TrackSpec) -> Option<String> {
    track
        .channels
        .pitch
        .as_ref()
        .and_then(|c| c.field.clone())
        .or_else(|| {
            track
                .channels
                .present()
                .into_iter()
                .find_map(|(_, channel)| channel.field.clone())
        })
}

/// A field's spoken label and unit: the spec's declarations win over the data's own
/// metadata (04 §3).
fn field_info(spec: &Spec, table: &Table, field: &str) -> FieldInfo {
    let declared = spec.data.fields.get(field);
    let meta = table.field_meta(field).ok();
    FieldInfo {
        name: Some(field.to_owned()),
        label: declared
            .and_then(|d| d.label.clone())
            .or_else(|| meta.as_ref().and_then(|m| m.label.clone())),
        unit: declared.and_then(|d| d.unit.clone()).or_else(|| {
            meta.as_ref()
                .and_then(|m| m.unit.as_ref().map(|u| u.text.clone()))
        }),
        ucum: meta.and_then(|m| m.unit.and_then(|u| u.ucum)),
    }
}

/// The time field, resolved as the encoder resolves it: the substrate's, else the field
/// declared with role `x`, else a field whose metadata has role `x`. Only fields the
/// table has count.
fn time_field(spec: &Spec, table: &Table) -> Option<String> {
    let exists = |field: &str| table.field_index(field).is_ok();
    if let Some(field) = spec
        .substrate
        .as_ref()
        .and_then(|s| s.time.as_ref())
        .and_then(|t| t.field.clone())
    {
        return exists(&field).then_some(field);
    }
    let declared = spec
        .data
        .fields
        .iter()
        .find(|(name, f)| f.role == Some(Role::X) && exists(name))
        .map(|(name, _)| name.clone());
    declared.or_else(|| {
        table
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .find(|name| {
                table
                    .field_meta(name)
                    .is_ok_and(|meta| meta.role == Some(sonarithm_data::Role::X))
            })
    })
}

/// Whether a field is declared or inferred temporal (03 §3.1), so plain numbers on it are
/// years.
fn is_temporal(spec: &Spec, table: &Table, field: &str) -> bool {
    match spec.data.fields.get(field).and_then(|f| f.field_type) {
        Some(declared) => declared == sonarithm_spec::data::FieldType::Temporal,
        None => table
            .field_meta(field)
            .is_ok_and(|m| m.ftype == Some(sonarithm_data::FieldType::Temporal)),
    }
}

/// A field that names rows when there is no time field: the first declared with role
/// `category` or `label`, else the first whose metadata has one of those roles, if its
/// values are categories.
fn label_field(spec: &Spec, table: &Table) -> Option<String> {
    let categorical = |field: &str| {
        table
            .column(field)
            .ok()
            .and_then(|c| columns::read(&c))
            .is_some_and(|v| matches!(v, Values::Categories(_)))
    };
    let declared = spec
        .data
        .fields
        .iter()
        .find(|(name, f)| matches!(f.role, Some(Role::Category | Role::Label)) && categorical(name))
        .map(|(name, _)| name.clone());
    declared.or_else(|| {
        table
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .find(|name| {
                table.field_meta(name).is_ok_and(|meta| {
                    matches!(
                        meta.role,
                        Some(sonarithm_data::Role::Category | sonarithm_data::Role::Label)
                    )
                }) && categorical(name)
            })
    })
}

/// The rows a track reads, in table order: all rows, or those its filter keeps.
fn filtered_rows(table: &Table, filter: Option<&Filter>) -> Result<Vec<usize>, CompileError> {
    let count = usize::try_from(table.num_rows()).unwrap_or(usize::MAX);
    let Some(filter) = filter else {
        return Ok((0..count).collect());
    };
    let failed = |message: String| CompileError::Engine {
        engine: ENGINE,
        message,
    };
    let batch = table.to_batch().map_err(|e| failed(e.to_string()))?;
    let schema = batch.schema();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
    fields.push(Arc::new(Field::new(ROW_COLUMN, DataType::Int64, false)));
    let tagged_schema = Arc::new(Schema::new(fields).with_metadata(schema.metadata().clone()));
    let numbers = Int64Array::from_iter_values(0..i64::try_from(count).unwrap_or(i64::MAX));
    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();
    columns.push(Arc::new(numbers));
    let tagged = RecordBatch::try_new(Arc::clone(&tagged_schema), columns)
        .map_err(|e| failed(e.to_string()))?;
    let tagged = table
        .next_generation(tagged_schema, vec![tagged])
        .map_err(|e| failed(e.to_string()))?;
    let transform = sonarithm_data::Transform::Filter {
        expr: filter.expr.clone(),
    };
    let kept =
        sonarithm_data::apply(&tagged, &transform, &mut Diagnostics::new()).map_err(|e| {
            failed(format!(
                "the track filter `{}` cannot be applied: {e}",
                filter.expr
            ))
        })?;
    let rows = kept.column(ROW_COLUMN).map_err(|e| failed(e.to_string()))?;
    Ok(rows
        .as_primitive_opt::<Int64Type>()
        .map(|rows| {
            rows.values()
                .iter()
                .filter_map(|&row| usize::try_from(row).ok())
                .collect()
        })
        .unwrap_or_default())
}

/// Copies each point's onset, length and pitch from the track's events: a note's by its
/// row; a glide's breakpoints, in order, to the points with a value inside its rows, when
/// the counts agree (the last breakpoint holds the final value and has no row).
fn assign_onsets(points: &mut [Point], track: &ScoreTrack) {
    let mut by_row: BTreeMap<u64, (Seconds, Seconds, Cents)> = BTreeMap::new();
    for event in &track.events {
        match event {
            ScoreEvent::Note(note) => {
                if let Some(src) = note.src {
                    by_row
                        .entry(src.row)
                        .or_insert((note.t, note.dur, note.pitch));
                }
            }
            ScoreEvent::Glide(glide) => {
                let Some(src) = glide.src else { continue };
                let members: Vec<usize> = points
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| {
                        p.t.is_none()
                            && p.value.is_some()
                            && p.row.table == src.table
                            && (src.start..src.end).contains(&p.row.row)
                    })
                    .map(|(i, _)| i)
                    .collect();
                let breakpoints = glide.curve.len().saturating_sub(1);
                if members.len() == breakpoints {
                    for (k, (&i, breakpoint)) in members.iter().zip(&glide.curve).enumerate() {
                        let next = glide.curve.get(k + 1).map_or(glide.t1, |b| b.t);
                        points[i].t = Some(breakpoint.t);
                        points[i].dur = Some(Seconds(next.0 - breakpoint.t.0));
                        points[i].pitch = Some(breakpoint.pitch);
                    }
                }
            }
            _ => {}
        }
    }
    for point in points.iter_mut() {
        if point.t.is_none()
            && let Some(&(t, dur, pitch)) = by_row.get(&point.row.row)
        {
            point.t = Some(t);
            point.dur = Some(dur);
            point.pitch = Some(pitch);
        }
    }
}

fn span_of(points: &[Point]) -> Option<XSpan> {
    let first = points.first()?;
    let last = points.last()?;
    Some(XSpan {
        start: first.position,
        end: last.position,
    })
}

/// The segments of a series as (semantic, point indices).
fn segments(series: &Series, annotations: &[Annotation]) -> Vec<(bool, Range<usize>)> {
    let n = series.points.len();
    if n == 0 {
        return Vec::new();
    }
    if let Some(semantic) = semantic_segments(series, annotations) {
        return semantic.into_iter().map(|r| (true, r)).collect();
    }
    let count = (n / MIN_SEGMENT_POINTS).clamp(1, MAX_SEGMENTS);
    let (base, extra) = (n / count, n % count);
    let mut start = 0;
    (0..count)
        .map(|k| {
            let len = base + usize::from(k < extra);
            let range = start..start + len;
            start += len;
            (false, range)
        })
        .collect()
}

/// Trend-segment annotations as runs of points, if they partition the series exactly.
fn semantic_segments(series: &Series, annotations: &[Annotation]) -> Option<Vec<Range<usize>>> {
    let first_row = series.points.first()?.row;
    let mut runs: Vec<Range<usize>> = annotations
        .iter()
        .filter(|a| a.source.as_str() == TREND_SEGMENTS && a.range.table == first_row.table)
        .filter_map(|a| {
            let members: Vec<usize> = series
                .points
                .iter()
                .enumerate()
                .filter(|(_, p)| (a.range.start..a.range.end).contains(&p.row.row))
                .map(|(i, _)| i)
                .collect();
            let (&lo, &hi) = (members.first()?, members.last()?);
            (hi - lo + 1 == members.len()).then_some(lo..hi + 1)
        })
        .collect();
    if runs.is_empty() {
        return None;
    }
    runs.sort_by_key(|r| r.start);
    let mut next = 0;
    for run in &runs {
        if run.start != next {
            return None;
        }
        next = run.end;
    }
    (next == series.points.len()).then_some(runs)
}

/// Puts the series, axes, insights and filters together.
pub(crate) fn assemble(spec: &Spec, all: &[Series], annotations: &[Annotation]) -> NavTree {
    let mut tree = NavTree::new(DatasetNode {
        title: spec.name.clone(),
        series: u32::try_from(all.len()).unwrap_or(u32::MAX),
        points: all.iter().map(|s| s.points.len() as u64).sum(),
        gap_earcons: spec.data.missing == Some(sonarithm_spec::data::Missing::Earcon),
    });
    let root = tree.root();
    let mut placed: Vec<(NavNodeId, Vec<NavNodeId>)> = Vec::new();
    for series in all {
        let node = NavNode::new(NavKind::Series(SeriesNode {
            track: series.track.clone(),
            field: series.value_field.name.clone(),
            unit: series.value_field.unit.clone(),
            ucum: series.value_field.ucum.clone(),
            points: series.points.len() as u64,
            range: series.range(),
            pitch_hz: series.pitch_hz(),
            polarity: series.polarity(),
        }))
        .with_label(series.label())
        .with_span(series.span());
        let series_id = tree.push(root, node);
        let mut point_ids = Vec::with_capacity(series.points.len());
        for (semantic, run) in segments(series, annotations) {
            let members = &series.points[run];
            let segment = NavNode::new(NavKind::Segment(SegmentNode {
                semantic,
                points: members.len() as u64,
                range: ValueRange::of(members.iter().filter_map(|p| p.value)),
            }))
            .with_span(span_of(members));
            let segment_id = tree.push(series_id, segment);
            for point in members {
                let index = point_ids.len() as u64;
                let node = NavNode::new(NavKind::Point(PointNode {
                    row: point.row,
                    index,
                    t: point.t,
                    x: point.x.clone(),
                    value: point.value,
                    provenance: point.provenance,
                }))
                .with_span(Some(XSpan::at(point.position)));
                point_ids.push(tree.push(segment_id, node));
            }
        }
        placed.push((series_id, point_ids));
    }
    add_axes(&mut tree, all);
    add_insights(&mut tree, all, &placed, annotations);
    let filters = tree.push(root, NavNode::new(NavKind::Filters));
    for (series, (series_id, _)) in all.iter().zip(&placed) {
        let node = NavNode::new(NavKind::Filter(FilterNode {
            series: *series_id,
            track: series.track.clone(),
        }))
        .with_label(series.label());
        tree.push(filters, node);
    }
    tree
}

/// The x axis, then one value axis per distinct value field.
fn add_axes(tree: &mut NavTree, all: &[Series]) {
    let axes = tree.push(tree.root(), NavNode::new(NavKind::Axes));
    let x_source = all
        .iter()
        .find(|s| s.x_field.name.is_some())
        .or(all.first());
    let x_type = x_source.map_or(XType::RowOrder, |s| s.x_type);
    let numeric_x = matches!(
        x_type,
        XType::Number | XType::Year | XType::Date | XType::Timestamp
    );
    let x_range = numeric_x
        .then(|| ValueRange::of(all.iter().flat_map(|s| s.points.iter().map(|p| p.position))))
        .flatten();
    let x_field = x_source.map(|s| s.x_field.clone()).unwrap_or_default();
    let mut x_axis = NavNode::new(NavKind::Axis(AxisNode {
        axis: AxisName::X,
        field: x_field.name.clone(),
        unit: x_field.unit.clone(),
        ucum: x_field.ucum.clone(),
        values: x_type,
        range: x_range,
    }));
    if let Some(label) = x_field.label.or(x_field.name) {
        x_axis = x_axis.with_label(label);
    }
    tree.push(axes, x_axis);

    let mut fields: Vec<Option<&str>> = Vec::new();
    for series in all {
        let field = series.value_field.name.as_deref();
        if !fields.contains(&field) {
            fields.push(field);
        }
    }
    if fields.is_empty() {
        fields.push(None);
    }
    for field in fields {
        let members: Vec<&Series> = all
            .iter()
            .filter(|s| s.value_field.name.as_deref() == field)
            .collect();
        let info = members
            .first()
            .map(|s| s.value_field.clone())
            .unwrap_or_default();
        let range = ValueRange::of(
            members
                .iter()
                .flat_map(|s| s.points.iter().filter_map(|p| p.value)),
        );
        let mut value_axis = NavNode::new(NavKind::Axis(AxisNode {
            axis: AxisName::Y,
            field: info.name.clone(),
            unit: info.unit.clone(),
            ucum: info.ucum.clone(),
            values: XType::Number,
            range,
        }));
        if let Some(label) = info.label.or(info.name) {
            value_axis = value_axis.with_label(label);
        }
        tree.push(axes, value_axis);
    }
}

/// One group per analyzer; each insight points at the first point it covers.
fn add_insights(
    tree: &mut NavTree,
    all: &[Series],
    placed: &[(NavNodeId, Vec<NavNodeId>)],
    annotations: &[Annotation],
) {
    let insights = tree.push(tree.root(), NavNode::new(NavKind::Insights));
    let mut groups: BTreeMap<&AnalyzerId, Vec<&Annotation>> = BTreeMap::new();
    for annotation in annotations {
        groups
            .entry(&annotation.source)
            .or_default()
            .push(annotation);
    }
    for (analyzer, members) in groups {
        let group = NavNode::new(NavKind::InsightGroup(InsightGroupNode {
            analyzer: analyzer.clone(),
        }))
        .with_label(analyzer.as_str());
        let group_id = tree.push(insights, group);
        let mut located: Vec<(Option<(NavNodeId, f64)>, &Annotation)> = members
            .into_iter()
            .map(|a| (target(all, placed, a), a))
            .collect();
        located.sort_by(|(a, x), (b, y)| {
            let key = |t: &Option<(NavNodeId, f64)>| t.map_or(f64::INFINITY, |(_, p)| p);
            key(a)
                .total_cmp(&key(b))
                .then(x.range.start.cmp(&y.range.start))
        });
        for (found, annotation) in located {
            let node = NavNode::new(NavKind::Insight(InsightNode {
                analyzer: annotation.source.clone(),
                rows: annotation.range,
                value: annotation.value,
                confidence: annotation.confidence,
                target: found.map(|(id, _)| id),
            }))
            .with_label(annotation.label.clone())
            .with_span(found.map(|(_, p)| XSpan::at(p)));
            tree.push(group_id, node);
        }
    }
}

/// The first point, in series order then x order, whose row the annotation covers.
fn target(
    all: &[Series],
    placed: &[(NavNodeId, Vec<NavNodeId>)],
    annotation: &Annotation,
) -> Option<(NavNodeId, f64)> {
    let range = annotation.range;
    all.iter().zip(placed).find_map(|(series, (_, ids))| {
        series
            .points
            .iter()
            .zip(ids)
            .find(|(p, _)| {
                p.row.table == range.table
                    && p.row.generation == range.generation
                    && (range.start..range.end).contains(&p.row.row)
            })
            .map(|(p, &id)| (id, p.position))
    })
}
