// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The plan builder: Score → [`RenderPlan`] (05 §5). Runs off the audio thread.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use sonarithm_core::engine::TrackRng;
use sonarithm_core::score::InvariantViolation;
use sonarithm_core::{
    Cents, Db, Diagnostic, Diagnostics, EarconKind, Frames, InstrumentRef, Note, Pan, SampleRate,
    Score, ScoreEvent, ScoreTrack, Seconds, ShaperStage, TrackId,
};
use sonarithm_dsp::env::{note_ramp, ramp_frames};
use sonarithm_dsp::{Compensation, PanGains};

use super::{
    Bus, EventKind, GlideRef, Instrument, PlanConfig, PlannedBreakpoint, PlannedEvent, RenderPlan,
    Tone, TrackPlan,
};
use crate::codes;

/// Why a Score cannot be planned.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum PlanError {
    /// The Score breaks an invariant of 05 §4.2.
    #[error("the Score breaks an invariant: {0}")]
    InvalidScore(#[from] InvariantViolation),
    /// A time does not fit in a frame count at this sample rate.
    #[error("the Score is too long to render at {rate}")]
    TooLong {
        /// The sample rate.
        rate: SampleRate,
    },
    /// A track has more events or breakpoints than a plan can index (2^32).
    #[error("track `{track}` has too many events to render")]
    TooManyEvents {
        /// The track.
        track: TrackId,
    },
}

/// A plan and the diagnostics raised while building it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Planned {
    /// The plan.
    pub plan: RenderPlan,
    /// Warnings, such as an unknown instrument replaced by the default.
    pub diagnostics: Diagnostics,
}

/// The nominal duration of an earcon (keyboard-and-navigation §11): `E-BOUNDARY` about
/// 60 ms, `E-LANDMARK` about 120 ms, `E-THRESHOLD` about 30 ms. The catalogue gives no
/// duration for `E-STREAM` and `E-ALARM`; 150 ms and 600 ms are A05 placeholders until
/// AUDAXIS designs them (B03).
#[must_use]
pub fn earcon_duration(kind: EarconKind) -> Seconds {
    crate::earcon::EarconSound::from(kind).duration()
}

impl RenderPlan {
    /// Builds the plan for `score` at the configured sample rate.
    ///
    /// Speech tracks are skipped (05 §4.1). A note with `taps > 1` becomes that many tones
    /// of equal length, each with its own ramps (A11Y-13). Every tone lasts at least two
    /// ramps of [`note_ramp`] (SDR-22), so a note shorter than 10 ms is lengthened to 10 ms.
    /// A glide with no breakpoints is silent.
    ///
    /// # Errors
    ///
    /// [`PlanError::InvalidScore`] if the Score breaks an invariant, [`PlanError::TooLong`]
    /// if a time has no frame count, [`PlanError::TooManyEvents`] if a track cannot be
    /// indexed with `u32`.
    pub fn build(score: &Score, config: &PlanConfig) -> Result<Planned, PlanError> {
        score.check()?;
        let rate = config.sample_rate;
        let mut diagnostics = Diagnostics::default();
        let mut length = frame(score.header.duration, rate)?;
        let mut tracks = Vec::new();
        // SDR-24: compensate here unless the compiler's loudness shaper already did.
        let phon = config
            .loudness_compensation
            .filter(|_| !score.header.shapers.contains(&ShaperStage::Loudness));
        for track in &score.tracks {
            let Some(bus) = Bus::for_role(track.role) else {
                continue;
            };
            let compensation = phon.and_then(|phon| compensation(track, phon));
            let plan = plan_track(
                track,
                bus,
                score.header.seed,
                rate,
                compensation,
                &mut diagnostics,
            )?;
            let end = plan.events.iter().map(PlannedEvent::end).max().unwrap_or(0);
            length = length.max(end);
            tracks.push(plan);
        }
        Ok(Planned {
            plan: Self {
                sample_rate: rate,
                length: Frames(length),
                tracks,
            },
            diagnostics,
        })
    }
}

fn frame(t: Seconds, rate: SampleRate) -> Result<u64, PlanError> {
    t.to_frames(rate)
        .map(|frames| frames.0)
        .ok_or(PlanError::TooLong { rate })
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "gains and brightness are f32 on the audio thread (coding-standards §10)"
)]
fn to_f32(x: f64) -> f32 {
    x as f32
}

