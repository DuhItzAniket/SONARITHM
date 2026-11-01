// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The project commands of the app core (RFC-0002 §8–§9; B05-T04): the session as a
//! `.sonarithm` bundle, the recent list, autosave and crash recovery.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use sonarithm_data::bundle::SourceKind;
use sonarithm_data::{BundleId, DataBundle, Format, SourceDescriptor, table_fingerprint};

use super::{App, Open, file_name, summarize};
use crate::codes;
use crate::history::History;
use crate::ipc::{
    AutosaveInfo, DiagnosticInfo, IpcError, ProjectDatasetInfo, ProjectSummary, RecentProject,
    Reply,
};
use crate::project::{self, EXTENSION, Project, ProjectDataset};

/// The shortest gap between two autosaves (RFC-0002 §8).
pub(super) const AUTOSAVE_GAP: Duration = Duration::from_secs(30);

/// The most projects the recent list keeps.
const RECENT_KEPT: usize = 10;

/// The recent list's file in the app-config folder: absolute paths, local only, never in a
/// project and never sent over the protocol (ADR-0017 §7).
const RECENT_FILE: &str = "recent-projects.json";

/// The project the session belongs to.
#[derive(Debug, Default)]
pub(super) struct ProjectState {
    /// The canonical project file, once saved or opened.
    pub(super) path: Option<PathBuf>,
    pub(super) name: String,
    pub(super) unsaved: bool,
    /// The spec the session edits.
    pub(super) active_spec: String,
    /// The other specs, kept as they were.
    pub(super) other_specs: Vec<(String, String)>,
    /// Linked datasets that are unavailable: kept, never embedded.
    pub(super) unavailable: Vec<ProjectDataset>,
    pub(super) reserved: std::collections::BTreeMap<String, Vec<u8>>,
    /// Warnings from the last open.
    pub(super) warnings: Vec<IpcError>,
    pub(super) last_autosave: Option<Instant>,
}

impl ProjectState {
    pub(super) fn new() -> Self {
        Self {
            name: "Untitled".to_owned(),
            active_spec: "main".to_owned(),
            ..Self::default()
        }
    }
}

pub(super) fn warning_info(error: &IpcError) -> DiagnosticInfo {
    DiagnosticInfo {
        severity: "warning".to_owned(),
        code: error.code.clone(),
        rules: Vec::new(),
        message: error.message.clone(),
        pointer: error.pointer.clone(),
    }
}

/// Checks a project file the user chose to open: a regular `.sonarithm` file.
fn project_file(path: &Path) -> Result<PathBuf, IpcError> {
    let name = file_name(path);
    let refuse = |why: &str| IpcError::new(&codes::FILE_REFUSED, format!("`{name}` {why}"));
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if extension.as_deref() != Some(EXTENSION) {
        return Err(refuse("is not a SONARITHM project (.sonarithm)."));
    }
    let canonical = std::fs::canonicalize(path)
        .map_err(|_| refuse("cannot be opened: it does not exist or cannot be read."))?;
    if !std::fs::metadata(&canonical).is_ok_and(|m| m.is_file()) {
        return Err(refuse("is not a regular file."));
    }
    Ok(canonical)
}

/// Checks a target the user chose to save to: absolute, `.sonarithm`, in a folder that
/// exists; returns it with a canonical folder.
fn project_target(text: &str) -> Result<PathBuf, IpcError> {
    let path = PathBuf::from(text);
    let refuse = |why: &str| IpcError::new(&codes::FILE_REFUSED, why.to_owned());
    if !path.is_absolute() {
        return Err(refuse(
            "The project path must be a full path chosen in the save dialog.",
        ));
    }
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if extension.as_deref() != Some(EXTENSION) {
        return Err(refuse("A project file must end in .sonarithm."));
    }
    let (Some(folder), Some(name)) = (path.parent(), path.file_name()) else {
        return Err(refuse("The project path has no file name."));
    };
    let folder = std::fs::canonicalize(folder)
        .ok()
        .filter(|f| f.is_dir())
        .ok_or_else(|| refuse("The project folder does not exist."))?;
    let target = folder.join(name);
    if target.exists() && !target.is_file() {
        return Err(refuse(
            "The project target exists and is not a file; choose another name.",
        ));
    }
    Ok(target)
}

impl App {
    /// The session as a project.
    fn to_project(&self) -> Result<Project, IpcError> {
        let text = self.spec.clone().ok_or_else(|| {
            IpcError::new(&codes::NO_SPEC, "The project has no spec to save yet.")
        })?;
        let mut specs = vec![(self.project.active_spec.clone(), text)];
        specs.extend(self.project.other_specs.iter().cloned());
        let mut datasets = Vec::new();
        for open in self.datasets.values() {
            let Some(table) = open.bundle.primary() else {
                continue;
            };
            let fingerprint = table_fingerprint(table)
                .map_err(|err| IpcError::from_diagnostic(&err.to_diagnostic()))?
                .to_string();
            datasets.push(ProjectDataset {
                id: open.summary.id.clone(),
                name: open.summary.name.clone(),
                rows: table.num_rows(),
                table: Some(table.clone()),
                origin: open.origin.clone(),
                linked: open.origin.is_some(),
                link: None,
                fingerprint,
            });
        }
        datasets.extend(self.project.unavailable.iter().cloned());
        datasets.sort_by(|a, b| a.id.cmp(&b.id));
        let mut project = Project::default();
        project.name.clone_from(&self.project.name);
        project.specs = specs;
        project.active_spec.clone_from(&self.project.active_spec);
        project.datasets = datasets;
        project.reserved.clone_from(&self.project.reserved);
        // Each chart's corrections (RFC-0005 §7), in the reserved annotations folder.
        self.corrections_into(&mut project.reserved);
        Ok(project)
    }

