// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reserved provenance columns and their propagation (03 §4, AGENTS.md rule 7).
//!
//! A table may carry `__snr_prov` (the [`ProvenanceClass`] of each row) and `__snr_conf`
//! (its [`Confidence`], null when not applicable). Absent columns mean `exact`. Sources
//! accept the class as text, as a text dictionary or as a `UInt8` code, and the confidence
//! as `Float32` or `Float64`; [`normalize`] rewrites both into one canonical form, so
//! fingerprints do not depend on how a file stored them.
//!
//! Propagation (03 §4 rule 2) works on [`RowProvenance`]: rows that move keep their
//! provenance ([`RowProvenance::take`]); a value computed from several rows takes the
//! weakest class and the lowest confidence ([`RowProvenance::combine`]); an aggregate
//! also records `derived` ([`derived`]).

use std::collections::BTreeMap;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{Float32Type, Float64Type, UInt8Type};
use arrow_array::{
    Array, ArrayRef, DictionaryArray, Float32Array, RecordBatch, StringArray, UInt8Array,
};
use arrow_schema::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};
use sonarithm_core::{Confidence, Provenance, ProvenanceClass};

use crate::error::DataError;
use crate::table::{Table, dictionary_indices};

/// The class column (03 §4).
pub const PROV_COLUMN: &str = "__snr_prov";

/// The confidence column (03 §4).
pub const CONF_COLUMN: &str = "__snr_conf";

/// The classes in the order 03 §4 lists them. A `UInt8` class column stores the position
/// in this list, and the canonical dictionary uses the same order, so its keys are the
/// same codes.
pub const CLASS_CODES: [ProvenanceClass; 8] = [
    ProvenanceClass::Exact,
    ProvenanceClass::Vector,
    ProvenanceClass::CvEstimate,
    ProvenanceClass::MlEstimate,
    ProvenanceClass::CloudEstimate,
    ProvenanceClass::UserEdited,
    ProvenanceClass::Derived,
    ProvenanceClass::Synthetic,
];

/// The name of a class as written in `__snr_prov` and in specs (`cv_estimate`).
#[must_use]
pub fn class_name(class: ProvenanceClass) -> &'static str {
    match class {
        ProvenanceClass::Vector => "vector",
        ProvenanceClass::CvEstimate => "cv_estimate",
        ProvenanceClass::MlEstimate => "ml_estimate",
        ProvenanceClass::CloudEstimate => "cloud_estimate",
        ProvenanceClass::UserEdited => "user_edited",
        ProvenanceClass::Derived => "derived",
        ProvenanceClass::Synthetic => "synthetic",
        // `ProvenanceClass` is non-exhaustive; a class added later reads as exact here
        // until this list learns it.
        _ => "exact",
    }
}

fn class_of_name(name: &str) -> Option<ProvenanceClass> {
    CLASS_CODES
        .into_iter()
        .find(|class| class_name(*class) == name)
}

fn class_code(class: ProvenanceClass) -> u8 {
    CLASS_CODES
        .iter()
        .position(|c| *c == class)
        .and_then(|i| u8::try_from(i).ok())
        .unwrap_or(0)
}

/// The provenance of an aggregate of values with combined provenance `inputs`: at least
/// `derived`, and an estimate class when any input was an estimate (03 §4 rule 2: the
/// class itself flags estimated inputs).
#[must_use]
pub fn derived(inputs: Provenance) -> Provenance {
    Provenance {
        class: ProvenanceClass::Derived.weaker(inputs.class),
        confidence: inputs.confidence,
    }
}

/// The provenance of every row of a table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowProvenance {
    rows: Vec<Provenance>,
}

impl RowProvenance {
    /// `rows` exact rows (what absent columns mean).
    #[must_use]
    pub fn exact(rows: usize) -> Self {
        Self {
            rows: vec![Provenance::EXACT; rows],
        }
    }

    /// Row provenance from a list.
    #[must_use]
    pub fn from_rows(rows: Vec<Provenance>) -> Self {
        Self { rows }
    }

