// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the vector-PDF tier (C01-T08): PLOTSONA's own object reader (objects, object
//! streams, filters, references), the content-stream interpreter (graphics and text
//! state, fonts, `ToUnicode` maps, Form XObjects) and the shared vector reader. Every
//! value it returns is `vector`, and every error a registered code.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_core::ProvenanceClass;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::fuzzing::pdf_chart;

fuzz_target!(|data: &[u8]| {
    let mut budgets = Budgets::DEFAULT;
    budgets.max_nodes = 100_000;
    budgets.max_points = 200_000;
    budgets.max_part_bytes = 8 << 20;
    budgets.max_inflated_bytes = 32 << 20;
    budgets.max_work = 100_000_000;
    match pdf_chart(data, &budgets) {
        Ok(chart) => {
            for series in &chart.series {
                assert_eq!(series.provenance.class, ProvenanceClass::Vector);
            }
        }
        Err(diagnostic) => assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3")),
    }
});
