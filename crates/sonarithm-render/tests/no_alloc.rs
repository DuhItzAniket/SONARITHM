// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A05-T10, coding-standards §11.5: the audio thread never allocates or frees memory.
//!
//! A scripted session drives [`Processor::process`] through every command and a plan swap
//! (testing-strategy §8: play, seek, loop, plan swap, cues, gain changes, stop). Each
//! block runs inside `allocation_counter::measure`, which counts every allocation and free
//! made by this thread; any count fails the test. Control tests show that the check fails
//! on an allocation or a free inside the measured block, and that allocations on other
//! threads are not counted. The checker replaces `assert_no_alloc`, whose BSD-1-Clause
//! licence `deny.toml` does not allow (F-038, approved by the coordinator).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use sonarithm_core::{Db, Frames, Hz, Pan, SampleRate, Seconds};
use sonarithm_render::fixtures::{BENCHMARK_SEED, benchmark_score};
use sonarithm_render::graph::{Command, Cue, GainTarget, LoopRange};
use sonarithm_render::{
    Bus, EarconSound, Graph, GraphConfig, Instrument, PlanConfig, Processor, RenderPlan, realtime,
};

fn graph(seconds: f64) -> Graph {
    let plan = RenderPlan::build(
        &benchmark_score(seconds, BENCHMARK_SEED),
        &PlanConfig::new(SampleRate::HZ_48000),
    )
    .unwrap()
    .plan;
    Graph::new(plan, GraphConfig::new(256))
}

/// Runs `f`, failing if this thread allocated or freed memory meanwhile. A free of memory
/// allocated elsewhere shows as a negative `count_current`.
fn assert_no_allocation(step: &str, block: usize, f: impl FnOnce()) {
    let info = allocation_counter::measure(f);
    assert!(
        info.count_total == 0 && info.count_current == 0,
        "{step}, block {block}: {info:?}"
    );
}

/// Processes `blocks` blocks of 256 frames, failing on any allocation or free.
fn blocks(audio: &mut Processor, out: &mut [f32], blocks: usize, step: &str) {
    for block in 0..blocks {
        assert_no_allocation(step, block, || {
            audio.process(out, Some(Duration::from_millis(block as u64)));
        });
    }
}

#[test]
fn a_scripted_session_never_allocates_on_the_audio_thread() {
    let (mut control, mut audio) = realtime(graph(6.0));
    let mut out = vec![0.0; 512];

    control.send(Command::Play);
    blocks(&mut audio, &mut out, 200, "play");
    control.send(Command::Seek(Frames(96_000)));
    blocks(&mut audio, &mut out, 50, "seek");
    control.send(Command::SetLoop(Some(LoopRange {
        start: Frames(48_000),
        end: Frames(60_000),
    })));
    blocks(&mut audio, &mut out, 200, "loop wraps");
    control.send(Command::SetGain(GainTarget::Bus(Bus::Context), Db(-20.0)));
    control.send(Command::SetGain(GainTarget::Master, Db(-6.0)));
    control.send(Command::SetGain(
        GainTarget::Duck,
        sonarithm_render::DUCK_LEVEL,
    ));
    control.send(Command::SetSpeed(1.5));
    blocks(&mut audio, &mut out, 50, "gain and speed");
    // More cues than may sound (the oldest fades) and, in one block, more than the slots
    // hold (a fading voice is stolen): F-211, B03-T09.
    let cue = Cue {
        frequency: Hz(660.0),
        gain: Db(-12.0),
        pan: Pan::CENTRE,
        instrument: Instrument::Square,
        duration: Seconds(0.12),
    };
    for _ in 0..6 {
        control.send(Command::Cue(cue));
        blocks(&mut audio, &mut out, 5, "cue");
    }
    for _ in 0..10 {
        control.send(Command::Cue(cue));
    }
    blocks(&mut audio, &mut out, 5, "cue pool full");
    // Every earcon live, more than the earcon pool holds (B03-T06).
    for sound in EarconSound::ALL {
        control.send(Command::Earcon(sound));
        blocks(&mut audio, &mut out, 3, "earcon");
    }
    // A plan swap: the new graph is built here, off the audio thread; the old one fades out
    // and is handed to the collector without being freed on the audio thread.
    control
        .swap(graph(6.0))
        .map_err(|_| "swap ring full")
        .unwrap();
    blocks(&mut audio, &mut out, 100, "plan swap");
    control.collect();
    control.send(Command::SetLoop(None));
    control.send(Command::Pause);
    blocks(&mut audio, &mut out, 20, "pause");
    control.send(Command::Play);
    blocks(&mut audio, &mut out, 20, "resume");
    control.send(Command::StopAll);
    blocks(&mut audio, &mut out, 20, "stop all");
    control.send(Command::Stop);
    blocks(&mut audio, &mut out, 20, "stop");
    // A flood of commands larger than one block's cap.
    for frame in 0..300 {
        control.send(Command::Seek(Frames(frame * 100)));
    }
    control.send(Command::Play);
    blocks(&mut audio, &mut out, 20, "command flood");
    // A block larger than the graph's maximum is processed in pieces.
    let mut large = vec![0.0; 2 * 4_096];
    blocks(&mut audio, &mut large, 5, "large block");
    assert_eq!(control.state().swaps, 1);
    control.collect();
    assert_eq!(
        control.live_allocations(),
        1,
        "the retired graph was collected"
    );
}

