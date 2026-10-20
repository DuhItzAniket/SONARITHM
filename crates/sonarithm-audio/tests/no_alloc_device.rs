// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A05-T10: no allocation in the device callback, through a whole backend (the null
//! device), with commands arriving while it runs. Needs the `count-allocations` feature:
//! `cargo test -p sonarithm-audio --features count-allocations --test no_alloc_device`.

#![cfg(feature = "count-allocations")]
#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::time::{Duration, Instant};

use sonarithm_audio::{NullBackend, NullConfig, Player, StreamConfig};
use sonarithm_core::{Db, Frames, Hz, Pan, SampleRate, Seconds};
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::graph::{Command, Cue, GainTarget, LoopRange};
use sonarithm_render::{Instrument, PlanConfig};

#[test]
fn the_device_callback_never_allocates() {
    let mut config = NullConfig::new(48_000, 128);
    config.paced = false;
    let mut player = Player::open(
        NullBackend::new(config),
        benchmark_score(8.0, BENCHMARK_SEED),
        PlanConfig::new(SampleRate::HZ_48000),
        StreamConfig::default(),
        256,
    )
    .unwrap();
    player.send(Command::SetLoop(Some(LoopRange {
        start: Frames(0),
        end: Frames(8 * 48_000),
    })));
    player.send(Command::Play);
    let start = Instant::now();
    let mut step = 0_u64;
    while start.elapsed() < Duration::from_secs(3) {
        step += 1;
        let command = match step % 4 {
            0 => Command::Seek(Frames((step * 9_973) % 300_000)),
            1 => Command::SetGain(GainTarget::Master, Db(-((step % 20) as f64))),
            2 => Command::Cue(Cue {
                frequency: Hz(880.0),
                gain: Db(-18.0),
                pan: Pan::CENTRE,
                instrument: Instrument::Sine,
                duration: Seconds(0.1),
            }),
            _ => Command::SetSpeed(1.0 + (step % 3) as f64 * 0.25),
        };
        player.send(command);
        player.tick().unwrap();
        std::thread::sleep(Duration::from_millis(2));
    }
    let stream = player.stream().unwrap();
    let callbacks = stream.status().callbacks();
    // Debug builds render the benchmark Score at about half real time: hundreds of callbacks.
    assert!(callbacks > 200, "only {callbacks} callbacks");
    assert_eq!(stream.status().allocations(), 0, "callbacks that allocated");
    assert!(!stream.status().fault());
}
