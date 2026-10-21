// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM headless app core (01 §4, 09): everything the desktop app does that is not
//! UI. One [`App`] holds a session (open datasets, the spec, the last compiled Score and its
//! undo history), the transport, a job queue and the settings, and every operation is a
//! request of the IPC protocol ([`ipc`], ADR-0017) through [`App::dispatch`].
//!
//! The Tauri host is a thin adapter over this crate (OD-15): it moves JSON between the
//! webview and `dispatch`, and forwards events to channels. Tests drive the same protocol
//! headless, with the silent null audio device.
//!
//! ```
//! use std::sync::Arc;
//! use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};
//! use sonarithm_app::ipc::{PlaybackStatus, Reply, Request, PROTOCOL_VERSION};
//!
//! # let dir = sonarithm_app::testing::TempDir::new("doc");
//! let config = AppConfig::new(dir.path()).with_audio(AudioOutput::null());
//! let mut app = App::new(config, Arc::new(CollectingSink::new()))?;
//! app.dispatch(Request::Hello { protocol: PROTOCOL_VERSION })?;
//! let spec = r#"{ "specVersion": "0.1",
//!     "data": { "source": { "kind": "inline", "values": [ { "x": 1, "y": 2 }, { "x": 2, "y": 3 } ] } },
//!     "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "y" } } }] }"#;
//! app.dispatch(Request::SpecSet { text: spec.to_owned() })?;
//! let Reply::Compile(report) = app.dispatch(Request::Compile)? else { unreachable!() };
//! assert!(report.ok);
//! // Compiling never starts playback (A11Y-15).
//! let Reply::Transport(state) = app.dispatch(Request::TransportState)? else { unreachable!() };
//! assert_eq!(state.status, PlaybackStatus::Stopped);
//! # Ok::<(), sonarithm_app::ipc::IpcError>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod app;
pub mod codes;
mod events;
mod history;
pub mod ipc;
mod jobs;
pub mod settings;
#[doc(hidden)]
pub mod testing;
mod transport;

pub use app::{App, AppConfig};
pub use events::{CollectingSink, EventSink, NullSink};
pub use transport::AudioOutput;
