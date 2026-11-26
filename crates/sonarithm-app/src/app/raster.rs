// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Raster chart calibration in the app core (C02-T10; ADR-0017 "Raster calibration").
//!
//! A PNG or JPEG chart is read only through a calibration (`SNR-ENG-328`), so it is not
//! opened like other data. `chart_raster_open` decodes it under PLOTSONA's raster budgets and
//! keeps its bytes in a session; the dialog checks anchors as the user enters them
//! (`chart_calibration_check`, fitting and the 10 §3.1 checks, no extraction); and
//! `chart_calibration_apply` reads the chart through the calibration and opens it as a
//! dataset, keeping the calibration in its correction document so corrections re-read it the
//! same way (C02-T11). At most [`MAX_SESSIONS`] are open; a fifth drops the oldest, whose
//! later calls give `SNR-APP-096`. Nothing here reads a clock.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sonarithm::raster::{CalibrationCheck, ReadValue};
use sonarithm_core::TableId;
use sonarithm_data::BundleId;
use sonarithm_spec::v02::Calibration;

use super::corrections::ChartState;
use super::{App, Open, file_name, summarize};
use crate::codes;
use crate::ipc::{
    AnchorText, AxisFitInfo, DiagnosticInfo, IpcError, PlotAreaInfo, RasterCalibration, Reply,
    Request, ValueAt,
};

/// The most calibration sessions open at once; a fifth drops the oldest.
pub(super) const MAX_SESSIONS: usize = 4;

/// The largest chart image, in bytes. Its `data:` URL is about a third larger (base64), so a
/// reply carries at most about 27 MB.
pub(super) const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

/// The extensions of the chart images this opens. The bytes' signature decides the format.
const RASTER_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

/// A chart image waiting for its calibration.
#[derive(Debug)]
struct Pending {
    /// The file name alone.
    name: String,
    bytes: Arc<Vec<u8>>,
}

/// The open calibration sessions, by ID, oldest first.
#[derive(Debug)]
pub(super) struct Sessions {
    open: BTreeMap<u64, Pending>,
    next: u64,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            open: BTreeMap::new(),
            next: 1,
        }
    }
}

fn no_session(id: u64) -> IpcError {
    IpcError::new(
        &codes::RASTER_SESSION,
        format!("No chart image is open for calibration with the ID {id}."),
    )
}

/// Checks a chart image the user chose: a regular file with a raster extension, no larger
/// than [`MAX_IMAGE_BYTES`]. Errors name the file alone, never its folder.
fn raster_file(path: &Path) -> Result<PathBuf, IpcError> {
    let name = file_name(path);
    let refuse = |why: &str| IpcError::new(&codes::FILE_REFUSED, format!("`{name}` {why}"));
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    if !extension
        .as_deref()
        .is_some_and(|ext| RASTER_EXTENSIONS.contains(&ext))
    {
        return Err(refuse("is not a PNG or JPEG chart image."));
    }
    let canonical = std::fs::canonicalize(path)
        .map_err(|_| refuse("cannot be opened: it does not exist or cannot be read."))?;
    let meta = std::fs::metadata(&canonical)
        .ok()
        .filter(std::fs::Metadata::is_file)
        .ok_or_else(|| refuse("is not a regular file."))?;
    if meta.len() > MAX_IMAGE_BYTES {
        return Err(refuse("is larger than 20 MB, the most a chart image may be."));
    }
    Ok(canonical)
}

/// The spec's calibration for what the dialog entered: the same JSON, which the spec type
/// reads (RFC-0004 §5).
fn spec_calibration(calibration: &RasterCalibration) -> Result<Calibration, IpcError> {
    let mut value = serde_json::json!({
        "axes": { "x": calibration.x, "y": calibration.y },
    });
    if let Some(area) = calibration.plot_area {
        value["plotArea"] = serde_json::json!({ "min": area.min, "max": area.max });
    }
    serde_json::from_value(value).map_err(|err| {
        IpcError::new(
            &codes::BAD_REQUEST,
            format!("The calibration is not valid: {err}."),
        )
    })
}

fn read_value(value: Option<&ReadValue>) -> Option<AnchorText> {
    match value? {
        ReadValue::Number(n) => Some(AnchorText::Number(*n)),
        ReadValue::Text(text) => Some(AnchorText::Text(text.clone())),
        _ => None,
    }
}

fn check_reply(checked: &CalibrationCheck, at: bool) -> Reply {
    let fit = |axis: &Option<sonarithm::raster::AxisFit>| {
        axis.map(|fit| AxisFitInfo {
            anchors: u32::try_from(fit.anchors).unwrap_or(u32::MAX),
            residual: fit.residual,
            confidence: fit.confidence,
        })
    };
    Reply::CalibrationCheck {
        diagnostics: checked.diagnostics.iter().map(DiagnosticInfo::from).collect(),
        x: fit(&checked.x),
        y: fit(&checked.y),
        value_at: at.then(|| ValueAt {
            x: read_value(checked.value_at[0].as_ref()),
            y: read_value(checked.value_at[1].as_ref()),
        }),
    }
}

