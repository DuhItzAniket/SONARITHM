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
    /// A `snr.*` field metadata value cannot be read and was ignored.
    INVALID_METADATA, "SNR-DATA-040", Warning,
    "A field metadata value cannot be read and was ignored",
    "Fix the `snr.*` value in the file (03 §3 lists the allowed values) or declare the field in the spec's `data.fields`.",
    []
);

code!(
    /// A field name the table does not have.
    UNKNOWN_FIELD, "SNR-DATA-050", Error,
    "Unknown field",
    "Name a field of the table; field names are case-sensitive.",
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
        INVALID_METADATA,
        UNKNOWN_FIELD,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-DATA code table");
