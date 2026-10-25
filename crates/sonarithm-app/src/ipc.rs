// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The IPC protocol v1 ([ADR-0017](../../docs/decisions/0017-ipc-protocol-v1.md)): every
//! request the UI can send, every reply and event it receives, and the typed error.
//!
//! A request is `{"command": "<name>", "args": {...}}` (no `args` for commands without
//! any); a reply is tagged by `reply` and an event by `event`. Field names are camelCase.
//! [`crate::App::dispatch`] is the only entry point; a host only moves this JSON.
//!
//! ```
//! use sonarithm_app::ipc::Request;
//!
//! let request: Request = serde_json::from_str(r#"{"command": "transport_seek", "args": {"seconds": 2.5}}"#)?;
//! assert_eq!(request, Request::TransportSeek { seconds: 2.5 });
//! let play: Request = serde_json::from_str(r#"{"command": "transport_play"}"#)?;
//! assert_eq!(play, Request::TransportPlay);
//! # Ok::<(), serde_json::Error>(())
//! ```

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sonarithm_core::codes::CodeInfo;
use sonarithm_core::{Diagnostic, Severity};
use ts_rs::TS;

use crate::settings::Settings;

/// The protocol version host and UI must share (ADR-0017 §5). Adding a command, an
/// optional field, a reply or an event keeps it; a breaking change increments it.
pub const PROTOCOL_VERSION: u32 = 1;

pub mod binary;
pub mod typescript;

/// Every command with the [`Reply`] variant it answers with, in [`Request`] order. The
/// TypeScript client's `ReplyFor<C>` is generated from it; the contract test checks it
/// against [`crate::App::dispatch`].
pub const COMMANDS: &[(&str, &str)] = &[
    ("hello", "hello"),
    ("dataset_open", "dataset"),
    ("dataset_close", "datasets"),
    ("dataset_list", "datasets"),
    ("spec_get", "spec"),
    ("spec_set", "spec"),
    ("spec_apply", "spec_patch"),
    ("spec_validate", "validation"),
    ("compile", "compile"),
    ("transport_state", "transport"),
    ("transport_play", "transport"),
    ("transport_pause", "transport"),
    ("transport_stop", "transport"),
    ("transport_seek", "transport"),
    ("transport_set_speed", "transport"),
    ("transport_set_loop", "transport"),
    ("stop_all", "transport"),
    ("export_render", "job"),
    ("job_status", "job"),
    ("job_cancel", "job"),
    ("job_list", "jobs"),
    ("history_undo", "history"),
    ("history_redo", "history"),
    ("settings_get", "settings"),
    ("settings_set", "settings"),
    ("devices_list", "devices"),
    ("device_select", "transport"),
    ("project_new", "project"),
    ("project_open", "project"),
    ("project_open_recent", "project"),
    ("project_save", "project"),
    ("project_save_as", "project"),
    ("project_recent", "recent"),
    ("project_recovery", "recovery"),
    ("project_recover", "project"),
    ("project_discard_recovery", "recovery"),
];

/// The commands whose reply is a binary view (ADR-0017 §6), sent with
/// [`crate::App::dispatch_binary`] and the transport's `binary`.
pub const BINARY_COMMANDS: &[&str] = &["view_get", "table_page"];

/// Every event kind with the channel that carries it ([`Event::channel`]).
pub const EVENTS: &[(&str, &str)] = &[
    ("clock", "clock"),
    ("job_progress", "jobs"),
    ("job_done", "jobs"),
    ("playback_finished", "app"),
    ("device_changed", "app"),
    ("audio_failed", "app"),
    ("settings_notice", "app"),
];

/// A command from the UI (09 §2.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "command",
    content = "args",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[non_exhaustive]
