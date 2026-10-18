// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Building tables from text columns, shared by the CSV, JSON and spreadsheet sources:
//! storage-type inference per column ([`crate::infer::text`]), units, unique field names and
//! the diagnostics that report what inference changed.

use std::sync::Arc;

use arrow_schema::{Field, Schema};
use sonarithm_core::{Diagnostic, Diagnostics, Generation, RowRange};

use super::ReadOptions;
use crate::codes;
use crate::error::DataError;
use crate::infer::text::{self, ColumnReport, Storage, TextOptions};
use crate::meta::{FieldMeta, Unit};
use crate::table::Table;

/// Values per column used to decide whether the first row is a header.
const HEADER_SAMPLE: usize = 100;

/// Values of one column, stored contiguously.
#[derive(Default)]
pub(super) struct RawColumn {
    text: String,
    ends: Vec<usize>,
}

impl RawColumn {
    pub(super) fn push(&mut self, value: &str) {
        self.text.push_str(value);
        self.ends.push(self.text.len());
    }

    /// Puts a value in front of the others.
    pub(super) fn prepend(&mut self, value: &str) {
        let mut with_first = Self::default();
        with_first.push(value);
        for v in self.values() {
            with_first.push(v);
        }
        *self = with_first;
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &str> + Clone {
        std::iter::once(0)
            .chain(self.ends.iter().copied())
            .zip(self.ends.iter().copied())
            .map(|(start, end)| self.text.get(start..end).unwrap_or_default())
    }
}

/// A table from named text columns of equal length: each column gets its inferred type and
/// unit, and every change inference made is reported.
pub(super) fn text_table(
    name: &str,
    names: &[String],
    columns: &[RawColumn],
    text_options: &TextOptions,
    options: &ReadOptions,
    diagnostics: &mut Diagnostics,
) -> Result<Table, DataError> {
    let (fields, arrays) = typed_fields(names, columns, text_options, options, diagnostics);
    let schema = Arc::new(Schema::new(fields));
    let batch =
        arrow_array::RecordBatch::try_new(Arc::clone(&schema), arrays).map_err(|source| {
            DataError::MalformedTable {
                table: name.to_owned(),
                source,
            }
        })?;
    Table::new(options.table, name, schema, vec![batch])
}

/// Infers each column's type and unit, reporting what was changed or assumed.
fn typed_fields(
    names: &[String],
    columns: &[RawColumn],
    text_options: &TextOptions,
    options: &ReadOptions,
    diagnostics: &mut Diagnostics,
) -> (Vec<Field>, Vec<arrow_array::ArrayRef>) {
    let mut fields = Vec::with_capacity(names.len());
    let mut arrays = Vec::with_capacity(names.len());
    for (name, column) in names.iter().zip(columns) {
        let (array, storage, report) = text::infer_column(column.values(), text_options);
        let mut meta = FieldMeta::new();
        let (unit, origin) = match &report.unit {
            Some(unit) => (Some(unit.clone()), "values"),
            None => (unit_from_name(name), "name"),
        };
        if let Some(unit) = unit.and_then(Unit::new) {
            diagnostics.push(Diagnostic::new(
                &codes::UNIT_FROM_TEXT,
                format!(
                    "Field `{name}` was given the unit `{}` found in its {origin}.",
                    unit.text
                ),
            ));
            meta = meta.with_unit(unit);
        }
        report_column(name, storage, &report, options, diagnostics);
        fields.push(meta.apply(Field::new(name, array.data_type().clone(), true)));
        arrays.push(array);
    }
    (fields, arrays)
}

/// A one-row range in the table being built, for diagnostics.
pub(super) fn row_range(options: &ReadOptions, row: u64) -> RowRange {
    RowRange {
        table: options.table,
        generation: Generation(0),
        start: row,
        end: row + 1,
    }
}

/// Diagnostics for values inference changed or flagged in one column.
fn report_column(
    name: &str,
    storage: Storage,
    report: &ColumnReport,
    options: &ReadOptions,
    diagnostics: &mut Diagnostics,
) {
    if report.nan > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::NAN_TO_NULL,
            format!(
                "Field `{name}`: {} NaN value(s) became missing values (03 §5).",
                report.nan
            ),
        ));
    }
    if report.errors > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::CELL_ERRORS,
            format!(
                "Field `{name}`: {} spreadsheet error value(s) such as #DIV/0! became missing values. Fix the formulas in the source.",
                report.errors
            ),
        ));
    }
    if report.infinite > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::INFINITE_VALUES,
            format!(
                "Field `{name}`: {} infinite value(s) were kept; they sound at the edge of the range with an off-scale cue.",
                report.infinite
            ),
        ));
    }
    if let Some(first) = report.first_unparsed {
        let kind = match storage {
            Storage::Int | Storage::Float => "numbers",
            Storage::Date | Storage::Timestamp { .. } => "dates",
            Storage::Bool | Storage::Text => "values",
        };
        diagnostics.push(
            Diagnostic::new(
                &codes::UNPARSED_VALUES,
                format!(
                    "Field `{name}` holds {kind}, but {} value(s) could not be read as {kind} and became missing, first at row {}. Fix those values or declare the field's type.",
                    report.unparsed,
                    first + 1
                ),
            )
            .rows(row_range(options, first)),
        );
    }
    if report.ambiguous_dates {
        diagnostics.push(Diagnostic::new(
            &codes::AMBIGUOUS_DATES,
            format!(
                "Field `{name}` holds dates that could be day-first or month-first, so it was kept as text. Set `dateFormat`, for example `%d/%m/%Y`."
            ),
        ));
    }
}

