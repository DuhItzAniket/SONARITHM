// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The compile pipeline (05 §2): validate and migrate, resolve the source, transform,
//! analyze, encode each track, add context, run the shapers in their fixed order, then
//! stamp the header with provenance.
//!
//! The pipeline is synchronous and deterministic: stages run in a fixed order, tracks in
//! spec order, and every collection in the Score is sorted. Cancellation is checked between
//! stages. In A03 the stages call whatever engines are registered; engines arrive from A06.

use std::collections::BTreeMap;

use serde::Serialize;
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};
use sonarithm_core::digest::Sha256Digest;
use sonarithm_core::engine::{
    Annotation, Budget, CancelToken, CompileContext, CompileError, Engine, MarkKind, Registry,
    SourceKind, SpecFragment, TrackRequest,
};
use sonarithm_core::ids::{AnalyzerId, EngineId, TrackId};
use sonarithm_core::score::{EngineVersion, Score, ScoreHeader, ShaperStage};
use sonarithm_core::units::Seconds;
use sonarithm_spec::data::Source;
use sonarithm_spec::sections::Detector;
use sonarithm_spec::{Prepared, Spec, prepare};

use crate::codes;
use crate::registry::build_registry;

/// What the compiler needs from its data type beyond what engines use.
pub trait CompilerData {
    /// SHA-256 fingerprint of the data (03 §2), recorded in the Score header.
    fn fingerprint(&self) -> Sha256Digest;
}

/// A successful compile.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Compiled {
    /// The Score.
    pub score: Score,
    /// The RFC 8785 canonical spec with defaults applied, for the render manifest.
    pub canonical_spec: String,
    /// Warnings and `Info` diagnostics, in pipeline order.
    pub diagnostics: Diagnostics,
}

/// Why a compile produced no Score.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Failure {
    /// The spec or an engine reported errors; all diagnostics so far are included.
    Invalid(Diagnostics),
    /// The compile was cancelled through its [`CancelToken`].
    Cancelled,
}

/// The compiler: a registry of engines and a budget. `D` is the data type its sources
/// produce (ADR-0014).
pub struct Compiler<D> {
    registry: Registry<D>,
    budget: Budget,
}

impl<D: CompilerData> Compiler<D> {
    /// A compiler with `engines`, registered in order, and the default [`Budget`].
    ///
    /// # Errors
    ///
    /// The registration conflict, as an `SNR-COMP-011` … `015` diagnostic.
    pub fn new(engines: &[&dyn Engine<D>]) -> Result<Self, Box<Diagnostic>> {
        Ok(Self {
            registry: build_registry(engines)?,
            budget: Budget::default(),
        })
    }

    /// The same compiler with another budget.
    #[must_use]
    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = budget;
        self
    }

    /// The registry.
    #[must_use]
    pub fn registry(&self) -> &Registry<D> {
        &self.registry
    }

    /// Compiles spec text into a Score.
    ///
    /// # Errors
    ///
    /// [`Failure::Invalid`] with every diagnostic so far, or [`Failure::Cancelled`].
    pub fn compile(&self, spec: &str, cancel: &CancelToken) -> Result<Compiled, Failure> {
        let prepared = prepare(spec, None).map_err(Failure::Invalid)?;
        self.compile_prepared(prepared, cancel)
    }

    /// Compiles an already prepared spec (for callers that also need [`Prepared`]).
    ///
    /// # Errors
    ///
    /// As for [`Compiler::compile`].
    pub fn compile_prepared(
        &self,
        prepared: Prepared,
        cancel: &CancelToken,
    ) -> Result<Compiled, Failure> {
        let spec = &prepared.resolved;
        let mut run = Run {
            registry: &self.registry,
            cx: CompileContext::new(spec.seed.unwrap_or(0), self.budget, cancel.clone()),
            used: BTreeMap::new(),
            diagnostics: prepared.diagnostics.clone(),
        };
        match run.stages(spec, prepared.hash) {
            Ok(score) => {
                let mut diagnostics = run.diagnostics;
                diagnostics.extend(run.cx.take_diagnostics());
                if diagnostics.has_errors() {
                    return Err(Failure::Invalid(diagnostics));
                }
                Ok(Compiled {
                    score,
                    canonical_spec: prepared.canonical,
                    diagnostics,
                })
            }
            Err(Stop::Cancelled) => Err(Failure::Cancelled),
            Err(Stop::Error(diagnostic)) => {
                let mut diagnostics = run.diagnostics;
                diagnostics.extend(run.cx.take_diagnostics());
                diagnostics.push(*diagnostic);
                Err(Failure::Invalid(diagnostics))
            }
        }
    }
}

