// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

use serde_json::json;
use sonarithm_core::SpecPointer;
use sonarithm_core::engine::{Budget, CancelToken, CompileContext, Registry};

use super::*;

fn context() -> CompileContext {
    CompileContext::new(7, Budget::default(), CancelToken::new())
}

fn chart_source(path: &str) -> SpecFragment {
    SpecFragment::new(
        SpecPointer::root().key("data").key("source"),
        &json!({"kind": "chart", "path": path}),
    )
    .unwrap()
}

/// A directory of its own for one test, with `files` written into it.
fn directory(test: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plotsona-{test}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (name, bytes) in files {
        std::fs::write(dir.join(name), bytes).unwrap();
    }
    dir
}

fn ingest(engine: &Plotsona, path: &str) -> Diagnostic {
    let mut registry = Registry::<DataBundle>::new();
    registry.add_engine(engine).unwrap();
    let (owner, source) = registry.source(SourceKind::Chart).unwrap();
    assert_eq!(owner, &ENGINE);
    let Err(CompileError::Diagnostic(diagnostic)) =
        source.ingest(&chart_source(path), &mut context())
    else {
        unreachable!("expected a diagnostic");
    };
    *diagnostic
}

#[test]
fn a_host_without_file_access_reads_no_chart() {
    let diagnostic = ingest(&Plotsona::new(), "chart.svg");
    assert_eq!(diagnostic.code.as_str(), "SNR-ENG-312");
    assert_eq!(
        diagnostic.location.as_ref().map(SpecPointer::as_str),
        Some("/data/source/path")
    );
}

#[test]
fn the_chart_source_reports_unknown_and_oversized_files_at_the_path() {
    let dir = directory(
        "source",
        &[("notes.txt", b"not a chart"), ("big.svg", &[b' '; 2048])],
    );
    let engine = Plotsona::with_base_dir(&dir);
    assert_eq!(ingest(&engine, "notes.txt").code.as_str(), "SNR-ENG-300");
    assert_eq!(ingest(&engine, "missing.svg").code.as_str(), "SNR-ENG-312");
    let mut budgets = Budgets::DEFAULT;
    budgets.max_bytes = 1024;
    let small = Plotsona::with_base_dir(&dir).with_budgets(budgets);
    let diagnostic = ingest(&small, "big.svg");
    assert_eq!(diagnostic.code.as_str(), "SNR-ENG-301");
    assert_eq!(
        diagnostic.location.as_ref().map(SpecPointer::as_str),
        Some("/data/source/path")
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn a_named_extractor_must_fit_the_container() {
    let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
    let options = ExtractOptions::new().with_extractor(Extractor::Office);
    let err = extract(svg, "chart.svg", &options).expect_err("mismatch");
    assert_eq!(err.code.as_str(), "SNR-ENG-310");
    let png = b"\x89PNG\r\n\x1A\n\0\0\0\rIHDR";
    let err = extract(png, "chart.png", &ExtractOptions::new()).expect_err("raster");
    assert_eq!(err.code.as_str(), "SNR-ENG-308");
}

#[test]
fn every_code_is_in_the_plotsona_block() {
    for code in codes::CODES.codes {
        let number: u32 = code
            .code
            .strip_prefix("SNR-ENG-")
            .and_then(|n| n.parse().ok())
            .unwrap();
        assert!((300..400).contains(&number), "{}", code.code);
    }
}

#[test]
fn the_manifest_owns_the_chart_source_key() {
    let manifest = Plotsona::manifest();
    assert_eq!(manifest.id, ENGINE);
    assert_eq!(manifest.spec_keys, ["data.source.kind=chart"]);
}

fn code(bytes: &[u8]) -> String {
    extract(bytes, "fixture", &ExtractOptions::new())
        .map_or_else(|d| d.code.as_str().to_owned(), |_| "ok".to_owned())
}

/// Hostile files fail within budget, through the whole extraction path (C01-T03).
#[test]
fn hostile_fixtures_fail_within_budget_with_registered_codes() {
    use crate::container::tests::package;

    let zeros = vec![0_u8; 8 << 20];
    let bomb = package(&[
        ("xl/workbook.xml", b"<workbook/>"),
        ("xl/charts/chart1.xml", &zeros),
    ]);
    assert_eq!(code(&bomb), "SNR-ENG-302");

    let laughs = br#"<?xml version="1.0"?>
<!DOCTYPE svg [<!ENTITY a "aaaaaaaaaa"><!ENTITY b "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;">]>
<svg xmlns="http://www.w3.org/2000/svg"><text>&b;</text></svg>"#;
    assert_eq!(code(laughs), "SNR-ENG-303");

    let entity_in_chart = package(&[
        ("ppt/presentation.xml", b"<p/>"),
        (
            "ppt/charts/chart2.xml",
            b"<!DOCTYPE c [<!ENTITY x SYSTEM \"http://example.invalid/x\">]><c>&x;</c>",
        ),
    ]);
    assert_eq!(code(&entity_in_chart), "SNR-ENG-303");

    let mut budgets = Budgets::DEFAULT;
    budgets.max_bytes = 64;
    let options = ExtractOptions::new().with_budgets(budgets);
    let oversized = vec![b' '; 65];
    assert_eq!(
        extract(&oversized, "big.svg", &options)
            .err()
            .map(|d| d.code.as_str().to_owned()),
        Some("SNR-ENG-301".to_owned())
    );
}

#[test]
fn office_packages_without_charts_have_no_chart() {
    use crate::container::tests::package;

    let empty = package(&[("word/document.xml", b"<d/>")]);
    assert_eq!(code(&empty), "SNR-ENG-309");
}
