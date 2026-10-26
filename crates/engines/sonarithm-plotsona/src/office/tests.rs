// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

use serde_json::Value;
use sonarithm_core::ProvenanceClass;

use super::*;
use crate::calibrate::ticks::parse_date;
use crate::container::tests::package;
use crate::{ExtractOptions, extract};

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/office")
}

/// Every Office chart of the corpus extracts with zero error, with `exact` provenance, in
/// XLSX, PPTX and DOCX (C01-T05 acceptance).
#[test]
fn corpus_charts_extract_with_zero_error() {
    let mut checked = 0;
    let mut files: Vec<PathBuf> = std::fs::read_dir(corpus())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".truth.json"))
        .collect();
    files.sort();
    for truth_path in files {
        let name = truth_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace(".truth.json", "");
        let kind = name.split('-').next().unwrap();
        let chart_path = corpus().join(format!("{name}.{kind}"));
        let truth: Value =
            serde_json::from_str(&std::fs::read_to_string(&truth_path).unwrap()).unwrap();
        let bytes = std::fs::read(&chart_path).unwrap();
        let extraction = extract(&bytes, &name, &ExtractOptions::new());
        assert!(extraction.is_ok(), "{name}: {:?}", extraction.err());
        let extraction = extraction.unwrap();
        let chart = &extraction.chart;
        let expected_type = truth["chart_type"].as_str().unwrap();
        let found_type = serde_json::to_value(chart.chart_type).unwrap();
        assert_eq!(
            found_type.as_str(),
            Some(expected_type),
            "{name}: chart type"
        );
        assert_eq!(
            chart.title.as_deref(),
            truth["title"].as_str(),
            "{name}: title"
        );
        let series = truth["series"].as_array().unwrap();
        assert_eq!(chart.series.len(), series.len(), "{name}: series count");
        for (found, expected) in chart.series.iter().zip(series) {
            assert_eq!(found.name, expected["name"].as_str().unwrap(), "{name}");
            assert_eq!(found.provenance.class, ProvenanceClass::Exact, "{name}");
            let ys: Vec<f64> = expected["y"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let found_ys: Vec<f64> = found.points.iter().map(|p| p.y.unwrap()).collect();
            assert_eq!(found_ys, ys, "{name}: values, zero error");
            let xs = expected["x"].as_array().unwrap();
            for (point, x) in found.points.iter().zip(xs) {
                let expected_x = match truth["x_kind"].as_str().unwrap() {
                    "number" => XValue::Number(x.as_f64().unwrap()),
                    "date" => XValue::Time(parse_date(x.as_str().unwrap()).unwrap()),
                    _ => XValue::Category(x.as_str().unwrap().to_owned()),
                };
                assert_eq!(point.x, expected_x, "{name}: x");
            }
        }
        assert!(
            extraction
                .report
                .diagnostics
                .iter()
                .all(|d| d.severity != sonarithm_core::Severity::Warning),
            "{name}: {:?}",
            extraction.report.diagnostics
        );
        checked += 1;
    }
    assert_eq!(checked, 22, "the Office corpus has 22 charts");
}

#[test]
fn the_workbook_is_preferred_and_reported() {
    let bytes = std::fs::read(corpus().join("pptx-line.pptx")).unwrap();
    let extraction = extract(&bytes, "pptx-line.pptx", &ExtractOptions::new()).unwrap();
    let codes: Vec<&str> = extraction
        .report
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["SNR-ENG-344"]);
    assert_eq!(extraction.report.extractor, "sonarithm-plotsona/office");
}

/// A chart part as Office writes it, with `series` as (name, categories, values) caches.
fn chart_xml(group: &str, series: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<c:chart><c:plotArea><c:{group}>{series}</c:{group}></c:plotArea></c:chart></c:chartSpace>"#
    )
}

fn xlsx_with(chart: &str) -> Vec<u8> {
    package(&[
        ("xl/workbook.xml", b"<workbook/>"),
        ("xl/charts/chart1.xml", chart.as_bytes()),
    ])
}

