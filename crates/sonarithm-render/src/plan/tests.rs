// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fmt::Write as _;

use proptest::prelude::*;
use sonarithm_core::{
    Breakpoint, Cents, Db, Earcon, EarconKind, EngineId, Glide, InstrumentRef, Mode, Note, Pan,
    SampleRate, Score, ScoreEvent, ScoreHeader, ScoreTrack, Seconds, SpecVersion, Speech, TrackId,
    TrackRole,
};

use super::build::polyphony;
use super::*;

const RATE: SampleRate = SampleRate::HZ_48000;

fn track(id: &str, role: TrackRole, instrument: &str, events: Vec<ScoreEvent>) -> ScoreTrack {
    let mut track = ScoreTrack::new(
        TrackId::new(id),
        role,
        EngineId::from_static("numisona"),
        InstrumentRef(instrument.to_owned()),
    );
    track.events = events;
    track
}

fn note(t: f64, dur: f64, cents: f64) -> ScoreEvent {
    ScoreEvent::Note(Note::new(Seconds(t), Seconds(dur), Cents(cents)))
}

/// A small Score with every event kind: overlapping notes, taps, a glide, an earcon and a
/// speech track that the plan skips.
fn fixture() -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 1729);
    header.duration = Seconds(2.0);
    let mut score = Score::new(header);
    let mut tapped = Note::new(Seconds(1.0), Seconds(0.3), Cents(700.0));
    tapped.taps = 3;
    tapped.gain = Db(-6.0);
    tapped.pan = Pan(0.5);
    let mut melody = track(
        "melody",
        TrackRole::Data,
        "mallet",
        vec![
            note(0.0, 0.5, 0.0),
            note(0.25, 0.5, 1200.0),
            note(0.5, 0.004, -1200.0),
            ScoreEvent::Note(tapped),
        ],
    );
    melody.gain = Db(-6.0);
    melody.pan = Pan(-0.25);
    let mut start = Breakpoint::new(Seconds(0.0), Cents(-500.0));
    start.gain = Db(-3.0);
    let end = Breakpoint::new(Seconds(1.5), Cents(500.0));
    let contour = track(
        "contour",
        TrackRole::Data,
        "theremin",
        vec![ScoreEvent::Glide(Glide::new(
            Seconds(0.0),
            Seconds(1.5),
            vec![start, end],
        ))],
    );
    let cues = track(
        "cues",
        TrackRole::Earcon,
        "sine",
        vec![ScoreEvent::Earcon(Earcon::new(
            Seconds(1.95),
            EarconKind::Landmark,
        ))],
    );
    let speech = track(
        "legend",
        TrackRole::Speech,
        "sine",
        vec![ScoreEvent::Speech(Speech::new(Seconds(0.0), "Temperature"))],
    );
    score.tracks = vec![melody, contour, cues, speech];
    score
}

/// The review form of a plan: one line per track and per event, rounded for legible diffs.
fn review(plan: &RenderPlan) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "plan {} Hz, {} frames, {} events, {} voices",
        plan.sample_rate.get(),
        plan.length.0,
        plan.event_count(),
        plan.voice_count()
    );
    for track in &plan.tracks {
        let _ = writeln!(
            out,
            "track {} on {:?}: gain {:.4}, {}, polyphony {}, voices {}, longest {}",
            track.id,
            track.bus,
            track.gain,
            track.instrument.name(),
            track.polyphony,
            track.voices,
            track.longest
        );
        for event in &track.events {
            let what = match event.kind {
                EventKind::Tone(tone) => format!(
                    "tone {:.2} Hz gain {:.4} pan {:.4}/{:.4} bright {:.2} {} ramp {}",
                    tone.frequency.0,
                    tone.gain,
                    tone.pan.left,
                    tone.pan.right,
                    tone.brightness,
                    tone.instrument.name(),
                    tone.ramp
                ),
                EventKind::Glide(glide) => {
                    let first = glide.first as usize;
                    let points: Vec<String> = track.breakpoints
                        [first..first + glide.count as usize]
                        .iter()
                        .map(|p| {
                            format!(
                                "{}:{:.2}c/{:.2}dB/{:.2}",
                                p.frame, p.cents, p.gain_db, p.pan
                            )
                        })
                        .collect();
                    format!(
                        "glide {} ramp {} [{}]",
                        glide.instrument.name(),
                        glide.ramp,
                        points.join(", ")
                    )
                }
                EventKind::Earcon { kind, gain } => format!("earcon {kind:?} gain {gain:.4}"),
            };
            let _ = writeln!(
                out,
                "  #{} {}+{} {what}",
                event.source, event.start, event.length
            );
        }
    }
    out
}

