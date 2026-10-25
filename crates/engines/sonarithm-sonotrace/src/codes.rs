// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONOTRACE's diagnostic codes, in its block of the shared `SNR-ENG` area:
//! `SNR-ENG-200` to `SNR-ENG-299` (coding-standards §4.2).
//!
//! 200–219 streams and live encoding; 220–239 transport helpers; 240–259 monitoring.
//! A released code is never reused.

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

/// A spec's `stream` source has no stream connected to it yet.
pub const STREAM_NOT_CONNECTED: CodeInfo = CodeInfo {
    code: "SNR-ENG-200",
    severity: Severity::Error,
    summary: "No stream is connected",
    help: "Connect the stream first (a connection opens only on your command); the spec then \
           compiles against the rows the stream holds.",
    rules: &["A11Y-16"],
    status: CodeStatus::Active,
};

/// A live update encoded events that have no data row, which live mode cannot place.
pub const LIVE_EVENT_WITHOUT_ROW: CodeInfo = CodeInfo {
    code: "SNR-ENG-201",
    severity: Severity::Warning,
    summary: "Live mode skipped events without a data row",
    help: "Live mode plays the events of new rows only. Context events such as ticks and \
           earcons come from AUDAXIS in live mode instead.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A spec names a monitoring soundscape this build does not have (sound-design §10.7).
pub const SOUNDSCAPE_UNAVAILABLE: CodeInfo = CodeInfo {
    code: "SNR-ENG-240",
    severity: Severity::Warning,
    summary: "The soundscape is not available",
    help: "Only the default soundscape exists until B06's evaluation chooses others; it plays            instead.",
    rules: &["SDR-35"],
    status: CodeStatus::Active,
};

/// The experimental comfort report of a monitoring session (SDR-39, INFER).
pub const COMFORT_BUDGET: CodeInfo = CodeInfo {
    code: "SNR-ENG-241",
    severity: Severity::Info,
    summary: "Experimental comfort budget",
    help: "SDR-39 is an inference: the share of the session the soundscape is bright is            reported, not enforced, until an experiment validates a budget (H-12).",
    rules: &["SDR-39"],
    status: CodeStatus::Active,
};

/// Every SONOTRACE code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-sonotrace",
    codes: &[
        STREAM_NOT_CONNECTED,
        LIVE_EVENT_WITHOUT_ROW,
        SOUNDSCAPE_UNAVAILABLE,
        COMFORT_BUDGET,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SONOTRACE code table");
