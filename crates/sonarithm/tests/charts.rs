// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C01-T12: charts through the facade: `read_chart`, its report and summary sentence from
//! the shared catalogue, and the spec that plays it through its chart-type template.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::PathBuf;

use sonarithm::{Sonarithm, read_chart};
use sonarithm_i18n::{PSEUDO_LOCALE, check_pseudo};

fn corpus(folder: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engines/sonarithm-plotsona/tests/corpus")
        .join(folder)
        .join(name)
}

/// 10 §5: the summary counts the points by how they were obtained, and adds the median
/// confidence when values have one.
#[test]
fn a_chart_reports_its_provenance_mix_in_one_sentence() {
    let exact = read_chart(corpus("office", "xlsx-line.xlsx")).unwrap();
    assert_eq!(
        exact.summary(),
        "36 points: 36 exact, 0 calibrated, 0 estimated"
    );
    let vector = read_chart(corpus("svg", "vl-line.svg")).unwrap();
    let summary = vector.summary();
    assert!(
        summary.starts_with("36 points: 0 exact, 36 calibrated, 0 estimated, median confidence 0."),
        "{summary}"
    );
    assert_eq!(
        vector.report().locator,
        "vl-line.svg",
        "P8: the file name only"
    );
}

/// NFR-09: the summary is one catalogue message, so it passes the pseudo-locale check.
#[test]
fn the_summary_comes_from_the_catalogue() {
    for (folder, name) in [("office", "xlsx-line.xlsx"), ("pdf", "plotly-line.pdf")] {
        let chart = read_chart(corpus(folder, name)).unwrap();
        let text = chart.summary_in(PSEUDO_LOCALE).unwrap();
        check_pseudo(&text).unwrap_or_else(|issue| panic!("{name}: {issue}: {text}"));
    }
}

/// A chart becomes a spec with its chart-type template, which compiles with the default
/// engines (PLOTSONA reads the chart source; NUMISONA plays the template).
#[test]
fn a_chart_plays_through_its_template_with_the_default_engines() {
    let chart = read_chart(corpus("chartjson", "vl-grouped-bar.json")).unwrap();
    assert_eq!(chart.template(), Some("chart-bar"));
    let spec = chart.spec().unwrap();
    let compiled = Sonarithm::builder()
        .with_default_engines()
        .build()
        .unwrap()
        .compile(&spec)
        .unwrap_or_else(|e| panic!("{e}\n{:#?}", e.diagnostics()));
    assert_eq!(compiled.score().tracks[0].events.len(), 12);
    assert!(
        compiled
            .score()
            .header
            .engines
            .keys()
            .any(|e| e.as_str() == "plotsona")
    );
}

/// A file that is not a chart fails with PLOTSONA's code, naming only the file.
#[test]
fn a_non_chart_is_a_located_error() {
    let err = read_chart(corpus("office", "xlsx-line.truth.json")).unwrap_err();
    let diagnostics = err.diagnostics();
    let code = diagnostics.iter().next().unwrap().code.as_str().to_owned();
    assert!(code.starts_with("SNR-ENG-3"), "{code}");
}
