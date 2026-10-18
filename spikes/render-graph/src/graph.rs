// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Draft render-graph contract for `sonarithm-render` (06 §2–§6; ADR-0004). One `process`
//! path serves real-time playback and offline export (P4, SDR-01).
//!
//! The contract has two halves, matching the threading model of 06 §2: a control half that
//! lives on a non-real-time thread, and a processor that is moved to the audio thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::score::Score;
use crate::voice::MathMode;

/// Fixed for the life of a prepared graph.
#[derive(Clone, Copy, Debug)]
pub struct RenderConfig {
    pub sample_rate: u32,
    /// Largest block `process` will be called with, in frames.
    pub max_block: usize,
    pub math: MathMode,
}

/// Counters the processor publishes with relaxed atomics (06 §4, engine state).
#[derive(Debug, Default)]
pub struct Counters {
    /// Score frame of the next block to render.
    pub pos: AtomicU64,
    /// Frames rendered since `prepare` (monotonic, unaffected by seeks).
    pub rendered: AtomicU64,
    pub dispatched: AtomicU64,
    pub stolen: AtomicU64,
    pub max_active: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GraphStats {
    pub voices: usize,
    pub max_active: u64,
    pub dispatched: u64,
    pub stolen: u64,
}

/// The audio-thread half. `process` must not allocate, lock, block or panic (06 §12).
pub trait GraphProcessor: Send {
    /// Renders the next block into interleaved stereo `out` (`out.len() / 2` frames, at most
    /// `max_block`). Every event whose start frame falls inside the block starts at its exact
    /// sample offset.
    fn process(&mut self, out: &mut [f32]);
}

/// The control half: runs on a non-real-time thread and may allocate. Not `Send`: it stays
/// on the thread that built it (Firewheel's context is `!Send`).
pub trait RenderGraph {
    fn name(&self) -> &'static str;

    /// Builds voice pools, buses and the event schedule for `score`; returns the processor.
    fn prepare(
        &mut self,
        score: &Score,
        cfg: &RenderConfig,
    ) -> Result<Box<dyn GraphProcessor>, String>;

    /// Control-thread tick (every few milliseconds in real time, before every block offline):
    /// feeds scheduled events and applies pending commands.
    fn update(&mut self);

    /// Moves the playhead to `frame`; sounding voices stop.
    #[cfg_attr(
        not(any(test, feature = "realtime")),
        expect(dead_code, reason = "used by `realtime` and tests")
    )]
    fn seek(&mut self, frame: u64);

    fn counters(&self) -> &Arc<Counters>;

    fn voices(&self) -> usize;

    fn stats(&self) -> GraphStats {
        let c = self.counters();
        GraphStats {
            voices: self.voices(),
            max_active: c.max_active.load(Ordering::Relaxed),
            dispatched: c.dispatched.load(Ordering::Relaxed),
            stolen: c.stolen.load(Ordering::Relaxed),
        }
    }
}
