// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Registry resolution and conflict tests (05 §3).

use std::sync::Arc;

use super::*;
use crate::score::{InstrumentRef, ShaperStage, TrackRole};

/// An encoder that returns an empty data track.
struct EmptyEncoder;

impl Encoder<()> for EmptyEncoder {
    fn encode(
        &self,
        track: &TrackRequest,
        (): &(),
        _: &[Annotation],
        _: &mut CompileContext,
    ) -> Result<ScoreTrack, CompileError> {
        Ok(ScoreTrack::new(
            track.id.clone(),
            TrackRole::Data,
            EngineId::from_static("test"),
            InstrumentRef("sine".to_owned()),
        ))
    }
}

struct NoSource;

impl Source<()> for NoSource {
    fn ingest(&self, _: &SpecFragment, _: &mut CompileContext) -> Result<(), CompileError> {
        Ok(())
    }
}

struct NoShaper;

impl Shaper for NoShaper {
    fn shape(&self, _: &mut Score, _: &mut CompileContext) -> Result<(), CompileError> {
        Ok(())
    }
}

/// What a fake engine registers.
#[derive(Clone, Default)]
struct Fake {
    id: &'static str,
    marks: Vec<MarkKind>,
    default: Option<MarkKind>,
    source: Option<SourceKind>,
    shaper: Option<ShaperStage>,
}

impl Engine<()> for Fake {
    fn manifest(&self) -> EngineManifest {
        EngineManifest::new(
            EngineId::from_static(self.id),
            self.id.to_uppercase(),
            EngineVersion("0.0.0".to_owned()),
        )
    }

    fn register(&self, registrar: &mut Registrar<'_, ()>) -> Result<(), RegistryError> {
        if !self.marks.is_empty() {
            let encoder: Arc<dyn Encoder<()>> = Arc::new(EmptyEncoder);
            registrar.encoder(&self.marks, &encoder)?;
        }
        if let Some(mark) = self.default {
            registrar.default_encoder(mark)?;
        }
        if let Some(kind) = self.source {
            registrar.source(kind, Arc::new(NoSource))?;
        }
        if let Some(stage) = self.shaper {
            registrar.shaper(stage, Arc::new(NoShaper))?;
        }
        Ok(())
    }
}

fn numisona() -> Fake {
    Fake {
        id: "numisona",
        marks: vec![MarkKind::Tone, MarkKind::Glide],
        ..Fake::default()
    }
}

fn id(engine: &'static str) -> EngineId {
    EngineId::from_static(engine)
}

#[test]
fn the_first_engine_for_a_mark_is_its_default_unless_another_claims_it() {
    let mut registry = Registry::new();
    registry.add_engine(&numisona()).unwrap();
    let plotsona = Fake {
        id: "plotsona",
        marks: vec![MarkKind::Tone],
        ..Fake::default()
    };
    registry.add_engine(&plotsona).unwrap();
    let engine_for = |registry: &Registry<()>, mark, engine: Option<&EngineId>| {
        registry
            .encoder(mark, engine)
            .map(|(engine, _)| engine.clone())
    };
    assert_eq!(
        engine_for(&registry, MarkKind::Tone, None),
        Some(id("numisona"))
    );
    assert_eq!(
        engine_for(&registry, MarkKind::Tone, Some(&id("plotsona"))),
        Some(id("plotsona"))
    );
    assert_eq!(engine_for(&registry, MarkKind::Grain, None), None);

    let mut claimed = Registry::new();
    claimed.add_engine(&numisona()).unwrap();
    let claimant = Fake {
        default: Some(MarkKind::Tone),
        ..plotsona
    };
    claimed.add_engine(&claimant).unwrap();
    assert_eq!(
        engine_for(&claimed, MarkKind::Tone, None),
        Some(id("plotsona"))
    );
}

#[test]
fn registering_an_engine_twice_fails() {
    let mut registry = Registry::new();
    registry.add_engine(&numisona()).unwrap();
    assert_eq!(
        registry.add_engine(&numisona()),
        Err(RegistryError::DuplicateEngine {
            engine: id("numisona")
        })
    );
}

#[test]
fn two_engines_cannot_share_a_source_shaper_or_mark_default() {
    let source = |engine| Fake {
        id: engine,
        source: Some(SourceKind::Function),
        ..Fake::default()
    };
    let mut registry = Registry::new();
    registry.add_engine(&source("curvasona")).unwrap();
    assert!(matches!(
        registry.add_engine(&source("plotsona")),
        Err(RegistryError::DuplicateSource {
            kind: SourceKind::Function,
            ..
        })
    ));

    let shaper = |engine| Fake {
        id: engine,
        shaper: Some(ShaperStage::Lint),
        ..Fake::default()
    };
    let mut registry = Registry::new();
    registry.add_engine(&shaper("echometric")).unwrap();
    assert!(matches!(
        registry.add_engine(&shaper("audaxis")),
        Err(RegistryError::DuplicateShaper {
            stage: ShaperStage::Lint,
            ..
        })
    ));

    let claim = |engine| Fake {
        id: engine,
        marks: vec![MarkKind::Tone],
        default: Some(MarkKind::Tone),
        ..Fake::default()
    };
    let mut registry = Registry::new();
    registry.add_engine(&claim("numisona")).unwrap();
    assert!(matches!(
        registry.add_engine(&claim("tonalens")),
        Err(RegistryError::DuplicateMarkDefault {
            mark: MarkKind::Tone,
            ..
        })
    ));
}

#[test]
fn an_engine_cannot_claim_a_default_without_an_encoder() {
    let mut registry = Registry::new();
    let claimant = Fake {
        id: "audaxis",
        default: Some(MarkKind::Tone),
        ..Fake::default()
    };
    assert!(matches!(
        registry.add_engine(&claimant),
        Err(RegistryError::DuplicateMarkDefault { .. })
    ));
}

#[test]
fn spec_fragments_parse_into_typed_values() {
    let pointer = SpecPointer::root().key("tracks").index(0);
    let fragment = SpecFragment::new(pointer.clone(), &[1, 2, 3]).unwrap();
    assert_eq!(fragment.json(), "[1,2,3]");
    assert_eq!(fragment.parse::<Vec<u8>>().unwrap(), [1, 2, 3]);
    assert!(matches!(
        fragment.parse::<String>(),
        Err(CompileError::Fragment { pointer: p, .. }) if p == pointer
    ));
}
