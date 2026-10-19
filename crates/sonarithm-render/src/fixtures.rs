// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reference Scores for benchmarks, golden tests and soak tests (DS-006).
//!
//! The benchmark Score has the PB-03 shape: four tracks and 10,000 note events over
//! 60 seconds. It is generated from a seed with integer arithmetic only (`SplitMix64`), so
//! it is the same on every platform.

use sonarithm_core::{
    Db, EngineId, Hz, InstrumentRef, Mode, Note, Pan, Score, ScoreEvent, ScoreHeader, ScoreTrack,
    Seconds, SpecVersion, TrackId, TrackRole,
};

/// The seed of the reference benchmark Score.
pub const BENCHMARK_SEED: u64 = 1729;

/// `SplitMix64`: integer-only, so every platform draws the same values.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1), from the top 53 bits.
    #[expect(clippy::cast_precision_loss, reason = "53 bits fit an f64 exactly")]
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }

    fn between(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }
}

/// One track of the benchmark Score: name, role, instrument, events, note length range in
/// seconds and pitch range in Hz.
type TrackShape = (
    &'static str,
    TrackRole,
    &'static str,
    u32,
    (f64, f64),
    (f64, f64),
);

const SHAPES: [TrackShape; 4] = [
    (
        "data-a",
        TrackRole::Data,
        "mallet",
        3_000,
        (0.040, 0.160),
        (400.0, 1_400.0),
    ),
    (
        "data-b",
        TrackRole::Data,
        "square",
        3_000,
        (0.040, 0.160),
        (200.0, 800.0),
    ),
    (
        "ticks",
        TrackRole::Context,
        "sine",
        2_000,
        (0.010, 0.020),
        (900.0, 1_100.0),
    ),
    (
        "monitor",
        TrackRole::Monitor,
        "triangle",
        2_000,
        (0.100, 0.400),
        (100.0, 400.0),
    ),
];

/// The benchmark Score (DS-006): `seconds` long, 10,000 notes over four tracks (two data
/// tracks, a context track and a monitoring track), drawn from `seed`.
#[must_use]
pub fn benchmark_score(seconds: f64, seed: u64) -> Score {
    let mut rng = Rng(seed);
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, seed);
    header.duration = Seconds(seconds);
    let mut score = Score::new(header);
    for (name, role, instrument, events, (short, long), (low, high)) in SHAPES {
        let mut notes: Vec<Note> = (0..events)
            .map(|_| {
                let duration = rng.between(short, long);
                let start = rng.between(0.0, seconds - duration);
                let pitch = Hz(rng.between(low, high)).to_cents();
                let mut note = Note::new(Seconds(start), Seconds(duration), pitch);
                note.pan = Pan(rng.between(-1.0, 1.0));
                note
            })
            .collect();
        notes.sort_by(|a, b| a.t.total_cmp(&b.t));
        let mut track = ScoreTrack::new(
            TrackId::new(name),
            role,
            EngineId::from_static("numisona"),
            InstrumentRef(instrument.to_owned()),
        );
        track.gain = Db(-12.0);
        track.events = notes.into_iter().map(ScoreEvent::Note).collect();
        score.tracks.push(track);
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_benchmark_score_has_the_pb_03_shape_and_is_valid() {
        let score = benchmark_score(60.0, BENCHMARK_SEED);
        assert_eq!(score.tracks.len(), 4);
        let events: usize = score.tracks.iter().map(|t| t.events.len()).sum();
        assert_eq!(events, 10_000);
        assert!(score.check().is_ok());
        assert_eq!(
            benchmark_score(60.0, BENCHMARK_SEED),
            score,
            "deterministic"
        );
    }
}
