// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `smooth`, `normalize` and `resample` (03 §6).
//!
//! Provenance (03 §4): a smoothed value combines the provenance of the rows it depends on
//! (the window for `sma`, every earlier row for `ema`); `normalize` keeps each row's
//! provenance, column statistics being parameters; `resample` gives `mean` intervals
//! `derived` provenance combined with their rows, keeps the chosen row's for `last`, and
//! combines the two neighbours of an interpolated point.

use std::collections::VecDeque;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{Date32Type, Float64Type, Int64Type};
use arrow_array::{Array, ArrayRef, Date32Array, Float64Array, Int64Array, UInt32Array};
use arrow_schema::{DataType, Field, TimeUnit};
use serde::{Deserialize, Serialize};
use sonarithm_core::{Confidence, Provenance, ProvenanceClass, Seconds};

use super::{Working, computed_field};
use crate::error::DataError;
use crate::infer::semantic::numbers;
use crate::meta::FieldMeta;
use crate::provenance::{RowProvenance, derived};

/// The most grid points `resample` may produce.
const MAX_GRID: u64 = 10_000_000;

/// How `smooth` averages (RFC-0001 §4, SDR-21).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmoothMethod {
    /// Exponential moving average with weight `alpha` in (0, 1] for the newest value.
    Ema,
    /// Simple moving average of the last `window` rows.
    Sma,
}

/// How `resample` fills each grid point (RFC-0001 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResampleMethod {
    /// Means of the numbers in each interval; other fields take the interval's last value.
    Mean,
    /// The last row of each interval.
    Last,
    /// Linear interpolation of numbers between the neighbouring rows; other fields take
    /// the previous row's value.
    Interpolate,
}

/// How `normalize` rescales (RFC-0001 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormalizeMethod {
    /// (x − mean) / standard deviation (population).
    ZScore,
    /// (x − min) / (max − min), into [0, 1].
    MinMax,
    /// Rank from 1, ties sharing their average rank.
    Rank,
}

fn number_column(
    working: &Working,
    op: &'static str,
    field: &str,
) -> Result<(usize, Float64Array), DataError> {
    let index = working.index(field)?;
    let column = working
        .columns
        .get(index)
        .cloned()
        .unwrap_or_else(|| arrow_array::new_empty_array(&DataType::Float64));
    let values = numbers(&column)
        .filter(|_| column.data_type().is_numeric())
        .ok_or_else(|| DataError::FieldTypeMismatch {
            op,
            field: field.to_owned(),
            expected: "numbers",
            found: column.data_type().to_string(),
        })?;
    Ok((index, values))
}

/// Puts a computed number column, keeping the source field's concept (and unit when
/// `keep_unit`).
fn put_numbers(
    working: &mut Working,
    source: usize,
    name: &str,
    values: Vec<Option<f64>>,
    keep_unit: bool,
) {
    let column: ArrayRef = Arc::new(Float64Array::from(values));
    let field = computed_field(name, &column);
    let (source_meta, _) = working
        .fields
        .get(source)
        .map(FieldMeta::read)
        .unwrap_or_default();
    let (mut meta, _) = FieldMeta::read(&field);
    meta.concept = source_meta.concept;
    if keep_unit {
        meta.unit = source_meta.unit;
    }
    working.put(meta.apply(field), column);
}

