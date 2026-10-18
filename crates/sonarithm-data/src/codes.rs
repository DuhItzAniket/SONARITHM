// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `SNR-DATA` diagnostic codes (coding-standards §4.2).
//!
//! Codes are grouped by tens: 001–009 sources and limits; 010–019 delimited text; 020–029
//! JSON and spreadsheets; 030–039 inference; 040–049 metadata and provenance; 050–059
//! transforms. A released code is never reused.

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

macro_rules! code {
    ($(#[$doc:meta])* $name:ident, $code:literal, $severity:ident, $summary:literal, $help:literal, [$($rule:literal),*]) => {
        $(#[$doc])*
        pub const $name: CodeInfo = CodeInfo {
            code: $code,
            severity: Severity::$severity,
            summary: $summary,
            help: $help,
            rules: &[$($rule),*],
            status: CodeStatus::Active,
        };
    };
}

code!(
    /// The input is not valid data in the expected format, or a file cannot be read.
    UNREADABLE_SOURCE, "SNR-DATA-001", Error,
    "The data source cannot be read",
    "Check that the file exists, is complete and is in the format `format` names (or its extension implies).",
    []
);

code!(
    /// This build cannot read the format, or the extension names no supported format.
    UNSUPPORTED_FORMAT, "SNR-DATA-002", Error,
    "The data format is not supported",
    "Set `format` to one of csv, tsv, json, ndjson, parquet, arrow, xlsx or ods, or convert the file.",
    []
);

code!(
    /// The input breaks a size limit.
    LIMIT_EXCEEDED, "SNR-DATA-003", Error,
    "The data source exceeds a size limit",
    "Filter or split the data, or raise the source limits for trusted input.",
    []
);

code!(
    /// A table's columns do not match its schema, or two tables share an ID.
    MALFORMED_TABLE, "SNR-DATA-004", Error,
    "A table is malformed",
    "Build each table from equal-length columns that match its schema, and give each table in a bundle its own ID.",
    []
);

code!(
    /// A table cannot be encoded as Arrow IPC or Parquet.
    WRITE_FAILED, "SNR-DATA-005", Error,
    "A table cannot be written",
    "Report the problem with the table's schema; every table the data layer builds can be written.",
    []
);

code!(
    /// A source option has a value that cannot be used.
    INVALID_OPTION, "SNR-DATA-006", Error,
    "A source option is invalid",
    "Fix the option in `data.source.options`; the message says what a valid value looks like.",
    []
);

code!(
    /// The CSV dialect that was detected.
    CSV_DIALECT, "SNR-DATA-010", Info,
    "The delimiter and header row were detected",
    "Set `delimiter` and `header` in the source options if the guess is wrong.",
    []
);

code!(
    /// Text was converted to UTF-8 from another encoding.
    ENCODING_CONVERTED, "SNR-DATA-011", Info,
    "The text encoding was detected and converted",
    "Set `encoding` (a WHATWG label such as windows-1252) if the letters look wrong.",
    []
);

code!(
    /// Rows have a different number of fields than the header.
    RAGGED_ROWS, "SNR-DATA-012", Warning,
    "Some rows have the wrong number of fields",
    "Check the delimiter and the quoting of values that contain it.",
    []
);

code!(
    /// Blank or repeated field names were replaced.
    COLUMN_NAMES_CHANGED, "SNR-DATA-013", Info,
    "Blank or repeated field names were replaced",
    "Give every column a unique name in the header row.",
    []
);

code!(
    /// Bytes that are not valid in the encoding were replaced.
    INVALID_BYTES, "SNR-DATA-014", Warning,
    "Invalid bytes were replaced",
    "Check the file's encoding and set `encoding` if it was guessed wrong.",
    []
);

code!(
    /// Lines above the header were skipped.
    PREAMBLE_SKIPPED, "SNR-DATA-015", Info,
    "Lines above the header were skipped",
    "Remove title lines above the header, or set `header` explicitly.",
    []
);

code!(
    /// The JSON shape that was read.
    JSON_SHAPE, "SNR-DATA-020", Info,
    "The JSON shape was detected",
    "Set `pointer` in the source options if the wrong part of the document was read.",
    []
);

code!(
    /// The JSON at the pointer is not a table.
    NOT_TABULAR, "SNR-DATA-021", Error,
    "The JSON is not a table",
    "Point `pointer` at an array of records, an array of rows or an object of equal-length arrays.",
    []
);

code!(
    /// The sheet and range that were read.
    SHEET_READ, "SNR-DATA-022", Info,
    "A spreadsheet sheet was read",
    "Set `sheet` and `range` in the source if the wrong cells were read.",
    []
);

code!(
    /// The workbook has no such sheet.
    SHEET_NOT_FOUND, "SNR-DATA-023", Error,
    "The sheet does not exist",
    "Name an existing sheet, or give its position counted from 0.",
    []
);

code!(
    /// Spreadsheet error values became missing values.
    CELL_ERRORS, "SNR-DATA-024", Warning,
    "Spreadsheet error values became missing values",
    "Fix the formulas that produce errors such as #DIV/0! in the source workbook.",
    []
);

code!(
    /// A field type was inferred.
    TYPE_INFERRED, "SNR-DATA-030", Info,
    "A field type was inferred",
    "Declare the field's `type` in `data.fields` if the inference is wrong (03 §3.1).",
    []
);

code!(
    /// A field role was inferred.
    ROLE_INFERRED, "SNR-DATA-031", Info,
    "A field role was inferred",
    "Declare the field's `role` in `data.fields` if the inference is wrong (03 §3.4).",
    []
);

code!(
    /// A field concept was inferred.
    CONCEPT_INFERRED, "SNR-DATA-032", Info,
    "A field concept was inferred",
    "Declare the field's `concept` in `data.fields` if the inference is wrong; the concept sets the default polarity (03 §3.3).",
    ["SDR-12"]
);

code!(
    /// NaN values became missing values.
    NAN_TO_NULL, "SNR-DATA-033", Info,
    "NaN values became missing values",
    "None needed: NaN is treated as missing (03 §5). Missing values are silent by default.",
    []
);

code!(
    /// Infinite values were kept.
    INFINITE_VALUES, "SNR-DATA-034", Warning,
    "Infinite values were kept",
    "Check whether the infinities are real; they sound at the range edge with an off-scale cue (03 §5).",
    ["A11Y-28"]
);

code!(
    /// Values that do not fit the column type became missing values.
    UNPARSED_VALUES, "SNR-DATA-035", Warning,
    "Some values do not fit the field type and became missing",
    "Fix the values the message points to, or declare the field's type in `data.fields`.",
    []
);

code!(
    /// Numeric dates could be day-first or month-first.
    AMBIGUOUS_DATES, "SNR-DATA-036", Warning,
    "Dates could be day-first or month-first, so they stayed text",
    "Set `dateFormat` in the source options, for example `%d/%m/%Y`.",
    []
);

code!(
    /// A unit was read from values or a field name.
    UNIT_FROM_TEXT, "SNR-DATA-037", Info,
    "A unit was read from the values or the field name",
    "Declare the unit in `data.fields` if it is wrong.",
    []
);

code!(
    /// A `snr.*` field metadata value cannot be read and was ignored.
    INVALID_METADATA, "SNR-DATA-040", Warning,
    "A field metadata value cannot be read and was ignored",
    "Fix the `snr.*` value in the file (03 §3 lists the allowed values) or declare the field in the spec's `data.fields`.",
    []
);

code!(
    /// A provenance column has an unsupported type or an invalid value.
    INVALID_PROVENANCE, "SNR-DATA-041", Error,
    "The provenance columns are invalid",
    "Write `__snr_prov` as class names (exact, vector, cv_estimate, ml_estimate, cloud_estimate, user_edited, derived, synthetic) and `__snr_conf` as numbers from 0 to 1 (03 §4).",
    []
);

code!(
    /// A field name the table does not have.
    UNKNOWN_FIELD, "SNR-DATA-050", Error,
    "Unknown field",
    "Name a field of the table; field names are case-sensitive.",
    []
);

code!(
    /// A transform needs another kind of field.
    FIELD_TYPE_MISMATCH, "SNR-DATA-051", Error,
    "The transform needs another kind of field",
    "Pick a field of the kind the message names, or convert one with `derive`.",
    []
);

code!(
    /// An expression does not compile or gives values of different types.
    INVALID_EXPRESSION, "SNR-DATA-052", Error,
    "The expression is invalid",
    "Write one rhai expression over the row's fields; statements, loops, functions and `**` are not available (03 §6).",
    []
);

code!(
    /// An expression failed on some rows; those values are missing.
    EXPRESSION_FAILED, "SNR-DATA-053", Warning,
    "The expression failed on some rows",
    "Check the rows the message names, for example for division by zero; their values are missing.",
    []
);

code!(
    /// An expression needs more operations per row than allowed.
    EXPRESSION_BUDGET, "SNR-DATA-054", Error,
    "The expression needs too many operations",
    "Simplify the expression; each row may use at most 10 000 operations.",
    []
);

code!(
    /// Transform parameters that cannot be used.
    INVALID_TRANSFORM, "SNR-DATA-055", Error,
    "Invalid transform parameters",
    "Fix the parameters the message names (RFC-0001 §4).",
    []
);

code!(
    /// A transform ran.
    TRANSFORM_APPLIED, "SNR-DATA-056", Info,
    "A transform ran",
    "None needed; the message states the rows in and out.",
    []
);

/// Every `SNR-DATA` code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Data,
    owner: "sonarithm-data",
    codes: &[
        UNREADABLE_SOURCE,
        UNSUPPORTED_FORMAT,
        LIMIT_EXCEEDED,
        MALFORMED_TABLE,
        WRITE_FAILED,
        INVALID_OPTION,
        CSV_DIALECT,
        ENCODING_CONVERTED,
        RAGGED_ROWS,
        COLUMN_NAMES_CHANGED,
        INVALID_BYTES,
        PREAMBLE_SKIPPED,
        JSON_SHAPE,
        NOT_TABULAR,
        SHEET_READ,
        SHEET_NOT_FOUND,
        CELL_ERRORS,
        TYPE_INFERRED,
        ROLE_INFERRED,
        CONCEPT_INFERRED,
        NAN_TO_NULL,
        INFINITE_VALUES,
        UNPARSED_VALUES,
        AMBIGUOUS_DATES,
        UNIT_FROM_TEXT,
        INVALID_METADATA,
        INVALID_PROVENANCE,
        UNKNOWN_FIELD,
        FIELD_TYPE_MISMATCH,
        INVALID_EXPRESSION,
        EXPRESSION_FAILED,
        EXPRESSION_BUDGET,
        INVALID_TRANSFORM,
        TRANSFORM_APPLIED,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-DATA code table");
