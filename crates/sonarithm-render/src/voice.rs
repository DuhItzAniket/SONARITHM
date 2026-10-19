// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Voices and voice pools (06 §3). Real-time only after construction.
//!
//! A voice plays one planned event: a tone, a glide or an earcon. Pools are preallocated
//! from the plan's polyphony (05 §5); when a pool is full, the oldest voice is stolen, a
//! deterministic rule (coding-standards §11.2), and the steal is counted.
//!
//! Glides update pitch, gain, pan and brightness every [`CONTROL_PERIOD`] frames of the
//! voice's own clock, so the output does not depend on how the host splits blocks.

use sonarithm_core::{Cents, Db, Hz, Pan, SampleRate, Seconds};
use sonarithm_dsp::env::{MIN_RAMP, ramp_frames};
use sonarithm_dsp::{
    Adsr, AdsrParams, Biquad, BiquadKind, Oscillator, PanGains, Stage, Waveform, sine_turns,
};

use crate::earcon::Earcon;
use crate::plan::{EventKind, Instrument, PlannedBreakpoint, PlannedEvent, TrackPlan};

/// Frames between glide parameter updates (0.67 ms at 48 kHz).
pub(crate) const CONTROL_PERIOD: u32 = 32;

/// Mallet partial amplitudes, normalized so their sum is 1 (sound-design §3.1 proposal:
/// 1, 0.5, 0.25, 0.125).
const MALLET_PARTIALS: [f64; 4] = [8.0 / 15.0, 4.0 / 15.0, 2.0 / 15.0, 1.0 / 15.0];

/// The mallet's decay: a time constant of 150 ms towards 30 % of the peak. A05
/// provisional values; sound-design §3.1 asks only for "percussive decay within the note"
/// (B07 co-design).
const MALLET_DECAY: Seconds = Seconds(0.150);
const MALLET_SUSTAIN: f64 = 0.3;

/// The square preset's comfort low-pass never opens above 8 kHz (sound-design §11;
/// A05 provisional value).
const SQUARE_COMFORT_CUTOFF: f64 = 8_000.0;

/// The highest partial frequency kept, as a fraction of the sample rate.
const PARTIAL_LIMIT: f64 = 0.45;

/// The low-pass cutoff for a brightness in [0, 1]: from 2 to 32 times the fundamental
/// (A05 provisional mapping), capped at 0.45 of the sample rate.
fn brightness_cutoff(
    frequency: f64,
    brightness: f64,
    instrument: Instrument,
    rate: SampleRate,
) -> Hz {
    let mut cutoff = frequency * sonarithm_core::math::exp2(1.0 + 4.0 * brightness.clamp(0.0, 1.0));
    if instrument == Instrument::Square {
        cutoff = cutoff.min(SQUARE_COMFORT_CUTOFF);
    }
    Hz(cutoff.min(PARTIAL_LIMIT * f64::from(rate.get())))
}

/// The sound source of a voice.
#[derive(Clone, Copy, Debug)]
enum Source {
    /// One oscillator: sine, triangle, saw or square.
    Wave(Oscillator),
    /// Harmonic partials of one phase (the mallet).
    Mallet {
        phase: f64,
        increment: f64,
        partials: usize,
    },
    /// An earcon synthesizer.
    Earcon(Earcon),
}

impl Source {
    fn new(instrument: Instrument) -> Self {
        let wave = |waveform| Self::Wave(Oscillator::new(waveform));
        match instrument {
            Instrument::Sine => wave(Waveform::Sine),
            Instrument::Square => wave(Waveform::Square),
            Instrument::Triangle => wave(Waveform::Triangle),
            Instrument::Saw => wave(Waveform::Saw),
            _ => Self::Mallet {
                phase: 0.0,
                increment: 0.0,
                partials: 0,
            },
        }
    }

