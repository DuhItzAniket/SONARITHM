// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(
    clippy::cast_possible_truncation,
    reason = "test frame counts are small"
)]

use sonarithm_core::{
    Breakpoint, Cents, Db, Earcon, EarconKind, EngineId, Frames, Glide, Hz, InstrumentRef, Mode,
    Note, Pan, SampleRate, Score, ScoreEvent, ScoreHeader, ScoreTrack, Seconds, SpecVersion,
    TrackId, TrackRole,
};

use super::*;
use crate::plan::PlanConfig;

const RATE: SampleRate = SampleRate::HZ_48000;

fn track(instrument: &str, role: TrackRole, events: Vec<ScoreEvent>) -> ScoreTrack {
    let mut track = ScoreTrack::new(
        TrackId::new(instrument),
        role,
        EngineId::from_static("numisona"),
        InstrumentRef(instrument.to_owned()),
    );
    track.events = events;
    track
}

fn score(duration: f64, tracks: Vec<ScoreTrack>) -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 1729);
    header.duration = Seconds(duration);
    let mut score = Score::new(header);
    score.tracks = tracks;
    score
}

fn note(t: f64, dur: f64, cents: f64) -> ScoreEvent {
    ScoreEvent::Note(Note::new(Seconds(t), Seconds(dur), Cents(cents)))
}

/// Every event kind and instrument, with onsets that fall inside blocks.
fn busy_score() -> Score {
    let mut tapped = Note::new(Seconds(0.3), Seconds(0.3), Cents(300.0));
    tapped.taps = 3;
    let glide = Glide::new(
        Seconds(0.1),
        Seconds(1.1),
        vec![
            Breakpoint::new(Seconds(0.1), Cents(-700.0)),
            Breakpoint::new(Seconds(0.6), Cents(500.0)),
            Breakpoint::new(Seconds(1.1), Cents(0.0)),
        ],
    );
    score(
        1.5,
        vec![
            track(
                "mallet",
                TrackRole::Data,
                vec![
                    note(0.0, 0.25, 0.0),
                    note(0.000_21, 0.2, 700.0),
                    ScoreEvent::Note(tapped),
                    note(0.9, 0.4, -300.0),
                ],
            ),
            track(
                "square",
                TrackRole::Data,
                vec![note(0.123_456, 0.5, 1_200.0)],
            ),
            track("saw", TrackRole::Data, vec![ScoreEvent::Glide(glide)]),
            track(
                "sine",
                TrackRole::Earcon,
                [
                    EarconKind::Boundary,
                    EarconKind::Landmark,
                    EarconKind::Threshold,
                    EarconKind::Alarm,
                ]
                .into_iter()
                .enumerate()
                .map(|(i, kind)| {
                    ScoreEvent::Earcon(Earcon::new(Seconds(0.2 * i as f64 + 0.05), kind))
                })
                .collect(),
            ),
        ],
    )
}

fn graph(score: &Score, block: usize) -> Graph {
    let plan = RenderPlan::build(score, &PlanConfig::new(RATE))
        .unwrap()
        .plan;
    Graph::new(plan, GraphConfig::offline(block))
}

/// The master limiter's look-ahead delay in frames.
fn latency() -> usize {
    graph(&drone(), 256).latency()
}

/// Plays the whole plan in blocks of `block` frames; returns interleaved stereo.
fn render(score: &Score, block: usize) -> Vec<f32> {
    let mut graph = graph(score, block);
    let frames = graph.plan().length.0 as usize + graph.latency();
    graph.apply(Command::Play);
    let mut out = vec![0.0; frames.div_ceil(block) * block * 2];
    for chunk in out.chunks_mut(block * 2) {
        graph.process(chunk);
    }
    out.truncate(frames * 2);
    out
}

fn left(out: &[f32]) -> Vec<f32> {
    out.iter().step_by(2).copied().collect()
}

/// The largest change between neighbouring samples of one channel.
fn worst_step(samples: &[f32]) -> f32 {
    samples
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max)
}