/// Why the stages stopped early.
enum Stop {
    Cancelled,
    Error(Box<Diagnostic>),
}

/// One compile in progress.
struct Run<'a, D> {
    registry: &'a Registry<D>,
    cx: CompileContext,
    /// Engines that contributed, for the header.
    used: BTreeMap<EngineId, EngineVersion>,
    diagnostics: Diagnostics,
}

impl<D: CompilerData> Run<'_, D> {
    fn stages(&mut self, spec: &Spec, spec_hash: Sha256Digest) -> Result<Score, Stop> {
        let data = self.resolve_source(spec)?;
        // Transforms run in sonarithm-data (A04); until then they are reported, not dropped.
        if !spec.transform.is_empty() {
            self.diagnostics.push(
                Diagnostic::new(
                    &codes::TRANSFORMS_NOT_APPLIED,
                    "This build does not apply `transform` yet (the data layer arrives in A04); the data is used as loaded.",
                )
                .at(SpecPointer::root().key("transform")),
            );
        }
        let annotations = self.analyze(spec, &data)?;
        let mut header = ScoreHeader::new(
            spec.spec_version,
            spec.mode.unwrap_or(sonarithm_spec::Mode::Analysis).into(),
            spec.seed.unwrap_or(0),
        );
        header.spec_hash = spec_hash;
        header.data_fingerprint = data.fingerprint();
        let mut score = Score::new(header);
        self.encode(spec, &data, &annotations, &mut score)?;
        self.add_context(spec, &data, &annotations, &mut score)?;
        score.header.duration = duration(&score);
        self.cx
            .check_duration(score.header.duration)
            .map_err(stop)?;
        self.shape(spec, &mut score)?;
        score.header.engines.clone_from(&self.used);
        // 05 §4.2: invariants are checked after every compile. A violation is an engine bug.
        if let Err(violation) = score.check() {
            debug_assert!(false, "compiled Score breaks an invariant: {violation}");
            return Err(Stop::Error(Box::new(Diagnostic::new(
                &codes::INVARIANT_BROKEN,
                format!(
                    "The compiled Score breaks an invariant ({violation}). This is an engine bug; please report it."
                ),
            ))));
        }
        Ok(score)
    }

    /// Resolves `data.source` through the registered source for its kind.
    fn resolve_source(&mut self, spec: &Spec) -> Result<D, Stop> {
        let registry = self.registry;
        let at = SpecPointer::root().key("data").key("source");
        let kind = source_kind(&spec.data.source);
        let (engine, source) = registry.source(kind).ok_or_else(|| {
            unavailable(
                &format!("Reading `{kind}` sources"),
                source_owner(kind),
                at.clone(),
            )
        })?;
        self.use_engine(&engine.clone());
        let request = fragment(at, &spec.data.source)?;
        let data = source.ingest(&request, &mut self.cx).map_err(stop)?;
        self.cx.check_cancelled().map_err(stop)?;
        Ok(data)
    }

    /// Runs the detectors of `analysis.detect` in order.
    fn analyze(&mut self, spec: &Spec, data: &D) -> Result<Vec<Annotation>, Stop> {
        let registry = self.registry;
        let mut annotations = Vec::new();
        let Some(analysis) = &spec.analysis else {
            return Ok(annotations);
        };
        let at = SpecPointer::root().key("analysis");
        let params = fragment(at.clone(), analysis)?;
        for (index, detector) in analysis.detect.iter().enumerate() {
            let id = AnalyzerId::from_static(detector_id(*detector));
            let (engine, analyzer) = registry.analyzer(&id).ok_or_else(|| {
                let at = at.key("detect").index(index);
                unavailable(
                    &format!("The `{id}` detector"),
                    detector_owner(*detector),
                    at,
                )
            })?;
            self.use_engine(&engine.clone());
            annotations.extend(
                analyzer
                    .analyze(data, &params, &mut self.cx)
                    .map_err(stop)?,
            );
        }
        self.cx.check_cancelled().map_err(stop)?;
        Ok(annotations)
    }

    /// Encodes each track in spec order.
    fn encode(
        &mut self,
        spec: &Spec,
        data: &D,
        annotations: &[Annotation],
        score: &mut Score,
    ) -> Result<(), Stop> {
        let registry = self.registry;
        for (index, track) in spec.tracks.iter().enumerate() {
            let at = SpecPointer::root().key("tracks").index(index);
            let mark = MarkKind::from(track.mark);
            let wanted = track
                .engine
                .as_ref()
                .map(|engine| EngineId::new(engine.clone()));
            let (engine, encoder) = registry.encoder(mark, wanted.as_ref()).ok_or_else(|| {
                let owner = wanted
                    .as_ref()
                    .map_or_else(|| mark_owner(mark).to_owned(), ToString::to_string);
                unavailable(&format!("Encoding `{mark}` marks"), &owner, at.clone())
            })?;
            self.use_engine(&engine.clone());
            let request =
                TrackRequest::new(TrackId::new(track.id.clone()), mark, fragment(at, track)?);
            let output = encoder
                .encode(&request, data, annotations, &mut self.cx)
                .map_err(stop)?;
            self.charge(output.events.len())?;
            score.tracks.push(output);
            self.cx.check_cancelled().map_err(stop)?;
        }
        Ok(())
    }

    /// Adds the context tracks (AUDAXIS), or warns that none can be added.
    fn add_context(
        &mut self,
        spec: &Spec,
        data: &D,
        annotations: &[Annotation],
        score: &mut Score,
    ) -> Result<(), Stop> {
        let registry = self.registry;
        let Some(context) = &spec.context else {
            return Ok(());
        };
        let at = SpecPointer::root().key("context");
        let providers: Vec<_> = registry.context_providers().collect();
        if providers.is_empty() {
            self.diagnostics.push(
                Diagnostic::new(
                    &codes::CONTEXT_UNAVAILABLE,
                    "Context cues (ticks, reference tone, legend, earcons) are not available in this build; AUDAXIS arrives in B03.",
                )
                .at(at.clone()),
            );
        }
        let params = fragment(at, context)?;
        for (engine, provider) in providers {
            self.use_engine(&engine.clone());
            let tracks = provider
                .context_tracks(&params, score, data, annotations, &mut self.cx)
                .map_err(stop)?;
            for track in tracks {
                self.charge(track.events.len())?;
                score.tracks.push(track);
            }
        }
        self.cx.check_cancelled().map_err(stop)
    }

    /// Runs the shapers in their fixed order; quantization only in Presentation (SDR-10).
    fn shape(&mut self, spec: &Spec, score: &mut Score) -> Result<(), Stop> {
        let registry = self.registry;
        for (stage, engine, shaper) in registry.shapers() {
            let presentation = spec.mode == Some(sonarithm_spec::Mode::Presentation);
            if stage == ShaperStage::Quantize && !presentation {
                continue;
            }
            self.use_engine(&engine.clone());
            shaper.shape(score, &mut self.cx).map_err(stop)?;
            score.header.shapers.push(stage);
            self.cx.check_cancelled().map_err(stop)?;
        }
        Ok(())
    }

    fn use_engine(&mut self, engine: &EngineId) {
        if let Some(manifest) = self
            .registry
            .engines()
            .find(|manifest| &manifest.id == engine)
        {
            self.used.insert(engine.clone(), manifest.version.clone());
        }
    }

    fn charge(&mut self, events: usize) -> Result<(), Stop> {
        let events = u64::try_from(events).unwrap_or(u64::MAX);
        self.cx.charge_events(events).map_err(stop)
    }
}

