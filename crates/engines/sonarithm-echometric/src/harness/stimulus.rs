// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A generated stimulus: its table, its ground truth and its provenance.
//!
//! The table is numeric and small. [`Stimulus::csv`] writes it in one canonical form (a
//! header row, LF line endings, Rust's shortest round-trip decimal for each number), and
//! [`Stimulus::digest`] hashes those bytes, so a stimulus can be committed, compared and
//! cited by its digest. [`Stimulus::to_bundle`] loads it into the data layer with declared
//! field semantics, ready for the compiler.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sonarithm_core::digest::Sha256Digest;
use sonarithm_data::{
    Concept, DataBundle, DataError, FieldMeta, FieldType, Format, ReadOptions, Role, Unit,
    read_bytes,
};

use super::generators::Params;
use super::generators::correlation::CorrelationTruth;
use super::generators::prices::PriceTruth;
use super::generators::reversals::ReversalTruth;
use super::generators::transits::TransitTruth;

/// A registered stimulus generator (docs/research-paper/datasets.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GeneratorId {
    /// GEN-001: trend reversals × density (Nees & Walker 2008).
    #[serde(rename = "GEN-001")]
    TrendReversals,
    /// GEN-002: price series with context cues (Smith & Walker 2005).
    #[serde(rename = "GEN-002")]
    PriceSeries,
    /// GEN-003: SNR-controlled transit series (Tucker Brown et al. 2022).
    #[serde(rename = "GEN-003")]
    Transits,
    /// GEN-004: correlation triplets (Elmquist et al. 2024).
    #[serde(rename = "GEN-004")]
    CorrelationTriplets,
}

impl GeneratorId {
    /// The registry ID, such as `GEN-001`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrendReversals => "GEN-001",
            Self::PriceSeries => "GEN-002",
            Self::Transits => "GEN-003",
            Self::CorrelationTriplets => "GEN-004",
        }
    }

    /// The version of the generator's algorithm. It changes whenever the same parameters
    /// and seed would give different bytes, and is recorded in every manifest, so an old
    /// stimulus can be traced to the code that made it (datasets.md "Provenance").
    #[must_use]
    pub const fn version(self) -> u32 {
        match self {
            Self::TrendReversals
            | Self::PriceSeries
            | Self::Transits
            | Self::CorrelationTriplets => 1,
        }
    }

    /// The `references.bib` key of the published design the generator follows.
    #[must_use]
    pub const fn basis(self) -> &'static str {
        match self {
            Self::TrendReversals => "nees2008density",
            Self::PriceSeries => "smith2005effects",
            Self::Transits => "tuckerbrown2022evaluating",
            Self::CorrelationTriplets => "elmquist2024parallel",
        }
    }
}

/// What a column holds, which sets its field semantics in [`Stimulus::to_bundle`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColumnKind {
    /// The x axis: time or position, quantitative, role `x`.
    Time,
    /// A measured value, quantitative.
    Value,
    /// A series label stored as a small integer, nominal, role `series`.
    Series,
}

/// One column of a generated table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Column {
    /// The field name.
    pub name: String,
    /// What the column holds.
    pub kind: ColumnKind,
    /// A human-readable label for legends and announcements.
    pub label: String,
    /// The unit, if any (03 §3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The data concept, if any (03 §3.3), which sets the default polarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concept: Option<String>,
    /// The values, one per row.
    pub values: Vec<f64>,
}

impl Column {
    pub(crate) fn new(name: &str, kind: ColumnKind, label: &str, values: Vec<f64>) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            label: label.to_owned(),
            unit: None,
            concept: None,
            values,
        }
    }

    pub(crate) fn with_unit(mut self, unit: &str) -> Self {
        self.unit = Some(unit.to_owned());
        self
    }

    pub(crate) fn with_concept(mut self, concept: &str) -> Self {
        self.concept = Some(concept.to_owned());
        self
    }
}

/// A point-estimation query: the listener is asked for the value at `x`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Query {
    /// The row asked about.
    pub row: usize,
    /// The row's x value (time or position).
    pub x: f64,
    /// The true value, which scores the answer (RMS error, evaluation-plan metrics).
    pub value: f64,
}

/// The ground truth of a stimulus: what a correct response is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Truth {
    /// GEN-001.
    Reversals(ReversalTruth),
    /// GEN-002.
    Prices(PriceTruth),
    /// GEN-003.
    Transits(TransitTruth),
    /// GEN-004.
    Correlations(CorrelationTruth),
}

/// Why a stimulus needs a second look: it leaves the published design or the evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Note {
    /// A parameter is outside what the cited study tested; the stimulus is experimental
    /// and names the rule or hypothesis it serves (design-rules.md: `INFER` stays flagged).
    Experimental {
        /// The rule or hypothesis, such as `SDR-20` or `H-02`.
        rule: String,
        /// What is outside the studied range.
        detail: String,
    },
    /// The stimulus departs from the published design (evaluation-plan "Replication
    /// gallery": every deviation is documented next to the source).
    Deviation {
        /// What differs and why.
        detail: String,
    },
}

