// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! [`App`]: the session and the command bus behind the IPC protocol.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use sonarithm::{Compiled, DataBundle, Datasets, Normalization, RenderOptions, Sonarithm, Spec};
use sonarithm_audaxis::announce::{Announcement, Composer};
use sonarithm_core::TableId;
use sonarithm_core::nav::{NavKind, NavNodeId, NavTree};
use sonarithm_data::{BundleId, ReadOptions, Table, TileCache, read_file};
use sonarithm_i18n::Localizer;
use sonarithm_speech::WordsPerMinute;

use crate::codes;
use crate::events::EventSink;
use crate::history::History;
use crate::ipc::StreamOfferInfo;
use crate::ipc::{
    CompileReport, Counts, DatasetSummary, DeviceSummary, DiagnosticInfo, Event, ExportRequest,
    ExportResult, FieldSummary, HistoryState, IpcError, JobKind, PROTOCOL_VERSION, PlaybackStatus,
    Reply, Request, ScoreInfo, SpecPatchResult, SpecState, TrackInfo,
};
use crate::jobs::{Failure, JobQueue, Output, Work};
use crate::settings::{Settings, SettingsStore};
use crate::speech::{SequencerConfig, SpeakerSource};
use crate::streams::{self, StreamAction};
use crate::transport::{Action, AudioOutput, OutputConfig, Transport};

mod corrections;
mod cursor;
mod functions;
mod nav;
mod profiles;
mod projects;

/// How the app core starts.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AppConfig {
    /// The folder for settings: the OS app-config folder (09 §5).
    pub config_dir: PathBuf,
    /// The audio output.
    pub audio: AudioOutput,
    /// The folder for autosaves: the OS app-data folder (09 §5); by default `config_dir`.
    pub data_dir: Option<PathBuf>,
    /// Makes the self-voicing speaker, if this host offers self-voicing (A11Y-19).
    pub speaker: Option<SpeakerSource>,
}

impl AppConfig {
    /// Settings in `config_dir`, audio on the system's devices.
    #[must_use]
    pub fn new(config_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: config_dir.into(),
            audio: AudioOutput::System,
            data_dir: None,
            speaker: None,
        }
    }

    /// The same configuration with optional self-voicing through `speaker` (B03-T07).
    #[must_use]
    pub fn with_speaker(mut self, speaker: SpeakerSource) -> Self {
        self.speaker = Some(speaker);
        self
    }

    /// The same configuration with autosaves under `data_dir`.
    #[must_use]
    pub fn with_data_dir(mut self, data_dir: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(data_dir.into());
        self
    }

    /// The same configuration with another audio output, such as [`AudioOutput::null`].
    #[must_use]
    pub fn with_audio(mut self, audio: AudioOutput) -> Self {
        self.audio = audio;
        self
    }
}

/// An open dataset.
struct Open {
    summary: DatasetSummary,
    bundle: DataBundle,
    /// The canonical file it was read from, if any: what a saved project links.
    origin: Option<PathBuf>,
    /// For a dataset read from a chart: its corrections (C02-T06).
    chart: Option<corrections::ChartState>,
    /// For a dataset sampled from a function: its definition (C03-T08). Projects cannot
    /// save it yet (`SNR-APP-089`).
    function: Option<sonarithm::FunctionSpec>,
}

/// What an undoable edit changed: the spec, or one chart dataset's corrections (C02-T06).
#[derive(Clone, Debug, PartialEq, Eq)]
enum Snapshot {
    /// The spec text.
    Spec(Option<String>),
    /// A chart dataset's correction document (RFC-0005).
    Corrections { dataset: u64, text: String },
}

/// The headless app core: one session (open datasets, the spec, the last compiled Score
/// and its undo history), the transport, the job queue and the settings.
///
/// Every operation is a [`Request`] through [`App::dispatch`] (ADR-0017). `App` is `Send`;
/// a host keeps it behind one lock. Its threads report through the [`EventSink`].
pub struct App {
    snr: Sonarithm,
    /// Formats announcements; `None` only if the embedded catalogue is broken.
    localizer: Option<Localizer>,
    /// Interaction choices made in this session: the explicit-UI-action layer (12 §3).
    session_prefs: crate::profile::InteractionPrefs,
    /// Stream sources described to the user and not yet approved, by token.
    offers: BTreeMap<u64, sonarithm_stream::Endpoint>,
    next_offer: u64,
    next_stream: u64,
    sink: Arc<dyn EventSink>,
    store: SettingsStore,
    settings: Settings,
    datasets: BTreeMap<u64, Open>,
    next_dataset: u64,
    spec: Option<String>,
    compiled: Option<Compiled>,
    history: History<Snapshot>,
    tiles: TileCache,
    config_dir: PathBuf,
    data_dir: PathBuf,
    project: projects::ProjectState,
    autosaves: crate::project::Autosaves,
    /// The live worker while playback follows a stream (B06-T07).
    live: Option<crate::live::LiveWorker>,
    next_live: u64,
    /// The one cursor (B04-T06).
    cursor: cursor::CursorModel,
    // Dropped in this order: the live worker and jobs stop before the transport.
    jobs: JobQueue,
    transport: Transport,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("datasets", &self.datasets.len())
            .field("spec", &self.spec.is_some())
            .field("compiled", &self.compiled.is_some())
            .finish_non_exhaustive()
    }
}