/// Converts a capability error into a stop reason.
fn stop(error: CompileError) -> Stop {
    match error {
        CompileError::Cancelled => Stop::Cancelled,
        CompileError::Diagnostic(diagnostic) => Stop::Error(diagnostic),
        CompileError::Budget(exceeded) => Stop::Error(Box::new(Diagnostic::new(
            &codes::BUDGET_EXCEEDED,
            format!(
                "Compilation stopped: {exceeded}. Shorten the data or the pace, or raise the budget."
            ),
        ))),
        other => Stop::Error(Box::new(Diagnostic::new(
            &codes::INVARIANT_BROKEN,
            format!("{other}."),
        ))),
    }
}

/// The end of the last event, or zero.
fn duration(score: &Score) -> Seconds {
    let end = score
        .tracks
        .iter()
        .flat_map(|track| &track.events)
        .map(|event| match event {
            sonarithm_core::score::ScoreEvent::Note(note) => note.t.0 + note.dur.0,
            sonarithm_core::score::ScoreEvent::Glide(glide) => glide.t1.0,
            other => other.start().0,
        })
        .fold(0.0, f64::max);
    Seconds(end)
}

fn fragment<T: Serialize + ?Sized>(at: SpecPointer, value: &T) -> Result<SpecFragment, Stop> {
    SpecFragment::new(at.clone(), value).map_err(|err| {
        Stop::Error(Box::new(
            Diagnostic::new(&codes::INVARIANT_BROKEN, format!("{err}.")).at(at),
        ))
    })
}