pub enum Request {
    /// The first request: checks the protocol version. Replies [`Reply::Hello`].
    Hello {
        /// The UI's [`PROTOCOL_VERSION`].
        protocol: u32,
    },
    /// Reads a data file and keeps it open in the session. Replies [`Reply::Dataset`].
    DatasetOpen {
        /// The file the user chose. Only its name is kept in summaries and logs.
        path: String,
    },
    /// Closes an open dataset. Replies [`Reply::Datasets`].
    DatasetClose {
        /// The dataset's ID, such as `d1`.
        id: String,
    },
    /// Lists the open datasets. Replies [`Reply::Datasets`].
    DatasetList,
    /// The session's spec. Replies [`Reply::Spec`].
    SpecGet,
    /// Replaces the session's spec text; undoable. Replies [`Reply::Spec`].
    SpecSet {
        /// The spec as JSON text.
        text: String,
    },
    /// Applies a JSON Patch (RFC 6902) to the session's spec, all or nothing, as one undoable
    /// edit. Replies [`Reply::SpecPatch`] with the validation diagnostics of the result.
    SpecApply {
        /// The operations, in order.
        patch: Vec<crate::patch::PatchOp>,
        /// A name for the edit in the history, such as `Set pitch range`.
        label: Option<String>,
    },
    /// Validates spec text without changing the session. Replies [`Reply::Validation`].
    SpecValidate {
        /// The spec as JSON text.
        text: String,
    },
    /// Compiles the session's spec against the open datasets. Replies [`Reply::Compile`].
    /// Compiling never starts playback (A11Y-15).
    Compile,
    /// The authoritative transport state. Replies [`Reply::Transport`].
    TransportState,
    /// Starts or resumes playback, opening the audio device on first use.
    TransportPlay,
    /// Pauses at the current position.
    TransportPause,
    /// Stops and returns to the start.
    TransportStop,
    /// Moves the playhead.
    TransportSeek {
        /// Seconds from the start of the Score, at most its duration.
        seconds: f64,
    },
    /// Changes the playback speed without changing pitch.
    TransportSetSpeed {
        /// The speed, 0.25 to 4.
        speed: f64,
    },
    /// Loops a range of the Score, or stops looping.
    TransportSetLoop {
        /// The range, or `null` to stop looping.
        range: Option<LoopSpan>,
    },
    /// Fades out every sound, cues included, and pauses: "stop all audio" (A11Y-15).
    StopAll,
    /// Renders the last compiled Score to a file in the background. Replies [`Reply::Job`].
    ExportRender {
        /// What to write.
        options: ExportRequest,
    },
    /// One job's state. Replies [`Reply::Job`].
    JobStatus {
        /// The job's ID.
        id: u64,
    },
    /// Cancels a job. Replies [`Reply::Job`].
    JobCancel {
        /// The job's ID.
        id: u64,
    },
    /// Every job of the session. Replies [`Reply::Jobs`].
    JobList,
    /// Undoes the last edit. Replies [`Reply::History`].
    HistoryUndo,
    /// Redoes the last undone edit. Replies [`Reply::History`].
    HistoryRedo,
    /// The settings. Replies [`Reply::Settings`].
    SettingsGet,
    /// Replaces the settings and saves them. Replies [`Reply::Settings`].
    SettingsSet {
        /// The new settings.
        settings: Settings,
    },
    /// The output devices. Replies [`Reply::Devices`].
    DevicesList,
    /// Chooses the output device (`null` for the system default) and saves it in the
    /// settings. Playback that was running pauses (sound-design §12 HS-05). Replies
    /// [`Reply::Transport`].
    DeviceSelect {
        /// The device's ID from `devices_list`, or `null`.
        id: Option<String>,
    },
    /// Starts an empty, unsaved project. Replies [`Reply::Project`].
    ProjectNew,
    /// Opens a `.sonarithm` project the user chose, replacing the session (RFC-0002).
    /// Replies [`Reply::Project`] with warnings for linked data that is unavailable.
    ProjectOpen {
        /// The project file.
        path: String,
    },
    /// Opens a project from the recent list by its position. Replies [`Reply::Project`].
    ProjectOpenRecent {
        /// The position in `project_recent`'s list.
        index: u32,
    },
    /// Saves the project to its file. Replies [`Reply::Project`].
    ProjectSave,
    /// Saves the project to a new file the user chose. Replies [`Reply::Project`].
    ProjectSaveAs {
        /// The project file, ending in `.sonarithm`.
        path: String,
        /// Embed every dataset; otherwise data opened from files is linked.
        embed_data: bool,
    },
    /// The recently opened or saved projects, by file name. Replies [`Reply::Recent`].
    ProjectRecent,
    /// Autosaves left by a session that did not end cleanly. Replies [`Reply::Recovery`].
    ProjectRecovery,
    /// Opens an autosave as an unsaved project. Replies [`Reply::Project`].
    ProjectRecover {
        /// The autosave's ID from `project_recovery`.
        id: u64,
    },
    /// Deletes every autosave. Replies [`Reply::Recovery`].
    ProjectDiscardRecovery,
    /// A downsampled view of two fields of a dataset for drawing (03 §7). Binary: a view of
    /// kind `m4Tile` or `lttbTile` with the columns `row`, `x` and `y` (ADR-0017 §6).
    ViewGet {
        /// The dataset's ID.
        dataset: String,
        /// The x field: numbers, dates or timestamps.
        x: String,
        /// The y field.
        y: String,
        /// The x range to draw; by default all of x.
        range: Option<ViewRange>,
        /// The plot's width in device pixels.
        px_width: u32,
        /// M4 for plots (pixel-exact, used when zooming), LTTB for overviews.
        method: ViewMethod,
    },
    /// A page of a dataset's rows for the data table. Binary: a `tablePage` view whose
    /// first column, `__snr_row`, holds each row's index.
    TablePage {
        /// The dataset's ID.
        dataset: String,
        /// The first row of the page, in sort order.
        offset: u64,
        /// The most rows, at most 10 000.
        limit: u32,
        /// The order; by default the table's.
        sort: Option<TableSort>,
    },
}

