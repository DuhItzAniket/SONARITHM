// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! B01-T08 (A11Y-15, T-13): stop all audio takes effect in the next block and is silent
//! once its 20 ms fade (sound-design §5) and the limiter's look-ahead have passed; the
//! sonification volume is gain inside the render graph. No audio backend has a system
//! volume control (`sonarithm-audio`'s `AudioBackend`), so SONARITHM's volume can never
//! change the system's.

#![allow(
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use sonarithm_core::{Db, SampleRate};
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::graph::{Command, GainTarget, PAUSE_FADE};
use sonarithm_render::{Graph, GraphConfig, PlanConfig, RenderPlan};

const RATE: SampleRate = SampleRate::HZ_48000;
const BLOCK: usize = 256;

fn graph() -> Graph {
    let plan = RenderPlan::build(
        &benchmark_score(6.0, BENCHMARK_SEED),
        &PlanConfig::new(RATE),
    )
    .unwrap()
    .plan;
    Graph::new(plan, GraphConfig::new(BLOCK))
}

/// Renders `blocks` blocks and returns each block's peak.
fn run(graph: &mut Graph, blocks: usize) -> Vec<f32> {
    let mut out = vec![0.0; BLOCK * 2];
    (0..blocks)
        .map(|_| {
            graph.process(&mut out);
            out.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()))
        })
        .collect()
}

#[test]
fn a11y_15_stop_all_acts_in_the_next_block_and_is_silent_after_its_fade() {
    let mut graph = graph();
    graph.apply(Command::Play);
    // One second in, past the start-up ramp, so there is sound to stop.
    let playing = run(&mut graph, 190);
    assert!(
        playing.iter().rev().take(20).any(|p| *p > 0.01),
        "the Score sounds"
    );
    let before = *playing.last().unwrap();

    graph.apply(Command::StopAll);
    let fade_frames = (PAUSE_FADE.0 * f64::from(RATE.get())).ceil() as usize;
    let settle_blocks = (fade_frames + graph.latency()).div_ceil(BLOCK) + 1;
    let after = run(&mut graph, settle_blocks + 20);

    // The first block after the command is already fading: it is no louder than before.
    assert!(
        after[0] <= before.max(0.01) * 1.05,
        "{} after {}",
        after[0],
        before
    );
    // Once the fade and the limiter's look-ahead have passed, the output is silent.
    for (i, peak) in after.iter().enumerate().skip(settle_blocks) {
        assert!(*peak < 1e-6, "block {i} after stop all: peak {peak}");
    }
}

#[test]
fn a11y_15_the_sonification_volume_is_gain_in_the_graph() {
    let mut reference = graph();
    let mut quieter = graph();
    reference.apply(Command::Play);
    quieter.apply(Command::Play);
    // A 20 dB step, both under the limiter: the fixture is loud enough to engage it at 0 dB.
    reference.apply(Command::SetGain(GainTarget::Master, Db(-12.0)));
    quieter.apply(Command::SetGain(GainTarget::Master, Db(-32.0)));
    let a = run(&mut reference, 400);
    let b = run(&mut quieter, 400);
    // The comparison holds only where the limiter left the reference alone.
    assert_eq!(reference.limited_frames(), 0, "pick a quieter fixture");
    // After the start-up ramp and the gain smoothing, the output is 20 dB (0.1×) lower.
    let (sum_a, sum_b) = a
        .iter()
        .zip(&b)
        .skip(200)
        .fold((0.0_f64, 0.0_f64), |(x, y), (pa, pb)| {
            (x + f64::from(*pa), y + f64::from(*pb))
        });
    assert!(sum_a > 0.0, "the reference sounds");
    let ratio = sum_b / sum_a;
    assert!((ratio - 0.1).abs() < 0.005, "volume ratio {ratio}");
}