fn unavailable(what: &str, owner: &str, at: SpecPointer) -> Stop {
    Stop::Error(Box::new(
        Diagnostic::new(
            &codes::ENGINE_NOT_AVAILABLE,
            format!("{what} needs `{owner}`, which is not available in this build."),
        )
        .at(at),
    ))
}

fn source_kind(source: &Source) -> SourceKind {
    match source {
        Source::Inline { .. } => SourceKind::Inline,
        Source::Chart { .. } => SourceKind::Chart,
        Source::Function { .. } => SourceKind::Function,
        Source::Stream { .. } => SourceKind::Stream,
        Source::Dataset { .. } => SourceKind::Dataset,
        _ => SourceKind::File,
    }
}

/// Who provides each source kind (07 summary matrix).
fn source_owner(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Chart => "plotsona",
        SourceKind::Function => "curvasona",
        SourceKind::Stream => "sonotrace",
        _ => "the platform data layer",
    }
}

/// The default encoder engine of each mark (07 summary matrix).
fn mark_owner(mark: MarkKind) -> &'static str {
    match mark {
        MarkKind::Grain => "tonalens",
        MarkKind::Audify => "curvasona",
        MarkKind::Model => "resonum",
        MarkKind::Sample => "the platform sample player",
        _ => "numisona",
    }
}

fn detector_id(detector: Detector) -> &'static str {
    match detector {
        Detector::Extrema => "extrema",
        Detector::Outliers => "outliers",
        Detector::Changepoints => "changepoints",
        Detector::Periodicity => "periodicity",
        Detector::TrendSegments => "trend-segments",
        _ => "critical-points",
    }
}

fn detector_owner(detector: Detector) -> &'static str {
    if detector == Detector::CriticalPoints {
        "curvasona"
    } else {
        "echometric"
    }
}
