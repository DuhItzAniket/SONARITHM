// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `RenderPlan`: a Score converted for the audio thread (05 §5).
//!
//! The renderer never interprets the Score on the audio thread. The plan builder runs on a
//! worker thread and resolves everything the audio thread would otherwise compute or look
//! up: times become frames (rounded half to even, 05 §4.1), pitches become frequencies,
//! decibels become linear gains, pan becomes channel gains, instrument names become
//! [`Instrument`] values, and each track gets a voice-pool size from the polyphony of its
//! events. Every value that affects samples is computed with `libm` or IEEE arithmetic, so
//! the plan is identical on every platform (ADR-0013).

mod build;
#[cfg(test)]
mod tests;

pub use build::{PlanError, Planned, earcon_duration};

use sonarithm_core::{Frames, Hz, SampleRate, TrackId, TrackRole};
use sonarithm_dsp::PanGains;

/// The mix buses of the render graph (06 §3). Speech tracks have no bus: speech is never
/// rendered to audio by default (05 §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bus {
    /// Data voices.
    Data,
    /// Context voices: ticks and the reference tone.
    Context,
    /// Earcons.
    Earcon,
    /// The monitoring soundscape.
    Monitor,
}

impl Bus {
    /// Every bus, in mixing order.
    pub const ALL: [Self; 4] = [Self::Data, Self::Context, Self::Earcon, Self::Monitor];

    /// The bus that plays a track of `role`, or `None` for speech.
    #[must_use]
    pub fn for_role(role: TrackRole) -> Option<Self> {
        match role {
            TrackRole::Data => Some(Self::Data),
            TrackRole::Context => Some(Self::Context),
            TrackRole::Earcon => Some(Self::Earcon),
            TrackRole::Monitor => Some(Self::Monitor),
            TrackRole::Speech => None,
        }
    }

    /// The position of the bus in [`Bus::ALL`].
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::Data => 0,
            Self::Context => 1,
            Self::Earcon => 2,
            Self::Monitor => 3,
        }
    }
}

/// A built-in instrument preset (sound-design §3.1, FR-071). Instrument libraries and
/// sampled instruments come with AXONOTE (C04).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Instrument {
    /// Harmonic partials with amplitudes 1, 0.5, 0.25, 0.125 and a decay within the note:
    /// the default timbre (SDR-29; partial amplitudes are a sound-design §3.1 proposal).
    #[default]
    Mallet,
    /// A pure sine with the same envelope (SDR-29).
    Sine,
    /// The band-limited discrete square for BLV chart reading (SDR-29), with a comfort
    /// low-pass (sound-design §11).
    Square,
    /// A triangle wave.
    Triangle,
    /// A band-limited sawtooth.
    Saw,
    /// A broadband click: seeded white noise low-passed near 8 kHz for comfort, for the
    /// tick track (SDR-16; sound-design §7.1). It ignores pitch and brightness.
    Click,
}

impl Instrument {
    /// The preset named `name` in a Score's `InstrumentRef`, or `None` if it is unknown.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "mallet" => Some(Self::Mallet),
            "sine" => Some(Self::Sine),
            "square" => Some(Self::Square),
            "triangle" => Some(Self::Triangle),
            "saw" => Some(Self::Saw),
            "click" => Some(Self::Click),
            _ => None,
        }
    }

    /// The preset's name, as written in a Score.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Mallet => "mallet",
            Self::Sine => "sine",
            Self::Square => "square",
            Self::Triangle => "triangle",
            Self::Saw => "saw",
            Self::Click => "click",
        }
    }
}

/// Settings fixed for the life of a plan.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PlanConfig {
    /// The device or export sample rate.
    pub sample_rate: SampleRate,
    /// ISO 226:2023 equal-loudness compensation at this loudness level in phon, or `None`
    /// for none. On by default (SDR-24) at
    /// [`REFERENCE_PHON`](sonarithm_dsp::loudness::REFERENCE_PHON). Skipped when the
    /// Score's header shows that a loudness shaper already ran.
    pub loudness_compensation: Option<f64>,
}

