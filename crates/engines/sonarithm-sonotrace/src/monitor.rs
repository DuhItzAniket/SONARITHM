// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The monitoring soundscape (sound-design §10, §11; SDR-35, SDR-39, A11Y-08; B06-T04).
//!
//! In `mode: "monitoring"` the Score gains a continuous ambient bed on the monitor bus and
//! the earcons of detected events:
//!
//! - **Salience, not volume** (§10.2). Each event brightens the bed for a moment and lets
//!   it settle back; the bed's level never changes (hearing safety, §12). A more salient
//!   event brightens it more.
//! - **Events** (§10.3). Every detector event plays `E-STREAM`.
//! - **Alarms** (§10.4, SDR-35). `E-ALARM` sounds only for an alarm the user configured in
//!   `playback.monitor.alarms` with `salience: "high"` (07 SONOTRACE); nothing else sounds
//!   urgent.
//! - **Comfort** (§11, SDR-39). The share of the session the bed spends bright is reported
//!   as an experimental `Info` diagnostic: SDR-39 is an inference, not a validated budget.
//!
//! Detector events are the compile's annotations (ECHOMETRIC's detectors from C05, or any
//! analyzer), placed at the time of the data note of their first row. The bed's design is
//! provisional: §10.7 makes the default soundscape a B06 deliverable evaluated with users
//! before it becomes a default, so every value below is a proposal.

use std::borrow::Borrow;

use sonarithm_core::diagnostic::Diagnostic;
use sonarithm_core::engine::{
    Annotation, CompileContext, CompileError, ContextProvider, SpecFragment,
};
use sonarithm_core::score::{Mode, TrackRole};
use sonarithm_core::{
    Breakpoint, Cents, Db, Earcon, EarconKind, EngineId, Glide, InstrumentRef, Score, ScoreEvent,
    ScoreTrack, Seconds, TrackId,
};
use sonarithm_data::DataBundle;
use sonarithm_spec::sections::{Detector, Salience};

use crate::codes;

/// The bed's pitch: A3, 220 Hz, low and calm. Proposal (§10.7).
pub const BED_PITCH: Cents = Cents(-1_200.0);
/// The bed's resting brightness. Proposal (§10.7).
pub const BED_BRIGHTNESS: f64 = 0.15;
/// The brightness an event of low, medium and high salience lifts the bed to. Proposal.
pub const SALIENCE_BRIGHTNESS: [f64; 3] = [0.35, 0.55, 0.8];
/// How fast the bed brightens on an event, and how long it takes to settle. Proposals.
pub const RISE: Seconds = Seconds(0.25);
/// See [`RISE`].
pub const SETTLE: Seconds = Seconds(2.0);
/// The built-in bed's instrument until B06's evaluation chooses a soundscape (§10.7).
pub const BED_INSTRUMENT: &str = "triangle";
/// The one soundscape this build has.
pub const DEFAULT_SOUNDSCAPE: &str = "default";
/// Brightness above which the bed counts as bright for the comfort report (SDR-39, INFER).
pub const COMFORT_BRIGHT: f64 = 0.5;

/// A detector event the soundscape reacts to.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct MonitorEvent {
    /// When it happened in the Score.
    pub t: Seconds,
    /// How salient it is.
    pub salience: Salience,
    /// A configured alarm: plays `E-ALARM` instead of `E-STREAM`.
    pub alarm: bool,
}

impl MonitorEvent {
    /// An event at `t`.
    #[must_use]
    pub const fn new(t: Seconds, salience: Salience, alarm: bool) -> Self {
        Self { t, salience, alarm }
    }
}

/// The soundscape of a session: the bed, the event earcons, and the comfort report.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Soundscape {
    /// The ambient bed on the monitor bus.
    pub bed: ScoreTrack,
    /// The events' earcons, on the earcon bus.
    pub earcons: ScoreTrack,
    /// The share of the session the bed is brighter than [`COMFORT_BRIGHT`], in [0, 1].
    pub bright_share: f64,
}

