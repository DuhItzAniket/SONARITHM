// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rendering through the facade: files, manifests and cues, determinism, and the in-memory
//! WAV that notebooks play.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::PathBuf;

use sonarithm::{AudioFormat, Error, Normalization, RenderOptions, Sonarithm, Spec};

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("facade-render")
        .join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A spec next to its CSV in `dir`.
fn temps_in(dir: &std::path::Path) -> Spec {
    std::fs::write(
        dir.join("temps.csv"),
        "day,temp\n1,8.5\n2,9.25\n3,7.0\n4,10.5\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("temps.spec.json"),
        r#"{ "specVersion": "0.1", "name": "Temperatures",
             "data": { "source": { "kind": "file", "path": "temps.csv" },
                       "fields": { "temp": { "concept": "temperature" } } },
             "substrate": { "time": { "field": "day" } },
             "tracks": [{ "id": "temps", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }] }"#,
    )
    .unwrap();
    Spec::from_file(dir.join("temps.spec.json")).unwrap()
}

#[test]
fn a_render_writes_audio_manifest_and_cues_without_user_paths() {
    let dir = tmp("files");
    let snr = Sonarithm::new();
    let compiled = snr.compile(&temps_in(&dir)).unwrap();
    let rendered = snr
        .render_to_file(&compiled, dir.join("temps.wav"), &RenderOptions::default())
        .unwrap();
    assert!(rendered.audio.is_file());
    assert!(rendered.manifest_path.is_file());
    assert!(rendered.cues.as_ref().unwrap().is_file());
    assert_eq!(rendered.manifest.output.file, "temps.wav");
    let manifest = std::fs::read_to_string(&rendered.manifest_path).unwrap();
    let dir_text = dir.to_string_lossy();
    assert!(!manifest.contains(&*dir_text), "no user paths (ADR-0012)");
    assert!(!manifest.contains("facade-render"), "no directories at all");
    let written = &rendered.manifest;
    assert_eq!(written.data.source.as_ref().unwrap().name, "temps.csv");
    assert_eq!(written.data.provenance.get("exact"), Some(&1.0));
    assert_eq!(
        written.score.hash,
        format!("sha256:{}", compiled.score_hash().unwrap())
    );
    assert!(
        written.spec.embedded.is_some(),
        "the canonical spec is embedded"
    );
}

#[test]
fn the_same_spec_renders_the_same_bytes_and_memory_matches_the_file() {
    let snr = Sonarithm::new();
    let first = tmp("determinism-a");
    let second = tmp("determinism-b");
    let compiled_a = snr.compile(&temps_in(&first)).unwrap();
    let compiled_b = snr.compile(&temps_in(&second)).unwrap();
    let options = RenderOptions::default();
    let a = snr
        .render_to_file(&compiled_a, first.join("t.wav"), &options)
        .unwrap();
    let b = snr
        .render_to_file(&compiled_b, second.join("t.wav"), &options)
        .unwrap();
    let bytes = std::fs::read(&a.audio).unwrap();
    assert_eq!(bytes, std::fs::read(&b.audio).unwrap());
    assert_eq!(
        std::fs::read(&a.manifest_path).unwrap(),
        std::fs::read(&b.manifest_path).unwrap(),
        "manifests are byte-identical: no timestamps"
    );
    let (memory, rate) = snr.render_wav(&compiled_a, &options).unwrap();
    assert_eq!(rate, 48_000);
    assert_eq!(memory, bytes, "notebook audio equals the file");
}

#[test]
fn formats_rates_and_levels_follow_the_options() {
    let dir = tmp("options");
    let snr = Sonarithm::new();
    let compiled = snr.compile(&temps_in(&dir)).unwrap();
    let flac = snr
        .render_to_file(
            &compiled,
            dir.join("t.flac"),
            &RenderOptions::new().with_sample_rate(44_100),
        )
        .unwrap();
    assert_eq!(flac.manifest.output.format, "flac");
    assert_eq!(flac.manifest.render.sample_rate, 44_100);
    let preserved = snr
        .render_wav(
            &compiled,
            &RenderOptions::new().with_normalization(Normalization::Preserve),
        )
        .unwrap()
        .0;
    let normalized = snr.render_wav(&compiled, &RenderOptions::new()).unwrap().0;
    assert_ne!(
        preserved, normalized,
        "the loudness target changes the level"
    );
    let no_cues = snr
        .render_to_file(
            &compiled,
            dir.join("quiet.wav"),
            &RenderOptions::new().without_cues(),
        )
        .unwrap();
    assert!(no_cues.cues.is_none());
    assert!(!dir.join("quiet.cues.vtt").exists());
}

#[test]
fn unsupported_outputs_are_explained() {
    let dir = tmp("errors");
    let snr = Sonarithm::new();
    let compiled = snr.compile(&temps_in(&dir)).unwrap();
    let Err(Error::Render(message)) =
        snr.render_to_file(&compiled, dir.join("t.mp3"), &RenderOptions::default())
    else {
        panic!("an mp3 path is not supported")
    };
    assert!(
        message.contains("t.mp3") && message.contains(".wav or .flac"),
        "{message}"
    );
    let Err(Error::Render(message)) = snr.render_to_file(
        &compiled,
        dir.join("t.wav"),
        &RenderOptions::new().with_format(AudioFormat::Flac),
    ) else {
        panic!("a format that contradicts the extension is refused")
    };
    assert!(message.contains(".wav"), "{message}");
    assert!(!message.contains("errors"), "no directories: {message}");
}
