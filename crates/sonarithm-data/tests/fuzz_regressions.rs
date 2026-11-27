// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Inputs the nightly fuzzers once failed on stay plain results, run with no panic hook:
//! the committed `seed-regression-*` files of the `columnar_read` corpus must be refused
//! (F-169), and its `seed-check-*` files must read and pass the fuzz target's checks
//! (F-241). Their first byte picks the format, as in the fuzz target.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use sonarithm_core::Sha256Digest;
use sonarithm_data::source::write_ipc;
use sonarithm_data::{
    DataError, Format, Limits, ReadOptions, Table, read_bytes, table_fingerprint,
};

/// The fuzz target's options: limits small enough that no input exhausts memory.
fn options() -> ReadOptions {
    let mut limits = Limits::DEFAULT;
    limits.max_bytes = 4 << 20;
    limits.max_rows = 200_000;
    limits.max_fields = 512;
    limits.max_cells = 2_000_000;
    ReadOptions::new().with_limits(limits)
}

/// Each corpus file named `prefix*`, with its name, format and file bytes.
fn seeds(prefix: &str) -> Vec<(String, Format, Vec<u8>)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/columnar_read");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(prefix) {
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
        found.push((name, format, file.to_vec()));
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(!found.is_empty(), "no {prefix} seeds found");
    found
}

#[test]
fn columnar_regression_seeds_are_refused_without_running_away() {
    for (name, format, file) in seeds("seed-regression-") {
        let read = read_bytes(&file, format, "fuzz.bin", &options());
        assert!(read.is_err(), "{name} should be refused");
    }
}

/// A table's fingerprint, or `None` for a type RFC-0008 §3 does not encode (an interval:
/// `SNR-DATA-007`, as designed). Any other error fails the test.
fn fingerprint(name: &str, table: &Table) -> Option<Sha256Digest> {
    match table_fingerprint(table) {
        Ok(digest) => Some(digest),
        Err(DataError::Unfingerprintable { .. }) => None,
        Err(error) => panic!("{name}: no fingerprint: {error}"),
    }
}

/// The fuzz target's `check`: a loaded table has a fingerprint unless one of its types has no
/// encoding, survives an Arrow IPC round trip unchanged, and keeps its fingerprint (or its
/// refusal).
#[test]
fn columnar_check_seeds_read_round_trip_and_keep_their_fingerprint() {
    for (name, format, file) in seeds("seed-check-") {
        let loaded = read_bytes(&file, format, "fuzz.bin", &options())
            .unwrap_or_else(|error| panic!("{name} should read: {error}"));
        let table = loaded.bundle.primary().unwrap();
        let before = fingerprint(&name, table);
        let bytes = write_ipc(table).unwrap();
        let again = read_bytes(
            &bytes,
            Format::Arrow,
            "fuzz.arrow",
            &options().without_inference(),
        )
        .unwrap();
        let reread = again.bundle.primary().unwrap();
        assert_eq!(
            reread.schema(),
            table.schema(),
            "{name}: the schema round-trips"
        );
        assert_eq!(
            reread.to_batch().unwrap(),
            table.to_batch().unwrap(),
            "{name}: the data round-trips"
        );
        assert_eq!(
            fingerprint(&name, reread),
            before,
            "{name}: the fingerprint is stable"
        );
    }
}