fn level(salience: Salience) -> f64 {
    match salience {
        Salience::Low => SALIENCE_BRIGHTNESS[0],
        Salience::Medium => SALIENCE_BRIGHTNESS[1],
        Salience::High => SALIENCE_BRIGHTNESS[2],
    }
}

/// The bed's brightness at `t`: the resting level, or the brightest event envelope
/// (a linear rise over [`RISE`], then a linear settle over [`SETTLE`]).
fn brightness(events: &[MonitorEvent], t: f64) -> f64 {
    events.iter().fold(BED_BRIGHTNESS, |bright, event| {
        let peak = level(event.salience);
        let since = t - event.t.0;
        let envelope = if since < 0.0 {
            BED_BRIGHTNESS
        } else if since < RISE.0 {
            BED_BRIGHTNESS + (peak - BED_BRIGHTNESS) * since / RISE.0
        } else if since < RISE.0 + SETTLE.0 {
            peak - (peak - BED_BRIGHTNESS) * (since - RISE.0) / SETTLE.0
        } else {
            BED_BRIGHTNESS
        };
        bright.max(envelope)
    })
}

/// The soundscape of a session of `duration` with `events` (SDR-35).
#[must_use]
pub fn soundscape(engine: &EngineId, duration: Seconds, events: &[MonitorEvent]) -> Soundscape {
    let end = duration.0.max(0.0);
    // Breakpoints at the session's ends and at each event's corners, so the linear
    // interpolation between them follows the envelope.
    let mut times = vec![0.0, end];
    for event in events {
        for t in [event.t.0, event.t.0 + RISE.0, event.t.0 + RISE.0 + SETTLE.0] {
            if (0.0..=end).contains(&t) {
                times.push(t);
            }
        }
    }
    times.sort_by(f64::total_cmp);
    times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let curve: Vec<Breakpoint> = times
        .iter()
        .map(|&t| {
            let mut point = Breakpoint::new(Seconds(t), BED_PITCH);
            point.brightness = brightness(events, t);
            point
        })
        .collect();
    let bright_share = bright_share(&curve, end);
    let mut bed = ScoreTrack::new(
        TrackId::new("monitor-bed"),
        TrackRole::Monitor,
        engine.clone(),
        InstrumentRef(BED_INSTRUMENT.to_owned()),
    );
    if end > 0.0 {
        bed.events = vec![ScoreEvent::Glide(Glide::new(
            Seconds(0.0),
            Seconds(end),
            curve,
        ))];
    }
    let mut earcons = ScoreTrack::new(
        TrackId::new("monitor-events"),
        TrackRole::Earcon,
        engine.clone(),
        InstrumentRef("sine".to_owned()),
    );
    let mut sorted: Vec<&MonitorEvent> = events
        .iter()
        .filter(|e| (0.0..end).contains(&e.t.0))
        .collect();
    sorted.sort_by(|a, b| a.t.0.total_cmp(&b.t.0));
    earcons.events = sorted
        .into_iter()
        .map(|event| {
            let kind = if event.alarm {
                EarconKind::Alarm
            } else {
                EarconKind::Stream
            };
            ScoreEvent::Earcon(Earcon::new(event.t, kind))
        })
        .collect();
    earcons.gain = Db::ZERO;
    Soundscape {
        bed,
        earcons,
        bright_share,
    }
}