impl PlanConfig {
    /// A configuration for `sample_rate`, with equal-loudness compensation (SDR-24).
    #[must_use]
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            sample_rate,
            loudness_compensation: Some(sonarithm_dsp::loudness::REFERENCE_PHON),
        }
    }

    /// The same configuration at another sample rate.
    #[must_use]
    pub fn at_rate(self, sample_rate: SampleRate) -> Self {
        Self {
            sample_rate,
            ..self
        }
    }

    /// The same configuration without equal-loudness compensation.
    #[must_use]
    pub fn without_compensation(self) -> Self {
        Self {
            loudness_compensation: None,
            ..self
        }
    }
}

/// A discrete tone, fully resolved.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Tone {
    /// The fundamental frequency.
    pub frequency: Hz,
    /// Linear gain of the note, relative to its track.
    pub gain: f32,
    /// Channel gains from the track and note pan.
    pub pan: PanGains,
    /// Brightness in [0, 1].
    pub brightness: f32,
    /// The instrument that plays the tone.
    pub instrument: Instrument,
    /// The onset and offset ramp in frames (SDR-22).
    pub ramp: u32,
}

/// A glide: its breakpoints are `TrackPlan::breakpoints[first..first + count]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct GlideRef {
    /// Index of the first breakpoint in the track's breakpoint list.
    pub first: u32,
    /// Number of breakpoints; at least 1.
    pub count: u32,
    /// The instrument that plays the glide.
    pub instrument: Instrument,
    /// The onset and offset ramp in frames (SDR-22).
    pub ramp: u32,
}

/// What a planned event plays.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum EventKind {
    /// A tone (a note, or one tap of a note).
    Tone(Tone),
    /// A continuous contour.
    Glide(GlideRef),
    /// An earcon.
    Earcon {
        /// Which earcon.
        kind: crate::earcon::EarconSound,
        /// Linear gain relative to the track.
        gain: f32,
    },
}

/// One event in frames. Events of a track are sorted by start frame, ties in Score order.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PlannedEvent {
    /// The first frame.
    pub start: u64,
    /// The length in frames; the sound ends at `start + length`.
    pub length: u64,
    /// Index of the Score event in its track, for navigation and diagnostics.
    pub source: u32,
    /// What to play.
    pub kind: EventKind,
}

impl PlannedEvent {
    /// The frame after the last one.
    #[must_use]
    pub fn end(&self) -> u64 {
        self.start.saturating_add(self.length)
    }
}

/// A glide breakpoint in frames.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PlannedBreakpoint {
    /// The absolute frame.
    pub frame: u64,
    /// Pitch in cents relative to A4.
    pub cents: f64,
    /// Gain in dB relative to the track.
    pub gain_db: f64,
    /// Stereo position, track pan included, in [−1, 1].
    pub pan: f64,
    /// Brightness in [0, 1].
    pub brightness: f64,
}

/// One track of the plan.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TrackPlan {
    /// The Score track ID.
    pub id: TrackId,
    /// The bus it plays on.
    pub bus: Bus,
    /// Linear track gain.
    pub gain: f32,
    /// The track's default instrument.
    pub instrument: Instrument,
    /// The most events sounding at once.
    pub polyphony: usize,
    /// The voice-pool size: twice the polyphony, so voices fading out during a seek or loop
    /// crossfade (06 §5) never force a steal.
    pub voices: usize,
    /// The longest event in frames, which bounds the search for events still sounding at a
    /// seek target.
    pub longest: u64,
    /// A seed for the track's noise sources, derived from the Score seed (SDR-02).
    pub noise_seed: u64,
    /// The events, sorted by start frame.
    pub events: Vec<PlannedEvent>,
    /// Glide breakpoints, referenced by [`GlideRef`].
    pub breakpoints: Vec<PlannedBreakpoint>,
}

/// A Score prepared for rendering at one sample rate.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RenderPlan {
    /// The sample rate every frame count refers to.
    pub sample_rate: SampleRate,
    /// The rendered length: the Score duration, or the end of the last event if later.
    pub length: Frames,
    /// The tracks that make sound, in Score order.
    pub tracks: Vec<TrackPlan>,
}

impl RenderPlan {
    /// The total voice-pool size over all tracks.
    #[must_use]
    pub fn voice_count(&self) -> usize {
        self.tracks.iter().map(|track| track.voices).sum()
    }

    /// The total number of planned events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.tracks.iter().map(|track| track.events.len()).sum()
    }
}
