// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PB-05: CSV ingest of 1M rows × 5 columns in at most 1.5 s (12 §8, testing-strategy §9.1).
//!
//! The input is generated in memory from a fixed seed (a xorshift generator), so every run
//! reads the same 46 MB: an ISO-8601 timestamp, a float, an integer, a 4-level category and
//! a ratio per row. `ingest` is the budgeted path (sniffing, type inference, semantic
//! inference, provenance normalization); `ingest_without_inference` isolates the parser.
//!
//! Run with `cargo bench -p sonarithm-data --bench csv_ingest`.

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    missing_docs,
    reason = "benchmark harness: fixed synthetic input, failures abort the run"
)]

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sonarithm_data::{Format, ReadOptions, read_bytes};

/// Rows of the PB-05 input.
const ROWS: u64 = 1_000_000;

/// The PB-05 input: 1M rows × 5 columns of typical sensor data.
fn input() -> String {
    let mut csv = String::with_capacity(48 << 20);
    csv.push_str("time,temperature,count,station,ratio\n");
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for i in 0..ROWS {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let seconds = i * 60;
        let day = seconds / 86_400;
        let rest = seconds % 86_400;
        writeln!(
            csv,
            "2024-{:02}-{:02}T{:02}:{:02}:00Z,{:.3},{},{},{}",
            1 + (day / 28) % 12,
            1 + day % 28,
            rest / 3600,
            (rest / 60) % 60,
            (state % 40_000) as f64 / 1000.0 - 10.0,
            state % 1000,
            ["north", "south", "east", "west"][(state % 4) as usize],
            (state % 1_000_000) as f64 / 1e6,
        )
        .unwrap();
    }
    csv
}

fn ingest(c: &mut Criterion) {
    let csv = input();
    let mut group = c.benchmark_group("pb05_csv_ingest_1m_rows_5_columns");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    group.throughput(Throughput::Bytes(csv.len() as u64));
    let options = ReadOptions::new();
    group.bench_function("ingest", |b| {
        b.iter(|| {
            let loaded =
                read_bytes(black_box(csv.as_bytes()), Format::Csv, "pb05.csv", &options).unwrap();
            assert_eq!(loaded.bundle.primary().unwrap().num_rows(), ROWS);
            loaded
        });
    });
    let parser_only = ReadOptions::new().without_inference();
    group.bench_function("ingest_without_inference", |b| {
        b.iter(|| {
            read_bytes(
                black_box(csv.as_bytes()),
                Format::Csv,
                "pb05.csv",
                &parser_only,
            )
            .unwrap()
        });
    });
    group.finish();
}

criterion_group!(benches, ingest);
criterion_main!(benches);
