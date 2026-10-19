// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compiles specs through the real compiler with NUMISONA registered and a test source:
//! `inline` sources are read by the data layer, `file` sources are the given bundle.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    dead_code,
    reason = "test helpers; clippy treats them as non-test code, and not every test uses every helper"
)]

use std::sync::Arc;

use sonarithm_compiler::{Compiled, Compiler, Failure};
use sonarithm_core::engine::{
    CancelToken, CompileContext, CompileError, Engine, EngineManifest, Registrar, RegistryError,
    Source, SourceKind, SpecFragment,
};
use sonarithm_core::ids::EngineId;
use sonarithm_core::score::review::ReviewOptions;
use sonarithm_core::score::{EngineVersion, Score, ScoreEvent};
use sonarithm_data::{DataBundle, Format, ReadOptions, read_bytes};
use sonarithm_numisona::Numisona;

/// Reads `inline` values with the data layer; any `file` source is the given bundle.
struct TestSource(Option<DataBundle>);

impl Source<DataBundle> for TestSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        _: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        match parsed {
            sonarithm_spec::Source::Inline { values } => {
                let bytes = serde_json::to_vec(&values).unwrap();
                let loaded = read_bytes(&bytes, Format::Json, "inline.json", &ReadOptions::new())
                    .map_err(|err| CompileError::Engine {
                    engine: EngineId::from_static("platform"),
                    message: err.to_string(),
                })?;
                Ok(loaded.bundle)
            }
            _ => self.0.clone().ok_or_else(|| CompileError::Engine {
                engine: EngineId::from_static("platform"),
                message: "no bundle was given for a file source".to_owned(),
            }),
        }
    }
}

/// A stand-in for the platform data layer.
struct Platform(Option<DataBundle>);

impl Engine<DataBundle> for Platform {
    fn manifest(&self) -> EngineManifest {
        EngineManifest::new(
            EngineId::from_static("platform"),
            "test platform",
            EngineVersion("0.0.0".to_owned()),
        )
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        let source = Arc::new(TestSource(self.0.clone()));
        registrar.source(SourceKind::Inline, source.clone())?;
        registrar.source(SourceKind::File, source)
    }
}

/// Compiles `spec` over `data` (used for `file` sources) with `numisona`.
pub(crate) fn compile_with(
    spec: &str,
    data: Option<DataBundle>,
    numisona: &Numisona,
) -> Result<Compiled, Failure> {
    let platform = Platform(data);
    let compiler = Compiler::new(&[&platform, numisona]).unwrap();
    compiler.compile(spec, &CancelToken::new())
}

/// Compiles `spec` over `data` with the default engine; panics with the diagnostics on
/// failure.
pub(crate) fn compile(spec: &str, data: Option<DataBundle>) -> Compiled {
    match compile_with(spec, data, &Numisona::new()) {
        Ok(compiled) => compiled,
        Err(Failure::Invalid(diagnostics)) => {
            let lines: Vec<String> = diagnostics.iter().map(ToString::to_string).collect();
            panic!("compile failed:\n{}", lines.join("\n"))
        }
        Err(other) => panic!("compile failed: {other:?}"),
    }
}

/// The diagnostics of a compile with an `SNR-ENG` code, as `code: message` lines.
pub(crate) fn engine_diagnostics(compiled: &Compiled) -> Vec<String> {
    compiled
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str().starts_with("SNR-ENG"))
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

/// Whether a compile reported `code`.
pub(crate) fn has_code(compiled: &Compiled, code: &str) -> bool {
    compiled.diagnostics.iter().any(|d| d.code.as_str() == code)
}

/// The review form plus the engine diagnostics, for snapshots.
pub(crate) fn review(compiled: &Compiled) -> String {
    let mut text = compiled.score.review_json(&ReviewOptions::default());
    text.push_str("\nNUMISONA diagnostics:\n");
    for line in engine_diagnostics(compiled) {
        text.push_str("- ");
        text.push_str(&line);
        text.push('\n');
    }
    text
}

/// The notes of the first track.
pub(crate) fn notes(score: &Score) -> Vec<sonarithm_core::score::Note> {
    score.tracks[0]
        .events
        .iter()
        .filter_map(|event| match event {
            ScoreEvent::Note(note) => Some(note.clone()),
            _ => None,
        })
        .collect()
}

/// Renders up to `seconds` of `score` through the render engine at 48 kHz, as interleaved
/// stereo samples.
pub(crate) fn render(score: &Score, seconds: f64) -> Vec<f32> {
    use sonarithm_core::units::SampleRate;
    use sonarithm_render::{Command, Graph, GraphConfig, PlanConfig, RenderPlan};
    const BLOCK: usize = 512;
    let plan = RenderPlan::build(score, &PlanConfig::new(SampleRate::HZ_48000))
        .unwrap()
        .plan;
    let mut graph = Graph::new(plan, GraphConfig::new(BLOCK));
    let length = usize::try_from(graph.plan().length.0).unwrap();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a few seconds of frames"
    )]
    let frames = length.min((seconds * 48_000.0) as usize);
    graph.apply(Command::Play);
    let mut out = vec![0.0; frames.div_ceil(BLOCK) * BLOCK * 2];
    for chunk in out.chunks_mut(BLOCK * 2) {
        graph.process(chunk);
    }
    out.truncate(frames * 2);
    out
}