fn run(bytes: &[u8]) -> Result<crate::Extraction, Box<Diagnostic>> {
    extract(bytes, "test.xlsx", &ExtractOptions::new())
}

#[test]
fn a_chart_without_caches_gives_a_diagnostic() {
    let series = r"<c:ser><c:tx><c:v>A</c:v></c:tx><c:val><c:numRef><c:f>Sheet1!$B$2:$B$4</c:f></c:numRef></c:val></c:ser>";
    let err = run(&xlsx_with(&chart_xml("lineChart", series))).unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-341");
    assert!(err.message.contains("`A`"));
}

#[test]
fn literals_gaps_and_inline_names_are_read() {
    let series = r#"<c:ser><c:tx><c:v>Inline</c:v></c:tx>
<c:cat><c:strLit><c:ptCount val="3"/><c:pt idx="0"><c:v>a</c:v></c:pt><c:pt idx="2"><c:v>c</c:v></c:pt></c:strLit></c:cat>
<c:val><c:numLit><c:formatCode>General</c:formatCode><c:ptCount val="3"/><c:pt idx="0"><c:v>1.5</c:v></c:pt><c:pt idx="2"><c:v>-2</c:v></c:pt></c:numLit></c:val></c:ser>"#;
    let extraction = run(&xlsx_with(&chart_xml("barChart", series))).unwrap();
    let s = extraction.chart.series.first().unwrap();
    assert_eq!(s.name, "Inline");
    assert_eq!(
        s.points,
        [
            Point::category("a", Some(1.5)),
            Point::category("", None),
            Point::category("c", Some(-2.0)),
        ]
    );
    // The workbook in this package is not readable, so the caches were used and said so.
    let codes: Vec<&str> = extraction
        .report
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["SNR-ENG-343"]);
    assert_eq!(extraction.report.series.first().map(|s| s.missing), Some(1));
}

