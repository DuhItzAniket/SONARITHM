// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_json::json;

use super::*;

#[test]
fn kinds_and_values_read_as_charts_write_them() {
    assert_eq!(
        XKind::infer([json!(1), json!(2.5), json!(null)].iter()),
        XKind::Number
    );
    assert_eq!(
        XKind::infer([json!("2020-01-05"), json!("2020-02-05T10:00:00")].iter()),
        XKind::Time
    );
    assert_eq!(
        XKind::infer([json!("2019"), json!("2020")].iter()),
        XKind::Category
    );
    assert_eq!(XKind::infer([json!("a"), json!(1)].iter()), XKind::Category);
    assert_eq!(number(&json!("2.5")), Some(2.5));
    assert_eq!(label(&json!(3)), "3");
    assert_eq!(
        title(Some(&json!({"text": ["Two", "lines"]}))).as_deref(),
        Some("Two lines")
    );
    assert_eq!(field(&json!({"a": {"b": 4}}), "a.b"), Some(&json!(4)));
    assert_eq!(field(&json!({"a.b": 5}), "a.b"), Some(&json!(5)));
}

use crate::{ExtractOptions, extract};

fn run(bytes: &[u8]) -> Result<crate::Extraction, Box<Diagnostic>> {
    extract(bytes, "chart.json", &ExtractOptions::new())
}

fn codes(extraction: &crate::Extraction) -> Vec<&str> {
    extraction
        .report
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect()
}

/// Every library's example set extracts with zero error (C01-T06 acceptance): Vega-Lite,
/// Vega, Plotly (from plotly.py, typed arrays included) and Highcharts, as JSON files and
/// inside HTML pages.
#[test]
fn corpus_charts_extract_with_zero_error() {
    let checked = crate::corpus::check(&crate::corpus::folder("chartjson"), 0.0);
    assert_eq!(checked.len(), 24, "the chart-JSON corpus has 24 charts");
    for library in ["vl-", "vega-", "plotly-", "highcharts-"] {
        assert!(
            checked
                .iter()
                .filter(|(n, _)| n.starts_with(library))
                .count()
                >= 4,
            "{library}"
        );
    }
}

/// A remote `data.url` gives a diagnostic and nothing is fetched (C01-T06 acceptance;
/// `sonarithm-app`'s `chart_sources_stay_offline` test runs the same files under the
/// network-deny harness).
#[test]
fn remote_data_is_reported_not_fetched() {
    let spec = br#"{"$schema": "https://vega.github.io/schema/vega-lite/v6.json",
        "data": {"url": "https://example.invalid/cars.json"}, "mark": "point",
        "encoding": {"x": {"field": "a", "type": "quantitative"}, "y": {"field": "b", "type": "quantitative"}}}"#;
    let err = run(spec).unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-346");
    assert_eq!(err.severity, sonarithm_core::Severity::Error);
    assert!(err.message.contains("example.invalid"));
    let vega = br#"{"$schema": "https://vega.github.io/schema/vega/v6.json",
        "data": [{"name": "t", "url": "data/stocks.csv"}, {"name": "u", "values": [{"x": 1, "y": 2}]}],
        "marks": [{"type": "symbol", "from": {"data": "t"}, "encode": {"enter": {"x": {"field": "x"}, "y": {"field": "y"}}}},
                  {"type": "symbol", "from": {"data": "u"}, "encode": {"enter": {"x": {"field": "x"}, "y": {"field": "y"}}}}]}"#;
    let partial = run(vega).unwrap();
    assert_eq!(partial.chart.points(), 1);
    assert_eq!(codes(&partial), ["SNR-ENG-346"]);
}

/// Data that needs a script gives a diagnostic and nothing runs (C01-T06 acceptance).
#[test]
fn script_data_is_reported_not_run() {
    let page = b"<!DOCTYPE html><html><body><script>
        Highcharts.chart('c', { title: { text: 'Live' }, series: [{ name: 'A', data: getData() }] });
    </script></body></html>";
    let err = run(page).unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-347");
    let signal = br#"{"$schema": "https://vega.github.io/schema/vega-lite/v6.json",
        "data": {"values": {"signal": "rows"}}, "mark": "bar",
        "encoding": {"x": {"field": "a"}, "y": {"field": "b"}}}"#;
    assert_eq!(run(signal).unwrap_err().code.as_str(), "SNR-ENG-347");
    let generator = br#"{"$schema": "https://vega.github.io/schema/vega-lite/v6.json",
        "data": {"sequence": {"start": 0, "stop": 10, "as": "x"}}, "mark": "point",
        "encoding": {"x": {"field": "x"}, "y": {"field": "x"}}}"#;
    assert_eq!(run(generator).unwrap_err().code.as_str(), "SNR-ENG-347");
    let reference =
        br#"{"data": [{"type": "scatter", "xsrc": "a:1", "x": "grid:col1", "y": [1, 2]}]}"#;
    let read = run(reference).unwrap();
    assert_eq!(codes(&read), ["SNR-ENG-347"]);
}

