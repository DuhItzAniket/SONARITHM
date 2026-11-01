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
pub mod meters;
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
    ("announce_repeat", "announce"),
    ("stream_offer", "stream_offer"),
    ("stream_connect", "streams"),
    ("stream_pause", "streams"),
    ("stream_resume", "streams"),
    ("stream_hide", "streams"),
    ("stream_stop", "streams"),
    ("stream_list", "streams"),
    ("live_start", "transport"),
    ("live_stop", "transport"),
    ("profile_list", "profiles"),
    ("profile_get", "profile"),
    ("profile_activate", "profiles"),
    ("profile_save", "profiles"),
    ("profile_import", "profiles"),
    ("profile_export", "profiles"),
    ("profile_flip_polarity", "compile"),
    ("settings_effective", "effective"),
    ("spec_quick", "spec"),
    ("corrections_get", "corrections"),
    ("corrections_apply", "corrections"),
    ("corrections_export", "corrections"),
    ("corrections_import", "corrections_import"),
    ("cursor_set", "cursor"),
    ("cursor_get", "cursor"),
    ("nav_move", "nav"),
    ("nav_where", "announce"),
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
    ("announcement", "announcements"),
    ("live_notice", "app"),
    ("cursor", "cursor"),
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
    /// Speaks the last announcement again (keyboard `nav.repeat`, R). During playback it
    /// pauses first, or ducks with "duck instead of pause" (keyboard §9.2). Replies
    /// [`Reply::Announce`].
    AnnounceRepeat,
    /// Describes a live stream source without opening it, for the consent prompt
    /// (ADR-0012, A11Y-16). Replies [`Reply::StreamOffer`] with a one-time token.
    StreamOffer {
        /// `websocket`, `osc`, `serial`, `mqtt` or `tail` (RFC-0001 stream sources).
        transport: String,
        /// The URL, for `websocket` and `mqtt`.
        #[serde(default)]
        url: Option<String>,
        /// The port, for `osc` and `serial`.
        #[serde(default)]
        port: Option<String>,
        /// The topic, for `mqtt`.
        #[serde(default)]
        topic: Option<String>,
        /// The file to follow, for `tail`: an absolute path the user chose.
        #[serde(default)]
        path: Option<String>,
    },
    /// Opens an offered stream. Send it only from the user's confirmation of that offer:
    /// one approval opens one connection. Replies [`Reply::Streams`].
    StreamConnect {
        /// The token from `stream_offer`.
        token: u64,
    },
    /// Pauses a stream: its data is discarded and its notices wait (A11Y-16). Replies
    /// [`Reply::Streams`].
    StreamPause {
        /// The stream's ID.
        id: String,
    },
    /// Resumes a paused or hidden stream. Replies [`Reply::Streams`].
    StreamResume {
        /// The stream's ID.
        id: String,
    },
    /// Hides a stream's notices: they go to the log only (A11Y-16). Replies
    /// [`Reply::Streams`].
    StreamHide {
        /// The stream's ID.
        id: String,
    },
    /// Stops and closes a stream. Replies [`Reply::Streams`].
    StreamStop {
        /// The stream's ID.
        id: String,
    },
    /// The open streams. Replies [`Reply::Streams`].
    StreamList,
    /// Plays an open stream live through the session's spec, whose source must be a
    /// stream (B06-T07). About once a second the newest window is compiled and swapped in;
    /// playback follows about 2 s behind the newest data and never stops at a window's
    /// end. Nothing sounds until `transport_play` (A11Y-15). Replies [`Reply::Transport`].
    LiveStart {
        /// The stream's ID.
        stream: String,
    },
    /// Stops following the stream; playback pauses on the last window. Replies
    /// [`Reply::Transport`].
    LiveStop,
    /// The profiles: the presets, the user's, and the project's (ADR-0020). Replies
    /// [`Reply::Profiles`].
    ProfileList,
    /// One profile. Replies [`Reply::Profile`].
    ProfileGet {
        /// Its ID from `profile_list`.
        id: String,
    },
    /// Makes a profile active, or none. Replies [`Reply::Profiles`].
    ProfileActivate {
        /// Its ID, or `None` for no profile.
        id: Option<String>,
    },
    /// Saves a user profile, or the project's (`id` `project`). Presets cannot be changed.
    /// Replies [`Reply::Profiles`].
    ProfileSave {
        /// The profile to replace, or `None` for a new one.
        #[serde(default)]
        id: Option<String>,
        /// The profile.
        profile: Box<crate::profile::Profile>,
    },
    /// Imports a profile file the user chose as a new user profile. Replies
    /// [`Reply::Profiles`] with any warnings.
    ProfileImport {
        /// The file, ending in `.json`.
        path: String,
    },
    /// Exports a profile to a file the user chose. Replies [`Reply::Profiles`].
    ProfileExport {
        /// The profile's ID.
        id: String,
        /// The file, ending in `.json`, in an existing folder.
        path: String,
    },
    /// The one-key polarity flip (SDR-15, FR-032): toggles `field` in the active profile
    /// (a copy, when a preset or nothing is active), saves it and recompiles. Replies
    /// [`Reply::Compile`].
    ProfileFlipPolarity {
        /// The field mapped to pitch.
        field: String,
    },
    /// Every effective setting with the layer it came from (12 §3). Replies
    /// [`Reply::Effective`].
    SettingsEffective,
    /// A quick spec over an open dataset: x along time, y as pitch, the marks the
    /// profile prefers (11 §1, SDR-38). An undoable edit. Replies [`Reply::Spec`].
    SpecQuick {
        /// The dataset's ID.
        dataset: String,
        /// The x field; by default the template's choice.
        #[serde(default)]
        x: Option<String>,
        /// The y field.
        y: String,
    },
    /// The corrections of a dataset opened from a chart (C02-T06, RFC-0005). Replies
    /// [`Reply::Corrections`].
    CorrectionsGet {
        /// The dataset's ID.
        dataset: String,
    },
    /// Applies a JSON Patch (RFC 6902) to a chart dataset's correction document, all or
    /// nothing, as one undoable edit; the dataset is re-extracted with the corrections
    /// (RFC-0005 §9). Replies [`Reply::Corrections`].
    CorrectionsApply {
        /// The dataset's ID.
        dataset: String,
        /// The operations on the correction document (RFC-0005 §2), in order.
        patch: Vec<crate::patch::PatchOp>,
        /// A name for the edit in the history, such as `Move Series 2#4 to 64`.
        label: Option<String>,
    },
    /// Writes a dataset's corrections to a correction file (RFC-0005 §5, §6, §11). The file
    /// holds no paths, names or times. Replies [`Reply::Corrections`].
    CorrectionsExport {
        /// The dataset's ID.
        dataset: String,
        /// The file, ending in `.json`, in an existing folder.
        path: String,
        /// Whether the person's notes go with the edits.
        #[serde(default = "yes")]
        notes: bool,
        /// Whether the chart's pixels go too (PNG charts only; off unless asked).
        #[serde(default)]
        include_image: bool,
    },
    /// Reads a correction file made on this chart and previews what it would do; with
    /// `confirm`, appends what applies as one undoable edit (RFC-0005 §11). Replies
    /// [`Reply::CorrectionsImport`].
    CorrectionsImport {
        /// The dataset's ID.
        dataset: String,
        /// The correction file (`.json`).
        path: String,
        /// Apply it, rather than only preview it.
        #[serde(default)]
        confirm: bool,
    },
    /// Moves the one cursor to a row of a dataset (B04-T06, ADR-0017 "Cursor"). Replies
    /// [`Reply::Cursor`] and sends a `cursor` event, which every view follows.
    CursorSet {
        /// The dataset's ID.
        dataset: String,
        /// The zero-based row in the dataset's first table: a table page's `__snr_row`.
        row: u64,
        /// Where the move came from; it decides what is spoken.
        origin: CursorOrigin,
    },
    /// Where the cursor is. Replies [`Reply::Cursor`].
    CursorGet,
    /// Moves the one cursor with a chart explorer verb (B03-T09, ADR-0017 "Navigation";
    /// keyboard-and-navigation §4.2). Replies [`Reply::Nav`], sends a `cursor` event with
    /// origin `nav` and announces the outcome.
    NavMove {
        /// The verb.
        verb: NavVerb,
    },
    /// Announces where the cursor is: every level with its position, the neighbours and the
    /// excluded series (keyboard §6, A11Y-07). The cursor does not move. Replies
    /// [`Reply::Announce`].
    NavWhere,
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
            Self::AnnounceRepeat => "announce_repeat",
            Self::StreamOffer { .. } => "stream_offer",
            Self::StreamConnect { .. } => "stream_connect",
            Self::StreamPause { .. } => "stream_pause",
            Self::StreamResume { .. } => "stream_resume",
            Self::StreamHide { .. } => "stream_hide",
            Self::StreamStop { .. } => "stream_stop",
            Self::StreamList => "stream_list",
            Self::LiveStart { .. } => "live_start",
            Self::LiveStop => "live_stop",
            Self::ProfileList => "profile_list",
            Self::ProfileGet { .. } => "profile_get",
            Self::ProfileActivate { .. } => "profile_activate",
            Self::ProfileSave { .. } => "profile_save",
            Self::ProfileImport { .. } => "profile_import",
            Self::ProfileExport { .. } => "profile_export",
            Self::ProfileFlipPolarity { .. } => "profile_flip_polarity",
            Self::SettingsEffective => "settings_effective",
            Self::SpecQuick { .. } => "spec_quick",
            Self::CorrectionsGet { .. } => "corrections_get",
            Self::CorrectionsApply { .. } => "corrections_apply",
            Self::CorrectionsExport { .. } => "corrections_export",
            Self::CorrectionsImport { .. } => "corrections_import",
            Self::CursorSet { .. } => "cursor_set",
            Self::CursorGet => "cursor_get",
            Self::NavMove { .. } => "nav_move",
            Self::NavWhere => "nav_where",
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
    /// The one cursor (B04-T06).
    Cursor(CursorState),
    /// What a chart explorer verb did (B03-T09).
    Nav(NavResult),
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
    /// Whether an announcement was queued.
    Announce {
        /// `false` when there was nothing to repeat.
        queued: bool,
    },
    /// A stream source, described for the consent prompt.
    StreamOffer(StreamOfferInfo),
    /// The open streams.
    Streams {
        /// In the order they were opened.
        streams: Vec<StreamInfo>,
    },
    /// The profiles.
    Profiles {
        /// Presets first, then the user's by ID, then the project's.
        profiles: Vec<ProfileInfo>,
        /// Warnings from the last import or activation.
        warnings: Vec<DiagnosticInfo>,
    },
    /// One profile.
    Profile {
        /// Its ID.
        id: String,
        /// The profile.
        profile: crate::profile::Profile,
    },
    /// The effective settings.
    Effective {
        /// Each setting with its value and layer.
        settings: Vec<crate::profile::EffectiveSetting>,
    },
    /// A chart dataset's corrections.
    Corrections(CorrectionsState),
    /// What a correction file would do, or did.
    CorrectionsImport(ImportPreview),
}

