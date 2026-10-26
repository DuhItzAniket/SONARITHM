// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the Office chart-part reader (C01-T05) without a zip container around it, so the
//! fuzzer reaches series, caches, literals, multi-level categories and axis titles
//! directly. Every value it returns is `exact`, and every error a registered code.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_core::ProvenanceClass;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::fuzzing::office_chart_part;

fuzz_target!(|data: &[u8]| {
    let mut budgets = Budgets::DEFAULT;
    budgets.max_nodes = 200_000;
    budgets.max_points = 200_000;
    budgets.max_work = 100_000_000;
    match office_chart_part(data, &budgets) {
        Ok(chart) => {
            assert!(chart.points() <= 200_000);
            for series in &chart.series {
                assert_eq!(series.provenance.class, ProvenanceClass::Exact);
            }
        }
        Err(diagnostic) => assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3")),
    }
});
