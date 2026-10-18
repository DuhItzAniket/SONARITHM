// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM render graph: scheduler, `RenderPlan`, voice pool, transport, real-time-safe
//! command and telemetry channels, and the offline driver.
//!
//! Skeleton created in phase A01; content arrives in A05 (docs/architecture/06-audio-engine.md).
//!
//! This crate is on the unsafe register (coding-standards §5), so it does not
//! `forbid(unsafe_code)`; the workspace-wide `deny` applies until registered unsafe code lands.

#![deny(missing_docs)]
