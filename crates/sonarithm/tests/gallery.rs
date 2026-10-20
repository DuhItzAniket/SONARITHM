// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The examples gallery (`examples/`): every spec compiles with no warning but the missing
//! context cues (AUDAXIS, B03), renders at the export loudness target, and reads its data
//! through a relative path.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::PathBuf;

use sonarithm::{RenderOptions, Severity, Sonarithm, Spec};

fn gallery() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn specs() -> Vec<PathBuf> {
    let mut specs: Vec<PathBuf> = std::fs::read_dir(gallery().join("specs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.to_string_lossy().ends_with(".spec.json"))
        .collect();
    specs.sort();
    specs
}

#[test]
fn the_gallery_has_one_spec_per_registered_dataset() {
    let names: Vec<String> = specs()
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        [
            "ds-001-temperature.spec.json",
            "ds-002-prices.spec.json",
            "ds-003-regions.spec.json",
            "ds-004-fruit.spec.json",
            "ds-005-signal.spec.json",
        ]
    );
}

#[test]
fn every_gallery_spec_compiles_and_renders() {
    let snr = Sonarithm::new();
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("gallery");
    std::fs::create_dir_all(&out).unwrap();
    for path in specs() {
        let spec = Spec::from_file(&path).unwrap();
        let text = spec.text().to_owned();
        assert!(
            text.contains("\"../data/"),
            "{}: data next to the gallery",
            spec.name()
        );
        let compiled = snr
            .compile(&spec)
            .unwrap_or_else(|err| panic!("{}: {err}", spec.name()));
        let warnings: Vec<String> = compiled
            .diagnostics()
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .map(|d| d.code.to_string())
            .collect();
        assert_eq!(
            warnings,
            ["SNR-COMP-023"],
            "{}: only the missing context cues",
            spec.name()
        );
        let stem = spec.name().trim_end_matches(".spec.json").to_owned();
        let rendered = snr
            .render_to_file(
                &compiled,
                out.join(format!("{stem}.wav")),
                &RenderOptions::default(),
            )
            .unwrap();
        let output = &rendered.manifest.output;
        assert!(output.frames > 0, "{stem}");
        let lufs = output.integrated_lufs.unwrap();
        assert!(
            (lufs + 16.0).abs() < 0.5,
            "{stem} renders at the -16 LUFS target, not {lufs}"
        );
        assert!(
            output.true_peak_dbtp <= -1.0 + 1e-6,
            "{stem} stays within -1 dBTP"
        );
    }
}
