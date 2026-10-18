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

pub mod bundle;
pub mod codes;
pub mod error;
mod infer;
pub mod meta;
pub mod provenance;
pub mod source;
pub mod table;
pub mod transform;

pub use bundle::{
    Annotation, AnnotationKind, BundleId, DataBundle, Format, SourceDescriptor, SourceKind,
};
pub use error::{DataError, Limit};
pub use meta::{Concept, FieldMeta, FieldType, Role, Unit};
pub use provenance::{ProvenanceMix, RowProvenance};
#[cfg(feature = "fs")]
pub use source::read_file;
pub use source::{CsvOptions, JsonOptions, Limits, Loaded, ReadOptions, read_bytes};
#[cfg(feature = "xlsx")]
pub use source::{SheetOptions, SheetRef};
pub use table::Table;
pub use transform::{SortOrder, Transform, TransformRecord, apply, apply_all};
