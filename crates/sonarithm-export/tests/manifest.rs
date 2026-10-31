// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A05-T08 acceptance: the render manifest validates against its generated schema and holds
//! no absolute paths; exports are deterministic and normalized as asked.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use sonarithm_core::{
    Cents, Db, EngineId, EngineVersion, InstrumentRef, Marker, MarkerKind, Mode, Note, Score,
    ScoreEvent, ScoreHeader, ScoreTrack, Seconds, SpecVersion, Speech, TrackId, TrackRole,
};
use sonarithm_export::manifest::{file_name, json_schema};
use sonarithm_export::{
    AudioFormat, DataSource, ExportOptions, ManifestInputs, Normalization, WavEncoding, export,
};

fn score() -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 1729);
    header.duration = Seconds(2.0);
    header.engines.insert(
        EngineId::from_static("numisona"),
        EngineVersion("0.1.0".to_owned()),
    );
    let mut score = Score::new(header);
    let mut melody = ScoreTrack::new(
        TrackId::new("melody"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("mallet".to_owned()),
    );
    melody.gain = Db(-18.0);
    melody.events = (0..6)
        .map(|i| {
            let t = 0.3 * f64::from(i);
            ScoreEvent::Note(Note::new(
                Seconds(t),
                Seconds(0.25),
                Cents(100.0 * f64::from(i)),
            ))
        })
        .collect();
    let mut legend = ScoreTrack::new(
        TrackId::new("legend"),
        TrackRole::Speech,
        EngineId::from_static("audaxis"),
        InstrumentRef("speech".to_owned()),
    );
    legend.events = vec![ScoreEvent::Speech(Speech::new(Seconds(0.0), "Rising"))];
    score.tracks = vec![melody, legend];
    score.markers = vec![Marker::new(Seconds(1.5), MarkerKind::Insight, "Maximum")];
    score
}

/// A fresh directory under the system temp directory.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sonarithm-export-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn provenance() -> ManifestInputs {
    let mut provenance = ManifestInputs::default();
    provenance.data_source = Some(DataSource::new(
        "file",
        r"C:\Users\someone\Documents\temps.csv",
    ));
    provenance.provenance = BTreeMap::from([("exact".to_owned(), 1.0)]);
    provenance.git = Some("0123abc".to_owned());
    provenance
}

#[test]
fn the_manifest_validates_against_its_schema_and_has_no_absolute_paths() {
    let dir = scratch("schema");
    let options = ExportOptions::new(AudioFormat::Flac);
    let exported = export(
        &score(),
        &options,
        &provenance(),
        &dir,
        "/home/someone/out/temps",
    )
    .unwrap();
    let text = std::fs::read_to_string(dir.join("temps.render.json")).unwrap();
    let instance: serde_json::Value = serde_json::from_str(&text).unwrap();

    let schema: serde_json::Value = serde_json::from_str(&json_schema()).unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler
        .add_resource("urn:sonarithm:render-manifest", schema)
        .unwrap();
    let index = compiler
        .compile("urn:sonarithm:render-manifest", &mut schemas)
        .unwrap();
    if let Err(error) = schemas.validate(&instance, index) {
        panic!("the manifest does not match its schema: {error}");
    }

    // ADR-0012: no absolute path, drive or user directory anywhere.
    for needle in ["Users", "someone", "home", ":\\", "\\\\", "/"] {
        let found = text
            .lines()
            .find(|line| line.contains(needle) && !line.contains("\"sha256:"));
        assert_eq!(found, None, "`{needle}` in the manifest");
    }
    let manifest = exported.manifest;
    assert_eq!(manifest.output.file, "temps.flac");
    assert_eq!(manifest.data.source.unwrap().name, "temps.csv");
    assert_eq!(manifest.cues.unwrap().count, 2);
    assert!(dir.join("temps.cues.vtt").exists());

    // The recorded hash is the hash of the file's bytes.
    let bytes = std::fs::read(dir.join("temps.flac")).unwrap();
    let hex = Sha256::digest(&bytes)
        .iter()
        .fold(String::new(), |hex, b| hex + &format!("{b:02x}"));
    assert_eq!(manifest.output.sha256, format!("sha256:{hex}"));
    assert_eq!(manifest.render.sample_rate, 48_000);
    assert_eq!(manifest.render.block_size, 256);
    assert_eq!(manifest.render.seed, 1729);
    assert_eq!(manifest.render.mode, "analysis");
    assert_eq!(manifest.output.frames, 96_000);
    assert_eq!(manifest.sonarithm.engines["numisona"], "0.1.0");
    let _ = std::fs::remove_dir_all(dir);
}

