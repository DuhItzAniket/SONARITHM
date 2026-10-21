// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Parquet read in place from part of a file (RFC-0002 §6, B05-T04).

#![cfg(feature = "fs")]
#![allow(
    clippy::unwrap_used,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::io::Write as _;

use sonarithm_data::source::write_parquet;
use sonarithm_data::{Format, ReadOptions, read_bytes, read_parquet_at, table_fingerprint};

#[test]
fn a_parquet_region_reads_like_the_whole_file() {
    let csv = b"day,temp\n1,8.5\n2,9.25\n3,7.0\n";
    let table = read_bytes(csv, Format::Csv, "t.csv", &ReadOptions::new())
        .unwrap()
        .bundle
        .primary()
        .unwrap()
        .clone();
    let parquet = write_parquet(&table).unwrap();
    let expected = read_bytes(&parquet, Format::Parquet, "t.parquet", &ReadOptions::new()).unwrap();

    // The member sits between other bytes, as in a ZIP.
    let path = std::env::temp_dir().join(format!("sonarithm-region-{}.bin", std::process::id()));
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(&[7; 1000]).unwrap();
    file.write_all(&parquet).unwrap();
    file.write_all(&[9; 333]).unwrap();
    drop(file);

    let file = std::fs::File::open(&path).unwrap();
    let loaded = read_parquet_at(
        file,
        1000,
        parquet.len() as u64,
        "t.parquet",
        &ReadOptions::new(),
    )
    .unwrap();
    let got = loaded.bundle.primary().unwrap();
    assert_eq!(
        table_fingerprint(got).unwrap(),
        table_fingerprint(expected.bundle.primary().unwrap()).unwrap()
    );
    // A region that cuts the file short is refused, not read past.
    let file = std::fs::File::open(&path).unwrap();
    assert!(
        read_parquet_at(
            file,
            1000,
            (parquet.len() - 10) as u64,
            "t.parquet",
            &ReadOptions::new()
        )
        .is_err()
    );
    let _ = std::fs::remove_file(&path);
}