/// The share of `[0, end]` where the piecewise-linear `curve` is brighter than
/// [`COMFORT_BRIGHT`].
fn bright_share(curve: &[Breakpoint], end: f64) -> f64 {
    if end <= 0.0 {
        return 0.0;
    }
    let mut bright = 0.0;
    for pair in curve.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let span = b.t.0 - a.t.0;
        if span <= 0.0 {
            continue;
        }
        let (ya, yb) = (a.brightness - COMFORT_BRIGHT, b.brightness - COMFORT_BRIGHT);
        bright += match (ya > 0.0, yb > 0.0) {
            (true, true) => span,
            (false, false) => 0.0,
            // One crossing: the part above the threshold.
            (true, false) => span * ya / (ya - yb),
            (false, true) => span * yb / (yb - ya),
        };
    }
    (bright / end).clamp(0.0, 1.0)
}

/// The events of `annotations`: each at the time of the first data note of its first
/// row, with the salience its detector's configured alarm gives (`low` when none does).
#[must_use]
pub fn events(
    score: &Score,
    annotations: &[Annotation],
    alarms: &[(Detector, Salience)],
) -> Vec<MonitorEvent> {
    annotations
        .iter()
        .filter_map(|annotation| {
            let t = score
                .tracks
                .iter()
                .filter(|track| track.role == TrackRole::Data)
                .flat_map(|track| &track.events)
                .filter_map(|event| match event {
                    ScoreEvent::Note(note) => note
                        .src
                        .filter(|src| src.row == annotation.range.start)
                        .map(|_| note.t),
                    _ => None,
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))?;
            let configured = alarms
                .iter()
                .find(|(detector, _)| detector_name(*detector) == annotation.source.as_str())
                .map(|(_, salience)| *salience);
            let salience = configured.unwrap_or(Salience::Low);
            Some(MonitorEvent::new(
                t,
                salience,
                configured == Some(Salience::High),
            ))
        })
        .collect()
}

/// The analyzer ID a detector's annotations carry: its spec name.
fn detector_name(detector: Detector) -> &'static str {
    match detector {
        Detector::Extrema => "extrema",
        Detector::Outliers => "outliers",
        Detector::Changepoints => "changepoints",
        Detector::Periodicity => "periodicity",
        Detector::TrendSegments => "trend-segments",
        _ => "critical-points",
    }
}

/// The context provider SONOTRACE registers: the soundscape of a monitoring Score.
pub(crate) struct MonitorContext;

impl<D: Borrow<DataBundle>> ContextProvider<D> for MonitorContext {
    fn context_tracks(
        &self,
        spec: &SpecFragment,
        score: &Score,
        _data: &D,
        annotations: &[Annotation],
        cx: &mut CompileContext,
    ) -> Result<Vec<ScoreTrack>, CompileError> {
        if score.header.mode != Mode::Monitoring {
            return Ok(Vec::new());
        }
        let spec: sonarithm_spec::Spec = spec.parse()?;
        let monitor = spec.playback.and_then(|p| p.monitor);
        let (soundscape_name, alarms) = monitor.map_or((None, Vec::new()), |m| {
            (
                m.soundscape,
                m.alarms
                    .into_iter()
                    .map(|alarm| (alarm.detector, alarm.salience))
                    .collect(),
            )
        });
        if let Some(name) = soundscape_name.filter(|n| n != DEFAULT_SOUNDSCAPE) {
            cx.report(Diagnostic::new(
                &codes::SOUNDSCAPE_UNAVAILABLE,
                format!(
                    "SONOTRACE: the soundscape `{name}` is not in this build; the default bed \
                     plays instead (sound-design §10.7)."
                ),
            ));
        }
        let events = events(score, annotations, &alarms);
        let soundscape = soundscape(&crate::ENGINE, score.header.duration, &events);
        cx.report(Diagnostic::new(
            &codes::COMFORT_BUDGET,
            format!(
                "SONOTRACE (experimental, SDR-39 INFER): the soundscape is bright for {:.0}% of \
                 the session. The comfort budget is an inference, not a validated limit.",
                soundscape.bright_share * 100.0
            ),
        ));
        Ok(vec![soundscape.bed, soundscape.earcons])
    }
}

#[cfg(test)]
#[path = "monitor/tests.rs"]
mod tests;
