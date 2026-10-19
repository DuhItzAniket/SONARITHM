// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Engine capability traits and the engine registry (05 §3, ADR-0007, ADR-0014).
//!
//! The nine engines are capability providers inside one compiler. Each implements
//! [`Engine`], whose [`Engine::register`] adds capability objects (sources, analyzers,
//! encoders, context providers, shapers) to a [`Registry`]. The registry is a plain value,
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

use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::diagnostic::{Diagnostic, SpecPointer};
use crate::ids::{AnalyzerId, EngineId, RowRange, RuleId, TrackId};
use crate::provenance::Confidence;
use crate::score::{EngineVersion, Score, ScoreTrack};

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

/// What an encoder is asked to encode.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TrackRequest {
    /// The track ID.
    pub id: TrackId,
    /// The track's mark.
    pub mark: MarkKind,
    /// The track object of the spec (`/tracks/<n>`).
    pub spec: SpecFragment,
}

impl TrackRequest {
    /// A request for track `id` with `mark`.
    #[must_use]
    pub fn new(id: TrackId, mark: MarkKind, spec: SpecFragment) -> Self {
        Self { id, mark, spec }
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

/// Adds context tracks: ticks, reference tones, legends, earcons (AUDAXIS). Navigation-tree
/// building joins this trait in B03.
pub trait ContextProvider<D>: Send + Sync {
    /// The context tracks for the encoded `score`, from the spec's `context` object.
    ///
    /// # Errors
    ///
    /// A [`CompileError`] when the context cannot be built.
    fn context_tracks(
        &self,
        context: &SpecFragment,
        score: &Score,
        data: &D,
        annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<Vec<ScoreTrack>, CompileError>;
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
