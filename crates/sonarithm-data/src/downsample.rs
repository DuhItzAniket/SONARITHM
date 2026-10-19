// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Downsampled views for drawing (03 §7): M4 for pixel-exact line charts, LTTB for
//! overviews, and a cache of M4 tiles.
//!
//! Downsampling affects only what is drawn. Sonification always reads the full data
//! (03 §7). Every function returns *row indices* into the table, so a drawn point keeps its
//! `RowRef` for linked selection and announcements.

use std::collections::BTreeMap;
use std::sync::Arc;

use arrow_array::{Array, ArrayRef, Float64Array};
use sonarithm_core::{Generation, TableId};

use crate::error::DataError;
use crate::infer::semantic::numbers;
use crate::table::Table;

/// An x range `[start, end]` to draw, in the units of the x field (days for dates, the
/// timestamp unit for timestamps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct XRange {
    /// The left edge.
    pub start: f64,
    /// The right edge.
    pub end: f64,
}

impl XRange {
    /// A range, or an error unless both edges are finite and `start < end`.
    ///
    /// # Errors
    ///
    /// [`DataError::InvalidView`] for an empty, reversed or infinite range.
    pub fn new(start: f64, end: f64) -> Result<Self, DataError> {
        if start.is_finite() && end.is_finite() && start < end {
            Ok(Self { start, end })
        } else {
            Err(DataError::InvalidView {
                reason: format!("the x range [{start}, {end}] must be finite and increasing"),
            })
        }
    }

    /// The pixel column of `x` for `width` columns: `floor(width · (x − start) / (end −
    /// start))`, with `end` itself in the last column; `None` outside the range. This is
    /// the mapping a canvas uses for columns `0..width`.
    #[must_use]
    pub fn column(&self, x: f64, width: u32) -> Option<u32> {
        if !(x >= self.start && x <= self.end) {
            return None;
        }
        let position = f64::from(width) * (x - self.start) / (self.end - self.start);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "0 <= position <= width"
        )]
        let column = position.floor() as u32;
        Some(column.min(width.saturating_sub(1)))
    }
}

/// The values of a number, date or timestamp field as `f64`.
fn axis(column: &ArrayRef, field: &str) -> Result<Float64Array, DataError> {
    numbers(column).ok_or_else(|| DataError::FieldTypeMismatch {
        op: "downsample",
        field: field.to_owned(),
        expected: "numbers, dates or timestamps",
        found: column.data_type().to_string(),
    })
}

/// The four extremes M4 keeps for one pixel column: (value, row).
#[derive(Clone, Copy)]
struct Column {
    first: (f64, usize),
    last: (f64, usize),
    min: (f64, usize),
    max: (f64, usize),
}

/// M4 (Jugel et al., VLDB 2014): for each of `width` pixel columns over `range`, the rows
/// with the first and last x and the minimum and maximum y. Rows with a missing x or y, or
/// outside the range, are left out; ties go to the earliest row (the latest for the last
/// x). The result is sorted and has at most `4 × width` rows.
///
/// Drawn as a polyline in row order, the rows give exactly the pixels of the full data's
/// polyline when the rows are sorted by x (the order a line chart draws them in).
///
/// # Errors
///
/// [`DataError::InvalidView`] for a zero width.
pub fn m4(
    x: &Float64Array,
    y: &Float64Array,
    range: XRange,
    width: u32,
) -> Result<Vec<u64>, DataError> {
    if width == 0 {
        return Err(DataError::InvalidView {
            reason: "the width must be at least one pixel".to_owned(),
        });
    }
    let mut columns: BTreeMap<u32, Column> = BTreeMap::new();
    for row in 0..x.len().min(y.len()) {
        if !(x.is_valid(row) && y.is_valid(row)) {
            continue;
        }
        let (xv, yv) = (x.value(row), y.value(row));
        if yv.is_nan() {
            continue;
        }
        let Some(key) = range.column(xv, width) else {
            continue;
        };
        columns
            .entry(key)
            .and_modify(|c| {
                if xv < c.first.0 {
                    c.first = (xv, row);
                }
                if xv >= c.last.0 {
                    c.last = (xv, row);
                }
                if yv < c.min.0 {
                    c.min = (yv, row);
                }
                if yv > c.max.0 {
                    c.max = (yv, row);
                }
            })
            .or_insert(Column {
                first: (xv, row),
                last: (xv, row),
                min: (yv, row),
                max: (yv, row),
            });
    }
    let mut rows: Vec<u64> = columns
        .values()
        .flat_map(|c| [c.first.1, c.last.1, c.min.1, c.max.1])
        .map(|r| u64::try_from(r).unwrap_or(u64::MAX))
        .collect();
    rows.sort_unstable();
    rows.dedup();
    Ok(rows)
}

