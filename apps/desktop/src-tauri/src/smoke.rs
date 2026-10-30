// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The CI smoke test (B01-T03): `sonarithm-desktop --smoke <report.json>` opens the real
//! window with a temporary settings folder and the null audio device, and the UI's smoke
//! module (`src/smoke.ts`) checks the empty workspace, the protocol handshake and that a
//! 1 MB view arrives as binary under the CSP (F-042), and measures the cold start (PB-07,
//! B01-T07) from the host's start to an interactive workspace. The UI hands its results to
//! [`smoke_report`], which adds what only the host can see, writes the report and exits
//! with a failing status if any check failed.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, State};

use crate::platform::Accelerators;

/// Rows in the fixture: an LTTB view of 45 000 of them is about 1 MB (3 columns × 8 bytes).
const ROWS: usize = 50_000;

/// A smoke run: where the report goes and the temporary folder it works in.
pub(crate) struct Smoke {
    report: PathBuf,
    dir: PathBuf,
    accelerators: Arc<AcceleratorState>,
    /// When the host started, in milliseconds since the Unix epoch: the cold start's origin
    /// (PB-07). It is read before the fixture is written, so the measurement includes that
    /// smoke-only work and is an upper bound.
    started_at_ms: u64,
}

/// The accelerator setting's outcome, recorded on the window's thread.
#[derive(Default)]
pub(crate) struct AcceleratorState(Mutex<Option<Accelerators>>);

impl AcceleratorState {
    pub(crate) fn record(&self, outcome: Accelerators) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(outcome);
    }

    fn get(&self) -> Option<Accelerators> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Smoke {
    /// `Some` when the command line asks for a smoke run; writes the fixture.
    pub(crate) fn from_args(args: impl Iterator<Item = String>) -> std::io::Result<Option<Self>> {
        let started_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| {
                u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
            });
        let args: Vec<String> = args.collect();
        let Some(report) = args
            .iter()
            .position(|arg| arg == "--smoke")
            .and_then(|i| args.get(i + 1))
        else {
            return Ok(None);
        };
        let dir = std::env::temp_dir().join(format!("sonarithm-smoke-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("config"))?;
        std::fs::write(dir.join("smoke.csv"), fixture(ROWS))?;
        Ok(Some(Self {
            report: PathBuf::from(report),
            dir,
            accelerators: Arc::default(),
            started_at_ms,
        }))
    }

    /// The settings folder: empty, so the run starts from defaults.
    pub(crate) fn config_dir(&self) -> PathBuf {
        self.dir.join("config")
    }

    /// Where the window records the accelerator setting.
    pub(crate) fn accelerators(&self) -> Arc<AcceleratorState> {
        Arc::clone(&self.accelerators)
    }

    /// Tells the UI to run its smoke module, and where the fixture is.
    pub(crate) fn init_script(&self) -> String {
        let config = serde_json::json!({
            "dataset": self.dir.join("smoke.csv"),
            "rows": ROWS,
            "startedAtMs": self.started_at_ms,
        });
        format!("window.__SONARITHM_SMOKE__ = {config};")
    }
}

/// `x,y` rows: a slow sine, so every view has shape.
fn fixture(rows: usize) -> String {
    let mut csv = String::from("x,y\n");
    for i in 0..rows {
        #[expect(
            clippy::cast_precision_loss,
            reason = "row indices stay far below 2^52"
        )]
        let x = i as f64;
        let _ = writeln!(csv, "{x},{:.6}", (x / 500.0).sin());
    }
    csv
}

/// Receives the UI's results, adds the host's, writes the report and exits: status 0 when
/// every check passed.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri passes command arguments by value"
)]
pub(crate) fn smoke_report(
    app: AppHandle,
    smoke: State<'_, Option<Smoke>>,
    results: serde_json::Value,
) -> Result<(), String> {
    let Some(smoke) = smoke.inner() else {
        return Err("Not a smoke run.".to_owned());
    };
    let accelerators = smoke.accelerators.get();
    let report = build_report(&results, accelerators);
    let ok = report["ok"] == true;
    let text = serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?;
    std::fs::write(&smoke.report, text).map_err(|err| err.to_string())?;
    remove(&smoke.dir);
    app.exit(i32::from(!ok));
    Ok(())
}

/// The report: the UI's checks, the host's facts and the verdict.
fn build_report(ui: &serde_json::Value, accelerators: Option<Accelerators>) -> serde_json::Value {
    // Keyboard §12.3: on WebView2 the setting must read back as off.
    let accelerators_ok = match accelerators {
        Some(Accelerators::Disabled) => true,
        Some(Accelerators::None) => !cfg!(windows),
        Some(Accelerators::Failed) | None => false,
    };
    let ui_ok = ui["ok"] == true;
    serde_json::json!({
        "ok": ui_ok && accelerators_ok,
        "ui": ui,
        "host": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "tauri": tauri::VERSION,
            "webview": tauri::webview_version().unwrap_or_default(),
            "browserAccelerators": accelerators,
        },
    })
}

fn remove(dir: &Path) {
    // Best effort: the folder is under the system's temporary directory.
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_flag_no_smoke_run() {
        let run = Smoke::from_args(["sonarithm-desktop".to_owned()].into_iter());
        assert!(matches!(run, Ok(None)));
    }

    #[test]
    fn the_fixture_has_a_header_and_every_row() {
        let csv = fixture(3);
        assert_eq!(csv.lines().count(), 4);
        assert!(csv.starts_with("x,y\n0,0.000000\n"));
    }

    #[test]
    fn kb_04_a_failed_accelerator_setting_fails_the_run() {
        let ui = serde_json::json!({ "ok": true });
        assert_eq!(build_report(&ui, Some(Accelerators::Failed))["ok"], false);
        assert_eq!(build_report(&ui, None)["ok"], false);
        assert_eq!(build_report(&ui, Some(Accelerators::Disabled))["ok"], true);
    }

    #[test]
    fn a_failed_ui_check_fails_the_run() {
        let ui = serde_json::json!({ "ok": false });
        assert_eq!(build_report(&ui, Some(Accelerators::Disabled))["ok"], false);
    }
}