/// An x range for [`Request::ViewGet`], in the x field's units (seconds for times).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ViewRange {
    /// The left edge.
    pub start: f64,
    /// The right edge.
    pub end: f64,
}

/// How a view is downsampled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ViewMethod {
    /// Min, max, first and last per pixel column: pixel-exact lines (M4, VLDB 2014).
    M4,
    /// Largest-Triangle-Three-Buckets: `pxWidth` representative points.
    Lttb,
}

/// The order of a table page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TableSort {
    /// The field to sort by.
    pub field: String,
    /// Largest first.
    pub descending: bool,
}

impl Request {
    /// The command name on the wire, such as `transport_seek`.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hello { .. } => "hello",
            Self::DatasetOpen { .. } => "dataset_open",
            Self::DatasetClose { .. } => "dataset_close",
            Self::DatasetList => "dataset_list",
            Self::SpecGet => "spec_get",
            Self::SpecSet { .. } => "spec_set",
            Self::SpecApply { .. } => "spec_apply",
            Self::SpecValidate { .. } => "spec_validate",
            Self::Compile => "compile",
            Self::TransportState => "transport_state",
            Self::TransportPlay => "transport_play",
            Self::TransportPause => "transport_pause",
            Self::TransportStop => "transport_stop",
            Self::TransportSeek { .. } => "transport_seek",
            Self::TransportSetSpeed { .. } => "transport_set_speed",
            Self::TransportSetLoop { .. } => "transport_set_loop",
            Self::StopAll => "stop_all",
            Self::ExportRender { .. } => "export_render",
            Self::JobStatus { .. } => "job_status",
            Self::JobCancel { .. } => "job_cancel",
            Self::JobList => "job_list",
            Self::HistoryUndo => "history_undo",
            Self::HistoryRedo => "history_redo",
            Self::SettingsGet => "settings_get",
            Self::SettingsSet { .. } => "settings_set",
            Self::DevicesList => "devices_list",
            Self::DeviceSelect { .. } => "device_select",
            Self::ProjectNew => "project_new",
            Self::ProjectOpen { .. } => "project_open",
            Self::ProjectOpenRecent { .. } => "project_open_recent",
            Self::ProjectSave => "project_save",
            Self::ProjectSaveAs { .. } => "project_save_as",
            Self::ProjectRecent => "project_recent",
            Self::ProjectRecovery => "project_recovery",
            Self::ProjectRecover { .. } => "project_recover",
            Self::ProjectDiscardRecovery => "project_discard_recovery",
            Self::ViewGet { .. } => "view_get",
            Self::TablePage { .. } => "table_page",
        }
    }
}

