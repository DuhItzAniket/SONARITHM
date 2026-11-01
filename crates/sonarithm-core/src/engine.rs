// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Engine capability traits and the engine registry (05 §3, ADR-0007, ADR-0014).
//!
//! The nine engines are capability providers inside one compiler. Each implements
//! [`Engine`], whose [`Engine::register`] adds capability objects (sources, analyzers,
//! encoders, context providers, shapers, templates) to a [`Registry`]. The registry is a plain value,
//! not a global, so tests, Python and WASM builds construct their own.
//!
//! Per ADR-0014 the traits are generic over the data type `D` the compiler passes around
//! (the Arrow-backed `DataBundle` of `sonarithm-data` from A04), and engines receive their
//! part of the spec as a [`SpecFragment`] that they parse into `sonarithm-spec` types. This
//! keeps `sonarithm-core` free of spec and data dependencies (02 §3 rule 1).

pub mod context;
pub mod registry;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::diagnostic::{Diagnostic, SpecPointer};
use crate::ids::{AnalyzerId, EngineId, RowRange, RuleId, TrackId};
use crate::nav::NavTree;
use crate::provenance::Confidence;
use crate::score::{EngineVersion, Mode, Score, ScoreTrack};

pub use context::{Budget, BudgetExceeded, CancelToken, CompileContext, TrackRng};
pub use registry::{Registrar, Registry, RegistryError};

/// A mark kind (04 §5.3): what an encoder turns data into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MarkKind {
    /// One discrete note per datum (parameter-mapping sonification).
    Tone,
    /// A continuous contour (parameter-mapping sonification).
    Glide,
    /// A grain cloud (TONALENS).
    Grain,
    /// The series played as a waveform (CURVASONA).
    Audify,
    /// A recorded sound per event (auditory icon).
    Sample,
    /// An excitable data model (RESONUM).
    Model,
    /// Tone, glide or audification chosen from density (SDR-20, experimental).
    Auto,
}

/// A data source kind (04 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SourceKind {
    /// A data file.
    File,
    /// Values written in the spec.
    Inline,
    /// A chart image or document (PLOTSONA).
    Chart,
    /// A sampled function (CURVASONA).
    Function,
    /// A live stream (SONOTRACE).
    Stream,
    /// A dataset already loaded in a project.
    Dataset,
}

/// What an engine is, for registries, manifests and the Score header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EngineManifest {
    /// The engine ID, such as `numisona`.
    pub id: EngineId,
    /// The display name, such as `NUMISONA`.
    pub name: String,
    /// The engine crate version.
    pub version: EngineVersion,
    /// The spec keys the engine owns (07 summary matrix), such as `music` or
    /// `data.source.kind=function`.
    pub spec_keys: Vec<String>,
}

impl EngineManifest {
    /// A manifest that owns no spec keys yet.
    #[must_use]
    pub fn new(id: EngineId, name: impl Into<String>, version: EngineVersion) -> Self {
        Self {
            id,
            name: name.into(),
            version,
            spec_keys: Vec::new(),
        }
    }
}

/// The part of a validated spec a capability needs, as compact JSON plus its location.
///
/// Engines parse it into `sonarithm-spec` types with [`SpecFragment::parse`]; the same
/// representation crosses the plugin boundary in Stage D (ADR-0009).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecFragment {
    pointer: SpecPointer,
    json: String,
}

impl SpecFragment {
    /// The fragment of `value` found at `pointer` in the spec.
    ///
    /// # Errors
    ///
    /// [`CompileError::Fragment`] if `value` cannot be serialized.
    pub fn new<T: Serialize + ?Sized>(
        pointer: SpecPointer,
        value: &T,
    ) -> Result<Self, CompileError> {
        match serde_json::to_string(value) {
            Ok(json) => Ok(Self { pointer, json }),
            Err(err) => Err(CompileError::Fragment {
                pointer,
                message: err.to_string(),
            }),
        }
    }

    /// Where the fragment is in the spec.
    #[must_use]
    pub fn pointer(&self) -> &SpecPointer {
        &self.pointer
    }

