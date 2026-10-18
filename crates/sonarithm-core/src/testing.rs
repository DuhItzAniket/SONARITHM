// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! proptest strategies for core types, shared with other crates through the non-default
//! `test-support` feature (testing-strategy §3).

use proptest::collection::vec;
use proptest::prelude::*;

use crate::ids::{EngineId, TrackId};
use crate::score::{
    Breakpoint, Earcon, EarconKind, Glide, InstrumentRef, Mode, Note, PitchRange, Score,
    ScoreEvent, ScoreHeader, ScoreTrack, ShaperStage, SpecVersion, TrackRole,
};
use crate::units::{Cents, Db, Pan, Seconds};

/// Any mode.
pub fn mode() -> impl Strategy<Value = Mode> {
    prop_oneof![
        Just(Mode::Analysis),
        Just(Mode::Presentation),
        Just(Mode::Monitoring)
    ]
}

/// A Score that satisfies every invariant of 05 §4.2.
pub fn valid_score() -> impl Strategy<Value = Score> {
    (mode(), any::<u64>(), 1.0_f64..120.0)
        .prop_flat_map(|(mode, seed, duration)| {
            let tracks = vec(track(duration), 0..4);
            (
                Just(mode),
                Just(seed),
                Just(duration),
                tracks,
                shapers(mode),
            )
        })
        .prop_map(|(mode, seed, duration, tracks, shapers)| {
            let mut header = ScoreHeader::new(SpecVersion::V0_1, mode, seed);
            header.duration = Seconds(duration);
            header.shapers = shapers;
            let mut score = Score::new(header);
            score.tracks = tracks;
            score
        })
}

/// Shapers in their fixed order; Quantize only outside Analysis mode (SDR-10).
fn shapers(mode: Mode) -> impl Strategy<Value = Vec<ShaperStage>> {
    let quantize = mode != Mode::Analysis;
    (any::<[bool; 4]>()).prop_map(move |on| {
        [
            ShaperStage::Quantize,
            ShaperStage::Loudness,
            ShaperStage::Spatial,
            ShaperStage::Lint,
        ]
        .into_iter()
        .zip(on)
        .filter(|(stage, on)| *on && (quantize || *stage != ShaperStage::Quantize))
        .map(|(stage, _)| stage)
        .collect()
    })
}

/// A data track whose events lie within `duration` and its pitch range.
fn track(duration: f64) -> impl Strategy<Value = ScoreTrack> {
    (-3000.0_f64..0.0, 100.0_f64..4000.0, -1.0_f64..=1.0)
        .prop_flat_map(move |(low, width, pan)| {
            let range = PitchRange {
                low: Cents(low),
                high: Cents(low + width),
            };
            (Just(range), Just(pan), vec(event(duration, range), 0..40))
        })
        .prop_map(|(range, pan, mut events)| {
            events.sort_by(|a, b| a.start().total_cmp(&b.start()));
            let mut track = ScoreTrack::new(
                TrackId::new("t"),
                TrackRole::Data,
                EngineId::from_static("numisona"),
                InstrumentRef("mallet".to_owned()),
            );
            track.pan = Pan(pan);
            track.gain = Db(-6.0);
            track.pitch_range = Some(range);
            track.events = events;
            track
        })
}

/// A pitch at fraction `w` of the range, clamped so rounding cannot leave it.
fn pitch_in(range: PitchRange, w: f64) -> Cents {
    let span = range.high.0 - range.low.0;
    Cents((range.low.0 + w * span).clamp(range.low.0, range.high.0))
}

fn event(duration: f64, range: PitchRange) -> impl Strategy<Value = ScoreEvent> {
    prop_oneof![
        (0.0_f64..0.8, 0.001_f64..0.1, 0.0_f64..=1.0, 1_u8..=8).prop_map(move |(u, v, w, taps)| {
            let mut note = Note::new(
                Seconds(u * duration),
                Seconds(v * duration),
                pitch_in(range, w),
            );
            note.taps = taps;
            ScoreEvent::Note(note)
        }),
        (0.0_f64..0.5, 0.0_f64..0.4, vec(0.0_f64..=1.0, 1..6)).prop_map(move |(u, v, ws)| {
            let t0 = u * duration;
            let t1 = t0 + v * duration;
            let steps = f64::from(u32::try_from(ws.len()).unwrap_or(u32::MAX));
            let curve = ws
                .iter()
                .zip((0_u32..).map(f64::from))
                .map(|(w, i)| {
                    let t = (t0 + (i / steps) * (t1 - t0)).min(t1);
                    Breakpoint::new(Seconds(t), pitch_in(range, *w))
                })
                .collect();
            ScoreEvent::Glide(Glide::new(Seconds(t0), Seconds(t1), curve))
        }),
        (0.0_f64..1.0).prop_map(move |u| {
            ScoreEvent::Earcon(Earcon::new(Seconds(u * duration), EarconKind::Boundary))
        }),
    ]
}