/// Largest-Triangle-Three-Buckets (Steinarsson 2013): `threshold` representative rows for
/// an overview, always including the first and last. Rows with a missing x or y are left
/// out. With fewer rows than `threshold`, or a threshold below 3, every row is kept.
#[must_use]
pub fn lttb(x: &Float64Array, y: &Float64Array, threshold: usize) -> Vec<u64> {
    let rows: Vec<usize> = (0..x.len().min(y.len()))
        .filter(|&r| {
            x.is_valid(r) && y.is_valid(r) && x.value(r).is_finite() && y.value(r).is_finite()
        })
        .collect();
    let to_u64 = |r: usize| u64::try_from(r).unwrap_or(u64::MAX);
    if threshold < 3 || rows.len() <= threshold {
        return rows.into_iter().map(to_u64).collect();
    }
    let point = |i: usize| {
        rows.get(i)
            .map_or((0.0, 0.0), |&r| (x.value(r), y.value(r)))
    };
    let inner = rows.len() - 2;
    let buckets = threshold - 2;
    // Bucket b covers inner points [b·inner/buckets, (b+1)·inner/buckets), offset by one.
    let bounds = |b: usize| (1 + b * inner / buckets, 1 + (b + 1) * inner / buckets);
    let mut chosen = Vec::with_capacity(threshold);
    chosen.push(0_usize);
    let mut previous = 0_usize;
    for b in 0..buckets {
        let (start, end) = bounds(b);
        let (next_start, next_end) = if b + 1 < buckets {
            bounds(b + 1)
        } else {
            (rows.len() - 1, rows.len())
        };
        #[expect(clippy::cast_precision_loss, reason = "bucket sizes are below 2^53")]
        let count = (next_end - next_start) as f64;
        let (sum_x, sum_y) = (next_start..next_end)
            .map(point)
            .fold((0.0, 0.0), |(a, b), (px, py)| (a + px, b + py));
        let (avg_x, avg_y) = (sum_x / count, sum_y / count);
        let (ax, ay) = point(previous);
        let mut best = (f64::NEG_INFINITY, start);
        for i in start..end {
            let (px, py) = point(i);
            let area = ((ax - avg_x) * (py - ay) - (ax - px) * (avg_y - ay)).abs();
            if area > best.0 {
                best = (area, i);
            }
        }
        chosen.push(best.1);
        previous = best.1;
    }
    chosen.push(rows.len() - 1);
    chosen
        .into_iter()
        .filter_map(|i| rows.get(i).copied())
        .map(to_u64)
        .collect()
}

/// M4 rows of two fields of a table.
///
/// # Errors
///
/// [`DataError::UnknownField`], [`DataError::FieldTypeMismatch`] for fields that are not
/// numbers, dates or timestamps, and [`DataError::InvalidView`].
pub fn m4_table(
    table: &Table,
    x: &str,
    y: &str,
    range: XRange,
    width: u32,
) -> Result<Vec<u64>, DataError> {
    let xs = axis(&table.column(x)?, x)?;
    let ys = axis(&table.column(y)?, y)?;
    m4(&xs, &ys, range, width)
}

/// What identifies an M4 tile: table generation, fields, x range and pixel width (03 §7).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileKey {
    /// The table.
    pub table: TableId,
    /// Its generation; a new generation never reuses an old tile.
    pub generation: Generation,
    /// The x field.
    pub x: String,
    /// The y field.
    pub y: String,
    /// The range edges as IEEE bits, so equal ranges compare equal exactly.
    pub range: (u64, u64),
    /// The width in pixels.
    pub width: u32,
}

/// A bounded cache of M4 tiles that evicts the least recently used tile. Recency is a
/// counter, not a clock, so behaviour is deterministic.
#[derive(Debug)]
pub struct TileCache {
    capacity: usize,
    tick: u64,
    tiles: BTreeMap<TileKey, (u64, Arc<Vec<u64>>)>,
}

