// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Render time per block against the device deadline (PB-02, testing-strategy §9.3): plays
//! the 60 s benchmark Score (DS-006) through a playback graph in blocks of BLOCK frames and
//! prints percentiles of the time each `Graph::process` call takes, next to the block's
//! duration at 48 kHz. A device misses a deadline only when a block takes longer than that.
//!
//! Run: `cargo run -p sonarithm-render --example block_times --release [-- BLOCK]`

#![allow(
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a measurement program reports to the console"
)]

use std::time::Instant;

use sonarithm_core::SampleRate;
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::graph::Command;
use sonarithm_render::{Graph, GraphConfig, PlanConfig, RenderPlan};

fn main() {
    let block = std::env::args()
        .nth(1)
        .and_then(|b| b.parse().ok())
        .unwrap_or(128_usize);
    let score = benchmark_score(60.0, BENCHMARK_SEED);
    let plan = RenderPlan::build(&score, &PlanConfig::new(SampleRate::HZ_48000))
        .unwrap()
        .plan;
    let mut graph = Graph::new(plan, GraphConfig::new(block));
    graph.apply(Command::Play);
    let mut out = vec![0.0_f32; 2 * block];
    let blocks = 60 * 48_000 / block;
    let mut times: Vec<f64> = Vec::with_capacity(blocks);
    for _ in 0..blocks {
        let started = Instant::now();
        graph.process(&mut out);
        times.push(started.elapsed().as_secs_f64() * 1e6);
        std::hint::black_box(&out);
    }
    times.sort_by(f64::total_cmp);
    let at = |q: f64| times[((times.len() - 1) as f64 * q).round() as usize];
    let deadline = block as f64 / 48_000.0 * 1e6;
    println!(
        "block {block} ({deadline:.0} us at 48 kHz), {blocks} blocks: p50 {:.1} us  p99 {:.1} us  p99.9 {:.1} us  max {:.1} us  (max is {:.1}% of the block)",
        at(0.5),
        at(0.99),
        at(0.999),
        at(1.0),
        100.0 * at(1.0) / deadline
    );
}
