// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Transforms (03 §6): pure functions from a table to its next generation, run in the
//! order a spec lists them, each propagating provenance (03 §4) and recorded in the
//! bundle's transform log, which render manifests include.
//!
//! Parameters mirror RFC-0001 §4; the compiler maps a spec's `transform` array onto
//! [`Transform`]. Expressions (`filter`, `derive`) use the sandboxed rhai subset described
//! in [`expr`](self#expressions).
//!
//! # Expressions
//!
//! See the module documentation of `transform::expr`: one rhai expression over the current
//! row, fields as variables or `col("name")`, missing values as `()`, deterministic maths,
//! and at most 10 000 operations per row.

mod aggregate;
mod expr;
mod rows;
mod series;

use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, Int64Array, RecordBatch, RecordBatchOptions};
use arrow_schema::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};
use sonarithm_core::{Diagnostic, Diagnostics, Seconds, TableId};

use crate::bundle::DataBundle;
use crate::codes;
use crate::error::DataError;
use crate::meta::{FieldMeta, FieldType};
use crate::provenance::{self, CONF_COLUMN, PROV_COLUMN, RowProvenance};
use crate::table::Table;

pub use aggregate::{AggregateFn, AggregateOp, BinMethod};
pub use expr::FUEL_PER_ROW;
pub use series::{NormalizeMethod, ResampleMethod, SmoothMethod};

/// One transform (03 §6, RFC-0001 §4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
#[non_exhaustive]
pub enum Transform {
    /// Keeps the rows for which `expr` is `true`.
    Filter {
        /// A boolean expression.
        expr: String,
    },
    /// Orders rows by one field; stable, missing values last.
    Sort {
        /// The field to sort by.
        field: String,
        /// The direction.
        #[serde(default)]
        order: SortOrder,
    },
    /// Groups rows by `groupby` (in order of first appearance; no fields: one group) and
    /// computes `ops` per group.
    Aggregate {
        /// The fields that form the group key.
        #[serde(default)]
        groupby: Vec<String>,
        /// The outputs.
        ops: Vec<AggregateOp>,
    },
    /// Adds each row's bin as `{as}` (start) and `{as}_end`.
    Bin {
        /// The number field to bin.
        field: String,
        /// How edges are placed.
        #[serde(default)]
        method: BinMethod,
        /// The number of bins (`equal-width`, `quantile`).
        #[serde(default)]
        count: Option<u32>,
        /// The edges, increasing (`edges`).
        #[serde(default)]
        edges: Option<Vec<f64>>,
        /// The name of the start field.
        #[serde(rename = "as")]
        output: String,
    },
    /// Smooths a number field with a moving average (SDR-21).
    Smooth {
        /// The number field.
        field: String,
        /// `ema` or `sma`.
        method: SmoothMethod,
        /// The weight of the newest value for `ema`, in (0, 1].
        #[serde(default)]
        alpha: Option<f64>,
        /// The number of rows for `sma`, at least 1.
        #[serde(default)]
        window: Option<u32>,
        /// The output field; by default the input field is replaced.
        #[serde(default, rename = "as")]
        output: Option<String>,
    },
    /// Regularizes the table on a time grid.
    Resample {
        /// The time field: dates, timestamps or numbers.
        field: String,
        /// The grid step; for a number field, in the field's own units.
        every: Seconds,
        /// How each grid point is filled.
        method: ResampleMethod,
    },
    /// Adds (or replaces) a field computed from each row.
    Derive {
        /// The expression.
        expr: String,
        /// The name of the new field.
        #[serde(rename = "as")]
        output: String,
    },
    /// Rescales a number field.
    Normalize {
        /// The number field.
        field: String,
        /// `z-score`, `min-max` or `rank`.
        method: NormalizeMethod,
        /// The output field; by default the input field is replaced.
        #[serde(default, rename = "as")]
        output: Option<String>,
    },
}

impl Transform {
    /// The operation name, as in specs (`filter`).
    #[must_use]
    pub const fn op(&self) -> &'static str {
        match self {
            Self::Filter { .. } => "filter",
            Self::Sort { .. } => "sort",
            Self::Aggregate { .. } => "aggregate",
            Self::Bin { .. } => "bin",
            Self::Smooth { .. } => "smooth",
            Self::Resample { .. } => "resample",
            Self::Normalize { .. } => "normalize",
            Self::Derive { .. } => "derive",
        }
    }
}

/// A sort direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Smallest first.
    #[default]
    Ascending,
    /// Largest first.
    Descending,
}

/// One entry of a bundle's transform log.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TransformRecord {
    /// The table the transform ran on.
    pub table: TableId,
    /// The transform.
    pub transform: Transform,
    /// Rows before.
    pub rows_in: u64,
    /// Rows after.
    pub rows_out: u64,
}

