// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The engine registry and its conflict rules (05 §3).
//!
//! Resolution rules:
//!
//! 1. A track's `engine` selects its encoder. Without one, the default encoder for the
//!    track's mark is used: the engine that claimed the default with
//!    [`Registrar::default_encoder`], or else the first engine that registered the mark.
//! 2. Two engines may not register the same [`SourceKind`], [`AnalyzerId`] or
//!    [`ShaperStage`], nor both claim the default encoder of a [`MarkKind`]. Registration
//!    fails fast with a [`RegistryError`], which the compiler reports as `SNR-COMP-0xx`.
//!
//! Every map is a `BTreeMap` and context providers keep registration order, so lookups and
//! iteration are deterministic. After an error the registry may hold part of the failing
//! engine's capabilities and should be discarded.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::{
    Analyzer, ContextProvider, Encoder, Engine, EngineManifest, MarkKind, Shaper, Source,
    SourceKind,
};
use crate::ids::{AnalyzerId, EngineId};
use crate::score::ShaperStage;

/// A registration conflict (05 §3 rule 3).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum RegistryError {
    /// An engine ID was registered twice.
    #[error("engine `{engine}` is registered twice")]
    DuplicateEngine {
        /// The engine.
        engine: EngineId,
    },
    /// Two engines register the same source kind.
    #[error("engines `{first}` and `{second}` both register the `{kind}` source")]
    DuplicateSource {
        /// The source kind.
        kind: SourceKind,
        /// The engine registered first.
        first: EngineId,
        /// The engine registered second.
        second: EngineId,
    },
    /// Two engines register the same analyzer ID.
    #[error("engines `{first}` and `{second}` both register the `{id}` analyzer")]
    DuplicateAnalyzer {
        /// The analyzer ID.
        id: AnalyzerId,
        /// The engine registered first.
        first: EngineId,
        /// The engine registered second.
        second: EngineId,
    },
    /// An engine registers an encoder for a mark twice.
    #[error("engine `{engine}` registers two encoders for the `{mark}` mark")]
    DuplicateEncoder {
        /// The mark.
        mark: MarkKind,
        /// The engine.
        engine: EngineId,
    },
    /// Two engines claim the default encoder of a mark, or an engine claims the default for
    /// a mark it has no encoder for.
    #[error("engine `{second}` cannot claim the default `{mark}` encoder; `{first}` holds it")]
    DuplicateMarkDefault {
        /// The mark.
        mark: MarkKind,
        /// The engine that holds the default (the claimant itself if it has no encoder).
        first: EngineId,
        /// The engine that tried to claim it.
        second: EngineId,
    },
    /// Two engines register a shaper for the same stage.
    #[error("engines `{first}` and `{second}` both register the `{stage:?}` shaper")]
    DuplicateShaper {
        /// The stage.
        stage: ShaperStage,
        /// The engine registered first.
        first: EngineId,
        /// The engine registered second.
        second: EngineId,
    },
}

/// A capability and the engine that registered it.
struct Entry<T: ?Sized> {
    engine: EngineId,
    capability: Arc<T>,
}

/// Every registered engine and capability. `D` is the compiler's data type (ADR-0014).
pub struct Registry<D> {
    engines: BTreeMap<EngineId, EngineManifest>,
    sources: BTreeMap<SourceKind, Entry<dyn Source<D>>>,
    analyzers: BTreeMap<AnalyzerId, Entry<dyn Analyzer<D>>>,
    encoders: BTreeMap<(MarkKind, EngineId), Arc<dyn Encoder<D>>>,
    first_encoder: BTreeMap<MarkKind, EngineId>,
    default_encoder: BTreeMap<MarkKind, EngineId>,
    context: Vec<Entry<dyn ContextProvider<D>>>,
    shapers: BTreeMap<ShaperStage, Entry<dyn Shaper>>,
}

impl<D> Default for Registry<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D> Registry<D> {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            engines: BTreeMap::new(),
            sources: BTreeMap::new(),
            analyzers: BTreeMap::new(),
            encoders: BTreeMap::new(),
            first_encoder: BTreeMap::new(),
            default_encoder: BTreeMap::new(),
            context: Vec::new(),
            shapers: BTreeMap::new(),
        }
    }

    /// Registers `engine` and every capability it adds.
    ///
    /// # Errors
    ///
    /// A [`RegistryError`] for a duplicate engine or a conflicting capability.
    pub fn add_engine(&mut self, engine: &dyn Engine<D>) -> Result<(), RegistryError> {
        let manifest = engine.manifest();
        let id = manifest.id.clone();
        if self.engines.contains_key(&id) {
            return Err(RegistryError::DuplicateEngine { engine: id });
        }
        self.engines.insert(id.clone(), manifest);
        engine.register(&mut Registrar {
            registry: self,
            engine: id,
        })
    }

    /// The manifests of the registered engines, by engine ID.
    pub fn engines(&self) -> impl Iterator<Item = &EngineManifest> {
        self.engines.values()
    }

    /// Whether an engine is registered.
    #[must_use]
    pub fn has_engine(&self, id: &EngineId) -> bool {
        self.engines.contains_key(id)
    }

    /// The source for `kind`, with its engine.
    #[must_use]
    pub fn source(&self, kind: SourceKind) -> Option<(&EngineId, &dyn Source<D>)> {
        self.sources
            .get(&kind)
            .map(|entry| (&entry.engine, &*entry.capability))
    }

    /// The analyzer `id`, with its engine.
    #[must_use]
    pub fn analyzer(&self, id: &AnalyzerId) -> Option<(&EngineId, &dyn Analyzer<D>)> {
        self.analyzers
            .get(id)
            .map(|entry| (&entry.engine, &*entry.capability))
    }

    /// The encoder for `mark`: `engine`'s if given, otherwise the default (rule 1).
    #[must_use]
    pub fn encoder(
        &self,
        mark: MarkKind,
        engine: Option<&EngineId>,
    ) -> Option<(&EngineId, &dyn Encoder<D>)> {
        let engine = match engine {
            Some(engine) => engine,
            None => self
                .default_encoder
                .get(&mark)
                .or_else(|| self.first_encoder.get(&mark))?,
        };
        self.encoders
            .get_key_value(&(mark, engine.clone()))
            .map(|((_, engine), encoder)| (engine, &**encoder))
    }

    /// The context providers in registration order.
    pub fn context_providers(&self) -> impl Iterator<Item = (&EngineId, &dyn ContextProvider<D>)> {
        self.context
            .iter()
            .map(|entry| (&entry.engine, &*entry.capability))
    }

    /// The shapers in stage order (05 §2).
    pub fn shapers(&self) -> impl Iterator<Item = (ShaperStage, &EngineId, &dyn Shaper)> {
        self.shapers
            .iter()
            .map(|(stage, entry)| (*stage, &entry.engine, &*entry.capability))
    }
}

