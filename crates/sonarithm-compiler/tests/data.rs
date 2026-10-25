// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The data layer as the compiler's data: inline and file sources through the platform
//! engine, and the spec's transforms applied by `sonarithm-data` (03 §6).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::sync::Arc;

use sonarithm_compiler::{Compiled, Compiler, Failure, Platform};
use sonarithm_core::engine::{
    Annotation, CancelToken, CompileContext, CompileError, Encoder, Engine, EngineManifest,
    MarkKind, Registrar, RegistryError, TrackRequest,
};
use sonarithm_core::ids::EngineId;
use sonarithm_core::score::{
    EngineVersion, InstrumentRef, Note, ScoreEvent, ScoreTrack, TrackRole,
};
use sonarithm_core::units::{Cents, Seconds};
use sonarithm_data::DataBundle;

/// One note per row of the primary table, so tests can count rows.
struct RowCounter;

impl Encoder<DataBundle> for RowCounter {
    fn encode(
        &self,
        track: &TrackRequest,
        data: &DataBundle,
        _: &[Annotation],
        _: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError> {
        let mut out = ScoreTrack::new(
            track.id.clone(),
            TrackRole::Data,
            EngineId::from_static("counter"),
            InstrumentRef("sine".to_owned()),
        );
        let rows = data.primary().map_or(0, |table| table.rows().len());
        for row in 0..rows {
            #[expect(clippy::cast_precision_loss, reason = "test row counts are small")]
            let t = Seconds(row as f64);
            out.events
                .push(ScoreEvent::Note(Note::new(t, Seconds(0.5), Cents(0.0))));
        }
        Ok(out)
    }
}

struct Counter;

impl Engine<DataBundle> for Counter {
    fn manifest(&self) -> EngineManifest {
        EngineManifest::new(
            EngineId::from_static("counter"),
            "row counter",
            EngineVersion("0.0.0".to_owned()),
        )
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        let encoder: Arc<dyn Encoder<DataBundle>> = Arc::new(RowCounter);
        registrar.encoder(&[MarkKind::Tone], &encoder)
    }
}

fn compile(platform: &Platform, spec: &str) -> Result<Compiled, Failure> {
    Compiler::new(&[platform, &Counter])
        .unwrap()
        .compile(spec, &CancelToken::new())
}

fn spec(source: &str, transform: &str) -> String {
    format!(
        r#"{{ "specVersion": "0.1", "data": {{ "source": {source} }}, "transform": {transform},
            "tracks": [{{ "id": "t", "mark": "tone", "channels": {{ "pitch": {{ "field": "y" }} }} }}] }}"#
    )
}

const VALUES: &str = r#"{ "kind": "inline", "values": [
    { "x": 1, "y": 3.0, "g": "a" }, { "x": 2, "y": 5.0, "g": "b" },
    { "x": 3, "y": 4.0, "g": "a" }, { "x": 4, "y": 9.0, "g": "b" } ] }"#;

fn rows(compiled: &Compiled) -> usize {
    compiled.score.tracks[0].events.len()
}

#[test]
fn inline_records_compile_and_are_fingerprinted() {
    let compiled = compile(&Platform::new(), &spec(VALUES, "[]")).unwrap();
    assert_eq!(rows(&compiled), 4);
    assert_ne!(
        compiled.score.header.data_fingerprint,
        sonarithm_core::digest::Sha256Digest::ZERO
    );
    assert!(
        compiled
            .score
            .header
            .engines
            .contains_key(&EngineId::from_static("platform"))
    );
}

#[test]
fn the_provenance_mix_reaches_the_compile_result() {
    let exact = compile(&Platform::new(), &spec(VALUES, "[]")).unwrap();
    assert_eq!(exact.provenance, [("exact".to_owned(), 1.0)].into());
    let estimated = r#"{ "kind": "inline", "values": [
        { "x": 1, "y": 3.0, "__snr_prov": "exact" }, { "x": 2, "y": 5.0, "__snr_prov": "cv_estimate", "__snr_conf": 0.5 },
        { "x": 3, "y": 4.0, "__snr_prov": "exact" }, { "x": 4, "y": 9.0, "__snr_prov": "exact" } ] }"#;
    let mixed = compile(&Platform::new(), &spec(estimated, "[]")).unwrap();
    assert_eq!(
        mixed.provenance,
        [("cv_estimate".to_owned(), 0.25), ("exact".to_owned(), 0.75)].into()
    );
}