impl TileCache {
    /// A cache that holds at most `capacity` tiles (at least one).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            tick: 0,
            tiles: BTreeMap::new(),
        }
    }

    /// The tile for these fields, range and width, computed on a miss.
    ///
    /// # Errors
    ///
    /// As [`m4_table`].
    pub fn tile(
        &mut self,
        table: &Table,
        x: &str,
        y: &str,
        range: XRange,
        width: u32,
    ) -> Result<Arc<Vec<u64>>, DataError> {
        let key = TileKey {
            table: table.id(),
            generation: table.generation(),
            x: x.to_owned(),
            y: y.to_owned(),
            range: (range.start.to_bits(), range.end.to_bits()),
            width,
        };
        self.tick += 1;
        if let Some((used, rows)) = self.tiles.get_mut(&key) {
            *used = self.tick;
            return Ok(Arc::clone(rows));
        }
        let rows = Arc::new(m4_table(table, x, y, range, width)?);
        if self.tiles.len() >= self.capacity
            && let Some(oldest) = self
                .tiles
                .iter()
                .min_by_key(|(_, (used, _))| *used)
                .map(|(k, _)| k.clone())
        {
            self.tiles.remove(&oldest);
        }
        self.tiles.insert(key, (self.tick, Arc::clone(&rows)));
        Ok(rows)
    }

    /// Drops the tiles of `table` that belong to generations other than `current`.
    pub fn retain_generation(&mut self, table: TableId, current: Generation) {
        self.tiles
            .retain(|key, _| key.table != table || key.generation == current);
    }

    /// The number of cached tiles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "test oracles compare exact data values and index small vectors"
)]
mod tests {
    use std::collections::BTreeSet;

    use arrow_array::RecordBatch;
    use arrow_schema::{DataType, Field, Schema};
    use proptest::prelude::*;

    use super::*;

    /// The M4 query of the paper, executed literally: per column, every row whose y is the
    /// column's minimum or maximum or whose x is its first or last.
    fn reference_m4(x: &[f64], y: &[f64], range: XRange, width: u32) -> BTreeSet<u64> {
        let mut stats: BTreeMap<u32, (f64, f64, f64, f64)> = BTreeMap::new();
        for (xv, yv) in x.iter().zip(y) {
            if let Some(k) = range.column(*xv, width) {
                let e = stats.entry(k).or_insert((*xv, *xv, *yv, *yv));
                *e = (e.0.min(*xv), e.1.max(*xv), e.2.min(*yv), e.3.max(*yv));
            }
        }
        (0..x.len())
            .filter(|&r| {
                range.column(x[r], width).is_some_and(|k| {
                    let (t_min, t_max, v_min, v_max) = stats[&k];
                    x[r] == t_min || x[r] == t_max || y[r] == v_min || y[r] == v_max
                })
            })
            .map(|r| r as u64)
            .collect()
    }