    /// Reads a table's provenance columns; absent columns mean exact rows.
    ///
    /// # Errors
    ///
    /// [`DataError::InvalidProvenance`] if a column has an unsupported type, an unknown
    /// class or a confidence outside [0, 1].
    pub fn read(table: &Table) -> Result<Self, DataError> {
        let rows = usize::try_from(table.num_rows()).unwrap_or(usize::MAX);
        let classes = match table.schema().index_of(PROV_COLUMN) {
            Ok(_) => Some(read_classes(&table.column(PROV_COLUMN)?)?),
            Err(_) => None,
        };
        let confidence = match table.schema().index_of(CONF_COLUMN) {
            Ok(_) => Some(read_confidence(&table.column(CONF_COLUMN)?)?),
            Err(_) => None,
        };
        let rows = (0..rows)
            .map(|row| Provenance {
                class: classes
                    .as_ref()
                    .and_then(|c| c.get(row).copied())
                    .unwrap_or_default(),
                confidence: confidence
                    .as_ref()
                    .and_then(|c| c.get(row).copied().flatten()),
            })
            .collect();
        Ok(Self { rows })
    }

    /// The number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether there are no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The provenance of one row; rows past the end are exact.
    #[must_use]
    pub fn get(&self, row: usize) -> Provenance {
        self.rows.get(row).copied().unwrap_or_default()
    }

    /// The rows, in order.
    #[must_use]
    pub fn rows(&self) -> &[Provenance] {
        &self.rows
    }

    /// Whether every row is exact with no confidence, so the columns can be left out.
    #[must_use]
    pub fn is_all_exact(&self) -> bool {
        self.rows.iter().all(|p| *p == Provenance::EXACT)
    }

    /// The provenance of the rows `rows` select, in that order (filters and sorts move
    /// rows with their provenance).
    #[must_use]
    pub fn take(&self, rows: &[usize]) -> Self {
        Self {
            rows: rows.iter().map(|&row| self.get(row)).collect(),
        }
    }

    /// The provenance of a value computed from `rows`: the weakest class and the lowest
    /// confidence (03 §4 rule 2). No rows give exact.
    #[must_use]
    pub fn combine(&self, rows: impl IntoIterator<Item = usize>) -> Provenance {
        rows.into_iter()
            .fold(Provenance::EXACT, |acc, row| acc.combine(self.get(row)))
    }

    /// How many rows each class has, and the confidence range (render manifests and the
    /// extraction announcement in 10 §5).
    #[must_use]
    pub fn mix(&self) -> ProvenanceMix {
        let mut counts = BTreeMap::new();
        let mut confidences: Vec<f32> = Vec::new();
        for p in &self.rows {
            *counts.entry(p.class).or_insert(0_u64) += 1;
            if let Some(c) = p.confidence {
                confidences.push(c.get());
            }
        }
        confidences.sort_by(f32::total_cmp);
        let median = match confidences.len() {
            0 => None,
            n if n % 2 == 1 => confidences.get(n / 2).copied(),
            n => match (confidences.get(n / 2 - 1), confidences.get(n / 2)) {
                (Some(a), Some(b)) => Some(a + (b - a) / 2.0),
                _ => None,
            },
        };
        ProvenanceMix {
            counts,
            min_confidence: confidences.first().copied().and_then(Confidence::new),
            median_confidence: median.and_then(Confidence::new),
        }
    }

    /// The canonical columns: `__snr_prov` as `Dictionary<UInt8, Utf8>` over all classes in
    /// [`CLASS_CODES`] order, and `__snr_conf` as nullable `Float32`.
    #[must_use]
    pub fn columns(&self) -> (ArrayRef, ArrayRef) {
        let keys: UInt8Array = self
            .rows
            .iter()
            .map(|p| Some(class_code(p.class)))
            .collect();
        let values = Arc::new(StringArray::from_iter_values(CLASS_CODES.map(class_name)));
        let classes: ArrayRef = match DictionaryArray::<UInt8Type>::try_new(keys, values) {
            Ok(array) => Arc::new(array),
            // Keys are always in range of the eight-entry dictionary.
            Err(_) => arrow_array::new_null_array(&prov_type(), self.rows.len()),
        };
        let confidence: Float32Array = self
            .rows
            .iter()
            .map(|p| p.confidence.map(Confidence::get))
            .collect();
        (classes, Arc::new(confidence))
    }
}