/// A table taken apart for transforming: data fields and columns, with the provenance
/// columns held separately.
pub(crate) struct Working {
    pub(crate) fields: Vec<Field>,
    pub(crate) columns: Vec<ArrayRef>,
    pub(crate) provenance: RowProvenance,
    pub(crate) rows: usize,
    pub(crate) table_name: String,
}

impl Working {
    fn of(table: &Table) -> Result<Self, DataError> {
        let provenance = RowProvenance::read(table)?;
        let batch = table.to_batch()?;
        let mut fields = Vec::new();
        let mut columns = Vec::new();
        for (field, column) in table.schema().fields().iter().zip(batch.columns()) {
            if field.name() != PROV_COLUMN && field.name() != CONF_COLUMN {
                fields.push(field.as_ref().clone());
                columns.push(Arc::clone(column));
            }
        }
        Ok(Self {
            fields,
            columns,
            provenance,
            rows: batch.num_rows(),
            table_name: table.name().to_owned(),
        })
    }

    /// The position of a field.
    pub(crate) fn index(&self, field: &str) -> Result<usize, DataError> {
        self.fields
            .iter()
            .position(|f| f.name() == field)
            .ok_or_else(|| DataError::UnknownField {
                table: self.table_name.clone(),
                field: field.to_owned(),
            })
    }

    /// The data schema (without provenance columns).
    pub(crate) fn schema(&self) -> Schema {
        Schema::new(self.fields.clone())
    }

    /// Adds a field, replacing one with the same name in place.
    pub(crate) fn put(&mut self, field: Field, column: ArrayRef) {
        let at = self.fields.iter().position(|f| f.name() == field.name());
        if let Some(i) = at
            && let (Some(f), Some(c)) = (self.fields.get_mut(i), self.columns.get_mut(i))
        {
            *f = field;
            *c = column;
        } else {
            self.fields.push(field);
            self.columns.push(column);
        }
    }

    /// The next generation of `table` with this content; provenance columns are written
    /// only when some row is not exact.
    fn into_table(mut self, table: &Table) -> Result<Table, DataError> {
        provenance::append_columns(&mut self.fields, &mut self.columns, &self.provenance, false);
        let schema =
            Arc::new(Schema::new(self.fields).with_metadata(table.schema().metadata().clone()));
        let options = RecordBatchOptions::new().with_row_count(Some(self.rows));
        let batch = RecordBatch::try_new_with_options(Arc::clone(&schema), self.columns, &options)
            .map_err(|source| DataError::MalformedTable {
                table: table.name().to_owned(),
                source,
            })?;
        table.next_generation(schema, vec![batch])
    }
}

/// A 64-bit column (timestamps, dates in milliseconds, durations) as its raw `Int64`
/// values; the buffer is shared, not copied.
pub(crate) fn raw_i64(column: &ArrayRef) -> Int64Array {
    reinterpret(column, &DataType::Int64)
        .as_primitive::<Int64Type>()
        .clone()
}

/// The same 64-bit buffer viewed as another 64-bit type (for example `Int64` as a
/// timestamp with a time zone); an all-missing column of that type if it is not 64-bit.
pub(crate) fn reinterpret(column: &ArrayRef, data_type: &DataType) -> ArrayRef {
    match column
        .to_data()
        .into_builder()
        .data_type(data_type.clone())
        .build()
    {
        Ok(data) => arrow_array::make_array(data),
        Err(_) => arrow_array::new_null_array(data_type, column.len()),
    }
}

/// A field for a computed column: its type from the storage, no other semantics.
pub(crate) fn computed_field(name: &str, column: &ArrayRef) -> Field {
    let ftype = if column.data_type().is_numeric() {
        FieldType::Quantitative
    } else if column.data_type().is_temporal() {
        FieldType::Temporal
    } else {
        FieldType::Nominal
    };
    FieldMeta::new()
        .with_type(ftype)
        .apply(Field::new(name, column.data_type().clone(), true))
}

