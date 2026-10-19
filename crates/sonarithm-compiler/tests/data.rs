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