/// How the rows of a table divide among provenance classes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceMix {
    /// Rows per class; classes without rows are left out.
    pub counts: BTreeMap<ProvenanceClass, u64>,
    /// The lowest confidence of any row that has one.
    pub min_confidence: Option<Confidence>,
    /// The median confidence of the rows that have one.
    pub median_confidence: Option<Confidence>,
}

impl ProvenanceMix {
    /// Rows whose class is an estimate.
    #[must_use]
    pub fn estimated(&self) -> u64 {
        self.counts
            .iter()
            .filter(|(class, _)| class.is_estimate())
            .map(|(_, n)| n)
            .sum()
    }
}

/// The canonical type of `__snr_prov`.
fn prov_type() -> DataType {
    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
}

fn invalid(reason: String) -> DataError {
    DataError::InvalidProvenance { reason }
}

fn read_classes(column: &ArrayRef) -> Result<Vec<ProvenanceClass>, DataError> {
    let by_name = |name: Option<&str>| -> Result<ProvenanceClass, DataError> {
        match name {
            None => Ok(ProvenanceClass::Exact),
            Some(name) => class_of_name(name.trim())
                .ok_or_else(|| invalid(format!("`{PROV_COLUMN}` has an unknown class `{name}`"))),
        }
    };
    match column.data_type() {
        DataType::Utf8 => column.as_string::<i32>().iter().map(by_name).collect(),
        DataType::LargeUtf8 => column.as_string::<i64>().iter().map(by_name).collect(),
        DataType::UInt8 => column
            .as_primitive::<UInt8Type>()
            .iter()
            .map(|code| match code {
                None => Ok(ProvenanceClass::Exact),
                Some(code) => CLASS_CODES.get(usize::from(code)).copied().ok_or_else(|| {
                    invalid(format!("`{PROV_COLUMN}` has an unknown class code {code}"))
                }),
            })
            .collect(),
        DataType::Dictionary(_, value)
            if matches!(**value, DataType::Utf8 | DataType::LargeUtf8) =>
        {
            let keys = dictionary_indices(column.as_ref()).map_err(|row| {
                invalid(format!(
                    "`{PROV_COLUMN}` row {row} has a dictionary key outside its values"
                ))
            })?;
            let decoded =
                arrow_select::take::take(column.as_any_dictionary().values().as_ref(), &keys, None)
                    .map_err(|err| invalid(err.to_string()))?;
            read_classes(&decoded)
        }
        other => Err(invalid(format!(
            "`{PROV_COLUMN}` must be text, a text dictionary or UInt8, not {other}"
        ))),
    }
}

fn read_confidence(column: &ArrayRef) -> Result<Vec<Option<Confidence>>, DataError> {
    let check = |value: Option<f64>| -> Result<Option<Confidence>, DataError> {
        match value {
            None => Ok(None),
            #[expect(clippy::cast_possible_truncation, reason = "values in [0, 1] fit f32")]
            Some(v) if (0.0..=1.0).contains(&v) => Ok(Confidence::new(v as f32)),
            Some(v) => Err(invalid(format!("`{CONF_COLUMN}` has {v}, outside [0, 1]"))),
        }
    };
    match column.data_type() {
        DataType::Float32 => column
            .as_primitive::<Float32Type>()
            .iter()
            .map(|v| check(v.map(f64::from)))
            .collect(),
        DataType::Float64 => column
            .as_primitive::<Float64Type>()
            .iter()
            .map(check)
            .collect(),
        DataType::Null => Ok(vec![None; column.len()]),
        other => Err(invalid(format!(
            "`{CONF_COLUMN}` must be Float32 or Float64, not {other}"
        ))),
    }
}

/// Appends the canonical provenance columns to data fields and columns, unless every row is
/// exact and `keep` is not set.
pub(crate) fn append_columns(
    fields: &mut Vec<Field>,
    columns: &mut Vec<ArrayRef>,
    provenance: &RowProvenance,
    keep: bool,
) {
    if keep || !provenance.is_all_exact() {
        let (classes, confidence) = provenance.columns();
        fields.push(Field::new(PROV_COLUMN, prov_type(), false));
        fields.push(Field::new(CONF_COLUMN, DataType::Float32, true));
        columns.push(classes);
        columns.push(confidence);
    }
}

