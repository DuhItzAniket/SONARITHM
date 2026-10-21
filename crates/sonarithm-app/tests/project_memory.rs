// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Embedded data is read in place (RFC-0002 §6, condition B; B05-T04): opening a project
//! never holds a copy of a Parquet member in memory.

#![allow(
    clippy::unwrap_used,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::fmt::Write as _;
use std::fs::File;

use sonarithm_app::project::{self, Project, ProjectDataset};
use sonarithm_app::testing::TempDir;
use sonarithm_data::source::read_exact_at;
use sonarithm_data::{Format, ReadOptions, read_bytes, read_parquet_at, table_fingerprint};

/// A table whose values do not compress, so its Parquet member is about its decoded size.
fn noisy_table(rows: u64) -> sonarithm_data::Table {
    let mut csv = String::from("day,temp\n");
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    for day in 0..rows {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let temp = f64::from(u32::try_from(state >> 40).unwrap()) / 7.0;
        writeln!(csv, "{day},{temp}").unwrap();
    }
    read_bytes(
        csv.as_bytes(),
        Format::Csv,
        "noise.csv",
        &ReadOptions::new(),
    )
    .unwrap()
    .bundle
    .primary()
    .unwrap()
    .clone()
}

#[test]
fn rfc_0002_condition_b_embedded_data_is_read_in_place() {
    let dir = TempDir::new("memory");
    let table = noisy_table(200_000);
    let mut project = Project::default();
    project.specs = vec![("main".to_owned(), "{}".to_owned())];
    project.active_spec = "main".to_owned();
    project.datasets.push(ProjectDataset::embedded(
        "d1",
        "noise.csv",
        table.clone(),
        table_fingerprint(&table).unwrap().to_string(),
    ));
    let path = dir.path().join("big.sonarithm");
    project::save(&project, &path, true).unwrap();

    // Where the stored member lies in the bundle.
    let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
    let entry = archive.by_name("data/d1.parquet").unwrap();
    let (start, len) = (entry.data_start().unwrap(), entry.size());
    drop(entry);
    let member = usize::try_from(len).unwrap();

    let options = ReadOptions::new();
    let in_place = {
        let file = File::open(&path).unwrap();
        allocation_counter::measure(|| {
            read_parquet_at(file, start, len, "noise.csv", &options).unwrap();
        })
    };
    let buffered = {
        let file = File::open(&path).unwrap();
        allocation_counter::measure(|| {
            let mut bytes = vec![0; member];
            read_exact_at(&file, &mut bytes, start).unwrap();
            read_bytes(&bytes, Format::Parquet, "noise.parquet", &options).unwrap();
        })
    };
    // The in-place read saves at least the member's size at its peak.
    assert!(
        in_place.bytes_max + len <= buffered.bytes_max,
        "in place {} B at peak, buffered {} B, member {len} B",
        in_place.bytes_max,
        buffered.bytes_max
    );

    // And the whole open still finds the data intact.
    let opened = project::open(&path, &mut || 1).unwrap();
    let dataset = &opened.project.datasets[0];
    assert_eq!(dataset.rows, 200_000);
}
