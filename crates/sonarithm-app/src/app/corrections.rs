// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chart corrections in the app core (C02-T06, RFC-0005 §9–§11, ADR-0026): each open chart
//! keeps a correction document; a correction is a JSON Patch on it, one undoable edit, and
//! the chart is re-extracted with the corrections, never edited in place. Correction files
//! are exported and imported here; a project keeps the document as the member
//! `annotations/corrections-<dataset-id>.json`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sonarithm::CorrectionSet;
use sonarithm_core::Sha256Digest;
use sonarithm_spec::correction_file::{
    self, ChartImage, ChartRef, CorrectionFile, FileEdit, PointRef,
};

use super::{App, Open, Snapshot, summarize};
use crate::codes;
use crate::ipc::{CorrectionsState, ImportPreview, IpcError, Reply};

/// A chart dataset's corrections and what re-extracting it needs.
#[derive(Clone, Debug)]
pub(super) struct ChartState {
    /// The chart file's bytes; `None` after a project reopens, until the chart is opened
    /// again (the project holds the extracted table, not the chart).
    pub(super) bytes: Option<Arc<Vec<u8>>>,
    /// The correction document.
    pub(super) file: CorrectionFile,
    /// The edits that did not apply to the current extraction.
    pub(super) not_applied: Vec<String>,
}

impl ChartState {
    /// An empty correction set for a chart just opened from `bytes`.
    pub(super) fn new(bytes: Vec<u8>, name: &str, extractor: &str, version: &str) -> Self {
        let mut chart = ChartRef::new(Sha256Digest::of(&bytes).to_string(), extractor, version);
        chart.name = Some(name.to_owned());
        Self {
            bytes: Some(Arc::new(bytes)),
            file: CorrectionFile::new(chart),
            not_applied: Vec::new(),
        }
    }

    fn has_content(&self) -> bool {
        !self.file.edits.is_empty() || self.file.calibration.is_some()
    }
}

/// The project member holding a dataset's corrections (RFC-0005 §7).
pub(super) fn member(dataset: &str) -> String {
    format!("annotations/corrections-{dataset}.json")
}

fn invalid(message: impl Into<String>) -> IpcError {
    IpcError::new(&codes::CORRECTIONS_INVALID, message)
}

fn read_text(text: &str) -> Result<CorrectionFile, IpcError> {
    correction_file::read(text.as_bytes())
        .map(|(file, _)| file)
        .map_err(|errors| {
            errors.iter().next().map_or_else(
                || invalid("The corrections do not read."),
                IpcError::from_diagnostic,
            )
        })
}

