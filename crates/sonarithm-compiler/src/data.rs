// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The data layer as the compiler's data (ADR-0014: the compiler fixes `D`; A04 built
//! `DataBundle`).
//!
//! - [`DataBundle`] implements [`CompilerData`]: its fingerprint is the data layer's
//!   canonical bundle fingerprint, and the spec's `transform` list runs through
//!   `sonarithm_data::apply_all` on the primary table (03 §6), so every step lands in the
//!   bundle's transform log.
//! - [`Platform`] is the platform data engine: it registers the `inline` source (records in
//!   the spec, read by the data layer's JSON reader); given a base directory, the `file`
//!   source (paths relative to the spec, P8); and given datasets, the `dataset` source
//!   (data the host already holds, such as a data frame handed to the Python package).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};
use sonarithm_core::digest::Sha256Digest;
use sonarithm_core::engine::{
    CompileContext, CompileError, Engine, EngineManifest, Registrar, RegistryError, Source,
    SourceKind, SpecFragment,
};
use sonarithm_core::ids::EngineId;
use sonarithm_core::score::EngineVersion;
use sonarithm_data::{DataBundle, Format, ReadOptions};
use sonarithm_spec::Transform;

use crate::codes;
use crate::pipeline::CompilerData;

/// The platform data engine's ID.
pub const PLATFORM: EngineId = EngineId::from_static("platform");

impl CompilerData for DataBundle {
    /// The data layer's canonical fingerprint (03 §2). It fails only if Arrow cannot write
    /// a table it holds, which its construction rules out; the zero digest then marks the
    /// header as unfingerprinted.
    fn fingerprint(&self) -> Sha256Digest {
        sonarithm_data::bundle_fingerprint(self).unwrap_or(Sha256Digest::ZERO)
    }

    fn transform(
        &mut self,
        transforms: &[Transform],
        at: &SpecPointer,
    ) -> Option<Result<Diagnostics, Box<Diagnostic>>> {
        let table = self.primary()?.id();
        let converted: Result<Vec<_>, _> = transforms
            .iter()
            .enumerate()
            .map(|(index, transform)| {
                data_transform(transform).map_err(|message| {
                    Box::new(
                        Diagnostic::new(&codes::TRANSFORM_UNREADABLE, message).at(at.index(index)),
                    )
                })
            })
            .collect();
        let converted = match converted {
            Ok(converted) => converted,
            Err(diagnostic) => return Some(Err(diagnostic)),
        };
        Some(
            sonarithm_data::apply_all(self, table, &converted)
                .map_err(|err| Box::new(err.to_diagnostic().at(at.clone()))),
        )
    }
}

/// A spec transform as the data layer's transform. The two share their JSON shape
/// (RFC-0001 §4) except for `resample.every`, a duration object in the spec and seconds in
/// the data layer, and a `bin` without `as`, which writes `{field}_bin`.
fn data_transform(transform: &Transform) -> Result<sonarithm_data::Transform, String> {
    let mut value = serde_json::to_value(transform).map_err(|err| err.to_string())?;
    if let Some(object) = value.as_object_mut() {
        match object.get("op").and_then(Value::as_str) {
            Some("resample") => {
                if let Some(every) = object.get("every").cloned() {
                    let seconds = every.get("s").and_then(Value::as_f64).or_else(|| {
                        every
                            .get("ms")
                            .and_then(Value::as_f64)
                            .map(|ms| ms / 1000.0)
                    });
                    if let Some(seconds) = seconds {
                        object.insert("every".to_owned(), Value::from(seconds));
                    }
                }
            }
            Some("bin") if !object.contains_key("as") => {
                let field = object
                    .get("field")
                    .and_then(Value::as_str)
                    .unwrap_or("value")
                    .to_owned();
                object.insert("as".to_owned(), Value::from(format!("{field}_bin")));
            }
            _ => {}
        }
    }
    serde_json::from_value(value).map_err(|err| format!("This transform cannot run: {err}."))
}

/// The platform data engine: `inline` sources, `file` sources under a base directory, and
/// `dataset` sources the host provides by ID.
#[derive(Clone, Debug, Default)]
pub struct Platform {
    base: Option<PathBuf>,
    datasets: BTreeMap<String, DataBundle>,
}

impl Platform {
    /// Reads `inline` sources only.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Also reads `file` sources, relative to `base` (the spec's directory).
    #[must_use]
    pub fn with_base_dir(base: impl Into<PathBuf>) -> Self {
        Self {
            base: Some(base.into()),
            datasets: BTreeMap::new(),
        }
    }

