// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RFC-0008 §8: the scheme-1 (Arrow IPC bytes) and scheme-2 (logical values) data
//! fingerprints on the same table. `bundle_fingerprint` runs inside every compile, so it
//! counts against PB-04 (≤ 1 s compile for 1M rows).
//!
//! The input is 1M rows × 5 columns (f64, i64, a Utf8 nominal of 4 values, a
//! `Dictionary<UInt8, Utf8>` ordinal of 3, a microsecond timestamp) plus `__snr_prov`
//! (`Dictionary<UInt8, Utf8>`) and `__snr_conf` (`Float32`), generated from a fixed seed
//! (a xorshift generator, as `csv_ingest` does; `sonarithm-data` cannot depend on
//! NUMISONA's generators). Variants: one batch or 16, and 0 % or 10 % nulls in the five
//! data columns.
//!
//! Throughput is in bytes of logical data: 8 per number or timestamp, 4 per confidence, 1
//! per dictionary key and the UTF-8 length of each text value, for valid cells. Peak
//! allocated bytes during one call of each scheme are measured with `allocation-counter`
//! and printed before the timings (criterion has no place for them).
//!
//! Run with `cargo bench -p sonarithm-data --bench fingerprint`. Record results in the
//! benchmarks log against PB-04; no number is claimed here.

#![allow(
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    missing_docs,
    reason = "benchmark harness: fixed synthetic input, failures abort the run"
)]

use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

use arrow_array::types::UInt8Type;
use arrow_array::{
    ArrayRef, DictionaryArray, Float32Array, Float64Array, Int64Array, RecordBatch, StringArray,
    TimestampMicrosecondArray, UInt8Array,
};
use arrow_schema::{DataType, Field, Schema, TimeUnit};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use sonarithm_core::TableId;
use sonarithm_data::{Table, table_fingerprint, table_fingerprint_v1};

/// Rows of the input.
const ROWS: usize = 1_000_000;

/// A fixed-seed xorshift stream.
struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// The input table in `batches` batches, with nulls in a tenth of the data cells when
/// `nulls` is set, and its size in logical bytes.
fn input(batches: usize, nulls: bool) -> (Table, u64) {
    let mut random = Xorshift(0x9E37_79B9_7F4A_7C15);
    let mut logical = 0_u64;
    let mut valid = |random: &mut Xorshift| !nulls || random.next() % 10 != 0;
    let mut values = Vec::with_capacity(ROWS);
    let mut counts = Vec::with_capacity(ROWS);
    let mut stations = Vec::with_capacity(ROWS);
    let mut levels = Vec::with_capacity(ROWS);
    let mut times = Vec::with_capacity(ROWS);
    let names = ["north", "south", "east", "west"];
    for row in 0..ROWS {
        let state = random.next();
        values.push(valid(&mut random).then(|| (state % 40_000) as f64 / 1000.0 - 10.0));
        counts.push(valid(&mut random).then(|| (state % 1000) as i64));
        stations.push(valid(&mut random).then(|| names[(state % 4) as usize]));
        levels.push(valid(&mut random).then(|| (state % 3) as u8));
        times.push(valid(&mut random).then(|| 1_704_067_200_000_000 + row as i64 * 60_000_000));
    }
    logical += 8 * values.iter().flatten().count() as u64;
    logical += 8 * counts.iter().flatten().count() as u64;
    logical += stations.iter().flatten().map(|s| s.len() as u64).sum::<u64>();
    logical += levels.iter().flatten().count() as u64;
    logical += 8 * times.iter().flatten().count() as u64;
    logical += (1 + 4) * ROWS as u64;
    let level_values: ArrayRef = Arc::new(StringArray::from(vec!["low", "medium", "high"]));
    let class_values: ArrayRef = Arc::new(StringArray::from(vec!["exact", "estimate"]));
    let classes: Vec<u8> = (0..ROWS).map(|row| u8::from(row % 7 == 0)).collect();
    let confidence: Vec<f32> = (0..ROWS).map(|row| if row % 7 == 0 { 0.8 } else { 1.0 }).collect();
    let dictionary = |values: &ArrayRef| {
        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(values.data_type().clone()))
    };
    let schema = Arc::new(Schema::new(vec![
        Field::new("value", DataType::Float64, true),
        Field::new("count", DataType::Int64, true),
        Field::new("station", DataType::Utf8, true),
        Field::new("level", dictionary(&level_values), true),
        Field::new("time", DataType::Timestamp(TimeUnit::Microsecond, None), true),
        Field::new("__snr_prov", dictionary(&class_values), true),
        Field::new("__snr_conf", DataType::Float32, true),
    ]));
    let size = ROWS.div_ceil(batches);
    let parts = (0..ROWS)
        .step_by(size)
        .map(|start| {
            let end = (start + size).min(ROWS);
            let columns: Vec<ArrayRef> = vec![
                Arc::new(Float64Array::from(values[start..end].to_vec())),
                Arc::new(Int64Array::from(counts[start..end].to_vec())),
                Arc::new(StringArray::from(stations[start..end].to_vec())),
                Arc::new(
                    DictionaryArray::<UInt8Type>::try_new(
                        UInt8Array::from(levels[start..end].to_vec()),
                        Arc::clone(&level_values),
                    )
                    .unwrap(),
                ),
                Arc::new(TimestampMicrosecondArray::from(times[start..end].to_vec())),
                Arc::new(
                    DictionaryArray::<UInt8Type>::try_new(
                        UInt8Array::from(classes[start..end].to_vec()),
                        Arc::clone(&class_values),
                    )
                    .unwrap(),
                ),
                Arc::new(Float32Array::from(confidence[start..end].to_vec())),
            ];
            RecordBatch::try_new(Arc::clone(&schema), columns).unwrap()
        })
        .collect();
    let table = Table::new(TableId(0), "bench", schema, parts).unwrap();
    (table, logical)
}

/// The most heap bytes allocated at once while `run` runs (`allocation-counter`).
fn peak(run: impl FnOnce()) -> u64 {
    allocation_counter::measure(run).bytes_max
}

fn fingerprints(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    for (batches, nulls) in [(1, false), (16, false), (1, true), (16, true)] {
        let (table, logical) = input(batches, nulls);
        let label = format!("{batches}b_{}nulls", if nulls { 10 } else { 0 });
        let v1 = peak(|| {
            black_box(table_fingerprint_v1(&table).unwrap());
        });
        let v2 = peak(|| {
            black_box(table_fingerprint(&table).unwrap());
        });
        println!("peak allocated bytes {label}: v1_ipc {v1}, v2_values {v2}");
        group.throughput(Throughput::Bytes(logical));
        group.bench_with_input(BenchmarkId::new("v1_ipc", &label), &table, |b, table| {
            b.iter(|| table_fingerprint_v1(black_box(table)).unwrap());
        });
        group.bench_with_input(BenchmarkId::new("v2_values", &label), &table, |b, table| {
            b.iter(|| table_fingerprint(black_box(table)).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, fingerprints);
criterion_main!(benches);
