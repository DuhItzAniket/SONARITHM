// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `SNR-SPEC` diagnostic codes (RFC-0001 §12).
//!
//! Codes are grouped by tens: 001–009 syntax, shape and version; 010–019 tracks; 020–029
//! channels; 030–039 scales and ranges; 040–049 mode and research rules; 050–059 data, pace
//! and transforms; 060–069 output and playback; 070–079 data-aware checks; 080–089 applied
//! defaults; 090–099 the chart correction file (RFC-0005). A released code
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

code!(
    /// Unknown scale tonic or mode (RFC-0004 §7).
    UNKNOWN_SCALE_NAME, "SNR-SPEC-043", Error,
    "Unknown scale tonic or mode",
    "Write the scale as a tonic and a mode, such as \"C major\" or \"D minor pentatonic\". Tonics are A to G with an optional # or b. Modes: major, minor, harmonic minor, melodic minor, dorian, phrygian, lydian, mixolydian, locrian, major pentatonic, minor pentatonic, blues, whole tone, chromatic. Or give custom cents: {\"tonic\": \"D\", \"cents\": [0, 200, …]}.",
    ["SDR-10"]
);

code!(
    /// Invalid tuning, or a tuning on a custom scale (RFC-0004 §7).
    INVALID_TUNING, "SNR-SPEC-044", Error,
    "Invalid tuning",
    "Use \"12-TET\" or {\"cents\": [0, …]} with twelve strictly increasing values below 1200 (\"just\" waits for a cited ratio table, RFC-0004 Q13). A custom scale carries its own cents, so give it no tuning.",
    ["SDR-10"]
);

code!(
    /// A lint rule ID the lint does not implement (RFC-0004 §4).
    UNKNOWN_LINT_RULE, "SNR-SPEC-045", Warning,
    "Unknown lint rule",
    "Lint rules are named by design-rule ID, such as SDR-17 or A11Y-14. This lint does not implement the rule, so the setting has no effect.",
    []
);

code!(
    /// A lint suppression without a reason (RFC-0004 §4).
    SUPPRESSION_WITHOUT_REASON, "SNR-SPEC-046", Error,
    "A lint suppression has no reason",
    "Say why the rule does not apply in `reason`; the lint reports it whenever it would have warned.",
    []
);

code!(
    /// An `experimental` switch this version does not know (RFC-0006 §2).
    UNKNOWN_EXPERIMENT, "SNR-SPEC-047", Warning,
    "Unknown experimental switch",
    "This SONARITHM does not know the switch, so it has no effect. Check its spelling, or use the version that introduced it.",
    []
);

code!(
    /// An invalid chart calibration (RFC-0004 §5).
    INVALID_CALIBRATION, "SNR-SPEC-054", Error,
    "Invalid chart calibration",
    "Give `axes.x` and `axes.y`, each with a kind (linear, log, date, category) and at least two anchors {\"at\": [x, y], \"value\": …} at distinct points with distinct values; log values above zero, dates in ISO 8601.",
    ["SDR-01"]
);

code!(
    /// An invalid chart correction (RFC-0004 §6).
    INVALID_CORRECTION, "SNR-SPEC-055", Error,
    "Invalid chart correction",
    "Write `corrections` as {\"edits\": [...]}; each edit has an `op` (add, move, delete, retype), the `series`, and for move, delete and retype the `point` index and the extracted value in `was`.",
    ["SDR-01"]
);

code!(
    /// `corrections.path` is reserved for RFC-0005 (RFC-0004 §6).
    CORRECTION_FILE_RESERVED, "SNR-SPEC-056", Error,
    "Correction files need spec 0.3",
    "`corrections.path` names a correction file (RFC-0005), which spec 0.3 defines; set `\"specVersion\": \"0.3\"` or write the edits inline in `corrections.edits`.",
    []
);

code!(
    /// Not a correction file, or one made by a newer major version (RFC-0005 §8).
    CORRECTION_FILE_UNREADABLE, "SNR-SPEC-090", Error,
    "Not a correction file this version reads",
    "A correction file has `\"format\": \"sonarithm-corrections\"` and a version this SONARITHM reads (1.x). A file from a newer major version needs a newer SONARITHM.",
    []
);

code!(
    /// A correction file with a shape error (RFC-0005 §8).
    CORRECTION_FILE_SHAPE, "SNR-SPEC-091", Error,
    "The correction file is malformed",
    "Fix the key at the reported pointer. Every edit has an `id` (e1, e2, …, unique), an `op`, a `series`, and `point` and `was` for move, delete and retype; `+e<n>` refers to an `add` earlier in the file.",
    []
);

code!(
    /// A correction file over a limit (RFC-0005 §8).
    CORRECTION_FILE_LIMIT, "SNR-SPEC-092", Error,
    "The correction file is too large",
    "A correction file holds at most 16 MiB, 100 000 edits, 1 000-character notes and a 4 MiB image; split the corrections or leave the image out.",
    []
);

code!(
    /// An unsafe `corrections.path` (RFC-0005 §7).
    CORRECTION_PATH_UNSAFE, "SNR-SPEC-093", Error,
    "The correction file path is not allowed",
    "Name the correction file by a path relative to the spec, without `..` past the spec's folder, a drive letter or a URL, so the spec carries no user paths.",
    []
);

code!(
    /// A correction file from a newer minor version (RFC-0005 §8).
    CORRECTION_FILE_NEWER, "SNR-SPEC-094", Warning,
    "The correction file is from a newer version",
    "Keys this version does not know were ignored; saving writes this version and drops them.",
    []
);

code!(
    /// A data-independent default was applied (05 §7: explain what you assumed).
    DEFAULT_APPLIED, "SNR-SPEC-080", Info,
    "Default applied",
    "The compiler filled in a default; write the key to choose another value.",
    []
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
        UNKNOWN_SCALE_NAME,
        INVALID_TUNING,
        UNKNOWN_LINT_RULE,
        SUPPRESSION_WITHOUT_REASON,
        UNKNOWN_EXPERIMENT,
        INVALID_PACE,
        INVALID_SOURCE,
        ABSOLUTE_PATH,
        INVALID_TRANSFORM,
        INVALID_CALIBRATION,
        INVALID_CORRECTION,
        CORRECTION_FILE_RESERVED,
        INVALID_OUTPUT,
        INVALID_PLAYBACK,
        UNKNOWN_FIELD,
        INCOMPATIBLE_FIELD,
        DEFAULT_APPLIED,
        CORRECTION_FILE_UNREADABLE,
        CORRECTION_FILE_SHAPE,
        CORRECTION_FILE_LIMIT,
        CORRECTION_PATH_UNSAFE,
        CORRECTION_FILE_NEWER,
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
