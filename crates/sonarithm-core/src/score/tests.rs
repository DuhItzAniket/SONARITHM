// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Invariant tests for the Score IR (05 §4.2).

use proptest::prelude::*;

use super::invariants::Location;
use super::*;
use crate::ids::{Generation, TableId};
use crate::testing::valid_score;
use crate::units::Hz;

/// A small valid Score whose values come from unit conversions.
fn fixture() -> Score {
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 1729);
    header.duration = Seconds(2.0);
    header.spec_hash = Sha256Digest::of(b"spec");
    header.data_fingerprint = Sha256Digest::of(b"data");
    header.engines.insert(
        EngineId::from_static("numisona"),
        EngineVersion("0.0.0".to_owned()),
    );
    header.shapers = vec![ShaperStage::Loudness, ShaperStage::Lint];
    let mut track = ScoreTrack::new(
        TrackId::new("anomaly"),
        TrackRole::Data,
        EngineId::from_static("numisona"),
        InstrumentRef("mallet".to_owned()),
    );
    let range = PitchRange {
        low: Hz(400.0).to_cents(),
        high: Hz(1400.0).to_cents(),
    };
    track.pitch_range = Some(range);
    for (i, hz) in [400.0, 523.251_130_6, 1400.0, 987.766_6]
        .into_iter()
        .enumerate()
    {
        let index = u32::try_from(i).unwrap();
        let mut note = Note::new(
            Seconds(f64::from(index) / 3.0),
            Seconds(0.25),
            Hz(hz).to_cents(),
        );
        note.gain = Db::from_gain(0.125);
        note.src = Some(RowRef {
            table: TableId(0),
            generation: Generation(1),
            row: u64::from(index),
        });
        note.conf = Confidence::new(0.875);
        track.events.push(ScoreEvent::Note(note));
    }
    track.events.push(ScoreEvent::Glide(Glide::new(
        Seconds(1.5),
        Seconds(1.75),
        vec![
            Breakpoint::new(Seconds(1.5), range.low),
            Breakpoint::new(Seconds(1.75), range.mirror(range.low)),
        ],
    )));
    let mut score = Score::new(header);
    score.tracks.push(track);
    score
        .markers
        .push(Marker::new(Seconds(1.0), MarkerKind::Insight, "maximum"));
    score
}

#[test]
fn fixture_satisfies_the_invariants() {
    assert_eq!(fixture().check(), Ok(()));
}

fn first_event_mut(score: &mut Score) -> &mut ScoreEvent {
    &mut score.tracks[0].events[0]
}

#[test]
fn invariant_1_events_must_be_sorted_and_start_at_or_after_zero() {
    let mut score = fixture();
    score.tracks[0].events.swap(0, 1);
    assert_eq!(
        score.check(),
        Err(InvariantViolation::Unsorted {
            at: Location::Event(0, 1)
        })
    );
    let mut score = fixture();
    if let ScoreEvent::Note(note) = first_event_mut(&mut score) {
        note.t = Seconds(-0.1);
    }
    assert!(matches!(
        score.check(),
        Err(InvariantViolation::Unsorted { .. })
    ));
}

#[test]
fn invariant_2_notes_end_by_the_duration_and_glides_are_monotonic() {
    let mut score = fixture();
    score.header.duration = Seconds(1.2);
    assert!(matches!(
        score.check(),
        Err(InvariantViolation::PastEnd { .. })
    ));
    let mut score = fixture();
    if let Some(ScoreEvent::Glide(glide)) = score.tracks[0].events.last_mut() {
        glide.curve.reverse();
    }
    assert!(matches!(
        score.check(),
        Err(InvariantViolation::GlideNotMonotonic { .. })
    ));
}

#[test]
fn invariant_3_pitches_stay_in_the_track_range() {
    let mut score = fixture();
    if let ScoreEvent::Note(note) = first_event_mut(&mut score) {
        note.pitch = Hz(1500.0).to_cents();
    }
    assert_eq!(
        score.check(),
        Err(InvariantViolation::PitchOutOfRange {
            at: Location::Event(0, 0)
        })
    );
}

/// SDR-10: Analysis mode keeps pitch unquantized, so the quantization shaper must not run.
#[test]
fn sdr_10_invariant_5_no_quantization_in_analysis_mode() {
    let mut score = fixture();
    score.header.shapers.insert(0, ShaperStage::Quantize);
    assert_eq!(score.check(), Err(InvariantViolation::QuantizedInAnalysis));
    score.header.mode = Mode::Presentation;
    assert_eq!(score.check(), Ok(()));
}

#[test]
fn numbers_must_be_finite_and_in_their_declared_ranges() {
    let mut score = fixture();
    if let ScoreEvent::Note(note) = first_event_mut(&mut score) {
        note.pan = Pan(1.5);
    }
    assert!(matches!(
        score.check(),
        Err(InvariantViolation::OutOfRange { field: "pan", .. })
    ));
    let mut score = fixture();
    if let ScoreEvent::Note(note) = first_event_mut(&mut score) {
        note.taps = 0;
    }
    assert!(matches!(
        score.check(),
        Err(InvariantViolation::OutOfRange { field: "taps", .. })
    ));
}

proptest! {
    /// Generated Scores satisfy invariants 1–3 and 5.
    #[test]
    fn generated_scores_satisfy_the_invariants(score in valid_score()) {
        prop_assert_eq!(score.check(), Ok(()));
    }

    /// Invariant 4 / SDR-15 at the range level: mirroring stays in range and mirroring
    /// twice returns the pitch within 1e-9 cents.
    #[test]
    fn sdr_15_mirroring_twice_is_the_identity(
        low in -6000.0_f64..6000.0,
        width in 1.0_f64..6000.0,
        w in 0.0_f64..=1.0,
    ) {
        let range = PitchRange { low: Cents(low), high: Cents(low + width) };
        let pitch = Cents((low + w * width).clamp(low, low + width));
        let once = range.mirror(pitch);
        prop_assert!(range.contains(once));
        prop_assert!((range.mirror(once).0 - pitch.0).abs() < 1e-9);
    }
}
