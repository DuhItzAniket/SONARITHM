// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A06-T08 acceptance: the Erie gallery subset imports, every imported spec is valid spec
//! v0.1, and every construct that is not imported exactly is in the report (04 §9).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::fmt::Write as _;

use sonarithm_numisona::import::erie::{self, Composition, Outcome};

const GALLERY: &[(&str, &str)] = &[
    ("time-series", include_str!("erie/time-series.json")),
    ("histogram", include_str!("erie/histogram.json")),
    ("sparsity-simple", include_str!("erie/sparsity-simple.json")),
    (
        "sparsity-and-length",
        include_str!("erie/sparsity-and-length.json"),
    ),
    ("interval", include_str!("erie/interval.json")),
    (
        "model-fit-overlay",
        include_str!("erie/model-fit-overlay.json"),
    ),
    (
        "residuals-sequence",
        include_str!("erie/residuals-sequence.json"),
    ),
    ("frequency", include_str!("erie/frequency.json")),
];

/// The import as review text: composition, specs and report.
fn review(name: &str, text: &str) -> String {
    let import = erie::import(text).unwrap_or_else(|err| panic!("{name}: {err}"));
    let mut out = format!("composition: {:?}\n", import.composition);
    for spec in &import.specs {
        let json = serde_json::to_string_pretty(spec).unwrap();
        writeln!(out, "\nspec:\n{json}").unwrap();
    }
    writeln!(out, "\nreport:").unwrap();
    for entry in import.report.entries() {
        writeln!(out, "- {entry}").unwrap();
    }
    out
}

#[test]
fn the_erie_gallery_subset_imports_as_valid_specs() {
    for (name, text) in GALLERY {
        let import = erie::import(text).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert!(!import.specs.is_empty(), "{name}");
        for spec in &import.specs {
            let json = serde_json::to_string(spec).unwrap();
            match sonarithm_spec::prepare(&json, None) {
                Ok(prepared) => assert!(!prepared.diagnostics.has_errors(), "{name}"),
                Err(diagnostics) => {
                    let messages: Vec<String> =
                        diagnostics.iter().map(ToString::to_string).collect();
                    panic!(
                        "{name}: imported spec is invalid:\n{}\n{json}",
                        messages.join("\n")
                    );
                }
            }
        }
        insta::assert_snapshot!(*name, review(name, text));
    }
}

#[test]
fn compositions_follow_overlay_and_sequence() {
    let composition = |name: &str| {
        let (_, text) = GALLERY.iter().find(|(n, _)| *n == name).unwrap();
        let import = erie::import(text).unwrap();
        (import.composition, import.specs.len())
    };
    assert_eq!(composition("time-series"), (Composition::Single, 1));
    assert_eq!(composition("model-fit-overlay"), (Composition::Overlay, 1));
    assert_eq!(
        composition("residuals-sequence"),
        (Composition::Sequence, 2)
    );
}

/// 04 §9: nothing is dropped silently. The constructs of the `frequency` fixture with no
/// v0.1 equivalent are each reported at their pointer.
#[test]
fn unmapped_constructs_are_reported_at_their_pointer() {
    let (_, text) = GALLERY.iter().find(|(n, _)| *n == "frequency").unwrap();
    let import = erie::import(text).unwrap();
    let unmapped: Vec<(&str, &str)> = import
        .report
        .with(Outcome::Unmapped)
        .map(|e| (e.pointer.as_str(), e.construct.as_str()))
        .collect();
    for expected in [
        ("/config/speechRate", "config.speechRate"),
        ("/transform/2", "transform.fold"),
        ("/tone/filter", "tone.filter"),
        ("/encoding/pitch/roundToNote", "encoding.pitch.roundToNote"),
        ("/encoding/repeat", "encoding.repeat"),
        ("/encoding/panAzimuth", "encoding.panAzimuth"),
    ] {
        assert!(
            unmapped.contains(&expected),
            "{expected:?} not in {unmapped:?}"
        );
    }
    let diagnostics = import.report.diagnostics();
    assert_eq!(diagnostics.len(), import.report.entries().len());
    assert!(diagnostics.iter().any(|d| {
        d.code.as_str() == "SNR-ENG-080"
            && d.location
                .as_ref()
                .map(sonarithm_core::diagnostic::SpecPointer::as_str)
                == Some("/encoding/repeat")
    }));
    assert!(diagnostics.iter().any(|d| d.code.as_str() == "SNR-ENG-081"));
}
