// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::sync::Arc;

use arrow_array::{Array, Float32Array, Float64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use proptest::prelude::*;

use super::*;
use crate::provenance::{CONF_COLUMN, PROV_COLUMN, RowProvenance};

fn schema(with_provenance: bool) -> SchemaRef {
    let mut fields = vec![
        Field::new("t", DataType::Float64, false),
        Field::new("v", DataType::Float64, true),
    ];
    if with_provenance {
        fields.push(Field::new(PROV_COLUMN, DataType::Utf8, true));
        fields.push(Field::new(CONF_COLUMN, DataType::Float32, true));
    }
    Arc::new(Schema::new(fields))
}

/// `n` rows whose `t` and `v` count up from `start`.
fn batch(schema: &SchemaRef, start: u64, n: usize) -> RecordBatch {
    #[expect(clippy::cast_precision_loss, reason = "small test counts")]
    let ts: Vec<f64> = (0..n).map(|i| (start + i as u64) as f64).collect();
    let mut columns: Vec<ArrayRef> = vec![
        Arc::new(Float64Array::from(ts.clone())),
        Arc::new(Float64Array::from(ts.clone())),
    ];
    if schema.fields().len() == 4 {
        let classes: Vec<&str> = (0..n)
            .map(|i| if i % 3 == 0 { "cv_estimate" } else { "exact" })
            .collect();
        let conf: Vec<Option<f32>> = (0..n).map(|i| (i % 3 == 0).then_some(0.5)).collect();
        columns.push(Arc::new(StringArray::from(classes)));
        columns.push(Arc::new(Float32Array::from(conf)));
    }
    RecordBatch::try_new(Arc::clone(schema), columns).unwrap()
}

/// The `v` values of a snapshot, in order.
fn values(snapshot: &Snapshot) -> Vec<f64> {
    snapshot
        .batches()
        .iter()
        .flat_map(|b| {
            let column = b.column(1).as_any().downcast_ref::<Float64Array>().unwrap();
            column.values().to_vec()
        })
        .collect()
}

fn stream(capacity: Capacity, with_provenance: bool) -> StreamTable {
    StreamTable::new(TableId(1), "live", schema(with_provenance), capacity).unwrap()
}

#[test]
fn rows_get_sequence_numbers_and_the_oldest_go_first() {
    let mut s = stream(Capacity::rows(5), false);
    let schema = schema(false);
    let a = s.append(batch(&schema, 0, 3)).unwrap();
    assert_eq!(a.seqs, 0..3);
    let b = s.append(batch(&schema, 3, 4)).unwrap();
    assert_eq!(b.seqs, 3..7);
    assert_eq!(b.evicted, 2);
    assert_eq!((s.first_seq(), s.next_seq(), s.len()), (2, 7, 5));
    assert_eq!(values(&s.snapshot()), [2.0, 3.0, 4.0, 5.0, 6.0]);
    assert!(b.diagnostics.is_empty());
}

#[test]
fn a_batch_larger_than_the_limit_keeps_its_newest_rows_and_says_so() {
    let mut s = stream(Capacity::rows(4), false);
    let appended = s.append(batch(&schema(false), 0, 10)).unwrap();
    assert_eq!(values(&s.snapshot()), [6.0, 7.0, 8.0, 9.0]);
    let codes: Vec<&str> = appended
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["SNR-DATA-071"]);
}

#[test]
fn a_reader_behind_the_window_is_told_what_it_missed() {
    let mut s = stream(Capacity::rows(3), false);
    let schema = schema(false);
    s.append(batch(&schema, 0, 3)).unwrap();
    let read = s.since(0);
    assert_eq!((read.skipped(), read.num_rows()), (0, 3));
    s.append(batch(&schema, 3, 5)).unwrap();
    let late = s.since(read.next_seq());
    assert_eq!(late.skipped(), 2);
    assert_eq!(late.first_seq(), 5);
    assert_eq!(values(&late), [5.0, 6.0, 7.0]);
    assert_eq!(
        late.diagnostic()
            .map(|d| d.code.as_str().to_owned())
            .as_deref(),
        Some("SNR-DATA-071")
    );
    // Caught up: nothing new, nothing skipped.
    let now = s.since(late.next_seq());
    assert_eq!((now.num_rows(), now.skipped()), (0, 0));
}

#[test]
fn a_time_window_keeps_the_last_seconds() {
    let mut s = stream(Capacity::window("t", 2.5, 1_000), false);
    s.append(batch(&schema(false), 0, 10)).unwrap();
    // Newest t is 9: rows with t >= 6.5 stay.
    assert_eq!(values(&s.snapshot()), [7.0, 8.0, 9.0]);
}