/// A chart dataset's corrections (C02-T06, RFC-0005).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CorrectionsState {
    /// The dataset's ID.
    pub dataset: String,
    /// The correction document, as a correction file holds it (RFC-0005 §2).
    pub text: String,
    /// The number of edits.
    pub edits: usize,
    /// The IDs of edits that did not apply to the current extraction (`SNR-ENG-386`).
    pub not_applied: Vec<String>,
    /// Whether corrections can change: the chart's file is open. A reopened project keeps
    /// them read-only until the chart is opened again.
    pub editable: bool,
    /// The undo token of the edit just made, if any.
    pub undo_token: Option<u64>,
    /// The dataset after the corrections.
    pub summary: DatasetSummary,
}

/// What importing a correction file does (RFC-0005 §11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImportPreview {
    /// The dataset's ID.
    pub dataset: String,
    /// The file's edits that apply, by their IDs in the file.
    pub applies: Vec<String>,
    /// The file's edits whose points the session already changed: listed, never applied.
    pub conflicts: Vec<String>,
    /// The file's edits the session already has: skipped.
    pub duplicates: Vec<String>,
    /// Whether the file's calibration differs from the session's (it is not taken).
    pub calibration_differs: bool,
    /// Whether the edits were applied (`confirm`).
    pub applied: bool,
    /// The corrections after the import, when applied.
    pub state: Option<CorrectionsState>,
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
    /// Text for the announcer (08 §4), in the order it may be spoken.
    Announcement(AnnouncementInfo),
    /// A live window did not compile (`SNR-APP-074`); playback goes on with the last one
    /// that did. Sent once per distinct problem.
    LiveNotice {
        /// What happened.
        error: IpcError,
    },
    /// The one cursor moved, or the data under it changed (B04-T06, ADR-0017 "Cursor").
    Cursor(CursorState),
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
            | Self::SettingsNotice { .. }
            | Self::LiveNotice { .. } => Channel::App,
            Self::Announcement(_) => Channel::Announcements,
            Self::Cursor(_) => Channel::Cursor,
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
    /// The one cursor (B04-T06).
    Cursor,
}