/// Rewrites a table's provenance columns in the canonical form; a table without them is
/// returned unchanged.
///
/// # Errors
///
/// [`DataError::InvalidProvenance`] as [`RowProvenance::read`].
pub fn normalize(table: &Table) -> Result<Table, DataError> {
    let has = |name| table.schema().index_of(name).is_ok();
    if !has(PROV_COLUMN) && !has(CONF_COLUMN) {
        return Ok(table.clone());
    }
    let provenance = RowProvenance::read(table)?;
    with_provenance(table, &provenance, true)
}

/// The table with its provenance columns replaced by `provenance` (canonical form, at the
/// end of the schema). When every row is exact the columns are dropped, unless `keep` is
/// set, since absent columns mean exact (03 §4 rule 1).
///
/// # Errors
///
/// [`DataError::MalformedTable`] if `provenance` has a different row count.
pub fn with_provenance(
    table: &Table,
    provenance: &RowProvenance,
    keep: bool,
) -> Result<Table, DataError> {
    let batch = table.to_batch()?;
    let mut fields: Vec<Field> = Vec::new();
    let mut columns: Vec<ArrayRef> = Vec::new();
    for (field, column) in table.schema().fields().iter().zip(batch.columns()) {
        if field.name() != PROV_COLUMN && field.name() != CONF_COLUMN {
            fields.push(field.as_ref().clone());
            columns.push(Arc::clone(column));
        }
    }
    append_columns(&mut fields, &mut columns, provenance, keep);
    let schema = Arc::new(Schema::new(fields).with_metadata(table.schema().metadata().clone()));
    let options = arrow_array::RecordBatchOptions::new().with_row_count(Some(batch.num_rows()));
    let batch = RecordBatch::try_new_with_options(Arc::clone(&schema), columns, &options).map_err(
        |source| DataError::MalformedTable {
            table: table.name().to_owned(),
            source,
        },
    )?;
    Table::new(table.id(), table.name(), schema, vec![batch])
}

#[cfg(test)]
mod tests {
    use arrow_array::Float64Array;
    use proptest::prelude::*;
    use sonarithm_core::TableId;

    use super::*;