impl App {
    /// Starts the app core: reads (and if needed migrates) the settings and starts the
    /// control and job threads. No audio device opens until the first play (09 §7).
    /// Settings notices go to `sink` as [`Event::SettingsNotice`].
    ///
    /// # Errors
    ///
    /// `SNR-APP-007` if a thread cannot start.
    pub fn new(config: AppConfig, sink: Arc<dyn EventSink>) -> Result<Self, IpcError> {
        let AppConfig {
            config_dir,
            audio,
            data_dir,
            speaker,
        } = config;
        let data_dir = data_dir.unwrap_or_else(|| config_dir.clone());
        let autosaves = crate::project::Autosaves::new(data_dir.join("autosave"));
        let mut store = SettingsStore::new(config_dir.clone());
        let loaded = store.load();
        for error in loaded.notices {
            sink.emit(Event::SettingsNotice { error });
        }
        let settings = loaded.settings;
        let transport = Transport::spawn(audio, output(&settings), Arc::clone(&sink), speaker)?;
        let jobs = JobQueue::new(Arc::clone(&sink));
        // AUDAXIS builds the navigation tree that announcements describe (B03).
        let snr = Sonarithm::builder()
            .with_default_engines()
            .with_navigation()
            .build()
            .unwrap_or_else(|_| Sonarithm::new());
        let app = Self {
            snr,
            localizer: Localizer::new(LOCALE).ok(),
            session_prefs: crate::profile::InteractionPrefs::default(),
            offers: BTreeMap::new(),
            next_offer: 1,
            next_stream: 1,
            sink,
            store,
            settings,
            datasets: BTreeMap::new(),
            next_dataset: 1,
            spec: None,
            compiled: None,
            history: History::default(),
            tiles: TileCache::new(crate::views::TILE_CACHE),
            config_dir,
            data_dir,
            project: projects::ProjectState::new(),
            autosaves,
            live: None,
            next_live: 1,
            cursor: cursor::CursorModel::default(),
            jobs,
            transport,
        };
        // The active profile's speech preferences (ADR-0020 §3).
        app.transport.configure(app.output_config())?;
        Ok(app)
    }

    /// Ends the session cleanly: the autosaves go (RFC-0002 §8). Dropping the app without
    /// this is what a crash looks like, and the next start offers the autosaves.
    pub fn shutdown(self) {
        self.autosaves.discard();
    }

    /// Runs one command (the command bus). Every session, transport, export, history,
    /// settings and device operation goes through here.
    ///
    /// # Errors
    ///
    /// [`IpcError`] when the command cannot run; spec and data problems come back as
    /// diagnostics in the reply instead.
    pub fn dispatch(&mut self, request: Request) -> Result<Reply, IpcError> {
        let reply = self.run(request);
        // The cursor follows the data under it after every command (B04-T06).
        if reply.is_ok() {
            self.sync_cursor();
        }
        reply
    }

