// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Streaming tables (03 §8; B06-T01): bounded, append-only storage for live data.
//!
//! A [`StreamTable`] keeps a ring of record batches. Every row gets a sequence number,
//! counting from 0 in arrival order, so readers ask for "the rows since sequence `s`"
//! ([`StreamTable::since`]) and SONOTRACE encodes only new rows (05 §6). Memory stays
//! bounded ([`Capacity`]): the oldest rows go first, whole or as a slice of a batch, and
//! rows are never removed from the middle. A reader that falls behind the window is told
//! how many rows it missed, with an `SNR-DATA-071` diagnostic (backpressure), instead of
//! receiving a gap.
//!
//! Snapshots ([`Snapshot`]) share the batches' buffers (Arrow arrays are reference
//! counted), so taking one is cheap and later appends never change it. Provenance columns
//! (`__snr_prov`, `__snr_conf`, 03 §4) are rewritten in their canonical form on append and
//! travel with their rows (AGENTS.md rule 7).
//!
//! The table lives in the data layer, not in `sonarithm-stream`, so engines (SONOTRACE's
//! `live` module) can read it without a dependency on the network adapters or tokio.

use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;

use arrow_array::{Array, ArrayRef, RecordBatch};
use arrow_schema::{DataType, SchemaRef, TimeUnit};
use sonarithm_core::{Diagnostic, Diagnostics, TableId};

use crate::codes;
use crate::error::DataError;
use crate::infer::semantic::numbers;
use crate::provenance;
use crate::table::Table;

/// How much a stream keeps. Every capacity has a hard row limit, so memory stays bounded
/// whatever the source's rate (PB-11).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Capacity {
    /// The newest `n` rows.
    Rows(u64),
    /// The rows whose time field is within `seconds` of the newest row's, and at most
    /// `max_rows` of them.
    Window {
        /// The time field: numbers (read as seconds), timestamps, dates or durations.
        field: String,
        /// The window's length in seconds.
        seconds: f64,
        /// The hard row limit.
        max_rows: u64,
    },
}

impl Capacity {
    /// The newest `n` rows.
    #[must_use]
    pub const fn rows(n: u64) -> Self {
        Self::Rows(n)
    }

    /// The last `seconds` of `field`, at most `max_rows` rows.
    #[must_use]
    pub fn window(field: impl Into<String>, seconds: f64, max_rows: u64) -> Self {
        Self::Window {
            field: field.into(),
            seconds,
            max_rows,
        }
    }

    /// The hard row limit.
    #[must_use]
    pub const fn max_rows(&self) -> u64 {
        match self {
            Self::Rows(n) => *n,
            Self::Window { max_rows, .. } => *max_rows,
        }
    }
}

/// What an append did.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Appended {
    /// The sequence numbers the new rows received.
    pub seqs: Range<u64>,
    /// Rows removed from the front to stay within the capacity, new rows included when the
    /// batch alone was larger than the row limit.
    pub evicted: u64,
    /// `SNR-DATA-071` when rows of this very batch could not be kept.
    pub diagnostics: Diagnostics,
}

#[derive(Clone, Debug)]
struct Chunk {
    first: u64,
    batch: RecordBatch,
}

/// A bounded, append-only table of live rows with sequence numbers (03 §8).
#[derive(Clone, Debug)]
pub struct StreamTable {
    id: TableId,
    name: String,
    /// The schema sources append in.
    input: SchemaRef,
    /// The schema rows are kept in (canonical provenance columns).
    schema: SchemaRef,
    capacity: Capacity,
    chunks: VecDeque<Chunk>,
    /// The sequence number of the oldest row kept.
    first: u64,
    /// The sequence number the next row will get.
    next: u64,
    held: u64,
    evicted: u64,
}

