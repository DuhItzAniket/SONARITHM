// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A05-T09 acceptance (06 §10, NFR-10): the true peak stays at or below −1 dBTP on stress
//! Scores, measured with an independent BS.1770 meter (ebur128), and a stream never opens
//! at full scale.

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use sonarithm_core::{
    Breakpoint, Cents, Db, Earcon, EarconKind, EngineId, Glide, InstrumentRef, Mode, Note, Pan,
    SampleRate, Score, ScoreEvent, ScoreHeader, ScoreTrack, Seconds, SpecVersion, TrackId,
    TrackRole,
};
use sonarithm_render::{Command, Graph, GraphConfig, OfflineRender, PlanConfig, RenderPlan};

const RATE: SampleRate = SampleRate::HZ_48000;

fn track(
    id: &str,
    instrument: &str,
    role: TrackRole,
    gain: f64,
    events: Vec<ScoreEvent>,
) -> ScoreTrack {
    let mut track = ScoreTrack::new(
        TrackId::new(id),
        role,
        EngineId::from_static("numisona"),
        InstrumentRef(instrument.to_owned()),
    );
    track.gain = Db(gain);
    track.events = events;
    track
}

fn score(duration: f64, tracks: Vec<ScoreTrack>) -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 99);
    header.duration = Seconds(duration);
    let mut score = Score::new(header);
    score.tracks = tracks;
    score
}

fn plan(score: &Score) -> RenderPlan {
    RenderPlan::build(score, &PlanConfig::new(RATE))
        .unwrap()
        .plan
}

/// The true peak of interleaved stereo in dBTP, per ITU-R BS.1770 (ebur128).
fn true_peak_dbtp(samples: &[f32]) -> f64 {
    let mut meter = ebur128::EbuR128::new(2, RATE.get(), ebur128::Mode::TRUE_PEAK).unwrap();
    meter.add_frames_f32(samples).unwrap();
    let peak = meter.true_peak(0).unwrap().max(meter.true_peak(1).unwrap());
    20.0 * sonarithm_core::math::log10(peak)
}

/// Sixteen loud, high, band-limited square waves at once, with track gain +12 dB.
fn chord() -> Score {
    let notes = (0..16)
        .map(|i| {
            let mut note = Note::new(
                Seconds(0.0),
                Seconds(2.0),
                Cents(1_200.0 + 150.0 * f64::from(i)),
            );
            note.timbre.brightness = 1.0;
            note.pan = Pan(if i % 2 == 0 { -1.0 } else { 1.0 });
            ScoreEvent::Note(note)
        })
        .collect();
    score(
        2.0,
        vec![track("chord", "square", TrackRole::Data, 12.0, notes)],
    )
}

/// Saw glides sweeping up and down the spectrum, plus fast loud taps and earcons.
fn sweeps_and_taps() -> Score {
    let glides = (0..4)
        .map(|i| {
            let t0 = 0.5 * f64::from(i);
            ScoreEvent::Glide(Glide::new(
                Seconds(t0),
                Seconds(t0 + 0.5),
                vec![
                    Breakpoint::new(Seconds(t0), Cents(-3_600.0)),
                    Breakpoint::new(Seconds(t0 + 0.25), Cents(3_600.0)),
                    Breakpoint::new(Seconds(t0 + 0.5), Cents(-3_600.0)),
                ],
            ))
        })
        .collect();
    let taps = (0..20)
        .map(|i| {
            let mut note = Note::new(Seconds(0.1 * f64::from(i)), Seconds(0.08), Cents(2_400.0));
            note.taps = 8;
            note.gain = Db(6.0);
            ScoreEvent::Note(note)
        })
        .collect();
    let earcons = [
        EarconKind::Alarm,
        EarconKind::Boundary,
        EarconKind::Landmark,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, kind)| {
        let mut earcon = Earcon::new(Seconds(0.6 * i as f64), kind);
        earcon.gain = Db(12.0);
        ScoreEvent::Earcon(earcon)
    })
    .collect();
    score(
        2.0,
        vec![
            track("sweeps", "saw", TrackRole::Data, 12.0, glides),
            track("taps", "mallet", TrackRole::Data, 12.0, taps),
            track("earcons", "sine", TrackRole::Earcon, 18.0, earcons),
        ],
    )
}