    fn run(&mut self, request: Request) -> Result<Reply, IpcError> {
        match request {
            Request::Hello { protocol } => hello(protocol),
            Request::DatasetOpen { path } => self.edit(|app| app.dataset_open(Path::new(&path))),
            Request::DatasetClose { id } => self.edit(|app| app.dataset_close(&id)),
            Request::DatasetList => Ok(self.datasets()),
            Request::SpecGet => Ok(Reply::Spec(SpecState {
                text: self.spec.clone(),
                undo_token: None,
            })),
            Request::SpecSet { text } => self.edit(|app| app.spec_set(text)),
            Request::SpecApply { patch, label } => self.edit(|app| app.spec_apply(&patch, label)),
            Request::SpecValidate { text } => Ok(validation(&text)),
            Request::Compile => self.compile(),
            Request::TransportState => self.act(Action::State),
            Request::TransportPlay => self.act(Action::Play),
            Request::TransportPause => self.pause(),
            Request::TransportStop => self.act(Action::Stop),
            Request::TransportSeek { seconds } => self.act(Action::Seek(seconds)),
            Request::TransportSetSpeed { speed } => self.act(Action::Speed(speed)),
            Request::TransportSetLoop { range } => self.act(Action::Loop(range)),
            Request::StopAll => self.act(Action::StopAll),
            Request::ExportRender { options } => self.export(&options),
            Request::JobStatus { id } => self.jobs.status(id).map(Reply::Job),
            Request::JobCancel { id } => self.jobs.cancel(id, &*self.sink).map(Reply::Job),
            Request::JobList => Ok(Reply::Jobs {
                jobs: self.jobs.list(),
            }),
            Request::HistoryUndo => self.edit(|app| Ok(app.step(true))),
            Request::HistoryRedo => self.edit(|app| Ok(app.step(false))),
            Request::SettingsGet => Ok(Reply::Settings(self.settings.clone())),
            Request::SettingsSet { settings } => self.settings_set(settings),
            Request::DevicesList => self.devices(),
            Request::DeviceSelect { id } => self.device_select(id),
            Request::AnnounceRepeat => Ok(Reply::Announce {
                queued: self.transport.repeat()?,
            }),
            Request::StreamOffer {
                transport,
                url,
                port,
                topic,
                path,
            } => self.stream_offer(
                &transport,
                url.as_deref(),
                port.as_deref(),
                topic.as_deref(),
                path.as_deref(),
            ),
            Request::StreamConnect { token } => self.stream_connect(token),
            Request::StreamPause { id } => self.stream_act(id, StreamAction::Pause),
            Request::StreamResume { id } => self.stream_act(id, StreamAction::Resume),
            Request::StreamHide { id } => self.stream_act(id, StreamAction::Hide),
            Request::StreamStop { id } => self.stream_act(id, StreamAction::Stop),
            Request::StreamList => Ok(Reply::Streams {
                streams: self.transport.stream_list()?,
            }),
            Request::LiveStart { stream } => self.live_start(stream),
            Request::LiveStop => {
                self.live = None;
                Ok(Reply::Transport(self.transport.live(None)?))
            }
            Request::ProfileList => Ok(self.profile_list(&[])),
            Request::ProfileGet { id } => self.profile_get(&id),
            Request::ProfileActivate { id } => self.profile_activate(id),
            Request::ProfileSave { id, profile } => self.profile_save(id.as_deref(), &profile),
            Request::ProfileImport { path } => self.profile_import(&path),
            Request::ProfileExport { id, path } => self.profile_export(&id, &path),
            Request::ProfileFlipPolarity { field } => self.profile_flip_polarity(&field),
            Request::SettingsEffective => self.settings_effective(),
            Request::SpecQuick { dataset, x, y } => self.spec_quick(&dataset, x, y),
            request @ (Request::CorrectionsGet { .. }
            | Request::CorrectionsApply { .. }
            | Request::CorrectionsExport { .. }
            | Request::CorrectionsImport { .. }) => self.corrections_request(request),
            Request::ProjectNew => Ok(self.project_new()),
            Request::ProjectOpen { path } => self.project_open(Path::new(&path)),
            Request::ProjectOpenRecent { index } => self.project_open_recent(index),
            Request::ProjectSave => self.project_save(),
            Request::ProjectSaveAs { path, embed_data } => self.project_save_as(&path, embed_data),
            Request::ProjectRecent => Ok(self.project_recent()),
            Request::ProjectRecovery => Ok(self.project_recovery()),
            Request::ProjectRecover { id } => self.project_recover(id),
            Request::ProjectDiscardRecovery => Ok(self.project_discard_recovery()),
            Request::CursorSet {
                dataset,
                row,
                origin,
            } => self.cursor_set(&dataset, row, origin),
            Request::CursorGet => Ok(self.cursor_get()),
            request @ (Request::FunctionCheck { .. }
            | Request::FunctionOpen { .. }
            | Request::AudifyCalculate { .. }) => self.functions(request),
            Request::NavMove { verb } => self.nav_move(&verb),
            Request::NavWhere => self.nav_where(),
            Request::ViewGet { .. } | Request::TablePage { .. } => Err(IpcError::new(
                &codes::BAD_REQUEST,
                "This command returns a binary view; send it with dispatch_binary.",
            )),
        }
    }

    /// Runs an editing command and, when it succeeds, marks the session changed (and
    /// autosaves when due).
    fn edit(
        &mut self,
        run: impl FnOnce(&mut Self) -> Result<Reply, IpcError>,
    ) -> Result<Reply, IpcError> {
        let reply = run(self)?;
        let changed = !matches!(&reply, Reply::History(state) if state.token.is_none());
        if changed {
            self.changed();
        }
        Ok(reply)
    }

    /// Describes a stream source for the consent prompt and keeps it under a one-time token.
    fn stream_offer(
        &mut self,
        transport: &str,
        url: Option<&str>,
        port: Option<&str>,
        topic: Option<&str>,
        path: Option<&str>,
    ) -> Result<Reply, IpcError> {
        let endpoint = streams::endpoint(transport, url, port, topic, path)?;
        let (transport, target, topic) = streams::target(&endpoint);
        let token = self.next_offer;
        self.next_offer += 1;
        self.offers.insert(token, endpoint);
        Ok(Reply::StreamOffer(StreamOfferInfo {
            token,
            transport: transport.to_owned(),
            target,
            topic,
        }))
    }

    /// Opens the offered stream: the user approved it (ADR-0012, A11Y-16). The token is
    /// used up, so one approval opens one connection.
    fn stream_connect(&mut self, token: u64) -> Result<Reply, IpcError> {
        let endpoint = self.offers.remove(&token).ok_or_else(|| {
            IpcError::new(
                &codes::STREAM_CONNECT,
                format!("No stream offer has the token {token}, or it was used already."),
            )
        })?;
        let options =
            sonarithm_stream::Options::new(sonarithm_stream::Capacity::Rows(streams::STREAM_ROWS));
        let consent = sonarithm_stream::Consent::user_approved(endpoint.clone());
        let connection = sonarithm_stream::connect(consent, options).map_err(|err| {
            IpcError::new(
                &codes::STREAM_CONNECT,
                format!("The stream could not start: {err}."),
            )
        })?;
        let id = format!("s{}", self.next_stream);
        self.next_stream += 1;
        self.transport.stream_open(id, endpoint, connection);
        Ok(Reply::Streams {
            streams: self.transport.stream_list()?,
        })
    }