impl StreamTable {
    /// An empty stream of rows with `schema`.
    ///
    /// # Errors
    ///
    /// [`DataError::InvalidStream`] for a zero row limit, a window that is not a positive
    /// finite number of seconds, or a time field the schema lacks or cannot read as time;
    /// [`DataError::InvalidProvenance`] for malformed provenance columns.
    pub fn new(
        id: TableId,
        name: impl Into<String>,
        schema: SchemaRef,
        capacity: Capacity,
    ) -> Result<Self, DataError> {
        if capacity.max_rows() == 0 {
            return Err(invalid("the row limit must be at least 1"));
        }
        if let Capacity::Window { field, seconds, .. } = &capacity {
            if !(seconds.is_finite() && *seconds > 0.0) {
                return Err(invalid("the window must be a positive number of seconds"));
            }
            let index = schema
                .index_of(field)
                .map_err(|_| invalid(&format!("the stream has no field `{field}`")))?;
            if scale(schema.field(index).data_type()).is_none() {
                return Err(invalid(&format!(
                    "`{field}` is not a number, timestamp, date or duration"
                )));
            }
        }
        let name = name.into();
        let empty = RecordBatch::new_empty(Arc::clone(&schema));
        let canonical = canonical(id, &name, empty)?;
        Ok(Self {
            id,
            name,
            input: schema,
            schema: canonical.schema(),
            capacity,
            chunks: VecDeque::new(),
            first: 0,
            next: 0,
            held: 0,
            evicted: 0,
        })
    }

    /// Appends a batch, then removes the oldest rows beyond the capacity.
    ///
    /// # Errors
    ///
    /// [`DataError::InvalidStream`] if the batch's fields differ from the stream's in name
    /// or type; [`DataError::InvalidProvenance`] for malformed provenance values.
    pub fn append(&mut self, batch: RecordBatch) -> Result<Appended, DataError> {
        let same = batch.schema().fields().len() == self.input.fields().len()
            && batch
                .schema()
                .fields()
                .iter()
                .zip(self.input.fields())
                .all(|(a, b)| a.name() == b.name() && a.data_type() == b.data_type());
        if !same {
            return Err(invalid("a batch's fields differ from the stream's"));
        }
        let (_, columns, _) = batch.into_parts();
        let batch = RecordBatch::try_new(Arc::clone(&self.input), columns)
            .map_err(|err| invalid(&err.to_string()))?;
        let batch = canonical(self.id, &self.name, batch)?;
        let rows = row_count(&batch);
        let seqs = self.next..self.next + rows;
        if rows > 0 {
            self.chunks.push_back(Chunk {
                first: self.next,
                batch,
            });
            self.next += rows;
            self.held += rows;
        }
        let before = self.evicted;
        self.evict();
        let evicted = self.evicted - before;
        let mut diagnostics = Diagnostics::new();
        if self.first > seqs.start {
            let lost = self.first.min(seqs.end) - seqs.start;
            diagnostics.push(dropped(lost, &self.name));
        }
        Ok(Appended {
            seqs,
            evicted,
            diagnostics,
        })
    }

    /// Removes rows from the front: beyond the row limit, then outside the time window.
    fn evict(&mut self) {
        let max = self.capacity.max_rows();
        while self.held > max {
            let excess = self.held - max;
            self.drop_front(excess);
        }
        if let Capacity::Window { field, seconds, .. } = &self.capacity {
            let (field, seconds) = (field.clone(), *seconds);
            let Some(newest) = self.newest_time(&field) else {
                return;
            };
            let cutoff = newest - seconds;
            while let Some(chunk) = self.chunks.front() {
                let old = times(&chunk.batch, &field)
                    .iter()
                    .take_while(|t| t.is_some_and(|t| t < cutoff))
                    .count();
                if old == 0 {
                    break;
                }
                self.drop_front(u64::try_from(old).unwrap_or(u64::MAX));
            }
        }
    }

    /// Drops up to `n` rows from the front chunk.
    fn drop_front(&mut self, n: u64) {
        let Some(chunk) = self.chunks.front_mut() else {
            return;
        };
        let rows = row_count(&chunk.batch);
        let n = n.min(rows);
        if n == rows {
            self.chunks.pop_front();
        } else {
            let keep = usize::try_from(rows - n).unwrap_or(0);
            let skip = usize::try_from(n).unwrap_or(0);
            chunk.batch = chunk.batch.slice(skip, keep);
            chunk.first += n;
        }
        self.first += n;
        self.held -= n;
        self.evicted += n;
    }