/// Moving averages (SDR-21: volatility reduction).
pub(super) fn smooth(
    working: &mut Working,
    field: &str,
    method: SmoothMethod,
    alpha: Option<f64>,
    window: Option<u32>,
    output: Option<&str>,
) -> Result<String, DataError> {
    let invalid = |reason: &str| DataError::InvalidTransform {
        op: "smooth",
        reason: reason.to_owned(),
    };
    let (index, values) = number_column(working, "smooth", field)?;
    let mut out = Vec::with_capacity(working.rows);
    let mut provenance = Vec::with_capacity(working.rows);
    let describe = match method {
        SmoothMethod::Ema => {
            let alpha = alpha
                .filter(|a| *a > 0.0 && *a <= 1.0)
                .ok_or_else(|| invalid("`alpha` must be in (0, 1]"))?;
            let mut state: Option<f64> = None;
            let mut inputs = Provenance::EXACT;
            for (row, value) in values.iter().enumerate() {
                let own = working.provenance.get(row);
                if let Some(v) = value {
                    inputs = inputs.combine(own);
                    let next = state.map_or(v, |s| s + alpha * (v - s));
                    state = Some(next);
                    out.push(Some(next));
                    provenance.push(inputs);
                } else {
                    out.push(None);
                    provenance.push(own);
                }
            }
            format!("`{field}`, ema alpha {alpha}")
        }
        SmoothMethod::Sma => {
            let window = window
                .filter(|w| *w >= 1)
                .ok_or_else(|| invalid("`window` must be at least 1"))?
                as usize;
            let mut sums = SlidingSum::default();
            let mut combined = SlidingProvenance::default();
            for (row, value) in values.iter().enumerate() {
                if let Some(v) = value {
                    sums.add(row, v);
                    combined.add(row, working.provenance.get(row));
                }
                sums.evict(row + 1 - window.min(row + 1));
                combined.evict(row + 1 - window.min(row + 1));
                if value.is_some() {
                    out.push(sums.mean());
                    provenance.push(combined.current());
                } else {
                    out.push(None);
                    provenance.push(working.provenance.get(row));
                }
            }
            format!("`{field}`, sma window {window}")
        }
    };
    put_numbers(working, index, output.unwrap_or(field), out, true);
    working.provenance = RowProvenance::from_rows(provenance);
    Ok(describe)
}

/// A windowed sum with Neumaier compensation, deterministic in row order.
#[derive(Default)]
struct SlidingSum {
    items: VecDeque<(usize, f64)>,
    sum: f64,
    compensation: f64,
}

impl SlidingSum {
    fn add(&mut self, row: usize, v: f64) {
        self.items.push_back((row, v));
        self.accumulate(v);
    }

    /// Drops values of rows before `first`.
    fn evict(&mut self, first: usize) {
        while let Some(&(row, v)) = self.items.front() {
            if row >= first {
                break;
            }
            self.items.pop_front();
            self.accumulate(-v);
        }
    }

    fn accumulate(&mut self, v: f64) {
        let t = self.sum + v;
        if self.sum.abs() >= v.abs() {
            self.compensation += (self.sum - t) + v;
        } else {
            self.compensation += (v - t) + self.sum;
        }
        self.sum = t;
    }

    fn mean(&self) -> Option<f64> {
        #[expect(clippy::cast_precision_loss, reason = "window sizes are below 2^53")]
        let n = self.items.len() as f64;
        (!self.items.is_empty()).then(|| (self.sum + self.compensation) / n)
    }
}

/// The weakest class and lowest confidence over a sliding window, with monotonic queues.
#[derive(Default)]
struct SlidingProvenance {
    classes: VecDeque<(usize, ProvenanceClass)>,
    confidences: VecDeque<(usize, Confidence)>,
}

impl SlidingProvenance {
    fn add(&mut self, row: usize, p: Provenance) {
        while self.classes.back().is_some_and(|(_, c)| *c <= p.class) {
            self.classes.pop_back();
        }
        self.classes.push_back((row, p.class));
        if let Some(c) = p.confidence {
            while self
                .confidences
                .back()
                .is_some_and(|(_, d)| d.get() >= c.get())
            {
                self.confidences.pop_back();
            }
            self.confidences.push_back((row, c));
        }
    }

    fn evict(&mut self, first: usize) {
        while self.classes.front().is_some_and(|(row, _)| *row < first) {
            self.classes.pop_front();
        }
        while self
            .confidences
            .front()
            .is_some_and(|(row, _)| *row < first)
        {
            self.confidences.pop_front();
        }
    }

    fn current(&self) -> Provenance {
        Provenance {
            class: self.classes.front().map(|(_, c)| *c).unwrap_or_default(),
            confidence: self.confidences.front().map(|(_, c)| *c),
        }
    }
}