    /// Plays stream `stream` live through the session's spec (B06-T07): a worker compiles
    /// the newest window about once a second and the transport swaps it in.
    fn live_start(&mut self, stream: String) -> Result<Reply, IpcError> {
        let text = self.spec.as_deref().ok_or_else(|| {
            IpcError::new(&codes::NO_SPEC, "The session has no spec to play live.")
        })?;
        let (effective, _) = self.effective_spec(text)?;
        let plan = crate::live::plan(&effective)?;
        let reader = self.transport.stream_reader(stream)?;
        self.live = None;
        let session = self.next_live;
        self.next_live += 1;
        let feed = self.transport.feed(session);
        self.transport.live(Some(session))?;
        let worker = crate::live::LiveWorker::spawn(
            self.snr.clone(),
            plan,
            self.open_datasets(),
            reader,
            feed,
        );
        match worker {
            Ok(worker) => self.live = Some(worker),
            Err(err) => {
                let _ = self.transport.live(None);
                return Err(err);
            }
        }
        self.act(Action::State)
    }

    fn stream_act(&mut self, id: String, action: StreamAction) -> Result<Reply, IpcError> {
        Ok(Reply::Streams {
            streams: self.transport.stream_act(id, action)?,
        })
    }

    /// The transport's output configuration: the settings' device and volume, and the
    /// effective speech preferences (ADR-0020 §3).
    fn output_config(&self) -> OutputConfig {
        let interaction = self.interaction();
        let mut config = output(&self.settings);
        config.speech.route = interaction.route;
        config.speech.duck_instead_of_pause = interaction.duck_instead_of_pause;
        config.speech.rate = WordsPerMinute(interaction.rate_wpm);
        config
    }

    /// Pauses, then announces the current value (A11Y-09, keyboard §9.2 item 3).
    fn pause(&mut self) -> Result<Reply, IpcError> {
        let reply = self.act(Action::Pause)?;
        if let Reply::Transport(state) = &reply
            && state.status == PlaybackStatus::Paused
            && let Some(announcement) = self.current_value(state.position_s)
        {
            self.transport.announce(announcement);
        }
        Ok(reply)
    }

    /// "Paused, …" with the value of the point at the playhead, in the listener's verbosity.
    fn current_value(&self, position_s: f64) -> Option<Announcement> {
        let tree = self.compiled.as_ref()?.nav()?;
        let localizer = self.localizer.as_ref()?;
        let node = point_at(tree, position_s)?;
        Composer::new(localizer, self.composer_settings())
            .paused(tree, node)
            .ok()
    }

    /// Runs a command whose reply is a binary view (ADR-0017 §6): `view_get` and
    /// `table_page`.
    ///
    /// # Errors
    ///
    /// `SNR-APP-009` for a command that replies in JSON, `SNR-APP-002` for an unknown
    /// dataset, and the data layer's errors (`SNR-DATA-060` for a view that cannot be drawn).
    pub fn dispatch_binary(&mut self, request: Request) -> Result<Vec<u8>, IpcError> {
        match request {
            Request::ViewGet {
                dataset,
                x,
                y,
                range,
                px_width,
                method,
            } => {
                let table = self.table(&dataset)?;
                crate::views::tile(&table, &mut self.tiles, &x, &y, range, px_width, method)
            }
            Request::TablePage {
                dataset,
                offset,
                limit,
                sort,
            } => crate::views::page(&self.table(&dataset)?, offset, limit, sort.as_ref()),
            other => Err(IpcError::new(
                &codes::BAD_REQUEST,
                format!("{} replies in JSON; send it with dispatch.", other.name()),
            )),
        }
    }

    /// The primary table of an open dataset.
    fn table(&self, id: &str) -> Result<Table, IpcError> {
        self.datasets
            .values()
            .find(|open| open.summary.id == id)
            .and_then(|open| open.bundle.primary().cloned())
            .ok_or_else(|| unknown_dataset(id))
    }

    /// [`App::dispatch`] for a request as JSON text, returning the reply as JSON: what a
    /// host that only moves text (a test harness, an Electron bridge) needs.
    ///
    /// # Errors
    ///
    /// `SNR-APP-009` when the text is not a request; otherwise as [`App::dispatch`].
    pub fn dispatch_json(&mut self, request: &str) -> Result<serde_json::Value, IpcError> {
        let request: Request = serde_json::from_str(request).map_err(|err| {
            IpcError::new(
                &codes::BAD_REQUEST,
                format!("The request is not a valid IPC message: {err}."),
            )
        })?;
        let reply = self.dispatch(request)?;
        serde_json::to_value(reply).map_err(|err| {
            IpcError::new(
                &codes::BAD_REQUEST,
                format!("The reply could not be encoded: {err}."),
            )
        })
    }

    /// Waits until every job is done, at most `limit` (tests and orderly shutdown).
    #[must_use]
    pub fn wait_for_jobs(&self, limit: Duration) -> bool {
        self.jobs.wait_idle(limit)
    }

    /// Makes the null device disappear, as an unplugged headset does (tests).
    #[doc(hidden)]
    pub fn unplug_audio(&self) {
        self.transport.unplug();
    }

    fn act(&self, action: Action) -> Result<Reply, IpcError> {
        self.transport.act(action).map(Reply::Transport)
    }

