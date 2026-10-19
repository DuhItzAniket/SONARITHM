// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! XLSX, XLSM, XLSB, XLS and ODS sources through calamine (10 §2), with sheet and range
//! selection. Formulas give their cached values.
//!
//! Cells become text in a lossless form (integral numbers as integers, other numbers in
//! their shortest round-trip form, dates as ISO-8601) and go through the same type
//! inference as CSV, so a column that mixes numbers with a stray `n/a` is handled the same
//! way in every format.

use std::io::Cursor;

use calamine::{Data, Reader};
use serde::Deserialize;
use sonarithm_core::{Diagnostic, Diagnostics};

use super::ReadOptions;
use super::columns::{RawColumn, detect_header, text_table, unique_names};
use crate::bundle::Format;
use crate::codes;
use crate::error::DataError;
use crate::infer::text::TextOptions;
use crate::table::Table;

/// The largest integer every `f64` below it represents exactly (2^53).
const EXACT_INTEGER: f64 = 9_007_199_254_740_992.0;

/// Which sheet to read: by name, or by position from 0.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum SheetRef {
    /// The sheet with this name.
    Name(String),
    /// The sheet at this position, from 0.
    Index(usize),
}

/// Options for spreadsheet sources: the spec's `data.source.sheet` plus
/// `data.source.options`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct SheetOptions {
    /// The sheet; by default the first sheet that holds data.
    pub sheet: Option<SheetRef>,
    /// A cell range in A1 notation, such as `B2:F200`; by default the used range.
    pub range: Option<String>,
    /// Whether the first row names the fields; detected when `None`.
    pub header: Option<bool>,
    /// A `strftime`-style format for date text that is not ISO-8601.
    pub date_format: Option<String>,
}

impl SheetOptions {
    /// The default options: the first sheet with data, its used range, header detected.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same options reading another sheet.
    #[must_use]
    pub fn with_sheet(mut self, sheet: SheetRef) -> Self {
        self.sheet = Some(sheet);
        self
    }

    /// The same options reading a cell range in A1 notation.
    #[must_use]
    pub fn with_range(mut self, range: impl Into<String>) -> Self {
        self.range = Some(range.into());
        self
    }

    /// The same options with the header row declared instead of detected.
    #[must_use]
    pub fn with_header(mut self, header: bool) -> Self {
        self.header = Some(header);
        self
    }
}

/// Reads one sheet of a workbook into a table.
pub(super) fn read_sheet(
    bytes: &[u8],
    format: Format,
    options: &ReadOptions,
) -> Result<(Table, Diagnostics), DataError> {
    let sheets = &options.sheet;
    let window = match &sheets.range {
        Some(a1) => Some(parse_range(a1).ok_or(DataError::InvalidOption {
            option: "range",
            reason: "use A1 notation such as B2:F200",
        })?),
        None => None,
    };
    let (sheet, names, cells) =
        super::contained(format, || select(bytes, format, sheets, window, options))?;
    let (first, columns) = dense(&cells, options)?;
    let Some(first) = first else {
        let schema = std::sync::Arc::new(arrow_schema::Schema::empty());
        return Ok((
            Table::new(options.table, &sheet, schema, Vec::new())?,
            Diagnostics::new(),
        ));
    };
    let width = columns.len();
    let mut columns = columns;
    let text_options = TextOptions {
        decimal_comma: false,
        date_format: sheets.date_format.clone(),
    };
    let first_refs: Vec<&str> = first.iter().map(String::as_str).collect();
    let header = sheets
        .header
        .unwrap_or_else(|| detect_header(&first_refs, &columns, &text_options));
    if !header {
        for (column, value) in columns.iter_mut().zip(&first_refs) {
            column.prepend(value);
        }
    }
    let given: Vec<Option<&str>> = first_refs.iter().map(|name| Some(*name)).collect();
    let (field_names, renamed) = unique_names(header.then_some(given.as_slice()), width);
    let mut diagnostics = Diagnostics::new();
    let table = text_table(
        &sheet,
        &field_names,
        &columns,
        &text_options,
        options,
        &mut diagnostics,
    )?;
    diagnostics.push(Diagnostic::new(
        &codes::SHEET_READ,
        format!(
            "Read sheet `{sheet}` ({} of {}){}: {}, {} fields, {} rows.",
            names.iter().position(|n| *n == sheet).map_or(0, |i| i + 1),
            names.len(),
            sheets
                .range
                .as_ref()
                .map(|r| format!(", range {r}"))
                .unwrap_or_default(),
            if header {
                "first row is the header"
            } else {
                "no header row"
            },
            table.num_fields(),
            table.num_rows(),
        ),
    ));
    if renamed > 0 {
        diagnostics.push(Diagnostic::new(
            &codes::COLUMN_NAMES_CHANGED,
            format!("{renamed} field name(s) were blank or repeated and were replaced (`column_N`, `name_2`)."),
        ));
    }
    Ok((table, diagnostics))
}

