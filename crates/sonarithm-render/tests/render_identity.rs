// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The benchmark render is bit-identical to the A05 render graph (PB-03, #185). The speed
//! work changes how voices are computed, never what they compute: the 60 s benchmark Score
//! (DS-006) must render to the same samples, so every golden case and replication record
//! stays valid (ADR-0013). The hash below was taken from the render before the speed work.

#![allow(
    clippy::unwrap_used,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use sonarithm_core::SampleRate;
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::{OfflineRender, PlanConfig, RenderPlan};

/// FNV-1a over the samples' bit patterns: stable across platforms and Rust versions.
fn fnv1a(samples: &[f32]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for sample in samples {
        for byte in sample.to_bits().to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

fn render(block: usize) -> Vec<f32> {
    let score = benchmark_score(60.0, BENCHMARK_SEED);
    let plan = RenderPlan::build(&score, &PlanConfig::new(SampleRate::HZ_48000))
        .unwrap()
        .plan;
    OfflineRender::new(plan, block).render_all()
}

/// The hash of the 60 s benchmark render with the A05 voices (before #185).
const A05_BENCHMARK_HASH: u64 = 4_290_137_234_021_000_538;

#[test]
fn the_benchmark_render_is_bit_identical_to_the_a05_graph() {
    let samples = render(256);
    assert_eq!(samples.len(), 2 * 48_000 * 60, "60 s of stereo frames");
    assert_eq!(fnv1a(&samples), A05_BENCHMARK_HASH);
}

#[test]
fn the_benchmark_render_does_not_depend_on_the_block_size() {
    // An odd block size splits every voice run differently from 256.
    assert_eq!(fnv1a(&render(61)), A05_BENCHMARK_HASH);
}