    fn dataset_open(&mut self, path: &Path) -> Result<Reply, IpcError> {
        let path = data_file(path)?;
        let number = self.next_dataset;
        // Each dataset's table gets its own ID, so cached view tiles never mix datasets.
        let ids = u32::try_from(number).unwrap_or(u32::MAX);
        let name = file_name(&path);
        let chart = open_chart(&path, &name, BundleId(number), TableId(ids))?;
        let mut chart_state = None;
        let (bundle, diagnostics) = if let Some((bundle, diagnostics, state)) = chart {
            chart_state = Some(state);
            (bundle, diagnostics)
        } else {
            let options = ReadOptions::new().with_ids(BundleId(number), TableId(ids));
            let loaded = read_file(&path, None, &options)
                .map_err(|err| IpcError::from_diagnostic(&err.to_diagnostic()))?;
            (loaded.bundle, loaded.diagnostics)
        };
        self.next_dataset += 1;
        let id = format!("d{number}");
        let summary = summarize(&id, &name, &bundle, &diagnostics);
        self.datasets.insert(
            number,
            Open {
                summary: summary.clone(),
                bundle,
                origin: Some(path.clone()),
                chart: chart_state,
                function: None,
            },
        );
        Ok(Reply::Dataset(summary))
    }

    fn dataset_close(&mut self, id: &str) -> Result<Reply, IpcError> {
        let number = self
            .datasets
            .iter()
            .find(|(_, open)| open.summary.id == id)
            .map(|(number, _)| *number)
            .ok_or_else(|| unknown_dataset(id))?;
        if let Some(open) = self.datasets.remove(&number)
            && let Some(table) = open.bundle.primary()
        {
            // No generation is current any more: the dataset's tiles go.
            self.tiles
                .retain_generation(table.id(), sonarithm_core::Generation(u64::MAX));
        }
        Ok(self.datasets())
    }

    fn datasets(&self) -> Reply {
        Reply::Datasets {
            datasets: self
                .datasets
                .values()
                .map(|open| open.summary.clone())
                .collect(),
        }
    }

    fn spec_set(&mut self, text: String) -> Result<Reply, IpcError> {
        if let Err(err) = Spec::from_json_str(text.as_str()) {
            return Err(first_error(&err));
        }
        let before = self.spec.replace(text.clone());
        let token = self.history.record(
            "Set spec",
            Snapshot::Spec(before),
            Snapshot::Spec(Some(text.clone())),
        );
        Ok(Reply::Spec(SpecState {
            text: Some(text),
            undo_token: Some(token),
        }))
    }

    fn spec_apply(
        &mut self,
        patch: &[crate::patch::PatchOp],
        label: Option<String>,
    ) -> Result<Reply, IpcError> {
        let text = self
            .spec
            .clone()
            .ok_or_else(|| IpcError::new(&codes::NO_SPEC, "The session has no spec to patch."))?;
        let document: serde_json::Value = serde_json::from_str(&text).map_err(|err| {
            IpcError::new(
                &codes::SPEC_PATCH,
                format!("The spec is not JSON, so it cannot be patched: {err}."),
            )
        })?;
        let patched = crate::patch::apply(&document, patch).map_err(|why| {
            IpcError::new(
                &codes::SPEC_PATCH,
                format!("The patch does not apply: {why}."),
            )
        })?;
        let new_text = serde_json::to_string_pretty(&patched).map_err(|err| {
            IpcError::new(
                &codes::SPEC_PATCH,
                format!("The patched spec cannot be written: {err}."),
            )
        })?;
        let label = label.unwrap_or_else(|| "Edit spec".to_owned());
        let token = self.history.record(
            label,
            Snapshot::Spec(Some(text)),
            Snapshot::Spec(Some(new_text.clone())),
        );
        self.spec = Some(new_text.clone());
        let (diagnostics, counts) = diagnose(&new_text);
        Ok(Reply::SpecPatch(SpecPatchResult {
            undo_token: token,
            text: new_text,
            diagnostics,
            counts,
        }))
    }

    /// The open datasets, by ID, as a compile reads them.
    fn open_datasets(&self) -> Datasets {
        let mut datasets = Datasets::new();
        for open in self.datasets.values() {
            datasets.insert(open.summary.id.clone(), open.bundle.clone());
        }
        datasets
    }

    fn compile(&mut self) -> Result<Reply, IpcError> {
        let text = self
            .spec
            .as_deref()
            .ok_or_else(|| IpcError::new(&codes::NO_SPEC, "The session has no spec to compile."))?;
        // The profile and project layers fill what the spec leaves unset (ADR-0020 §3).
        let (effective, _) = self.effective_spec(text)?;
        let spec = Spec::from_json_str(effective).map_err(|err| first_error(&err))?;
        let datasets = self.open_datasets();
        // A compile replaces a live stream's Scores; the transport leaves live mode.
        self.live = None;
        match self.snr.compile_with(&spec, &datasets) {
            Ok(compiled) => {
                let report = report(true, compiled.diagnostics().iter(), Some(&compiled));
                // The filters' excluded series carry over by label (B03-T09), so the
                // transport plays the new Score without them.
                let (excluded, series) = (self.excluded_tracks(), self.nav_series_track());
                self.compiled = Some(compiled);
                self.renumber_nav(&excluded, series.as_ref());
                self.load_playback();
                Ok(Reply::Compile(report))
            }
            Err(err) => {
                let diagnostics = err.diagnostics();
                if diagnostics.is_empty() {
                    return Err(IpcError::new(
                        &codes::COMPILE_FAILED,
                        format!("The compile failed: {err}."),
                    ));
                }
                Ok(Reply::Compile(report(false, diagnostics.iter(), None)))
            }
        }
    }

