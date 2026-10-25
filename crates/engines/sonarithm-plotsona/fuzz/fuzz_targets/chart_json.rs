// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the chart-JSON readers (C01-T06): the first byte picks Vega-Lite, Vega, Plotly or
//! Highcharts, the rest is the document. Every reader sees every input, whatever its keys.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::fuzzing::chart_json;

fuzz_target!(|data: &[u8]| {
    let Some((&library, json)) = data.split_first() else {
        return;
    };
    let mut budgets = Budgets::DEFAULT;
    budgets.max_points = 200_000;
    budgets.max_work = 100_000_000;
    match chart_json(library, json, &budgets) {
        Ok(chart) => assert!(chart.points() <= 200_000),
        Err(diagnostic) => assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3")),
    }
});
