// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A03-T09: the compiler skeleton compiles a spec with an inline source end to end, through
//! every stage, with no-op test engines.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::sync::Arc;

use sonarithm_compiler::{Compiled, Compiler, CompilerData, Failure};
use sonarithm_core::digest::Sha256Digest;
use sonarithm_core::engine::{
    Annotation, Budget, CancelToken, CompileContext, CompileError, Encoder, Engine, EngineManifest,
    MarkKind, Registrar, RegistryError, Shaper, Source, SourceKind, SpecFragment, TrackRequest,
};
use sonarithm_core::ids::EngineId;
use sonarithm_core::score::{
    EngineVersion, InstrumentRef, Note, Score, ScoreEvent, ScoreTrack, ShaperStage, TrackRole,
};
use sonarithm_core::units::{Cents, Seconds};

/// Inline records, standing in for the Arrow `DataBundle` of A04.
struct InlineTable {
    rows: usize,
    fingerprint: Sha256Digest,
}

impl CompilerData for InlineTable {
    fn fingerprint(&self) -> Sha256Digest {
        self.fingerprint
    }
}

/// Reads `inline` sources.
struct InlineSource;

impl Source<InlineTable> for InlineSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        _: &mut CompileContext,
    ) -> Result<InlineTable, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        let sonarithm_spec::Source::Inline { values } = parsed else {
            return Err(CompileError::Engine {
                engine: EngineId::from_static("platform"),
                message: "not an inline source".to_owned(),
            });
        };
        Ok(InlineTable {
            rows: values.len(),
            fingerprint: Sha256Digest::of(source.json().as_bytes()),
        })
    }
}

/// Encodes one note per row with a seeded random pitch, or nothing when `silent`.
struct ToneEncoder {
    silent: bool,
}

impl Encoder<InlineTable> for ToneEncoder {
    fn encode(
        &self,
        track: &TrackRequest,
        data: &InlineTable,
        _: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError> {
        let mut out = ScoreTrack::new(
            track.id.clone(),
            TrackRole::Data,
            EngineId::from_static("numisona"),
            InstrumentRef("mallet".to_owned()),
        );
        if !self.silent {
            let mut rng = cx.rng(&track.id);
            for row in 0..data.rows {
                let t = Seconds(f64::from(u32::try_from(row).unwrap()) / 3.0);
                let pitch = Cents(rng.next_f64() * 1200.0);
                out.events
                    .push(ScoreEvent::Note(Note::new(t, Seconds(0.25), pitch)));
            }
        }
        Ok(out)
    }
}

/// A shaper that changes nothing.
struct NoShaper;

impl Shaper for NoShaper {
    fn shape(&self, _: &mut Score, _: &mut CompileContext) -> Result<(), CompileError> {
        Ok(())
    }
}

/// A configurable test engine.
struct TestEngine {
    id: &'static str,
    source: bool,
    tone: Option<bool>,
    shapers: &'static [ShaperStage],
}

impl Engine<InlineTable> for TestEngine {
    fn manifest(&self) -> EngineManifest {
        EngineManifest::new(
            EngineId::from_static(self.id),
            self.id,
            EngineVersion("0.0.0".to_owned()),
        )
    }

    fn register(&self, registrar: &mut Registrar<'_, InlineTable>) -> Result<(), RegistryError> {
        if self.source {
            registrar.source(SourceKind::Inline, Arc::new(InlineSource))?;
        }
        if let Some(silent) = self.tone {
            let encoder: Arc<dyn Encoder<InlineTable>> = Arc::new(ToneEncoder { silent });
            registrar.encoder(&[MarkKind::Tone], &encoder)?;
        }
        for stage in self.shapers {
            registrar.shaper(*stage, Arc::new(NoShaper))?;
        }
        Ok(())
    }
}

const PLATFORM: TestEngine = TestEngine {
    id: "platform",
    source: true,
    tone: None,
    shapers: &[],
};

fn numisona(silent: bool) -> TestEngine {
    TestEngine {
        id: "numisona",
        source: false,
        tone: Some(silent),
        shapers: &[],
    }
}

const SPEC: &str = r#"{
  "specVersion": "0.1", "seed": 1729,
  "data": { "source": { "kind": "inline", "values": [{ "t": 1, "v": 2.5 }, { "t": 2, "v": 3.5 }, { "t": 3, "v": 1.0 }] } },
  "tracks": [{ "id": "v", "mark": "tone", "channels": { "pitch": { "field": "v" } } }]
}"#;

fn compile(engines: &[&dyn Engine<InlineTable>], spec: &str) -> Result<Compiled, Failure> {
    Compiler::new(engines)
        .unwrap()
        .compile(spec, &CancelToken::new())
}

fn codes(failure: Failure) -> Vec<String> {
    match failure {
        Failure::Invalid(diagnostics) => diagnostics.iter().map(|d| d.code.to_string()).collect(),
        Failure::Cancelled => vec!["cancelled".to_owned()],
        _ => Vec::new(),
    }
}