/// Rescales a number field; row provenance is unchanged.
pub(super) fn normalize(
    working: &mut Working,
    field: &str,
    method: NormalizeMethod,
    output: Option<&str>,
) -> Result<String, DataError> {
    let (index, values) = number_column(working, "normalize", field)?;
    let present: Vec<f64> = values.iter().flatten().filter(|v| v.is_finite()).collect();
    #[expect(clippy::cast_precision_loss, reason = "row counts are below 2^53")]
    let n = present.len() as f64;
    let out: Vec<Option<f64>> = match method {
        NormalizeMethod::ZScore => {
            let mean = compensated(&present) / n;
            let squares: Vec<f64> = present.iter().map(|v| (v - mean) * (v - mean)).collect();
            let sd = (compensated(&squares) / n).sqrt();
            values
                .iter()
                .map(|v| {
                    v.filter(|v| v.is_finite())
                        .map(|v| if sd > 0.0 { (v - mean) / sd } else { 0.0 })
                })
                .collect()
        }
        NormalizeMethod::MinMax => {
            let lo = present.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = present.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let range = hi - lo;
            values
                .iter()
                .map(|v| {
                    v.filter(|v| v.is_finite())
                        .map(|v| if range > 0.0 { (v - lo) / range } else { 0.0 })
                })
                .collect()
        }
        NormalizeMethod::Rank => ranks(&values),
    };
    put_numbers(working, index, output.unwrap_or(field), out, false);
    Ok(format!(
        "`{field}`, {}",
        match method {
            NormalizeMethod::ZScore => "z-score",
            NormalizeMethod::MinMax => "min-max",
            NormalizeMethod::Rank => "rank",
        }
    ))
}

fn compensated(values: &[f64]) -> f64 {
    let mut sum = SlidingSum::default();
    for &v in values {
        sum.accumulate(v);
    }
    sum.sum + sum.compensation
}

/// Ranks from 1 in ascending order (`total_cmp`), ties sharing their average rank.
fn ranks(values: &Float64Array) -> Vec<Option<f64>> {
    let mut order: Vec<usize> = (0..values.len()).filter(|&i| values.is_valid(i)).collect();
    order.sort_by(|&a, &b| values.value(a).total_cmp(&values.value(b)));
    let mut out = vec![None; values.len()];
    let mut start = 0;
    while start < order.len() {
        let value = order.get(start).map(|&i| values.value(i));
        let end = order[start..]
            .iter()
            .position(|&i| Some(values.value(i)) != value)
            .map_or(order.len(), |p| start + p);
        #[expect(clippy::cast_precision_loss, reason = "row counts are below 2^53")]
        let average = (start + end + 1) as f64 / 2.0;
        for &i in order.get(start..end).unwrap_or_default() {
            if let Some(slot) = out.get_mut(i) {
                *slot = Some(average);
            }
        }
        start = end;
    }
    out
}

/// The time axis of `resample`: integer ticks (dates, timestamps, integers) with an integer
/// step, or floats. Integer axes use exact integer arithmetic, so nanosecond timestamps
/// land on the right grid points.
enum Axis {
    Ticks(Vec<Option<i64>>, i64),
    Floats(Vec<Option<f64>>, f64),
}

