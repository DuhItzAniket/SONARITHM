// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! AUDAXIS, SONARITHM's navigation and context engine (docs/architecture/07-engines.md,
//! AUDAXIS; 08).
//!
//! AUDAXIS registers one context provider. It builds the navigation tree
//! ([`nav::tree`], 08 §2) that the compiler returns beside the Score, and, from B03-T05,
//! the context tracks (ticks, the reference tone, the spoken legend). The cursor and its
//! verbs, the announcement composer, the earcons and spatial placement follow in B03.
//!
//! The engine reads its own part of the spec and the data; it never depends on another
//! engine (02 §3 rule 2). Everything it builds is ordered and deterministic (ADR-0013).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod columns;
pub mod nav;

use std::borrow::Borrow;
use std::sync::Arc;

use sonarithm_core::engine::{
    Annotation, CompileContext, CompileError, ContextProvider, Engine, EngineManifest, Registrar,
    RegistryError, SpecFragment,
};
use sonarithm_core::ids::EngineId;
use sonarithm_core::nav::NavTree;
use sonarithm_core::score::{EngineVersion, Score, ScoreTrack};
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
        Ok(())
    }
}

/// The context provider AUDAXIS registers.
struct Context;

impl<D: Borrow<DataBundle>> ContextProvider<D> for Context {
    fn context_tracks(
        &self,
        _context: &SpecFragment,
        _score: &Score,
        _data: &D,
        _annotations: &[Annotation],
        _cx: &mut CompileContext,
    ) -> Result<Vec<ScoreTrack>, CompileError> {
        // The tick track, the reference tone and the spoken legend arrive in B03-T05.
        Ok(Vec::new())
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
