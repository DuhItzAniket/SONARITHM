// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The quick-start builder: one dataset, an x and a y field, and every other choice from
//! the evidence-backed defaults.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sonarithm_data::DataBundle;
use sonarithm_numisona::templates::{Bindings, TemplateError, template};
use sonarithm_spec::Mark;

use crate::api::{Datasets, Sonarithm};
use crate::compiled::Compiled;
use crate::error::Error;
use crate::spec::Spec;

/// The template behind [`Quick`] (SDR-03).
const TEMPLATE: &str = "line-overview";

/// The dataset ID a [`Quick::data`] bundle gets in the generated spec.
pub const QUICK_DATASET: &str = "main";

/// Builds and compiles a default sonification of one dataset ([`Sonarithm::quick`]).
///
/// It applies NUMISONA's `line-overview` template: x along time at 3 points per second
/// (SDR-19), y as pitch in the 400–1400 Hz log-frequency default (SDR-08, SDR-09), and
/// polarity from the y field's concept (SDR-12..14). Without `x` or `y`, the template
/// binds the dataset's fields by role, then by kind.
///
/// ```
/// use sonarithm::{Format, ReadOptions, Sonarithm};
///
/// let temps = sonarithm::read_bytes(
///     b"year,anomaly\n2000,0.39\n2001,0.54\n2002,0.63\n",
///     Format::Csv,
///     "temps.csv",
///     &ReadOptions::new(),
/// )?
/// .bundle;
/// let snr = Sonarithm::new();
/// let compiled = snr.quick().data(temps).x("year").y("anomaly").compile()?;
/// assert_eq!(compiled.score().tracks[0].events.len(), 3);
/// # Ok::<(), sonarithm::Error>(())
/// ```
#[derive(Clone, Debug)]
#[must_use]
pub struct Quick {
    snr: Sonarithm,
    data: Option<QuickData>,
    x: Option<String>,
    y: Option<String>,
    mark: Option<Mark>,
    range_hz: Option<[f64; 2]>,
    concept: Option<String>,
    name: Option<String>,
}

#[derive(Clone, Debug)]
enum QuickData {
    /// A file, read once to bind the fields; the spec names it relative to its directory.
    #[cfg_attr(
        not(feature = "fs"),
        expect(dead_code, reason = "only `data_file` builds it, and it needs `fs`")
    )]
    File { path: PathBuf, bundle: DataBundle },
    /// Data in memory, passed as the `main` dataset.
    Bundle(DataBundle),
}

impl QuickData {
    fn bundle(&self) -> &DataBundle {
        match self {
            Self::File { bundle, .. } | Self::Bundle(bundle) => bundle,
        }
    }
}

impl Quick {
    pub(crate) fn new(snr: Sonarithm) -> Self {
        Self {
            snr,
            data: None,
            x: None,
            y: None,
            mark: None,
            range_hz: None,
            concept: None,
            name: None,
        }
    }

    /// Uses a data file (CSV, TSV, JSON, NDJSON, Parquet, Arrow IPC, XLSX or ODS, from its
    /// extension). The generated spec reads it as a `file` source relative to its directory,
    /// so the spec stays portable (P8).
    ///
    /// # Errors
    ///
    /// [`Error::Data`] when the file cannot be read.
    #[cfg(feature = "fs")]
    pub fn data_file(mut self, path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let loaded = sonarithm_data::read_file(path, None, &sonarithm_data::ReadOptions::new())?;
        self.data = Some(QuickData::File {
            path: path.to_path_buf(),
            bundle: loaded.bundle,
        });
        Ok(self)
    }

    /// Uses data already in memory, such as a table from Python. The generated spec reads it
    /// as the `dataset` source [`QUICK_DATASET`].
    pub fn data(mut self, bundle: DataBundle) -> Self {
        self.data = Some(QuickData::Bundle(bundle));
        self
    }

    /// The field along time.
    pub fn x(mut self, field: impl Into<String>) -> Self {
        self.x = Some(field.into());
        self
    }

