// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Score invariants of 05 §4.2 and testing-strategy §3.
//!
//! The compiler checks them in debug builds after every compile (`debug_assert!`), and
//! property tests check them on generated Scores. Invariant 4 (polarity flips mirror
//! pitches) is a property of encoders and is tested with them (A06); invariant 6
//! (byte-identical output) is tested through the canonical serialization.

use super::{Breakpoint, Mode, Score, ScoreEvent, ShaperStage};
use crate::units::Seconds;

/// A broken Score invariant. `track` and `event` are zero-based indices.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum InvariantViolation {
    /// A number is NaN or infinite.
    #[error("{at}: `{field}` is not a finite number")]
    NonFinite {
        /// Where.
        at: Location,
        /// Which field.
        field: &'static str,
    },
    /// A number lies outside its declared range (pan, brightness, taps, durations).
    #[error("{at}: `{field}` is outside its range")]
    OutOfRange {
        /// Where.
        at: Location,
        /// Which field.
        field: &'static str,
    },
    /// Invariant 1: an event starts before 0 s or before the previous event.
    #[error("{at}: events must start at or after 0 s and be sorted by start time")]
    Unsorted {
        /// Where.
        at: Location,
    },
    /// Invariant 2: a note or glide ends after the Score duration.
    #[error("{at}: the event ends after the Score duration")]
    PastEnd {
        /// Where.
        at: Location,
    },
    /// Invariant 2: glide breakpoints are not in time order inside `[t0, t1]`.
    #[error("{at}: glide breakpoints must be in time order between t0 and t1")]
    GlideNotMonotonic {
        /// Where.
        at: Location,
    },
    /// Invariant 3: a pitch lies outside the track's resolved pitch range.
    #[error("{at}: the pitch lies outside the track's pitch range")]
    PitchOutOfRange {
        /// Where.
        at: Location,
    },
    /// Invariant 5: the quantization shaper ran in Analysis mode (SDR-10).
    #[error("the quantization shaper ran in Analysis mode (SDR-10)")]
    QuantizedInAnalysis,
}

/// Where in the Score a violation is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    /// The header.
    Header,
    /// A track's own fields.
    Track(usize),
    /// An event of a track.
    Event(usize, usize),
    /// A marker.
    Marker(usize),
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Header => f.write_str("header"),
            Self::Track(track) => write!(f, "track {track}"),
            Self::Event(track, event) => write!(f, "track {track}, event {event}"),
            Self::Marker(marker) => write!(f, "marker {marker}"),
        }
    }
}

type Check = Result<(), InvariantViolation>;

fn finite(at: Location, field: &'static str, value: f64) -> Check {
    if value.is_finite() {
        Ok(())
    } else {
        Err(InvariantViolation::NonFinite { at, field })
    }
}

fn within(at: Location, field: &'static str, value: f64, low: f64, high: f64) -> Check {
    finite(at, field, value)?;
    if (low..=high).contains(&value) {
        Ok(())
    } else {
        Err(InvariantViolation::OutOfRange { at, field })
    }
}

impl Score {
    /// Checks the invariants of 05 §4.2 (1, 2, 3 and 5) plus finiteness and declared
    /// ranges of every number, and returns the first violation.
    ///
    /// # Errors
    ///
    /// The first [`InvariantViolation`] found, in header, track, event, marker order.
    pub fn check(&self) -> Result<(), InvariantViolation> {
        let header = &self.header;
        let duration = header.duration.0;
        within(Location::Header, "duration", duration, 0.0, f64::MAX)?;
        if header.mode == Mode::Analysis && header.shapers.contains(&ShaperStage::Quantize) {
            return Err(InvariantViolation::QuantizedInAnalysis);
        }
        for (index, track) in self.tracks.iter().enumerate() {
            let at = Location::Track(index);
            finite(at, "gain", track.gain.0)?;
            within(at, "pan", track.pan.0, -1.0, 1.0)?;
            if let Some(range) = &track.pitch_range {
                finite(at, "pitch_range.low", range.low.0)?;
                within(at, "pitch_range.high", range.high.0, range.low.0, f64::MAX)?;
            }
            let mut previous = Seconds::ZERO;
            for (event_index, event) in track.events.iter().enumerate() {
                let at = Location::Event(index, event_index);
                check_event(at, event, duration)?;
                let start = event.start();
                if start.0 < previous.0 {
                    return Err(InvariantViolation::Unsorted { at });
                }
                previous = start;
                if let Some(range) = &track.pitch_range {
                    let in_range = match event {
                        ScoreEvent::Note(note) => range.contains(note.pitch),
                        ScoreEvent::Glide(glide) => {
                            glide.curve.iter().all(|point| range.contains(point.pitch))
                        }
                        _ => true,
                    };
                    if !in_range {
                        return Err(InvariantViolation::PitchOutOfRange { at });
                    }
                }
            }
        }
        let mut previous = 0.0;
        for (index, marker) in self.markers.iter().enumerate() {
            let at = Location::Marker(index);
            within(at, "t", marker.t.0, previous, duration)
                .map_err(|_| InvariantViolation::Unsorted { at })?;
            previous = marker.t.0;
        }
        Ok(())
    }
}

/// Finiteness, ranges and end time of one event. Start times are checked by the caller.
fn check_event(at: Location, event: &ScoreEvent, duration: f64) -> Check {
    match event {
        ScoreEvent::Note(note) => {
            within(at, "t", note.t.0, 0.0, f64::MAX)
                .map_err(|_| InvariantViolation::Unsorted { at })?;
            within(at, "dur", note.dur.0, f64::MIN_POSITIVE, f64::MAX)?;
            finite(at, "pitch", note.pitch.0)?;
            finite(at, "gain", note.gain.0)?;
            within(at, "pan", note.pan.0, -1.0, 1.0)?;
            within(at, "timbre.brightness", note.timbre.brightness, 0.0, 1.0)?;
            if note.taps == 0 {
                return Err(InvariantViolation::OutOfRange { at, field: "taps" });
            }
            if note.t.0 + note.dur.0 > duration {
                return Err(InvariantViolation::PastEnd { at });
            }
        }
        ScoreEvent::Glide(glide) => {
            within(at, "t0", glide.t0.0, 0.0, f64::MAX)
                .map_err(|_| InvariantViolation::Unsorted { at })?;
            within(at, "t1", glide.t1.0, glide.t0.0, f64::MAX)
                .map_err(|_| InvariantViolation::GlideNotMonotonic { at })?;
            if glide.t1.0 > duration {
                return Err(InvariantViolation::PastEnd { at });
            }
            let mut previous = glide.t0.0;
            for point in &glide.curve {
                check_breakpoint(at, point)?;
                if point.t.0 < previous || point.t.0 > glide.t1.0 {
                    return Err(InvariantViolation::GlideNotMonotonic { at });
                }
                previous = point.t.0;
            }
        }
        ScoreEvent::Earcon(earcon) => {
            within(at, "t", earcon.t.0, 0.0, duration)?;
            finite(at, "gain", earcon.gain.0)?;
        }
        ScoreEvent::Speech(speech) => {
            within(at, "t", speech.t.0, 0.0, duration)?;
        }
    }
    Ok(())
}

fn check_breakpoint(at: Location, point: &Breakpoint) -> Check {
    finite(at, "curve.t", point.t.0)?;
    finite(at, "curve.pitch", point.pitch.0)?;
    finite(at, "curve.gain", point.gain.0)?;
    within(at, "curve.pan", point.pan.0, -1.0, 1.0)?;
    within(at, "curve.brightness", point.brightness, 0.0, 1.0)
}
