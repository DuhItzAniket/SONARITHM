// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the whole chart path: sniffing, container and XML budgets, every extractor and
//! the long table (C01-T03). Budgets are small enough that no input can exhaust the
//! fuzzer's memory; every error must be a registered PLOTSONA code.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_plotsona::{Budgets, ExtractOptions, extract};

fuzz_target!(|data: &[u8]| {
    let mut budgets = Budgets::DEFAULT;
    budgets.max_bytes = 4 << 20;
    budgets.max_part_bytes = 8 << 20;
    budgets.max_inflated_bytes = 32 << 20;
    budgets.max_nodes = 200_000;
    budgets.max_points = 200_000;
    budgets.max_work = 200_000_000;
    let options = ExtractOptions::new().with_budgets(budgets);
    match extract(data, "fuzz.chart", &options) {
        Ok(extraction) => {
            let rows = extraction.bundle.primary().map_or(0, |t| t.num_rows());
            assert_eq!(usize::try_from(rows).ok(), Some(extraction.report.points));
            assert!(extraction.report.points <= 200_000);
        }
        Err(diagnostic) => assert!(
            diagnostic.code.as_str().starts_with("SNR-ENG-3"),
            "unregistered code {}",
            diagnostic.code.as_str()
        ),
    }
});