/// sound-design §6.3: one static gain to −16 LUFS, within −1 dBTP.
#[test]
fn exports_are_normalized_within_the_true_peak_limit() {
    let dir = scratch("loudness");
    let options = ExportOptions::new(AudioFormat::Wav(WavEncoding::Float32));
    let manifest = export(&score(), &options, &ManifestInputs::default(), &dir, "norm")
        .unwrap()
        .manifest;
    let lufs = manifest.output.integrated_lufs.unwrap();
    let peak = manifest.output.true_peak_dbtp;
    assert!(peak <= -0.99, "{peak} dBTP");
    assert!(
        (lufs + 16.0).abs() <= 0.1 || (peak + 1.0).abs() <= 0.05,
        "{lufs} LUFS at {peak} dBTP"
    );
    assert!(manifest.output.gain_db > 0.0, "the quiet Score was raised");

    let mut preserve = options.clone();
    preserve.normalization = Normalization::Preserve;
    let kept = export(
        &score(),
        &preserve,
        &ManifestInputs::default(),
        &dir,
        "kept",
    )
    .unwrap()
    .manifest;
    assert_eq!(kept.output.gain_db, 0.0);
    assert!(kept.output.integrated_lufs.unwrap() < lufs);
    let _ = std::fs::remove_dir_all(dir);
}

/// SDR-01, ADR-0013: the same Score exports to the same bytes and the same manifest.
#[test]
fn exports_are_deterministic() {
    let (a, b) = (scratch("det-a"), scratch("det-b"));
    let options = ExportOptions::new(AudioFormat::Wav(WavEncoding::Pcm24));
    let first = export(&score(), &options, &provenance(), &a, "x")
        .unwrap()
        .manifest;
    let second = export(&score(), &options, &provenance(), &b, "x")
        .unwrap()
        .manifest;
    assert_eq!(first, second);
    assert_eq!(
        std::fs::read(a.join("x.render.json")).unwrap(),
        std::fs::read(b.join("x.render.json")).unwrap()
    );
    let _ = (std::fs::remove_dir_all(a), std::fs::remove_dir_all(b));
}

#[test]
fn names_keep_only_their_last_component() {
    assert_eq!(file_name(r"C:\Users\someone\temps.csv"), "temps.csv");
    assert_eq!(file_name("/home/someone/data/temps.csv"), "temps.csv");
    assert_eq!(file_name("temps.csv"), "temps.csv");
    assert_eq!(
        file_name("C:temps.csv"),
        "temps.csv",
        "a drive-relative path"
    );
}

/// Validates `instance` against the schema text `schema`.
fn validates(schema: &str, instance: &serde_json::Value) -> Result<(), String> {
    let schema: serde_json::Value = serde_json::from_str(schema).unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler
        .add_resource("urn:sonarithm:manifest", schema)
        .unwrap();
    let index = compiler
        .compile("urn:sonarithm:manifest", &mut schemas)
        .unwrap();
    schemas
        .validate(instance, index)
        .map_err(|error| error.to_string())
}

/// RFC-0006 §4, Q6: manifest version 2 records the experiments that were on in
/// `spec.experimental`, and leaves the field out when none was, so a manifest without
/// experiments still matches the version 1 schema apart from its version number.
#[test]
fn manifest_version_2_records_experiments_and_version_1_still_reads() {
    let dir = scratch("experiments");
    let options = ExportOptions::new(AudioFormat::Wav(WavEncoding::Pcm24));
    let mut inputs = provenance();
    inputs.experimental = vec!["lowConfidenceTimbre".to_owned()];
    let exported = export(&score(), &options, &inputs, &dir, "on").unwrap();
    assert_eq!(exported.manifest.manifest_version, 2);
    assert_eq!(exported.manifest.spec.experimental, ["lowConfidenceTimbre"]);
    let text = std::fs::read_to_string(dir.join("on.render.json")).unwrap();
    let on: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        on["spec"]["experimental"],
        serde_json::json!(["lowConfidenceTimbre"])
    );
    validates(&json_schema(), &on).unwrap();

    let off = export(&score(), &options, &provenance(), &dir, "off").unwrap();
    let text = std::fs::read_to_string(dir.join("off.render.json")).unwrap();
    let mut old: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(old["spec"].get("experimental").is_none());
    assert_eq!(off.manifest.spec.experimental, Vec::<String>::new());
    old["manifestVersion"] = serde_json::json!(1);
    let v1 = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../schemas/sonarithm-render-manifest-1.schema.json"),
    )
    .unwrap();
    validates(&v1, &old).unwrap();
    let read: sonarithm_export::manifest::RenderManifest = serde_json::from_value(old).unwrap();
    assert_eq!(read.manifest_version, 1);
    let _ = std::fs::remove_dir_all(dir);
}