    fn export(&mut self, request: &ExportRequest) -> Result<Reply, IpcError> {
        let compiled = self.compiled.clone().ok_or_else(|| {
            IpcError::new(
                &codes::NOT_COMPILED,
                "There is no compiled Score to export.",
            )
        })?;
        let path = export_path(&request.path)?;
        let mut options = RenderOptions::new();
        if let Some(rate) = request.sample_rate {
            options = options.with_sample_rate(rate);
        }
        if !request.cues {
            options = options.without_cues();
        }
        if request.preserve_level {
            options = options.with_normalization(Normalization::Preserve);
        }
        let snr = self.snr.clone();
        let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let work: Work = Box::new(move |context| {
            if context.cancelled() {
                return Err(Failure::Cancelled);
            }
            let rendered = snr
                .render_to_file(&compiled, &path, &options)
                .map_err(|err| {
                    Failure::Failed(IpcError::new(
                        &codes::EXPORT_FAILED,
                        scrub(&format!("The export failed: {err}."), &folder),
                    ))
                })?;
            context.progress(1.0);
            if context.cancelled() {
                // The user no longer wants these files.
                let _ = std::fs::remove_file(&rendered.audio);
                let _ = std::fs::remove_file(&rendered.manifest_path);
                if let Some(cues) = &rendered.cues {
                    let _ = std::fs::remove_file(cues);
                }
                return Err(Failure::Cancelled);
            }
            let output = &rendered.manifest.output;
            Ok(Output::Export(ExportResult {
                audio: file_name(&rendered.audio),
                manifest: file_name(&rendered.manifest_path),
                cues: rendered.cues.as_deref().map(file_name),
                duration_s: rendered.manifest.render.duration_s,
                sample_rate: rendered.manifest.render.sample_rate,
                integrated_lufs: output.integrated_lufs,
                true_peak_dbtp: output.true_peak_dbtp,
            }))
        });
        let reply = self.jobs.submit(JobKind::Export, work).map(Reply::Job)?;
        // The filters are playback state: say the export holds every series (B03-T09).
        self.announce_export_scope();
        Ok(reply)
    }

    fn step(&mut self, back: bool) -> Reply {
        let edit = if back {
            self.history.undo()
        } else {
            self.history.redo()
        };
        let mut corrections = None;
        let (token, label) = match edit {
            Some(edit) => {
                match if back { edit.before } else { edit.after } {
                    Snapshot::Spec(spec) => self.spec = spec,
                    Snapshot::Corrections { dataset, text } => {
                        corrections = self.restore_corrections(dataset, &text);
                    }
                }
                (Some(edit.token), Some(edit.label))
            }
            None => (None, None),
        };
        Reply::History(HistoryState {
            token,
            label,
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            spec: SpecState {
                text: self.spec.clone(),
                undo_token: None,
            },
            corrections,
        })
    }

    fn settings_set(&mut self, settings: Settings) -> Result<Reply, IpcError> {
        self.store.save(&settings)?;
        self.settings = settings;
        self.transport.configure(self.output_config())?;
        Ok(Reply::Settings(self.settings.clone()))
    }

    fn devices(&self) -> Result<Reply, IpcError> {
        let chosen = self.settings.audio.device.as_deref();
        let devices = self
            .transport
            .devices()?
            .into_iter()
            .map(|device| DeviceSummary {
                selected: chosen.map_or(device.is_default, |id| id == device.id),
                id: device.id,
                name: device.name,
                is_default: device.is_default,
            })
            .collect();
        Ok(Reply::Devices { devices })
    }

    fn device_select(&mut self, id: Option<String>) -> Result<Reply, IpcError> {
        if let Some(wanted) = &id
            && !self
                .transport
                .devices()?
                .iter()
                .any(|device| &device.id == wanted)
        {
            return Err(IpcError::new(
                &codes::AUDIO_UNAVAILABLE,
                format!("No output device has the ID {wanted}."),
            ));
        }
        let mut settings = self.settings.clone();
        settings.audio.device = id;
        self.store.save(&settings)?;
        self.settings = settings;
        self.transport
            .configure(self.output_config())
            .map(Reply::Transport)
    }
}

/// The validation diagnostics of spec text.
fn diagnose(text: &str) -> (Vec<DiagnosticInfo>, Counts) {
    let diagnostics = match Spec::from_json_str(text) {
        Ok(spec) => spec.validate(),
        Err(err) => err.diagnostics(),
    };
    let infos: Vec<DiagnosticInfo> = diagnostics.iter().map(DiagnosticInfo::from).collect();
    let counts = count(&infos);
    (infos, counts)
}

fn validation(text: &str) -> Reply {
    let (diagnostics, counts) = diagnose(text);
    Reply::Validation {
        diagnostics,
        counts,
    }
}

fn count(diagnostics: &[DiagnosticInfo]) -> Counts {
    let mut counts = Counts::default();
    for diagnostic in diagnostics {
        match diagnostic.severity.as_str() {
            "error" => counts.errors += 1,
            "warning" => counts.warnings += 1,
            _ => counts.info += 1,
        }
    }
    counts
}

