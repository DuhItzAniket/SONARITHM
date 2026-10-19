// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PB-03: offline render ≥ 50× real time for the 4-track, 10,000-event benchmark Score
//! (DS-006), single-threaded, 256-frame blocks at 48 kHz. Budget verification runs on
//! reference hardware with `cargo bench -p sonarithm-render` (testing-strategy §9.1); the
//! real-time factor is printed by `cargo run -p sonarithm-render --example render_speed
//! --release`.

#![allow(
    clippy::unwrap_used,
    missing_docs,
    reason = "a benchmark harness, not library code"
)]

use criterion::{Criterion, criterion_group, criterion_main};
use sonarithm_core::SampleRate;
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::{OfflineRender, PlanConfig, RenderPlan};

fn render(c: &mut Criterion) {
    let score = benchmark_score(60.0, BENCHMARK_SEED);
    let config = PlanConfig::new(SampleRate::HZ_48000);
    let plan = RenderPlan::build(&score, &config).unwrap().plan;
    let mut group = c.benchmark_group("pb_03");
    group.sample_size(10);
    group.bench_function("plan_benchmark_score", |b| {
        b.iter(|| RenderPlan::build(&score, &config).unwrap());
    });
    group.bench_function("render_benchmark_score_60s_256", |b| {
        b.iter(|| OfflineRender::new(plan.clone(), 256).render_all());
    });
    group.finish();
}

criterion_group!(benches, render);
criterion_main!(benches);
