// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The error type of the data layer (coding-standards §4.1).
//!
//! A [`DataError`] stops the operation that raised it. Problems that do not, such as an
//! inferred type or an ignored metadata value, are [`Diagnostic`]s instead. Every error
//! converts to a diagnostic with a stable code through [`DataError::to_diagnostic`].

use std::fmt;

use arrow_schema::ArrowError;
use sonarithm_core::Diagnostic;

use crate::bundle::Format;
use crate::codes;

/// A size limit of a source ([`Limits`](crate::source::Limits)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Limit {
    /// Input size in bytes.
    Bytes,
    /// Rows.
    Rows,
    /// Fields (columns).
    Fields,
    /// Cells (rows × fields).
    Cells,
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Bytes => "bytes",
            Self::Rows => "rows",
            Self::Fields => "fields",
            Self::Cells => "cells",
        })
    }
}

/// An error from the data layer.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DataError {
    /// The input is not valid data in the expected format.
    #[error("the {format} data cannot be read: {reason}")]
    Unreadable {
        /// The format the input was read as.
        format: Format,
        /// What the parser reported.
        reason: String,
    },
    /// This build has no reader for the format (a disabled feature).
    #[error("this build cannot read {format} data")]
    UnsupportedFormat {
        /// The format.
        format: Format,
    },
    /// A file extension that names no supported format.
    #[error("the format of `{locator}` cannot be told from its extension")]
    UnknownFormat {
        /// The file name.
        locator: String,
    },
    /// The input breaks a size limit.
    #[error("the data has {found} {limit}, more than the limit of {max}")]
    LimitExceeded {
        /// Which limit.
        limit: Limit,
        /// The limit.
        max: u64,
        /// The size found (or reached when reading stopped).
        found: u64,
    },
    /// A file cannot be opened or read.
    #[cfg(feature = "fs")]
    #[error("`{locator}` cannot be read: {source}")]
    Io {
        /// The file name.
        locator: String,
        /// The operating-system error.
        #[source]
        source: std::io::Error,
    },
    /// The JSON at the pointer is not a table.
    #[error("the JSON at `{pointer}` is {found}, not a table")]
    NotTabular {
        /// The JSON pointer (empty for the root).
        pointer: String,
        /// What was found instead.
        found: &'static str,
    },
    /// The workbook has no sheet with this name or position.
    #[error("the workbook has no sheet {sheet} ({available} sheets)")]
    SheetNotFound {
        /// The sheet asked for (a name, or `#n` for a position).
        sheet: String,
        /// How many sheets the workbook has.
        available: usize,
    },
    /// A provenance column has an unsupported type or an invalid value.
    #[error("invalid provenance: {reason}")]
    InvalidProvenance {
        /// What is wrong.
        reason: String,
    },
    /// A source option has a value that cannot be used.
    #[error("the `{option}` option is invalid: {reason}")]
    InvalidOption {
        /// The option name, as written in a spec.
        option: &'static str,
        /// What a valid value looks like.
        reason: &'static str,
    },
    /// A table cannot be encoded.
    #[error("the table cannot be written as {format}: {reason}")]
    Write {
        /// The output format.
        format: Format,
        /// What the encoder reported.
        reason: String,
    },
    /// A table's batches do not match its schema, or its columns differ in length.
    #[error("table `{table}` is malformed: {source}")]
    MalformedTable {
        /// The table name.
        table: String,
        /// What Arrow reported.
        #[source]
        source: ArrowError,
    },
    /// Two tables in one bundle share an ID.
    #[error("table ID {id} is used twice in one bundle")]
    DuplicateTable {
        /// The repeated ID.
        id: u32,
    },
    /// A field name that the table does not have.
    #[error("table `{table}` has no field `{field}`")]
    UnknownField {
        /// The table name.
        table: String,
        /// The missing field.
        field: String,
    },
}

impl DataError {
    /// The diagnostic that reports this error to the user.
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let (code, message) = match self {
            Self::Unreadable { format, reason } => (
                &codes::UNREADABLE_SOURCE,
                format!(
                    "The data cannot be read as {format}: {reason}. Check that the file is \
                     complete and that `format` names its real format."
                ),
            ),
            Self::UnsupportedFormat { format } => (
                &codes::UNSUPPORTED_FORMAT,
                format!(
                    "This build of SONARITHM cannot read {format} data. Convert the file to \
                     CSV or Parquet, or use a build with the format's feature enabled."
                ),
            ),
            Self::UnknownFormat { locator } => (
                &codes::UNSUPPORTED_FORMAT,
                format!(
                    "The format of `{locator}` cannot be told from its extension. Set \
                     `format` in the data source."
                ),
            ),
            Self::LimitExceeded { limit, max, found } => (
                &codes::LIMIT_EXCEEDED,
                format!(
                    "The data has {found} {limit}, more than the limit of {max}. Filter or \
                     split the data, or raise the limit for trusted input."
                ),
            ),
            #[cfg(feature = "fs")]
            Self::Io { locator, source } => (
                &codes::UNREADABLE_SOURCE,
                format!("`{locator}` cannot be read: {source}. Check the path and permissions."),
            ),
            Self::NotTabular { pointer, found } => (
                &codes::NOT_TABULAR,
                format!(
                    "The JSON at `{}` is {found}, not a table. Point `pointer` at an array of \
                     records, an array of rows or an object of equal-length arrays.",
                    if pointer.is_empty() { "/" } else { pointer }
                ),
            ),
            Self::SheetNotFound { sheet, available } => (
                &codes::SHEET_NOT_FOUND,
                format!(
                    "The workbook has no sheet `{sheet}`; it has {available} sheet(s). Check \
                     `sheet` (a name, or a position counted from 0)."
                ),
            ),
            Self::InvalidProvenance { reason } => (
                &codes::INVALID_PROVENANCE,
                format!(
                    "The provenance columns are invalid: {reason}. Use the class names of \
                     03 §4 and confidences from 0 to 1."
                ),
            ),
            Self::InvalidOption { option, reason } => (
                &codes::INVALID_OPTION,
                format!("The source option `{option}` is invalid: {reason}."),
            ),
            Self::Write { format, reason } => (
                &codes::WRITE_FAILED,
                format!("The table cannot be written as {format}: {reason}."),
            ),
            Self::MalformedTable { table, .. } => (
                &codes::MALFORMED_TABLE,
                format!(
                    "Table `{table}` is malformed: its columns do not match its schema. \
                     Rebuild it from equal-length columns."
                ),
            ),
            Self::DuplicateTable { id } => (
                &codes::MALFORMED_TABLE,
                format!("Table ID {id} is used twice in one bundle. Give each table its own ID."),
            ),
            Self::UnknownField { table, field } => (
                &codes::UNKNOWN_FIELD,
                format!(
                    "Table `{table}` has no field `{field}`. Check the spelling against the \
                     data's column names."
                ),
            ),
        };
        Diagnostic::new(code, message)
    }
}