    /// The fragment as compact JSON.
    #[must_use]
    pub fn json(&self) -> &str {
        &self.json
    }

    /// The fragment as a typed value.
    ///
    /// # Errors
    ///
    /// [`CompileError::Fragment`] if the JSON does not match `T`.
    pub fn parse<T: DeserializeOwned>(&self) -> Result<T, CompileError> {
        serde_json::from_str(&self.json).map_err(|err| CompileError::Fragment {
            pointer: self.pointer.clone(),
            message: err.to_string(),
        })
    }
}

/// One analyzer result: an extremum, outlier, change point, critical point or insight.
/// Annotations are kept apart from the data, so re-analysis never mutates it (03 §2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Annotation {
    /// The analyzer that produced it.
    pub source: AnalyzerId,
    /// A short label, also the text equivalent (A11Y-24).
    pub label: String,
    /// The rows it covers.
    pub range: RowRange,
    /// The value it reports, if any.
    pub value: Option<f64>,
    /// How confident the analyzer is.
    pub confidence: Confidence,
    /// The design rules behind it.
    pub rules: Vec<RuleId>,
}

impl Annotation {
    /// A certain annotation with no value and no rules.
    #[must_use]
    pub fn new(source: AnalyzerId, label: impl Into<String>, range: RowRange) -> Self {
        Self {
            source,
            label: label.into(),
            range,
            value: None,
            confidence: Confidence::CERTAIN,
            rules: Vec::new(),
        }
    }
}

/// What an encoder is asked to encode: the track, plus the spec sections every track's
/// encoding depends on (05 §2). Encoders own the `substrate` (RFC-0001 §2), and they read
/// field semantics and the missing-value treatment from `data`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TrackRequest {
    /// The track ID.
    pub id: TrackId,
    /// The track's mark.
    pub mark: MarkKind,
    /// The track object of the spec (`/tracks/<n>`).
    pub spec: SpecFragment,
    /// The spec's `substrate` (`/substrate`): the time field and pace.
    pub substrate: Option<SpecFragment>,
    /// The spec's declared field semantics (`/data/fields`), which win over the data's own
    /// metadata (04 §3).
    pub fields: Option<SpecFragment>,
    /// The spec's missing-value treatment (`/data/missing`).
    pub missing: Option<SpecFragment>,
    /// The spec's mode (SDR-36).
    pub mode: Mode,
    /// The experimental switches the spec turns on (`experimental`, RFC-0006), known to
    /// this SONARITHM, by key. Each engine acts only on the switches it owns.
    pub experiments: BTreeSet<String>,
}

impl TrackRequest {
    /// A request for track `id` with `mark`, in Analysis mode, without substrate, field
    /// semantics, missing-value treatment or experiments.
    #[must_use]
    pub fn new(id: TrackId, mark: MarkKind, spec: SpecFragment) -> Self {
        Self {
            id,
            mark,
            spec,
            substrate: None,
            fields: None,
            missing: None,
            mode: Mode::Analysis,
            experiments: BTreeSet::new(),
        }
    }

    /// The same request with the spec's `substrate`.
    #[must_use]
    pub fn with_substrate(mut self, substrate: SpecFragment) -> Self {
        self.substrate = Some(substrate);
        self
    }

    /// The same request with the spec's `data.fields`.
    #[must_use]
    pub fn with_fields(mut self, fields: SpecFragment) -> Self {
        self.fields = Some(fields);
        self
    }

    /// The same request with the spec's `data.missing`.
    #[must_use]
    pub fn with_missing(mut self, missing: SpecFragment) -> Self {
        self.missing = Some(missing);
        self
    }

    /// The same request in `mode`.
    #[must_use]
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// The same request with the experimental switches that are on (RFC-0006).
    #[must_use]
    pub fn with_experiments(mut self, experiments: BTreeSet<String>) -> Self {
        self.experiments = experiments;
        self
    }

    /// Whether the spec turns on experimental switch `key` (RFC-0006).
    #[must_use]
    pub fn experiment(&self, key: &str) -> bool {
        self.experiments.contains(key)
    }
}