    /// Replaces the session with `opened`'s project, unsaved or saved at `path`.
    fn load_project(&mut self, opened: project::Opened, path: Option<PathBuf>, numbers: &[u64]) {
        self.datasets.clear();
        // A row of the old data means nothing in the new (B04-T06).
        self.clear_cursor();
        self.tiles = sonarithm_data::TileCache::new(crate::views::TILE_CACHE);
        self.compiled = None;
        self.transport.load(None, Vec::new());
        self.history = History::default();
        let project = opened.project;
        let mut state = super::projects::ProjectState::new();
        state.name = project.name;
        state.active_spec.clone_from(&project.active_spec);
        state.reserved = project.reserved;
        // A project's profile may carry keys; they are never applied (keyboard §13).
        let keys = state
            .reserved
            .get(super::profiles::PROJECT_MEMBER)
            .and_then(|bytes| crate::profile::parse(bytes).ok())
            .is_some_and(|(profile, _)| !profile.interaction.keymap.is_empty());
        if keys {
            self.sink.emit(crate::ipc::Event::SettingsNotice {
                error: IpcError::new(
                    &codes::KEYMAP_REFUSED,
                    "This project's profile sets keys; they were not applied, because a project \
                     cannot change your keyboard. Set keys in Preferences › Keyboard.",
                ),
            });
        }
        for (id, text) in project.specs {
            if id == state.active_spec {
                self.spec = Some(text);
            } else {
                state.other_specs.push((id, text));
            }
        }
        for (dataset, number) in project.datasets.into_iter().zip(numbers) {
            let Some(table) = dataset.table.clone() else {
                state.unavailable.push(dataset);
                continue;
            };
            let source =
                SourceDescriptor::new(SourceKind::File(Format::Parquet), &dataset.name, "project");
            let bundle = DataBundle::single(BundleId(*number), source, table);
            // Corrections come back with their history, read-only until the chart is opened.
            let chart = super::corrections::from_member(&state.reserved, &dataset.id);
            let summary = summarize(
                &dataset.id,
                &dataset.name,
                &bundle,
                &sonarithm_core::Diagnostics::new(),
            );
            self.datasets.insert(
                *number,
                Open {
                    summary,
                    bundle,
                    origin: dataset.origin,
                    chart,
                },
            );
        }
        self.next_dataset = numbers.last().map_or(1, |n| n + 1);
        state.path = path;
        state.warnings = opened.warnings;
        // The project's profile (ADR-0020 §2): a broken one is reported and not used.
        if let Some(bytes) = state.reserved.get(super::profiles::PROJECT_MEMBER) {
            match crate::profile::parse(bytes) {
                Ok((_, warnings)) => state.warnings.extend(warnings),
                Err(err) => state.warnings.push(IpcError::new(
                    &codes::PROFILE_INVALID,
                    format!("The project's profile is not used: {}", err.message),
                )),
            }
        }
        self.project = state;
    }

    /// Opens the project at `path` (canonical, checked).
    fn open_project_at(&mut self, path: &Path, keep_path: bool) -> Result<Reply, IpcError> {
        let mut numbers = Vec::new();
        let mut next = 1_u64;
        let opened = project::open(path, &mut || {
            numbers.push(next);
            let id = u32::try_from(next).unwrap_or(u32::MAX);
            next += 1;
            id
        })?;
        self.load_project(opened, keep_path.then(|| path.to_path_buf()), &numbers);
        if keep_path {
            self.remember(path);
        } else {
            self.project.unsaved = true;
        }
        Ok(self.project_summary())
    }

    pub(super) fn project_new(&mut self) -> Reply {
        let opened = project::Opened {
            project: Project {
                name: "Untitled".to_owned(),
                active_spec: "main".to_owned(),
                ..Project::default()
            },
            warnings: Vec::new(),
        };
        self.load_project(opened, None, &[]);
        self.spec = None;
        self.project_summary()
    }

    pub(super) fn project_open(&mut self, path: &Path) -> Result<Reply, IpcError> {
        let path = project_file(path)?;
        self.open_project_at(&path, true)
    }

    pub(super) fn project_open_recent(&mut self, index: u32) -> Result<Reply, IpcError> {
        let path = self.recent().get(index as usize).cloned().ok_or_else(|| {
            IpcError::new(
                &codes::FILE_REFUSED,
                format!("No recent project has the position {index}."),
            )
        })?;
        let path = project_file(&path)?;
        self.open_project_at(&path, true)
    }

