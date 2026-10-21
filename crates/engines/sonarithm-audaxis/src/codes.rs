// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! AUDAXIS's diagnostic codes, in the `SNR-ENG` area shared by engine crates
//! (coding-standards §4.2: the message names the engine).
//!
//! AUDAXIS uses `SNR-ENG-100` to `SNR-ENG-199`: 100–103 spatial placement and the
//! never-alone guard (A11Y-14), 104–119 context cues (SDR-16, SDR-17). A released code is never reused (coding-standards §4.2).

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

code!(
    /// A static reference tone was asked for.
    STATIC_REFERENCE, "SNR-ENG-104", Warning,
    "Static reference tone",
    "Use the default dynamic-extreme reference (`context.reference.kind: dynamic-extreme`); a static reference did not help listeners (SDR-17).",
    ["SDR-17"]
);

code!(
    /// The reference tone cannot be built.
    REFERENCE_UNAVAILABLE, "SNR-ENG-105", Warning,
    "Reference tone left out",
    "Point `context.reference.track` at a data track with at least two different values, and keep a reference value inside the data's range.",
    ["SDR-17"]
);

code!(
    /// Ticks were asked for but cannot be placed.
    TICKS_UNAVAILABLE, "SNR-ENG-106", Warning,
    "Ticks left out",
    "Ticks need a data track with at least two points at different x positions, and a positive `context.ticks.every`.",
    ["SDR-16"]
);

code!(
    /// The automatic tick choice.
    TICKS_DECIDED, "SNR-ENG-107", Info,
    "Ticks chosen automatically",
    "Set `context.ticks.enabled` to `true` or `false`, and `context.ticks.every` to choose the interval.",
    ["SDR-16"]
);

/// Every AUDAXIS code, sorted.
const ALL: &[CodeInfo] = &[
    PAN_ONLY,
    TIMBRE_ONLY,
    PITCH_EXTREME,
    SPATIAL_MODE_UNAVAILABLE,
    STATIC_REFERENCE,
    REFERENCE_UNAVAILABLE,
    TICKS_UNAVAILABLE,
    TICKS_DECIDED,
];

/// AUDAXIS's code table, for the registry page (`cargo xtask codes`).
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-audaxis",
    codes: ALL,
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-ENG code table");