/// Adds one engine's capabilities to a [`Registry`]; handed to [`Engine::register`].
pub struct Registrar<'a, D> {
    registry: &'a mut Registry<D>,
    engine: EngineId,
}

impl<D> Registrar<'_, D> {
    /// The engine being registered.
    #[must_use]
    pub fn engine(&self) -> &EngineId {
        &self.engine
    }

    /// Registers the source for `kind`.
    ///
    /// # Errors
    ///
    /// [`RegistryError::DuplicateSource`] if another engine registered `kind`.
    pub fn source(
        &mut self,
        kind: SourceKind,
        source: Arc<dyn Source<D>>,
    ) -> Result<(), RegistryError> {
        if let Some(first) = self.registry.sources.get(&kind) {
            return Err(RegistryError::DuplicateSource {
                kind,
                first: first.engine.clone(),
                second: self.engine.clone(),
            });
        }
        self.registry.sources.insert(kind, self.entry(source));
        Ok(())
    }

    /// Registers the analyzer `id`.
    ///
    /// # Errors
    ///
    /// [`RegistryError::DuplicateAnalyzer`] if another engine registered `id`.
    pub fn analyzer(
        &mut self,
        id: AnalyzerId,
        analyzer: Arc<dyn Analyzer<D>>,
    ) -> Result<(), RegistryError> {
        if let Some(first) = self.registry.analyzers.get(&id) {
            return Err(RegistryError::DuplicateAnalyzer {
                id,
                first: first.engine.clone(),
                second: self.engine.clone(),
            });
        }
        let entry = self.entry(analyzer);
        self.registry.analyzers.insert(id, entry);
        Ok(())
    }

    /// Registers `encoder` for each of `marks`. The first engine to register a mark is its
    /// default unless another engine claims it with [`Registrar::default_encoder`].
    ///
    /// # Errors
    ///
    /// [`RegistryError::DuplicateEncoder`] if this engine already has an encoder for a mark.
    pub fn encoder(
        &mut self,
        marks: &[MarkKind],
        encoder: &Arc<dyn Encoder<D>>,
    ) -> Result<(), RegistryError> {
        for mark in marks {
            let key = (*mark, self.engine.clone());
            if self.registry.encoders.contains_key(&key) {
                return Err(RegistryError::DuplicateEncoder {
                    mark: *mark,
                    engine: self.engine.clone(),
                });
            }
            self.registry.encoders.insert(key, Arc::clone(encoder));
            self.registry
                .first_encoder
                .entry(*mark)
                .or_insert_with(|| self.engine.clone());
        }
        Ok(())
    }

    /// Claims the default encoder for `mark`. Register the encoder first.
    ///
    /// # Errors
    ///
    /// [`RegistryError::DuplicateMarkDefault`] if another engine holds the default, or this
    /// engine has no encoder for `mark`.
    pub fn default_encoder(&mut self, mark: MarkKind) -> Result<(), RegistryError> {
        let has_encoder = self
            .registry
            .encoders
            .contains_key(&(mark, self.engine.clone()));
        let holder = self.registry.default_encoder.get(&mark);
        if !has_encoder || holder.is_some_and(|holder| *holder != self.engine) {
            return Err(RegistryError::DuplicateMarkDefault {
                mark,
                first: holder.cloned().unwrap_or_else(|| self.engine.clone()),
                second: self.engine.clone(),
            });
        }
        self.registry
            .default_encoder
            .insert(mark, self.engine.clone());
        Ok(())
    }

    /// Registers a context provider. Providers run in registration order.
    pub fn context(&mut self, provider: Arc<dyn ContextProvider<D>>) {
        let entry = self.entry(provider);
        self.registry.context.push(entry);
    }

    /// Registers the shaper for `stage`.
    ///
    /// # Errors
    ///
    /// [`RegistryError::DuplicateShaper`] if another engine registered `stage`.
    pub fn shaper(
        &mut self,
        stage: ShaperStage,
        shaper: Arc<dyn Shaper>,
    ) -> Result<(), RegistryError> {
        if let Some(first) = self.registry.shapers.get(&stage) {
            return Err(RegistryError::DuplicateShaper {
                stage,
                first: first.engine.clone(),
                second: self.engine.clone(),
            });
        }
        let entry = self.entry(shaper);
        self.registry.shapers.insert(stage, entry);
        Ok(())
    }

    fn entry<T: ?Sized>(&self, capability: Arc<T>) -> Entry<T> {
        Entry {
            engine: self.engine.clone(),
            capability,
        }
    }
}