/// C03-T05, AGENTS.md rule 2: playing, seeking into and speeding up an audification reads
/// its preallocated buffer and never allocates on the audio thread.
#[test]
fn rt_audification_never_allocates_on_the_audio_thread() {
    use sonarithm_core::score::{Audify, Score, ScoreEvent, ScoreHeader, ScoreTrack};
    use sonarithm_core::{EngineId, InstrumentRef, Mode, SpecVersion, TrackId, TrackRole};
    #[expect(clippy::cast_possible_truncation, reason = "samples are f32")]
    let samples: Vec<f32> = (0..44_000)
        .map(|i| (0.8 * sonarithm_core::math::sin(0.0628 * f64::from(i))) as f32)
        .collect();
    let event = Audify::new(Seconds::ZERO, 22_000.0, samples);
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 7);
    header.duration = Seconds(2.5);
    let mut score = Score::new(header);
    let mut track = ScoreTrack::new(
        TrackId::new("audify"),
        TrackRole::Data,
        EngineId::from_static("curvasona"),
        InstrumentRef("sine".to_owned()),
    );
    track.events.push(ScoreEvent::Audify(event));
    score.tracks.push(track);
    let plan = RenderPlan::build(&score, &PlanConfig::new(SampleRate::HZ_48000))
        .unwrap()
        .plan;
    let (mut control, mut audio) = realtime(Graph::new(plan, GraphConfig::new(256)));
    let mut out = vec![0.0; 512];
    control.send(Command::Play);
    blocks(&mut audio, &mut out, 100, "audify play");
    control.send(Command::Seek(Frames(60_000)));
    blocks(&mut audio, &mut out, 50, "audify seek");
    control.send(Command::SetSpeed(1.5));
    blocks(&mut audio, &mut out, 50, "audify speed");
}

/// Control: the checker is live, so an allocation inside a measured block fails the check.
#[test]
fn b01_t04_the_meter_window_never_allocates_and_reaches_the_control_side() {
    // The meter window (B01-T04) runs inside `Graph::process` on the audio thread; its
    // levels reach the control thread through the engine state's triple buffer, with no
    // lock.
    let (mut control, mut audio) = realtime(graph(6.0));
    let mut out = vec![0.0; 512];
    control.send(Command::Play);
    // 400 blocks of 256 frames: about 2 s, many complete 50 ms windows.
    blocks(&mut audio, &mut out, 400, "metering");
    let meter = control.state().graph.meter;
    assert!(meter.peak[0] > 0.0 && meter.peak[1] > 0.0, "{meter:?}");
    assert!(meter.peak[0] <= 1.0 && meter.peak[1] <= 1.0, "{meter:?}");
}

#[test]
#[should_panic(expected = "allocating block")]
fn an_allocation_in_the_measured_block_fails_the_check() {
    let (_control, mut audio) = realtime(graph(1.0));
    let mut out = vec![0.0; 512];
    assert_no_allocation("allocating block", 0, || {
        audio.process(&mut out, None);
        std::hint::black_box(vec![0_u8; 16]);
    });
}

/// Control: freeing memory allocated elsewhere inside a measured block fails the check too.
#[test]
#[should_panic(expected = "freeing block")]
fn a_free_in_the_measured_block_fails_the_check() {
    let (_control, mut audio) = realtime(graph(1.0));
    let mut out = vec![0.0; 512];
    let elsewhere = std::hint::black_box(vec![0_u8; 16]);
    assert_no_allocation("freeing block", 0, move || {
        audio.process(&mut out, None);
        drop(elsewhere);
    });
}

/// Control: the count is per thread. Another thread allocating throughout the measurement
/// is not counted, so the check sees only the audio thread's own allocations.
#[test]
fn allocations_on_another_thread_are_not_counted() {
    let stop = Arc::new(AtomicBool::new(false));
    let made = Arc::new(AtomicU64::new(0));
    let other = {
        let (stop, made) = (Arc::clone(&stop), Arc::clone(&made));
        std::thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                std::hint::black_box(vec![0_u8; 64]);
                made.fetch_add(1, Ordering::Release);
            }
        })
    };
    let (_control, mut audio) = realtime(graph(1.0));
    let mut out = vec![0.0; 512];
    let before = made.load(Ordering::Acquire);
    let info = allocation_counter::measure(|| {
        for block in 0..50 {
            audio.process(&mut out, Some(Duration::from_millis(block)));
        }
        // Measure until the other thread has allocated and freed many times.
        while made.load(Ordering::Acquire) < before + 1_000 {
            std::hint::spin_loop();
        }
    });
    stop.store(true, Ordering::Release);
    other.join().unwrap();
    assert!(made.load(Ordering::Acquire) >= before + 1_000);
    assert_eq!((info.count_total, info.count_current), (0, 0), "{info:?}");
}