/// A03-T09 acceptance: an inline-source spec compiles end to end to an empty Score.
#[test]
fn an_inline_spec_compiles_end_to_end_to_an_empty_score() {
    let compiled = compile(&[&PLATFORM, &numisona(true)], SPEC).unwrap();
    let score = &compiled.score;
    assert_eq!(score.tracks.len(), 1);
    assert_eq!(score.tracks[0].events.len(), 0, "no-op encoder");
    assert_eq!(score.header.duration, Seconds(0.0));
    assert_eq!(score.header.seed, 1729);
    let engines: Vec<&str> = score.header.engines.keys().map(EngineId::as_str).collect();
    assert_eq!(engines, ["numisona", "platform"]);
    let prepared = sonarithm_spec::prepare(SPEC, None).unwrap();
    assert_eq!(score.header.spec_hash, prepared.hash);
    assert_eq!(compiled.canonical_spec, prepared.canonical);
    let warnings: Vec<&str> = compiled
        .diagnostics
        .iter()
        .filter(|d| !d.code.as_str().ends_with("080"))
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(warnings, ["SNR-COMP-023"], "no AUDAXIS in this build");
    insta::assert_snapshot!(
        score.review_json(&sonarithm_core::score::review::ReviewOptions::default())
    );
}

#[test]
fn the_same_input_gives_byte_identical_scores() {
    let engines: [&dyn Engine<InlineTable>; 2] = [&PLATFORM, &numisona(false)];
    let first = compile(&engines, SPEC).unwrap().score;
    let second = compile(&engines, SPEC).unwrap().score;
    assert_eq!(first.tracks[0].events.len(), 3);
    assert_eq!(
        first.canonical_hash().unwrap(),
        second.canonical_hash().unwrap()
    );
    assert_eq!(first.header.duration, Seconds(2.0 / 3.0 + 0.25));
}

/// 05 §3 rule 2: a capability that is not registered gives SNR-COMP-010.
#[test]
fn missing_engines_are_reported_with_snr_comp_010() {
    let Err(failure) = compile(&[&PLATFORM], SPEC) else {
        panic!("compiled without an encoder");
    };
    let Failure::Invalid(diagnostics) = failure else {
        panic!("not invalid");
    };
    let last = diagnostics.iter().last().unwrap();
    assert_eq!(last.code.as_str(), "SNR-COMP-010");
    assert_eq!(last.location.as_ref().unwrap().as_str(), "/tracks/0");
    assert!(last.message.contains("numisona"), "{}", last.message);

    let file = SPEC.replace(r#""kind": "inline", "values": [{ "t": 1, "v": 2.5 }, { "t": 2, "v": 3.5 }, { "t": 3, "v": 1.0 }]"#, r#""kind": "file", "path": "a.csv""#);
    assert_eq!(
        codes(compile(&[&PLATFORM, &numisona(true)], &file).unwrap_err())
            .last()
            .unwrap(),
        "SNR-COMP-010"
    );
}

/// SDR-10: the quantization shaper never runs in Analysis mode.
#[test]
fn sdr_10_quantize_runs_only_in_presentation_mode() {
    let axonote = TestEngine {
        id: "axonote",
        source: false,
        tone: None,
        shapers: &[ShaperStage::Quantize, ShaperStage::Lint],
    };
    let engines: [&dyn Engine<InlineTable>; 3] = [&PLATFORM, &numisona(true), &axonote];
    let analysis = compile(&engines, SPEC).unwrap().score;
    assert_eq!(analysis.header.shapers, [ShaperStage::Lint]);
    let presentation = SPEC.replace(r#""seed": 1729"#, r#""seed": 1729, "mode": "presentation""#);
    let presented = compile(&engines, &presentation).unwrap().score;
    assert_eq!(
        presented.header.shapers,
        [ShaperStage::Quantize, ShaperStage::Lint]
    );
}

#[test]
fn budgets_and_cancellation_stop_the_compile() {
    let compiler = Compiler::new(&[&PLATFORM, &numisona(false)])
        .unwrap()
        .with_budget(Budget {
            max_events: 2,
            ..Budget::default()
        });
    let failure = compiler.compile(SPEC, &CancelToken::new()).unwrap_err();
    assert_eq!(codes(failure).last().unwrap(), "SNR-COMP-021");

    let cancel = CancelToken::new();
    cancel.cancel();
    let compiler = Compiler::new(&[&PLATFORM, &numisona(true)]).unwrap();
    assert!(matches!(
        compiler.compile(SPEC, &cancel),
        Err(Failure::Cancelled)
    ));
}

#[test]
fn invalid_specs_fail_with_spec_diagnostics() {
    let reversed = SPEC.replace(
        r#""field": "v""#,
        r#""field": "v", "range": { "hz": [1400, 400] }"#,
    );
    let failure = compile(&[&PLATFORM, &numisona(true)], &reversed).unwrap_err();
    assert!(codes(failure).contains(&"SNR-SPEC-031".to_owned()));
}
