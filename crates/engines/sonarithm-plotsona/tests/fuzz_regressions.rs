// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Inputs the fuzzers once failed on stay plain results, run with no panic hook: each
//! `seed-regression-*` file of the `pdf_chart` corpus reads as the fuzz target requires,
//! a chart whose values are all `vector`, or an error with a PLOTSONA code.
//! `seed-regression-cid-widths-overflow` is a font whose `/W` array starts at a huge CID,
//! so the first CID plus an index overflowed `u32` (`pdf_chart` fuzz job on #517).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use sonarithm_core::ProvenanceClass;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::fuzzing::pdf_chart;

#[test]
fn pdf_regression_seeds_read_or_are_refused_with_a_code() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/pdf_chart");
    let mut budgets = Budgets::DEFAULT;
    budgets.max_nodes = 100_000;
    budgets.max_points = 200_000;
    budgets.max_part_bytes = 8 << 20;
    budgets.max_inflated_bytes = 32 << 20;
    budgets.max_work = 100_000_000;
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("seed-regression-"))
        .collect();
    names.sort();
    assert!(!names.is_empty(), "no regression seeds found");
    for name in names {
        let bytes = std::fs::read(dir.join(&name)).unwrap();
        match pdf_chart(&bytes, &budgets) {
            Ok(chart) => {
                for series in &chart.series {
                    assert_eq!(series.provenance.class, ProvenanceClass::Vector, "{name}");
                }
            }
            Err(diagnostic) => assert!(
                diagnostic.code.as_str().starts_with("SNR-ENG-3"),
                "{name}: {}",
                diagnostic.code.as_str()
            ),
        }
    }
}
