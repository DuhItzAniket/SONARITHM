// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The rows a track plays, in the order it plays them (RFC-0009 §1): the rows its filter
//! keeps, without the rows that have no time value, ordered by the time field. NUMISONA's
//! encoders select and order a track's rows the same way (`encode.rs`, steps 1 and 2); an
//! analyzer that reads a track's series reads it in this order, so what it finds is what the
//! listener hears. A test in ECHOMETRIC checks the two agree on every encoded note.

use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, Int64Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use sonarithm_core::diagnostic::Diagnostics;

use crate::error::DataError;
use crate::meta::Role;
use crate::table::Table;
use crate::transform::{Transform, apply};
use crate::view::axis_values;

/// The column that carries each row's index through the filter.
const ROW_COLUMN: &str = "__snr_play_row";

/// The rows of `table` a track plays, in play order: those `filter` (the data layer's
/// expression language) keeps, in table order; then, when `time` names a field or the table
/// has a field with role `x`, without the rows whose time value is missing, ordered by it
/// (stable, so rows with equal times keep their table order). A time field that is not
/// numeric or temporal leaves the table order.
///
/// # Errors
///
/// The filter's [`DataError`] when it cannot be applied.
pub fn play_rows(
    table: &Table,
    filter: Option<&str>,
    time: Option<&str>,
) -> Result<Vec<u64>, DataError> {
    let mut rows = filtered(table, filter)?;
    let time = time.map(str::to_owned).or_else(|| x_field(table));
    if let Some(field) = time
        && let Ok(values) = axis_values(table, &field)
    {
        let value = |row: u64| {
            usize::try_from(row)
                .ok()
                .filter(|&at| at < values.len() && values.is_valid(at))
                .map(|at| values.value(at))
                .filter(|v| !v.is_nan())
        };
        rows.retain(|&row| value(row).is_some());
        rows.sort_by(|&a, &b| {
            let (a, b) = (value(a).unwrap_or(f64::NAN), value(b).unwrap_or(f64::NAN));
            a.total_cmp(&b)
        });
    }
    Ok(rows)
}

/// The field whose metadata gives it role `x`, if any.
fn x_field(table: &Table) -> Option<String> {
    table
        .schema()
        .fields()
        .iter()
        .find(|field| {
            table
                .field_meta(field.name())
                .is_ok_and(|meta| meta.role == Some(Role::X))
        })
        .map(|field| field.name().clone())
}

/// The rows `filter` keeps, in table order; every row without one.
fn filtered(table: &Table, filter: Option<&str>) -> Result<Vec<u64>, DataError> {
    let count = table.num_rows();
    let Some(expr) = filter else {
        return Ok((0..count).collect());
    };
    let batch = table.to_batch()?;
    let schema = batch.schema();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
    fields.push(Arc::new(Field::new(ROW_COLUMN, DataType::Int64, false)));
    let tagged_schema = Arc::new(Schema::new(fields).with_metadata(schema.metadata().clone()));
    let numbers = Int64Array::from_iter_values(0..i64::try_from(count).unwrap_or(i64::MAX));
    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();
    columns.push(Arc::new(numbers));
    let tagged = RecordBatch::try_new(Arc::clone(&tagged_schema), columns).map_err(|err| {
        DataError::InvalidView {
            reason: err.to_string(),
        }
    })?;
    let tagged = table.next_generation(tagged_schema, vec![tagged])?;
    let kept = apply(
        &tagged,
        &Transform::Filter {
            expr: expr.to_owned(),
        },
        &mut Diagnostics::new(),
    )?;
    let rows = kept.column(ROW_COLUMN)?;
    Ok(rows
        .as_primitive_opt::<Int64Type>()
        .map(|rows| {
            rows.values()
                .iter()
                .filter_map(|&row| u64::try_from(row).ok())
                .collect()
        })
        .unwrap_or_default())
}