/// A05-T04 acceptance: an event starts at its exact sample, whatever the block size.
/// Every oscillator starts at phase 0, where its value is 0 (the band-limited square's
/// step is centred on its midpoint), so the first non-zero sample is the one after the
/// onset, plus the limiter's look-ahead delay.
#[test]
fn events_land_at_exact_sample_offsets() {
    for onset in [0_u64, 1, 255, 256, 1_000, 4_097] {
        let t = onset as f64 / 48_000.0;
        let score = score(
            0.2,
            vec![track("square", TrackRole::Data, vec![note(t, 0.05, 0.0)])],
        );
        for block in [1, 64, 256, 1_000] {
            let out = left(&render(&score, block));
            let first = out.iter().position(|s| *s != 0.0);
            assert_eq!(
                first,
                Some(onset as usize + 1 + latency()),
                "onset {onset}, block {block}"
            );
        }
    }
}

/// Sample-accurate scheduling makes the output bit-identical for any block size
/// (testing-strategy §5.2, block-size invariance).
#[test]
fn output_is_independent_of_block_size() {
    let score = busy_score();
    let reference = render(&score, 256);
    assert!(
        reference.iter().any(|s| s.abs() > 0.05),
        "the Score is audible"
    );
    for block in [1, 63, 64, 1_000, 4_096] {
        assert!(render(&score, block) == reference, "block {block}");
    }
}

#[test]
fn every_event_is_dispatched_once() {
    let score = busy_score();
    let mut graph = graph(&score, 256);
    let events = graph.plan().event_count() as u64;
    graph.apply(Command::Play);
    let mut out = vec![0.0; 512];
    while graph.stats().playing || graph.stats().rendered == 0 {
        graph.process(&mut out);
    }
    assert_eq!(graph.stats().dispatched, events);
    assert_eq!(graph.stats().stolen, 0);
}

/// A sustained 440 Hz sine at the track level.
fn drone() -> Score {
    score(
        4.0,
        vec![track("sine", TrackRole::Data, vec![note(0.0, 4.0, 0.0)])],
    )
}

/// The steepest step of a 440 Hz sine of amplitude `a` is 2π·440/48000·a per sample.
fn natural_step(peak: f32) -> f32 {
    core::f32::consts::TAU * 440.0 / 48_000.0 * peak
}

/// 06 §5: a seek crossfades instead of cutting, so the output never steps.
#[test]
fn a_seek_crossfades_without_a_step() {
    let mut graph = graph(&drone(), 256);
    graph.apply(Command::Play);
    let mut out = vec![0.0; 2 * 48_000];
    graph.process(&mut out[..48_000]);
    graph.apply(Command::Seek(Frames(96_000 + 123)));
    graph.process(&mut out[48_000..]);
    let samples = left(&out);
    let peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    assert!(
        worst_step(&samples) <= 2.0 * natural_step(peak),
        "step {}",
        worst_step(&samples)
    );
    assert_eq!(graph.stats().position, Frames(96_000 + 123 + 24_000));
}

/// 06 §5: loop wraps are seamless crossfades, and playback stays inside the loop.
#[test]
fn a_loop_wraps_with_a_crossfade() {
    let mut graph = graph(&drone(), 256);
    graph.apply(Command::SetLoop(Some(LoopRange {
        start: Frames(10_000),
        end: Frames(20_000),
    })));
    graph.apply(Command::Seek(Frames(10_000)));
    graph.apply(Command::Play);
    let mut out = vec![0.0; 2 * 48_000];
    graph.process(&mut out);
    let samples = left(&out);
    let peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    assert!(worst_step(&samples) <= 2.0 * natural_step(peak));
    let position = graph.stats().position.0;
    assert!((10_000..20_000).contains(&position), "{position}");
    assert!(graph.stats().playing);
}

/// Pause fades out within 20 ms (sound-design §5), freezes the position, and play resumes
/// the note that was sounding.
#[test]
fn pause_fades_out_and_play_resumes() {
    let mut graph = graph(&drone(), 128);
    graph.apply(Command::Play);
    let mut out = vec![0.0; 2 * 4_800];
    graph.process(&mut out);
    graph.apply(Command::Pause);
    graph.process(&mut out);
    let samples = left(&out);
    assert!(
        samples[961 + latency()..].iter().all(|s| *s == 0.0),
        "silent after 20 ms"
    );
    assert!(worst_step(&samples) <= 2.0 * natural_step(1.0));
    let paused = graph.stats().position;
    assert_eq!(paused, Frames(4_800));
    graph.process(&mut out);
    assert_eq!(graph.stats().position, paused);
    graph.apply(Command::Play);
    graph.process(&mut out);
    assert!(
        left(&out).iter().any(|s| s.abs() > 0.1),
        "the drone resumes"
    );
}

