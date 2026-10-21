// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! AUDAXIS's diagnostic codes, in the `SNR-ENG` area shared by engine crates
//! (coding-standards §4.2: the message names the engine).
//!
//! AUDAXIS uses `SNR-ENG-100` to `SNR-ENG-199`: 100–119 spatial placement and the
//! never-alone guard (A11Y-14). A released code is never reused (coding-standards §4.2).

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
    /// Only the stereo position varies along a data track.
    PAN_ONLY, "SNR-ENG-100", Warning,
    "Only pan varies",
    "Map the field to pitch or duration as well, so it survives mono output, NVDA sound split and one-ear listening.",
    ["A11Y-14"]
);

code!(
    /// Only the timbre varies along a data track.
    TIMBRE_ONLY, "SNR-ENG-101", Warning,
    "Only timbre varies",
    "Map the field to pitch, duration or taps as well, or announce the categories.",
    ["A11Y-14", "SDR-25"]
);

code!(
    /// A data track plays pitches outside the expanded SDR-08 range.
    PITCH_EXTREME, "SNR-ENG-102", Warning,
    "Pitches outside the expanded range",
    "Keep data pitches within 65 to 1480 Hz (SDR-08), so they survive high-frequency hearing loss and small speakers.",
    ["A11Y-14", "SDR-08"]
);

code!(
    /// A spatial mode that arrives in D03 was asked for.
    SPATIAL_MODE_UNAVAILABLE, "SNR-ENG-103", Warning,
    "Spatial mode not available",
    "Binaural and multichannel output arrive in D03; this build plays stereo.",
    ["SDR-30"]
);

/// Every AUDAXIS code, sorted.
const ALL: &[CodeInfo] = &[
    PAN_ONLY,
    TIMBRE_ONLY,
    PITCH_EXTREME,
    SPATIAL_MODE_UNAVAILABLE,
];

/// AUDAXIS's code table, for the registry page (`cargo xtask codes`).
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-audaxis",
    codes: ALL,
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-ENG code table");