    /// Also reads the `dataset` source `{"kind": "dataset", "id": id}` as `bundle`: data
    /// the host already holds. Adding an ID twice keeps the last bundle.
    #[must_use]
    pub fn with_dataset(mut self, id: impl Into<String>, bundle: DataBundle) -> Self {
        self.datasets.insert(id.into(), bundle);
        self
    }
}

impl Engine<DataBundle> for Platform {
    fn manifest(&self) -> EngineManifest {
        EngineManifest::new(
            PLATFORM,
            "SONARITHM data platform",
            EngineVersion(env!("CARGO_PKG_VERSION").to_owned()),
        )
    }

    fn register(&self, registrar: &mut Registrar<'_, DataBundle>) -> Result<(), RegistryError> {
        registrar.source(SourceKind::Inline, Arc::new(InlineSource))?;
        if let Some(base) = &self.base {
            registrar.source(
                SourceKind::File,
                Arc::new(FileSource { base: base.clone() }),
            )?;
        }
        if !self.datasets.is_empty() {
            registrar.source(
                SourceKind::Dataset,
                Arc::new(DatasetSource {
                    datasets: self.datasets.clone(),
                }),
            )?;
        }
        Ok(())
    }
}

/// Data the host provides by ID.
struct DatasetSource {
    datasets: BTreeMap<String, DataBundle>,
}

impl Source<DataBundle> for DatasetSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        _cx: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        let sonarithm_spec::Source::Dataset { id } = parsed else {
            return Err(not_this_kind("dataset"));
        };
        self.datasets.get(&id).cloned().ok_or_else(|| {
            let known: Vec<String> = self.datasets.keys().map(|id| format!("`{id}`")).collect();
            CompileError::from(
                Diagnostic::new(
                    &codes::UNKNOWN_DATASET,
                    format!(
                        "No dataset `{id}` was provided; the host provided {}.",
                        known.join(", ")
                    ),
                )
                .at(source.pointer().key("id")),
            )
        })
    }
}

/// Records written in the spec.
struct InlineSource;

impl Source<DataBundle> for InlineSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        cx: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        let sonarithm_spec::Source::Inline { values } = parsed else {
            return Err(not_this_kind("inline"));
        };
        let bytes = serde_json::to_vec(&values).map_err(|err| CompileError::Engine {
            engine: PLATFORM,
            message: err.to_string(),
        })?;
        load(&bytes, Format::Json, "inline.json", source.pointer(), cx)
    }
}

/// Files relative to a base directory.
struct FileSource {
    base: PathBuf,
}

impl Source<DataBundle> for FileSource {
    fn ingest(
        &self,
        source: &SpecFragment,
        cx: &mut CompileContext,
    ) -> Result<DataBundle, CompileError> {
        let parsed: sonarithm_spec::Source = source.parse()?;
        let sonarithm_spec::Source::File { path, format, .. } = parsed else {
            return Err(not_this_kind("file"));
        };
        let at = source.pointer().key("path");
        let full = self.base.join(Path::new(&path));
        let format = match format {
            Some(format) => serde_json::to_value(format)
                .ok()
                .and_then(|name| name.as_str().and_then(format_named)),
            None => full
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(Format::from_extension),
        };
        let Some(format) = format else {
            return Err(CompileError::from(
                Diagnostic::new(
                    &codes::DATA_UNREADABLE,
                    format!("The format of `{path}` is not known; set `format`."),
                )
                .at(at),
            ));
        };
        let bytes = std::fs::read(&full).map_err(|err| {
            CompileError::from(
                Diagnostic::new(
                    &codes::DATA_UNREADABLE,
                    format!("`{path}` cannot be read: {err}."),
                )
                .at(at.clone()),
            )
        })?;
        load(&bytes, format, &path, source.pointer(), cx)
    }
}

fn format_named(name: &str) -> Option<Format> {
    Format::from_extension(name)
}

fn load(
    bytes: &[u8],
    format: Format,
    locator: &str,
    at: &SpecPointer,
    cx: &mut CompileContext,
) -> Result<DataBundle, CompileError> {
    let loaded = sonarithm_data::read_bytes(bytes, format, locator, &ReadOptions::new())
        .map_err(|err| CompileError::from(err.to_diagnostic().at(at.clone())))?;
    for diagnostic in loaded.diagnostics {
        cx.report(diagnostic.at(at.clone()));
    }
    Ok(loaded.bundle)
}

fn not_this_kind(kind: &str) -> CompileError {
    CompileError::Engine {
        engine: PLATFORM,
        message: format!("the {kind} source was handed another source kind"),
    }
}
