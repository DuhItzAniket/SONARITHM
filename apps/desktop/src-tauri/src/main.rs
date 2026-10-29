// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM desktop host (09 §1, B01-T03): a thin Tauri 2 adapter over the headless app
//! core. It moves IPC requests between the webview and `sonarithm_app::App::dispatch`,
//! forwards app-core events to Tauri channels and owns the window. Everything else lives in
//! `sonarithm-app`, so another host (Electron, a test harness) replaces only this crate
//! (OD-15, ADR-0017).
//!
//! Command line: `--smoke <report.json>` runs the CI smoke test (`smoke.rs`) and exits.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Only `platform/windows.rs` may use `unsafe` (coding-standards §5, ADR-0025).
#![deny(unsafe_code)]

mod channels;
mod commands;
mod navigation;
mod platform;
mod smoke;

use std::process::ExitCode;
use std::sync::Arc;

use sonarithm_app::{App, AppConfig, AudioOutput};
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};

use crate::channels::Router;
use crate::commands::Core;
use crate::smoke::Smoke;

/// The main window's label; the capability in `capabilities/default.json` names it.
const MAIN_WINDOW: &str = "main";

fn main() -> ExitCode {
    let smoke = match Smoke::from_args(std::env::args()) {
        Ok(smoke) => smoke,
        Err(err) => return fail(&format!("smoke test setup: {err}")),
    };
    let router = Arc::new(Router::default());
    let app = tauri::Builder::default()
        .manage(Arc::clone(&router))
        .manage(smoke)
        .setup(move |app| {
            let smoke = app.state::<Option<Smoke>>().inner();
            let (config_dir, data_dir, audio) = match smoke {
                // The smoke test never touches the user's settings or audio devices.
                Some(smoke) => (smoke.config_dir(), smoke.config_dir(), AudioOutput::null()),
                None => (
                    app.path().app_config_dir()?,
                    app.path().app_data_dir()?,
                    AudioOutput::System,
                ),
            };
            let config = AppConfig::new(config_dir)
                .with_data_dir(data_dir)
                .with_audio(audio);
            let sink: Arc<dyn sonarithm_app::EventSink> = router.clone();
            let core = App::new(config, sink)?;
            app.manage(Core::new(core));

            let mut window = WebviewWindowBuilder::new(app, MAIN_WINDOW, WebviewUrl::default())
                .title(PRODUCT_NAME)
                .inner_size(1280.0, 800.0)
                .min_inner_size(640.0, 480.0)
                // No remote content, ever (09 §4).
                .on_navigation(navigation::allowed);
            if let Some(script) = smoke.as_ref().map(Smoke::init_script) {
                window = window.initialization_script(script);
            }
            let window = window.build()?;
            let state = smoke.as_ref().map(Smoke::accelerators);
            window.with_webview(move |webview| {
                let disabled = platform::disable_browser_accelerators(&webview);
                if let Some(state) = state {
                    state.record(disabled);
                }
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ipc_dispatch,
            commands::ipc_binary,
            commands::ipc_subscribe,
            commands::ipc_unsubscribe,
            smoke::smoke_report,
        ])
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(err) => return fail(&format!("could not start: {err}")),
    };
    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            // An orderly exit discards the autosaves; a crash keeps them (RFC-0002 §8).
            if let Some(core) = handle.try_state::<Core>() {
                core.shutdown();
            }
        }
    });
    ExitCode::SUCCESS
}

/// The window title: the product name, which is not translated (12 §5).
const PRODUCT_NAME: &str = "SONARITHM";

/// Reports a start-up failure on standard error. The message names no user folder.
#[expect(
    clippy::print_stderr,
    reason = "start-up failures have no window to show them in"
)]
fn fail(message: &str) -> ExitCode {
    eprintln!("sonarithm-desktop: {message}");
    ExitCode::FAILURE
}
