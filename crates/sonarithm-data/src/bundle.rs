// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! [`DataBundle`]: the unit the compiler consumes (03 §2).

use std::fmt;

use serde::{Deserialize, Serialize};
use sonarithm_core::{AnalyzerId, Confidence, RowRange, RuleId, TableId};

use crate::error::DataError;
use crate::table::Table;
use crate::transform::TransformRecord;

/// Identifies a data bundle within a project or session.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct BundleId(pub u64);

/// A file format that `sonarithm-data` reads. Names match the spec's `data.source.format`
/// (RFC-0001 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Format {
    /// Comma-separated values (the delimiter is sniffed).
    Csv,
    /// Tab-separated values.
    Tsv,
    /// A JSON document holding records or columns.
    Json,
    /// Newline-delimited JSON records.
    Ndjson,
    /// Apache Parquet.
    Parquet,
    /// Arrow IPC, file (Feather v2) or stream format.
    Arrow,
    /// Office Open XML workbook (also XLSM, XLSB and legacy XLS).
    Xlsx,
    /// An `OpenDocument` spreadsheet.
    Ods,
}

impl Format {
    /// The format name used in specs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Json => "json",
            Self::Ndjson => "ndjson",
            Self::Parquet => "parquet",
            Self::Arrow => "arrow",
            Self::Xlsx => "xlsx",
            Self::Ods => "ods",
        }
    }

    /// The format a file extension implies, case-insensitively, or `None`.
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_ascii_lowercase().as_str() {
            "csv" | "txt" => Self::Csv,
            "tsv" | "tab" => Self::Tsv,
            "json" => Self::Json,
            "ndjson" | "jsonl" => Self::Ndjson,
            "parquet" | "pq" => Self::Parquet,
            "arrow" | "feather" | "ipc" | "arrows" => Self::Arrow,
            "xlsx" | "xlsm" | "xlsb" | "xls" => Self::Xlsx,
            "ods" => Self::Ods,
            _ => return None,
        })
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where a bundle's data came from (03 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SourceKind {
    /// A file in one of the supported formats.
    File(Format),
    /// Values written in the spec (`data.source.kind = "inline"`).
    Inline,
    /// Arrow data handed over in memory, for example from Python.
    Memory,
    /// Data extracted from a chart file by PLOTSONA; the descriptor's extractor names the
    /// tier, such as `sonarithm-plotsona/office` (10 §3).
    Chart,
    /// Samples of a function computed by CURVASONA (`data.source.kind = "function"`).
    Function,
}

/// Unix time in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixMillis(pub i64);

/// How a bundle was obtained (03 §2). Recorded in render manifests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SourceDescriptor {
    /// The kind of source.
    pub kind: SourceKind,
    /// The file name without its directories, or another label that identifies the source
    /// without exposing user paths (P8, ADR-0012).
    pub locator: String,
    /// The component that produced the data, such as `sonarithm-data/csv`.
    pub extractor: String,
    /// The version of that component.
    pub extractor_version: String,
    /// When the data was loaded. Set by the application, never by the data layer (no wall
    /// clock in compile paths, coding-standards §10), and excluded from fingerprints.
    pub ingested_at: Option<UnixMillis>,
}

impl SourceDescriptor {
    /// A descriptor for data produced by this crate's `extractor`; the locator is redacted
    /// with [`redact_locator`].
    #[must_use]
    pub fn new(kind: SourceKind, locator: &str, extractor: &str) -> Self {
        Self {
            kind,
            locator: redact_locator(locator),
            extractor: format!("sonarithm-data/{extractor}"),
            extractor_version: env!("CARGO_PKG_VERSION").to_owned(),
            ingested_at: None,
        }
    }
}

/// The last component of a path, so that descriptors, manifests and diagnostics never
/// carry a user's directories (P8). Both `/` and `\` separate components.
#[must_use]
pub fn redact_locator(locator: &str) -> String {
    locator
        .rsplit(['/', '\\'])
        .find(|part| !part.is_empty())
        .unwrap_or_default()
        .to_owned()
}