#[test]
fn multi_level_categories_and_1904_dates() {
    let multi = r#"<c:ser><c:cat><c:multiLvlStrRef><c:multiLvlStrCache><c:ptCount val="3"/>
<c:lvl><c:pt idx="0"><c:v>Q1</c:v></c:pt><c:pt idx="1"><c:v>Q2</c:v></c:pt><c:pt idx="2"><c:v>Q1</c:v></c:pt></c:lvl>
<c:lvl><c:pt idx="0"><c:v>2025</c:v></c:pt><c:pt idx="2"><c:v>2026</c:v></c:pt></c:lvl>
</c:multiLvlStrCache></c:multiLvlStrRef></c:cat>
<c:val><c:numLit><c:ptCount val="3"/><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt><c:pt idx="2"><c:v>3</c:v></c:pt></c:numLit></c:val></c:ser>"#;
    let chart = run(&xlsx_with(&chart_xml("lineChart", multi)))
        .unwrap()
        .chart;
    let labels: Vec<&XValue> = chart.series[0].points.iter().map(|p| &p.x).collect();
    assert_eq!(
        labels,
        [
            &XValue::Category("2025 / Q1".into()),
            &XValue::Category("2025 / Q2".into()),
            &XValue::Category("2026 / Q1".into()),
        ]
    );
    let dated = r#"<c:ser><c:cat><c:numLit><c:formatCode>d-mmm-yy</c:formatCode><c:ptCount val="1"/><c:pt idx="0"><c:v>0</c:v></c:pt></c:numLit></c:cat>
<c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>7</c:v></c:pt></c:numLit></c:val></c:ser>"#;
    let xml =
        chart_xml("lineChart", dated).replace("<c:chart>", r#"<c:date1904 val="1"/><c:chart>"#);
    let chart = run(&xlsx_with(&xml)).unwrap().chart;
    assert_eq!(
        chart.series[0].points[0].x,
        XValue::Time(parse_date("1904-01-01").unwrap())
    );
}

#[test]
fn an_external_data_link_is_not_followed() {
    let chart = chart_xml(
        "lineChart",
        r#"<c:ser><c:val><c:numRef><c:f>[1]Sheet1!$B$2</c:f><c:numCache><c:ptCount val="1"/><c:pt idx="0"><c:v>4</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#,
    )
    .replace("</c:chartSpace>", r#"<c:externalData r:id="rId1"/></c:chartSpace>"#);
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/oleObject" Target="file:///C:/secret/book.xlsx" TargetMode="External"/></Relationships>"#;
    let bytes = package(&[
        ("ppt/presentation.xml", b"<p/>"),
        ("ppt/charts/chart1.xml", chart.as_bytes()),
        ("ppt/charts/_rels/chart1.xml.rels", rels.as_bytes()),
    ]);
    let extraction = run(&bytes).unwrap();
    assert_eq!(extraction.chart.series[0].points[0].y, Some(4.0));
    let codes: Vec<&str> = extraction
        .report
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["SNR-ENG-342"]);
}

#[test]
fn several_charts_read_the_first_by_number() {
    let one = chart_xml(
        "pieChart",
        r#"<c:ser><c:val><c:numLit><c:ptCount val="1"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val></c:ser>"#,
    );
    let bytes = package(&[
        ("xl/workbook.xml", b"<workbook/>"),
        ("xl/charts/chart10.xml", b"<c:chartSpace xmlns:c=\"c\"/>"),
        ("xl/charts/chart2.xml", one.as_bytes()),
    ]);
    let extraction = run(&bytes).unwrap();
    assert_eq!(extraction.chart.chart_type, Some(ChartType::Pie));
    assert!(
        extraction
            .report
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "SNR-ENG-340" && d.message.contains("chart2.xml"))
    );
    assert_eq!(
        chart_parts(&[
            "xl/charts/chart12.xml".to_owned(),
            "xl/charts/chartEx1.xml".to_owned(),
            "xl/charts/colors1.xml".to_owned(),
            "word/charts/chart1.xml".to_owned(),
            "xl/charts/chart.xml".to_owned(),
            "xl/charts/chart2.xml".to_owned(),
        ]),
        [
            "word/charts/chart1.xml",
            "xl/charts/chart2.xml",
            "xl/charts/chart12.xml"
        ]
    );
}

#[test]
fn a_huge_declared_point_count_fails_before_allocating() {
    let series =
        r#"<c:ser><c:val><c:numLit><c:ptCount val="999999999999"/></c:numLit></c:val></c:ser>"#;
    let err = run(&xlsx_with(&chart_xml("lineChart", series))).unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-306");
}

#[test]
fn formulas_targets_and_formats() {
    assert_eq!(
        parse_range("Sheet1!$B$2:$B$13"),
        Some(("Sheet1".to_owned(), (1, 1), (12, 1)))
    );
    assert_eq!(
        parse_range("'Q''s data'!AA10"),
        Some(("Q's data".to_owned(), (9, 26), (9, 26)))
    );
    for formula in [
        "[1]Sheet1!$A$1",
        "(Sheet1!A1,Sheet1!A3)",
        "MyName",
        "Sheet1!A0",
    ] {
        assert_eq!(parse_range(formula), None, "{formula}");
    }
    assert_eq!(
        resolve_target("ppt/charts", "../embeddings/book.xlsx").as_deref(),
        Some("ppt/embeddings/book.xlsx")
    );
    assert_eq!(resolve_target("a", "../../escape"), None);
    for (format, date) in [
        ("General", false),
        ("0.00", false),
        ("yyyy-mm-dd", true),
        ("d-mmm-yy", true),
        ("mmm", true),
        ("h:mm", false),
        ("\"day\" 0", false),
        ("[Red]0.0", false),
    ] {
        assert_eq!(is_date_format(format), date, "{format}");
    }
}