/// The number in an edit ID `e<n>`.
fn number(id: &str) -> u64 {
    id.strip_prefix('e')
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// An edit's content without its ID, for finding duplicates (RFC-0005 §11 step 3).
fn content(edit: &FileEdit) -> serde_json::Value {
    let mut value = serde_json::to_value(edit).unwrap_or_default();
    if let Some(object) = value.as_object_mut() {
        for key in ["id", "note", "before"] {
            object.remove(key);
        }
    }
    value
}

fn with_id(edit: &FileEdit, id: &str, point: Option<PointRef>) -> FileEdit {
    let mut value = serde_json::to_value(edit).unwrap_or_default();
    value["id"] = serde_json::Value::from(id);
    if let Some(point) = point {
        value["point"] = serde_json::to_value(point).unwrap_or_default();
    }
    serde_json::from_value(value).unwrap_or_else(|_| edit.clone())
}

impl App {
    fn chart_open(&self, dataset: &str) -> Result<(u64, &Open, &ChartState), IpcError> {
        let (number, open) = self
            .datasets
            .iter()
            .find(|(_, open)| open.summary.id == dataset)
            .ok_or_else(|| super::unknown_dataset(dataset))?;
        let chart = open.chart.as_ref().ok_or_else(|| {
            IpcError::new(
                &codes::CORRECTIONS_NOT_HERE,
                format!("`{}` was not opened from a chart.", open.summary.name),
            )
        })?;
        Ok((*number, open, chart))
    }

    fn state(&self, number: u64, undo_token: Option<u64>) -> Option<CorrectionsState> {
        let open = self.datasets.get(&number)?;
        let chart = open.chart.as_ref()?;
        Some(CorrectionsState {
            dataset: open.summary.id.clone(),
            text: correction_file::write(&chart.file),
            edits: chart.file.edits.len(),
            not_applied: chart.not_applied.clone(),
            editable: chart.bytes.is_some(),
            undo_token,
            summary: open.summary.clone(),
        })
    }

    /// The correction commands (C02-T06).
    pub(super) fn corrections_request(
        &mut self,
        request: crate::ipc::Request,
    ) -> Result<Reply, IpcError> {
        use crate::ipc::Request;
        match request {
            Request::CorrectionsApply {
                dataset,
                patch,
                label,
            } => self.edit(|app| app.corrections_apply(&dataset, &patch, label)),
            Request::CorrectionsExport {
                dataset,
                path,
                notes,
                include_image,
            } => self.corrections_export(&dataset, &path, notes, include_image),
            Request::CorrectionsImport {
                dataset,
                path,
                confirm,
            } => self.edit(|app| app.corrections_import(&dataset, &path, confirm)),
            Request::CorrectionsGet { dataset } => self.corrections_get(&dataset),
            other => Err(IpcError::new(
                &codes::BAD_REQUEST,
                format!("`{}` is not a correction command.", other.name()),
            )),
        }
    }

    pub(super) fn corrections_get(&self, dataset: &str) -> Result<Reply, IpcError> {
        let (number, _, _) = self.chart_open(dataset)?;
        self.state(number, None)
            .map(Reply::Corrections)
            .ok_or_else(|| super::unknown_dataset(dataset))
    }

    /// Re-extracts dataset `number` with `file`'s corrections and keeps both. The extracted
    /// table replaces the old one; its cached view tiles go.
    fn reextract(&mut self, number: u64, file: CorrectionFile) -> Result<(), IpcError> {
        let open = self
            .datasets
            .get(&number)
            .ok_or_else(|| super::unknown_dataset(""))?;
        let chart = open
            .chart
            .as_ref()
            .ok_or_else(|| super::unknown_dataset(""))?;
        let bytes = chart.bytes.clone().ok_or_else(|| {
            IpcError::new(
                &codes::CORRECTIONS_READ_ONLY,
                format!("Open `{}` again to correct it.", open.summary.name),
            )
        })?;
        let (bundle_id, table_id) = (
            sonarithm_data::BundleId(number),
            open.bundle
                .primary()
                .map_or(sonarithm_core::TableId(0), sonarithm_data::Table::id),
        );
        let options = sonarithm::ExtractOptions::new()
            .with_ids(bundle_id, table_id)
            .with_corrections(CorrectionSet::File(Box::new(file.clone())));
        let chart =
            sonarithm::read_chart_bytes(&bytes, &open.summary.name, &options).map_err(|err| {
                err.diagnostics().iter().next().map_or_else(
                    || invalid("The chart could not be read again."),
                    IpcError::from_diagnostic,
                )
            })?;
        let not_applied: Vec<String> = chart
            .report()
            .corrections
            .iter()
            .filter(|r| !r.applied)
            .map(|r| r.edit.clone())
            .collect();
        let mut diagnostics = sonarithm_core::Diagnostics::new();
        diagnostics.push(sonarithm::summary_diagnostic(&chart));
        for diagnostic in &chart.report().diagnostics {
            diagnostics.push(diagnostic.clone());
        }
        let bundle = chart.into_bundle();
        self.tiles
            .retain_generation(table_id, sonarithm_core::Generation(u64::MAX));
        if let Some(open) = self.datasets.get_mut(&number) {
            let id = open.summary.id.clone();
            let name = open.summary.name.clone();
            open.summary = summarize(&id, &name, &bundle, &diagnostics);
            open.bundle = bundle;
            if let Some(chart) = open.chart.as_mut() {
                chart.file = file;
                chart.not_applied = not_applied;
            }
        }
        Ok(())
    }

    /// Records a change of dataset `number`'s corrections from `before` to `after` as one
    /// undoable edit.
    fn record(
        &mut self,
        number: u64,
        label: String,
        before: &CorrectionFile,
        after: &CorrectionFile,
    ) -> Result<u64, IpcError> {
        self.reextract(number, after.clone())?;
        Ok(self.history.record(
            label,
            Snapshot::Corrections {
                dataset: number,
                text: correction_file::write(before),
            },
            Snapshot::Corrections {
                dataset: number,
                text: correction_file::write(after),
            },
        ))
    }

    /// Undo and redo: puts back the corrections in `text`.
    pub(super) fn restore_corrections(
        &mut self,
        number: u64,
        text: &str,
    ) -> Option<CorrectionsState> {
        let file = read_text(text).ok()?;
        self.reextract(number, file).ok()?;
        self.state(number, None)
    }

    pub(super) fn corrections_apply(
        &mut self,
        dataset: &str,
        patch: &[crate::patch::PatchOp],
        label: Option<String>,
    ) -> Result<Reply, IpcError> {
        let (number, _, chart) = self.chart_open(dataset)?;
        if chart.bytes.is_none() {
            return Err(IpcError::new(
                &codes::CORRECTIONS_READ_ONLY,
                "Open the chart file again to correct it.",
            ));
        }
        let before = chart.file.clone();
        let document = serde_json::to_value(&before).map_err(|err| invalid(err.to_string()))?;
        let patched = crate::patch::apply(&document, patch)
            .map_err(|why| invalid(format!("The patch does not apply: {why}.")))?;
        let text = serde_json::to_string(&patched).map_err(|err| invalid(err.to_string()))?;
        let after = read_text(&text)?;
        if after.chart != before.chart {
            return Err(invalid(
                "A correction cannot change which chart the corrections belong to.",
            ));
        }
        let token = self.record(
            number,
            label.unwrap_or_else(|| "Correct chart".to_owned()),
            &before,
            &after,
        )?;
        self.state(number, Some(token))
            .map(Reply::Corrections)
            .ok_or_else(|| super::unknown_dataset(dataset))
    }

    pub(super) fn corrections_export(
        &self,
        dataset: &str,
        path: &str,
        notes: bool,
        include_image: bool,
    ) -> Result<Reply, IpcError> {
        let (number, _, chart) = self.chart_open(dataset)?;
        let mut file = chart.file.clone();
        if !notes {
            for edit in &mut file.edits {
                let mut value = serde_json::to_value(&*edit).unwrap_or_default();
                if let Some(object) = value.as_object_mut() {
                    object.remove("note");
                }
                *edit = serde_json::from_value(value).unwrap_or_else(|_| edit.clone());
            }
        }
        if include_image {
            let bytes = chart.bytes.as_ref().ok_or_else(|| {
                IpcError::new(
                    &codes::CORRECTIONS_READ_ONLY,
                    "The chart file is not open, so its image cannot be included.",
                )
            })?;
            file.image = Some(png_image(bytes).ok_or_else(|| {
                IpcError::new(
                    &codes::CORRECTIONS_NOT_HERE,
                    "Only a PNG chart's image can go in a correction file.",
                )
            })?);
        }
        let target = export_target(path)?;
        let text = correction_file::write(&file);
        crate::settings::write_atomically(&target, text.as_bytes(), None).map_err(|detail| {
            IpcError::new(
                &codes::EXPORT_FAILED,
                format!("The corrections could not be exported: {detail}."),
            )
        })?;
        self.state(number, None)
            .map(Reply::Corrections)
            .ok_or_else(|| super::unknown_dataset(dataset))
    }

    pub(super) fn corrections_import(
        &mut self,
        dataset: &str,
        path: &str,
        confirm: bool,
    ) -> Result<Reply, IpcError> {
        let (number, _, chart) = self.chart_open(dataset)?;
        let source = import_source(path)?;
        let size = std::fs::metadata(&source).map_or(u64::MAX, |m| m.len());
        if size > correction_file::MAX_BYTES as u64 {
            return Err(invalid("The correction file is larger than 16 MiB."));
        }
        let bytes =
            std::fs::read(&source).map_err(|_| invalid("The correction file cannot be read."))?;
        let imported = read_text(&String::from_utf8_lossy(&bytes))?;
        if imported.chart.sha256 != chart.file.chart.sha256
            || imported.chart.extractor != chart.file.chart.extractor
        {
            return Err(IpcError::new(
                &codes::CORRECTIONS_NOT_HERE,
                "These corrections were made on another chart, or read by another extractor; nothing was imported.",
            ));
        }
        if chart.bytes.is_none() {
            return Err(IpcError::new(
                &codes::CORRECTIONS_READ_ONLY,
                "Open the chart file again to import corrections into it.",
            ));
        }
        let before = chart.file.clone();
        let plan = self.plan_import(number, &before, &imported)?;
        let calibration_differs = imported.calibration.is_some()
            && imported.calibration != before.calibration
            && before.calibration.is_some();
        let mut preview = ImportPreview {
            dataset: dataset.to_owned(),
            applies: plan.applies.iter().map(|(old, _)| old.clone()).collect(),
            conflicts: plan.conflicts,
            duplicates: plan.duplicates,
            calibration_differs,
            applied: false,
            state: None,
        };
        if confirm {
            let mut after = before.clone();
            after
                .edits
                .extend(plan.applies.into_iter().map(|(_, edit)| edit));
            if after.calibration.is_none() {
                after.calibration.clone_from(&imported.calibration);
            }
            let token = self.record(number, "Import corrections".to_owned(), &before, &after)?;
            preview.applied = true;
            preview.state = self.state(number, Some(token));
        }
        Ok(Reply::CorrectionsImport(preview))
    }

    /// Renumbers the imported edits after the session's, rewrites their references to added
    /// points, skips duplicates and leaves out conflicts: edits whose guard fails when they
    /// run after the session's, and edits on points only a left-out `add` made (§11).
    fn plan_import(
        &self,
        number: u64,
        session: &CorrectionFile,
        imported: &CorrectionFile,
    ) -> Result<Plan, IpcError> {
        let open = self
            .datasets
            .get(&number)
            .ok_or_else(|| super::unknown_dataset(""))?;
        let bytes = open
            .chart
            .as_ref()
            .and_then(|c| c.bytes.clone())
            .ok_or_else(|| super::unknown_dataset(""))?;
        let mut next = session.edits.iter().map(number_of).max().unwrap_or(0) + 1;
        let existing: BTreeMap<String, String> = session
            .edits
            .iter()
            .map(|e| (content(e).to_string(), e.id().to_owned()))
            .collect();
        let mut mapping: BTreeMap<String, String> = BTreeMap::new();
        let mut duplicates = Vec::new();
        let mut candidates: Vec<(String, FileEdit)> = Vec::new();
        for edit in &imported.edits {
            let point = edit.point().map(|p| match p {
                PointRef::Added(target) => PointRef::Added(
                    target
                        .strip_prefix('+')
                        .and_then(|t| mapping.get(t))
                        .map_or_else(|| target.clone(), |n| format!("+{n}")),
                ),
                PointRef::Index(i) => PointRef::Index(*i),
            });
            let rewritten = with_id(edit, edit.id(), point.clone());
            if let Some(same) = existing.get(&content(&rewritten).to_string()) {
                mapping.insert(edit.id().to_owned(), same.clone());
                duplicates.push(edit.id().to_owned());
                continue;
            }
            let id = format!("e{next}");
            next += 1;
            mapping.insert(edit.id().to_owned(), id.clone());
            candidates.push((edit.id().to_owned(), with_id(edit, &id, point)));
        }
        let mut conflicts = Vec::new();
        loop {
            let mut trial = session.clone();
            trial
                .edits
                .extend(candidates.iter().map(|(_, e)| e.clone()));
            // A candidate on an added point whose `add` was left out cannot apply either.
            let added: Vec<String> = trial
                .edits
                .iter()
                .filter(|e| matches!(e, FileEdit::Add { .. }))
                .map(|e| e.id().to_owned())
                .collect();
            let orphan = candidates.iter().position(|(_, e)| {
                e.point()
                    .and_then(PointRef::added)
                    .is_some_and(|target| !added.iter().any(|a| a == target))
            });
            if let Some(index) = orphan {
                conflicts.push(candidates.remove(index).0);
                continue;
            }
            let options = sonarithm::ExtractOptions::new()
                .with_corrections(CorrectionSet::File(Box::new(trial)));
            let chart = sonarithm::read_chart_bytes(&bytes, &open.summary.name, &options).map_err(
                |err| {
                    err.diagnostics().iter().next().map_or_else(
                        || invalid("The chart could not be read again."),
                        IpcError::from_diagnostic,
                    )
                },
            )?;
            let failed: Vec<String> = chart
                .report()
                .corrections
                .iter()
                .filter(|r| !r.applied)
                .map(|r| r.edit.clone())
                .collect();
            let Some(index) = candidates
                .iter()
                .position(|(_, e)| failed.iter().any(|f| f == e.id()))
            else {
                break;
            };
            conflicts.push(candidates.remove(index).0);
        }
        Ok(Plan {
            applies: candidates,
            conflicts,
            duplicates,
        })
    }

    /// Writes each chart dataset's corrections into the project's members.
    pub(super) fn corrections_into(&self, reserved: &mut BTreeMap<String, Vec<u8>>) {
        for open in self.datasets.values() {
            let key = member(&open.summary.id);
            reserved.remove(&key);
            if let Some(chart) = open.chart.as_ref().filter(|c| c.has_content()) {
                reserved.insert(key, correction_file::write(&chart.file).into_bytes());
            }
        }
    }
}

fn number_of(edit: &FileEdit) -> u64 {
    number(edit.id())
}

/// The edits an import appends (`(id in the file, renumbered edit)`), and the ones it does
/// not.
struct Plan {
    applies: Vec<(String, FileEdit)>,
    conflicts: Vec<String>,
    duplicates: Vec<String>,
}

/// The corrections a reopened project keeps for dataset `id`, read-only.
pub(super) fn from_member(reserved: &BTreeMap<String, Vec<u8>>, id: &str) -> Option<ChartState> {
    let bytes = reserved.get(&member(id))?;
    let (file, _) = correction_file::read(bytes).ok()?;
    Some(ChartState {
        bytes: None,
        file,
        not_applied: Vec::new(),
    })
}

/// A PNG chart as a correction file's image (RFC-0005 §6): only the chunks that draw it
/// (`IHDR`, `PLTE`, `tRNS`, `IDAT`, `IEND`), so no text, time or EXIF chunk travels, as
/// base64, with the whole image as its region. `None` for anything but a well-formed PNG.
fn png_image(bytes: &[u8]) -> Option<ChartImage> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    let rest = bytes.strip_prefix(SIGNATURE)?;
    let mut out = SIGNATURE.to_vec();
    let mut at = 0;
    let (mut width, mut height) = (0_u32, 0_u32);
    while at + 12 <= rest.len() {
        let length =
            usize::try_from(u32::from_be_bytes(rest.get(at..at + 4)?.try_into().ok()?)).ok()?;
        let end = at.checked_add(12)?.checked_add(length)?;
        let chunk = rest.get(at..end)?;
        let kind = chunk.get(4..8)?;
        if kind == b"IHDR" {
            width = u32::from_be_bytes(chunk.get(8..12)?.try_into().ok()?);
            height = u32::from_be_bytes(chunk.get(12..16)?.try_into().ok()?);
        }
        if [b"IHDR", b"PLTE", b"tRNS", b"IDAT", b"IEND"]
            .iter()
            .any(|k| kind == *k)
        {
            out.extend_from_slice(chunk);
        }
        at = end;
        if kind == b"IEND" {
            break;
        }
    }
    let data = crate::ipc::typescript::base64(&out);
    if width == 0 || height == 0 || data.len() > correction_file::MAX_IMAGE {
        return None;
    }
    serde_json::from_value(serde_json::json!({
        "media": "image/png",
        "data": data,
        "region": { "min": [0, 0], "max": [width, height] },
    }))
    .ok()
}