    fn set_frequency(&mut self, frequency: Hz, rate: SampleRate) {
        match self {
            Self::Wave(osc) => osc.set_frequency(frequency, rate),
            Self::Mallet {
                increment,
                partials,
                ..
            } => {
                let inc = frequency.0 / f64::from(rate.get());
                *increment = if inc.is_finite() && inc > 0.0 {
                    inc
                } else {
                    0.0
                };
                *partials = (1..=MALLET_PARTIALS.len())
                    .take_while(|k| *k as f64 * *increment < PARTIAL_LIMIT)
                    .count();
            }
            Self::Earcon(_) => {}
        }
    }

    #[inline]
    fn next_sample(&mut self) -> f32 {
        match self {
            Self::Wave(osc) => osc.next_sample(),
            Self::Mallet {
                phase,
                increment,
                partials,
            } => {
                let p = *phase;
                let mut sum = 0.0;
                for (k, amplitude) in MALLET_PARTIALS.iter().enumerate().take(*partials) {
                    sum += amplitude * sine_turns(p * (k + 1) as f64);
                }
                *phase += *increment;
                if *phase >= 1.0 {
                    *phase -= 1.0;
                }
                to_f32(sum)
            }
            Self::Earcon(earcon) => earcon.next_sample(),
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "samples and gains are f32 (coding-standards §10)"
)]
#[inline]
fn to_f32(x: f64) -> f32 {
    x as f32
}

/// A glide's position in its breakpoint list, in the voice's frames.
#[derive(Clone, Copy, Debug)]
struct GlideState {
    first: u32,
    count: u32,
    /// The breakpoint at or before the current frame, relative to `first`.
    segment: u32,
    /// Frame of the event start, to turn breakpoint frames into voice frames.
    origin: u64,
    /// Speed at the start, to scale breakpoint frames to device frames.
    speed: f64,
}

/// A forced fade-out (seek, loop wrap, pause) of `total` frames.
#[derive(Clone, Copy, Debug)]
struct Fade {
    remaining: u32,
    total: u32,
}

/// One sounding event.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Voice {
    active: bool,
    /// Start order, for oldest-first stealing.
    age: u64,
    /// Frames played since the event's start (offset included).
    pos: u64,
    /// The frame at which the sound ends.
    len: u64,
    /// The frame at which the release starts.
    release_at: u64,
    env: Adsr,
    fade: Option<Fade>,
    source: Source,
    filter: Option<Biquad>,
    glide: Option<GlideState>,
    instrument: Instrument,
    left: f32,
    right: f32,
    /// Per-frame steps of `left` and `right` towards the next glide control point.
    step_left: f32,
    step_right: f32,
    countdown: u32,
}

impl Default for Voice {
    fn default() -> Self {
        let rate = SampleRate::HZ_48000;
        Self {
            active: false,
            age: 0,
            pos: 0,
            len: 0,
            release_at: 0,
            env: Adsr::new(&AdsrParams::note(Seconds(1.0)), rate),
            fade: None,
            source: Source::new(Instrument::Sine),
            filter: None,
            glide: None,
            instrument: Instrument::Sine,
            left: 0.0,
            right: 0.0,
            step_left: 0.0,
            step_right: 0.0,
            countdown: 0,
        }
    }
}

/// How a voice starts: at its onset, or part-way through (after a seek or resume).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Start {
    /// Device frames to skip into the event.
    pub(crate) offset: u64,
    /// Playback speed (score frames per device frame).
    pub(crate) speed: f64,
    /// Start order.
    pub(crate) age: u64,
    /// Seed for noise sources.
    pub(crate) noise_seed: u64,
}

impl Voice {
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn age(&self) -> u64 {
        self.age
    }

