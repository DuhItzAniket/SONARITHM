// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM data layer: an Arrow-backed `DataBundle`, file sources, type and concept
//! inference, transforms and M4/LTTB downsampling, with provenance carried on every value
//! (docs/architecture/03-data-model.md, ADR-0006).
//!
//! Field semantics live in Arrow field metadata under the `snr.` prefix ([`meta`]), so
//! they survive Parquet and Arrow IPC round trips.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codes;
pub mod meta;

pub use meta::{Concept, FieldMeta, FieldType, Role, Unit};
