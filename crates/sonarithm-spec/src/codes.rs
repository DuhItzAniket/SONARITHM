// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `SNR-SPEC` diagnostic codes (RFC-0001 §12).
//!
//! Codes are grouped by tens: 001–009 syntax, shape and version; 010–019 tracks; 020–029
//! channels; 030–039 scales and ranges; 040–049 mode and research rules; 050–059 data, pace
//! and transforms; 060–069 output and playback; 070–079 data-aware checks. A released code
//! is never reused (coding-standards §4.2).

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
    /// Invalid JSON.
    SYNTAX, "SNR-SPEC-001", Error,
    "Invalid JSON",
    "Fix the JSON syntax at the reported line and column. Comments and trailing commas are not allowed.",
    []
);

code!(
    /// A value has the wrong type or is not allowed here.
    INVALID_VALUE, "SNR-SPEC-002", Error,
    "A value has the wrong type or is not allowed here",
    "Use a value of the type the key expects; the message lists the allowed values.",
    []
);

code!(
    /// A required key is missing.
    MISSING_KEY, "SNR-SPEC-003", Error,
    "A required key is missing",
    "Add the key; RFC-0001 lists the required keys of each object.",
    []
);

code!(
    /// Unknown key.
    UNKNOWN_KEY, "SNR-SPEC-004", Error,
    "Unknown key",
    "Remove the key or fix its spelling. Vendor extensions go in top-level keys that start with `x-`.",
    []
);

code!(
    /// Unsupported `specVersion`.
    UNSUPPORTED_VERSION, "SNR-SPEC-005", Error,
    "Unsupported `specVersion`",
    "Use `\"0.1\"`, or a SONARITHM version that reads this spec version.",
    []
);

code!(
    /// Older `specVersion` migrated in memory.
    MIGRATED_VERSION, "SNR-SPEC-006", Warning,
    "Older `specVersion` migrated in memory",
    "Run `sonarithm spec migrate` to update the file; the compiler never rewrites it.",
    []
);

code!(
    /// The spec has no tracks.
    NO_TRACKS, "SNR-SPEC-010", Error,
    "The spec has no tracks",
    "Add at least one track to `tracks`.",
    []
);

code!(
    /// Duplicate track ID.
    DUPLICATE_TRACK_ID, "SNR-SPEC-011", Error,
    "Duplicate track ID",
    "Give every track a unique `id`.",
    []
);

code!(
    /// Invalid track ID.
    INVALID_TRACK_ID, "SNR-SPEC-012", Error,
    "Invalid track ID",
    "Start the ID with a letter and use only letters, digits, `_` and `-`, up to 64 characters.",
    []
);

code!(
    /// Reference to an unknown track.
    UNKNOWN_TRACK, "SNR-SPEC-013", Error,
    "Reference to an unknown track",
    "Name a track `id` that exists in `tracks`.",
    []
);

code!(
    /// A channel has both `field` and `value`.
    FIELD_AND_VALUE, "SNR-SPEC-020", Error,
    "A channel has both `field` and `value`",
    "Keep `field` to map data, or `value` for a constant, not both.",
    []
);

code!(
    /// A channel has neither `field` nor `value`.
    NO_FIELD_OR_VALUE, "SNR-SPEC-021", Error,
    "A channel has neither `field` nor `value`",
    "Add `field` to map data, or `value` for a constant.",
    []
);

code!(
    /// Channel not valid for the mark.
    CHANNEL_NOT_FOR_MARK, "SNR-SPEC-022", Error,
    "Channel not valid for the mark",
    "Remove the channel or choose a mark that accepts it (04 §5.3).",
    []
);

code!(
    /// Unit not accepted by the channel.
    UNIT_NOT_ACCEPTED, "SNR-SPEC-023", Error,
    "Unit not accepted by the channel",
    "Use one of the units the channel accepts (04 §5.3).",
    []
);

code!(
    /// Value outside the channel's bounds.
    OUT_OF_BOUNDS, "SNR-SPEC-024", Error,
    "Value outside the channel's bounds",
    "Use a value inside the channel's bounds, for example pan from -1 to 1.",
    []
);

code!(
    /// Mapping option on a constant channel.
    OPTION_ON_CONSTANT, "SNR-SPEC-025", Error,
    "Mapping option on a constant channel",
    "`scale`, `range`, `space` and `polarity` apply only to a channel mapped from a `field`.",
    []
);

code!(
    /// `space` on a channel other than pitch.
    SPACE_NOT_PITCH, "SNR-SPEC-026", Error,
    "`space` on a channel other than pitch",
    "Remove `space`; it applies only to the pitch channel.",
    []
);

code!(
    /// Invalid scale parameters.
    INVALID_SCALE, "SNR-SPEC-030", Error,
    "Invalid scale parameters",
    "Give `pow` and `psychophysical` scales a positive `exponent`, and `psychophysical` a `reference`.",
    ["SDR-11"]
);