/// What an analyzer found (03 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AnnotationKind {
    /// A local or global minimum or maximum.
    Extremum,
    /// A value far from its neighbours.
    Outlier,
    /// A change in level, trend or variance.
    ChangePoint,
    /// A repeating pattern.
    Periodicity,
    /// A critical point of a function (CURVASONA).
    CriticalPoint,
    /// A described insight.
    Insight,
}

/// Analyzer output, kept apart from the tables so that re-analysis never mutates data
/// (03 §2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Annotation {
    /// What was found.
    pub kind: AnnotationKind,
    /// The rows it covers.
    pub range: RowRange,
    /// The value, where one applies.
    pub value: Option<f64>,
    /// How sure the analyzer is.
    pub confidence: Confidence,
    /// The analyzer that produced it.
    pub source: AnalyzerId,
    /// The design rules it serves.
    pub rules: Vec<RuleId>,
}

impl Annotation {
    /// An annotation without a value or rules.
    #[must_use]
    pub fn new(
        kind: AnnotationKind,
        range: RowRange,
        confidence: Confidence,
        source: AnalyzerId,
    ) -> Self {
        Self {
            kind,
            range,
            value: None,
            confidence,
            source,
            rules: Vec::new(),
        }
    }
}

/// The unit the compiler consumes: one or more tables, where they came from, and analyzer
/// annotations (03 §2). It usually holds one table.
#[derive(Clone, Debug, PartialEq)]
pub struct DataBundle {
    id: BundleId,
    tables: Vec<Table>,
    source: SourceDescriptor,
    annotations: Vec<Annotation>,
    transforms: Vec<TransformRecord>,
}

