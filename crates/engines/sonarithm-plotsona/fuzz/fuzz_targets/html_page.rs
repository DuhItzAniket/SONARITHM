// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the HTML script scanner and the JavaScript literal reader (C01-T06): a page in,
//! a chart or a registered code out, and nothing ever runs.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::fuzzing::html_page;

fuzz_target!(|data: &[u8]| {
    let mut budgets = Budgets::DEFAULT;
    budgets.max_points = 200_000;
    budgets.max_work = 100_000_000;
    match html_page(data, &budgets) {
        Ok(chart) => assert!(chart.points() <= 200_000),
        Err(diagnostic) => assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3")),
    }
});
