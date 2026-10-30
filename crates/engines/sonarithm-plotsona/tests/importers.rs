// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C01-T10: the Vega-Lite and MAIDR importers. Each example imports with its report and
//! compiles to a Score; the snapshot lists what each became, and two Scores are reviewed.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sonarithm_compiler::{Compiled, Compiler, Failure, Platform};
use sonarithm_core::engine::CancelToken;
use sonarithm_core::score::review::ReviewOptions;
use sonarithm_core::score::{EngineVersion, ScoreEvent};
use sonarithm_data::DataBundle;
use sonarithm_numisona::Numisona;
use sonarithm_plotsona::import::{Import, maidr, vegalite};
use sonarithm_plotsona::{Budgets, Plotsona};

fn examples(folder: &str) -> Vec<(PathBuf, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/import")
        .join(folder);
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| {
            Path::new(n)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        })
        .collect();
    names.sort();
    names.into_iter().map(|n| (dir.clone(), n)).collect()
}

fn compile(dir: &Path, spec: &Value) -> Compiled {
    let platform = Platform::with_base_dir(dir);
    let plotsona = Plotsona::with_base_dir(dir);
    let numisona = Numisona::new();
    let compiler: Compiler<DataBundle> = Compiler::new(&[&platform, &plotsona, &numisona]).unwrap();
    match compiler.compile(&spec.to_string(), &CancelToken::new()) {
        Ok(done) => done,
        Err(Failure::Invalid(diagnostics)) => panic!(
            "{spec:#}\n{}",
            diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
        Err(other) => panic!("{other:?}"),
    }
}

fn summary(name: &str, import: &Import, compiled: &Compiled) -> String {
    let notes: usize = compiled
        .score
        .tracks
        .iter()
        .flat_map(|t| &t.events)
        .filter(|e| matches!(e, ScoreEvent::Note(_)))
        .count();
    let design = import.spec.get("template").map_or_else(
        || "tracks written out".to_owned(),
        |t| {
            format!(
                "{} {}",
                t["id"],
                t.get("bind").map_or(String::new(), ToString::to_string)
            )
        },
    );
    let mut line = format!("{name}: {design}; {notes} notes");
    for entry in import.report.entries() {
        write!(line, "\n  - {entry}").unwrap();
    }
    line
}

fn run(
    folder: &str,
    importer: fn(&[u8], &Budgets) -> Result<Import, Box<sonarithm_core::Diagnostic>>,
) -> Vec<String> {
    let mut lines = Vec::new();
    for (dir, name) in examples(folder) {
        let bytes = std::fs::read(dir.join(&name)).unwrap();
        let import = importer(&bytes, &Budgets::DEFAULT).unwrap_or_else(|d| panic!("{name}: {d}"));
        assert_eq!(import.spec["specVersion"], "0.2", "{name}");
        let compiled = compile(&dir, &import.spec);
        assert!(!compiled.score.tracks.is_empty(), "{name}");
        // Every report entry becomes a located diagnostic.
        assert_eq!(
            import.report.diagnostics(folder).len(),
            import.report.entries().len()
        );
        lines.push(summary(&name, &import, &compiled));
    }
    lines
}

/// A Vega-Lite gallery subset (chart types of the gallery, written for SONARITHM) imports,
/// reports what it leaves out, and compiles.
#[test]
fn the_vega_lite_examples_import_and_compile() {
    let lines = run("vegalite", vegalite::import);
    assert!(lines.len() >= 14);
    insta::assert_snapshot!(lines.join("\n"));
}

/// MAIDR's layer types import, report what they leave out, and compile.
#[test]
fn the_maidr_examples_import_and_compile() {
    let lines = run("maidr", maidr::import);
    assert!(lines.len() >= 12);
    insta::assert_snapshot!(lines.join("\n"));
}

/// Score snapshots of one import from each format.
#[test]
fn imported_scores_are_reviewed() {
    for (folder, name, importer) in [
        (
            "vegalite",
            "multi-line.vl.json",
            vegalite::import as fn(&[u8], &Budgets) -> _,
        ),
        ("maidr", "dodged-bar.json", maidr::import),
    ] {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/import")
            .join(folder);
        let import = importer(&std::fs::read(dir.join(name)).unwrap(), &Budgets::DEFAULT).unwrap();
        let mut score = compile(&dir, &import.spec).score;
        for version in score.header.engines.values_mut() {
            *version = EngineVersion("*".to_owned());
        }
        insta::assert_snapshot!(
            format!("{folder}-{name}"),
            score.review_json(&ReviewOptions::default())
        );
    }
}
