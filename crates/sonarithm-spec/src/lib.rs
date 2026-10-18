// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM spec types: JSON (de)serialization, JSON Schema export, semantic validation,
//! version migration and importers (Erie, Vega-Lite).
//!
//! Spec v0.1 is defined by RFC-0001 and docs/architecture/04-sonification-spec.md.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codes;
pub mod data;
pub mod json;
pub mod schema;
pub mod sections;
pub mod spec;
pub mod track;
pub mod values;

pub use data::{DataSpec, FieldSpec, Source, Transform};
pub use json::JsonValue;
pub use schema::json_schema;
pub use sections::{Analysis, Context, Music, Output, Pace, Playback, Substrate};
pub use spec::{Mode, SPEC_VERSION, Spec};
pub use track::{Channel, Channels, Mark, TrackSpec};
