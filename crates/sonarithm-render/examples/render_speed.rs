// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PB-03 real-time factor: renders the 60 s benchmark Score (DS-006: 4 tracks, 10,000
//! events) offline five times on one thread and prints the median real-time factor
//! (audio seconds per second of render time), as the benchmarks log records it.
//!
//! Run: `cargo run -p sonarithm-render --example render_speed --release [-- BLOCK]`

#![allow(
    clippy::print_stdout,
    clippy::unwrap_used,
    reason = "a measurement program reports to the console"
)]

use std::time::Instant;

use sonarithm_core::SampleRate;
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::{OfflineRender, PlanConfig, RenderPlan};

fn main() {
    let block = std::env::args()
        .nth(1)
        .and_then(|b| b.parse().ok())
        .unwrap_or(256_usize);
    let score = benchmark_score(60.0, BENCHMARK_SEED);
    let started = Instant::now();
    let planned = RenderPlan::build(&score, &PlanConfig::new(SampleRate::HZ_48000)).unwrap();
    let plan_ms = started.elapsed().as_secs_f64() * 1e3;
    let plan = planned.plan;
    println!(
        "benchmark Score: {} tracks, {} events, {} voices; plan built in {plan_ms:.1} ms",
        plan.tracks.len(),
        plan.event_count(),
        plan.voice_count()
    );
    let mut factors: Vec<f64> = (0..5)
        .map(|_| {
            let started = Instant::now();
            let samples = OfflineRender::new(plan.clone(), block).render_all();
            let seconds = started.elapsed().as_secs_f64();
            std::hint::black_box(samples);
            60.0 / seconds
        })
        .collect();
    factors.sort_by(f64::total_cmp);
    println!(
        "block {block}: real-time factor median {:.0}x (runs {:.0}x to {:.0}x), budget 50x (PB-03)",
        factors[2], factors[0], factors[4]
    );
}
