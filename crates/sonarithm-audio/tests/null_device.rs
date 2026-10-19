// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The backend contract on the null device: playback, latency reporting, and recovery from
//! device loss with the transport position kept (A05-T06; the real-device check is the
//! manual hot-plug script in docs/engineering/manual-tests/).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::time::{Duration, Instant};

use sonarithm_audio::{AudioBackend, NullBackend, NullConfig, Player, PlayerEvent, StreamConfig};
use sonarithm_core::{
    Cents, Db, EngineId, InstrumentRef, Mode, Note, Score, ScoreEvent, ScoreHeader, ScoreTrack,
    Seconds, SpecVersion, TrackId, TrackRole,
};
use sonarithm_render::graph::{Command, GainTarget};
use sonarithm_render::{PlanConfig, realtime};

fn drone(seconds: f64) -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 1);
    header.duration = Seconds(seconds);
    let mut score = Score::new(header);
    let mut track = ScoreTrack::new(
        TrackId::new("drone"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("sine".to_owned()),
    );
    track.gain = Db(-30.0);
    track.events = vec![ScoreEvent::Note(Note::new(
        Seconds(0.0),
        Seconds(seconds),
        Cents(0.0),
    ))];
    score.tracks = vec![track];
    score
}

fn null(paced: bool) -> NullBackend {
    let mut config = NullConfig::new(48_000, 256);
    config.paced = paced;
    config.capture = 2 * 48_000;
    NullBackend::new(config)
}

/// Ticks until `done` holds or two seconds pass.
fn tick_until<B: AudioBackend>(
    player: &mut Player<B>,
    done: impl Fn(&sonarithm_render::EngineState) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        player.tick().unwrap();
        if done(&player.state()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("timed out; state {:?}", player.state());
}

#[test]
fn the_null_device_plays_the_graph() {
    let mut player = Player::open(
        null(false),
        drone(4.0),
        PlanConfig::new(sonarithm_core::SampleRate::HZ_48000),
        StreamConfig::default(),
        256,
    )
    .unwrap();
    player.send(Command::Play);
    tick_until(&mut player, |state| state.graph.position.0 >= 24_000);
    let stream = player.stream().unwrap();
    assert_eq!(stream.device(), "null");
    assert_eq!(stream.latency().buffer_frames, Some(256));
    assert!(stream.latency().reported.is_some());
    assert_eq!(
        stream.status().late_callbacks(),
        0,
        "the null clock is never late"
    );
    let captured = player.backend_mut().take_captured();
    assert!(
        captured.iter().any(|s| s.abs() > 1e-3),
        "the drone is audible"
    );
    // −30 dB track, −20 dB start-up ramp at first: nowhere near full scale.
    assert!(captured.iter().all(|s| s.abs() < 0.05));
}

/// 06 §11 and HS-05: after the device is unplugged, the player rebuilds on the default
/// device, keeps the position and the levels, and stays paused.
#[test]
fn unplugging_keeps_the_position_and_pauses() {
    let mut player = Player::open(
        null(true),
        drone(30.0),
        PlanConfig::new(sonarithm_core::SampleRate::HZ_48000),
        StreamConfig::default(),
        256,
    )
    .unwrap();
    player.send(Command::SetGain(GainTarget::Master, Db(-6.0)));
    player.send(Command::Play);
    tick_until(&mut player, |state| state.graph.position.0 >= 9_600);
    let control = player.backend_mut().control().unwrap();
    control.unplug();
    let before = player.state().graph.position;
    let mut event = None;
    let deadline = Instant::now() + Duration::from_secs(10);
    while event.is_none() && Instant::now() < deadline {
        event = player.tick().unwrap();
    }
    assert_eq!(
        event,
        Some(PlayerEvent::DeviceChanged {
            device: "null".to_owned()
        })
    );
    tick_until(&mut player, |state| state.blocks > 2);
    let after = player.state().graph;
    assert!(!after.playing, "paused after a device change (HS-05)");
    // The position is the last one published before the loss, within one block.
    assert!(
        after.position.0.abs_diff(before.0) <= 256,
        "{before:?} → {:?}",
        after.position
    );
    player.send(Command::Play);
    tick_until(&mut player, |state| {
        state.graph.position.0 > before.0 + 4_800
    });
}

#[test]
fn a_plan_at_the_wrong_rate_is_refused() {
    let plan = sonarithm_render::RenderPlan::build(
        &drone(1.0),
        &PlanConfig::new(sonarithm_core::SampleRate::new(44_100).unwrap()),
    )
    .unwrap()
    .plan;
    let (_control, processor) = realtime(sonarithm_render::Graph::new(
        plan,
        sonarithm_render::GraphConfig::new(256),
    ));
    let error = null(false)
        .open(&StreamConfig::default(), processor)
        .err()
        .unwrap();
    assert!(error.to_string().contains("48000"), "{error}");
}

/// A smoke test of the real default device, silent: an empty Score through cpal for half a
/// second. Run it by hand with `cargo test -p sonarithm-audio -- --ignored`; CI machines
/// have no sound card.
#[test]
#[ignore = "needs an audio output device"]
fn cpal_opens_the_default_device_silently() {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 0);
    header.duration = Seconds(1.0);
    let backend = sonarithm_audio::CpalBackend::new();
    let rate = backend.sample_rate(None).unwrap();
    let mut player = Player::open(
        backend,
        Score::new(header),
        PlanConfig::new(sonarithm_core::SampleRate::new(rate).unwrap()),
        StreamConfig::default(),
        256,
    )
    .unwrap();
    player.send(Command::Play);
    std::thread::sleep(Duration::from_millis(500));
    player.tick().unwrap();
    let state = player.state();
    let stream = player.stream().unwrap();
    println!(
        "device {}, {} Hz, {} channels, buffer {:?}, reported latency {:?}, callbacks {}, late {}, xruns {}",
        stream.device(),
        stream.sample_rate(),
        stream.channels(),
        stream.latency().buffer_frames,
        stream.latency().reported,
        stream.status().callbacks(),
        stream.status().late_callbacks(),
        stream.status().xruns()
    );
    assert!(state.blocks > 0, "the callback ran");
    assert!(state.graph.peak.iter().all(|p| *p == 0.0), "silence");
}