/// A successful reply.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "reply",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[non_exhaustive]
pub enum Reply {
    /// The protocol versions agree.
    Hello {
        /// The host's [`PROTOCOL_VERSION`].
        protocol: u32,
        /// The SONARITHM version of the host.
        app_version: String,
    },
    /// One dataset.
    Dataset(DatasetSummary),
    /// The open datasets, in the order they were opened.
    Datasets {
        /// The datasets.
        datasets: Vec<DatasetSummary>,
    },
    /// The session's spec.
    Spec(SpecState),
    /// The spec after a patch.
    SpecPatch(SpecPatchResult),
    /// The diagnostics of spec text.
    Validation {
        /// Every diagnostic, in the validator's order.
        diagnostics: Vec<DiagnosticInfo>,
        /// Errors, warnings and infos.
        counts: Counts,
    },
    /// The result of a compile.
    Compile(CompileReport),
    /// The transport, after the command.
    Transport(TransportState),
    /// One job.
    Job(JobInfo),
    /// Every job.
    Jobs {
        /// The jobs, oldest first.
        jobs: Vec<JobInfo>,
    },
    /// The result of an undo or redo.
    History(HistoryState),
    /// The settings in effect.
    Settings(Settings),
    /// The output devices.
    Devices {
        /// The devices, the system default marked.
        devices: Vec<DeviceSummary>,
    },
    /// The project after opening or saving.
    Project(ProjectSummary),
    /// The recent projects.
    Recent {
        /// Most recent first.
        projects: Vec<RecentProject>,
    },
    /// Autosaves that can be recovered.
    Recovery {
        /// Newest first.
        autosaves: Vec<AutosaveInfo>,
    },
}

/// A command that could not run (ADR-0017 §3, 12 §1). Spec and data problems are not
/// errors: they come back as diagnostics in a [`CompileReport`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{code}: {message}")]
pub struct IpcError {
    /// The stable code, such as `SNR-APP-002`.
    pub code: String,
    /// The cause, in one or two sentences.
    pub message: String,
    /// How to fix it, if there is a known way.
    pub hint: Option<String>,
    /// A JSON pointer into the spec, when the error concerns one.
    pub pointer: Option<String>,
}

impl IpcError {
    /// An error with a registered code, its help as the hint.
    #[must_use]
    pub fn new(code: &CodeInfo, message: impl Into<String>) -> Self {
        Self {
            code: code.code.to_owned(),
            message: message.into(),
            hint: Some(code.help.to_owned()),
            pointer: None,
        }
    }

    /// An error from a diagnostic, such as a data layer failure.
    #[must_use]
    pub fn from_diagnostic(diagnostic: &Diagnostic) -> Self {
        Self {
            code: diagnostic.code.as_str().to_owned(),
            message: diagnostic.message.clone(),
            hint: None,
            pointer: diagnostic
                .location
                .as_ref()
                .map(|pointer| pointer.as_str().to_owned()),
        }
    }
}

/// An event pushed to the UI on one of the channels of ADR-0017 §4.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "event",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[non_exhaustive]
pub enum Event {
    /// A transport clock anchor: on every change and every 2 s while playing (06 §9).
    Clock(ClockAnchor),
    /// A job made progress (at most 10 per second per job).
    JobProgress {
        /// The job's ID.
        id: u64,
        /// Done so far, 0 to 1.
        progress: f32,
    },
    /// A job finished, failed or was cancelled.
    JobDone(JobInfo),
    /// Playback reached the end of the Score and stopped.
    PlaybackFinished {
        /// The Score's duration in seconds.
        duration_s: f64,
    },
    /// Playback moved to another device and is paused (HS-05). The UI announces it.
    DeviceChanged {
        /// The new device's name.
        device: String,
    },
    /// The audio device failed; playback is paused.
    AudioFailed {
        /// What happened.
        error: IpcError,
    },
    /// A notice about the settings, such as a recovery from a damaged file.
    SettingsNotice {
        /// The notice.
        error: IpcError,
    },
}