#[test]
fn transforms_run_in_the_data_layer() {
    let filtered = compile(
        &Platform::new(),
        &spec(VALUES, r#"[{ "op": "filter", "expr": "y > 3.5" }]"#),
    )
    .unwrap();
    assert_eq!(rows(&filtered), 3);
    assert!(
        !filtered
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "SNR-COMP-022")
    );
    let grouped = compile(
        &Platform::new(),
        &spec(
            VALUES,
            r#"[{ "op": "aggregate", "groupby": ["g"], "ops": [{ "op": "mean", "field": "y", "as": "y" }] }]"#,
        ),
    )
    .unwrap();
    assert_eq!(rows(&grouped), 2);
    let binned = compile(
        &Platform::new(),
        &spec(VALUES, r#"[{ "op": "bin", "field": "y", "count": 2 }, { "op": "resample", "field": "x", "every": { "ms": 2000 }, "method": "mean" }]"#),
    )
    .unwrap();
    assert!(rows(&binned) >= 2);
}

#[test]
fn a_failing_transform_stops_the_compile_with_its_location() {
    let result = compile(
        &Platform::new(),
        &spec(VALUES, r#"[{ "op": "filter", "expr": "y >" }]"#),
    );
    let Err(Failure::Invalid(diagnostics)) = result else {
        panic!("expected an error")
    };
    let error = diagnostics.iter().find(|d| d.is_error()).unwrap();
    assert_eq!(error.location.as_ref().unwrap().as_str(), "/transform");
}

#[test]
fn file_sources_read_relative_to_the_base_directory() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("compiler-data");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("series.csv"), "x,y\n1,2\n2,4\n3,8\n").unwrap();
    let source = r#"{ "kind": "file", "path": "series.csv" }"#;
    let compiled = compile(&Platform::with_base_dir(&dir), &spec(source, "[]")).unwrap();
    assert_eq!(rows(&compiled), 3);
    // Without a base directory, file sources are not registered.
    let result = compile(&Platform::new(), &spec(source, "[]"));
    let Err(Failure::Invalid(diagnostics)) = result else {
        panic!("expected an error")
    };
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_str() == "SNR-COMP-010")
    );
    let missing = r#"{ "kind": "file", "path": "absent.csv" }"#;
    let result = compile(&Platform::with_base_dir(&dir), &spec(missing, "[]"));
    let Err(Failure::Invalid(diagnostics)) = result else {
        panic!("expected an error")
    };
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_str() == "SNR-COMP-024")
    );
}

#[test]
fn dataset_sources_read_what_the_host_provides() {
    let loaded = sonarithm_data::read_bytes(
        b"x,y\n1,2\n2,4\n",
        sonarithm_data::Format::Csv,
        "frame.csv",
        &sonarithm_data::ReadOptions::new(),
    )
    .unwrap();
    let platform = Platform::new().with_dataset("main", loaded.bundle.clone());
    let source = r#"{ "kind": "dataset", "id": "main" }"#;
    let compiled = compile(&platform, &spec(source, "[]")).unwrap();
    assert_eq!(rows(&compiled), 2);
    assert_eq!(
        compiled.score.header.data_fingerprint,
        sonarithm_data::bundle_fingerprint(&loaded.bundle).unwrap(),
        "the host's data, unchanged"
    );
    let other = r#"{ "kind": "dataset", "id": "other" }"#;
    let Err(Failure::Invalid(diagnostics)) = compile(&platform, &spec(other, "[]")) else {
        panic!("expected an error")
    };
    let error = diagnostics.iter().find(|d| d.is_error()).unwrap();
    assert_eq!(error.code.as_str(), "SNR-COMP-026");
    assert_eq!(error.location.as_ref().unwrap().as_str(), "/data/source/id");
    // Without datasets, the source kind is not registered.
    let Err(Failure::Invalid(diagnostics)) = compile(&Platform::new(), &spec(source, "[]")) else {
        panic!("expected an error")
    };
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_str() == "SNR-COMP-010")
    );
}

/// RFC-0004: until a template registry exists (C01-T09), a template is reported as unknown
/// rather than compiled into something else.
#[test]
fn snr_comp_030_templates_are_not_in_the_registry_yet() {
    let template = format!(
        r#"{{ "specVersion": "0.2", "data": {{ "source": {VALUES} }},
            "template": {{ "id": "bar-discrete", "bind": {{ "value": "y" }} }} }}"#
    );
    let Err(Failure::Invalid(diagnostics)) = compile(&Platform::new(), &template) else {
        panic!("a template cannot compile yet");
    };
    let unknown = diagnostics
        .iter()
        .find(|d| d.code.as_str() == "SNR-COMP-030")
        .unwrap();
    assert_eq!(
        unknown
            .location
            .as_ref()
            .map(sonarithm_core::SpecPointer::as_str),
        Some("/template/id")
    );
}

/// A v0.1 spec still compiles to the same Score under the v0.2 reader, and records its
/// own version.
#[test]
fn v0_1_specs_compile_and_keep_their_version() {
    let old = compile(&Platform::new(), &spec(VALUES, "[]")).unwrap();
    let new = compile(
        &Platform::new(),
        &spec(VALUES, "[]").replacen("\"0.1\"", "\"0.2\"", 1),
    )
    .unwrap();
    assert_eq!(old.score.tracks, new.score.tracks);
    assert_eq!(
        old.score.header.spec_version,
        sonarithm_core::score::SpecVersion::V0_1
    );
    assert_eq!(
        new.score.header.spec_version,
        sonarithm_core::score::SpecVersion::V0_2
    );
}
