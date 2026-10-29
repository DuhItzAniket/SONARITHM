// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Inputs the nightly fuzzers once failed on stay plain results: the committed
//! `seed-regression-*` files of the `columnar_read` corpus (F-169). Their first byte picks
//! the format, as in the fuzz target.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use sonarithm_data::{Format, Limits, ReadOptions, read_bytes};

#[test]
fn columnar_regression_seeds_are_refused_without_running_away() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/columnar_read");
    let mut limits = Limits::DEFAULT;
    limits.max_bytes = 4 << 20;
    limits.max_rows = 200_000;
    limits.max_fields = 512;
    limits.max_cells = 2_000_000;
    let options = ReadOptions::new().with_limits(limits);
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("seed-regression-") {
            continue;
        }
        let bytes = std::fs::read(entry.path()).unwrap();
        let Some((&switches, file)) = bytes.split_first() else {
            continue;
        };
        let format = if switches & 1 == 0 {
            Format::Parquet
        } else {
            Format::Arrow
        };
        let read = read_bytes(file, format, "fuzz.bin", &options);
        assert!(read.is_err(), "{name} should be refused");
        checked += 1;
    }
    assert!(checked >= 1, "no regression seeds found");
}