/// Applies one transform, returning the table's next generation.
///
/// # Errors
///
/// [`DataError::UnknownField`], [`DataError::FieldTypeMismatch`],
/// [`DataError::InvalidTransform`] for bad parameters; [`DataError::InvalidExpression`] and
/// [`DataError::ExpressionBudget`] for expressions.
pub fn apply(
    table: &Table,
    transform: &Transform,
    diagnostics: &mut Diagnostics,
) -> Result<Table, DataError> {
    let mut working = Working::of(table)?;
    let rows_in = working.rows;
    let summary = match transform {
        Transform::Filter { expr } => rows::filter(&mut working, expr, diagnostics)?,
        Transform::Sort { field, order } => rows::sort(&mut working, field, *order)?,
        Transform::Aggregate { groupby, ops } => aggregate::aggregate(&mut working, groupby, ops)?,
        Transform::Bin {
            field,
            method,
            count,
            edges,
            output,
        } => aggregate::bin(
            &mut working,
            field,
            *method,
            *count,
            edges.as_deref(),
            output,
        )?,
        Transform::Smooth {
            field,
            method,
            alpha,
            window,
            output,
        } => series::smooth(
            &mut working,
            field,
            *method,
            *alpha,
            *window,
            output.as_deref(),
        )?,
        Transform::Resample {
            field,
            every,
            method,
        } => series::resample(&mut working, field, *every, *method)?,
        Transform::Derive { expr, output } => {
            rows::derive(&mut working, expr, output, diagnostics)?
        }
        Transform::Normalize {
            field,
            method,
            output,
        } => series::normalize(&mut working, field, *method, output.as_deref())?,
    };
    diagnostics.push(Diagnostic::new(
        &codes::TRANSFORM_APPLIED,
        format!(
            "Transform {} ({summary}): {rows_in} rows in, {} rows out.",
            transform.op(),
            working.rows
        ),
    ));
    working.into_table(table)
}

/// Applies transforms in order to one table of a bundle, replacing it with its last
/// generation and appending each step to the bundle's transform log.
///
/// # Errors
///
/// As [`apply`]; [`DataError::UnknownTable`] if the bundle has no such table. The bundle is
/// unchanged when a transform fails.
pub fn apply_all(
    bundle: &mut DataBundle,
    table: TableId,
    transforms: &[Transform],
) -> Result<Diagnostics, DataError> {
    let mut current = bundle
        .table(table)
        .cloned()
        .ok_or(DataError::UnknownTable { id: table.0 })?;
    let mut diagnostics = Diagnostics::new();
    let mut records = Vec::with_capacity(transforms.len());
    for transform in transforms {
        let next = apply(&current, transform, &mut diagnostics)?;
        records.push(TransformRecord {
            table,
            transform: transform.clone(),
            rows_in: current.num_rows(),
            rows_out: next.num_rows(),
        });
        current = next;
    }
    bundle.put_table(current);
    for record in records {
        bundle.log_transform(record);
    }
    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    use arrow_array::Float64Array;
    use sonarithm_core::Generation;

    use super::*;
    use crate::bundle::{BundleId, Format, SourceDescriptor, SourceKind};

    fn bundle() -> DataBundle {
        let schema = Arc::new(Schema::new(vec![Field::new("v", DataType::Float64, true)]));
        let batch = RecordBatch::try_new(
            schema,
            vec![Arc::new(Float64Array::from(vec![3.0, -1.0, 2.0]))],
        )
        .unwrap();
        let source = SourceDescriptor::new(SourceKind::File(Format::Csv), "v.csv", "csv");
        DataBundle::single(
            BundleId(1),
            source,
            Table::from_batch(TableId(0), "v", batch),
        )
    }

    #[test]
    fn apply_all_logs_each_step_and_keeps_the_bundle_on_failure() {
        let mut b = bundle();
        let steps = [
            Transform::Filter {
                expr: "v > 0.0".into(),
            },
            Transform::Sort {
                field: "v".into(),
                order: SortOrder::Ascending,
            },
        ];
        let diagnostics = apply_all(&mut b, TableId(0), &steps).unwrap();
        assert_eq!(diagnostics.len(), 2);
        let log: Vec<_> = b
            .transform_log()
            .iter()
            .map(|r| (r.transform.op(), r.rows_in, r.rows_out))
            .collect();
        assert_eq!(log, [("filter", 3, 2), ("sort", 2, 2)]);
        assert_eq!(b.primary().unwrap().generation(), Generation(2));
        let before = b.clone();
        let bad = [Transform::Sort {
            field: "nope".into(),
            order: SortOrder::Ascending,
        }];
        assert!(apply_all(&mut b, TableId(0), &bad).is_err());
        assert_eq!(b, before);
        assert!(apply_all(&mut b, TableId(9), &steps).is_err());
    }

    #[test]
    fn transforms_deserialize_from_spec_json() {
        let parsed: Vec<Transform> = serde_json::from_str(
            r#"[{"op": "filter", "expr": "t > 0"},
                {"op": "sort", "field": "t", "order": "descending"},
                {"op": "derive", "expr": "v * 2", "as": "v2"}]"#,
        )
        .unwrap();
        assert_eq!(
            parsed[1],
            Transform::Sort {
                field: "t".into(),
                order: SortOrder::Descending
            }
        );
        assert_eq!(parsed[2].op(), "derive");
        assert!(
            serde_json::from_str::<Transform>(r#"{"op": "filter", "expr": "x", "extra": 1}"#)
                .is_err()
        );
    }
}
