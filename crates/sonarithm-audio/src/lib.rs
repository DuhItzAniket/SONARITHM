// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM audio output: the [`AudioBackend`] trait, the cpal backend, a timer-driven null
//! device for tests, device enumeration, hot-plug recovery ([`Player`]) and latency
//! reporting (docs/architecture/06-audio-engine.md §6, §11; ADR-0004). This is the only crate
//! that may depend on cpal (02 §3 rule 3).
//!
//! **Hearing safety.** Playback graphs start 20 dB down and rise over 500 ms, and the
//! master limiter holds −1 dBTP (06 §10). On Windows, an OEM enhancement APO can still add
//! gain after the limiter when cpal opens the stream (+20.6 dB on the development laptop,
//! F-032); keep test levels at −30 dBFS or lower on such systems.
//!
//! This crate is on the unsafe register (coding-standards §5), so it does not
//! `forbid(unsafe_code)`; the workspace-wide `deny` applies until registered unsafe code lands.

#![deny(missing_docs)]

mod backend;
pub mod calibration;
mod callback;
pub mod cpal_backend;
#[cfg(windows)]
pub mod latency;
pub mod null;
pub mod player;
#[cfg(windows)]
pub mod wasapi_backend;

pub use backend::{
    AudioBackend, AudioError, DeviceInfo, LatencyInfo, Stream, StreamConfig, StreamStatus,
};
pub use calibration::LatencyCalibration;
pub use cpal_backend::CpalBackend;
pub use null::{NullBackend, NullConfig};
pub use player::{Player, PlayerError, PlayerEvent};
#[cfg(windows)]
pub use wasapi_backend::{WasapiBackend, WasapiMode, WasapiStreamInfo};
