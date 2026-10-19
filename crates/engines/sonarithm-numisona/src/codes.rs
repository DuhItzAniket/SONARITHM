// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! NUMISONA's diagnostic codes, in the `SNR-ENG` area shared by engine crates
//! (coding-standards §4.2: the message names the engine).
//!
//! NUMISONA uses `SNR-ENG-001` to `SNR-ENG-099`: 001–019 applied defaults (`Info`, FR-026),
//! 020–029 polarity and the concept table. A released code is never reused
//! (coding-standards §4.2).

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

macro_rules! code {
    (
        $(#[$doc:meta])* $name:ident, $code:literal, $severity:ident,
        $summary:literal, $help:literal, [$($rule:literal),*]
    ) => {
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
    /// `polarity: "auto"` was resolved from the field's concept.
    POLARITY_FROM_CONCEPT, "SNR-ENG-020", Info,
    "Polarity taken from the concept table",
    "Set `polarity` to `positive` or `negative` to choose the direction yourself.",
    ["SDR-12", "SDR-13", "SDR-14"]
);

code!(
    /// A concept read from the data is flagged and the polarity is automatic. Concepts
    /// declared in the spec get `SNR-SPEC-042` instead.
    FLAGGED_CONCEPT, "SNR-ENG-021", Warning,
    "Flagged concept with automatic polarity",
    "Listeners disagree on the pitch direction for this concept; set `polarity` explicitly, or flip it in your profile.",
    ["SDR-13", "SDR-14", "SDR-41"]
);

code!(
    /// The field's concept is not in the concept table, so the generic default applies.
    UNKNOWN_CONCEPT, "SNR-ENG-022", Info,
    "Concept not in the concept table",
    "The generic default (positive) applies; check the concept's spelling or set `polarity`.",
    ["SDR-12"]
);

code!(
    /// The listener's profile chose or flipped the polarity.
    POLARITY_FROM_PROFILE, "SNR-ENG-023", Info,
    "Polarity set by the listener's profile",
    "Change or clear the polarity setting in the profile to restore the spec's polarity.",
    ["SDR-15"]
);

/// Every NUMISONA code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-numisona",
    codes: &[
        POLARITY_FROM_CONCEPT,
        FLAGGED_CONCEPT,
        UNKNOWN_CONCEPT,
        POLARITY_FROM_PROFILE,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-ENG code table");