/// 06 §10: true peak ≤ −1 dBTP on stress Scores.
#[test]
fn the_true_peak_never_exceeds_minus_1_dbtp() {
    for (name, score) in [("chord", chord()), ("sweeps and taps", sweeps_and_taps())] {
        let render = OfflineRender::new(plan(&score), 256);
        let samples = render.render_all();
        let dbtp = true_peak_dbtp(&samples);
        println!("{name}: true peak {dbtp:.3} dBTP");
        assert!(dbtp <= -1.0, "{name}: {dbtp:.2} dBTP");
        // The Scores are loud enough to need limiting: the limiter is doing the work.
        let sample_peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
        assert!(sample_peak > 0.8, "{name}: peak {sample_peak}");
    }
}

fn drone(gain: f64) -> Score {
    score(
        2.0,
        vec![track(
            "drone",
            "sine",
            TrackRole::Data,
            gain,
            vec![ScoreEvent::Note(Note::new(
                Seconds(0.0),
                Seconds(2.0),
                Cents(0.0),
            ))],
        )],
    )
}

/// Plays `score` through a playback graph (start-up ramp on) for one second.
fn play_first_second(score: &Score) -> (Vec<f32>, usize) {
    let mut graph = Graph::new(plan(score), GraphConfig::new(256));
    graph.apply(Command::Play);
    let mut out = vec![0.0_f32; 2 * 48_000];
    for block in out.chunks_mut(512) {
        graph.process(block);
    }
    (out, graph.latency())
}

/// NFR-10 (06 §10): a stream opens 20 dB down and rises over 500 ms.
#[test]
fn nfr_10_a_stream_opens_20_db_down_and_rises_over_500_ms() {
    let (out, latency) = play_first_second(&drone(-6.0));
    // Peak per 10 ms window, after the limiter's look-ahead delay.
    let window = |i: usize| {
        let start = 2 * (latency + 480 * i);
        out[start..start + 960]
            .iter()
            .fold(0.0_f32, |m, s| m.max(s.abs()))
    };
    let steady = window(80);
    // 40–50 ms: the note's own 40 ms onset ramp is over, the start-up ramp is not.
    let early = 20.0 * sonarithm_core::math::log10(f64::from(window(4) / steady));
    assert!((-19.0..=-17.0).contains(&early), "at 45 ms: {early:.2} dB");
    for i in 5..50 {
        assert!(window(i) >= window(i - 1), "the ramp falls at {i}0 ms");
    }
    assert!(window(51) >= 0.999 * steady, "the ramp ends by 510 ms");
}

/// NFR-10: even a Score far above full scale starts under the −1 dBTP ceiling.
#[test]
fn nfr_10_no_full_scale_onset() {
    let (out, _) = play_first_second(&chord());
    let first_block = &out[..2 * 512];
    let peak = first_block.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    assert!(peak < 0.5, "first 512 frames peak at {peak}");
    assert!(true_peak_dbtp(&out) <= -1.0);
}

/// An export starts as the Score does: no start-up ramp offline.
#[test]
fn offline_renders_have_no_startup_ramp() {
    let score = score(
        0.5,
        vec![track(
            "tone",
            "sine",
            TrackRole::Data,
            -12.0,
            vec![ScoreEvent::Note(Note::new(
                Seconds(0.0),
                Seconds(0.5),
                Cents(0.0),
            ))],
        )],
    );
    let samples = OfflineRender::new(plan(&score), 256).render_all();
    let early = samples[2 * 2_400..2 * 2_880]
        .iter()
        .fold(0.0_f32, |m, s| m.max(s.abs()));
    let late = samples[2 * 19_200..2 * 19_680]
        .iter()
        .fold(0.0_f32, |m, s| m.max(s.abs()));
    assert!((early / late - 1.0).abs() < 0.01, "{early} vs {late}");
}
