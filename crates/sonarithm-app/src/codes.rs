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

/// `project_save` without a path: the project was never saved.
pub const NO_PROJECT_PATH: CodeInfo = CodeInfo {
    code: "SNR-APP-030",
    severity: Severity::Error,
    summary: "The project has no file yet",
    help: "Use project_save_as to choose where to save it.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The project was made by a newer major format version (RFC-0002 §5).
pub const PROJECT_MAJOR: CodeInfo = CodeInfo {
    code: "SNR-APP-031",
    severity: Severity::Error,
    summary: "The project needs a newer SONARITHM",
    help: "Update SONARITHM to open this project; this version does not guess at newer formats.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The bundle is unsafe, oversized, altered or malformed (RFC-0002 §6).
pub const PROJECT_DAMAGED: CodeInfo = CodeInfo {
    code: "SNR-APP-032",
    severity: Severity::Error,
    summary: "The project cannot be opened safely",
    help: "The file is damaged, altered or not a SONARITHM project. Open a copy you trust, or \
           an autosave.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A project would store an absolute path (RFC-0002 §4).
pub const PROJECT_PATH: CodeInfo = CodeInfo {
    code: "SNR-APP-033",
    severity: Severity::Error,
    summary: "A project never stores an absolute path",
    help: "Open files as datasets and refer to them by ID, and embed data that is on another \
           drive than the project.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Linked data is missing, changed or not a data file (RFC-0002 §4).
pub const LINKED_DATA: CodeInfo = CodeInfo {
    code: "SNR-APP-034",
    severity: Severity::Warning,
    summary: "Linked data is unavailable",
    help: "Put the file back where the project expects it, unchanged, or open the data again \
           and save the project.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A newer minor format version, or members this version does not know (RFC-0002 §5).
pub const PROJECT_NEWER: CodeInfo = CodeInfo {
    code: "SNR-APP-035",
    severity: Severity::Warning,
    summary: "Parts of the project are unknown to this version",
    help: "The project opens; saving it with this version drops what it does not know.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A spec patch could not be applied; the spec is unchanged.
pub const SPEC_PATCH: CodeInfo = CodeInfo {
    code: "SNR-APP-040",
    severity: Severity::Error,
    summary: "The spec patch does not apply",
    help: "Check each operation's path against the current spec (spec_get); a patch applies \
           completely or not at all.",
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

/// No open stream has this ID (B06-T05).
pub const UNKNOWN_STREAM: CodeInfo = CodeInfo {
    code: "SNR-APP-070",
    severity: Severity::Error,
    summary: "No such stream",
    help: "List the open streams with stream_list; a stopped stream is gone.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A stream source cannot be offered: an unknown transport or a bad address.
pub const STREAM_SOURCE: CodeInfo = CodeInfo {
    code: "SNR-APP-071",
    severity: Severity::Error,
    summary: "The stream source is not valid",
    help: "Give a transport (websocket, osc, serial, mqtt or tail) and its address; a file to            follow is an absolute path in an existing folder.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A stream cannot connect: no offer has this token, or the stream could not start
/// (ADR-0012: every connection needs the user's approval of one offer).
pub const STREAM_CONNECT: CodeInfo = CodeInfo {
    code: "SNR-APP-072",
    severity: Severity::Error,
    summary: "The stream cannot connect",
    help: "Offer the stream again with stream_offer, show the user what it connects to, and            connect with the token only after the user approves.",
    rules: &["A11Y-16"],
    status: CodeStatus::Active,
};

/// The spec cannot play live: its source is not a stream, or its pace is a total
/// duration (B06-T07).
pub const LIVE_SPEC: CodeInfo = CodeInfo {
    code: "SNR-APP-073",
    severity: Severity::Error,
    summary: "The spec cannot play live",
    help: "Give the spec a stream source and a pace of secondsPerUnit or pointsPerSecond,            then start live playback again.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A live window did not compile; playback goes on with the last window that did.
pub const LIVE_COMPILE: CodeInfo = CodeInfo {
    code: "SNR-APP-074",
    severity: Severity::Warning,
    summary: "A live window did not compile",
    help: "Playback goes on with the last window that compiled. Check the stream's fields            against the spec.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A profile is not valid: not JSON, not a profile, a value out of range, or a path
/// (ADR-0020 §1).
pub const PROFILE_INVALID: CodeInfo = CodeInfo {
    code: "SNR-APP-080",
    severity: Severity::Error,
    summary: "The profile is not valid",
    help: "Use a profile exported from SONARITHM, or fix the value the message names.",
    rules: &["SDR-38"],
    status: CodeStatus::Active,
};

/// A profile of a newer major version (ADR-0020 §1).
pub const PROFILE_MAJOR: CodeInfo = CodeInfo {
    code: "SNR-APP-081",
    severity: Severity::Error,
    summary: "The profile needs a newer SONARITHM",
    help: "Update SONARITHM to use this profile; this version does not guess at newer formats.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A profile of a newer minor version: what this version does not know was left out.
pub const PROFILE_NEWER: CodeInfo = CodeInfo {
    code: "SNR-APP-082",
    severity: Severity::Warning,
    summary: "Parts of the profile are unknown to this version",
    help: "The profile works; saving it with this version drops what it does not know.",
    rules: &[],
    status: CodeStatus::Active,
};

/// No profile has this ID, or a preset was to be changed.
pub const UNKNOWN_PROFILE: CodeInfo = CodeInfo {
    code: "SNR-APP-083",
    severity: Severity::Error,
    summary: "No such profile",
    help: "List the profiles with profile_list. Presets cannot be changed: save a copy.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A chart correction set that is not valid (C02-T06, RFC-0005): a patch that breaks it,
/// or a correction file that does not read.
pub const CORRECTIONS_INVALID: CodeInfo = CodeInfo {
    code: "SNR-APP-084",
    severity: Severity::Error,
    summary: "The corrections are not valid",
    help: "A correction set follows RFC-0005: edits with IDs e1, e2, …, `before` on every edit            of an existing point, and `+e<n>` only for an earlier `add`. Nothing was changed.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Corrections that cannot change: the chart's file is not open (C02-T06).
pub const CORRECTIONS_READ_ONLY: CodeInfo = CodeInfo {
    code: "SNR-APP-085",
    severity: Severity::Error,
    summary: "These corrections cannot change now",
    help: "A project keeps a chart's corrections and their history, but not the chart file.            Open the chart file again to correct it; export works as it is.",
    rules: &[],
    status: CodeStatus::Active,
};

/// Corrections for a dataset that is not a chart, or a file made on another chart (C02-T06).
pub const CORRECTIONS_NOT_HERE: CodeInfo = CodeInfo {
    code: "SNR-APP-086",
    severity: Severity::Error,
    summary: "These corrections do not belong to this dataset",
    help: "Corrections apply to a dataset opened from a chart, and a correction file only to            the chart it was made on (same SHA-256 and extractor, RFC-0005 §3).",
    rules: &[],
    status: CodeStatus::Active,
};

/// A cursor row outside its dataset (B04-T06).
pub const CURSOR_ROW: CodeInfo = CodeInfo {
    code: "SNR-APP-087",
    severity: Severity::Error,
    summary: "The row is outside the dataset",
    help: "A cursor row counts from 0 and is below the dataset's row count (`rows` in its \
           summary). Nothing was changed.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A function's expression or variables are too large to read (C03-T08).
pub const FUNCTION_INPUT_TOO_LARGE: CodeInfo = CodeInfo {
    code: "SNR-APP-088",
    severity: Severity::Error,
    summary: "The function is too large to read",
    help: "An expression is at most 4096 bytes and names at most 64 variables before it is read; the expression language then allows 1024 characters and 16 variables (ADR-0028). Shorten it, or split it with variables.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A project with a function dataset cannot be saved yet (C03-T08).
pub const FUNCTION_NOT_SAVED: CodeInfo = CodeInfo {
    code: "SNR-APP-089",
    severity: Severity::Error,
    summary: "A function dataset cannot be saved in a project yet",
    help: "Project files do not store functions yet: saving the samples would lose the function, so nothing was saved. Close the function dataset to save the project, or keep the function's spec. The project format gains functions in a later release.",
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
        NO_PROJECT_PATH,
        PROJECT_MAJOR,
        PROJECT_DAMAGED,
        PROJECT_PATH,
        LINKED_DATA,
        PROJECT_NEWER,
        SPEC_PATCH,
        BINARY_VIEW,
        FILE_REFUSED,
        UNKNOWN_STREAM,
        STREAM_SOURCE,
        STREAM_CONNECT,
        LIVE_SPEC,
        LIVE_COMPILE,
        PROFILE_INVALID,
        PROFILE_MAJOR,
        PROFILE_NEWER,
        UNKNOWN_PROFILE,
        CORRECTIONS_INVALID,
        CORRECTIONS_READ_ONLY,
        CORRECTIONS_NOT_HERE,
        CURSOR_ROW,
        FUNCTION_INPUT_TOO_LARGE,
        FUNCTION_NOT_SAVED,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-APP code table");