#[test]
fn a_plan_resolves_frames_frequencies_gains_and_voices() {
    let planned = RenderPlan::build(&fixture(), &PlanConfig::new(RATE)).unwrap();
    insta::assert_snapshot!(review(&planned.plan));
    let codes: Vec<&str> = planned
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(
        codes,
        ["SNR-RENDER-001"],
        "the unknown `theremin` is reported"
    );
}

#[test]
fn speech_tracks_are_not_planned() {
    let planned = RenderPlan::build(&fixture(), &PlanConfig::new(RATE)).unwrap();
    let ids: Vec<&str> = planned.plan.tracks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["melody", "contour", "cues"]);
}

#[test]
fn an_invalid_score_is_rejected() {
    let mut score = fixture();
    score.tracks[0].events.reverse();
    assert!(matches!(
        RenderPlan::build(&score, &PlanConfig::new(RATE)),
        Err(PlanError::InvalidScore(_))
    ));
}

/// 05 §4.1: times convert to frames by rounding half to even.
#[test]
fn times_round_half_to_even() {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 0);
    header.duration = Seconds(1.0);
    let mut score = Score::new(header);
    // 2.5 and 3.5 frames at 1 kHz.
    score.tracks = vec![track(
        "t",
        TrackRole::Data,
        "sine",
        vec![note(0.0025, 0.2, 0.0), note(0.0035, 0.2, 0.0)],
    )];
    let rate = SampleRate::new(1_000).unwrap();
    let plan = RenderPlan::build(&score, &PlanConfig::new(rate))
        .unwrap()
        .plan;
    let starts: Vec<u64> = plan.tracks[0].events.iter().map(|e| e.start).collect();
    assert_eq!(starts, [2, 4]);
}

fn brute_force_polyphony(events: &[PlannedEvent]) -> usize {
    let mut frames: Vec<u64> = events.iter().map(|e| e.start).collect();
    frames.sort_unstable();
    frames
        .iter()
        .map(|&f| {
            events
                .iter()
                .filter(|e| e.start <= f && f < e.end())
                .count()
        })
        .max()
        .unwrap_or(0)
}

/// A valid Score with up to three tracks of notes (some tapped), glides and earcons.
/// (`sonarithm_core::testing::valid_score` would do, but enabling core's `test-support`
/// feature from a dev-dependency trips `cargo xtask check-deps`: `cargo metadata` unifies
/// it into the normal graph.)
fn valid_score() -> impl Strategy<Value = Score> {
    let event = (
        0.0_f64..9.0,
        0.0_f64..1.0,
        -1200.0_f64..1200.0,
        0_u8..4,
        1_u8..=8,
    );
    let events = proptest::collection::vec(event, 0..30);
    proptest::collection::vec(events, 0..3).prop_map(|tracks| {
        let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 7);
        header.duration = Seconds(10.0);
        let mut score = Score::new(header);
        for (index, mut specs) in tracks.into_iter().enumerate() {
            specs.sort_by(|a, b| a.0.total_cmp(&b.0));
            let events = specs
                .into_iter()
                .map(|(t, dur, cents, kind, taps)| match kind {
                    0 => ScoreEvent::Glide(Glide::new(
                        Seconds(t),
                        Seconds(t + dur),
                        vec![Breakpoint::new(Seconds(t), Cents(cents))],
                    )),
                    1 => ScoreEvent::Earcon(Earcon::new(Seconds(t), EarconKind::Threshold)),
                    _ => {
                        let mut note = Note::new(Seconds(t), Seconds(dur), Cents(cents));
                        note.taps = taps;
                        ScoreEvent::Note(note)
                    }
                })
                .collect();
            score
                .tracks
                .push(track(&format!("t{index}"), TrackRole::Data, "sine", events));
        }
        score
    })
}