#[test]
fn script_tags_with_src_are_never_loaded_and_pages_without_charts_say_so() {
    let page = br#"<html><head><script src="https://example.invalid/chart.js"></script></head>
        <body><p>No chart here.</p><script>console.log("hi");</script></body></html>"#;
    assert_eq!(run(page).unwrap_err().code.as_str(), "SNR-ENG-309");
    let json_block = br#"<html><body><script type="application/json">
        {"data": [{"type": "bar", "x": ["a", "b"], "y": [3, 4]}], "layout": {"barmode": "stack"}}
        </script></body></html>"#;
    let read = run(json_block).unwrap();
    assert_eq!(read.report.extractor, "sonarithm-plotsona/plotly");
    assert_eq!(read.chart.chart_type, Some(crate::ChartType::StackedBar));
}

#[test]
fn vega_lite_transforms_aggregates_and_folds() {
    let spec = br#"{"$schema": "https://vega.github.io/schema/vega-lite/v6.json",
        "data": {"values": [{"k": "a", "v": 1}, {"k": "a", "v": 3}, {"k": "b", "v": 5}]},
        "transform": [{"filter": "datum.v > 0"}],
        "mark": "bar", "encoding": {"x": {"field": "k", "type": "nominal"},
                                    "y": {"field": "v", "aggregate": "mean", "type": "quantitative"}}}"#;
    let read = run(spec).unwrap();
    assert_eq!(codes(&read), ["SNR-ENG-348"]);
    let series = &read.chart.series[0];
    assert_eq!(
        series.provenance.class,
        sonarithm_core::ProvenanceClass::Derived
    );
    assert_eq!(
        series.points.iter().map(|p| p.y).collect::<Vec<_>>(),
        [Some(2.0), Some(5.0)]
    );
    let wide = br#"{"$schema": "https://vega.github.io/schema/vega-lite/v6.json",
        "data": {"values": [{"t": 1, "a": 10, "b": 20}, {"t": 2, "a": 11, "b": 21}]},
        "transform": [{"fold": ["a", "b"], "as": ["series", "value"]}],
        "mark": "line", "encoding": {"x": {"field": "t", "type": "quantitative"},
                                     "y": {"field": "value", "type": "quantitative"},
                                     "color": {"field": "series"}}}"#;
    let read = run(wide).unwrap();
    assert_eq!(codes(&read), Vec::<&str>::new());
    let names: Vec<&str> = read.chart.series.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["a", "b"]);
    assert_eq!(read.chart.series[1].points[1].y, Some(21.0));
}

#[test]
fn plotly_templates_are_never_read_as_traces() {
    let figure = br#"{"data": [{"type": "bar", "x": ["a"], "y": [1]}],
        "layout": {"template": {"data": {"scatter": [{"type": "scatter", "x": [9], "y": [9]}]}}}}"#;
    let read = run(figure).unwrap();
    assert_eq!(read.chart.series.len(), 1);
    assert_eq!(read.chart.points(), 1);
}

/// Every input the fuzzers ever crashed on stays a plain result (F-168): the committed
/// `seed-regression-*` files of the chart-JSON and HTML fuzz corpora.
#[test]
fn fuzz_regressions_read_without_panicking() {
    let fuzz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus");
    let mut checked = 0;
    for target in ["html_page", "chart_json"] {
        let Ok(entries) = std::fs::read_dir(fuzz.join(target)) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("seed-regression-") {
                continue;
            }
            let bytes = std::fs::read(entry.path()).unwrap_or_default();
            let result = if target == "html_page" {
                crate::fuzzing::html_page(&bytes, &crate::Budgets::DEFAULT)
            } else {
                crate::fuzzing::chart_json(0, &bytes, &crate::Budgets::DEFAULT)
            };
            if let Err(diagnostic) = result {
                assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3"), "{name}");
            }
            checked += 1;
        }
    }
    assert!(checked >= 1, "no regression seeds found");
}
