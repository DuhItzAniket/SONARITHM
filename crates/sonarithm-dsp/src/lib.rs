// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM DSP: oscillators, envelopes, filters, panners and granular, wavetable and modal
//! nodes; ISO 226 loudness compensation and EBU R128 metering.
//!
//! Skeleton created in phase A01; content arrives in A05 (docs/architecture/06-audio-engine.md).
//!
//! This crate is on the unsafe register (coding-standards §5), so it does not
//! `forbid(unsafe_code)`; the workspace-wide `deny` applies until registered unsafe code lands.

#![deny(missing_docs)]