proptest! {
    /// A05-T03 acceptance: the polyphony analysis equals a brute-force count of overlapping
    /// events, and every plan invariant holds on generated Scores.
    #[test]
    fn plans_of_valid_scores_keep_their_invariants(score in valid_score()) {
        let planned = RenderPlan::build(&score, &PlanConfig::new(RATE)).unwrap();
        let plan = &planned.plan;
        prop_assert_eq!(&RenderPlan::build(&score, &PlanConfig::new(RATE)).unwrap().plan, plan);
        for track in &plan.tracks {
            prop_assert_eq!(track.polyphony, brute_force_polyphony(&track.events));
            prop_assert_eq!(track.voices, 2 * track.polyphony);
            prop_assert!(track.events.windows(2).all(|w| w[0].start <= w[1].start));
            for event in &track.events {
                prop_assert!(event.end() <= plan.length.0);
                prop_assert!(event.length <= track.longest);
                if let EventKind::Tone(tone) = event.kind {
                    // SDR-22: ramps of at least 5 ms (240 frames), and room for both.
                    prop_assert!(tone.ramp >= 240);
                    prop_assert!(event.length >= 2 * u64::from(tone.ramp));
                }
            }
        }
    }

    #[test]
    fn polyphony_counts_overlaps(spans in proptest::collection::vec((0_u64..1_000, 1_u64..300), 0..60)) {
        let mut events: Vec<PlannedEvent> = spans
            .iter()
            .map(|&(start, length)| PlannedEvent {
                start,
                length,
                source: 0,
                kind: EventKind::Earcon { kind: EarconKind::Boundary, gain: 1.0 },
            })
            .collect();
        events.sort_by_key(|e| e.start);
        prop_assert_eq!(polyphony(&events), brute_force_polyphony(&events));
    }
}

fn tone_gains(plan: &RenderPlan) -> Vec<f32> {
    plan.tracks[0]
        .events
        .iter()
        .filter_map(|event| match event.kind {
            EventKind::Tone(tone) => Some(tone.gain),
            _ => None,
        })
        .collect()
}

fn two_notes() -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 0);
    header.duration = Seconds(1.0);
    let mut score = Score::new(header);
    // 220 Hz and 880 Hz: the 60-phon contour needs about 8 dB more at 220 Hz.
    score.tracks = vec![track(
        "t",
        TrackRole::Data,
        "sine",
        vec![note(0.0, 0.4, -1_200.0), note(0.5, 0.4, 1_200.0)],
    )];
    score
}

/// SDR-24 (DEFAULT): equal-loudness compensation is on, attenuates the more sensitive
/// pitch and never boosts.
#[test]
fn sdr_24_compensation_is_on_by_default() {
    let plan = RenderPlan::build(&two_notes(), &PlanConfig::new(RATE))
        .unwrap()
        .plan;
    let gains = tone_gains(&plan);
    assert_eq!(gains[0], 1.0, "the least sensitive pitch plays at 0 dB");
    let db = 20.0 * sonarithm_core::math::log10(f64::from(gains[1]));
    assert!((-10.0..-6.0).contains(&db), "880 Hz at {db:.2} dB");
}

#[test]
fn sdr_24_compensation_can_be_turned_off_and_is_not_applied_twice() {
    let off = PlanConfig::new(RATE).without_compensation();
    let plan = RenderPlan::build(&two_notes(), &off).unwrap().plan;
    assert_eq!(tone_gains(&plan), [1.0, 1.0]);
    let mut shaped = two_notes();
    shaped.header.shapers = vec![sonarithm_core::ShaperStage::Loudness];
    let plan = RenderPlan::build(&shaped, &PlanConfig::new(RATE))
        .unwrap()
        .plan;
    assert_eq!(
        tone_gains(&plan),
        [1.0, 1.0],
        "the compiler's loudness shaper already ran"
    );
}
