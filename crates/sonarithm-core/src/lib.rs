// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Foundational SONARITHM types: units (`Hz`, `Cents`, `Db`, `Seconds`, `Frames`), IDs,
//! the `Score` IR, engine capability traits, provenance and diagnostics.
//!
//! This crate performs no I/O and depends on no other SONARITHM crate, async runtime,
//! audio device, file-system or network crate (docs/architecture/02-repository-and-crate-layout.md
//! §3 rule 1, enforced by `cargo xtask check-deps`).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codes;
pub mod diagnostic;
pub mod ids;
pub mod math;
pub mod provenance;
pub mod units;

pub use diagnostic::{Code, Diagnostic, Diagnostics, Severity, SpecPointer};
pub use ids::{AnalyzerId, EngineId, Generation, RowRange, RowRef, RuleId, TableId, TrackId};
pub use provenance::{Confidence, Provenance, ProvenanceClass};
pub use units::{Cents, Db, Frames, Hz, Midi, Pan, SampleRate, Seconds};