/// An announcement, sequenced with sonification (B03-T07). The host sends `speech` to the
/// screen reader unless SONARITHM already spoke it (`selfVoiced`), and shows `display`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AnnouncementInfo {
    /// Its order.
    pub seq: u64,
    /// The text to speak, with spoken unit names.
    pub speech: String,
    /// The text to show and to send to braille, with unit symbols.
    pub display: String,
    /// Its class (keyboard §9.1).
    pub class: AnnouncementClass,
    /// Errors, device loss and alarms only (08 §4); everything else is polite.
    pub assertive: bool,
    /// It cuts off speech in progress (navigation, responses, earcon text).
    pub interrupt: bool,
    /// SONARITHM spoke it through self-voicing: show it, but do not send it to the
    /// screen reader.
    pub self_voiced: bool,
    /// The navigation node it describes, if any.
    pub node: Option<u32>,
}

/// Where a cursor move came from (B04-T06, ADR-0017 "Cursor"). It decides what is spoken:
/// a move from the table is not announced, because the screen reader already reads the
/// focused row (INFER, OD-30); plot and explorer moves are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum CursorOrigin {
    /// The data table's focused row.
    Table,
    /// A click or a key on the plot.
    Plot,
    /// The chart explorer's movement verbs (B03-T09).
    Nav,
}