impl Axis {
    fn new(column: &ArrayRef, field: &str, every: Seconds) -> Result<Self, DataError> {
        let invalid = |reason: String| DataError::InvalidTransform {
            op: "resample",
            reason,
        };
        if !(every.0.is_finite() && every.0 > 0.0) {
            return Err(invalid("`every` must be a positive duration".to_owned()));
        }
        let ticks = |per_second: f64| -> Result<i64, DataError> {
            let step = every.0 * per_second;
            if step.fract() != 0.0 || !(1.0..9.2e18).contains(&step) {
                return Err(invalid(format!(
                    "`every` must be a whole number of the time unit of `{field}`"
                )));
            }
            #[expect(
                clippy::cast_possible_truncation,
                reason = "integral and in range, checked above"
            )]
            Ok(step as i64)
        };
        Ok(match column.data_type() {
            DataType::Date32 => Self::Ticks(
                column
                    .as_primitive::<Date32Type>()
                    .iter()
                    .map(|v| v.map(i64::from))
                    .collect(),
                ticks(1.0 / 86_400.0)?,
            ),
            DataType::Timestamp(unit, _) => {
                let per_second = match unit {
                    TimeUnit::Second => 1.0,
                    TimeUnit::Millisecond => 1e3,
                    TimeUnit::Microsecond => 1e6,
                    TimeUnit::Nanosecond => 1e9,
                };
                Self::Ticks(super::raw_i64(column).iter().collect(), ticks(per_second)?)
            }
            DataType::Int64 => Self::Ticks(
                column.as_primitive::<Int64Type>().iter().collect(),
                ticks(1.0)?,
            ),
            DataType::Float64 => Self::Floats(
                column.as_primitive::<Float64Type>().iter().collect(),
                every.0,
            ),
            other => {
                return Err(DataError::FieldTypeMismatch {
                    op: "resample",
                    field: field.to_owned(),
                    expected: "dates, timestamps or numbers",
                    found: other.to_string(),
                });
            }
        })
    }

    fn tick(&self, row: usize) -> i64 {
        match self {
            Self::Ticks(t, _) => t.get(row).copied().flatten().unwrap_or(0),
            Self::Floats(..) => 0,
        }
    }

    fn float(&self, row: usize) -> f64 {
        match self {
            Self::Floats(t, _) => t.get(row).copied().flatten().unwrap_or(0.0),
            Self::Ticks(..) => 0.0,
        }
    }

    /// Rows with a time, sorted by time (stable).
    fn sorted_rows(&self) -> Vec<usize> {
        let mut rows: Vec<usize> = match self {
            Self::Ticks(t, _) => (0..t.len())
                .filter(|&i| t.get(i).copied().flatten().is_some())
                .collect(),
            Self::Floats(t, _) => (0..t.len())
                .filter(|&i| t.get(i).copied().flatten().is_some_and(f64::is_finite))
                .collect(),
        };
        match self {
            Self::Ticks(..) => rows.sort_by_key(|&r| self.tick(r)),
            Self::Floats(..) => rows.sort_by(|&a, &b| self.float(a).total_cmp(&self.float(b))),
        }
        rows
    }

    /// The grid step at or before the row's time.
    fn floor_step(&self, row: usize) -> i64 {
        match self {
            Self::Ticks(_, step) => self.tick(row).div_euclid(*step),
            #[expect(
                clippy::cast_possible_truncation,
                reason = "bounded by the MAX_GRID check"
            )]
            Self::Floats(_, step) => (self.float(row) / step).floor() as i64,
        }
    }

    /// The grid step at or after the row's time.
    fn ceil_step(&self, row: usize) -> i64 {
        match self {
            Self::Ticks(_, step) => -(-self.tick(row)).div_euclid(*step),
            #[expect(
                clippy::cast_possible_truncation,
                reason = "bounded by the MAX_GRID check"
            )]
            Self::Floats(_, step) => (self.float(row) / step).ceil() as i64,
        }
    }

    /// How the row's time compares with grid step `k`.
    fn compare(&self, row: usize, k: i64) -> std::cmp::Ordering {
        match self {
            Self::Ticks(_, step) => {
                i128::from(self.tick(row)).cmp(&(i128::from(k) * i128::from(*step)))
            }
            Self::Floats(_, step) => self.float(row).total_cmp(&(int_f64(k) * step)),
        }
    }

    /// The fraction of the way from row `a` to row `b` at grid step `k`.
    fn fraction(&self, a: usize, b: usize, k: i64) -> f64 {
        match self {
            Self::Ticks(_, step) => {
                let g = i128::from(k) * i128::from(*step);
                let (ta, tb) = (i128::from(self.tick(a)), i128::from(self.tick(b)));
                i128_f64(g - ta) / i128_f64(tb - ta)
            }
            Self::Floats(_, step) => {
                let (ta, tb) = (self.float(a), self.float(b));
                (int_f64(k) * step - ta) / (tb - ta)
            }
        }
    }

    /// Grid step `k` as a float time (float axes only).
    fn grid(&self, k: i64) -> f64 {
        match self {
            Self::Floats(_, step) => int_f64(k) * step,
            Self::Ticks(..) => 0.0,
        }
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "grid steps and tick differences are far below 2^53 in practice"
)]
fn int_f64(k: i64) -> f64 {
    k as f64
}

#[expect(
    clippy::cast_precision_loss,
    reason = "tick differences are far below 2^53 in practice"
)]
fn i128_f64(k: i128) -> f64 {
    k as f64
}