/// Base64 (RFC 4648 §4, with padding), for the image's `data:` URL.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let symbol = |shift: u32| char::from(ALPHABET[((n >> shift) & 63) as usize]);
        out.push(symbol(18));
        out.push(symbol(12));
        out.push(if chunk.len() > 1 { symbol(6) } else { '=' });
        out.push(if chunk.len() > 2 { symbol(0) } else { '=' });
    }
    out
}

impl App {
    /// The raster calibration commands (ADR-0017 "Raster calibration").
    pub(super) fn raster_request(&mut self, request: Request) -> Result<Reply, IpcError> {
        match request {
            Request::ChartRasterOpen { path } => self.raster_open(Path::new(&path)),
            Request::ChartCalibrationCheck {
                id,
                calibration,
                at,
            } => self.calibration_check(id, &calibration, at),
            Request::ChartCalibrationApply { id, calibration } => {
                self.edit(|app| app.calibration_apply(id, &calibration))
            }
            Request::ChartRasterClose { id } => self.raster_close(id),
            _ => Err(IpcError::new(
                &codes::BAD_REQUEST,
                "This command is not a raster calibration command.",
            )),
        }
    }

    fn raster_open(&mut self, path: &Path) -> Result<Reply, IpcError> {
        let canonical = raster_file(path)?;
        let name = file_name(&canonical);
        let bytes = std::fs::read(&canonical).map_err(|_| {
            IpcError::new(
                &codes::FILE_REFUSED,
                format!("`{name}` cannot be opened: it does not exist or cannot be read."),
            )
        })?;
        // Decoded under the budgets before anything is encoded or kept.
        let info = sonarithm::raster::inspect(&bytes, &sonarithm::Budgets::DEFAULT)
            .map_err(|diagnostic| IpcError::from_diagnostic(&diagnostic))?;
        let image = format!("data:{};base64,{}", info.mime, base64(&bytes));
        let sessions = &mut self.raster;
        let id = sessions.next;
        sessions.next += 1;
        sessions.open.insert(
            id,
            Pending {
                name: name.clone(),
                bytes: Arc::new(bytes),
            },
        );
        while sessions.open.len() > MAX_SESSIONS {
            sessions.open.pop_first();
        }
        Ok(Reply::RasterChart {
            id,
            name,
            width_px: info.width,
            height_px: info.height,
            image,
            plot_area: info.plot.map(|rect| PlotAreaInfo {
                min: rect.min,
                max: rect.max,
            }),
        })
    }

    fn calibration_check(
        &self,
        id: u64,
        calibration: &RasterCalibration,
        at: Option<[f64; 2]>,
    ) -> Result<Reply, IpcError> {
        if !self.raster.open.contains_key(&id) {
            return Err(no_session(id));
        }
        let calibration = spec_calibration(calibration)?;
        let checked = sonarithm::raster::check(&calibration, at);
        Ok(check_reply(&checked, at.is_some()))
    }

    fn calibration_apply(
        &mut self,
        id: u64,
        calibration: &RasterCalibration,
    ) -> Result<Reply, IpcError> {
        let pending = self.raster.open.get(&id).ok_or_else(|| no_session(id))?;
        let calibration = spec_calibration(calibration)?;
        let number = self.next_dataset;
        let ids = u32::try_from(number).unwrap_or(u32::MAX);
        let options = sonarithm::ExtractOptions::new()
            .with_ids(BundleId(number), TableId(ids))
            .with_calibration(calibration.clone());
        let chart = sonarithm::read_chart_bytes(&pending.bytes, &pending.name, &options)
            .map_err(|err| {
                err.diagnostics().iter().next().map_or_else(
                    || {
                        IpcError::new(
                            &codes::FILE_REFUSED,
                            format!("`{}` holds no chart.", pending.name),
                        )
                    },
                    IpcError::from_diagnostic,
                )
            })?;
        let mut diagnostics = sonarithm_core::Diagnostics::new();
        diagnostics.push(sonarithm::summary_diagnostic(&chart));
        for diagnostic in &chart.report().diagnostics {
            diagnostics.push(diagnostic.clone());
        }
        let extractor = chart.report().extractor.clone();
        let version = chart.bundle().source().extractor_version.clone();
        let Some(pending) = self.raster.open.remove(&id) else {
            return Err(no_session(id));
        };
        let bytes = Arc::try_unwrap(pending.bytes).unwrap_or_else(|shared| (*shared).clone());
        let mut state = ChartState::new(bytes, &pending.name, &extractor, &version);
        state.file.calibration = Some(calibration);
        let bundle = chart.into_bundle();
        self.next_dataset += 1;
        let summary = summarize(&format!("d{number}"), &pending.name, &bundle, &diagnostics);
        self.datasets.insert(
            number,
            Open {
                summary: summary.clone(),
                bundle,
                // A raster chart is embedded in a project, not linked: the project keeps
                // its table and its correction document, which holds the calibration.
                origin: None,
                chart: Some(state),
                function: None,
            },
        );
        Ok(Reply::Dataset(summary))
    }

    fn raster_close(&mut self, id: u64) -> Result<Reply, IpcError> {
        self.raster
            .open
            .remove(&id)
            .map(|_| Reply::RasterClosed { id })
            .ok_or_else(|| no_session(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc_4648_test_vectors() {
        for (input, output) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(input.as_bytes()), output);
        }
    }
}
