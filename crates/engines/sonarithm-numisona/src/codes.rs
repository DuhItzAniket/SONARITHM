// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! NUMISONA's diagnostic codes, in the `SNR-ENG` area shared by engine crates
//! (coding-standards §4.2: the message names the engine).
//!
//! NUMISONA uses `SNR-ENG-001` to `SNR-ENG-099`: 001–019 applied defaults and encoder
//! notes (`Info`, FR-026), 020–029 polarity and the concept table, 040–059 encoding
//! problems, 080–089 importers. A released code is never reused
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
    /// The time field came from `role: x`, or time follows row order.
    DEFAULT_TIME, "SNR-ENG-001", Info,
    "Time field chosen by default",
    "Set `substrate.time.field` to choose the field that orders the data in time.",
    []
);

code!(
    /// The track uses the default instrument.
    DEFAULT_INSTRUMENT, "SNR-ENG-002", Info,
    "Default instrument",
    "Set the track's `instrument` to choose another preset.",
    ["SDR-29"]
);

code!(
    /// Notes use the default duration.
    DEFAULT_DURATION, "SNR-ENG-003", Info,
    "Default note duration",
    "Set `channels.duration` to a value or a field.",
    []
);

code!(
    /// Loudness is constant at the default level.
    DEFAULT_LOUDNESS, "SNR-ENG-004", Info,
    "Default constant loudness",
    "Set `channels.loudness.value` to change the level; avoid mapping data to loudness.",
    ["SDR-23"]
);

code!(
    /// The track maps nothing to pitch.
    DEFAULT_PITCH, "SNR-ENG-005", Info,
    "Constant default pitch",
    "Map a field to `channels.pitch` to carry the data in pitch (SDR-07).",
    ["SDR-07", "SDR-08"]
);

code!(
    /// A mapped channel without `range` uses its default range.
    CHANNEL_RANGE, "SNR-ENG-006", Info,
    "Default channel range",
    "Set the channel's `range` to choose the output range.",
    []
);

code!(
    /// A redundant encoding of the pitch field was added.
    REDUNDANT_ENCODING, "SNR-ENG-007", Info,
    "Redundant encoding added",
    "Remove the entry from `redundant` to drop it.",
    ["A11Y-13"]
);

code!(
    /// Categories of a field were given timbres.
    TIMBRE_CATEGORIES, "SNR-ENG-008", Info,
    "Timbre per category",
    "The legend speaks this assignment; set `instrument` to change the first timbre.",
    ["SDR-25"]
);

code!(
    /// A psychophysical scale is in use; its exponent is guidance.
    PSYCHOPHYSICAL, "SNR-ENG-009", Info,
    "Psychophysical scaling, as guidance",
    "Exponents come from magnitude-estimation studies and are a starting point, not a law.",
    ["SDR-11"]
);

code!(
    /// Rows with missing values were silenced or interpolated.
    MISSING_VALUES, "SNR-ENG-010", Info,
    "Missing values",
    "Set `data.missing` to `silence`, `earcon` or `interpolate`.",
    []
);

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

code!(
    /// The data has no table.
    NO_TABLE, "SNR-ENG-040", Error,
    "No data table",
    "Check that the data source produced a table.",
    []
);

code!(
    /// A channel names a field the data does not have.
    UNKNOWN_FIELD, "SNR-ENG-041", Error,
    "Unknown field",
    "Check the field name against the data's columns.",
    []
);

code!(
    /// A field's column type cannot drive a channel.
    UNSUPPORTED_FIELD, "SNR-ENG-042", Error,
    "Unsupported field type",
    "Map numbers, dates, text or dictionary fields; convert other columns with a transform.",
    []
);

code!(
    /// A continuous scale was asked to place categories.
    FIELD_NOT_NUMERIC, "SNR-ENG-043", Error,
    "Categorical field on a continuous scale",
    "Use a `band` or `ordinal` scale, or map the field to `timbre`.",
    ["SDR-25"]
);

code!(
    /// A scale or pitch range cannot be built for the data.
    SCALE_FAILED, "SNR-ENG-044", Error,
    "Scale cannot be fitted",
    "Fix the scale's domain, exponent or reference, or the channel's range.",
    []
);

code!(
    /// A track filter cannot be evaluated.
    FILTER_FAILED, "SNR-ENG-045", Error,
    "Track filter failed",
    "Fix the expression; see the data layer's expression language.",
    []
);

code!(
    /// Values outside the domain were clamped to its ends.
    OFF_SCALE, "SNR-ENG-046", Warning,
    "Values outside the domain",
    "Widen the scale's domain, or keep the clamping; the values sound at the range's ends.",
    []
);