impl Event {
    /// The channel that carries the event.
    #[must_use]
    pub fn channel(&self) -> Channel {
        match self {
            Self::Clock(_) => Channel::Clock,
            Self::JobProgress { .. } | Self::JobDone(_) => Channel::Jobs,
            Self::PlaybackFinished { .. }
            | Self::DeviceChanged { .. }
            | Self::AudioFailed { .. }
            | Self::SettingsNotice { .. } => Channel::App,
        }
    }
}

/// The event channels of ADR-0017 §4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Channel {
    /// Transport clock anchors.
    Clock,
    /// Job progress and results.
    Jobs,
    /// Diagnostics outside a reply.
    Diagnostics,
    /// Text for the announcer (B03).
    Announcements,
    /// Rare app-wide notices.
    App,
    /// Binary meter frames (B01-T04).
    Meters,
}

/// An open dataset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DatasetSummary {
    /// The session's ID for it, such as `d1`; specs name it as a `dataset` source.
    pub id: String,
    /// The file name, without directories (P8).
    pub name: String,
    /// The number of rows in its first table.
    pub rows: u64,
    /// Its fields, in column order.
    pub fields: Vec<FieldSummary>,
    /// What the reader inferred or changed, as diagnostics.
    pub diagnostics: Vec<DiagnosticInfo>,
}

/// One field of a dataset, with its inferred semantics (03 §3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FieldSummary {
    /// The field name.
    pub name: String,
    /// The Arrow data type, such as `Float64`.
    pub data_type: String,
    /// The field type: `quantitative`, `temporal`, `ordinal` or `nominal`.
    pub field_type: Option<String>,
    /// The role, such as `x` or `y`.
    pub role: Option<String>,
    /// The unit, as written.
    pub unit: Option<String>,
    /// The concept, such as `temperature`.
    pub concept: Option<String>,
}

/// A diagnostic for the UI (05 §7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DiagnosticInfo {
    /// `error`, `warning` or `info`.
    pub severity: String,
    /// The stable code.
    pub code: String,
    /// The design rules involved.
    pub rules: Vec<String>,
    /// The cause and the fix.
    pub message: String,
    /// A JSON pointer into the spec.
    pub pointer: Option<String>,
}

impl From<&Diagnostic> for DiagnosticInfo {
    fn from(diagnostic: &Diagnostic) -> Self {
        let severity = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        Self {
            severity: severity.to_owned(),
            code: diagnostic.code.as_str().to_owned(),
            rules: diagnostic.rules.iter().map(ToString::to_string).collect(),
            message: diagnostic.message.clone(),
            pointer: diagnostic
                .location
                .as_ref()
                .map(|pointer| pointer.as_str().to_owned()),
        }
    }
}

/// The session's spec.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SpecState {
    /// The spec text, or `null` before one is set.
    pub text: Option<String>,
    /// The undo token of the edit that produced it, when the request was an edit.
    pub undo_token: Option<u64>,
}

/// The spec after an applied patch (09 §2.1 `SpecPatchResult`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SpecPatchResult {
    /// The edit's undo token.
    pub undo_token: u64,
    /// The spec text after the patch.
    pub text: String,
    /// The validation diagnostics of the patched spec (an invalid spec is kept, so editing
    /// can pass through invalid states).
    pub diagnostics: Vec<DiagnosticInfo>,
    /// Errors, warnings and infos.
    pub counts: Counts,
}

/// The result of a compile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CompileReport {
    /// Whether a Score was produced. When `false`, the previous Score stays loaded.
    pub ok: bool,
    /// Every diagnostic, applied defaults included.
    pub diagnostics: Vec<DiagnosticInfo>,
    /// Errors, warnings and infos.
    pub counts: Counts,
    /// The Score's facts, when one was produced.
    pub score: Option<ScoreInfo>,
}