/// Regularizes a table on a time grid of step `every`.
pub(super) fn resample(
    working: &mut Working,
    field: &str,
    every: Seconds,
    method: ResampleMethod,
) -> Result<String, DataError> {
    let time_index = working.index(field)?;
    let time = working
        .columns
        .get(time_index)
        .cloned()
        .unwrap_or_else(|| arrow_array::new_empty_array(&DataType::Int64));
    let axis = Axis::new(&time, field, every)?;
    let rows = axis.sorted_rows();
    let (Some(&first), Some(&last)) = (rows.first(), rows.last()) else {
        take_all(working, &[])?;
        return Ok(format!("on `{field}`, no times"));
    };
    let start = if method == ResampleMethod::Interpolate {
        axis.ceil_step(first)
    } else {
        axis.floor_step(first)
    };
    let end = axis.floor_step(last);
    let count = u64::try_from(i128::from(end) - i128::from(start) + 1).unwrap_or(u64::MAX);
    if count > MAX_GRID {
        return Err(DataError::InvalidTransform {
            op: "resample",
            reason: format!("`every` makes more than {MAX_GRID} grid points"),
        });
    }
    let grid: Vec<i64> = (start..=end).collect();
    let plan = match method {
        ResampleMethod::Mean | ResampleMethod::Last => intervals(&axis, &rows, start, grid.len()),
        ResampleMethod::Interpolate => Vec::new(),
    };
    let mut columns = Vec::with_capacity(working.columns.len());
    let mut fields = Vec::with_capacity(working.fields.len());
    for (index, (field_def, column)) in working.fields.iter().zip(&working.columns).enumerate() {
        if index == time_index {
            columns.push(grid_column(&time, &axis, &grid));
            fields.push(field_def.clone());
            continue;
        }
        let numeric = column.data_type().is_numeric();
        let built: ArrayRef = match method {
            ResampleMethod::Mean if numeric => mean_column(column, &plan),
            ResampleMethod::Mean | ResampleMethod::Last => {
                let any_row = method == ResampleMethod::Last;
                let picks: UInt32Array = plan
                    .iter()
                    .map(|rows| last_present(column, rows, any_row))
                    .collect();
                take(column, &picks, &working.table_name)?
            }
            ResampleMethod::Interpolate => {
                interpolate_column(column, &axis, &rows, &grid, numeric, &working.table_name)?
            }
        };
        let field = if built.data_type() == column.data_type() {
            field_def.clone()
        } else {
            Field::new(field_def.name(), built.data_type().clone(), true)
                .with_metadata(field_def.metadata().clone())
        };
        fields.push(field);
        columns.push(built);
    }
    let provenance: Vec<Provenance> = match method {
        ResampleMethod::Mean => plan
            .iter()
            .map(|rows| derived(working.provenance.combine(rows.iter().copied())))
            .collect(),
        ResampleMethod::Last => plan
            .iter()
            .map(|rows| {
                rows.last().map_or_else(
                    || derived(Provenance::EXACT),
                    |&r| working.provenance.get(r),
                )
            })
            .collect(),
        ResampleMethod::Interpolate => grid
            .iter()
            .map(|&k| match neighbours(&axis, &rows, k) {
                Neighbours::Exact(r) => working.provenance.get(r),
                Neighbours::Between(a, b) => derived(working.provenance.combine([a, b])),
                Neighbours::None => derived(Provenance::EXACT),
            })
            .collect(),
    };
    working.fields = fields;
    working.columns = columns;
    working.rows = grid.len();
    working.provenance = RowProvenance::from_rows(provenance);
    let how = match method {
        ResampleMethod::Mean => "mean",
        ResampleMethod::Last => "last",
        ResampleMethod::Interpolate => "interpolate",
    };
    Ok(format!("on `{field}` every {} s, {how}", every.0))
}

/// The rows of each grid interval, in time order.
fn intervals(axis: &Axis, rows: &[usize], start: i64, count: usize) -> Vec<Vec<usize>> {
    let mut plan = vec![Vec::new(); count];
    for &row in rows {
        let k = axis.floor_step(row).saturating_sub(start);
        if let Some(slot) = usize::try_from(k).ok().and_then(|k| plan.get_mut(k)) {
            slot.push(row);
        }
    }
    plan
}

/// The grid times in the time column's own type.
fn grid_column(time: &ArrayRef, axis: &Axis, grid: &[i64]) -> ArrayRef {
    match axis {
        Axis::Floats(..) => Arc::new(
            grid.iter()
                .map(|&k| Some(axis.grid(k)))
                .collect::<Float64Array>(),
        ),
        Axis::Ticks(_, step) => {
            let ticks: Vec<i64> = grid.iter().map(|&k| k.saturating_mul(*step)).collect();
            match time.data_type() {
                DataType::Date32 => Arc::new(
                    ticks
                        .iter()
                        .map(|&t| i32::try_from(t).ok())
                        .collect::<Date32Array>(),
                ),
                DataType::Int64 => Arc::new(Int64Array::from(ticks)),
                other => {
                    super::reinterpret(&(Arc::new(Int64Array::from(ticks)) as ArrayRef), other)
                }
            }
        }
    }
}

