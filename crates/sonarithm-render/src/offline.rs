// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The offline driver (06 §6, `OfflineBackend`): a plain loop at a fixed block size over
//! the same [`Graph::process`] that real-time playback calls (P4), so an export sounds
//! exactly like playback.
//!
//! No floating-point mode is changed for the render. The DSP keeps its state free of
//! subnormal numbers by construction (filters flush their state, envelopes settle at exact
//! levels), so the real-time callback and this driver compute the same samples on every
//! platform, wasm32 included (coding-standards §10, §11.4).

use sonarithm_core::{Frames, SampleRate};

use crate::graph::{Command, Graph, GraphConfig};
use crate::plan::RenderPlan;

/// The default offline block size: 256 frames (testing-strategy §5.1).
pub const DEFAULT_BLOCK: usize = 256;

/// Renders a plan from its start to its end in fixed blocks.
pub struct OfflineRender {
    graph: Graph,
    block: usize,
    length: u64,
    rendered: u64,
}

impl OfflineRender {
    /// A render of `plan` in blocks of `block_size` frames (at least 1), playing from the
    /// start. Allocates the graph.
    #[must_use]
    pub fn new(plan: RenderPlan, block_size: usize) -> Self {
        let block = block_size.max(1);
        let length = plan.length.0;
        let mut graph = Graph::new(plan, GraphConfig::new(block));
        graph.apply(Command::Play);
        Self {
            graph,
            block,
            length,
            rendered: 0,
        }
    }

    /// The sample rate of the plan.
    #[must_use]
    pub fn sample_rate(&self) -> SampleRate {
        self.graph.plan().sample_rate
    }

    /// The block size in frames.
    #[must_use]
    pub fn block_size(&self) -> usize {
        self.block
    }

    /// The total length in frames.
    #[must_use]
    pub fn length(&self) -> Frames {
        Frames(self.length)
    }

    /// Frames not rendered yet.
    #[must_use]
    pub fn remaining(&self) -> u64 {
        self.length - self.rendered
    }

    /// Renders the next block into the start of `out` (interleaved stereo; at least
    /// `2 × block_size` samples) and returns the samples written: a full block, a shorter
    /// last block, or nothing at the end.
    pub fn next_block<'a>(&mut self, out: &'a mut [f32]) -> &'a [f32] {
        let frames = usize::try_from(self.remaining())
            .unwrap_or(usize::MAX)
            .min(self.block)
            .min(out.len() / 2);
        let block = &mut out[..2 * frames];
        if frames > 0 {
            self.graph.process(block);
            self.rendered += frames as u64;
        }
        block
    }

    /// Renders everything into one interleaved stereo buffer.
    #[must_use]
    pub fn render_all(mut self) -> Vec<f32> {
        let total = usize::try_from(self.length).unwrap_or(usize::MAX);
        let mut out = Vec::with_capacity(2 * total);
        let mut block = vec![0.0; 2 * self.block];
        loop {
            let samples = self.next_block(&mut block);
            if samples.is_empty() {
                break;
            }
            out.extend_from_slice(samples);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use sonarithm_core::{
        Cents, EngineId, InstrumentRef, Mode, Note, Score, ScoreEvent, ScoreHeader, ScoreTrack,
        Seconds, SpecVersion, TrackId, TrackRole,
    };

    use super::*;
    use crate::plan::PlanConfig;

    fn plan() -> RenderPlan {
        let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 1);
        header.duration = Seconds(0.5);
        let mut score = Score::new(header);
        let mut track = ScoreTrack::new(
            TrackId::new("t"),
            TrackRole::Data,
            EngineId::from_static("numisona"),
            InstrumentRef("mallet".to_owned()),
        );
        track.events = vec![ScoreEvent::Note(Note::new(
            Seconds(0.1),
            Seconds(0.3),
            Cents(0.0),
        ))];
        score.tracks = vec![track];
        RenderPlan::build(&score, &PlanConfig::new(SampleRate::HZ_48000))
            .unwrap()
            .plan
    }

    #[test]
    fn the_render_covers_the_plan_exactly_in_any_block_size() {
        let reference = OfflineRender::new(plan(), DEFAULT_BLOCK).render_all();
        assert_eq!(reference.len(), 2 * 24_000);
        assert!(reference.iter().any(|s| s.abs() > 0.1));
        for block in [1, 100, 1_000, 30_000] {
            assert!(
                OfflineRender::new(plan(), block).render_all() == reference,
                "block {block}"
            );
        }
    }

    #[test]
    fn the_last_block_is_short_and_then_empty() {
        let mut render = OfflineRender::new(plan(), 1_000);
        let mut buffer = vec![0.0; 2_000];
        for _ in 0..24 {
            assert_eq!(render.next_block(&mut buffer).len(), 2_000);
        }
        assert_eq!(render.remaining(), 0);
        assert_eq!(render.next_block(&mut buffer).len(), 0);
    }
}