/// A non-empty cell: zero-based row and column, and its text.
type Cell = (u32, u32, String);

/// A1 corners, zero-based and inclusive.
type Window = ((u32, u32), (u32, u32));

/// Opens the workbook and returns the chosen sheet's name, all sheet names and the
/// non-empty cells of the chosen range.
fn select(
    bytes: &[u8],
    format: Format,
    sheets: &SheetOptions,
    window: Option<Window>,
    options: &ReadOptions,
) -> Result<(String, Vec<String>, Vec<Cell>), DataError> {
    let unreadable = |reason: String| DataError::Unreadable { format, reason };
    let mut workbook = calamine::open_workbook_auto_from_rs(Cursor::new(bytes))
        .map_err(|err| unreadable(err.to_string()))?;
    let names = workbook.sheet_names();
    if let Some(wanted) = &sheets.sheet {
        let name = match wanted {
            SheetRef::Name(name) => names.iter().find(|n| *n == name),
            SheetRef::Index(index) => names.get(*index),
        }
        .cloned()
        .ok_or_else(|| DataError::SheetNotFound {
            sheet: match wanted {
                SheetRef::Name(name) => name.clone(),
                SheetRef::Index(index) => format!("#{index}"),
            },
            available: names.len(),
        })?;
        let cells = sheet_cells(&mut workbook, &name, window, options, format)?;
        return Ok((name, names, cells));
    }
    // The first sheet with a non-empty cell, or the first sheet.
    for name in &names {
        let cells = sheet_cells(&mut workbook, name, window, options, format)?;
        if !cells.is_empty() {
            return Ok((name.clone(), names, cells));
        }
    }
    let first = names.first().cloned().unwrap_or_default();
    Ok((first, names, Vec::new()))
}

/// The non-empty cells of one sheet inside `window`. XLSX is streamed cell by cell, so a
/// hostile workbook cannot make the reader allocate its whole bounding box; the cell count
/// is capped by the limits as cells arrive. Other formats go through calamine's ranges.
fn sheet_cells<RS: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<RS>,
    name: &str,
    window: Option<Window>,
    options: &ReadOptions,
    format: Format,
) -> Result<Vec<Cell>, DataError> {
    let unreadable = |reason: String| DataError::Unreadable { format, reason };
    let inside = |row: u32, column: u32| {
        window.is_none_or(|((r0, c0), (r1, c1))| {
            (r0..=r1).contains(&row) && (c0..=c1).contains(&column)
        })
    };
    let max_cells = options.limits.max_cells;
    let mut cells: Vec<Cell> = Vec::new();
    let mut keep = |row: u32, column: u32, data: &Data| -> Result<(), DataError> {
        if *data == Data::Empty || !inside(row, column) {
            return Ok(());
        }
        cells.push((row, column, cell_text(data)));
        let found = super::to_u64(cells.len());
        if found > max_cells {
            return Err(DataError::LimitExceeded {
                limit: crate::error::Limit::Cells,
                max: max_cells,
                found,
            });
        }
        Ok(())
    };
    if let calamine::Sheets::Xlsx(xlsx) = workbook {
        let mut reader = xlsx
            .worksheet_cells_reader(name)
            .map_err(|err| unreadable(err.to_string()))?;
        while let Some(cell) = reader
            .next_cell()
            .map_err(|err| unreadable(err.to_string()))?
        {
            let (row, column) = cell.get_position();
            keep(row, column, &Data::from(cell.get_value().clone()))?;
        }
    } else {
        let range = workbook
            .worksheet_range(name)
            .map_err(|err| unreadable(err.to_string()))?;
        let (row0, column0) = range.start().unwrap_or((0, 0));
        for (row, column, data) in range.used_cells() {
            let (Ok(row), Ok(column)) = (u32::try_from(row), u32::try_from(column)) else {
                continue;
            };
            keep(
                row0.saturating_add(row),
                column0.saturating_add(column),
                data,
            )?;
        }
    }
    Ok(cells)
}

