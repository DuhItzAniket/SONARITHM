// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC's diagnostic codes, in its block of the shared `SNR-ENG` area:
//! `SNR-ENG-600` to `SNR-ENG-699` (coding-standards §4.2).
//!
//! 600–619 study protocols, 620–639 study logs. The perceptual lint (C05) uses its own
//! area, `SNR-LINT`. A released code is never reused (coding-standards §4.2).

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
    /// The protocol file is not TOML, or its tables do not have the protocol's shape.
    PROTOCOL_PARSE, "SNR-ENG-600", Error,
    "The study protocol cannot be read",
    "Check the TOML syntax and the field names against the `sonarithm-study/0` format; the message names the line.",
    []
);

code!(
    /// The protocol declares another format.
    PROTOCOL_FORMAT, "SNR-ENG-601", Error,
    "Unknown study protocol format",
    "Set `format = \"sonarithm-study/0\"`.",
    []
);

code!(
    /// An experiment, condition or block ID is malformed or repeated.
    PROTOCOL_ID, "SNR-ENG-602", Error,
    "Invalid or repeated ID in the study protocol",
    "Use 1 to 32 letters, digits and hyphens, unique within their kind.",
    []
);

code!(
    /// A condition's spec path is absolute or leaves the protocol's folder.
    PROTOCOL_PATH, "SNR-ENG-603", Error,
    "Spec path outside the protocol folder",
    "Give spec paths relative to the protocol file, without `..`, so logs never record user paths (ADR-0012).",
    []
);

code!(
    /// The protocol has no conditions, no blocks, an empty block or too many trials.
    PROTOCOL_EMPTY, "SNR-ENG-604", Error,
    "The study protocol has no runnable trials, or too many",
    "Add conditions and blocks with `repeats` of at least 1, and keep a protocol at or under 2000 trials.",
    []
);

code!(
    /// A block names a condition that is not defined.
    PROTOCOL_REFERENCE, "SNR-ENG-605", Error,
    "Unknown condition in a block",
    "Define the condition in a `[[conditions]]` table or remove it from the block.",
    []
);

code!(
    /// A block's stimulus parameters are invalid, or its ground truth cannot score the task.
    PROTOCOL_STIMULUS, "SNR-ENG-606", Error,
    "The block's stimulus cannot be generated or scored",
    "Fix the generator parameters, or pick a generator whose ground truth answers the task (point estimation: GEN-001 or GEN-002; trend: GEN-001; count: GEN-003; choice: GEN-004).",
    ["SDR-40"]
);

code!(
    /// The response format does not fit the task.
    PROTOCOL_RESPONSE, "SNR-ENG-607", Error,
    "The response does not fit the task",
    "Point estimation and counting take a number range; trend identification takes three choices (increased, decreased, stayed the same); choice tasks take one option per answer.",
    ["SDR-40"]
);

code!(
    /// The log cannot be read as a `sonarithm-study-log/0` log.
    LOG_UNREADABLE, "SNR-ENG-620", Error,
    "The study log cannot be read",
    "Resume from the log file the session wrote; the message names the first bad line.",
    []
);

code!(
    /// The log belongs to another session or to another version of the protocol.
    LOG_MISMATCH, "SNR-ENG-621", Error,
    "The study log does not match the protocol or the session",
    "Resume with the protocol file and session ID the log was started with; start a new session for a changed protocol.",
    []
);

/// Every code ECHOMETRIC emits, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-echometric",
    codes: &[
        PROTOCOL_PARSE,
        PROTOCOL_FORMAT,
        PROTOCOL_ID,
        PROTOCOL_PATH,
        PROTOCOL_EMPTY,
        PROTOCOL_REFERENCE,
        PROTOCOL_STIMULUS,
        PROTOCOL_RESPONSE,
        LOG_UNREADABLE,
        LOG_MISMATCH,
    ],
};

const _: () = assert!(validate(&CODES).is_ok());
