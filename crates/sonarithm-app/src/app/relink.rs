// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Check and relink a linked dataset the project could not verify (RFC-0008 §6, §11;
//! ADR-0017 "Relinking linked data").
//!
//! `dataset_relink_check` reads the file again and reports its row count, without using
//! the data. `dataset_relink` reads it once more, makes the dataset available, and marks
//! the project unsaved. It is not on the undo history (OD-32): closing the project without
//! saving reverts it, and the announcement says so.

use std::path::Path;

use sonarithm_audaxis::announce::Announcement;
use sonarithm_data::{BundleId, Limits, ReadOptions};
use sonarithm_i18n::Args;

use super::{App, Open, file_name, summarize, unknown_dataset};
use crate::codes;
use crate::ipc::{IpcError, RelinkReason, Reply};
use crate::project::{self, ProjectDataset};

impl App {
    /// The unavailable linked dataset `id`, with the project's folder its link starts from.
    fn relinkable(&self, id: &str) -> Result<(usize, &Path), IpcError> {
        let Some(index) = self
            .project
            .unavailable
            .iter()
            .position(|dataset| dataset.id == id)
        else {
            return Err(
                if self.datasets.values().any(|open| open.summary.id == id) {
                    IpcError::new(
                        &codes::NOTHING_TO_RELINK,
                        format!("`{id}` is available, so there is nothing to relink."),
                    )
                } else {
                    unknown_dataset(id)
                },
            );
        };
        let dataset = &self.project.unavailable[index];
        if !dataset.linked || dataset.kept.is_some() || dataset.function.is_some() {
            return Err(IpcError::new(
                &codes::NOTHING_TO_RELINK,
                format!(
                    "`{}` is not linked to a file, so there is nothing to relink.",
                    file_name(Path::new(&dataset.name))
                ),
            ));
        }
        let folder = self
            .project
            .path
            .as_deref()
            .and_then(Path::parent)
            .ok_or_else(|| {
                IpcError::new(
                    &codes::LINKED_DATA,
                    format!(
                        "The linked data `{}` cannot be found, because the project has no folder yet, so it is unavailable.",
                        file_name(Path::new(&dataset.name))
                    ),
                )
            })?;
        Ok((index, folder))
    }

    /// Reads the file again and replies with what relinking would use (RFC-0008 §6).
    pub(super) fn dataset_relink_check(&self, id: &str) -> Result<Reply, IpcError> {
        let (index, folder) = self.relinkable(id)?;
        let dataset = &self.project.unavailable[index];
        let read = project::relink(folder, dataset, &read_options(0))?;
        Ok(Reply::RelinkCheck {
            name: file_name(Path::new(&dataset.name)),
            rows: read
                .loaded
                .bundle
                .primary()
                .map_or(0, sonarithm_data::Table::num_rows),
            recorded_rows: dataset.rows,
            reason: reason(dataset),
        })
    }

    /// Reads the file once more and makes the dataset available, recording the file's size,
    /// SHA-256 and scheme-2 fingerprint for the next save (RFC-0008 §6).
    pub(super) fn dataset_relink(&mut self, id: &str) -> Result<Reply, IpcError> {
        let (index, folder) = self.relinkable(id)?;
        let number = self.next_dataset;
        let read = project::relink(
            folder,
            &self.project.unavailable[index],
            &read_options(number),
        )?;
        let dataset = self.project.unavailable.remove(index);
        self.next_dataset += 1;
        let summary = summarize(
            &dataset.id,
            &dataset.name,
            &read.loaded.bundle,
            &read.loaded.diagnostics,
        );
        self.datasets.insert(
            number,
            Open {
                summary: summary.clone(),
                bundle: read.loaded.bundle,
                origin: Some(read.origin),
                chart: None,
                function: None,
                file: Some(read.file),
            },
        );
        // Marked unsaved here, before the reply is built, so the reply says so.
        self.changed();
        self.announce_relinked(&dataset.name);
        Ok(Reply::Relinked {
            dataset: summary,
            project: self.project_info(),
        })
    }

    /// "`name` is relinked. Save the project to keep it, or close without saving to undo."
    fn announce_relinked(&self, name: &str) {
        let args = Args::new().text("name", file_name(Path::new(name)));
        if let Some(text) = self
            .localizer
            .as_ref()
            .and_then(|localizer| localizer.format("announce-dataset-relinked", &args).ok())
        {
            self.transport
                .announce(Announcement::status(text.clone(), text));
        }
    }
}

/// Why `dataset` could not be verified: saved before RFC-0008, or its file changed.
fn reason(dataset: &ProjectDataset) -> RelinkReason {
    if dataset.fingerprint_scheme == sonarithm_core::score::FINGERPRINT_SCHEME_1 {
        RelinkReason::OlderBuild
    } else {
        RelinkReason::Changed
    }
}

/// RFC-0002 §4's limits, with dataset `number`'s IDs.
fn read_options(number: u64) -> ReadOptions {
    let ids = u32::try_from(number).unwrap_or(u32::MAX);
    ReadOptions::new()
        .with_limits(Limits::DEFAULT)
        .with_ids(BundleId(number), sonarithm_core::TableId(ids))
}
