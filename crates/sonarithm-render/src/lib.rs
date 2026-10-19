// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM render graph: scheduler, `RenderPlan`, voice pool, transport, real-time-safe
//! command and telemetry channels, and the offline driver
//! (docs/architecture/06-audio-engine.md).
//!
//! The plan builder ([`RenderPlan::build`]) turns a Score into frames, frequencies and
//! linear gains off the audio thread (05 §5). A [`Graph`] plays a plan: voice pools, buses,
//! the master stage, the sample-accurate scheduler and the transport (06 §3, §5). Its
//! [`Graph::process`] is the one render path for real-time playback and offline export.
//!
//! This crate is on the unsafe register (coding-standards §5), so it does not
//! `forbid(unsafe_code)`; the workspace-wide `deny` applies until registered unsafe code lands.

#![deny(missing_docs)]
#![allow(
    clippy::cast_precision_loss,
    reason = "frame counts and indices become f64 positions; all stay below 2^53 (coding-standards §2.2)"
)]

pub mod codes;
mod earcon;
pub mod graph;
pub mod plan;
pub mod rt;
mod voice;

pub use graph::{Command, Cue, GainTarget, Graph, GraphConfig, GraphStats, LoopRange};
pub use plan::{
    Bus, EventKind, GlideRef, Instrument, PlanConfig, PlanError, Planned, PlannedBreakpoint,
    PlannedEvent, RenderPlan, Tone, TrackPlan,
};
pub use rt::{Controller, EngineState, Processor, realtime};