    /// The field mapped to pitch.
    pub fn y(mut self, field: impl Into<String>) -> Self {
        self.y = Some(field.into());
        self
    }

    /// The mark: `tone` (the template's), `glide`, or the experimental `auto` (SDR-20).
    pub fn mark(mut self, mark: Mark) -> Self {
        self.mark = Some(mark);
        self
    }

    /// The pitch range in hertz, low to high, instead of the 400–1400 Hz default (SDR-08).
    pub fn range_hz(mut self, low: f64, high: f64) -> Self {
        self.range_hz = Some([low, high]);
        self
    }

    /// The y field's concept, such as `temperature`, which sets its polarity (SDR-12..14).
    pub fn concept(mut self, concept: impl Into<String>) -> Self {
        self.concept = Some(concept.into());
        self
    }

    /// The spec's name; by default the data file's stem, or `quick`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The generated spec, to save, edit or compile later.
    ///
    /// # Errors
    ///
    /// [`Error::NoData`] without data, and [`Error::Template`] when a field is missing or
    /// not numeric, or no numeric fields are left to bind.
    pub fn spec(&self) -> Result<Spec, Error> {
        let data = self.data.as_ref().ok_or(Error::NoData)?;
        let bundle = data.bundle();
        let table = bundle.primary().ok_or(TemplateError::NoTable)?;
        let template = template(TEMPLATE)?;
        let mut bindings = if self.x.is_none() || self.y.is_none() {
            template.bind(table)?
        } else {
            Bindings::new()
        };
        for (slot, field) in [("x", &self.x), ("y", &self.y)] {
            if let Some(field) = field {
                bindings.insert(slot.to_owned(), field.clone());
            }
        }
        if bindings.contains_key("x") && bindings.get("x") == bindings.get("y") {
            return Err(Error::Template(TemplateError::BadBinding {
                template: template.id,
                slot: "y".to_owned(),
                field: bindings.get("y").cloned().unwrap_or_default(),
                reason: "x and y name the same field; choose another with `y`",
            }));
        }
        let (source, base, stem) = match data {
            QuickData::File { path, .. } => {
                let file = crate::error::file_name(path);
                let stem = Path::new(&file)
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned());
                let base = path.parent().map(Path::to_path_buf);
                (json!({ "kind": "file", "path": file }), base, stem)
            }
            QuickData::Bundle(_) => (
                json!({ "kind": "dataset", "id": QUICK_DATASET }),
                None,
                None,
            ),
        };
        let mut value = template.apply(bundle, &bindings, &source)?;
        let name = self
            .name
            .clone()
            .or(stem)
            .unwrap_or_else(|| "quick".to_owned());
        value["name"] = json!(name);
        // The track is named after what it plays, so descriptions read "In anomaly, …".
        if let Some(y) = bindings.get("y") {
            value["tracks"][0]["id"] = json!(y);
        }
        if let Some(mark) = self.mark {
            value["tracks"][0]["mark"] = serde_json::to_value(mark).unwrap_or(Value::Null);
        }
        if let Some(range) = self.range_hz {
            value["tracks"][0]["channels"]["pitch"]["range"] = json!({ "hz": range });
        }
        if let (Some(concept), Some(y)) = (&self.concept, bindings.get("y")) {
            value["data"]["fields"] = json!({ y.as_str(): { "concept": concept } });
        }
        let spec = Spec::from_value(&value).with_name(format!("{name}.spec.json"));
        Ok(match base {
            Some(base) => spec.with_base_dir(base),
            None => spec,
        })
    }

    /// Compiles the generated spec.
    ///
    /// # Errors
    ///
    /// As [`Quick::spec`] and [`Sonarithm::compile`].
    pub fn compile(&self) -> Result<Compiled, Error> {
        let spec = self.spec()?;
        match &self.data {
            Some(QuickData::Bundle(bundle)) => self
                .snr
                .compile_with(&spec, &Datasets::new().with(QUICK_DATASET, bundle.clone())),
            _ => self.snr.compile(&spec),
        }
    }
}
