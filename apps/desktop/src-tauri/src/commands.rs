// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The host's commands (ADR-0017 §2, §6): one generic handler for JSON replies, one for
//! binary views, and two for event channels. Each only checks that the request is a
//! protocol message and calls `sonarithm-app`; the app core validates everything else,
//! paths included (ADR-0017 §7). Commands run on Tauri's worker pool (`async`), never on the
//! window's thread, so a long dispatch cannot freeze the UI.

use std::sync::{Arc, Mutex, PoisonError};

use sonarithm_app::App;
use sonarithm_app::codes;
use sonarithm_app::ipc::{Channel, IpcError, Reply, Request};
use tauri::State;
use tauri::ipc::Response;

use crate::channels::Router;

/// The app core, behind one lock as ADR-0017 §1 asks; empty once the app has shut down.
pub(crate) struct Core(Mutex<Option<App>>);

impl Core {
    pub(crate) fn new(app: App) -> Self {
        Self(Mutex::new(Some(app)))
    }

    /// Runs `f` on the app core.
    fn with<T>(&self, f: impl FnOnce(&mut App) -> Result<T, IpcError>) -> Result<T, IpcError> {
        let mut guard = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        match guard.as_mut() {
            Some(app) => f(app),
            None => Err(IpcError::new(
                &codes::CONTROL_THREAD,
                "SONARITHM is closing and no longer takes commands.",
            )),
        }
    }

    /// Ends the session cleanly (RFC-0002 §8).
    pub(crate) fn shutdown(&self) {
        let app = self.0.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(app) = app {
            app.shutdown();
        }
    }
}

/// Parses a request as the protocol's [`Request`], with the app core's error for anything
/// else (`SNR-APP-009`), so the UI sees one error shape whatever went wrong.
pub(crate) fn parse(request: serde_json::Value) -> Result<Request, IpcError> {
    serde_json::from_value(request).map_err(|err| {
        IpcError::new(
            &codes::BAD_REQUEST,
            format!("The request is not a valid IPC message: {err}."),
        )
    })
}

/// A command with a JSON reply: `App::dispatch`.
#[tauri::command(async)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri passes command arguments by value"
)]
pub(crate) fn ipc_dispatch(
    core: State<'_, Core>,
    request: serde_json::Value,
) -> Result<Reply, IpcError> {
    let request = parse(request)?;
    core.with(|app| app.dispatch(request))
}

/// A command with a binary reply (`view_get`, `table_page`): `App::dispatch_binary`. The
/// bytes travel as a raw IPC body, never as JSON (F-041, F-042).
#[tauri::command(async)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri passes command arguments by value"
)]
pub(crate) fn ipc_binary(
    core: State<'_, Core>,
    request: serde_json::Value,
) -> Result<Response, IpcError> {
    let request = parse(request)?;
    core.with(|app| app.dispatch_binary(request))
        .map(Response::new)
}

/// Sends every event of `channel` to `on_event` until unsubscribed; returns the
/// subscription's ID.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri passes command arguments by value"
)]
pub(crate) fn ipc_subscribe(
    router: State<'_, Arc<Router>>,
    channel: Channel,
    on_event: tauri::ipc::Channel,
) -> u64 {
    router.subscribe(channel, Box::new(on_event))
}

/// The native announcer route (B03-T04): the UI's announcer chose it; the host only makes
/// the platform call. Synchronous, so Tauri runs it on the window's thread, as UI
/// Automation expects.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri passes command arguments by value"
)]
pub(crate) fn announce_native(
    window: tauri::WebviewWindow,
    text: String,
    assertive: bool,
) -> Result<(), String> {
    crate::platform::announce(&window, &text, assertive)
}

/// Ends a subscription; unknown IDs are ignored.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri passes command arguments by value"
)]
pub(crate) fn ipc_unsubscribe(router: State<'_, Arc<Router>>, id: u64) {
    router.unsubscribe(id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_protocol_request_parses() {
        let request = parse(serde_json::json!({ "command": "transport_play" }));
        assert_eq!(request, Ok(Request::TransportPlay));
    }

    #[test]
    fn anything_else_is_the_app_cores_bad_request_error() {
        let err = parse(serde_json::json!({ "command": "format_disk" }))
            .expect_err("an unknown command must not parse");
        assert_eq!(err.code, codes::BAD_REQUEST.code);
    }
}
