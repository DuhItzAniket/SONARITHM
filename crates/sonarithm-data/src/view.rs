// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pages of table rows for views (03 §7; B04-T03): the always-available data table reads a
//! table page by page, optionally sorted, without the UI ever holding the whole table.
//!
//! A page keeps each row's index (so the table view links rows to the cursor through
//! `RowRef`) and converts every column to one of four plain types the UI can show:
//! 64-bit floats, 64-bit integers, text and booleans. Dates and timestamps become ISO 8601
//! text, which a screen reader reads as a date. Provenance columns are pages' text like any
//! other, so estimates are named in the table (03 §4, AGENTS.md rule 7).

use arrow_array::cast::AsArray as _;
use arrow_array::types::{
    Date32Type, Date64Type, Float16Type, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type,
    Int64Type, TimestampMicrosecondType, TimestampMillisecondType, TimestampNanosecondType,
    TimestampSecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow_array::{Array, ArrayRef, Float64Array};
use arrow_ord::ord::make_comparator;
use arrow_schema::{DataType, SortOptions, TimeUnit};

use sonarithm_core::provenance::{Confidence, ProvenanceClass};

use crate::error::DataError;
use crate::infer::semantic::numbers;
use crate::provenance::{CLASS_CODES, CONF_COLUMN, PROV_COLUMN, RowProvenance};
use crate::table::Table;

fn has_provenance(table: &Table) -> bool {
    let schema = table.schema();
    schema.index_of(PROV_COLUMN).is_ok() || schema.index_of(CONF_COLUMN).is_ok()
}

/// A class's code: its position in the 03 §4 list.
fn class_code(class: ProvenanceClass) -> u8 {
    CLASS_CODES
        .iter()
        .position(|c| *c == class)
        .and_then(|i| u8::try_from(i).ok())
        .unwrap_or(0)
}

/// The most rows one page may hold.
pub const MAX_PAGE_ROWS: u32 = 10_000;

/// A column of a page.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum PageValues {
    /// Floats; `None` where a row has no value.
    F64(Vec<Option<f64>>),
    /// Integers.
    I64(Vec<Option<i64>>),
    /// Text.
    Text(Vec<Option<String>>),
    /// Booleans.
    Bool(Vec<Option<bool>>),
    /// Provenance class codes, the positions in `provenance::CLASS_CODES` (03 §4).
    Provenance(Vec<u8>),
    /// Confidences; `None` where not applicable.
    Confidence(Vec<Option<f32>>),
}

/// One page of a table.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Page {
    /// The table's row count.
    pub total_rows: u64,
    /// The row indices on the page, in display order.
    pub rows: Vec<u64>,
    /// Every column, in table order: name and values.
    pub columns: Vec<(String, PageValues)>,
}

/// How a page is ordered.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PageSort {
    /// The field to sort by.
    pub field: String,
    /// Largest first.
    pub descending: bool,
}

impl PageSort {
    /// An order by `field`.
    #[must_use]
    pub fn new(field: impl Into<String>, descending: bool) -> Self {
        Self {
            field: field.into(),
            descending,
        }
    }
}

/// The values of `field` as floats: numbers, dates, timestamps and durations (03 §7).
///
/// # Errors
///
/// [`DataError::UnknownField`], or [`DataError::FieldTypeMismatch`] for other types.
pub fn axis_values(table: &Table, field: &str) -> Result<Float64Array, DataError> {
    let column = table.column(field)?;
    numbers(&column).ok_or_else(|| DataError::FieldTypeMismatch {
        op: "view",
        field: field.to_owned(),
        expected: "numbers, dates or timestamps",
        found: column.data_type().to_string(),
    })
}

