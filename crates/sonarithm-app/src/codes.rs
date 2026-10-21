// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `SNR-APP` diagnostic codes of the headless app core (ADR-0017 §3). 001–009: the IPC
//! protocol, sessions and transport; 010–019: settings; 020–029: jobs and export; 030–039:
//! projects; 040–049: spec editing; 050–059: binary views; 060–069: files the user chose. A
//! released code is never reused (coding-standards §4.2).

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

/// The UI and the host speak different IPC protocol versions (09 §2.2).
pub const PROTOCOL_MISMATCH: CodeInfo = CodeInfo {
    code: "SNR-APP-001",
    severity: Severity::Error,
    summary: "IPC protocol version mismatch",
    help: "Build the user interface and the host from the same commit, so both speak the same \
           protocol version.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A request names a dataset that is not open.
pub const UNKNOWN_DATASET: CodeInfo = CodeInfo {
    code: "SNR-APP-002",
    severity: Severity::Error,
    summary: "No open dataset has this ID",
    help: "dataset_open returns the ID of each dataset it opens; dataset_list lists them.",
    rules: &[],
    status: CodeStatus::Active,
};

/// `compile` was called before a spec was set.
pub const NO_SPEC: CodeInfo = CodeInfo {
    code: "SNR-APP-003",
    severity: Severity::Error,
    summary: "No spec to compile",
    help: "Set a spec with spec_set first; its data source can name an open dataset as \
           {\"kind\": \"dataset\", \"id\": \"d1\"}.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Playback or export was asked for before a spec compiled.
pub const NOT_COMPILED: CodeInfo = CodeInfo {
    code: "SNR-APP-004",
    severity: Severity::Error,
    summary: "Nothing is compiled yet",
    help: "Compile the spec first; playback and export use the last Score that compiled.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A transport value is out of range: a position, speed or loop.
pub const TRANSPORT_VALUE: CodeInfo = CodeInfo {
    code: "SNR-APP-005",
    severity: Severity::Error,
    summary: "Transport value out of range",
    help: "Positions are seconds from 0 to the Score's duration, speeds are 0.25 to 4 times, \
           and a loop needs a start before its end.",
    rules: &["A11Y-09"],
    status: CodeStatus::Active,
};

/// The audio device could not be opened or failed.
pub const AUDIO_UNAVAILABLE: CodeInfo = CodeInfo {
    code: "SNR-APP-006",
    severity: Severity::Error,
    summary: "Audio output is not available",
    help: "Check that an output device is connected and not in exclusive use by another \
           application, or choose another device with device_select.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The app core's control thread stopped responding.
pub const CONTROL_THREAD: CodeInfo = CodeInfo {
    code: "SNR-APP-007",
    severity: Severity::Error,
    summary: "The playback control thread is not responding",
    help: "Restart SONARITHM. If it happens again, report it with the log file.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A compile failed without a diagnostic to show.
pub const COMPILE_FAILED: CodeInfo = CodeInfo {
    code: "SNR-APP-008",
    severity: Severity::Error,
    summary: "The compile failed",
    help: "Check the spec and its data source, then compile again. If it happens again, report            it with the spec.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A request is not a valid IPC message.
pub const BAD_REQUEST: CodeInfo = CodeInfo {
    code: "SNR-APP-009",
    severity: Severity::Error,
    summary: "Invalid IPC request",
    help: "Send a request generated from the protocol types: {\"command\": \"<name>\", \"args\":            {...}}.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Settings could not be written.
pub const SETTINGS_WRITE: CodeInfo = CodeInfo {
    code: "SNR-APP-010",
    severity: Severity::Error,
    summary: "Settings could not be saved",
    help: "Check that the settings folder is writable and the disk is not full. The previous \
           settings stay in place.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The settings file was unreadable; the backup or the defaults were used.
pub const SETTINGS_RECOVERED: CodeInfo = CodeInfo {
    code: "SNR-APP-011",
    severity: Severity::Warning,
    summary: "Settings were recovered",
    help: "The settings file could not be read, so the last good copy or the defaults are in \
           use. Saving settings writes a new file.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The settings file comes from a newer version of SONARITHM.
pub const SETTINGS_NEWER: CodeInfo = CodeInfo {
    code: "SNR-APP-012",
    severity: Severity::Warning,
    summary: "Settings come from a newer version",
    help: "This version uses its defaults and does not overwrite the newer file. Update \
           SONARITHM to use those settings.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A setting has an invalid value.
pub const SETTINGS_VALUE: CodeInfo = CodeInfo {
    code: "SNR-APP-013",
    severity: Severity::Error,
    summary: "Invalid setting",
    help: "The sonification volume is −60 to 0 dB, and a buffer size is 16 to 8192 frames.",
    rules: &["A11Y-15"],
    status: CodeStatus::Active,
};

/// A job ID is unknown.
pub const UNKNOWN_JOB: CodeInfo = CodeInfo {
    code: "SNR-APP-020",
    severity: Severity::Error,
    summary: "No job has this ID",
    help: "export_render returns the job's ID; job_list lists the jobs of this session.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The export path is not usable.
pub const EXPORT_PATH: CodeInfo = CodeInfo {
    code: "SNR-APP-021",
    severity: Severity::Error,
    summary: "Export path not usable",
    help: "Choose a file ending in .wav or .flac in a folder that exists.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The export failed while rendering or writing.
pub const EXPORT_FAILED: CodeInfo = CodeInfo {
    code: "SNR-APP-022",
    severity: Severity::Error,
    summary: "Export failed",
    help: "Check that the folder is writable and the disk is not full, then export again.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A buffer is not a binary view this version reads (ADR-0017 §6).
pub const BINARY_VIEW: CodeInfo = CodeInfo {
    code: "SNR-APP-050",
    severity: Severity::Error,
    summary: "Not a binary view this version reads",
    help: "Build the user interface and the host from the same commit; a view's header version, \
           kind and data types must all be known to the reader.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A file the user chose cannot be used: missing, not a regular file, or not a data file.
pub const FILE_REFUSED: CodeInfo = CodeInfo {
    code: "SNR-APP-060",
    severity: Severity::Error,
    summary: "The file cannot be opened",
    help: "Choose an existing CSV, TSV, JSON, NDJSON, Parquet, Arrow, XLSX or ODS file.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Every `SNR-APP` code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::App,
    owner: "sonarithm-app",
    codes: &[
        PROTOCOL_MISMATCH,
        UNKNOWN_DATASET,
        NO_SPEC,
        NOT_COMPILED,
        TRANSPORT_VALUE,
        AUDIO_UNAVAILABLE,
        CONTROL_THREAD,
        COMPILE_FAILED,
        BAD_REQUEST,
        SETTINGS_WRITE,
        SETTINGS_RECOVERED,
        SETTINGS_NEWER,
        SETTINGS_VALUE,
        UNKNOWN_JOB,
        EXPORT_PATH,
        EXPORT_FAILED,
        BINARY_VIEW,
        FILE_REFUSED,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-APP code table");
