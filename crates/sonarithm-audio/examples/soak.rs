// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The device soak (PB-02, testing-strategy §9.3): plays the 60 s benchmark Score (DS-006)
//! in a loop on a real device and reports underruns, late callbacks and, with the
//! `count-allocations` feature, callbacks that allocated.
//!
//! ```text
//! cargo run -p sonarithm-audio --example soak --release [--features count-allocations] -- [options]
//!   --backend wasapi-raw|wasapi-exclusive|cpal|null   (default: wasapi-raw on Windows, cpal elsewhere)
//!   --seconds N        soak length (default 600)
//!   --buffer N         buffer to request in frames (default 128; cpal and null. WASAPI uses the
//!                      engine period in shared mode and the minimum period in exclusive mode)
//!   --level DB         master level in dB (default -120: inaudible). Capped at -30.
//!   --strict           exit with status 1 on any allocating callback or fault (CI). Xruns are
//!                      reported, not gated: CI machines miss deadlines for reasons of their own
//! ```
//!
//! The default level is inaudible: the soak measures the render path and the device timing,
//! not the sound, so it can run unattended (hearing safety).

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a measurement program reports to the console"
)]

use std::process::ExitCode;
use std::time::{Duration, Instant};

use sonarithm_audio::{AudioBackend, CpalBackend, NullBackend, NullConfig, Player, StreamConfig};
use sonarithm_core::{Db, Frames, SampleRate};
use sonarithm_render::PlanConfig;
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::graph::{Command, GainTarget, LoopRange};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let seconds: u64 = value("--seconds")
        .and_then(|s| s.parse().ok())
        .unwrap_or(600);
    let buffer: u32 = value("--buffer")
        .and_then(|s| s.parse().ok())
        .unwrap_or(128);
    let level = value("--level")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(-120.0)
        .min(-30.0);
    let default = if cfg!(windows) { "wasapi-raw" } else { "cpal" };
    let backend = value("--backend").unwrap_or_else(|| default.to_owned());
    let config = StreamConfig::default().with_buffer(buffer);
    let clean = match backend.as_str() {
        "cpal" => soak(CpalBackend::new(), &config, seconds, level),
        "null" => soak(
            NullBackend::new(NullConfig::new(48_000, buffer as usize)),
            &config,
            seconds,
            level,
        ),
        #[cfg(windows)]
        "wasapi-raw" => soak(
            sonarithm_audio::WasapiBackend::new(sonarithm_audio::WasapiMode::SharedRaw),
            &config,
            seconds,
            level,
        ),
        #[cfg(windows)]
        "wasapi-exclusive" => soak(
            sonarithm_audio::WasapiBackend::new(sonarithm_audio::WasapiMode::Exclusive),
            &config,
            seconds,
            level,
        ),
        other => {
            eprintln!("unknown backend `{other}`");
            false
        }
    };
    if clean || !args.iter().any(|a| a == "--strict") {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Runs the soak; true when it ended with no allocating callback or fault.
fn soak<B: AudioBackend>(backend: B, config: &StreamConfig, seconds: u64, level: f64) -> bool {
    let name = backend.name();
    let Some(rate) = backend.sample_rate(None).ok().and_then(SampleRate::new) else {
        eprintln!("no output device");
        return false;
    };
    let mut player = match Player::open(
        backend,
        benchmark_score(60.0, BENCHMARK_SEED),
        PlanConfig::new(rate),
        config.clone(),
        512,
    ) {
        Ok(player) => player,
        Err(error) => {
            eprintln!("cannot open the device: {error}");
            return false;
        }
    };
    player.send(Command::SetGain(GainTarget::Master, Db(level)));
    player.send(Command::SetLoop(Some(LoopRange {
        start: Frames(0),
        end: Frames(60 * u64::from(rate.get())),
    })));
    player.send(Command::Play);
    let (device, buffer) = player
        .stream()
        .map(|s| (s.device().to_owned(), s.latency().buffer_frames))
        .unwrap_or_default();
    println!(
        "soak: {name} on {device}, {} Hz, buffer {buffer:?} frames, {seconds} s, master {level} dB",
        rate.get()
    );
    let start = Instant::now();
    let mut report = Instant::now();
    while start.elapsed() < Duration::from_secs(seconds) {
        if let Err(error) = player.tick() {
            eprintln!("device error: {error}");
        }
        if report.elapsed() >= Duration::from_secs(10) {
            report = Instant::now();
            print_line(&mut player, start.elapsed());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    print_line(&mut player, start.elapsed())
}

/// Prints the counters; true when there was no allocating callback or fault.
fn print_line<B: AudioBackend>(player: &mut Player<B>, elapsed: Duration) -> bool {
    let state = player.state();
    let Some(stream) = player.stream() else {
        return false;
    };
    let status = stream.status();
    println!(
        "{:6.0} s  callbacks {}  xruns {}  late callbacks {}  allocating callbacks {}  faults {}  voices stolen {}",
        elapsed.as_secs_f64(),
        status.callbacks(),
        status.xruns(),
        status.late_callbacks(),
        status.allocations(),
        u8::from(status.fault()),
        state.graph.stolen
    );
    status.allocations() == 0 && !status.fault()
}