/// Rows `offset..offset + limit` of `table` in `sort` order (stable; missing values last),
/// or in table order.
///
/// # Errors
///
/// [`DataError::InvalidView`] for a limit above [`MAX_PAGE_ROWS`];
/// [`DataError::UnknownField`] or [`DataError::FieldTypeMismatch`] for a sort field that is
/// missing or cannot be ordered.
pub fn table_page(
    table: &Table,
    offset: u64,
    limit: u32,
    sort: Option<&PageSort>,
) -> Result<Page, DataError> {
    if limit > MAX_PAGE_ROWS {
        return Err(DataError::InvalidView {
            reason: format!("a page holds at most {MAX_PAGE_ROWS} rows"),
        });
    }
    let total = table.num_rows();
    let order: Vec<u64> = match sort {
        None => {
            let end = offset.saturating_add(u64::from(limit)).min(total);
            (offset.min(total)..end).collect()
        }
        Some(sort) => {
            let column = table.column(&sort.field)?;
            let options = SortOptions {
                descending: sort.descending,
                nulls_first: false,
            };
            let compare =
                make_comparator(column.as_ref(), column.as_ref(), options).map_err(|_| {
                    DataError::FieldTypeMismatch {
                        op: "sort",
                        field: sort.field.clone(),
                        expected: "comparable values",
                        found: column.data_type().to_string(),
                    }
                })?;
            let mut rows: Vec<usize> = (0..column.len()).collect();
            rows.sort_by(|&a, &b| compare(a, b));
            rows.into_iter()
                .skip(usize::try_from(offset).unwrap_or(usize::MAX))
                .take(limit as usize)
                .map(|r| r as u64)
                .collect()
        }
    };
    let mut columns = Vec::new();
    let provenance = has_provenance(table)
        .then(|| RowProvenance::read(table))
        .transpose()?;
    for field in table.schema().fields() {
        let name = field.name();
        let page_values = match (&provenance, name.as_str()) {
            (Some(rows), PROV_COLUMN) => PageValues::Provenance(
                order
                    .iter()
                    .map(|&r| class_code(rows.get(usize::try_from(r).unwrap_or(0)).class))
                    .collect(),
            ),
            (Some(rows), CONF_COLUMN) => PageValues::Confidence(
                order
                    .iter()
                    .map(|&r| {
                        rows.get(usize::try_from(r).unwrap_or(0))
                            .confidence
                            .map(Confidence::get)
                    })
                    .collect(),
            ),
            _ => values(&table.column(name)?, &order),
        };
        columns.push((name.clone(), page_values));
    }
    Ok(Page {
        total_rows: total,
        rows: order,
        columns,
    })
}

/// The values of `rows` in `column`, in one of the page types.
fn values(column: &ArrayRef, rows: &[u64]) -> PageValues {
    let idx = |r: u64| usize::try_from(r).unwrap_or(usize::MAX);
    let valid = |r: u64| idx(r) < column.len() && column.is_valid(idx(r));
    macro_rules! ints {
        ($t:ty) => {{
            let a = column.as_primitive::<$t>();
            PageValues::I64(
                rows.iter()
                    .map(|&r| {
                        valid(r)
                            .then(|| i64::try_from(a.value(idx(r))).ok())
                            .flatten()
                    })
                    .collect(),
            )
        }};
    }
    macro_rules! floats {
        ($t:ty) => {{
            let a = column.as_primitive::<$t>();
            PageValues::F64(
                rows.iter()
                    .map(|&r| valid(r).then(|| f64::from(a.value(idx(r)))))
                    .collect(),
            )
        }};
    }
    macro_rules! timestamps {
        ($t:ty, $unit:expr) => {{
            let a = column.as_primitive::<$t>();
            PageValues::Text(
                rows.iter()
                    .map(|&r| {
                        valid(r)
                            .then(|| timestamp_text(a.value(idx(r)), $unit))
                            .flatten()
                    })
                    .collect(),
            )
        }};
    }
    match column.data_type() {
        DataType::Float64 => floats!(Float64Type),
        DataType::Float32 => floats!(Float32Type),
        DataType::Float16 => {
            let a = column.as_primitive::<Float16Type>();
            PageValues::F64(
                rows.iter()
                    .map(|&r| valid(r).then(|| f64::from(a.value(idx(r)))))
                    .collect(),
            )
        }
        DataType::Int8 => ints!(Int8Type),
        DataType::Int16 => ints!(Int16Type),
        DataType::Int32 => ints!(Int32Type),
        DataType::Int64 => ints!(Int64Type),
        DataType::UInt8 => ints!(UInt8Type),
        DataType::UInt16 => ints!(UInt16Type),
        DataType::UInt32 => ints!(UInt32Type),
        DataType::UInt64 => ints!(UInt64Type),
        DataType::Boolean => {
            let a = column.as_boolean();
            PageValues::Bool(
                rows.iter()
                    .map(|&r| valid(r).then(|| a.value(idx(r))))
                    .collect(),
            )
        }
        DataType::Date32 => {
            let a = column.as_primitive::<Date32Type>();
            PageValues::Text(
                rows.iter()
                    .map(|&r| {
                        valid(r)
                            .then(|| date_text(i64::from(a.value(idx(r)))))
                            .flatten()
                    })
                    .collect(),
            )
        }
        DataType::Date64 => timestamps!(Date64Type, TimeUnit::Millisecond),
        DataType::Timestamp(TimeUnit::Second, _) => {
            timestamps!(TimestampSecondType, TimeUnit::Second)
        }
        DataType::Timestamp(TimeUnit::Millisecond, _) => {
            timestamps!(TimestampMillisecondType, TimeUnit::Millisecond)
        }
        DataType::Timestamp(TimeUnit::Microsecond, _) => {
            timestamps!(TimestampMicrosecondType, TimeUnit::Microsecond)
        }
        DataType::Timestamp(TimeUnit::Nanosecond, _) => {
            timestamps!(TimestampNanosecondType, TimeUnit::Nanosecond)
        }
        _ => text(column, rows),
    }
}