/// 06 §5: speed changes the time mapping, never the pitch.
#[test]
fn speed_scales_timing_but_not_pitch() {
    let score = score(
        2.0,
        vec![track("sine", TrackRole::Data, vec![note(1.0, 0.5, 0.0)])],
    );
    let mut graph = graph(&score, 256);
    graph.apply(Command::SetSpeed(2.0));
    graph.apply(Command::Play);
    let mut out = vec![0.0; 2 * 48_000];
    graph.process(&mut out);
    let samples = left(&out);
    let onset = samples.iter().position(|s| *s != 0.0).unwrap();
    let onset = onset - latency();
    assert!((24_000..24_002).contains(&onset), "onset {onset}");
    // 0.5 s at double speed lasts 0.25 s; count upward zero crossings in its middle.
    let middle = &samples[onset + 2_400..onset + 9_600];
    let crossings = middle
        .windows(2)
        .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
        .count();
    assert!(
        (65..=67).contains(&crossings),
        "{crossings} cycles in 150 ms at 440 Hz"
    );
}

/// A cue sounds in the next block, even while the transport is stopped.
#[test]
fn a_cue_plays_in_the_next_block() {
    let mut graph = graph(&drone(), 256);
    graph.apply(Command::Cue(Cue {
        frequency: Hz(880.0),
        gain: Db(-6.0),
        pan: Pan::CENTRE,
        instrument: Instrument::Square,
        duration: Seconds(0.12),
    }));
    let mut out = vec![0.0; 512];
    graph.process(&mut out);
    let first = left(&out).iter().position(|s| *s != 0.0);
    assert_eq!(
        first,
        Some(1 + latency()),
        "sounding from the frame after the onset"
    );
    assert!(!graph.stats().playing);
}

/// Bus and master levels move smoothly to their targets (coding-standards §11.3).
#[test]
fn gain_changes_are_smoothed() {
    let mut graph = graph(&drone(), 256);
    graph.apply(Command::Play);
    let mut out = vec![0.0; 2 * 4_800];
    graph.process(&mut out);
    graph.apply(Command::SetGain(GainTarget::Master, Db(-60.0)));
    graph.process(&mut out);
    let samples = left(&out);
    assert!(worst_step(&samples) <= 2.0 * natural_step(1.0));
    assert!(
        samples[480 + latency()..].iter().all(|s| s.abs() < 0.002),
        "at −60 dB after 10 ms"
    );
}

#[test]
fn playback_stops_at_the_end_of_the_plan() {
    let score = score(
        0.1,
        vec![track("sine", TrackRole::Data, vec![note(0.0, 0.1, 0.0)])],
    );
    let mut graph = graph(&score, 256);
    graph.apply(Command::Play);
    let mut out = vec![0.0; 2 * 9_600];
    graph.process(&mut out);
    let stats = graph.stats();
    assert!(!stats.playing);
    assert_eq!(stats.active_voices, 0);
    assert!(left(&out)[4_800 + latency()..].iter().all(|s| *s == 0.0));
}

/// SDR-16: the click instrument is a short broadband burst, the same on every render, and
/// it ignores pitch (sound-design §7.1).
#[test]
fn sdr_16_the_click_is_short_broadband_noise() {
    let clicks = |cents: f64| {
        score(
            0.2,
            vec![track(
                "click",
                TrackRole::Context,
                vec![note(0.05, 0.010, cents)],
            )],
        )
    };
    let samples = left(&render(&clicks(0.0), 256));
    assert_eq!(
        samples,
        left(&render(&clicks(1_200.0), 256)),
        "pitch is ignored"
    );
    assert_eq!(samples, left(&render(&clicks(0.0), 64)), "deterministic");
    let sounding: Vec<usize> = (0..samples.len())
        .filter(|&i| samples[i].abs() > 1e-4)
        .collect();
    let (first, last) = (sounding[0], sounding[sounding.len() - 1]);
    let length = (last - first) as f64 / 48_000.0;
    assert!((0.008..=0.012).contains(&length), "about 10 ms: {length} s");
    // Noise crosses zero far more often than any data tone in the SDR-08 range would.
    let crossings = samples[first..=last]
        .windows(2)
        .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
        .count();
    let per_second = crossings as f64 / length;
    assert!(per_second > 4_000.0, "{per_second} crossings per second");
}