    fn newest_time(&self, field: &str) -> Option<f64> {
        self.chunks.iter().rev().find_map(|chunk| {
            times(&chunk.batch, field)
                .into_iter()
                .rev()
                .flatten()
                .next()
        })
    }

    /// Every row kept, as a snapshot.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        self.since(self.first)
    }

    /// The rows with sequence numbers from `seq` on. If some of those rows were already
    /// dropped, the snapshot starts at the oldest row kept and reports how many it skipped.
    #[must_use]
    pub fn since(&self, seq: u64) -> Snapshot {
        let skipped = self.first.saturating_sub(seq);
        let from = seq.max(self.first).min(self.next);
        let mut batches = Vec::new();
        for chunk in &self.chunks {
            let rows = row_count(&chunk.batch);
            let end = chunk.first + rows;
            if end <= from {
                continue;
            }
            let offset = from.saturating_sub(chunk.first);
            let batch = if offset == 0 {
                chunk.batch.clone()
            } else {
                chunk.batch.slice(
                    usize::try_from(offset).unwrap_or(0),
                    usize::try_from(rows - offset).unwrap_or(0),
                )
            };
            batches.push(batch);
        }
        Snapshot {
            schema: Arc::clone(&self.schema),
            name: self.name.clone(),
            first: from,
            rows: self.next - from,
            skipped,
            batches,
        }
    }

    /// The sequence number of the oldest row kept.
    #[must_use]
    pub const fn first_seq(&self) -> u64 {
        self.first
    }

    /// The sequence number the next row will get: the number of rows ever appended.
    #[must_use]
    pub const fn next_seq(&self) -> u64 {
        self.next
    }

    /// The number of rows kept.
    #[must_use]
    pub const fn len(&self) -> u64 {
        self.held
    }

    /// Whether no rows are kept.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.held == 0
    }

    /// Rows removed so far to stay within the capacity.
    #[must_use]
    pub const fn evicted(&self) -> u64 {
        self.evicted
    }

    /// The schema rows are kept in (provenance columns in canonical form).
    #[must_use]
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// The capacity.
    #[must_use]
    pub const fn capacity(&self) -> &Capacity {
        &self.capacity
    }
}

/// A consistent view of a stream's rows `[first_seq, next_seq)`, sharing their buffers.
#[derive(Clone, Debug)]
pub struct Snapshot {
    schema: SchemaRef,
    name: String,
    first: u64,
    rows: u64,
    skipped: u64,
    batches: Vec<RecordBatch>,
}

impl Snapshot {
    /// The sequence number of its first row.
    #[must_use]
    pub const fn first_seq(&self) -> u64 {
        self.first
    }

    /// One past the sequence number of its last row.
    #[must_use]
    pub const fn next_seq(&self) -> u64 {
        self.first + self.rows
    }

    /// The number of rows.
    #[must_use]
    pub const fn num_rows(&self) -> u64 {
        self.rows
    }

    /// Rows the reader asked for that were dropped before it read them.
    #[must_use]
    pub const fn skipped(&self) -> u64 {
        self.skipped
    }

    /// `SNR-DATA-071` when rows were skipped.
    #[must_use]
    pub fn diagnostic(&self) -> Option<Diagnostic> {
        (self.skipped > 0).then(|| dropped(self.skipped, &self.name))
    }

    /// The batches, oldest first.
    #[must_use]
    pub fn batches(&self) -> &[RecordBatch] {
        &self.batches
    }

    /// The schema.
    #[must_use]
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// The earliest and latest time in `field`, in seconds (numbers as seconds; timestamps,
    /// dates and durations converted, as for [`Capacity::window`]); `None` without times.
    #[must_use]
    pub fn time_span(&self, field: &str) -> Option<(f64, f64)> {
        self.batches
            .iter()
            .flat_map(|batch| times(batch, field))
            .flatten()
            .fold(None, |span, t| match span {
                None => Some((t, t)),
                Some((lo, hi)) => Some((f64::min(lo, t), f64::max(hi, t))),
            })
    }