/// Diagnostic counts by severity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Counts {
    /// Errors.
    pub errors: u32,
    /// Warnings.
    pub warnings: u32,
    /// Infos (applied defaults).
    pub info: u32,
}

/// Facts about a compiled Score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ScoreInfo {
    /// The length in seconds.
    pub duration_s: f64,
    /// The mode: `analysis`, `presentation` or `monitoring` (SDR-36).
    pub mode: String,
    /// SHA-256 of the canonical spec.
    pub spec_hash: String,
    /// The canonical Score hash.
    pub score_hash: Option<String>,
    /// One entry per track.
    pub tracks: Vec<TrackInfo>,
    /// The engines that contributed, with their versions.
    pub engines: BTreeMap<String, String>,
}

/// One track of a Score.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TrackInfo {
    /// The track ID.
    pub id: String,
    /// The instrument that plays it.
    pub instrument: String,
    /// Its events.
    pub events: u64,
    /// Notes from estimated data (03 §4).
    pub estimated: u64,
}

/// Whether the transport is playing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlaybackStatus {
    /// At the start, silent.
    #[default]
    Stopped,
    /// Sounding.
    Playing,
    /// Silent at a position.
    Paused,
}

/// A loop range in seconds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LoopSpan {
    /// Where the loop starts.
    pub start_s: f64,
    /// Where playback wraps to the start.
    pub end_s: f64,
}

/// The authoritative transport state (09 §3): the UI renders it and never assumes an intent
/// succeeded.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransportState {
    /// Stopped, playing or paused.
    pub status: PlaybackStatus,
    /// The playhead in seconds.
    pub position_s: f64,
    /// The Score's duration in seconds; 0 before a compile.
    pub duration_s: f64,
    /// The speed.
    pub speed: f64,
    /// The loop, if any.
    pub loop_range: Option<LoopSpan>,
    /// Whether a compiled Score is loaded.
    pub loaded: bool,
    /// The output device's name once the device is open.
    pub device: Option<String>,
}

/// A transport clock anchor (06 §9): the UI extrapolates the playhead from it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClockAnchor {
    /// The transport at the anchor.
    pub transport: TransportState,
    /// The device's playback time of the anchor's block in seconds, when the backend
    /// reports one.
    pub host_time_s: Option<f64>,
    /// Counts up with every anchor.
    pub seq: u64,
}

/// What to export.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExportRequest {
    /// The audio file the user chose: `.wav` or `.flac`. The manifest and cues go beside it.
    pub path: String,
    /// The sample rate in hertz; by default the spec's.
    #[serde(default)]
    #[ts(optional = nullable)]
    pub sample_rate: Option<u32>,
    /// Write the `WebVTT` cue sidecar (default yes).
    #[serde(default = "yes")]
    pub cues: bool,
    /// Keep the rendered level instead of normalizing loudness.
    #[serde(default)]
    pub preserve_level: bool,
}

impl ExportRequest {
    /// An export to `path` with every default.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            sample_rate: None,
            cues: true,
            preserve_level: false,
        }
    }
}

const fn yes() -> bool {
    true
}

/// What an export wrote.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExportResult {
    /// The audio file.
    pub audio: String,
    /// The render manifest.
    pub manifest: String,
    /// The cue sidecar, if written.
    pub cues: Option<String>,
    /// The length in seconds.
    pub duration_s: f64,
    /// The sample rate in hertz.
    pub sample_rate: u32,
    /// Integrated loudness in LUFS, `null` for silence.
    pub integrated_lufs: Option<f64>,
    /// True peak in dBTP.
    pub true_peak_dbtp: f64,
}

/// What kind of work a job does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum JobKind {
    /// An offline render to a file.
    Export,
}

/// Where a job is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum JobState {
    /// Waiting for the worker.
    Queued,
    /// Working.
    Running,
    /// Done; the result is set.
    Finished,
    /// Failed; the error is set.
    Failed,
    /// Cancelled before it finished.
    Cancelled,
}