/// A level of the cursor's path, which the UI names through its own catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum PathLevel {
    /// The dataset.
    Dataset,
    /// A series (a data track).
    Series,
    /// A segment of a series.
    Segment,
    /// A point.
    Point,
    /// The axes group (B03-T09).
    Axes,
    /// One axis.
    Axis,
    /// The insights group.
    Insights,
    /// A group of insights of one kind.
    InsightGroup,
    /// One insight.
    Insight,
    /// The filters group.
    Filters,
    /// The filter of one series.
    Filter,
}

/// One step of the cursor's path, from the dataset down (the breadcrumb).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CursorPathItem {
    /// The level.
    pub level: PathLevel,
    /// A data name, shown as it is: the dataset's file name, a series' track or field, a
    /// segment's span or a point's x.
    pub label: String,
}

/// The one cursor (B04-T06; ADR-0017 "Cursor"): Rust's, the only one (A11Y-10). The table,
/// the plot, the breadcrumb and the readout show it, and the chart explorer's movement
/// verbs (B03-T09) read and move this same cursor.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CursorState {
    /// Its order: higher is newer.
    pub seq: u64,
    /// The dataset's ID; `None` when there is no cursor.
    pub dataset: Option<String>,
    /// The zero-based row.
    pub row: Option<u64>,
    /// The point's x value when the row is a point of the compiled Score.
    pub x: Option<f64>,
    /// The cursor's navigation node: the row's point, or, after an explorer move, any
    /// node (a series, a segment, an axis, an insight, a filter, ADR-0017 "Navigation").
    pub node: Option<u32>,
    /// From the dataset down to the node; only the dataset without a compiled Score.
    pub path: Vec<CursorPathItem>,
    /// The series the explorer's filters took out of playback and view, in node order.
    /// Exports still render every series: the filter is session state (B03-T09).
    pub excluded: Vec<SeriesRef>,
    /// Where the move came from; `None` when Rust moved it (data changed under it).
    pub origin: Option<CursorOrigin>,
}