fn mean_column(column: &ArrayRef, plan: &[Vec<usize>]) -> ArrayRef {
    let values =
        numbers(column).unwrap_or_else(|| Float64Array::from(vec![None::<f64>; column.len()]));
    Arc::new(
        plan.iter()
            .map(|rows| {
                let present: Vec<f64> = rows
                    .iter()
                    .filter(|&&r| values.is_valid(r))
                    .map(|&r| values.value(r))
                    .collect();
                #[expect(clippy::cast_precision_loss, reason = "interval sizes are below 2^53")]
                let n = present.len() as f64;
                (!present.is_empty()).then(|| compensated(&present) / n)
            })
            .collect::<Float64Array>(),
    )
}

/// The last row of an interval (`any_row`), or its last row with a value.
fn last_present(column: &ArrayRef, rows: &[usize], any_row: bool) -> Option<u32> {
    rows.iter()
        .rev()
        .find(|&&r| any_row || column.is_valid(r))
        .and_then(|&r| u32::try_from(r).ok())
}

enum Neighbours {
    Exact(usize),
    Between(usize, usize),
    None,
}

/// The rows around grid step `k`: a row exactly at it, or the rows just before and after.
fn neighbours(axis: &Axis, rows: &[usize], k: i64) -> Neighbours {
    let after = rows.partition_point(|&r| axis.compare(r, k).is_le());
    match (
        after.checked_sub(1).and_then(|i| rows.get(i)),
        rows.get(after),
    ) {
        (Some(&p), _) if axis.compare(p, k).is_eq() => Neighbours::Exact(p),
        (Some(&p), Some(&n)) => Neighbours::Between(p, n),
        _ => Neighbours::None,
    }
}

fn interpolate_column(
    column: &ArrayRef,
    axis: &Axis,
    rows: &[usize],
    grid: &[i64],
    numeric: bool,
    table: &str,
) -> Result<ArrayRef, DataError> {
    if numeric {
        let values =
            numbers(column).unwrap_or_else(|| Float64Array::from(vec![None::<f64>; column.len()]));
        let at = |r: usize| values.is_valid(r).then(|| values.value(r));
        let out: Float64Array = grid
            .iter()
            .map(|&k| match neighbours(axis, rows, k) {
                Neighbours::Exact(r) => at(r),
                Neighbours::Between(a, b) => {
                    let (va, vb) = (at(a)?, at(b)?);
                    Some(va + (vb - va) * axis.fraction(a, b, k))
                }
                Neighbours::None => None,
            })
            .collect();
        return Ok(Arc::new(out));
    }
    let picks: UInt32Array = grid
        .iter()
        .map(|&k| match neighbours(axis, rows, k) {
            Neighbours::Exact(r) | Neighbours::Between(r, _) => u32::try_from(r).ok(),
            Neighbours::None => None,
        })
        .collect();
    take(column, &picks, table)
}

fn take(column: &ArrayRef, indices: &UInt32Array, table: &str) -> Result<ArrayRef, DataError> {
    arrow_select::take::take(column.as_ref(), indices, None).map_err(|source| {
        DataError::MalformedTable {
            table: table.to_owned(),
            source,
        }
    })
}

fn take_all(working: &mut Working, rows: &[u32]) -> Result<(), DataError> {
    let indices = UInt32Array::from(rows.to_vec());
    let mut columns = Vec::with_capacity(working.columns.len());
    for column in &working.columns {
        columns.push(take(column, &indices, &working.table_name)?);
    }
    working.columns = columns;
    working.rows = rows.len();
    working.provenance = working
        .provenance
        .take(&rows.iter().map(|&r| r as usize).collect::<Vec<_>>());
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "test data is small and integral"
)]
mod tests {
    use arrow_array::{RecordBatch, StringArray, TimestampSecondArray};
    use arrow_schema::Schema;
    use proptest::prelude::*;
    use sonarithm_core::{Diagnostics, TableId};

    use super::*;
    use crate::provenance::with_provenance;
    use crate::table::Table;
    use crate::transform::aggregate::floats;
    use crate::transform::{Transform, apply};

