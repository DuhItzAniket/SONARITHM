// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A06-T07: every template renders every compatible example dataset, and templates are
//! data-agnostic designs (SDR-03).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use serde_json::{Value, json};
use sonarithm_numisona::examples;
use sonarithm_numisona::templates::{Bindings, TEMPLATES, template};
use support::{compile, render};

fn source(name: &str) -> Value {
    json!({ "kind": "file", "path": format!("{name}.json") })
}

/// A06-T07 acceptance: each template is applied to each example dataset it can bind, the
/// spec compiles without errors, and the Score renders to audible, finite, unclipped audio
/// (the first 4 s, through the render engine). The snapshot is the compatibility matrix with
/// the bindings and event counts.
#[test]
fn every_template_renders_every_compatible_example_dataset() {
    let datasets = examples::all().unwrap();
    let mut matrix = Vec::new();
    for template in &TEMPLATES {
        let mut compatible = 0;
        for (name, bundle) in &datasets {
            let table = bundle.primary().unwrap();
            let Ok(bindings) = template.bind(table) else {
                matrix.push(format!("{} × {name}: not compatible", template.id));
                continue;
            };
            compatible += 1;
            let spec = template.apply(bundle, &bindings, &source(name)).unwrap();
            let compiled = compile(&spec.to_string(), Some(bundle.clone()));
            let events: usize = compiled.score.tracks.iter().map(|t| t.events.len()).sum();
            assert!(events > 0, "{} × {name}", template.id);
            let audio = render(&compiled.score, 4.0);
            let peak = audio.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
            assert!(
                audio.iter().all(|s| s.is_finite()),
                "{} × {name}",
                template.id
            );
            assert!(
                peak > 1e-3 && peak <= 1.0,
                "{} × {name}: peak {peak}",
                template.id
            );
            matrix.push(format!(
                "{} × {name}: {bindings:?}, {events} events",
                template.id
            ));
        }
        assert!(
            compatible >= 2,
            "{} fits fewer than two example datasets",
            template.id
        );
    }
    insta::assert_snapshot!(matrix.join("\n"));
}

/// The template designs, as reviewed specs on one dataset each.
#[test]
fn template_specs_are_reviewed() {
    let cases = [
        ("line-overview", "temperature"),
        ("bar-discrete", "fruit"),
        ("scatter-xy", "scatter"),
        ("histogram", "waiting_times"),
        ("box-five-number", "reaction_times"),
        ("heatmap-rowscan", "grid"),
    ];
    let datasets = examples::all().unwrap();
    for (id, dataset) in cases {
        let bundle = &datasets.iter().find(|(n, _)| *n == dataset).unwrap().1;
        let template = template(id).unwrap();
        let bindings = template.bind(bundle.primary().unwrap()).unwrap();
        let mut spec = template.apply(bundle, &bindings, &source(dataset)).unwrap();
        // The box template's inline summary is long; its first group is enough to review.
        if let Some(values) = spec
            .pointer_mut("/data/source/values")
            .and_then(Value::as_array_mut)
        {
            values.truncate(5);
        }
        insta::assert_snapshot!(id, serde_json::to_string_pretty(&spec).unwrap());
    }
}

/// SDR-03 (MUST): a template is data-agnostic and reusable. Applied to two datasets with
/// compatible schemas, it gives the same design, differing only in the bound fields.
#[test]
fn sdr_03_templates_reapply_to_any_compatible_dataset() {
    let line = template("line-overview").unwrap();
    let datasets = examples::all().unwrap();
    let spec_for = |name: &str| {
        let bundle = &datasets.iter().find(|(n, _)| *n == name).unwrap().1;
        let bindings = line.bind(bundle.primary().unwrap()).unwrap();
        let spec = line.apply(bundle, &bindings, &source("data")).unwrap();
        (spec, bindings)
    };
    let (temperature, t) = spec_for("temperature");
    let (prices, p) = spec_for("prices");
    let rename = |spec: &Value, bindings: &Bindings| {
        let mut text = spec.to_string();
        for (slot, field) in bindings {
            text = text.replace(&format!("\"{field}\""), &format!("\"<{slot}>\""));
        }
        text
    };
    assert_eq!(rename(&temperature, &t), rename(&prices, &p));
    assert_ne!(t, p);
    // Explicit bindings are checked against the data.
    let table = datasets[0].1.primary().unwrap();
    let mut wrong = Bindings::new();
    wrong.insert("x".to_owned(), "nope".to_owned());
    wrong.insert("y".to_owned(), "temp".to_owned());
    assert!(line.check(table, &wrong).is_err());
}

/// The five-number summary is marked as derived data (03 §4) and ordered low to high.
#[test]
fn box_plots_play_five_derived_numbers_per_group() {
    let bundle = examples::reaction_times().unwrap();
    let boxes = template("box-five-number").unwrap();
    let bindings = boxes.bind(bundle.primary().unwrap()).unwrap();
    let spec = boxes
        .apply(&bundle, &bindings, &source("reaction"))
        .unwrap();
    let values = spec
        .pointer("/data/source/values")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(values.len(), 15);
    for group in values.chunks(5) {
        assert!(group.iter().all(|row| row["__snr_prov"] == "derived"));
        let numbers: Vec<f64> = group
            .iter()
            .map(|row| row["value"].as_f64().unwrap())
            .collect();
        assert!(numbers.windows(2).all(|pair| pair[0] <= pair[1]));
    }
    let compiled = compile(&spec.to_string(), None);
    assert_eq!(support::notes(&compiled.score).len(), 15);
}