/// A series named for the UI: its navigation node and its data name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SeriesRef {
    /// The series' navigation node.
    pub node: u32,
    /// Its track or field name, shown as it is.
    pub label: String,
}

/// A chart explorer verb (B03-T09, ADR-0017 "Navigation"): the AUDAXIS verbs one to one,
/// with their keys in keyboard-and-navigation §4.2.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "verb",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[non_exhaustive]
pub enum NavVerb {
    /// The next node at this level (Right).
    Next,
    /// The previous node at this level (Left).
    Previous,
    /// Forward by 10 % of the nodes at this level (Jump+Right).
    JumpForward,
    /// Back by 10 % (Jump+Left).
    JumpBack,
    /// The first node at this level (Home).
    First,
    /// The last node at this level (End).
    Last,
    /// Into the child at the current x (Down).
    Into,
    /// Out to the parent (Up, Backspace).
    Out,
    /// Enter: a point's full description, a filter's toggle, else Into.
    Activate,
    /// The next series at the same x (`PageDown`).
    NextSeries,
    /// The previous series (`PageUp`).
    PreviousSeries,
    /// The current series' maximum (Jump+Up).
    Maximum,
    /// The current series' minimum (Jump+Down).
    Minimum,
    /// The next insight of the series (N).
    NextInsight,
    /// The previous insight (Shift+N).
    PreviousInsight,
    /// A landmark (X, Y, I, F, D).
    Landmark {
        /// Which.
        landmark: NavLandmark,
    },
    /// The point nearest an x value (G).
    GoTo {
        /// The x value.
        x: NavX,
    },
    /// The next point equal to or crossing a value (Primary+F in the explorer).
    FindValue {
        /// The value.
        value: f64,
    },
}

/// A landmark of [`NavVerb::Landmark`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum NavLandmark {
    /// The x axis (X).
    XAxis,
    /// The first value axis (Y).
    YAxis,
    /// The insights group (I).
    Insights,
    /// The filters group (F).
    Filters,
    /// The dataset root (D).
    Dataset,
}

/// Where [`NavVerb::GoTo`] goes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum NavX {
    /// A number, a date (days since 1970-01-01) or an instant (seconds since the epoch).
    Number(f64),
    /// A category label, compared without case.
    Category(String),
}

/// What kind of thing a verb did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum NavOutcome {
    /// The cursor moved.
    Moved,
    /// The cursor stayed: the move would leave the structure.
    Boundary,
    /// The cursor stayed: the verb does not apply here.
    Invalid,
    /// Enter on a point: its full description was spoken; the cursor stayed.
    Describe,
    /// Enter on a filter: its series is now in or out of playback and view.
    Toggled,
}

/// The result of `nav_move` (B03-T09; 09 §3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NavResult {
    /// The cursor after the verb, as the `cursor` event sent with it.
    pub cursor: CursorState,
    /// What the verb did.
    pub outcome: NavOutcome,
    /// For a boundary: `first`, `last`, `top` or `bottom` (keyboard §11).
    pub edge: Option<String>,
    /// For an invalid verb, why, such as `noInsights` or `notHere`.
    pub reason: Option<String>,
    /// For a toggle: whether the series is now included.
    pub included: Option<bool>,
    /// The earcon of the outcome, such as `E-BOUNDARY` (A11Y-08).
    pub earcon: Option<String>,
    /// The x span to highlight: the node's, when it has one.
    pub highlight: Option<NavSpan>,
    /// Whether the outcome was announced (on the `announcements` channel, never here).
    pub announced: bool,
}

/// An x span, inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NavSpan {
    /// The first x.
    pub start: f64,
    /// The last x.
    pub end: f64,
}

/// A stream source described for the consent prompt; nothing is open yet. Files are named
/// alone, never with their folders (ADR-0017 §7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StreamOfferInfo {
    /// The one-time token for `stream_connect`.
    pub token: u64,
    /// The transport: `websocket`, `osc`, `serial`, `mqtt` or `tail`.
    pub transport: String,
    /// What it connects to: a URL, a port, or a file name.
    pub target: String,
    /// The MQTT topic, if any.
    pub topic: Option<String>,
}

