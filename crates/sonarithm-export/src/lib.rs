// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM export: WAV, FLAC and MIDI writers, the render manifest (`*.render.json`) and
//! the speech and cue sidecar (docs/architecture/06-audio-engine.md §7).
//!
//! [`write_audio`] drives an [`OfflineRender`](sonarithm_render::OfflineRender) to the
//! end and encodes it block by block: 24-bit PCM or 32-bit float WAV through hound, 24-bit
//! FLAC through flacenc.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audio;

pub use audio::{AudioFormat, AudioReport, ExportError, FlacStage, WavEncoding, write_audio};