/// Whether the first record is a header: some typed column's first value does not fit the
/// column's type. With only text columns, a row of distinct non-empty names is a header.
pub(super) fn detect_header(first: &[&str], columns: &[RawColumn], options: &TextOptions) -> bool {
    let mut header_votes = 0;
    let mut data_votes = 0;
    for (value, column) in first.iter().zip(columns) {
        let sample: Vec<&str> = column.values().take(HEADER_SAMPLE).collect();
        let storage = text::storage_of(sample.iter().copied(), options);
        if storage == Storage::Text {
            continue;
        }
        if text::fits(value, storage, options) && !value.trim().is_empty() {
            data_votes += 1;
        } else {
            header_votes += 1;
        }
    }
    if header_votes > 0 {
        return true;
    }
    if data_votes > 0 {
        return false;
    }
    let mut names: Vec<&str> = first.iter().map(|v| v.trim()).collect();
    let all_named = names.iter().all(|name| !name.is_empty());
    names.sort_unstable();
    all_named && names.windows(2).all(|pair| pair[0] != pair[1])
}

/// Field names: the given names with blanks replaced by `column_N` and repeats suffixed
/// `_2`, `_3`…; or `column_1`… when no names are given. Returns the names and how many
/// given names were replaced.
pub(super) fn unique_names(header: Option<&[Option<&str>]>, width: usize) -> (Vec<String>, usize) {
    let mut names: Vec<String> = Vec::with_capacity(width);
    let mut renamed = 0;
    for index in 0..width {
        let given = header
            .and_then(|h| h.get(index).copied().flatten())
            .map_or("", str::trim);
        let mut name = if given.is_empty() {
            if header.is_some() {
                renamed += 1;
            }
            format!("column_{}", index + 1)
        } else {
            given.to_owned()
        };
        if names.contains(&name) {
            renamed += 1;
            let mut suffix = 2;
            while names.contains(&format!("{name}_{suffix}")) {
                suffix += 1;
            }
            name = format!("{name}_{suffix}");
        }
        names.push(name);
    }
    (names, renamed)
}

/// A unit written in brackets at the end of a field name: `temp (°C)`, `price [USD]`.
pub(super) fn unit_from_name(name: &str) -> Option<String> {
    let name = name.trim_end();
    let (open, close) = match name.chars().next_back()? {
        ')' => ('(', ')'),
        ']' => ('[', ']'),
        _ => return None,
    };
    let start = name.rfind(open)?;
    let unit = name.get(start + 1..name.len() - close.len_utf8())?.trim();
    let prefix = name.get(..start)?.trim();
    (!prefix.is_empty() && !unit.is_empty() && unit.chars().count() <= 12).then(|| unit.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_made_unique_and_blanks_numbered() {
        let given = [Some("x"), None, Some(" x "), Some(""), Some("x_2")];
        let (names, renamed) = unique_names(Some(&given), 5);
        assert_eq!(names, ["x", "column_2", "x_2", "column_4", "x_2_2"]);
        assert_eq!(renamed, 4);
        assert_eq!(
            unique_names(None, 2),
            (vec!["column_1".to_owned(), "column_2".to_owned()], 0)
        );
    }

    #[test]
    fn unit_names_need_a_prefix() {
        assert_eq!(unit_from_name("distance [km]").as_deref(), Some("km"));
        assert_eq!(unit_from_name("(note)"), None);
        assert_eq!(unit_from_name("plain"), None);
    }
}