fn resolve(name: &InstrumentRef, track: &TrackId, diagnostics: &mut Diagnostics) -> Instrument {
    Instrument::from_name(&name.0).unwrap_or_else(|| {
        diagnostics.push(Diagnostic::new(
            &codes::UNKNOWN_INSTRUMENT,
            format!(
                "track `{track}`: unknown instrument `{}`; the mallet preset plays instead",
                name.0
            ),
        ));
        Instrument::default()
    })
}

/// The equal-loudness compensation for a track's pitch range: its resolved
/// `pitch_range`, or else the span of its note and glide pitches. `None` for a track with
/// no pitched events.
fn compensation(track: &ScoreTrack, phon: f64) -> Option<Compensation> {
    let (low, high) = if let Some(range) = track.pitch_range {
        (range.low, range.high)
    } else {
        let pitches = track.events.iter().flat_map(|event| -> Vec<Cents> {
            match event {
                ScoreEvent::Note(note) => vec![note.pitch],
                ScoreEvent::Glide(glide) => glide.curve.iter().map(|p| p.pitch).collect(),
                _ => Vec::new(),
            }
        });
        pitches.fold(None, |span: Option<(Cents, Cents)>, pitch| match span {
            None => Some((pitch, pitch)),
            Some((low, high)) => Some((Cents(low.0.min(pitch.0)), Cents(high.0.max(pitch.0)))),
        })?
    };
    Some(Compensation::for_range(low.to_hz(), high.to_hz(), phon))
}

fn plan_track(
    track: &ScoreTrack,
    bus: Bus,
    seed: u64,
    rate: SampleRate,
    compensation: Option<Compensation>,
    diagnostics: &mut Diagnostics,
) -> Result<TrackPlan, PlanError> {
    let instrument = resolve(&track.instrument, &track.id, diagnostics);
    let too_many = || PlanError::TooManyEvents {
        track: track.id.clone(),
    };
    let mut events = Vec::with_capacity(track.events.len());
    let mut breakpoints = Vec::new();
    let mut buffers: Vec<std::sync::Arc<[f32]>> = Vec::new();
    for (index, event) in track.events.iter().enumerate() {
        let source = u32::try_from(index).map_err(|_| too_many())?;
        match event {
            ScoreEvent::Note(note) => {
                let instrument = match &note.timbre.preset {
                    Some(preset) => resolve(preset, &track.id, diagnostics),
                    None => instrument,
                };
                let level = compensation.map_or(Db::ZERO, |c| c.gain(note.pitch.to_hz()));
                plan_note(
                    note,
                    track.pan,
                    instrument,
                    level,
                    source,
                    rate,
                    &mut events,
                )?;
            }
            ScoreEvent::Glide(glide) => {
                if glide.curve.is_empty() {
                    continue;
                }
                let start = frame(glide.t0, rate)?;
                let ramp = ramp_frames(note_ramp(glide.t1 - glide.t0), rate);
                let end = frame(glide.t1, rate)?.max(start + 2 * u64::from(ramp));
                let first = u32::try_from(breakpoints.len()).map_err(|_| too_many())?;
                for point in &glide.curve {
                    breakpoints.push(PlannedBreakpoint {
                        frame: frame(point.t, rate)?,
                        cents: point.pitch.0,
                        gain_db: point.gain.0
                            + compensation.map_or(0.0, |c| c.gain(point.pitch.to_hz()).0),
                        pan: Pan::clamped(track.pan.0 + point.pan.0).0,
                        brightness: point.brightness.clamp(0.0, 1.0),
                    });
                }
                let count = u32::try_from(glide.curve.len()).map_err(|_| too_many())?;
                events.push(PlannedEvent {
                    start,
                    length: end - start,
                    source,
                    kind: EventKind::Glide(GlideRef {
                        first,
                        count,
                        instrument,
                        ramp,
                    }),
                });
            }
            ScoreEvent::Earcon(earcon) => {
                let start = frame(earcon.t, rate)?;
                let length = frame(earcon_duration(earcon.kind), rate)?.max(1);
                events.push(PlannedEvent {
                    start,
                    length,
                    source,
                    kind: EventKind::Earcon {
                        kind: earcon.kind.into(),
                        gain: to_f32(earcon.gain.to_gain()),
                    },
                });
            }
            // Audification (RFC-0007 §3, SDR-31): resampled here, on the worker thread, to the
            // plan's rate; the voice plays the buffer as it is.
            ScoreEvent::Audify(audify) => {
                plan_audify(audify, source, rate, &mut buffers, &mut events)
                    .map_err(|()| too_many())?;
            }
            // Speech is routed to the announcer or the sidecar, never rendered (05 §4.1).
            _ => {}
        }
    }
    // Taps can interleave with later notes; a stable sort keeps Score order for ties.
    events.sort_by_key(|event| event.start);
    let polyphony = polyphony(&events);
    Ok(TrackPlan {
        id: track.id.clone(),
        bus,
        gain: to_f32(track.gain.to_gain()),
        instrument,
        polyphony,
        voices: 2 * polyphony,
        longest: events.iter().map(|event| event.length).max().unwrap_or(0),
        noise_seed: TrackRng::new(seed, &track.id).next_u64(),
        events,
        breakpoints,
        buffers,
    })
}