/// Why a capability failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CompileError {
    /// A problem the user can fix, with its code and location.
    #[error("{0}")]
    Diagnostic(Box<Diagnostic>),
    /// The compile was cancelled through its [`CancelToken`].
    #[error("the compile was cancelled")]
    Cancelled,
    /// The compile exceeded its [`Budget`].
    #[error(transparent)]
    Budget(#[from] BudgetExceeded),
    /// A spec fragment did not have the shape the engine expected; a bug, because the
    /// spec was validated first.
    #[error("spec fragment at {pointer} could not be read: {message}")]
    Fragment {
        /// Where the fragment is.
        pointer: SpecPointer,
        /// The parser's message.
        message: String,
    },
    /// A template's slots could not be bound to the data (RFC-0004 §2).
    #[error("template `{template}`: {error}")]
    Binding {
        /// The template ID.
        template: String,
        /// What went wrong.
        error: BindingError,
    },
    /// An engine failed for a reason that is not the user's to fix.
    #[error("engine {engine} failed: {message}")]
    Engine {
        /// The engine.
        engine: EngineId,
        /// What went wrong.
        message: String,
    },
}

impl From<Diagnostic> for CompileError {
    fn from(diagnostic: Diagnostic) -> Self {
        Self::Diagnostic(Box::new(diagnostic))
    }
}

/// Resolves a data source into the compiler's data type (platform, PLOTSONA, CURVASONA,
/// SONOTRACE).
pub trait Source<D>: Send + Sync {
    /// Reads the source described by `source` (the spec's `data.source` object).
    ///
    /// # Errors
    ///
    /// A [`CompileError`] when the source cannot be read.
    fn ingest(&self, source: &SpecFragment, cx: &mut CompileContext) -> Result<D, CompileError>;
}

/// Finds structure in data: extrema, outliers, change points, critical points (ECHOMETRIC,
/// CURVASONA).
pub trait Analyzer<D>: Send + Sync {
    /// Analyzes `data` with the spec's `analysis` object as parameters.
    ///
    /// # Errors
    ///
    /// A [`CompileError`] when the analysis cannot run.
    fn analyze(
        &self,
        data: &D,
        params: &SpecFragment,
        cx: &mut CompileContext,
    ) -> Result<Vec<Annotation>, CompileError>;
}