/// The cells' bounding box as dense text columns: the first row separately (a header
/// candidate) and the rest as columns. The box is checked against the limits before it
/// is allocated.
fn dense(
    cells: &[Cell],
    options: &ReadOptions,
) -> Result<(Option<Vec<String>>, Vec<RawColumn>), DataError> {
    let (Some(r0), Some(r1)) = (
        cells.iter().map(|c| c.0).min(),
        cells.iter().map(|c| c.0).max(),
    ) else {
        return Ok((None, Vec::new()));
    };
    let (c0, c1) = (
        cells.iter().map(|c| c.1).min().unwrap_or(0),
        cells.iter().map(|c| c.1).max().unwrap_or(0),
    );
    let height = u64::from(r1 - r0) + 1;
    let width = usize::try_from(c1 - c0)
        .unwrap_or(usize::MAX)
        .saturating_add(1);
    options.limits.check_shape(height, width)?;
    let mut sorted: Vec<&Cell> = cells.iter().collect();
    sorted.sort_by_key(|c| (c.0, c.1));
    let mut first = vec![String::new(); width];
    let mut columns: Vec<RawColumn> = (0..width).map(|_| RawColumn::default()).collect();
    let mut next = sorted.iter().peekable();
    for row in r0..=r1 {
        for (offset, column) in (c0..=c1).enumerate() {
            let text = match next.peek() {
                Some(cell) if (cell.0, cell.1) == (row, column) => {
                    next.next().map_or("", |c| c.2.as_str())
                }
                _ => "",
            };
            if row == r0 {
                if let Some(slot) = first.get_mut(offset) {
                    text.clone_into(slot);
                }
            } else if let Some(raw) = columns.get_mut(offset) {
                raw.push(text);
            }
        }
    }
    Ok((Some(first), columns))
}

/// A cell as the text that type inference reads.
fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::Int(int) => int.to_string(),
        Data::Float(float) => float_text(*float),
        Data::String(text) | Data::DateTimeIso(text) | Data::DurationIso(text) => text.clone(),
        Data::Bool(value) => value.to_string(),
        Data::Error(error) => error.to_string(),
        Data::DateTime(datetime) if datetime.is_duration() => {
            // A duration: seconds, as a plain number.
            float_text(datetime.as_f64() * 86_400.0)
        }
        Data::DateTime(datetime) => {
            let (year, month, day, hour, minute, second, milli) = datetime.to_ymd_hms_milli();
            if (hour, minute, second, milli) == (0, 0, 0, 0) {
                format!("{year:04}-{month:02}-{day:02}")
            } else {
                format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milli:03}"
                )
            }
        }
    }
}

/// Integral values in the exact range as integers (spreadsheets store every number as a
/// float), others in their shortest round-trip form.
fn float_text(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < EXACT_INTEGER {
        // Integral and below 2^53 in magnitude, so the cast is exact.
        #[expect(clippy::cast_possible_truncation, reason = "integral and below 2^53")]
        let int = value as i64;
        int.to_string()
    } else {
        format!("{value:?}")
    }
}

/// A1-notation range (`B2:F200`, or one cell) as zero-based (row, column) corners.
fn parse_range(a1: &str) -> Option<((u32, u32), (u32, u32))> {
    let (start, end) = a1.split_once(':').unwrap_or((a1, a1));
    let start = parse_cell(start.trim())?;
    let end = parse_cell(end.trim())?;
    (start.0 <= end.0 && start.1 <= end.1).then_some((start, end))
}

fn parse_cell(cell: &str) -> Option<(u32, u32)> {
    let cell = cell.replace('$', "");
    let split = cell.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = cell.split_at(split);
    if letters.is_empty() || letters.len() > 3 || !letters.chars().all(|c| c.is_ascii_alphabetic())
    {
        return None;
    }
    let column = letters.chars().try_fold(0_u32, |acc, c| {
        acc.checked_mul(26)?
            .checked_add(u32::from(c.to_ascii_uppercase()) - u32::from('A') + 1)
    })?;
    let row: u32 = digits.parse().ok()?;
    Some((row.checked_sub(1)?, column - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a1_ranges_parse_to_zero_based_corners() {
        assert_eq!(parse_range("B2:F200"), Some(((1, 1), (199, 5))));
        assert_eq!(parse_range("$A$1"), Some(((0, 0), (0, 0))));
        assert_eq!(parse_range("aa10:AB12"), Some(((9, 26), (11, 27))));
        for bad in ["", "A0", "1A", "B2:A1", "ABCD1", "A1:B"] {
            assert_eq!(parse_range(bad), None, "{bad}");
        }
    }

    #[test]
    fn floats_render_losslessly() {
        assert_eq!(float_text(3.0), "3");
        assert_eq!(float_text(-0.0), "0");
        assert_eq!(float_text(0.1), "0.1");
        assert_eq!(float_text(1e300), "1e300");
        assert_eq!(float_text(f64::INFINITY), "inf");
        assert_eq!(float_text(EXACT_INTEGER), "9007199254740992.0");
    }
}
