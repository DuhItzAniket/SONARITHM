// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the SVG tier (C01-T07): the scene walk (transforms, styles, path data), axis
//! finding, calibration and mark reading. Every value it returns is `vector`, and every
//! error a registered code.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_core::ProvenanceClass;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::fuzzing::svg_chart;

fuzz_target!(|data: &[u8]| {
    let mut budgets = Budgets::DEFAULT;
    budgets.max_nodes = 200_000;
    budgets.max_points = 200_000;
    budgets.max_work = 100_000_000;
    match svg_chart(data, &budgets) {
        Ok(chart) => {
            for series in &chart.series {
                assert_eq!(series.provenance.class, ProvenanceClass::Vector);
            }
        }
        Err(diagnostic) => assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3")),
    }
});