    /// The rows within `seconds` of the newest time in `field`: the window a live spec
    /// compiles (B06-T07). Rows arrive in time order, as [`Capacity::window`] assumes, so
    /// only rows at the front are dropped; without times every row stays.
    #[must_use]
    pub fn within(&self, field: &str, seconds: f64) -> Self {
        let Some((_, newest)) = self.time_span(field) else {
            return self.clone();
        };
        let cutoff = newest - seconds;
        let mut dropped = 0_u64;
        let mut batches = Vec::with_capacity(self.batches.len());
        for batch in &self.batches {
            if !batches.is_empty() {
                batches.push(batch.clone());
                continue;
            }
            let old = times(batch, field)
                .iter()
                .take_while(|t| t.is_some_and(|t| t < cutoff))
                .count();
            dropped += u64::try_from(old).unwrap_or(u64::MAX);
            if old < batch.num_rows() {
                batches.push(batch.slice(old, batch.num_rows() - old));
            }
        }
        Self {
            schema: Arc::clone(&self.schema),
            name: self.name.clone(),
            first: self.first + dropped,
            rows: self.rows - dropped,
            skipped: self.skipped,
            batches,
        }
    }

    /// The rows as a [`Table`] with `id`, for compiling and viewing.
    ///
    /// # Errors
    ///
    /// [`DataError::MalformedTable`] never in practice: every batch has the schema.
    pub fn to_table(&self, id: TableId) -> Result<Table, DataError> {
        Table::new(
            id,
            self.name.clone(),
            Arc::clone(&self.schema),
            self.batches.clone(),
        )
    }
}

fn invalid(reason: &str) -> DataError {
    DataError::InvalidStream {
        reason: reason.to_owned(),
    }
}

fn dropped(rows: u64, name: &str) -> Diagnostic {
    Diagnostic::new(
        &codes::STREAM_ROWS_DROPPED,
        format!(
            "{rows} row(s) of stream `{name}` were dropped before they were read: the source \
             is faster than the window keeps. Widen the window, or read the stream more often."
        ),
    )
}

fn row_count(batch: &RecordBatch) -> u64 {
    u64::try_from(batch.num_rows()).unwrap_or(u64::MAX)
}

/// The batch with its provenance columns in canonical form (03 §4).
fn canonical(id: TableId, name: &str, batch: RecordBatch) -> Result<RecordBatch, DataError> {
    let table = Table::from_batch(id, name, batch);
    provenance::normalize(&table)?.to_batch()
}

/// Seconds per unit of a time column, or `None` if it is not one.
fn scale(data_type: &DataType) -> Option<f64> {
    let unit = |unit: &TimeUnit| match unit {
        TimeUnit::Second => 1.0,
        TimeUnit::Millisecond => 1e-3,
        TimeUnit::Microsecond => 1e-6,
        TimeUnit::Nanosecond => 1e-9,
    };
    match data_type {
        DataType::Timestamp(u, _) | DataType::Duration(u) => Some(unit(u)),
        DataType::Date32 => Some(86_400.0),
        DataType::Date64 => Some(1e-3),
        t if t.is_numeric() => Some(1.0),
        _ => None,
    }
}

/// A batch's times in seconds, `None` for nulls.
fn times(batch: &RecordBatch, field: &str) -> Vec<Option<f64>> {
    let Some(column): Option<&ArrayRef> = batch
        .schema()
        .index_of(field)
        .ok()
        .map(|index| batch.column(index))
    else {
        return Vec::new();
    };
    let factor = scale(column.data_type()).unwrap_or(1.0);
    numbers(column).map_or_else(Vec::new, |values| {
        (0..values.len())
            .map(|i| values.is_valid(i).then(|| values.value(i) * factor))
            .collect()
    })
}

#[cfg(test)]
mod tests;