    /// Pixels of a polyline through `rows` (in order) on a `width × height` canvas, drawn
    /// with Bresenham segments between pixel-snapped points.
    fn raster(
        x: &[f64],
        y: &[f64],
        rows: &[u64],
        range: XRange,
        width: u32,
        height: u32,
    ) -> BTreeSet<(i64, i64)> {
        let (lo, hi) = y
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &v| {
                (a.min(v), b.max(v))
            });
        let py = |v: f64| {
            if hi > lo {
                ((f64::from(height) * (v - lo) / (hi - lo)).floor() as i64)
                    .min(i64::from(height) - 1)
            } else {
                0
            }
        };
        let points: Vec<(i64, i64)> = rows
            .iter()
            .filter_map(|&r| {
                range
                    .column(x[r as usize], width)
                    .map(|c| (i64::from(c), py(y[r as usize])))
            })
            .collect();
        let mut pixels = BTreeSet::new();
        for pair in points.windows(2) {
            let ((mut x0, mut y0), (x1, y1)) = (pair[0], pair[1]);
            let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
            let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
            let mut err = dx + dy;
            loop {
                pixels.insert((x0, y0));
                if (x0, y0) == (x1, y1) {
                    break;
                }
                let e2 = 2 * err;
                if e2 >= dy {
                    err += dy;
                    x0 += sx;
                }
                if e2 <= dx {
                    err += dx;
                    y0 += sy;
                }
            }
        }
        if let [only] = points.as_slice() {
            pixels.insert(*only);
        }
        pixels
    }

    fn walk() -> impl Strategy<Value = (Vec<f64>, Vec<f64>)> {
        prop::collection::vec((0.0_f64..3.0, -5.0_f64..5.0), 2..400).prop_map(|steps| {
            let mut t = 0.0;
            let mut v = 0.0;
            steps
                .into_iter()
                .map(|(dt, dv)| {
                    t += dt;
                    v += dv;
                    (t, v)
                })
                .unzip()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn m4_is_pixel_exact_against_the_full_polyline((x, y) in walk(), width in 1_u32..120, height in 1_u32..80) {
            let range = XRange::new(x[0], x[x.len() - 1] + 1e-9).unwrap();
            let rows = m4(&Float64Array::from(x.clone()), &Float64Array::from(y.clone()), range, width).unwrap();
            let all: Vec<u64> = (0..x.len() as u64).collect();
            prop_assert_eq!(raster(&x, &y, &rows, range, width, height), raster(&x, &y, &all, range, width, height));
            // M4 picks a subset of the paper's reference query and at most four rows per column.
            let reference = reference_m4(&x, &y, range, width);
            prop_assert!(rows.iter().all(|r| reference.contains(r)));
            prop_assert!(rows.len() <= 4 * width as usize);
            // Every column keeps its minimum and maximum.
            for (lo, hi) in reference.iter().map(|&r| range.column(x[r as usize], width)).collect::<BTreeSet<_>>().into_iter().flatten().map(|k| {
                let in_col: Vec<f64> = (0..x.len()).filter(|&r| range.column(x[r], width) == Some(k)).map(|r| y[r]).collect();
                (in_col.iter().copied().fold(f64::INFINITY, f64::min), in_col.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            }) {
                prop_assert!(rows.iter().any(|&r| y[r as usize] == lo));
                prop_assert!(rows.iter().any(|&r| y[r as usize] == hi));
            }
        }

        #[test]
        fn lttb_keeps_the_ends_and_the_threshold((x, y) in walk(), threshold in 3_usize..60) {
            let rows = lttb(&Float64Array::from(x.clone()), &Float64Array::from(y), threshold);
            prop_assert_eq!(rows.len(), threshold.min(x.len()));
            prop_assert_eq!(rows.first().copied(), Some(0));
            prop_assert_eq!(rows.last().copied(), Some(x.len() as u64 - 1));
            prop_assert!(rows.windows(2).all(|w| w[0] < w[1]));
        }
    }

    #[test]
    fn missing_values_and_points_outside_the_range_are_skipped() {
        let x = Float64Array::from(vec![Some(0.0), Some(1.0), None, Some(2.0), Some(9.0)]);
        let y = Float64Array::from(vec![Some(1.0), None, Some(3.0), Some(2.0), Some(4.0)]);
        let rows = m4(&x, &y, XRange::new(0.0, 4.0).unwrap(), 2).unwrap();
        assert_eq!(rows, [0, 3]);
        assert!(XRange::new(1.0, 1.0).is_err());
        assert!(m4(&x, &y, XRange::new(0.0, 1.0).unwrap(), 0).is_err());
    }

    fn table(generation_bump: bool) -> Table {
        let schema = Arc::new(Schema::new(vec![
            Field::new("t", DataType::Float64, false),
            Field::new("v", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Float64Array::from(
                    (0..100).map(f64::from).collect::<Vec<_>>(),
                )),
                Arc::new(Float64Array::from(
                    (0..100).map(|i| f64::from(i % 7)).collect::<Vec<_>>(),
                )),
            ],
        )
        .unwrap();
        let t = Table::from_batch(TableId(1), "t", batch);
        if generation_bump {
            t.next_generation(Arc::clone(t.schema()), t.batches().to_vec())
                .unwrap()
        } else {
            t
        }
    }

    #[test]
    fn tiles_are_cached_per_generation_with_lru_eviction() {
        let mut cache = TileCache::new(2);
        let t = table(false);
        let range = XRange::new(0.0, 99.0).unwrap();
        let a = cache.tile(&t, "t", "v", range, 10).unwrap();
        let again = cache.tile(&t, "t", "v", range, 10).unwrap();
        assert!(Arc::ptr_eq(&a, &again));
        let b = cache.tile(&t, "t", "v", range, 20).unwrap();
        assert!(!Arc::ptr_eq(&a, &b));
        // Touch the first tile, then add a third: the second (least recently used) goes.
        cache.tile(&t, "t", "v", range, 10).unwrap();
        cache.tile(&t, "t", "v", range, 30).unwrap();
        assert_eq!(cache.len(), 2);
        assert!(Arc::ptr_eq(
            &a,
            &cache.tile(&t, "t", "v", range, 10).unwrap()
        ));
        let next = table(true);
        let c = cache.tile(&next, "t", "v", range, 10).unwrap();
        assert!(
            !Arc::ptr_eq(&a, &c),
            "a new generation never reuses an old tile"
        );
        cache.retain_generation(TableId(1), next.generation());
        assert_eq!(cache.len(), 1);
        let err = cache.tile(&t, "t", "nope", range, 10).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-050");
    }
}