    /// Starts `event` of `track`. RT-safe: arithmetic and `libm` only.
    pub(crate) fn start(
        &mut self,
        event: &PlannedEvent,
        track: &TrackPlan,
        rate: SampleRate,
        at: Start,
    ) {
        let scale = |frames: u64| -> u64 {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss,
                reason = "frame counts below 2^53, speed validated positive by the transport"
            )]
            let scaled = (frames as f64 / at.speed).round() as u64;
            scaled
        };
        let sound = Sound::of(event, track, rate, at.noise_seed);
        let len = scale(event.length).max(2 * u64::from(sound.ramp));
        let params = envelope(
            event.kind,
            sound.instrument,
            sound.ramp,
            rate,
            at.offset > 0,
        );
        let mut env = Adsr::new(&params, rate);
        env.trigger();
        let gain = sound.gain * track.gain;
        let Sound {
            instrument,
            pan,
            frequency,
            brightness,
            source,
            ..
        } = sound;
        *self = Self {
            active: at.offset < len,
            age: at.age,
            pos: at.offset,
            len,
            release_at: len.saturating_sub(u64::from(env.release_frames())),
            env,
            fade: None,
            source,
            filter: None,
            glide: match event.kind {
                EventKind::Glide(glide) => Some(GlideState {
                    first: glide.first,
                    count: glide.count,
                    segment: 0,
                    origin: event.start,
                    speed: at.speed,
                }),
                _ => None,
            },
            instrument,
            left: gain * pan.left,
            right: gain * pan.right,
            step_left: 0.0,
            step_right: 0.0,
            countdown: 0,
        };
        self.source.set_frequency(Hz(frequency), rate);
        if !matches!(instrument, Instrument::Sine) && !matches!(self.source, Source::Earcon(_)) {
            let cutoff = brightness_cutoff(frequency, brightness, instrument, rate);
            self.filter = Some(Biquad::new(
                BiquadKind::LowPass,
                cutoff,
                sonarithm_dsp::filter::BUTTERWORTH_Q,
                rate,
            ));
        }
    }

    /// Fades the voice out over `frames` (at least one), from whatever it is playing.
    pub(crate) fn fade_out(&mut self, frames: u32) {
        if self.active {
            let frames = frames.max(1);
            self.fade = Some(match self.fade {
                Some(fade) if fade.remaining <= frames => fade,
                _ => Fade {
                    remaining: frames,
                    total: frames,
                },
            });
        }
    }

    /// Adds `left.len()` frames into the bus buffers. RT-safe: bounded by the buffer
    /// length; reads the track's breakpoints for glides.
    pub(crate) fn render(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        track: &TrackPlan,
        rate: SampleRate,
    ) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            if !self.active {
                return;
            }
            if self.glide.is_some() {
                if self.countdown == 0 {
                    self.control(track, rate);
                }
                self.countdown -= 1;
                self.left += self.step_left;
                self.right += self.step_right;
            }
            // `>=`: a voice resumed past its release point releases at once.
            if self.pos >= self.release_at
                && matches!(
                    self.env.stage(),
                    Stage::Attack | Stage::Decay | Stage::Sustain
                )
            {
                self.env.release();
            }
            let mut level = self.env.next_level();
            if let Some(fade) = &mut self.fade {
                let x = f64::from(fade.remaining) / f64::from(fade.total);
                let s = sine_turns(x / 4.0);
                level *= to_f32(s * s);
                if fade.remaining == 0 {
                    self.active = false;
                }
                fade.remaining = fade.remaining.saturating_sub(1);
            }
            let mut sample = self.source.next_sample();
            if let Some(filter) = &mut self.filter {
                sample = filter.process(sample);
            }
            let sample = sample * level;
            *l += sample * self.left;
            *r += sample * self.right;
            self.pos += 1;
            if self.pos >= self.len || (!self.env.is_active() && self.pos > 0) {
                self.active = false;
            }
        }
    }

    /// The glide control step: parameters at the next control point, reached linearly.
    fn control(&mut self, track: &TrackPlan, rate: SampleRate) {
        self.countdown = CONTROL_PERIOD;
        let Some(glide) = &mut self.glide else { return };
        let points = track
            .breakpoints
            .get(glide.first as usize..(glide.first + glide.count) as usize)
            .unwrap_or_default();
        // The score frame of the next control point.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "frame counts below 2^53; speed validated positive"
        )]
        let frame =
            glide.origin + ((self.pos + u64::from(CONTROL_PERIOD)) as f64 * glide.speed) as u64;
        while (glide.segment as usize + 1) < points.len()
            && points
                .get(glide.segment as usize + 1)
                .is_some_and(|p| p.frame <= frame)
        {
            glide.segment += 1;
        }
        let Some(&a) = points.get(glide.segment as usize) else {
            return;
        };
        let b = points.get(glide.segment as usize + 1).copied().unwrap_or(a);
        let w = if b.frame > a.frame && frame > a.frame {
            #[expect(clippy::cast_precision_loss, reason = "frame counts below 2^53")]
            let w = ((frame - a.frame) as f64 / (b.frame - a.frame) as f64).min(1.0);
            w
        } else {
            0.0
        };
        let mix = |x: f64, y: f64| x + (y - x) * w;
        let frequency = Cents(mix(a.cents, b.cents)).to_hz();
        self.source.set_frequency(frequency, rate);
        let gain = to_f32(Db(mix(a.gain_db, b.gain_db)).to_gain()) * track.gain;
        let pan = PanGains::constant_power(Pan(mix(a.pan, b.pan)));
        let period = f32::from(u16::try_from(CONTROL_PERIOD).unwrap_or(u16::MAX));
        self.step_left = (gain * pan.left - self.left) / period;
        self.step_right = (gain * pan.right - self.right) / period;
        if let Some(filter) = &mut self.filter {
            let cutoff = brightness_cutoff(
                frequency.0,
                mix(a.brightness, b.brightness),
                self.instrument,
                rate,
            );
            filter.set(cutoff, sonarithm_dsp::filter::BUTTERWORTH_Q, rate);
        }
    }
}

