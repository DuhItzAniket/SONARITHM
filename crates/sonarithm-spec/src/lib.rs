// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM spec types: JSON (de)serialization, JSON Schema export, semantic validation,
//! version migration and importers (Erie, Vega-Lite).
//!
//! Spec v0.1 is defined by RFC-0001, v0.2 by RFC-0004, and both by
//! docs/architecture/04-sonification-spec.md. This crate writes v0.2 and reads v0.1
//! documents as they are.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod canonical;
pub mod codes;
pub mod correction_file;
pub mod data;
pub mod defaults;
pub mod experimental;
pub mod json;
pub mod migrate;
pub mod parse;
pub mod prepare;
pub mod report;
pub mod schema;
pub mod sections;
pub mod spec;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;
pub mod track;
pub mod v02;
pub mod validate;
pub mod values;

pub use data::{DataSpec, FieldSpec, Source, Transform};
pub use json::JsonValue;
pub use parse::parse;
pub use prepare::{Prepared, prepare};
pub use report::{SpecReport, reports};
pub use schema::json_schema;
pub use sections::{Analysis, Context, Music, Output, Pace, Playback, Substrate};
pub use spec::{Mode, READABLE_VERSIONS, SPEC_VERSION, Spec};
pub use track::{Channel, Channels, Mark, TrackSpec};
pub use validate::{FieldCatalog, validate};