/// Text columns, dictionaries of text (provenance) and anything else, as text.
fn text(column: &ArrayRef, rows: &[u64]) -> PageValues {
    let idx = |r: u64| usize::try_from(r).unwrap_or(usize::MAX);
    let cell = |r: u64| -> Option<String> {
        let i = idx(r);
        if i >= column.len() || column.is_null(i) {
            return None;
        }
        match column.data_type() {
            DataType::Utf8 => Some(column.as_string::<i32>().value(i).to_owned()),
            DataType::LargeUtf8 => Some(column.as_string::<i64>().value(i).to_owned()),
            DataType::Utf8View => Some(column.as_string_view().value(i).to_owned()),
            DataType::Dictionary(_, _) => {
                let dict = column.as_any_dictionary();
                let key = dict.normalized_keys().get(i).copied()?;
                let values = dict.values();
                match values.data_type() {
                    DataType::Utf8 => Some(values.as_string::<i32>().value(key).to_owned()),
                    DataType::LargeUtf8 => Some(values.as_string::<i64>().value(key).to_owned()),
                    _ => None,
                }
            }
            _ => numbers(column).and_then(|n| n.is_valid(i).then(|| n.value(i).to_string())),
        }
    };
    PageValues::Text(rows.iter().map(|&r| cell(r)).collect())
}

/// `days` since 1970-01-01 as `YYYY-MM-DD`.
fn date_text(days: i64) -> Option<String> {
    let epoch = jiff::civil::date(1970, 1, 1);
    let span = jiff::Span::new().try_days(days).ok()?;
    epoch.checked_add(span).ok().map(|d| d.to_string())
}

/// A timestamp in `unit` since the epoch as RFC 3339 UTC text.
fn timestamp_text(value: i64, unit: TimeUnit) -> Option<String> {
    let nanos: i128 = match unit {
        TimeUnit::Second => i128::from(value) * 1_000_000_000,
        TimeUnit::Millisecond => i128::from(value) * 1_000_000,
        TimeUnit::Microsecond => i128::from(value) * 1_000,
        TimeUnit::Nanosecond => i128::from(value),
    };
    jiff::Timestamp::from_nanosecond(nanos)
        .ok()
        .map(|t| t.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{
        BooleanArray, Date32Array, Int64Array, RecordBatch, StringArray, TimestampMillisecondArray,
    };
    use arrow_schema::{Field, Schema};
    use sonarithm_core::TableId;

    use super::*;

    fn table() -> Table {
        let schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Float64, true),
            Field::new("n", DataType::Int64, true),
            Field::new("name", DataType::Utf8, true),
            Field::new("ok", DataType::Boolean, true),
            Field::new("day", DataType::Date32, true),
            Field::new("at", DataType::Timestamp(TimeUnit::Millisecond, None), true),
        ]));
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Float64Array::from(vec![Some(3.0), None, Some(1.0)])),
                Arc::new(Int64Array::from(vec![Some(30), Some(10), Some(20)])),
                Arc::new(StringArray::from(vec![Some("c"), Some("a"), None])),
                Arc::new(BooleanArray::from(vec![Some(true), Some(false), None])),
                Arc::new(Date32Array::from(vec![Some(0), Some(19_000), None])),
                Arc::new(TimestampMillisecondArray::from(vec![
                    Some(1_500),
                    None,
                    Some(0),
                ])),
            ],
        )
        .unwrap();
        Table::from_batch(TableId(1), "t", batch)
    }

    #[test]
    fn a_page_converts_every_type_and_keeps_row_indices() {
        let page = table_page(&table(), 1, 5, None).unwrap();
        assert_eq!(page.total_rows, 3);
        assert_eq!(page.rows, [1, 2]);
        assert_eq!(page.columns[0].1, PageValues::F64(vec![None, Some(1.0)]));
        assert_eq!(page.columns[1].1, PageValues::I64(vec![Some(10), Some(20)]));
        assert_eq!(
            page.columns[2].1,
            PageValues::Text(vec![Some("a".into()), None])
        );
        assert_eq!(page.columns[3].1, PageValues::Bool(vec![Some(false), None]));
        assert_eq!(
            page.columns[4].1,
            PageValues::Text(vec![Some("2022-01-08".into()), None])
        );
        assert_eq!(
            page.columns[5].1,
            PageValues::Text(vec![None, Some("1970-01-01T00:00:00Z".into())])
        );
    }

    #[test]
    fn sorted_pages_put_missing_values_last() {
        let sort = PageSort {
            field: "x".into(),
            descending: false,
        };
        assert_eq!(
            table_page(&table(), 0, 10, Some(&sort)).unwrap().rows,
            [2, 0, 1]
        );
        let sort = PageSort {
            field: "n".into(),
            descending: true,
        };
        assert_eq!(table_page(&table(), 1, 1, Some(&sort)).unwrap().rows, [2]);
        assert!(table_page(&table(), 0, MAX_PAGE_ROWS + 1, None).is_err());
        assert!(axis_values(&table(), "name").is_err());
    }
}