    fn table(columns: Vec<(&str, ArrayRef)>) -> Table {
        let fields: Vec<Field> = columns
            .iter()
            .map(|(n, a)| Field::new(*n, a.data_type().clone(), true))
            .collect();
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(fields)),
            columns.into_iter().map(|(_, a)| a).collect(),
        )
        .unwrap();
        Table::from_batch(TableId(0), "t", batch)
    }

    fn any_class() -> impl Strategy<Value = ProvenanceClass> {
        prop::sample::select(CLASS_CODES.to_vec())
    }

    fn any_provenance() -> impl Strategy<Value = Provenance> {
        (any_class(), prop::option::of(0.0_f32..=1.0)).prop_map(|(class, c)| Provenance {
            class,
            confidence: c.and_then(Confidence::new),
        })
    }

    #[test]
    fn absent_columns_mean_exact() {
        let t = table(vec![("v", Arc::new(Float64Array::from(vec![1.0, 2.0])))]);
        let p = RowProvenance::read(&t).unwrap();
        assert_eq!(p, RowProvenance::exact(2));
        assert!(p.is_all_exact());
        assert_eq!(normalize(&t).unwrap(), t);
    }

    #[test]
    fn text_codes_and_dictionaries_normalize_to_one_form() {
        let v: ArrayRef = Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0]));
        let text: ArrayRef = Arc::new(StringArray::from(vec![
            Some("exact"),
            Some("cv_estimate"),
            None,
        ]));
        let codes: ArrayRef = Arc::new(UInt8Array::from(vec![0, 2, 0]));
        let dict: ArrayRef = Arc::new(
            vec!["exact", "cv_estimate", "exact"]
                .into_iter()
                .collect::<DictionaryArray<arrow_array::types::Int32Type>>(),
        );
        let conf: ArrayRef = Arc::new(Float64Array::from(vec![None, Some(0.75), None]));
        let forms: Vec<Table> = [text, codes, dict]
            .into_iter()
            .map(|prov| {
                normalize(&table(vec![
                    ("v", v.clone()),
                    (PROV_COLUMN, prov),
                    (CONF_COLUMN, conf.clone()),
                ]))
                .unwrap()
            })
            .collect();
        assert_eq!(forms[0], forms[1]);
        assert_eq!(forms[1], forms[2]);
        let p = RowProvenance::read(&forms[0]).unwrap();
        assert_eq!(p.get(1).class, ProvenanceClass::CvEstimate);
        assert_eq!(p.get(1).confidence, Confidence::new(0.75));
        assert_eq!(forms[0].schema().field(1).data_type(), &prov_type());
    }

    #[test]
    fn unknown_classes_and_bad_confidence_are_errors() {
        let v: ArrayRef = Arc::new(Float64Array::from(vec![1.0]));
        let bad_class: ArrayRef = Arc::new(StringArray::from(vec!["guess"]));
        let err = normalize(&table(vec![("v", v.clone()), (PROV_COLUMN, bad_class)])).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-041");
        let bad_conf: ArrayRef = Arc::new(Float64Array::from(vec![1.5]));
        let err = normalize(&table(vec![("v", v.clone()), (CONF_COLUMN, bad_conf)])).unwrap_err();
        assert!(err.to_string().contains("outside [0, 1]"), "{err}");
        let bad_code: ArrayRef = Arc::new(UInt8Array::from(vec![8]));
        assert!(normalize(&table(vec![("v", v), (PROV_COLUMN, bad_code)])).is_err());
    }

    #[test]
    fn aggregates_record_derived_and_flag_estimates_by_class() {
        let exact = Provenance::EXACT;
        assert_eq!(derived(exact).class, ProvenanceClass::Derived);
        let estimate = Provenance {
            class: ProvenanceClass::MlEstimate,
            confidence: Confidence::new(0.6),
        };
        assert_eq!(derived(exact.combine(estimate)), estimate);
    }

    #[test]
    fn the_mix_counts_classes_and_confidence() {
        let rows = vec![
            Provenance::EXACT,
            Provenance {
                class: ProvenanceClass::CvEstimate,
                confidence: Confidence::new(0.5),
            },
            Provenance {
                class: ProvenanceClass::CvEstimate,
                confidence: Confidence::new(0.9),
            },
        ];
        let mix = RowProvenance::from_rows(rows).mix();
        assert_eq!(mix.counts.get(&ProvenanceClass::CvEstimate), Some(&2));
        assert_eq!(mix.estimated(), 2);
        assert_eq!(mix.min_confidence, Confidence::new(0.5));
        let median = mix.median_confidence.unwrap().get();
        assert!((median - 0.7).abs() < 1e-6, "{median}");
    }

    proptest! {
        #[test]
        fn columns_round_trip(rows in prop::collection::vec(any_provenance(), 0..40)) {
            let p = RowProvenance::from_rows(rows);
            let v: ArrayRef = Arc::new(Float64Array::from(vec![0.0; p.len()]));
            let t = with_provenance(&table(vec![("v", v)]), &p, true).unwrap();
            prop_assert_eq!(RowProvenance::read(&t).unwrap(), p);
        }

        #[test]
        fn combining_takes_the_weakest_class_and_the_lowest_confidence(
            rows in prop::collection::vec(any_provenance(), 1..20),
            picks in prop::collection::vec(0_usize..20, 0..10),
        ) {
            let p = RowProvenance::from_rows(rows.clone());
            let picks: Vec<usize> = picks.into_iter().filter(|&i| i < rows.len()).collect();
            let combined = p.combine(picks.iter().copied());
            let weakest = picks.iter().map(|&i| rows[i].class).max().unwrap_or_default();
            let lowest = picks.iter().filter_map(|&i| rows[i].confidence).map(Confidence::get).reduce(f32::min);
            prop_assert_eq!(combined.class, weakest);
            prop_assert_eq!(combined.confidence.map(Confidence::get), lowest);
            // Order does not matter.
            let reversed = p.combine(picks.iter().rev().copied());
            prop_assert_eq!(combined, reversed);
        }

        #[test]
        fn taking_rows_moves_their_provenance(
            rows in prop::collection::vec(any_provenance(), 1..20),
            order in prop::collection::vec(0_usize..20, 0..20),
        ) {
            let p = RowProvenance::from_rows(rows.clone());
            let order: Vec<usize> = order.into_iter().filter(|&i| i < rows.len()).collect();
            let taken = p.take(&order);
            for (i, &row) in order.iter().enumerate() {
                prop_assert_eq!(taken.get(i), rows[row]);
            }
        }
    }
}