/// A profile in the list (ADR-0020).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProfileInfo {
    /// Its ID: `preset:…`, a user profile's, or `project`.
    pub id: String,
    /// Its name.
    pub name: String,
    /// A built-in preset: read-only, and a proposal until co-design (B07).
    pub preset: bool,
    /// Whether it is the active profile.
    pub active: bool,
}

/// An open stream (A11Y-16).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StreamInfo {
    /// The stream's ID, such as `s1`.
    pub id: String,
    /// The transport.
    pub transport: String,
    /// What it connects to: a URL, a port, or a file name.
    pub target: String,
    /// The connection.
    pub connection: StreamConnection,
    /// Whether its notices are spoken, waiting or logged only.
    pub notices: StreamNotices,
    /// Messages received.
    pub received: u64,
    /// Messages dropped because they did not fit the schema.
    pub invalid: u64,
}

/// Where a stream's connection is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum StreamConnection {
    /// Opening for the first time.
    Connecting,
    /// Receiving.
    Connected,
    /// Connected, discarding data until resumed.
    Paused,
    /// Lost; trying again.
    Reconnecting,
    /// Stopped.
    Stopped,
}

/// What happens to a stream's notices (A11Y-16).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum StreamNotices {
    /// Spoken, at most one every 5 s, summed up.
    Active,
    /// Waiting until the stream resumes; stale ones are dropped.
    Paused,
    /// Logged only.
    Hidden,
}

/// The class of an announcement (keyboard §9.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum AnnouncementClass {
    /// A cursor move.
    Navigation,
    /// The answer to a command: where am I, repeat, the value on pause.
    Response,
    /// A status change or an error.
    Status,
    /// A stream notice.
    Stream,
    /// The text of an earcon.
    EarconText,
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
    /// Whether playback follows a live stream (`live_start`).
    pub live: bool,
    /// Underruns the output reported since the session started (PB-02, PB-11).
    pub xruns: u64,
    /// The compiled Score's tracks that playback leaves out, because the explorer's filters
    /// took their series out (B03-T09, ADR-0017 "Navigation"). Empty when the transport
    /// plays the compiled Score as it is; exports always render it whole.
    pub excluded_tracks: Vec<String>,
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
    /// How far the heard sound lags the engine's position, in seconds (B04-T04, 06 §9): the
    /// UI's playhead shows `positionS − outputLatencyS × speed`. In order of precedence:
    /// the device's calibrated latency where one is stored (ADR-0016, per-device latency
    /// calibration: `audio.calibratedLatencyS` in the settings), then the backend's reported
    /// latency, then `None`, which the UI takes as "no correction", never as a guess. It is
    /// computed on the control thread when the anchor is built, never in the audio
    /// callback. Added in protocol 1 (additive, approved by the coordinator on 2026-10-30).
    pub output_latency_s: Option<f64>,
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
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
    /// The corrections after the step, when it undid or redid a correction (C02-T06).
    #[serde(default)]
    pub corrections: Option<CorrectionsState>,
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
            live: false,
            xruns: 0,
            excluded_tracks: Vec::new(),
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
                output_latency_s: None,
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
            Event::SettingsNotice {
                error: error.clone(),
            },
            Event::Announcement(AnnouncementInfo {
                seq: 1,
                speech: "Paused. 12.5".into(),
                display: "Paused. 12.5".into(),
                class: AnnouncementClass::Response,
                assertive: false,
                interrupt: true,
                self_voiced: false,
                node: Some(7),
            }),
            Event::LiveNotice { error },
            Event::Cursor(CursorState::default()),
        ];
        assert_eq!(events.len(), EVENTS.len());
        for (event, (name, channel)) in events.iter().zip(EVENTS) {
            let json = serde_json::to_value(event).unwrap();
            assert_eq!(json["event"], *name);
            assert_eq!(serde_json::to_value(event.channel()).unwrap(), *channel);
        }
    }
}