fn hello(protocol: u32) -> Result<Reply, IpcError> {
    if protocol != PROTOCOL_VERSION {
        return Err(IpcError::new(
            &codes::PROTOCOL_MISMATCH,
            format!(
                "The user interface speaks IPC protocol {protocol}, but this host speaks protocol {PROTOCOL_VERSION}."
            ),
        ));
    }
    Ok(Reply::Hello {
        protocol: PROTOCOL_VERSION,
        app_version: sonarithm::VERSION.to_owned(),
    })
}

fn output(settings: &Settings) -> OutputConfig {
    let mut speech = SequencerConfig::default();
    speech.route = settings.speech.route;
    speech.duck_instead_of_pause = settings.speech.duck_instead_of_pause;
    speech.rate = WordsPerMinute(settings.speech.rate_wpm);
    OutputConfig {
        device: settings.audio.device.clone(),
        buffer_frames: settings.audio.buffer_frames,
        volume_db: settings.audio.volume_db,
        calibrated_latency_s: settings.audio.calibrated_latency_s.clone(),
        speech,
    }
}

/// The locale announcements are composed in until the UI language is a setting (12 §5).
const LOCALE: &str = "en-US";

/// The point of the first series that sounds at `position_s`: the last one starting at or
/// before it, or the first point when the playhead is before them all.
fn point_at(tree: &NavTree, position_s: f64) -> Option<NavNodeId> {
    let series_of = |mut id: NavNodeId| loop {
        let node = tree.node(id)?;
        if matches!(node.kind, NavKind::Series(_)) {
            return Some(id);
        }
        id = node.parent?;
    };
    let first_series = tree
        .iter()
        .find(|(_, node)| matches!(node.kind, NavKind::Series(_)))
        .map(|(id, _)| id)?;
    let mut first = None;
    let mut best: Option<(NavNodeId, f64)> = None;
    for (id, node) in tree.iter() {
        let NavKind::Point(point) = &node.kind else {
            continue;
        };
        if series_of(id) != Some(first_series) {
            continue;
        }
        let Some(t) = point.t else {
            continue;
        };
        first = first.or(Some(id));
        if t.0 <= position_s && best.is_none_or(|(_, at)| t.0 >= at) {
            best = Some((id, t.0));
        }
    }
    best.map(|(id, _)| id).or(first)
}

fn unknown_dataset(id: &str) -> IpcError {
    IpcError::new(
        &codes::UNKNOWN_DATASET,
        format!("No open dataset has the ID {id}."),
    )
}

/// The first error of a facade error, as an IPC error.
/// The first error diagnostic of a failed compile, or the failure itself.
pub(crate) fn first_error(err: &sonarithm::Error) -> IpcError {
    let diagnostics = err.diagnostics();
    diagnostics.iter().find(|d| d.is_error()).map_or_else(
        || IpcError::new(&codes::COMPILE_FAILED, err.to_string()),
        IpcError::from_diagnostic,
    )
}

/// Checks a UI-supplied export path: absolute, `.wav` or `.flac`, in a folder that exists
/// (09 §4: no path traversal from UI strings).
fn export_path(text: &str) -> Result<PathBuf, IpcError> {
    let path = PathBuf::from(text);
    let refuse = |why: &str| IpcError::new(&codes::EXPORT_PATH, why.to_owned());
    if !path.is_absolute() {
        return Err(refuse(
            "The export path must be a full path chosen in the save dialog.",
        ));
    }
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    if !matches!(extension.as_deref(), Some("wav" | "flac")) {
        return Err(refuse("The export file must end in .wav or .flac."));
    }
    let (Some(folder), Some(name)) = (path.parent(), path.file_name()) else {
        return Err(refuse("The export path has no file name."));
    };
    // Canonical, so `..` and links cannot move the target elsewhere (ADR-0017 §7).
    let folder = std::fs::canonicalize(folder)
        .ok()
        .filter(|folder| folder.is_dir())
        .ok_or_else(|| refuse("The export folder does not exist."))?;
    let target = folder.join(name);
    if target.exists() && !target.is_file() {
        return Err(refuse(
            "The export target exists and is not a file; choose another name.",
        ));
    }
    Ok(target)
}

/// The data extensions A04 reads.
const DATA_EXTENSIONS: [&str; 8] = [
    "csv", "tsv", "json", "ndjson", "parquet", "arrow", "xlsx", "ods",
];

/// The extensions of files that hold only charts (C01-T12). A `.json` file may hold a
/// chart (Vega-Lite, Vega, Plotly, Highcharts) or data; an XLSX workbook opens as data, its
/// cells being the chart's data.
const CHART_EXTENSIONS: [&str; 6] = ["pptx", "docx", "svg", "pdf", "html", "htm"];

/// The chart in `path` as a dataset, with the extraction's diagnostics and its summary
/// sentence (`SNR-ENG-385`, 10 §5), or `None` for a data file. A file with a chart
/// extension that holds no chart is an error; a `.json` file that holds none is data.
fn open_chart(
    path: &Path,
    name: &str,
    bundle: BundleId,
    table: TableId,
) -> Result<
    Option<(
        DataBundle,
        sonarithm_core::Diagnostics,
        corrections::ChartState,
    )>,
    IpcError,
> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let chart_only = CHART_EXTENSIONS.contains(&extension.as_str());
    if !chart_only && extension != "json" {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|_| {
        IpcError::new(
            &codes::FILE_REFUSED,
            format!("`{name}` cannot be opened: it does not exist or cannot be read."),
        )
    })?;
    let options = sonarithm::ExtractOptions::new().with_ids(bundle, table);
    match sonarithm::read_chart_bytes(&bytes, name, &options) {
        Ok(chart) => {
            let mut diagnostics = sonarithm_core::Diagnostics::new();
            diagnostics.push(sonarithm::summary_diagnostic(&chart));
            for diagnostic in &chart.report().diagnostics {
                diagnostics.push(diagnostic.clone());
            }
            let extractor = chart.report().extractor.clone();
            let version = chart.bundle().source().extractor_version.clone();
            let state = corrections::ChartState::new(bytes, name, &extractor, &version);
            Ok(Some((chart.into_bundle(), diagnostics, state)))
        }
        Err(err) if chart_only => Err(err.diagnostics().iter().next().map_or_else(
            || IpcError::new(&codes::FILE_REFUSED, format!("`{name}` holds no chart.")),
            IpcError::from_diagnostic,
        )),
        Err(_) => Ok(None),
    }
}

/// Checks a UI-supplied data file: it exists, is a regular file (not a directory, device or
/// pipe) and has a data extension. Returns its canonical path (ADR-0017 §7). Errors name
/// the file only, never its folder.
fn data_file(path: &Path) -> Result<PathBuf, IpcError> {
    let name = file_name(path);
    let refuse = |why: &str| IpcError::new(&codes::FILE_REFUSED, format!("`{name}` {why}"));
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    if !extension
        .as_deref()
        .is_some_and(|ext| DATA_EXTENSIONS.contains(&ext) || CHART_EXTENSIONS.contains(&ext))
    {
        return Err(refuse("is not a data file SONARITHM reads."));
    }
    let canonical = std::fs::canonicalize(path)
        .map_err(|_| refuse("cannot be opened: it does not exist or cannot be read."))?;
    let regular = std::fs::metadata(&canonical).is_ok_and(|meta| meta.is_file());
    if !regular {
        return Err(refuse("is not a regular file."));
    }
    Ok(canonical)
}

/// A path's file name alone, for replies and messages (03 §2 `locator_redacted`).
fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || "the file".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// `text` without any mention of `folder`, in its display forms (a backstop: lower layers
/// already name files alone).
fn scrub(text: &str, folder: &Path) -> String {
    let shown = folder.display().to_string();
    if shown.is_empty() {
        return text.to_owned();
    }
    let mut out = text.to_owned();
    for form in [
        shown.clone(),
        shown.replace('\\', "/"),
        shown.replace('/', "\\"),
    ] {
        out = out.replace(&form, "");
    }
    out
}

fn summarize(
    id: &str,
    name: &str,
    bundle: &DataBundle,
    diagnostics: &sonarithm_core::Diagnostics,
) -> DatasetSummary {
    let table = bundle.primary();
    let fields = table
        .map(|table| {
            table
                .schema()
                .fields()
                .iter()
                .filter(|field| !field.name().starts_with("__snr_"))
                .map(|field| {
                    let meta = table.field_meta(field.name()).unwrap_or_default();
                    FieldSummary {
                        name: field.name().clone(),
                        data_type: field.data_type().to_string(),
                        field_type: meta.ftype.map(|t| t.as_str().to_owned()),
                        role: meta.role.map(|r| r.as_str().to_owned()),
                        unit: meta.unit.map(|u| u.text),
                        concept: meta.concept.map(|c| c.as_str().to_owned()),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    DatasetSummary {
        id: id.to_owned(),
        name: name.to_owned(),
        rows: table.map_or(0, sonarithm_data::Table::num_rows),
        fields,
        diagnostics: diagnostics.iter().map(DiagnosticInfo::from).collect(),
    }
}

fn report<'a>(
    ok: bool,
    diagnostics: impl Iterator<Item = &'a sonarithm_core::Diagnostic>,
    compiled: Option<&Compiled>,
) -> CompileReport {
    let diagnostics: Vec<DiagnosticInfo> = diagnostics.map(DiagnosticInfo::from).collect();
    let counts = count(&diagnostics);
    let score = compiled.map(|compiled| {
        let summary = compiled.summary();
        ScoreInfo {
            duration_s: summary.duration_s,
            mode: serde_json::to_value(summary.mode)
                .ok()
                .and_then(|mode| mode.as_str().map(str::to_owned))
                .unwrap_or_default(),
            spec_hash: summary.spec_hash,
            score_hash: summary.score_hash,
            tracks: summary
                .tracks
                .iter()
                .map(|track| TrackInfo {
                    id: track.id.clone(),
                    instrument: track.instrument.clone(),
                    events: u64::try_from(track.events).unwrap_or(u64::MAX),
                    estimated: u64::try_from(track.estimated).unwrap_or(u64::MAX),
                })
                .collect(),
            engines: summary.engines,
        }
    });
    CompileReport {
        ok,
        diagnostics,
        counts,
        score,
    }
}