impl DataBundle {
    /// A bundle of tables with distinct IDs.
    ///
    /// # Errors
    ///
    /// [`DataError::DuplicateTable`] if two tables share an ID.
    pub fn new(
        id: BundleId,
        source: SourceDescriptor,
        tables: Vec<Table>,
    ) -> Result<Self, DataError> {
        let mut ids: Vec<TableId> = tables.iter().map(Table::id).collect();
        ids.sort_unstable();
        if let Some(pair) = ids.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(DataError::DuplicateTable { id: pair[0].0 });
        }
        Ok(Self {
            id,
            tables,
            source,
            annotations: Vec::new(),
            transforms: Vec::new(),
        })
    }

    /// A bundle holding one table.
    #[must_use]
    pub fn single(id: BundleId, source: SourceDescriptor, table: Table) -> Self {
        Self {
            id,
            tables: vec![table],
            source,
            annotations: Vec::new(),
            transforms: Vec::new(),
        }
    }

    /// The bundle ID.
    #[must_use]
    pub const fn id(&self) -> BundleId {
        self.id
    }

    /// The tables, in the order the source produced them.
    #[must_use]
    pub fn tables(&self) -> &[Table] {
        &self.tables
    }

    /// The first table, which single-table sources produce.
    #[must_use]
    pub fn primary(&self) -> Option<&Table> {
        self.tables.first()
    }

    /// The table with an ID.
    #[must_use]
    pub fn table(&self, id: TableId) -> Option<&Table> {
        self.tables.iter().find(|table| table.id() == id)
    }

    /// Replaces the table that has the same ID as `table` (for example its next
    /// generation) and returns the old one, or adds `table` and returns `None`.
    pub fn put_table(&mut self, table: Table) -> Option<Table> {
        if let Some(slot) = self.tables.iter_mut().find(|slot| slot.id() == table.id()) {
            return Some(std::mem::replace(slot, table));
        }
        self.tables.push(table);
        None
    }

    /// Where the data came from.
    #[must_use]
    pub const fn source(&self) -> &SourceDescriptor {
        &self.source
    }

    /// The source descriptor, for example to set `ingested_at`.
    pub fn source_mut(&mut self) -> &mut SourceDescriptor {
        &mut self.source
    }

    /// Analyzer annotations, in the order they were added.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Adds an analyzer annotation.
    pub fn annotate(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// The transforms applied to the bundle's tables, in order (03 §6); render manifests
    /// include it.
    #[must_use]
    pub fn transform_log(&self) -> &[TransformRecord] {
        &self.transforms
    }

    pub(crate) fn log_transform(&mut self, record: TransformRecord) {
        self.transforms.push(record);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch};
    use arrow_schema::{DataType, Field, Schema};
    use sonarithm_core::Generation;

    use super::*;

    fn table(id: u32) -> Table {
        let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int32, false)]));
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(Int32Array::from(vec![1, 2]))]).unwrap();
        Table::from_batch(TableId(id), "t", batch)
    }

    fn source() -> SourceDescriptor {
        SourceDescriptor::new(
            SourceKind::File(Format::Csv),
            "C:\\Users\\me\\data\\temps.csv",
            "csv",
        )
    }

    #[test]
    fn locators_never_keep_directories() {
        assert_eq!(redact_locator("/home/me/data/temps.csv"), "temps.csv");
        assert_eq!(redact_locator("C:\\Users\\me\\temps.csv"), "temps.csv");
        assert_eq!(redact_locator("dir/"), "dir");
        assert_eq!(redact_locator("temps.csv"), "temps.csv");
        assert_eq!(redact_locator(""), "");
        assert_eq!(source().locator, "temps.csv");
        assert_eq!(source().extractor, "sonarithm-data/csv");
    }

    #[test]
    fn table_ids_are_unique_within_a_bundle() {
        let err = DataBundle::new(BundleId(1), source(), vec![table(0), table(0)]).unwrap_err();
        assert!(matches!(err, DataError::DuplicateTable { id: 0 }));
        let bundle = DataBundle::new(BundleId(1), source(), vec![table(0), table(1)]).unwrap();
        assert_eq!(bundle.tables().len(), 2);
        assert_eq!(bundle.table(TableId(1)).unwrap().id(), TableId(1));
    }

    #[test]
    fn putting_a_table_replaces_its_previous_generation() {
        let mut bundle = DataBundle::single(BundleId(1), source(), table(0));
        let first = bundle.primary().unwrap().clone();
        let next = first
            .next_generation(Arc::clone(first.schema()), first.batches().to_vec())
            .unwrap();
        assert_eq!(bundle.put_table(next), Some(first));
        assert_eq!(bundle.primary().unwrap().generation(), Generation(1));
        assert_eq!(bundle.put_table(table(7)), None);
        assert_eq!(bundle.tables().len(), 2);
    }

    #[test]
    fn formats_follow_the_spec_names_and_extensions() {
        assert_eq!(Format::from_extension("FEATHER"), Some(Format::Arrow));
        assert_eq!(Format::from_extension("jsonl"), Some(Format::Ndjson));
        assert_eq!(Format::from_extension("png"), None);
        assert_eq!(
            serde_json::to_string(&Format::Ndjson).unwrap(),
            r#""ndjson""#
        );
        assert_eq!(
            serde_json::to_string(&SourceKind::File(Format::Csv)).unwrap(),
            r#"{"file":"csv"}"#
        );
    }

    #[test]
    fn annotations_are_kept_in_order() {
        let mut bundle = DataBundle::single(BundleId(1), source(), table(0));
        let range = bundle.primary().unwrap().rows();
        for kind in [AnnotationKind::Extremum, AnnotationKind::Outlier] {
            bundle.annotate(Annotation::new(
                kind,
                range,
                Confidence::CERTAIN,
                AnalyzerId::from_static("extrema"),
            ));
        }
        let kinds: Vec<_> = bundle.annotations().iter().map(|a| a.kind).collect();
        assert_eq!(kinds, [AnnotationKind::Extremum, AnnotationKind::Outlier]);
    }
}