    pub(super) fn project_save(&mut self) -> Result<Reply, IpcError> {
        let path = self.project.path.clone().ok_or_else(|| {
            IpcError::new(
                &codes::NO_PROJECT_PATH,
                "The project has not been saved to a file yet.",
            )
        })?;
        self.save_to(&path, false)
    }

    pub(super) fn project_save_as(&mut self, path: &str, embed: bool) -> Result<Reply, IpcError> {
        let path = project_target(path)?;
        if self.project.name == "Untitled"
            && let Some(stem) = path.file_stem()
        {
            self.project.name = stem.to_string_lossy().into_owned();
        }
        self.save_to(&path, embed)
    }

    fn save_to(&mut self, path: &Path, embed: bool) -> Result<Reply, IpcError> {
        let project = self.to_project()?;
        project::save(&project, path, embed)?;
        self.project.path = Some(path.to_path_buf());
        self.project.unsaved = false;
        self.project.warnings.clear();
        self.autosaves.discard();
        self.remember(path);
        Ok(self.project_summary())
    }

    pub(super) fn project_recent(&self) -> Reply {
        let projects = self
            .recent()
            .iter()
            .enumerate()
            .map(|(i, path)| RecentProject {
                index: u32::try_from(i).unwrap_or(u32::MAX),
                name: file_name(path),
            })
            .collect();
        Reply::Recent { projects }
    }

    pub(super) fn project_recovery(&self) -> Reply {
        Reply::Recovery {
            autosaves: self
                .autosaves
                .list()
                .into_iter()
                .map(|a| AutosaveInfo {
                    id: a.id,
                    modified_s: a.modified_s,
                })
                .collect(),
        }
    }

    pub(super) fn project_recover(&mut self, id: u64) -> Result<Reply, IpcError> {
        let path = self.autosaves.get(id).ok_or_else(|| {
            IpcError::new(
                &codes::FILE_REFUSED,
                format!("No autosave has the ID {id}."),
            )
        })?;
        self.open_project_at(&path, false)
    }

    pub(super) fn project_discard_recovery(&self) -> Reply {
        self.autosaves.discard();
        self.project_recovery()
    }

    pub(super) fn project_summary(&self) -> Reply {
        let mut specs = vec![self.project.active_spec.clone()];
        specs.extend(self.project.other_specs.iter().map(|(id, _)| id.clone()));
        let mut datasets: Vec<ProjectDatasetInfo> = self
            .datasets
            .values()
            .map(|open| ProjectDatasetInfo {
                id: open.summary.id.clone(),
                name: open.summary.name.clone(),
                linked: open.origin.is_some(),
                available: true,
            })
            .collect();
        datasets.extend(self.project.unavailable.iter().map(|d| ProjectDatasetInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            linked: true,
            available: false,
        }));
        Reply::Project(ProjectSummary {
            name: self.project.name.clone(),
            file_name: self.project.path.as_deref().map(file_name),
            unsaved: self.project.unsaved,
            active_spec: self
                .spec
                .is_some()
                .then(|| self.project.active_spec.clone()),
            specs,
            datasets,
            diagnostics: self.project.warnings.iter().map(warning_info).collect(),
        })
    }

    /// Marks the session changed and autosaves when due (RFC-0002 §8).
    pub(super) fn changed(&mut self) {
        self.project.unsaved = true;
        let due = self
            .project
            .last_autosave
            .is_none_or(|at| at.elapsed() >= AUTOSAVE_GAP);
        if due {
            self.autosave_now();
        }
    }

    /// Writes an autosave now, with every dataset embedded. Best effort: a failure leaves
    /// the previous autosaves.
    pub fn autosave_now(&mut self) {
        let Ok(project) = self.to_project() else {
            return;
        };
        let anchor = self.autosaves_dir().join("autosave.sonarithm");
        if let Ok(bytes) = project::write(&project, &anchor, true)
            && self.autosaves.write(&bytes).is_ok()
        {
            self.project.last_autosave = Some(Instant::now());
        }
    }

    fn autosaves_dir(&self) -> PathBuf {
        self.data_dir.join("autosave")
    }

    /// The recent list, most recent first.
    fn recent(&self) -> Vec<PathBuf> {
        std::fs::read(self.config_dir.join(RECENT_FILE))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Vec<String>>(&bytes).ok())
            .unwrap_or_default()
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }

    /// Puts `path` first in the recent list (best effort).
    fn remember(&self, path: &Path) {
        let mut list = self.recent();
        list.retain(|p| p != path);
        list.insert(0, path.to_path_buf());
        list.truncate(RECENT_KEPT);
        let text: Vec<String> = list
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        if let Ok(bytes) = serde_json::to_vec_pretty(&text) {
            let _ = std::fs::create_dir_all(&self.config_dir);
            let _ =
                crate::settings::write_atomically(&self.config_dir.join(RECENT_FILE), &bytes, None);
        }
    }
}