/// A generated stimulus.
#[derive(Clone, Debug, PartialEq)]
pub struct Stimulus {
    /// The generator that made it.
    pub generator: GeneratorId,
    /// The seed. With the parameters and the generator version it fixes every value.
    pub seed: u64,
    /// The parameters, with every default filled in.
    pub params: Params,
    /// The table, in column order.
    pub columns: Vec<Column>,
    /// The ground truth.
    pub truth: Truth,
    /// Departures from the published design or the evidence.
    pub notes: Vec<Note>,
}

/// The provenance record of a stimulus: everything needed to regenerate and check it
/// (datasets.md "Provenance"; the study log's `stimulus_id`, `seed` and `manifest_ref`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StimulusManifest {
    /// The generator ID.
    pub generator: GeneratorId,
    /// The generator's algorithm version.
    pub generator_version: u32,
    /// The published design, as a `references.bib` key.
    pub basis: String,
    /// The seed.
    pub seed: u64,
    /// The parameters, with defaults filled in.
    pub params: Params,
    /// The number of rows.
    pub rows: usize,
    /// The column names, in order.
    pub columns: Vec<String>,
    /// SHA-256 of [`Stimulus::csv`].
    pub table_sha256: Sha256Digest,
    /// The ground truth.
    pub truth: Truth,
    /// Departures from the published design or the evidence.
    pub notes: Vec<Note>,
}

impl Stimulus {
    /// The number of rows.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.columns.first().map_or(0, |column| column.values.len())
    }

    /// The column named `name`.
    #[must_use]
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == name)
    }

    /// The table as canonical CSV: a header row of column names, then one line per row,
    /// LF line endings, each number in Rust's shortest round-trip decimal form (never an
    /// exponent, and `-0` written as `0`).
    #[must_use]
    pub fn csv(&self) -> String {
        let mut out = String::new();
        let names: Vec<&str> = self
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect();
        out.push_str(&names.join(","));
        out.push('\n');
        for row in 0..self.rows() {
            for (index, column) in self.columns.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                let value = column.values.get(row).copied().unwrap_or(f64::NAN);
                // Adding zero turns -0 into +0, so both print as "0".
                let _ = write!(out, "{}", value + 0.0);
            }
            out.push('\n');
        }
        out
    }

    /// SHA-256 of [`Stimulus::csv`].
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        Sha256Digest::of(self.csv().as_bytes())
    }

    /// The provenance record.
    #[must_use]
    pub fn manifest(&self) -> StimulusManifest {
        StimulusManifest {
            generator: self.generator,
            generator_version: self.generator.version(),
            basis: self.generator.basis().to_owned(),
            seed: self.seed,
            params: self.params.clone(),
            rows: self.rows(),
            columns: self
                .columns
                .iter()
                .map(|column| column.name.clone())
                .collect(),
            table_sha256: self.digest(),
            truth: self.truth.clone(),
            notes: self.notes.clone(),
        }
    }

    /// Loads the table into a [`DataBundle`] named `name`, with each column's type, role,
    /// label, unit and concept declared (03 §3), so inference fills nothing in.
    ///
    /// # Errors
    ///
    /// A [`DataError`] only if the data layer rejects the generated records.
    pub fn to_bundle(&self, name: &str) -> Result<DataBundle, DataError> {
        let rows: Vec<Value> = (0..self.rows())
            .map(|row| {
                let mut record = Map::new();
                for column in &self.columns {
                    let value = column.values.get(row).copied().unwrap_or(f64::NAN);
                    record.insert(column.name.clone(), Value::from(value + 0.0));
                }
                Value::Object(record)
            })
            .collect();
        let bytes = serde_json::to_vec(&Value::Array(rows)).unwrap_or_default();
        let fields: BTreeMap<String, FieldMeta> = self
            .columns
            .iter()
            .map(|column| (column.name.clone(), field_meta(column)))
            .collect();
        let options = ReadOptions::new().with_fields(fields);
        read_bytes(&bytes, Format::Json, &format!("{name}.json"), &options)
            .map(|loaded| loaded.bundle)
    }
}

fn field_meta(column: &Column) -> FieldMeta {
    let meta = match column.kind {
        ColumnKind::Time => FieldMeta::new()
            .with_type(FieldType::Quantitative)
            .with_role(Role::X),
        ColumnKind::Value => FieldMeta::new().with_type(FieldType::Quantitative),
        ColumnKind::Series => FieldMeta::new()
            .with_type(FieldType::Nominal)
            .with_role(Role::Series),
    }
    .with_label(column.label.as_str());
    let meta = match column.unit.as_deref().and_then(Unit::new) {
        Some(unit) => meta.with_unit(unit),
        None => meta,
    };
    match column.concept.as_deref().and_then(Concept::new) {
        Some(concept) => meta.with_concept(concept),
        None => meta,
    }
}