/// Turns data into one Score track (NUMISONA, TONALENS, CURVASONA, RESONUM).
pub trait Encoder<D>: Send + Sync {
    /// Encodes one track.
    ///
    /// # Errors
    ///
    /// A [`CompileError`] when the track cannot be encoded.
    fn encode(
        &self,
        track: &TrackRequest,
        data: &D,
        annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError>;
}

/// Adds context tracks (ticks, reference tones, legends, earcons) and builds the navigation
/// tree (AUDAXIS; 05 §2, §8). The navigation tree joined this trait in B03, as ADR-0014
/// foresaw; its default builds none, so providers that only add tracks are unchanged.
pub trait ContextProvider<D>: Send + Sync {
    /// The context tracks for the encoded `score`. `spec` is the whole resolved spec
    /// (pointer `""`): the cues depend on its `context` object and also on the tracks, the
    /// substrate and the field semantics (B03-T05).
    ///
    /// # Errors
    ///
    /// A [`CompileError`] when the context cannot be built.
    fn context_tracks(
        &self,
        spec: &SpecFragment,
        score: &Score,
        data: &D,
        annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<Vec<ScoreTrack>, CompileError>;

    /// Whether this provider serves the cues the spec's `context` object asks for (ticks,
    /// reference tone, legend, earcons; 04 §5.4). The compiler warns (`SNR-COMP-023`) when
    /// no registered provider does; a provider of other context tracks (function cues, a
    /// monitoring soundscape) keeps the default, `false` (#434).
    fn serves_context_cues(&self) -> bool {
        false
    }

    /// The navigation tree of the finished `score` (08 §2), or `None` if this provider
    /// builds none. `spec` is the whole resolved spec (pointer `""`), because the tree needs
    /// the tracks, the substrate, the field semantics and the title. The compiler calls it
    /// after the shapers and keeps the first tree a provider returns.
    ///
    /// # Errors
    ///
    /// A [`CompileError`] when the tree cannot be built.
    fn nav_tree(
        &self,
        spec: &SpecFragment,
        score: &Score,
        data: &D,
        annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<Option<NavTree>, CompileError> {
        let _ = (spec, score, data, annotations, cx);
        Ok(None)
    }
}

/// Why a template's slots could not be bound (RFC-0004 §2). The compiler reports
/// [`BindingError::Invalid`] as `SNR-COMP-031` and [`BindingError::Unbound`] as
/// `SNR-COMP-032`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BindingError {
    /// `bind` names a slot the template does not have, a field the data does not have, or a
    /// field of the wrong kind.
    #[error("slot `{slot}` cannot take field `{field}`: {reason}")]
    Invalid {
        /// The slot.
        slot: String,
        /// The field.
        field: String,
        /// Why, as a short clause.
        reason: String,
    },
    /// A required slot found no field.
    #[error("slot `{slot}` needs a {kind} field, and the data has none left")]
    Unbound {
        /// The slot.
        slot: String,
        /// What it needs, such as `numeric`.
        kind: String,
    },
}

/// What a template expands to (RFC-0004 §2).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TemplateExpansion {
    /// Slot name → field name, as bound.
    pub bindings: BTreeMap<String, String>,
    /// A JSON object of the top-level spec sections the template designs: some of
    /// `transform`, `substrate`, `tracks` and `context`, and `data` holding `fields` or a
    /// computed `source` (values derived from the data, marked `derived`).
    pub sections: SpecFragment,
}

impl TemplateExpansion {
    /// An expansion.
    #[must_use]
    pub const fn new(bindings: BTreeMap<String, String>, sections: SpecFragment) -> Self {
        Self { bindings, sections }
    }
}

/// Expands a spec's `template` into the sections it designs (NUMISONA, PLOTSONA; RFC-0004
/// §2). Designs are data-agnostic (SDR-03): a template never names a field until it is
/// bound.
pub trait TemplateProvider<D>: Send + Sync {
    /// Binds the template's slots to `data`'s fields and returns its sections. `template`
    /// is the spec's `template` object (`id`, `bind`); `data` has had the spec's own
    /// `transform` applied.
    ///
    /// # Errors
    ///
    /// [`CompileError::Binding`] when the slots cannot be bound, or another
    /// [`CompileError`].
    fn expand(
        &self,
        template: &SpecFragment,
        data: &D,
        cx: &mut CompileContext,
    ) -> Result<TemplateExpansion, CompileError>;
}

/// Transforms the merged Score in a fixed slot: quantize, loudness, spatial, lint (05 §2).
pub trait Shaper: Send + Sync {
    /// Shapes the Score in place. The lint shaper reports diagnostics and never mutates.
    ///
    /// # Errors
    ///
    /// A [`CompileError`] when shaping fails.
    fn shape(&self, score: &mut Score, cx: &mut CompileContext) -> Result<(), CompileError>;
}

/// An engine: a manifest plus the capabilities it registers (ADR-0007).
pub trait Engine<D>: Send + Sync {
    /// The engine's manifest.
    fn manifest(&self) -> EngineManifest;

    /// Adds the engine's capabilities.
    ///
    /// # Errors
    ///
    /// A [`RegistryError`] when a capability conflicts with one already registered.
    fn register(&self, registrar: &mut Registrar<'_, D>) -> Result<(), RegistryError>;
}

impl fmt::Display for MarkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Tone => "tone",
            Self::Glide => "glide",
            Self::Grain => "grain",
            Self::Audify => "audify",
            Self::Sample => "sample",
            Self::Model => "model",
            Self::Auto => "auto",
        })
    }
}

impl fmt::Display for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::File => "file",
            Self::Inline => "inline",
            Self::Chart => "chart",
            Self::Function => "function",
            Self::Stream => "stream",
            Self::Dataset => "dataset",
        })
    }
}
