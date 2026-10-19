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
    /// A `snr.*` field metadata value cannot be read and was ignored.
    INVALID_METADATA, "SNR-DATA-040", Warning,
    "A field metadata value cannot be read and was ignored",
    "Fix the `snr.*` value in the file (03 §3 lists the allowed values) or declare the field in the spec's `data.fields`.",
    []
);

/// Every `SNR-DATA` code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Data,
    owner: "sonarithm-data",
    codes: &[INVALID_METADATA],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-DATA code table");
