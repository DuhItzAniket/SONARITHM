// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM audio output: the `AudioBackend` trait, the cpal backend, device enumeration,
//! hot-plug handling and latency reporting. This is the only crate that may depend on cpal
//! (docs/architecture/02-repository-and-crate-layout.md §3 rule 3).
//!
//! Skeleton created in phase A01; content arrives in A05 (docs/architecture/06-audio-engine.md).
//!
//! This crate is on the unsafe register (coding-standards §5), so it does not
//! `forbid(unsafe_code)`; the workspace-wide `deny` applies until registered unsafe code lands.

#![deny(missing_docs)]