code!(
    /// Range written high to low.
    REVERSED_RANGE, "SNR-SPEC-031", Error,
    "Range written high to low",
    "Write the range low to high and use `\"polarity\": \"negative\"` to invert the mapping.",
    ["SDR-15"]
);

code!(
    /// Invalid scale domain.
    INVALID_DOMAIN, "SNR-SPEC-032", Error,
    "Invalid scale domain",
    "Write the domain as `[min, max]` with min below max; a `log` scale needs a positive domain.",
    []
);

code!(
    /// `music` outside Presentation mode.
    MUSIC_OUTSIDE_PRESENTATION, "SNR-SPEC-040", Error,
    "`music` outside Presentation mode",
    "Set `\"mode\": \"presentation\"` or remove `music`; Analysis mode keeps pitch unquantized.",
    ["SDR-10"]
);

code!(
    /// Interpretive (L4) description requested.
    L4_DESCRIPTION, "SNR-SPEC-041", Error,
    "Interpretive (L4) description requested",
    "Use levels L1 to L3; interpretation is never automatic.",
    ["A11Y-02"]
);

code!(
    /// Flagged concept with automatic polarity.
    FLAGGED_CONCEPT, "SNR-SPEC-042", Warning,
    "Flagged concept with automatic polarity",
    "Set `polarity` explicitly; listeners disagree on the direction for this concept.",
    ["SDR-13", "SDR-14", "SDR-41"]
);

code!(
    /// Invalid pace.
    INVALID_PACE, "SNR-SPEC-050", Error,
    "Invalid pace",
    "Use a positive, finite pace with exactly one of `pointsPerSecond`, `duration` or `secondsPerUnit`.",
    []
);

code!(
    /// Invalid data source parameters.
    INVALID_SOURCE, "SNR-SPEC-051", Error,
    "Invalid data source parameters",
    "Fix the source parameters the message names (RFC-0001 §4).",
    []
);

code!(
    /// Absolute path in the spec.
    ABSOLUTE_PATH, "SNR-SPEC-052", Warning,
    "Absolute path in the spec",
    "Use a path relative to the spec, so the spec carries no user paths and stays portable.",
    []
);

code!(
    /// Invalid transform parameters.
    INVALID_TRANSFORM, "SNR-SPEC-053", Error,
    "Invalid transform parameters",
    "Fix the transform parameters the message names (RFC-0001 §4).",
    []
);

code!(
    /// Invalid output parameters.
    INVALID_OUTPUT, "SNR-SPEC-060", Error,
    "Invalid output parameters",
    "Use a sample rate from 8000 to 384000 Hz, 1 to 64 channels, and a power-of-two block size from 16 to 8192.",
    []
);

code!(
    /// Invalid playback parameters.
    INVALID_PLAYBACK, "SNR-SPEC-061", Error,
    "Invalid playback parameters",
    "Use a positive, finite playback speed.",
    []
);

code!(
    /// Unknown field.
    UNKNOWN_FIELD, "SNR-SPEC-070", Error,
    "Unknown field",
    "Name a field of the data, or declare it in `data.fields`.",
    []
);

code!(
    /// Field type not compatible with the channel.
    INCOMPATIBLE_FIELD, "SNR-SPEC-071", Error,
    "Field type not compatible with the channel",
    "Map nominal fields to timbre and quantitative fields to pitch, duration or rate.",
    ["SDR-25"]
);

/// Every `SNR-SPEC` code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Spec,
    owner: "sonarithm-spec",
    codes: &[
        SYNTAX,
        INVALID_VALUE,
        MISSING_KEY,
        UNKNOWN_KEY,
        UNSUPPORTED_VERSION,
        MIGRATED_VERSION,
        NO_TRACKS,
        DUPLICATE_TRACK_ID,
        INVALID_TRACK_ID,
        UNKNOWN_TRACK,
        FIELD_AND_VALUE,
        NO_FIELD_OR_VALUE,
        CHANNEL_NOT_FOR_MARK,
        UNIT_NOT_ACCEPTED,
        OUT_OF_BOUNDS,
        OPTION_ON_CONSTANT,
        SPACE_NOT_PITCH,
        INVALID_SCALE,
        REVERSED_RANGE,
        INVALID_DOMAIN,
        MUSIC_OUTSIDE_PRESENTATION,
        L4_DESCRIPTION,
        FLAGGED_CONCEPT,
        INVALID_PACE,
        INVALID_SOURCE,
        ABSOLUTE_PATH,
        INVALID_TRANSFORM,
        INVALID_OUTPUT,
        INVALID_PLAYBACK,
        UNKNOWN_FIELD,
        INCOMPATIBLE_FIELD,
    ],
};

const _: () = assert!(
    validate(&CODES).is_ok(),
    "the SNR-SPEC code table is invalid; run `cargo test -p sonarithm-spec codes` for details"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_table_is_valid() {
        assert_eq!(validate(&CODES), Ok(()));
    }
}