/// What a voice plays, resolved from its event.
struct Sound {
    instrument: Instrument,
    ramp: u32,
    gain: f32,
    pan: PanGains,
    frequency: f64,
    brightness: f64,
    source: Source,
}

impl Sound {
    fn of(event: &PlannedEvent, track: &TrackPlan, rate: SampleRate, seed: u64) -> Self {
        match event.kind {
            EventKind::Tone(tone) => Self {
                instrument: tone.instrument,
                ramp: tone.ramp,
                gain: tone.gain,
                pan: tone.pan,
                frequency: tone.frequency.0,
                brightness: f64::from(tone.brightness),
                source: Source::new(tone.instrument),
            },
            EventKind::Glide(glide) => {
                let point = track
                    .breakpoints
                    .get(glide.first as usize)
                    .copied()
                    .unwrap_or_else(|| silent_point(event.start));
                Self {
                    instrument: glide.instrument,
                    ramp: glide.ramp,
                    gain: to_f32(Db(point.gain_db).to_gain()),
                    pan: PanGains::constant_power(Pan(point.pan)),
                    frequency: Cents(point.cents).to_hz().0,
                    brightness: point.brightness,
                    source: Source::new(glide.instrument),
                }
            }
            EventKind::Earcon { kind, gain } => Self {
                instrument: Instrument::Sine,
                ramp: ramp_frames(MIN_RAMP, rate),
                gain,
                pan: PanGains::constant_power(Pan::CENTRE),
                frequency: 0.0,
                brightness: 1.0,
                source: Source::Earcon(Earcon::new(kind, rate, seed)),
            },
        }
    }
}

/// The envelope for an event. A resumed voice fades in over the minimum ramp instead of
/// its full onset.
fn envelope(
    kind: EventKind,
    instrument: Instrument,
    ramp: u32,
    rate: SampleRate,
    resumed: bool,
) -> AdsrParams {
    let params = match (kind, instrument) {
        (EventKind::Earcon { kind, .. }, _) => crate::earcon::envelope(kind),
        (EventKind::Tone(_), Instrument::Mallet) => AdsrParams {
            decay: MALLET_DECAY,
            sustain: MALLET_SUSTAIN,
            ..ramped(ramp, rate)
        },
        _ => ramped(ramp, rate),
    };
    if resumed {
        AdsrParams {
            attack: MIN_RAMP,
            ..params
        }
    } else {
        params
    }
}

fn ramped(ramp: u32, rate: SampleRate) -> AdsrParams {
    let ramp = sonarithm_core::Frames(u64::from(ramp)).to_seconds(rate);
    AdsrParams {
        attack: ramp,
        decay: MIN_RAMP,
        sustain: 1.0,
        release: ramp,
    }
}

