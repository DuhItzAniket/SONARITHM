// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Errors of the localization layer.

/// Why a message or value could not be formatted.
///
/// Every variant is a programming or catalogue error: the embedded catalogues are checked
/// by this crate's tests, so callers normally see these only while developing.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum I18nError {
    /// The requested locale is not a valid BCP 47 language identifier.
    #[error("`{0}` is not a valid locale identifier")]
    InvalidLocale(String),
    /// A catalogue file does not parse, or defines a message twice.
    #[error("catalogue {locale}/{file} is invalid: {details}")]
    Catalogue {
        /// The catalogue locale.
        locale: String,
        /// The file name inside the locale directory.
        file: String,
        /// The parser or bundle errors.
        details: String,
    },
    /// No message has this ID.
    #[error("no message `{id}` in the {locale} catalogue")]
    MissingMessage {
        /// The message ID.
        id: String,
        /// The catalogue locale.
        locale: String,
    },
    /// The message exists but has no value (it only has attributes).
    #[error("message `{id}` has no value")]
    NoValue {
        /// The message ID.
        id: String,
    },
    /// Fluent reported errors while formatting, such as a missing argument.
    #[error("formatting `{id}` failed: {details}")]
    Format {
        /// The message ID.
        id: String,
        /// The resolver errors.
        details: String,
    },
    /// A number argument is NaN or infinite. Missing values are announced as such by the
    /// caller; they are never formatted as numbers.
    #[error("argument `{name}` is not a finite number")]
    NonFinite {
        /// The argument name.
        name: String,
    },
    /// A date is outside the Gregorian calendar's valid range.
    #[error("{year}-{month:02}-{day:02} is not a valid date")]
    InvalidDate {
        /// The year.
        year: i32,
        /// The month, 1–12.
        month: u8,
        /// The day of the month.
        day: u8,
    },
}
