// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A06-T09: the rule-conformance suite (testing-strategy §6) and the rule coverage report.
//!
//! The design-rules catalogue names an owner for every rule. This file holds the
//! conformance tests for the rules whose owner includes NUMISONA that other test files do
//! not already cover end to end, and `every_rule_numisona_owns_has_a_named_test` checks that
//! each owned rule has at least one test named after it (`sdr_08_…`, `a11y_13_…`) somewhere
//! in this crate. The coverage report is a snapshot of that mapping.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::json;
use sonarithm_data::DataBundle;
use sonarithm_numisona::examples;
use sonarithm_numisona::import::erie;
use support::{compile, has_code, render};

/// The rule IDs whose owner names NUMISONA in docs/research/design-rules.md.
fn owned_rules() -> Vec<String> {
    let catalogue = include_str!("../../../../docs/research/design-rules.md");
    catalogue
        .lines()
        .filter(|line| line.starts_with("| SDR-") || line.starts_with("| A11Y-"))
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let owner = cells.get(cells.len().saturating_sub(2))?;
            owner.contains("NUMISONA").then(|| cells[1].to_owned())
        })
        .collect()
}

/// Every test function in this crate's sources and tests, by name.
fn test_names() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names = Vec::new();
    let mut stack = vec![root.join("src"), root.join("tests")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                names.extend(
                    text.lines()
                        .filter_map(|line| line.trim().strip_prefix("fn "))
                        .filter_map(|rest| rest.split('(').next())
                        .map(str::to_owned),
                );
            }
        }
    }
    names.sort();
    names
}

/// The rule coverage report: every rule NUMISONA owns (in the catalogue's Owner column),
/// with the tests named after it. A rule without a test fails the suite.
#[test]
fn every_rule_numisona_owns_has_a_named_test() {
    let names = test_names();
    let mut report = Vec::new();
    let mut missing = Vec::new();
    // SDR-05 (Erie import) and SDR-15 (the polarity flip) are owned by the spec importers
    // and the profiles, but NUMISONA implements them, so they are covered here too.
    let mut rules = owned_rules();
    rules.extend(["SDR-05".to_owned(), "SDR-15".to_owned()]);
    for rule in rules {
        let prefix = format!("{}_", rule.to_lowercase().replace('-', "_"));
        let tests: Vec<&String> = names.iter().filter(|n| n.starts_with(&prefix)).collect();
        if tests.is_empty() {
            missing.push(rule.clone());
        }
        let list: Vec<&str> = tests.iter().map(|t| t.as_str()).collect();
        report.push(format!("{rule}: {}", list.join(", ")));
    }
    assert!(
        missing.is_empty(),
        "rules without a conformance test: {missing:?}"
    );
    insta::assert_snapshot!(report.join("\n"));
}

/// SDR-05 (SHOULD): Erie specs import and render. Every spec of the gallery subset compiles
/// with NUMISONA and renders to audible, finite audio; file sources get a synthetic stand-in.
#[test]
fn sdr_05_the_erie_gallery_subset_imports_and_renders() {
    let gallery = [
        include_str!("erie/time-series.json"),
        include_str!("erie/histogram.json"),
        include_str!("erie/sparsity-simple.json"),
        include_str!("erie/sparsity-and-length.json"),
        include_str!("erie/interval.json"),
        include_str!("erie/model-fit-overlay.json"),
        include_str!("erie/residuals-sequence.json"),
        include_str!("erie/frequency.json"),
    ];
    let mut rendered = 0;
    for text in gallery {
        let import = erie::import(text).unwrap();
        for spec in &import.specs {
            let compiled = compile(&spec.to_string(), Some(penguins()));
            let audio = render(&compiled.score, 4.0);
            let peak = audio.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
            assert!(audio.iter().all(|s| s.is_finite()));
            assert!(peak > 1e-3 && peak <= 1.0, "{spec}: peak {peak}");
            rendered += 1;
        }
    }
    assert_eq!(rendered, 9);
}

/// A stand-in for the penguins file the histogram fixture names.
fn penguins() -> DataBundle {
    let rows: Vec<serde_json::Value> = (0..60)
        .map(|i| json!({ "body_mass": 3000 + (i * 37) % 2500 }))
        .collect();
    let bytes = serde_json::to_vec(&rows).unwrap();
    sonarithm_data::read_bytes(
        &bytes,
        sonarithm_data::Format::Json,
        "penguins.json",
        &sonarithm_data::ReadOptions::new(),
    )
    .unwrap()
    .bundle
}

/// SDR-21 (SHOULD): volatile series are slowed or smoothed. NUMISONA counts the trend
/// reversals of the pitch field and, beyond those studied, advises smoothing or a slower
/// pace; a zigzag around a ramp gets the advice, and the 2-point moving average that makes
/// it monotone does not.
#[test]
fn sdr_21_volatile_series_get_smoothing_advice() {
    let values: Vec<serde_json::Value> = (0..20)
        .map(|i| json!({ "i": i, "y": f64::from(i) + if i % 2 == 0 { 0.8 } else { -0.8 } }))
        .collect();
    let spec = |transform: serde_json::Value| {
        json!({
            "specVersion": "0.1",
            "data": { "source": { "kind": "inline", "values": values } },
            "transform": transform,
            "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "y" } } }],
        })
        .to_string()
    };
    let raw = compile(&spec(json!([])), None);
    let advice = raw
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "SNR-ENG-012")
        .unwrap();
    assert!(advice.message.contains("18 times") && advice.rules[0].as_str() == "SDR-21");
    let smooth = json!([{ "op": "smooth", "field": "y", "method": "sma", "window": 2 }]);
    let smoothed = compile(&spec(smooth), None);
    assert!(!has_code(&smoothed, "SNR-ENG-012"));
    // The temperature example's daily noise is volatile too.
    let daily = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "file", "path": "temperature.json" } },
        "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }],
    });
    assert!(has_code(
        &compile(&daily.to_string(), Some(examples::temperature().unwrap())),
        "SNR-ENG-012"
    ));
}

/// The NUMISONA owners in the catalogue are the ones this suite expects; a catalogue
/// change shows up here.
#[test]
fn the_owned_rules_are_known() {
    let owned = owned_rules();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for rule in &owned {
        *counts
            .entry(if rule.starts_with("SDR") {
                "SDR"
            } else {
                "A11Y"
            })
            .or_default() += 1;
    }
    insta::assert_snapshot!(format!("{owned:?}"));
    assert!(counts["SDR"] >= 15);
}
