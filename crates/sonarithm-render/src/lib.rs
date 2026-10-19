// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM render graph: scheduler, `RenderPlan`, voice pool, transport, real-time-safe
//! command and telemetry channels, and the offline driver
//! (docs/architecture/06-audio-engine.md).
//!
//! The plan builder ([`RenderPlan::build`]) turns a Score into frames, frequencies and
//! linear gains off the audio thread (05 §5).
//!
//! This crate is on the unsafe register (coding-standards §5), so it does not
//! `forbid(unsafe_code)`; the workspace-wide `deny` applies until registered unsafe code lands.

#![deny(missing_docs)]

pub mod codes;
pub mod plan;

pub use plan::{
    Bus, EventKind, GlideRef, Instrument, PlanConfig, PlanError, Planned, PlannedBreakpoint,
    PlannedEvent, RenderPlan, Tone, TrackPlan,
};
