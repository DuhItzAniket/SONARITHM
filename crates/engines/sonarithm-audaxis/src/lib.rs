// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! AUDAXIS, SONARITHM's navigation and context engine (docs/architecture/07-engines.md,
//! AUDAXIS; 08).
//!
//! AUDAXIS registers one context provider and the `Spatial` shaper ([`spatial`]: stereo
//! placement and the never-alone guard, A11Y-14). The context provider builds the navigation tree
//! ([`nav::tree`], 08 §2) that the compiler returns beside the Score, and the context
//! tracks ([`context`]: ticks, the reference tone, the spoken legend). The [`nav::Cursor`]
//! moves through the tree with the verbs of keyboard-and-navigation §4, and [`announce`]
//! turns its outcomes into catalogue text for the screen reader. The full earcon set and
//! spatial placement follow in B03.
//!
//! The engine reads its own part of the spec and the data; it never depends on another
//! engine (02 §3 rule 2). Everything it builds is ordered and deterministic (ADR-0013).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod announce;
pub mod braille;
pub mod codes;
mod columns;
pub mod context;
pub mod earcons;
pub mod nav;
pub mod spatial;

use std::borrow::Borrow;
use std::sync::Arc;

use sonarithm_core::engine::{
    Annotation, CompileContext, CompileError, ContextProvider, Engine, EngineManifest, Registrar,
    RegistryError, SpecFragment,
};
use sonarithm_core::ids::EngineId;
use sonarithm_core::nav::NavTree;
use sonarithm_core::score::{EngineVersion, Score, ScoreTrack, ShaperStage};
use sonarithm_data::DataBundle;

/// The engine ID.
pub const ENGINE: EngineId = EngineId::from_static("audaxis");

/// The AUDAXIS engine.
#[derive(Clone, Debug, Default)]
pub struct Audaxis;

impl Audaxis {
    /// The engine.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// The engine's manifest.
    #[must_use]
    pub fn manifest() -> EngineManifest {
        let mut manifest = EngineManifest::new(
            ENGINE,
            "AUDAXIS",
            EngineVersion(env!("CARGO_PKG_VERSION").to_owned()),
        );
        manifest.spec_keys = vec!["context".to_owned()];
        manifest
    }
}

impl<D: Borrow<DataBundle> + 'static> Engine<D> for Audaxis {
    fn manifest(&self) -> EngineManifest {
        Self::manifest()
    }

    fn register(&self, registrar: &mut Registrar<'_, D>) -> Result<(), RegistryError> {
        registrar.context(Arc::new(Context));
        registrar.shaper(ShaperStage::Spatial, Arc::new(spatial::Spatial))
    }
}

/// The context provider AUDAXIS registers.
struct Context;

impl<D: Borrow<DataBundle>> ContextProvider<D> for Context {
    fn context_tracks(
        &self,
        spec: &SpecFragment,
        score: &Score,
        data: &D,
        annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<Vec<ScoreTrack>, CompileError> {
        let spec: sonarithm_spec::Spec = spec.parse()?;
        context::tracks(&spec, score, data.borrow(), annotations, cx)
    }

    fn nav_tree(
        &self,
        spec: &SpecFragment,
        score: &Score,
        data: &D,
        annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<Option<NavTree>, CompileError> {
        let spec: sonarithm_spec::Spec = spec.parse()?;
        nav::tree::build(&spec, score, data.borrow(), annotations, cx).map(Some)
    }
}