impl JobState {
    /// Whether the job is over.
    #[must_use]
    pub const fn is_done(self) -> bool {
        matches!(self, Self::Finished | Self::Failed | Self::Cancelled)
    }
}

/// A job and its outcome.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct JobInfo {
    /// The job's ID, counting from 1.
    pub id: u64,
    /// What it does.
    pub kind: JobKind,
    /// Where it is.
    pub state: JobState,
    /// Done so far, 0 to 1.
    pub progress: f32,
    /// The export's result, once finished.
    pub export: Option<ExportResult>,
    /// Why it failed.
    pub error: Option<IpcError>,
}

/// The history after an undo or redo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct HistoryState {
    /// The token of the edit undone or redone; `null` when there was nothing to do.
    pub token: Option<u64>,
    /// The edit's label, such as `Set spec`.
    pub label: Option<String>,
    /// Whether another undo is possible.
    pub can_undo: bool,
    /// Whether a redo is possible.
    pub can_redo: bool,
    /// The spec after the step.
    pub spec: SpecState,
}

/// A project (RFC-0002 §9). Files are named alone, never with their folders (ADR-0017 §7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProjectSummary {
    /// The display name.
    pub name: String,
    /// The project file's name, once saved.
    pub file_name: Option<String>,
    /// Whether there are changes since the last save.
    pub unsaved: bool,
    /// The spec IDs, in order.
    pub specs: Vec<String>,
    /// The spec the session edits.
    pub active_spec: Option<String>,
    /// The datasets.
    pub datasets: Vec<ProjectDatasetInfo>,
    /// Warnings from opening: unavailable linked data, newer format parts.
    pub diagnostics: Vec<DiagnosticInfo>,
}

/// A dataset of a project.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProjectDatasetInfo {
    /// The ID specs name it by.
    pub id: String,
    /// The original file name.
    pub name: String,
    /// Linked to a file rather than embedded.
    pub linked: bool,
    /// Whether its data is loaded (linked data can be missing or changed).
    pub available: bool,
}

/// A recent project.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RecentProject {
    /// The position for `project_open_recent`.
    pub index: u32,
    /// The file name.
    pub name: String,
}

/// An autosave (RFC-0002 §8).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AutosaveInfo {
    /// The ID for `project_recover`.
    pub id: u64,
    /// When it was written: the file system's time, in seconds since 1970.
    pub modified_s: f64,
}

/// An output device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DeviceSummary {
    /// The ID for `device_select`.
    pub id: String,
    /// The name to show and announce.
    pub name: String,
    /// Whether it is the system default.
    pub is_default: bool,
    /// Whether SONARITHM is set to use it.
    pub selected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_event_travels_on_its_declared_channel() {
        let transport = TransportState {
            status: PlaybackStatus::Stopped,
            position_s: 0.0,
            duration_s: 1.0,
            speed: 1.0,
            loop_range: None,
            loaded: true,
            device: None,
        };
        let job = JobInfo {
            id: 1,
            kind: JobKind::Export,
            state: JobState::Finished,
            progress: 1.0,
            export: None,
            error: None,
        };
        let error = IpcError {
            code: "SNR-APP-011".into(),
            message: "m".into(),
            hint: None,
            pointer: None,
        };
        let events = [
            Event::Clock(ClockAnchor {
                transport,
                host_time_s: None,
                seq: 1,
            }),
            Event::JobProgress {
                id: 1,
                progress: 0.5,
            },
            Event::JobDone(job),
            Event::PlaybackFinished { duration_s: 1.0 },
            Event::DeviceChanged {
                device: "null".into(),
            },
            Event::AudioFailed {
                error: error.clone(),
            },
            Event::SettingsNotice { error },
        ];
        assert_eq!(events.len(), EVENTS.len());
        for (event, (name, channel)) in events.iter().zip(EVENTS) {
            let json = serde_json::to_value(event).unwrap();
            assert_eq!(json["event"], *name);
            assert_eq!(serde_json::to_value(event.channel()).unwrap(), *channel);
        }
    }
}