#[test]
fn invalid_streams_and_batches_are_refused() {
    let schema = schema(false);
    let new = |capacity| StreamTable::new(TableId(1), "s", Arc::clone(&schema), capacity);
    assert!(matches!(
        new(Capacity::rows(0)),
        Err(DataError::InvalidStream { .. })
    ));
    assert!(new(Capacity::window("t", 0.0, 10)).is_err());
    assert!(new(Capacity::window("missing", 1.0, 10)).is_err());
    let mut s = stream(Capacity::rows(10), false);
    let other = batch(&self::schema(true), 0, 2);
    assert!(matches!(
        s.append(other),
        Err(DataError::InvalidStream { .. })
    ));
    assert_eq!(
        DataError::InvalidStream { reason: "x".into() }
            .to_diagnostic()
            .code
            .as_str(),
        "SNR-DATA-070"
    );
}

#[test]
fn provenance_columns_survive_in_canonical_form() {
    let mut s = stream(Capacity::rows(8), true);
    let schema = schema(true);
    s.append(batch(&schema, 0, 6)).unwrap();
    s.append(batch(&schema, 6, 6)).unwrap();
    let table = s.snapshot().to_table(TableId(1)).unwrap();
    assert_eq!(table.num_rows(), 8);
    let provenance = RowProvenance::read(&table).unwrap();
    // Rows 4..12 kept; in each batch every third row (0, 3 of batch 2: rows 6, 9) is an
    // estimate with confidence 0.5. Kept estimates: rows 6 and 9 (and none of 0, 3).
    assert_eq!(provenance.mix().estimated(), 2);
    let prov = table.schema().field_with_name(PROV_COLUMN).unwrap();
    assert!(matches!(prov.data_type(), DataType::Dictionary(..)));
}

proptest! {
    /// Memory stays bounded, the rows kept are always the newest contiguous run, and no row
    /// is ever lost in the middle: a reader that keeps up sees every row exactly once.
    #[test]
    fn stream_rows_stay_bounded_and_contiguous(
        limit in 1_u64..40,
        sizes in proptest::collection::vec(0_usize..30, 1..40),
        read_every in 1_usize..5,
    ) {
        let schema = schema(false);
        let mut s = stream(Capacity::rows(limit), false);
        let mut cursor = 0_u64;
        let mut seen = 0_u64;
        let mut skipped = 0_u64;
        let mut appended = 0_u64;
        // A read covers [cursor + skipped, next): contiguous, starting where the reader
        // left off unless rows it never saw were dropped, which it is told.
        let mut read = |s: &StreamTable, cursor: &mut u64| -> Result<(), TestCaseError> {
            let snapshot = s.since(*cursor);
            prop_assert_eq!(snapshot.first_seq(), *cursor + snapshot.skipped());
            let got = values(&snapshot);
            #[expect(clippy::cast_precision_loss, reason = "small test counts")]
            let expected: Vec<f64> = (snapshot.first_seq()..snapshot.next_seq()).map(|v| v as f64).collect();
            prop_assert_eq!(got, expected);
            seen += snapshot.num_rows();
            skipped += snapshot.skipped();
            *cursor = snapshot.next_seq();
            Ok(())
        };
        for (step, n) in sizes.iter().enumerate() {
            s.append(batch(&schema, appended, *n)).unwrap();
            appended += *n as u64;
            prop_assert!(s.len() <= limit);
            prop_assert_eq!(s.next_seq() - s.first_seq(), s.len());
            prop_assert_eq!(s.next_seq(), appended);
            // The kept rows are exactly the newest ones, in order.
            let kept = values(&s.snapshot());
            #[expect(clippy::cast_precision_loss, reason = "small test counts")]
            let expected: Vec<f64> = (s.first_seq()..s.next_seq()).map(|v| v as f64).collect();
            prop_assert_eq!(kept, expected);
            if step % read_every == 0 {
                read(&s, &mut cursor)?;
            }
        }
        read(&s, &mut cursor)?;
        // Every row was either seen exactly once or reported as skipped.
        prop_assert_eq!(seen + skipped, appended);
    }

    /// A snapshot never changes, whatever is appended after it.
    #[test]
    fn snapshots_are_consistent(
        limit in 1_u64..20,
        first in 0_usize..20,
        later in proptest::collection::vec(0_usize..20, 0..10),
    ) {
        let schema = schema(true);
        let mut s = stream(Capacity::rows(limit), true);
        s.append(batch(&schema, 0, first)).unwrap();
        let snapshot = s.snapshot();
        let before = values(&snapshot);
        let mut n = first as u64;
        for size in later {
            s.append(batch(&schema, n, size)).unwrap();
            n += size as u64;
        }
        prop_assert_eq!(values(&snapshot), before);
        prop_assert_eq!(snapshot.num_rows(), snapshot.batches().iter().map(|b| b.num_rows() as u64).sum::<u64>());
    }
}