/// A file to export to: an absolute `.json` path in an existing folder.
fn export_target(path: &str) -> Result<PathBuf, IpcError> {
    let path = Path::new(path);
    let name = path.file_name().filter(|_| path.is_absolute());
    let folder = path.parent().and_then(|f| f.canonicalize().ok());
    let refuse = || {
        IpcError::new(
            &codes::EXPORT_PATH,
            "Export corrections to a .json file in an existing folder.",
        )
    };
    let (Some(name), Some(folder)) = (name, folder) else {
        return Err(refuse());
    };
    if !name
        .to_string_lossy()
        .to_ascii_lowercase()
        .ends_with(".json")
    {
        return Err(refuse());
    }
    let target = folder.join(name);
    if target.is_dir() {
        return Err(refuse());
    }
    Ok(target)
}

/// A correction file to import: an existing regular `.json` file. Errors name the file only.
fn import_source(path: &str) -> Result<PathBuf, IpcError> {
    let path = Path::new(path);
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let refuse = |why: &str| IpcError::new(&codes::FILE_REFUSED, format!("`{name}` {why}"));
    if !name.to_ascii_lowercase().ends_with(".json") {
        return Err(refuse("is not a correction file (.json)."));
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| refuse("cannot be found."))?;
    if !canonical.is_file() {
        return Err(refuse("is not a file."));
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::png_image;

    fn chunk(kind: [u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = u32::try_from(data.len()).unwrap().to_be_bytes().to_vec();
        out.extend_from_slice(&kind);
        out.extend_from_slice(data);
        out.extend_from_slice(&[0; 4]);
        out
    }

    /// RFC-0005 §6: only the chunks that draw the image travel; text and time chunks,
    /// which can carry personal data, are dropped.
    #[test]
    fn png_images_lose_their_metadata() {
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = 640_u32.to_be_bytes().to_vec();
        header.extend_from_slice(&480_u32.to_be_bytes());
        header.extend_from_slice(&[8, 2, 0, 0, 0]);
        png.extend(chunk(*b"IHDR", &header));
        png.extend(chunk(*b"tEXt", b"Author\0Someone"));
        png.extend(chunk(*b"tIME", &[7, 234, 10, 30, 12, 0, 0]));
        png.extend(chunk(*b"IDAT", &[1, 2, 3]));
        png.extend(chunk(*b"IEND", &[]));
        let image = serde_json::to_value(png_image(&png).unwrap()).unwrap();
        assert_eq!(image["media"], "image/png");
        assert_eq!(image["region"]["max"], serde_json::json!([640.0, 480.0]));
        let data = image["data"].as_str().unwrap();
        let kept = crate::ipc::typescript::base64(&{
            let mut kept = b"\x89PNG\r\n\x1a\n".to_vec();
            kept.extend(chunk(*b"IHDR", &header));
            kept.extend(chunk(*b"IDAT", &[1, 2, 3]));
            kept.extend(chunk(*b"IEND", &[]));
            kept
        });
        assert_eq!(data, kept);
        assert!(png_image(b"GIF89a").is_none());
    }
}