fn silent_point(frame: u64) -> PlannedBreakpoint {
    PlannedBreakpoint {
        frame,
        cents: 0.0,
        gain_db: f64::NEG_INFINITY,
        pan: 0.0,
        brightness: 0.5,
    }
}

/// A fixed set of voices for one track.
#[derive(Debug)]
pub(crate) struct VoicePool {
    voices: Vec<Voice>,
}

impl VoicePool {
    /// A pool of `size` idle voices. Allocates: call off the audio thread.
    pub(crate) fn new(size: usize) -> Self {
        Self {
            voices: vec![Voice::default(); size.max(1)],
        }
    }

    /// A free voice, or the oldest one if none is free. Returns the voice and whether it
    /// was stolen. RT-safe: bounded by the pool size.
    pub(crate) fn allocate(&mut self) -> (&mut Voice, bool) {
        let free = self.voices.iter().position(|voice| !voice.is_active());
        let (index, stolen) = if let Some(index) = free {
            (index, false)
        } else {
            let oldest = self
                .voices
                .iter()
                .enumerate()
                .min_by_key(|(_, voice)| voice.age())
                .map_or(0, |(index, _)| index);
            (oldest, true)
        };
        // The pool is never empty (`new` keeps at least one voice).
        let len = self.voices.len();
        let voice = &mut self.voices[index.min(len - 1)];
        (voice, stolen)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Voice> {
        self.voices.iter_mut()
    }

    pub(crate) fn active(&self) -> usize {
        self.voices.iter().filter(|voice| voice.is_active()).count()
    }
}

#[cfg(test)]
mod tests {
    use sonarithm_core::TrackId;

    use super::*;
    use crate::plan::{Bus, Tone};

    fn tone_track() -> (TrackPlan, PlannedEvent) {
        let tone = Tone {
            frequency: Hz(440.0),
            gain: 1.0,
            pan: PanGains::constant_power(Pan::CENTRE),
            brightness: 0.5,
            instrument: Instrument::Sine,
            ramp: 240,
        };
        let event = PlannedEvent {
            start: 0,
            length: 4_800,
            source: 0,
            kind: EventKind::Tone(tone),
        };
        let track = TrackPlan {
            id: TrackId::new("t"),
            bus: Bus::Data,
            gain: 1.0,
            instrument: Instrument::Sine,
            polyphony: 1,
            voices: 2,
            longest: 4_800,
            noise_seed: 0,
            events: vec![event],
            breakpoints: Vec::new(),
        };
        (track, event)
    }

    /// coding-standards §11.2: a full pool steals its oldest voice, a deterministic rule.
    #[test]
    fn a_full_pool_steals_the_oldest_voice() {
        let (track, event) = tone_track();
        let mut pool = VoicePool::new(2);
        for age in [5, 3] {
            let (voice, stolen) = pool.allocate();
            assert!(!stolen);
            voice.start(
                &event,
                &track,
                SampleRate::HZ_48000,
                Start {
                    offset: 0,
                    speed: 1.0,
                    age,
                    noise_seed: 0,
                },
            );
        }
        let (voice, stolen) = pool.allocate();
        assert!(stolen);
        assert_eq!(voice.age(), 3);
        assert_eq!(pool.active(), 2);
    }

    #[test]
    fn a_tone_ends_silent_at_its_length() {
        let (track, event) = tone_track();
        let mut voice = Voice::default();
        voice.start(
            &event,
            &track,
            SampleRate::HZ_48000,
            Start {
                offset: 0,
                speed: 1.0,
                age: 0,
                noise_seed: 0,
            },
        );
        let (mut left, mut right) = (vec![0.0; 5_000], vec![0.0; 5_000]);
        voice.render(&mut left, &mut right, &track, SampleRate::HZ_48000);
        assert!(!voice.is_active());
        assert!(
            left[4_799].abs() < 1e-6,
            "released to silence by the last frame"
        );
        assert!(left[4_800..].iter().all(|s| *s == 0.0));
    }
}