code!(
    /// Rows without a time value were left out.
    ROWS_WITHOUT_TIME, "SNR-ENG-047", Warning,
    "Rows without a time value",
    "Fill or filter the missing time values.",
    []
);

code!(
    /// `secondsPerUnit` without a numeric time field.
    PACE_WITHOUT_UNITS, "SNR-ENG-048", Warning,
    "Pace in units without a numeric time field",
    "Name a numeric or temporal time field, or use `pointsPerSecond`.",
    []
);

code!(
    /// Discrete tones come faster than about 8 per second.
    PACE_ABOVE_LINT, "SNR-ENG-049", Warning,
    "Tones faster than 8 per second",
    "Slow the pace to 2-4 points per second for overviews, or use a glide.",
    ["SDR-19", "SDR-20"]
);

code!(
    /// Taps faster than about 10 per second.
    TAPS_TOO_FAST, "SNR-ENG-050", Warning,
    "Taps may fuse",
    "Lengthen the notes, lower the tap range or slow the pace.",
    ["SDR-20"]
);

code!(
    /// Loudness carries data.
    LOUDNESS_CARRIES_DATA, "SNR-ENG-051", Warning,
    "Loudness carries data",
    "Map the field to pitch, duration or taps instead; keep loudness constant.",
    ["SDR-23"]
);

code!(
    /// Pan is the only carrier of a field.
    PAN_SOLE_CARRIER, "SNR-ENG-052", Warning,
    "Pan is the only carrier",
    "Map the field to another channel as well.",
    ["A11Y-14"]
);

code!(
    /// Pan repeats the pitch field in Analysis mode.
    REDUNDANT_SPATIAL, "SNR-ENG-053", Warning,
    "Pan repeats the pitch field",
    "Map pan to another field or to a constant in Analysis mode.",
    ["SDR-27"]
);

code!(
    /// More categories than timbres.
    TIMBRE_REPEATS, "SNR-ENG-054", Warning,
    "Timbres repeat",
    "Split the track by category, or announce the categories.",
    ["SDR-25", "A11Y-14"]
);

code!(
    /// A channel or encoding cannot be represented for this mark or IR version.
    CHANNEL_NOT_RENDERED, "SNR-ENG-055", Warning,
    "Channel not encoded",
    "Remove the channel, or use a mark that supports it.",
    []
);

code!(
    /// A redundant encoding was not added.
    REDUNDANT_IGNORED, "SNR-ENG-056", Warning,
    "Redundant encoding not added",
    "Map a field to pitch, or drop the explicit channel the redundant encoding would add.",
    ["A11Y-13"]
);

code!(
    /// An Erie construct with no spec v0.1 equivalent was left out of an import (04 §9).
    ERIE_UNMAPPED, "SNR-ENG-080", Warning,
    "Erie construct not imported",
    "Spec v0.1 has no equivalent; the note says what the import does instead. Adjust the imported spec by hand if the construct matters.",
    ["SDR-05"]
);

code!(
    /// An Erie construct was imported with a change of meaning (04 §9).
    ERIE_APPROXIMATED, "SNR-ENG-081", Info,
    "Erie construct imported approximately",
    "Check the imported spec against the note; the construct maps onto spec v0.1 with the stated difference.",
    ["SDR-05"]
);

/// Every NUMISONA code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-numisona",
    codes: &[
        DEFAULT_TIME,
        DEFAULT_INSTRUMENT,
        DEFAULT_DURATION,
        DEFAULT_LOUDNESS,
        DEFAULT_PITCH,
        CHANNEL_RANGE,
        REDUNDANT_ENCODING,
        TIMBRE_CATEGORIES,
        PSYCHOPHYSICAL,
        MISSING_VALUES,
        POLARITY_FROM_CONCEPT,
        FLAGGED_CONCEPT,
        UNKNOWN_CONCEPT,
        POLARITY_FROM_PROFILE,
        NO_TABLE,
        UNKNOWN_FIELD,
        UNSUPPORTED_FIELD,
        FIELD_NOT_NUMERIC,
        SCALE_FAILED,
        FILTER_FAILED,
        OFF_SCALE,
        ROWS_WITHOUT_TIME,
        PACE_WITHOUT_UNITS,
        PACE_ABOVE_LINT,
        TAPS_TOO_FAST,
        LOUDNESS_CARRIES_DATA,
        PAN_SOLE_CARRIER,
        REDUNDANT_SPATIAL,
        TIMBRE_REPEATS,
        CHANNEL_NOT_RENDERED,
        REDUNDANT_IGNORED,
        ERIE_UNMAPPED,
        ERIE_APPROXIMATED,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-ENG code table");
