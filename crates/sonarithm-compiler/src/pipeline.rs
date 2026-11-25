// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The compile pipeline (05 §2): validate and migrate, resolve the source, transform,
//! expand a template (RFC-0004 §2), analyze, encode each track, add context, run the shapers in their fixed order, then
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
use sonarithm_core::ids::{AnalyzerId, EngineId, RuleId, TrackId};
use sonarithm_core::nav::NavTree;
use sonarithm_core::score::{EngineVersion, Score, ScoreHeader, ShaperStage};
use sonarithm_core::units::Seconds;
use sonarithm_spec::data::Source;
use sonarithm_spec::sections::Detector;
use sonarithm_spec::v02::TemplateRef;
use sonarithm_spec::{Prepared, Spec, Transform, prepare};

use crate::codes;
use crate::registry::build_registry;
use crate::template;

/// What the compiler needs from its data type beyond what engines use.
pub trait CompilerData {
    /// SHA-256 fingerprint of the data (03 §2), recorded in the Score header.
    fn fingerprint(&self) -> Sha256Digest;

    /// Applies the spec's `transform` list (03 §6), located at `at`, and returns the data
    /// layer's diagnostics; `None` when this data type cannot apply transforms, in which
    /// case the compiler warns (`SNR-COMP-022`) and uses the data as loaded.
    ///
    /// # Errors
    ///
    /// The `Some(Err(…))` diagnostic of the first transform that cannot run.
    fn transform(
        &mut self,
        transforms: &[sonarithm_spec::Transform],
        at: &SpecPointer,
    ) -> Option<Result<Diagnostics, Box<Diagnostic>>> {
        let _ = (transforms, at);
        None
    }

    /// The share of rows in each provenance class (03 §4), such as `{"exact": 0.7,
    /// "cv_estimate": 0.3}`, for render manifests. Empty when this data type does not track
    /// provenance.
    fn provenance_mix(&self) -> BTreeMap<String, f64> {
        BTreeMap::new()
    }
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
    /// The share of data rows in each provenance class after transforms
    /// ([`CompilerData::provenance_mix`]), for the render manifest.
    pub provenance: BTreeMap<String, f64>,
    /// The navigation tree (05 §8), when a registered context provider builds one
    /// (AUDAXIS). It is not part of the Score and never changes its hash.
    pub nav: Option<NavTree>,
    /// The keys of the experimental switches that are on (RFC-0006), sorted, for the
    /// render manifest and the description.
    pub experiments: Vec<String>,
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
        let written = &prepared.spec;
        let spec = &prepared.resolved;
        let experiments = experiments_on(spec);
        let mut diagnostics = prepared.diagnostics.clone();
        diagnostics.extend(experiment_notes(&experiments));
        let mut run = Run {
            registry: &self.registry,
            cx: CompileContext::new(spec.seed.unwrap_or(0), self.budget, cancel.clone()),
            used: BTreeMap::new(),
            diagnostics,
        };
        match run.stages(written, spec, prepared.hash) {
            Ok(Finished {
                score,
                provenance,
                nav,
            }) => {
                let mut diagnostics = run.diagnostics;
                diagnostics.extend(run.cx.take_diagnostics());
                if diagnostics.has_errors() {
                    return Err(Failure::Invalid(diagnostics));
                }
                Ok(Compiled {
                    score,
                    canonical_spec: prepared.canonical,
                    diagnostics,
                    provenance,
                    nav,
                    experiments,
                })
            }
            Err(Stop::Cancelled) => Err(Failure::Cancelled),
            Err(Stop::Reported) => {
                let mut diagnostics = run.diagnostics;
                diagnostics.extend(run.cx.take_diagnostics());
                Err(Failure::Invalid(diagnostics))
            }
            Err(Stop::Error(diagnostic)) => {
                let mut diagnostics = run.diagnostics;
                diagnostics.extend(run.cx.take_diagnostics());
                diagnostics.push(*diagnostic);
                Err(Failure::Invalid(diagnostics))
            }
        }
    }
}

/// What the stages produce.
struct Finished {
    score: Score,
    provenance: BTreeMap<String, f64>,
    nav: Option<NavTree>,
}