/// Plans an audification (RFC-0007 §3, SDR-31): its samples are resampled here, on the
/// worker thread, to the plan's rate, and the voice plays the buffer as it is. `Err` when
/// the track has more buffers or frames than the plan can index.
fn plan_audify(
    audify: &sonarithm_core::Audify,
    source: u32,
    rate: SampleRate,
    buffers: &mut Vec<std::sync::Arc<[f32]>>,
    events: &mut Vec<PlannedEvent>,
) -> Result<(), ()> {
    let resampled = crate::resample::resample(&audify.samples, audify.rate, f64::from(rate.get()));
    if resampled.is_empty() {
        return Ok(());
    }
    let start = frame(audify.t, rate).map_err(|_| ())?;
    let length = u64::try_from(resampled.len()).map_err(|_| ())?;
    let buffer = u32::try_from(buffers.len()).map_err(|_| ())?;
    buffers.push(std::sync::Arc::from(resampled));
    events.push(PlannedEvent {
        start,
        length,
        source,
        kind: EventKind::Audify {
            buffer,
            gain: to_f32(audify.gain.to_gain()),
        },
    });
    Ok(())
}

/// Plans a note as one tone per tap (A11Y-13).
fn plan_note(
    note: &Note,
    track_pan: Pan,
    instrument: Instrument,
    compensation: Db,
    source: u32,
    rate: SampleRate,
    events: &mut Vec<PlannedEvent>,
) -> Result<(), PlanError> {
    let taps = note.taps.max(1);
    let tap = Seconds(note.dur.0 / f64::from(taps));
    let ramp = ramp_frames(note_ramp(tap), rate);
    let tone = Tone {
        frequency: note.pitch.to_hz(),
        gain: to_f32((note.gain + compensation).to_gain()),
        pan: PanGains::constant_power(Pan::clamped(track_pan.0 + note.pan.0)),
        brightness: to_f32(note.timbre.brightness.clamp(0.0, 1.0)),
        instrument,
        ramp,
    };
    for k in 0..taps {
        let t0 = note.t + Seconds(note.dur.0 * f64::from(k) / f64::from(taps));
        let t1 = note.t + Seconds(note.dur.0 * f64::from(k + 1) / f64::from(taps));
        let start = frame(t0, rate)?;
        let end = frame(t1, rate)?.max(start + 2 * u64::from(ramp));
        events.push(PlannedEvent {
            start,
            length: end - start,
            source,
            kind: EventKind::Tone(tone),
        });
    }
    Ok(())
}

/// The most events sounding at once: a sweep over start-sorted events with a heap of end
/// frames. An event ending at frame `f` frees its voice for an event starting at `f`.
pub(super) fn polyphony(events: &[PlannedEvent]) -> usize {
    let mut ends = BinaryHeap::new();
    let mut most = 0;
    for event in events {
        while ends.peek().is_some_and(|Reverse(end)| *end <= event.start) {
            ends.pop();
        }
        ends.push(Reverse(event.end()));
        most = most.max(ends.len());
    }
    most
}
