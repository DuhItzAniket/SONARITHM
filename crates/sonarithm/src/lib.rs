// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM turns data into structured, navigable sound. This facade is its stable public
//! API: compile a declarative spec plus data into a Score (the canonical sound IR), inspect
//! and describe it, and (from A05's export crate) render it to audio files.
//!
//! ```
//! use sonarithm::{Sonarithm, Spec};
//!
//! let snr = Sonarithm::builder().with_default_engines().build()?;
//! let spec = Spec::from_json_str(r#"{
//!     "specVersion": "0.1",
//!     "name": "Temperatures",
//!     "data": { "source": { "kind": "inline", "values": [
//!         { "day": 1, "temp": 8.5 }, { "day": 2, "temp": 9.25 }, { "day": 3, "temp": 7.0 } ] } },
//!     "substrate": { "time": { "field": "day" } },
//!     "tracks": [{ "id": "temps", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
//! }"#)?;
//! let compiled = snr.compile(&spec)?;
//! for d in compiled.diagnostics().iter() {
//!     println!("{d}"); // applied defaults, each with its design-rule ID
//! }
//! println!("{}", compiled.describe()); // L1/L2 text, deterministic
//! # Ok::<(), sonarithm::Error>(())
//! ```
//!
//! A quick start covers the common case, one x and one y field:
//!
//! ```no_run
//! # use sonarithm::Sonarithm;
//! let compiled = Sonarithm::new()
//!     .quick()
//!     .data_file("temps.csv")?
//!     .x("year")
//!     .y("anomaly")
//!     .compile()?;
//! # Ok::<(), sonarithm::Error>(())
//! ```
//!
//! # Stability
//!
//! The types and functions of this crate follow the project's versioning policy
//! (docs/engineering/release-process.md): before 1.0, a minor release may break them, and
//! every break is in the changelog. Types re-exported from the implementation crates
//! (`sonarithm-core`, `-spec`, `-data`) are part of this API where they appear in it; the
//! implementation crates themselves carry no separate promise
//! (docs/architecture/11-bindings-cli-plugins.md §1).
//!
//! # Features
//!
//! | Feature | Default | Effect |
//! |---|---|---|
//! | `fs` | on | Read specs and data files from paths |
//! | `xlsx` | on | XLSX and ODS sources |
//! | `zstd` | on | Zstandard-compressed Parquet and Arrow IPC (builds C code) |
//! | `full` | off | Everything non-copyleft |
//!
//! Without default features the crate builds for `wasm32-unknown-unknown`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod api;
mod chart;
mod compiled;
mod error;
mod quick;
mod render;
mod spec;

pub use api::{Builder, Datasets, Sonarithm};
#[cfg(feature = "fs")]
pub use chart::read_chart;
pub use chart::{Chart, read_chart_bytes, summary_diagnostic, summary_sentence};
pub use compiled::{Compiled, Description, DiagnosticCounts, Summary, TrackSummary};
pub use error::Error;
pub use quick::{QUICK_DATASET, Quick};
pub use render::{
    AudioFormat, Normalization, RenderManifest, RenderOptions, Rendered, WavEncoding,
};
pub use sonarithm_plotsona::{
    ChartData, ChartType, ClassCounts, CorrectionRecord, CorrectionSet, ExtractOptions,
    ExtractionReport, SeriesReport,
};
pub use spec::{Migration, Spec};

pub use sonarithm_core::engine::{CancelToken, EngineManifest};
pub use sonarithm_core::score::{Mode, SpecVersion, TrackRole};
pub use sonarithm_core::{Diagnostic, Diagnostics, Score, Severity};
#[cfg(feature = "fs")]
pub use sonarithm_data::read_file;
pub use sonarithm_data::{
    Capacity, DataBundle, DataError, Format, ReadOptions, Snapshot, StreamTable, Table,
    read_batches, read_bytes,
};
pub use sonarithm_numisona::polarity::PolarityProfile;
pub use sonarithm_spec::{Mark, SPEC_VERSION};

/// Diagnostics as `miette` reports, with spans in the spec text and help from the code
/// registry, for command-line and editor front ends (12 §1). miette's narratable handler
/// renders them as plain text for screen readers.
pub use sonarithm_spec::report::{SpecReport, reports};

/// The Score IR types (05 §4): tracks, events, markers and the header.
pub use sonarithm_core::score;
/// Unit newtypes used in the Score: `Seconds`, `Cents`, `Hz`, `Db`, `Pan`.
pub use sonarithm_core::units;

/// The SONARITHM version of this build, such as `0.1.0-alpha`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The JSON Schemas SONARITHM publishes (`sonarithm schema --kind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaKind {
    /// The spec schema of the current version (`schemas/sonarithm-spec-0.4.schema.json`).
    Spec,
}

impl SchemaKind {
    /// Every kind, in order.
    pub const ALL: [Self; 1] = [Self::Spec];

    /// The kind's name on the command line, such as `spec`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Spec => "spec",
        }
    }
}

/// The JSON Schema of `kind`, as published in `schemas/`.
///
/// ```
/// let schema = sonarithm::schema(sonarithm::SchemaKind::Spec);
/// assert!(schema.contains("specVersion"));
/// ```
#[must_use]
pub fn schema(kind: SchemaKind) -> String {
    match kind {
        SchemaKind::Spec => sonarithm_spec::json_schema(),
    }
}