/// Why the stages stopped early.
enum Stop {
    Cancelled,
    Error(Box<Diagnostic>),
    /// The errors are already in the run's diagnostics.
    Reported,
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
    /// The stages. `written` is the spec as written, which a template merges with;
    /// `resolved` has its defaults.
    fn stages(
        &mut self,
        written: &Spec,
        resolved: &Spec,
        spec_hash: Sha256Digest,
    ) -> Result<Finished, Stop> {
        let spec = resolved;
        // Chart corrections (RFC-0004 §6, RFC-0005) are applied by the chart source that
        // reads the chart (PLOTSONA, RFC-0005 amendment 3), before `transform`.
        let mut data = self.resolve_source(spec)?;
        self.run_transforms(
            &mut data,
            &spec.transform,
            SpecPointer::root().key("transform"),
        )?;
        let expanded;
        let spec = match &written.template {
            Some(template) => {
                expanded = self.expand(written, template, &mut data)?;
                &expanded
            }
            None => spec,
        };
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
        // The lowest IR version that can read the Score (RFC-0007 §4).
        score.header.ir_version = score.required_ir_version();
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
        let nav = self.navigate(spec, &score, &data, &annotations)?;
        Ok(Finished {
            provenance: data.provenance_mix(),
            score,
            nav,
        })
    }

    /// Applies `transforms`, located at `at`, to `data`.
    fn run_transforms(
        &mut self,
        data: &mut D,
        transforms: &[Transform],
        at: SpecPointer,
    ) -> Result<(), Stop> {
        if transforms.is_empty() {
            return Ok(());
        }
        match data.transform(transforms, &at) {
            Some(Ok(diagnostics)) => self.diagnostics.extend(diagnostics),
            Some(Err(diagnostic)) => return Err(Stop::Error(diagnostic)),
            // Reported, never dropped silently.
            None => self.diagnostics.push(
                Diagnostic::new(
                    &codes::TRANSFORMS_NOT_APPLIED,
                    "This compiler's data type cannot apply `transform`; the data is used as loaded.",
                )
                .at(at),
            ),
        }
        self.cx.check_cancelled().map_err(stop)
    }

    /// Expands `template` against `data` (RFC-0004 §2) and returns the merged spec with
    /// its defaults. A computed source replaces `data`; the template's transforms then run.
    fn expand(
        &mut self,
        written: &Spec,
        template: &TemplateRef,
        data: &mut D,
    ) -> Result<Spec, Stop> {
        let registry = self.registry;
        let Some((engine, provider)) = registry.template(&template.id) else {
            let known: Vec<&str> = registry.templates().map(|(id, _)| id).collect();
            return Err(Stop::Error(Box::new(template::unknown(template, &known))));
        };
        self.use_engine(&engine.clone());
        let request = fragment(SpecPointer::root().key("template"), template)?;
        let expansion = provider
            .expand(&request, data, &mut self.cx)
            .map_err(stop)?;
        let merged = template::merge(written, template, &expansion).map_err(Stop::Error)?;
        for diagnostic in merged.diagnostics {
            self.diagnostics.push(diagnostic);
        }
        let prepared = match sonarithm_spec::prepare::prepare_value(merged.value, None) {
            Ok(prepared) => prepared,
            Err(diagnostics) => {
                self.report_new(&diagnostics);
                return Err(Stop::Reported);
            }
        };
        self.report_new(&prepared.diagnostics);
        if merged.computed_source {
            *data = self.resolve_source(&prepared.resolved)?;
        }
        self.run_transforms(
            data,
            &merged.transforms,
            SpecPointer::root().key("template"),
        )?;
        Ok(prepared.resolved)
    }

    /// Adds the diagnostics not already reported (the merged spec repeats the warnings of
    /// the sections written in the spec).
    fn report_new(&mut self, diagnostics: &Diagnostics) {
        for diagnostic in diagnostics {
            if !self.diagnostics.iter().any(|seen| seen == diagnostic) {
                self.diagnostics.push(diagnostic.clone());
            }
        }
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
            let request = track_request(spec, track, mark, at)?;
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
        if spec.context.is_none() {
            return Ok(());
        }
        let at = SpecPointer::root().key("context");
        let providers: Vec<_> = registry.context_providers().collect();
        // #434: the warning is about the `context` cues, so only a provider that serves them
        // counts; function cues or a monitoring soundscape do not.
        if !providers
            .iter()
            .any(|(_, provider)| provider.serves_context_cues())
        {
            self.diagnostics.push(
                Diagnostic::new(
                    &codes::CONTEXT_UNAVAILABLE,
                    "Context cues (ticks, reference tone, legend, earcons) are not available: no engine that provides them (AUDAXIS) is registered in this build.",
                )
                .at(at),
            );
        }
        // The whole resolved spec (pointer ""): context cues depend on the tracks, the
        // substrate and the field semantics as well as `context` (05 §3).
        let params = fragment(SpecPointer::root(), spec)?;
        for (engine, provider) in providers {
            let tracks = provider
                .context_tracks(&params, score, data, annotations, &mut self.cx)
                .map_err(stop)?;
            // #434: the header lists engines that contributed content, so a provider is
            // recorded only when it adds a track.
            if !tracks.is_empty() {
                self.use_engine(&engine.clone());
            }
            for track in tracks {
                self.charge(track.events.len())?;
                score.tracks.push(track);
            }
        }
        self.cx.check_cancelled().map_err(stop)
    }