/// ADR-0016: the WASAPI backend opens the default device in RAW shared mode, silently.
/// Run by hand: `cargo test -p sonarithm-audio -- --ignored`.
#[cfg(windows)]
#[test]
#[ignore = "needs an audio output device"]
fn wasapi_opens_the_default_device_raw_and_silently() {
    wasapi_silent(sonarithm_audio::WasapiMode::SharedRaw);
}

/// ADR-0016: exclusive mode opens too (silently; it briefly takes the device from other
/// applications).
#[cfg(windows)]
#[test]
#[ignore = "needs an audio output device and takes it exclusively"]
fn wasapi_opens_the_default_device_exclusively_and_silently() {
    wasapi_silent(sonarithm_audio::WasapiMode::Exclusive);
}

#[cfg(windows)]
fn wasapi_silent(mode: sonarithm_audio::WasapiMode) {
    use sonarithm_audio::WasapiBackend;
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 0);
    header.duration = Seconds(1.0);
    let backend = WasapiBackend::new(mode);
    let rate = backend.sample_rate(None).unwrap();
    let mut player = Player::open(
        backend,
        Score::new(header),
        PlanConfig::new(sonarithm_core::SampleRate::new(rate).unwrap()),
        StreamConfig::default(),
        256,
    )
    .unwrap();
    player.send(Command::Play);
    std::thread::sleep(Duration::from_millis(1_000));
    player.tick().unwrap();
    let state = player.state();
    let info = player.backend_mut().last_stream().unwrap();
    let stream = player.stream().unwrap();
    println!(
        "device {}, {} Hz, {} channels, {info:?}, reported latency {:?}, callbacks {}, late {}, xruns {}",
        stream.device(),
        stream.sample_rate(),
        stream.channels(),
        stream.latency().reported,
        stream.status().callbacks(),
        stream.status().late_callbacks(),
        stream.status().xruns()
    );
    assert!(state.blocks > 0, "the callback ran");
    assert!(state.graph.peak.iter().all(|p| *p == 0.0), "silence");
}
