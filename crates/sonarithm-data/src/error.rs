// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The error type of the data layer (coding-standards §4.1).
//!
//! A [`DataError`] stops the operation that raised it. Problems that do not, such as an
//! inferred type or an ignored metadata value, are [`Diagnostic`]s instead. Every error
//! converts to a diagnostic with a stable code through [`DataError::to_diagnostic`].

use arrow_schema::ArrowError;
use sonarithm_core::Diagnostic;

use crate::codes;

/// An error from the data layer.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DataError {
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