    /// The navigation tree of the finished Score: the first one a context provider builds
    /// (05 §8). Building it records no engine in the header, because the tree is not part
    /// of the Score.
    fn navigate(
        &mut self,
        spec: &Spec,
        score: &Score,
        data: &D,
        annotations: &[Annotation],
    ) -> Result<Option<NavTree>, Stop> {
        let registry = self.registry;
        let mut providers = registry.context_providers().peekable();
        if providers.peek().is_none() {
            return Ok(None);
        }
        let whole = fragment(SpecPointer::root(), spec)?;
        for (_, provider) in providers {
            let tree = provider
                .nav_tree(&whole, score, data, annotations, &mut self.cx)
                .map_err(stop)?;
            if tree.is_some() {
                self.cx.check_cancelled().map_err(stop)?;
                return Ok(tree);
            }
        }
        Ok(None)
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

/// The keys of the known experimental switches the spec turns on, sorted (RFC-0006 §1).
fn experiments_on(spec: &Spec) -> Vec<String> {
    spec.experimental
        .as_ref()
        .map(|block| block.on().into_iter().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// One `SNR-COMP-037` Info per switch that is on, naming it and its evidence (RFC-0006 §4).
fn experiment_notes(experiments: &[String]) -> Vec<Diagnostic> {
    experiments
        .iter()
        .filter_map(|key| sonarithm_spec::experimental::experiment(key))
        .map(|experiment| {
            let mut diagnostic = Diagnostic::new(
                &codes::EXPERIMENT_ON,
                format!(
                    "Experimental: {} ({}, not yet backed by evidence).",
                    experiment.summary,
                    experiment.evidence.join(", ")
                ),
            )
            .at(SpecPointer::root().key("experimental").key(experiment.key));
            diagnostic.rules = experiment
                .evidence
                .iter()
                .copied()
                .filter(|id| id.starts_with("SDR-") || id.starts_with("A11Y-"))
                .map(RuleId::from_static)
                .collect();
            diagnostic
        })
        .collect()
}

/// The request for one track: the track, the substrate, the declared field semantics,
/// the missing-value treatment, the mode and the experiments that are on.
fn track_request(
    spec: &Spec,
    track: &sonarithm_spec::TrackSpec,
    mark: MarkKind,
    at: SpecPointer,
) -> Result<TrackRequest, Stop> {
    let root = SpecPointer::root();
    let mode = spec.mode.unwrap_or(sonarithm_spec::Mode::Analysis).into();
    let mut request = TrackRequest::new(TrackId::new(track.id.clone()), mark, fragment(at, track)?)
        .with_mode(mode)
        .with_experiments(experiments_on(spec).into_iter().collect());
    if let Some(substrate) = &spec.substrate {
        request = request.with_substrate(fragment(root.key("substrate"), substrate)?);
    }
    if !spec.data.fields.is_empty() {
        let at = root.key("data").key("fields");
        request = request.with_fields(fragment(at, &spec.data.fields)?);
    }
    if let Some(missing) = &spec.data.missing {
        let at = root.key("data").key("missing");
        request = request.with_missing(fragment(at, missing)?);
    }
    Ok(request)
}

/// Converts a capability error into a stop reason.
fn stop(error: CompileError) -> Stop {
    match error {
        CompileError::Cancelled => Stop::Cancelled,
        CompileError::Diagnostic(diagnostic) => Stop::Error(diagnostic),
        error @ CompileError::Binding { .. } => Stop::Error(Box::new(
            template::binding_diagnostic(&error)
                .unwrap_or_else(|| Diagnostic::new(&codes::INVALID_BINDING, format!("{error}."))),
        )),
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
            sonarithm_core::score::ScoreEvent::Audify(audify) => audify.t.0 + audify.duration().0,
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use sonarithm_core::testing::valid_score;

    use super::duration;

    proptest! {
        /// The header duration the compiler stamps covers every event of any valid Score,
        /// and never exceeds what the Score's own invariant allows. Uses core's shared
        /// strategies through `test-support` (#121).
        #[test]
        fn computed_duration_covers_every_event(score in valid_score()) {
            let computed = duration(&score);
            prop_assert!(computed.0 <= score.header.duration.0);
            for event in score.tracks.iter().flat_map(|track| &track.events) {
                prop_assert!(event.start().0 <= computed.0);
            }
        }
    }
}