    fn series(t: &[i64], v: &[Option<f64>]) -> Table {
        let schema = Arc::new(Schema::new(vec![
            Field::new(
                "t",
                DataType::Timestamp(TimeUnit::Second, Some("UTC".into())),
                true,
            ),
            Field::new("v", DataType::Float64, true),
            Field::new("s", DataType::Utf8, true),
        ]));
        let labels: StringArray = t.iter().map(|i| Some(format!("r{i}"))).collect();
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(TimestampSecondArray::from(t.to_vec()).with_timezone("UTC")),
                Arc::new(Float64Array::from(v.to_vec())),
                Arc::new(labels),
            ],
        )
        .unwrap();
        Table::from_batch(TableId(0), "t", batch)
    }

    fn run(t: &Table, transform: &Transform) -> Table {
        apply(t, transform, &mut Diagnostics::new()).unwrap()
    }

    fn smooth_t(method: SmoothMethod, alpha: Option<f64>, window: Option<u32>) -> Transform {
        Transform::Smooth {
            field: "v".into(),
            method,
            alpha,
            window,
            output: Some("sm".into()),
        }
    }

    #[test]
    fn moving_averages() {
        let t = series(&[0, 1, 2, 3], &[Some(2.0), Some(4.0), None, Some(8.0)]);
        let out = run(&t, &smooth_t(SmoothMethod::Sma, None, Some(2)));
        assert_eq!(
            floats(&out.column("sm").unwrap()),
            [Some(2.0), Some(3.0), None, Some(8.0)]
        );
        let out = run(&t, &smooth_t(SmoothMethod::Ema, Some(0.5), None));
        assert_eq!(
            floats(&out.column("sm").unwrap()),
            [Some(2.0), Some(3.0), None, Some(5.5)]
        );
        let err = apply(
            &t,
            &smooth_t(SmoothMethod::Ema, Some(0.0), None),
            &mut Diagnostics::new(),
        )
        .unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-055");
    }

    #[test]
    fn smoothing_combines_the_window_provenance() {
        let estimate = Provenance {
            class: ProvenanceClass::CvEstimate,
            confidence: Confidence::new(0.5),
        };
        let p = RowProvenance::from_rows(vec![estimate, Provenance::EXACT, Provenance::EXACT]);
        let t = with_provenance(
            &series(&[0, 1, 2], &[Some(1.0), Some(2.0), Some(3.0)]),
            &p,
            true,
        )
        .unwrap();
        let out = run(&t, &smooth_t(SmoothMethod::Sma, None, Some(2)));
        let q = RowProvenance::read(&out).unwrap();
        assert_eq!(q.get(1), estimate, "the window of row 1 holds row 0");
        assert_eq!(
            q.get(2),
            Provenance::EXACT,
            "the window of row 2 no longer does"
        );
        let out = run(&t, &smooth_t(SmoothMethod::Ema, Some(0.5), None));
        assert_eq!(
            RowProvenance::read(&out).unwrap().get(2),
            estimate,
            "ema depends on every earlier row"
        );
    }

    #[test]
    fn normalize_methods() {
        let t = series(&[0, 1, 2, 3], &[Some(1.0), Some(3.0), Some(3.0), None]);
        let norm = |method| Transform::Normalize {
            field: "v".into(),
            method,
            output: Some("n".into()),
        };
        assert_eq!(
            floats(&run(&t, &norm(NormalizeMethod::MinMax)).column("n").unwrap()),
            [Some(0.0), Some(1.0), Some(1.0), None]
        );
        assert_eq!(
            floats(&run(&t, &norm(NormalizeMethod::Rank)).column("n").unwrap()),
            [Some(1.0), Some(2.5), Some(2.5), None]
        );
        let z = floats(&run(&t, &norm(NormalizeMethod::ZScore)).column("n").unwrap());
        let mean: f64 = z.iter().flatten().sum::<f64>() / 3.0;
        assert!(mean.abs() < 1e-12, "{z:?}");
    }

    #[test]
    fn resample_mean_last_and_interpolate() {
        let t = series(
            &[0, 10, 30, 40],
            &[Some(1.0), Some(3.0), Some(5.0), Some(7.0)],
        );
        let resample = |method| Transform::Resample {
            field: "t".into(),
            every: Seconds(20.0),
            method,
        };
        let out = run(&t, &resample(ResampleMethod::Mean));
        assert_eq!(out.num_rows(), 3);
        assert_eq!(
            floats(&out.column("v").unwrap()),
            [Some(2.0), Some(5.0), Some(7.0)]
        );
        let times = out.column("t").unwrap();
        let times: Vec<_> = times
            .as_primitive::<arrow_array::types::TimestampSecondType>()
            .iter()
            .collect();
        assert_eq!(times, [Some(0), Some(20), Some(40)]);
        assert_eq!(
            RowProvenance::read(&out).unwrap().get(0).class,
            ProvenanceClass::Derived
        );
        let out = run(&t, &resample(ResampleMethod::Last));
        assert_eq!(
            floats(&out.column("v").unwrap()),
            [Some(3.0), Some(5.0), Some(7.0)]
        );
        assert_eq!(out.column("s").unwrap().as_string::<i32>().value(0), "r10");
        let out = run(&t, &resample(ResampleMethod::Interpolate));
        assert_eq!(
            floats(&out.column("v").unwrap()),
            [Some(1.0), Some(4.0), Some(7.0)]
        );
        // The exact grid points keep their (exact) provenance; 20 s is interpolated.
        let p = RowProvenance::read(&out).unwrap();
        assert_eq!(p.get(1).class, ProvenanceClass::Derived);
        let err = apply(
            &t,
            &Transform::Resample {
                field: "t".into(),
                every: Seconds(0.5),
                method: ResampleMethod::Mean,
            },
            &mut Diagnostics::new(),
        )
        .unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-055");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn smoothing_stays_inside_the_value_range(values in prop::collection::vec(-100.0_f64..100.0, 1..50), window in 1_u32..8, alpha in 0.01_f64..=1.0) {
            let t: Vec<i64> = (0..values.len() as i64).collect();
            let v: Vec<Option<f64>> = values.iter().copied().map(Some).collect();
            let table = series(&t, &v);
            let (lo, hi) = values.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &x| (a.min(x), b.max(x)));
            for transform in [smooth_t(SmoothMethod::Sma, None, Some(window)), smooth_t(SmoothMethod::Ema, Some(alpha), None)] {
                for s in floats(&run(&table, &transform).column("sm").unwrap()).into_iter().flatten() {
                    prop_assert!(s >= lo - 1e-9 && s <= hi + 1e-9, "{lo} <= {s} <= {hi}");
                }
            }
            let identity = floats(&run(&table, &smooth_t(SmoothMethod::Sma, None, Some(1))).column("sm").unwrap());
            prop_assert_eq!(identity, v);
        }

        #[test]
        fn resampled_times_are_a_regular_grid(times in prop::collection::btree_set(0_i64..10_000, 1..40), step in 1_i64..500) {
            let t: Vec<i64> = times.into_iter().collect();
            let v: Vec<Option<f64>> = t.iter().map(|&x| Some(x as f64)).collect();
            let out = run(&series(&t, &v), &Transform::Resample { field: "t".into(), every: Seconds(step as f64), method: ResampleMethod::Mean });
            let grid = out.column("t").unwrap();
            let grid: Vec<i64> = grid.as_primitive::<arrow_array::types::TimestampSecondType>().iter().flatten().collect();
            prop_assert!(grid.windows(2).all(|w| w[1] - w[0] == step));
            prop_assert!(grid.iter().all(|g| g % step == 0));
            // Every interval mean lies inside its interval (the values are the times).
            for (g, m) in grid.iter().zip(floats(&out.column("v").unwrap())) {
                if let Some(m) = m {
                    prop_assert!(m >= *g as f64 && m < (*g + step) as f64);
                }
            }
        }

        #[test]
        fn min_max_is_inside_the_unit_interval_and_ranks_are_permutation_averages(values in prop::collection::vec(-50_i32..50, 1..40)) {
            let t: Vec<i64> = (0..values.len() as i64).collect();
            let v: Vec<Option<f64>> = values.iter().map(|&x| Some(f64::from(x))).collect();
            let table = series(&t, &v);
            let norm = |method| Transform::Normalize { field: "v".into(), method, output: Some("n".into()) };
            for x in floats(&run(&table, &norm(NormalizeMethod::MinMax)).column("n").unwrap()).into_iter().flatten() {
                prop_assert!((0.0..=1.0).contains(&x));
            }
            let ranks: f64 = floats(&run(&table, &norm(NormalizeMethod::Rank)).column("n").unwrap()).into_iter().flatten().sum();
            let n = values.len() as f64;
            prop_assert!((ranks - n * (n + 1.0) / 2.0).abs() < 1e-9);
        }
    }
}
