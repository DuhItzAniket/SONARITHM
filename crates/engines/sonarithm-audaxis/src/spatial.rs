// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spatial placement (07 AUDAXIS, `spatial`; 06 §3): the `Spatial` shaper.
//!
//! In stereo, the default spatial mode, the render engine places every voice with a
//! constant-power panner, so a position never changes a voice's power. The shaper keeps
//! every position finite and within [−1, 1], then runs the never-alone guard (A11Y-14):
//! essential information must survive mono output, NVDA's sound split and high-frequency
//! hearing loss, so it may not be carried only by pan, only by timbre, or by pitches
//! outside the expanded SDR-08 range. The guard looks at the finished Score, so it covers
//! every encoder; a track the encoder already warned about for pan is not warned twice.
//! Binaural and multichannel placement arrive in D03 (SDR-30).

use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};
use sonarithm_core::engine::{CompileContext, CompileError, Shaper};
use sonarithm_core::score::{Score, ScoreEvent, ScoreTrack, TrackRole};
use sonarithm_core::units::{Cents, Pan};

use crate::codes;

/// The lowest data pitch that stays inside the expanded SDR-08 range, in Hz.
pub const LOWEST_HZ: f64 = 65.0;

/// The highest data pitch that stays inside the expanded SDR-08 range, in Hz.
pub const HIGHEST_HZ: f64 = 1480.0;

/// Differences below this count as no variation.
const EPSILON: f64 = 1e-9;

/// The `Spatial` shaper AUDAXIS registers.
pub struct Spatial;

impl Shaper for Spatial {
    fn shape(&self, score: &mut Score, cx: &mut CompileContext) -> Result<(), CompileError> {
        for track in &mut score.tracks {
            place(track);
        }
        for (index, track) in score.tracks.iter().enumerate() {
            if track.role != TrackRole::Data {
                continue;
            }
            cx.check_cancelled()?;
            let at = SpecPointer::root().key("tracks").index(index);
            let found = Carriers::of(track);
            for finding in found.findings() {
                if finding == Finding::PanOnly && warned_about_pan(cx, &at) {
                    continue;
                }
                cx.report(finding.diagnostic(track, &found).at(at.clone()));
            }
        }
        Ok(())
    }
}

/// Keeps every position of a track finite and within [−1, 1].
fn place(track: &mut ScoreTrack) {
    track.pan = Pan::clamped(track.pan.0);
    for event in &mut track.events {
        match event {
            ScoreEvent::Note(note) => note.pan = Pan::clamped(note.pan.0),
            ScoreEvent::Glide(glide) => {
                for point in &mut glide.curve {
                    point.pan = Pan::clamped(point.pan.0);
                }
            }
            _ => {}
        }
    }
}

/// Whether the encoder already reported pan as the only carrier of this track.
fn warned_about_pan(cx: &CompileContext, track: &SpecPointer) -> bool {
    let pan = format!("{track}/channels/pan");
    cx.diagnostics().iter().any(|d| {
        d.rules.iter().any(|r| r.as_str() == "A11Y-14")
            && d.location.as_ref().is_some_and(|l| l.to_string() == pan)
    })
}

/// A guard finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Finding {
    PanOnly,
    TimbreOnly,
    PitchExtreme,
}

impl Finding {
    fn diagnostic(self, track: &ScoreTrack, found: &Carriers) -> Diagnostic {
        let id = &track.id;
        match self {
            Self::PanOnly => Diagnostic::new(
                &codes::PAN_ONLY,
                format!(
                    "Along track `{id}` only the stereo position changes; in mono, with NVDA's sound split or with one ear, the track carries no information."
                ),
            ),
            Self::TimbreOnly => Diagnostic::new(
                &codes::TIMBRE_ONLY,
                format!(
                    "Along track `{id}` only the timbre changes; listeners who cannot tell the timbres apart get no information."
                ),
            ),
            Self::PitchExtreme => {
                let (low, high) = found.hz.unwrap_or((LOWEST_HZ, HIGHEST_HZ));
                Diagnostic::new(
                    &codes::PITCH_EXTREME,
                    format!(
                        "Track `{id}` plays {low:.0} to {high:.0} Hz, outside 65 to 1480 Hz (SDR-08); pitches out there are lost with high-frequency hearing loss or small speakers."
                    ),
                )
            }
        }
    }
}

/// Which sound properties vary along a data track.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag per sound property; the guard reads them together"
)]
pub(crate) struct Carriers {
    pan: bool,
    pitch: bool,
    timing: bool,
    gain: bool,
    timbre: bool,
    /// The lowest and highest pitch, in Hz.
    hz: Option<(f64, f64)>,
}

/// The range of a set of values seen so far.
#[derive(Clone, Copy, Default)]
struct Span(Option<(f64, f64)>);

impl Span {
    fn add(&mut self, value: f64) {
        if value.is_finite() {
            self.0 = Some(
                self.0
                    .map_or((value, value), |(lo, hi)| (lo.min(value), hi.max(value))),
            );
        }
    }

    fn varies(self) -> bool {
        self.0.is_some_and(|(lo, hi)| hi - lo > EPSILON)
    }
}

impl Carriers {
    /// The carriers of a track's notes and glides.
    pub(crate) fn of(track: &ScoreTrack) -> Self {
        let (mut pan, mut pitch, mut duration, mut taps, mut gain, mut brightness) = (
            Span::default(),
            Span::default(),
            Span::default(),
            Span::default(),
            Span::default(),
            Span::default(),
        );
        let mut presets = Vec::new();
        for event in &track.events {
            match event {
                ScoreEvent::Note(note) => {
                    pan.add(note.pan.0);
                    pitch.add(note.pitch.0);
                    duration.add(note.dur.0);
                    taps.add(f64::from(note.taps));
                    gain.add(note.gain.0);
                    brightness.add(note.timbre.brightness);
                    if !presets.contains(&note.timbre.preset) {
                        presets.push(note.timbre.preset.clone());
                    }
                }
                ScoreEvent::Glide(glide) => {
                    for point in &glide.curve {
                        pan.add(point.pan.0);
                        pitch.add(point.pitch.0);
                        gain.add(point.gain.0);
                        brightness.add(point.brightness);
                    }
                }
                _ => {}
            }
        }
        Self {
            pan: pan.varies(),
            pitch: pitch.varies(),
            timing: duration.varies() || taps.varies(),
            gain: gain.varies(),
            timbre: presets.len() > 1 || brightness.varies(),
            hz: pitch
                .0
                .map(|(lo, hi)| (Cents(lo).to_hz().0, Cents(hi).to_hz().0)),
        }
    }

    /// Whether something that survives mono and one ear varies: pitch, timing or level.
    fn survives_mono(self) -> bool {
        self.pitch || self.timing || self.gain
    }

    fn findings(self) -> Vec<Finding> {
        let mut findings = Vec::new();
        if self.pan && !self.survives_mono() {
            findings.push(Finding::PanOnly);
        }
        if self.timbre && !self.survives_mono() {
            findings.push(Finding::TimbreOnly);
        }
        // Only a pitch that varies carries data; a constant one outside the range is not
        // information lost.
        if self.pitch
            && self
                .hz
                .is_some_and(|(low, high)| low < LOWEST_HZ - EPSILON || high > HIGHEST_HZ + EPSILON)
        {
            findings.push(Finding::PitchExtreme);
        }
        findings
    }
}
