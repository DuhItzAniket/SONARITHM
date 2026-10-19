// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Manual hot-plug test (A05-T06): plays a quiet pulse once a second through the default
//! device and prints the transport position. Unplug and replug the output device while it
//! runs; see docs/engineering/manual-tests/a05-device-hotplug.md for the steps.
//!
//! Hearing safety: the pulses peak near −50 dBFS (track gain −47 dB, centre pan −3 dB) and
//! the stream opens a further 20 dB down. cpal cannot open RAW streams, and an OEM
//! enhancement APO can add about 20 dB after SONARITHM's limiter (F-032), so device tests
//! without RAW stay at or below −50 dBFS. Start with the system volume low.
//!
//! Run: `cargo run -p sonarithm-audio --example hotplug --release`

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a manual test program reports to the console"
)]

use std::io::BufRead as _;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use sonarithm_audio::{CpalBackend, Player, PlayerEvent, StreamConfig};
use sonarithm_core::{
    Cents, Db, EngineId, InstrumentRef, Mode, Note, SampleRate, Score, ScoreEvent, ScoreHeader,
    ScoreTrack, Seconds, SpecVersion, TrackId, TrackRole,
};
use sonarithm_render::PlanConfig;
use sonarithm_render::graph::Command;

const MINUTES: u32 = 3;

fn score() -> Score {
    let seconds = 60 * MINUTES;
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 0);
    header.duration = Seconds(f64::from(seconds));
    let mut score = Score::new(header);
    let mut track = ScoreTrack::new(
        TrackId::new("pulse"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("sine".to_owned()),
    );
    track.gain = Db(-47.0);
    track.events = (0..seconds)
        .map(|s| {
            // A rising pitch every 10 s makes the position audible.
            let cents = 100.0 * f64::from(s % 10);
            ScoreEvent::Note(Note::new(Seconds(f64::from(s)), Seconds(0.2), Cents(cents)))
        })
        .collect();
    score.tracks = vec![track];
    score
}

fn main() {
    let mut player = match Player::open(
        CpalBackend::new(),
        score(),
        PlanConfig::new(SampleRate::HZ_48000),
        StreamConfig::default(),
        256,
    ) {
        Ok(player) => player,
        Err(error) => {
            eprintln!("cannot open the default output device: {error}");
            return;
        }
    };
    let device = player
        .stream()
        .map(|s| s.device().to_owned())
        .unwrap_or_default();
    println!(
        "Playing on {device}. Unplug or switch the output device; press Enter to resume after a change."
    );
    let (enter_tx, enter) = mpsc::channel();
    std::thread::spawn(move || {
        for _ in std::io::stdin().lock().lines() {
            if enter_tx.send(()).is_err() {
                break;
            }
        }
    });
    player.send(Command::Play);
    let start = Instant::now();
    let mut report = Instant::now();
    while start.elapsed() < Duration::from_secs(u64::from(60 * MINUTES)) {
        match player.tick() {
            Ok(Some(PlayerEvent::DeviceChanged { device })) => {
                println!("Audio device changed to {device}; paused. Press Enter to resume.");
            }
            Ok(_) => {}
            Err(error) => println!("no device yet ({error}); retrying"),
        }
        if enter.try_recv().is_ok() {
            player.send(Command::Play);
        }
        if report.elapsed() >= Duration::from_secs(1) {
            report = Instant::now();
            let state = player.state();
            let rate = player
                .stream()
                .map_or(48_000, sonarithm_audio::Stream::sample_rate);
            #[expect(clippy::cast_precision_loss, reason = "positions below 2^53 frames")]
            let seconds = state.graph.position.0 as f64 / f64::from(rate);
            let (late, xruns) = player.stream().map_or((0, 0), |s| {
                (s.status().late_callbacks(), s.status().xruns())
            });
            println!(
                "position {seconds:7.2} s  playing {}  late callbacks {late}  xruns {xruns}",
                state.graph.playing
            );
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
