// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use sonarithm_core::{
    Cents, Db, EngineId, Frames, InstrumentRef, Mode, Note, SampleRate, Score, ScoreEvent,
    ScoreHeader, ScoreTrack, Seconds, SpecVersion, TrackId, TrackRole,
};

use super::*;
use crate::graph::{GainTarget, GraphConfig};
use crate::plan::{PlanConfig, RenderPlan};

/// A plan with one note per second for `seconds`.
fn graph(seconds: u32) -> Graph {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 3);
    header.duration = Seconds(f64::from(seconds));
    let mut score = Score::new(header);
    let mut track = ScoreTrack::new(
        TrackId::new("t"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("sine".to_owned()),
    );
    track.events = (0..seconds)
        .map(|s| ScoreEvent::Note(Note::new(Seconds(f64::from(s)), Seconds(0.8), Cents(0.0))))
        .collect();
    score.tracks = vec![track];
    let plan = RenderPlan::build(&score, &PlanConfig::new(SampleRate::HZ_48000))
        .unwrap()
        .plan;
    Graph::new(plan, GraphConfig::new(256))
}

#[test]
fn commands_reach_the_audio_thread_and_state_comes_back() {
    let (mut control, mut audio) = realtime(graph(4));
    control.send(Command::Seek(Frames(48_000)));
    control.send(Command::Play);
    let mut out = vec![0.0; 512];
    audio.process(&mut out, Some(Duration::from_millis(7)));
    let state = control.state();
    assert!(state.graph.playing);
    assert_eq!(state.graph.position, Frames(48_256));
    assert_eq!(state.host_time, Some(Duration::from_millis(7)));
    assert_eq!(state.blocks, 1);
    audio.report_xruns(1);
    audio.process(&mut out, None);
    assert_eq!(control.state().xruns, 1);
}

/// 06 §4: a full ring never blocks; queued commands coalesce (the latest seek wins) and
/// arrive in order once the audio thread drains the ring.
#[test]
fn a_full_ring_coalesces_and_never_blocks() {
    let (mut control, mut audio) = realtime(graph(4));
    for frame in 0..COMMAND_CAPACITY as u64 {
        control.send(Command::SetGain(
            GainTarget::Master,
            Db(-(frame as f64) / 100.0),
        ));
    }
    assert_eq!(control.overflows(), 0);
    for frame in 1..=500_u64 {
        control.send(Command::Seek(Frames(frame)));
        control.send(Command::SetSpeed(1.0 + frame as f64 / 1_000.0));
    }
    assert_eq!(control.overflows(), 1_000);
    assert_eq!(
        control.pending.len(),
        2,
        "one seek and one speed change remain"
    );
    let mut out = vec![0.0; 512];
    // 64 commands per block: 16 blocks drain the ring, then flushing sends the rest.
    for _ in 0..16 {
        audio.process(&mut out, None);
    }
    assert_eq!(control.flush(), 0);
    audio.process(&mut out, None);
    let state = control.state();
    assert_eq!(state.graph.position, Frames(500), "the latest seek won");
    assert!((state.graph.speed - 1.5).abs() < 1e-12);
}

/// 06 §4: a plan swap keeps the transport and frees the old graph on the control thread.
#[test]
fn a_plan_swap_keeps_the_transport_and_retires_the_old_graph() {
    let (mut control, mut audio) = realtime(graph(4));
    control.send(Command::Play);
    let mut out = vec![0.0; 2 * 4_800];
    audio.process(&mut out, None);
    assert_eq!(control.live_allocations(), 1);
    control.swap(graph(6)).map_err(|_| "ring full").unwrap();
    audio.process(&mut out, None);
    let state = control.state();
    assert_eq!(state.swaps, 1);
    assert!(state.graph.playing);
    assert_eq!(state.graph.position, Frames(9_600));
    // The old graph fades out for 20 ms, then the audio thread hands it to the collector.
    audio.process(&mut out, None);
    assert_eq!(control.live_allocations(), 2, "queued, not yet freed");
    control.collect();
    assert_eq!(control.live_allocations(), 1);
}

#[test]
fn a_third_waiting_swap_is_refused() {
    let (mut control, _audio) = realtime(graph(1));
    assert!(control.swap(graph(1)).is_ok());
    assert!(control.swap(graph(1)).is_ok());
    assert!(control.swap(graph(1)).is_err());
}

/// testing-strategy §8: the command ring, the plan swap and the collector under shuttle's
/// randomized schedules. The control thread sends seeks and swaps while the audio thread
/// renders; every schedule must end with the last seek applied, every swap taken, and every
/// retired graph freed by the collector.
#[test]
fn shuttle_the_rt_protocol_under_random_schedules() {
    shuttle::check_random(
        || {
            let (mut control, audio) = realtime(graph(2));
            let worker = shuttle::thread::spawn(move || {
                let mut audio = audio;
                let mut out = vec![0.0; 128];
                for _ in 0..40 {
                    audio.process(&mut out, None);
                    shuttle::thread::yield_now();
                }
                audio
            });
            let mut swapped = 0;
            for frame in 1..=20_u64 {
                control.send(Command::Seek(Frames(frame * 100)));
                if frame % 7 == 0 && control.swap(graph(2)).is_ok() {
                    swapped += 1;
                }
                control.collect();
                shuttle::thread::yield_now();
            }
            let mut audio = worker.join().unwrap();
            let mut out = vec![0.0; 2 * 2_048];
            while control.flush() > 0 {
                audio.process(&mut out, None);
            }
            // Drain any command or swap still in the rings, then let retiring graphs go.
            for _ in 0..4 {
                audio.process(&mut out, None);
            }
            let state = control.state();
            assert_eq!(state.swaps, swapped);
            assert_eq!(state.graph.position, Frames(2_000), "the last seek wins");
            control.collect();
            assert_eq!(
                control.live_allocations(),
                1,
                "only the current graph is alive"
            );
            drop(audio);
            control.collect();
            assert_eq!(control.live_allocations(), 0);
        },
        200,
    );
}
