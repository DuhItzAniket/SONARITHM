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
    sine_turns_unit,
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

/// The mallet's brightness is a spectral tilt rather than a filter: partial `k` is weighted
/// by `t^(k−1)` with `t = 2^(2b − 1)` (0.5 at brightness 0, 1 at 0.5, 2 at 1; A05
/// provisional mapping), then the weights are normalized to sum to 1.
fn mallet_weights(increment: f64, brightness: f64) -> [f64; 4] {
    let tilt = sonarithm_core::math::exp2(2.0 * brightness.clamp(0.0, 1.0) - 1.0);
    let mut weights = [0.0; 4];
    let mut factor = 1.0;
    for (k, (weight, amplitude)) in weights.iter_mut().zip(MALLET_PARTIALS).enumerate() {
        // Band-limited: partials at or above 0.45 of the sample rate are left out.
        #[expect(clippy::cast_precision_loss, reason = "k is at most 4")]
        let harmonic = (k + 1) as f64;
        if harmonic * increment < PARTIAL_LIMIT {
            *weight = amplitude * factor;
        }
        factor *= tilt;
    }
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 {
        for weight in &mut weights {
            *weight /= sum;
        }
    }
    weights
}

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

/// Samples per chunk when the mallet computes its phases ahead of its partials.
const MALLET_CHUNK: usize = 64;

/// The low-pass cutoff of the click (sound-design §7.1: "low-passed near 8 kHz for
/// comfort").
const CLICK_CUTOFF: Hz = Hz(8_000.0);

/// The sound source of a voice.
#[derive(Clone, Copy, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "voices live in preallocated pools; boxing would allocate on the audio thread"
)]
enum Source {
    /// One oscillator: sine, triangle, saw or square.
    Wave(Oscillator),
    /// Harmonic partials of one phase (the mallet).
    Mallet {
        phase: f64,
        increment: f64,
        brightness: f64,
        weights: [f64; 4],
    },
    /// An earcon synthesizer.
    Earcon(Earcon),
    /// Seeded white noise through a low-pass (the click).
    Noise { state: u64, filter: Biquad },
    /// An audified series: a buffer of the track plan, read from `pos` in steps of `step`
    /// (the transport speed), linearly interpolated (RFC-0007 §3).
    Buffer { index: u32, pos: f64, step: f64 },
}

impl Source {
    /// A click with seeded noise (SDR-02), so every render of a Score sounds the same.
    fn click(seed: u64, rate: SampleRate) -> Self {
        Self::Noise {
            // xorshift needs a non-zero state.
            state: crate::earcon::splitmix(seed) | 1,
            filter: Biquad::new(
                BiquadKind::LowPass,
                CLICK_CUTOFF,
                sonarithm_dsp::filter::BUTTERWORTH_Q,
                rate,
            ),
        }
    }

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
                brightness: 0.5,
                weights: [0.0; 4],
            },
        }
    }

    fn set_frequency(&mut self, frequency: Hz, rate: SampleRate) {
        match self {
            Self::Wave(osc) => osc.set_frequency(frequency, rate),
            Self::Mallet {
                increment,
                brightness,
                weights,
                ..
            } => {
                let inc = frequency.0 / f64::from(rate.get());
                *increment = if inc.is_finite() && inc > 0.0 {
                    inc
                } else {
                    0.0
                };
                *weights = mallet_weights(*increment, *brightness);
            }
            Self::Earcon(_) | Self::Noise { .. } | Self::Buffer { .. } => {}
        }
    }

    /// Writes the next `out.len()` samples: the same samples as calling `next_sample` for
    /// each.
    fn fill(&mut self, out: &mut [f32]) {
        match self {
            Self::Wave(osc) => osc.fill(out),
            // With the phase in [0, 1) and an increment below 1/2 the phase stays in [0, 1),
            // so the partials can use the cheaper sine (PB-03). Any other mallet, which only
            // a frequency near the sample rate can produce, plays sample by sample.
            Self::Mallet {
                phase,
                increment,
                weights,
                ..
            } if *phase < 1.0 && *increment < 0.5 => {
                mallet_fill(out, phase, *increment, *weights);
            }
            _ => {
                for sample in out {
                    *sample = self.next_sample();
                }
            }
        }
    }

    /// Sets the brightness of the mallet's partials; other sources are filtered instead.
    fn set_brightness(&mut self, value: f64) {
        if let Self::Mallet {
            increment,
            brightness,
            weights,
            ..
        } = self
        {
            *brightness = value;
            *weights = mallet_weights(*increment, value);
        }
    }

    #[inline]
    fn next_sample(&mut self) -> f32 {
        match self {
            Self::Wave(osc) => osc.next_sample(),
            Self::Mallet {
                phase,
                increment,
                weights,
                ..
            } => {
                // sin(kθ) by the Chebyshev recurrence sin(kθ) = 2cos θ · sin((k−1)θ) −
                // sin((k−2)θ): two polynomial evaluations for all four partials.
                let p = *phase;
                let (s1, cos) = (sine_turns(p), sine_turns(p + 0.25));
                let s2 = 2.0 * cos * s1;
                let s3 = 2.0 * cos * s2 - s1;
                let s4 = 2.0 * cos * s3 - s2;
                let [w1, w2, w3, w4] = *weights;
                let sum = w1 * s1 + w2 * s2 + w3 * s3 + w4 * s4;
                *phase += *increment;
                if *phase >= 1.0 {
                    *phase -= 1.0;
                }
                to_f32(sum)
            }
            Self::Earcon(earcon) => earcon.next_sample(),
            // Buffers are read in `Voice::render`, which has the track plan.
            Self::Buffer { .. } => 0.0,
            Self::Noise { state, filter } => {
                *state ^= *state << 13;
                *state ^= *state >> 7;
                *state ^= *state << 17;
                // The top 24 bits as a uniform value in [−1, 1).
                #[expect(clippy::cast_precision_loss, reason = "24 bits fit an f32 exactly")]
                let uniform = (*state >> 40) as f32 / 8_388_608.0 - 1.0;
                filter.process(uniform)
            }
        }
    }
}

/// The mallet's samples for a phase in [0, 1) and an increment below 1/2: the same
/// arithmetic as `Source::next_sample`, with the phases of a chunk computed first so the
/// partials of several samples can be computed per instruction (PB-03).
fn mallet_fill(out: &mut [f32], phase: &mut f64, increment: f64, weights: [f64; 4]) {
    let [w1, w2, w3, w4] = weights;
    let mut phases = [0.0; MALLET_CHUNK];
    for chunk in out.chunks_mut(MALLET_CHUNK) {
        for slot in phases.iter_mut().take(chunk.len()) {
            *slot = *phase;
            *phase += increment;
            if *phase >= 1.0 {
                *phase -= 1.0;
            }
        }
        for (sample, &p) in chunk.iter_mut().zip(phases.iter()) {
            // p is in [0, 1) and p + 1/4 in [1/4, 5/4]: both in `sine_turns_unit`'s range.
            let (s1, cos) = (sine_turns_unit(p), sine_turns_unit(p + 0.25));
            let s2 = 2.0 * cos * s1;
            let s3 = 2.0 * cos * s2 - s1;
            let s4 = 2.0 * cos * s3 - s2;
            *sample = to_f32(w1 * s1 + w2 * s2 + w3 * s3 + w4 * s4);
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
        self.source.set_brightness(brightness);
        self.source.set_frequency(Hz(frequency), rate);
        // A buffer starts part-way through after a seek, and follows the transport speed.
        if let Source::Buffer { pos, step, .. } = &mut self.source {
            #[expect(
                clippy::cast_precision_loss,
                reason = "frame offsets below 2^53"
            )]
            let offset = at.offset as f64;
            *pos = offset * at.speed;
            *step = at.speed;
        }
        // The sine is pure, the mallet tilts its partials, earcons are fixed: the other
        // waveforms get a brightness low-pass.
        if matches!(self.source, Source::Wave(_)) && instrument != Instrument::Sine {
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

    /// Adds `left.len()` frames into the bus buffers. The work is done in runs between
    /// the voice's own events (release point, end, glide control points), each run in
    /// tight loops over `scratch`: source, filter, envelope and fade, then mix. Every frame
    /// gets the same operations in the same order whatever the run boundaries, so the
    /// output does not depend on the block size.
    ///
    /// RT-safe: bounded by the buffer length; `scratch` is preallocated by the graph.
    pub(crate) fn render(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        track: &TrackPlan,
        rate: SampleRate,
        scratch: &mut [f32],
    ) {
        let frames = left.len().min(right.len());
        let mut done = 0;
        while done < frames && self.active {
            if self.glide.is_some() && self.countdown == 0 {
                self.control(track, rate);
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
            let mut run = (frames - done).min(scratch.len());
            if self.pos < self.release_at {
                run = run.min(usize::try_from(self.release_at - self.pos).unwrap_or(usize::MAX));
            }
            run = run.min(usize::try_from(self.len.saturating_sub(self.pos)).unwrap_or(usize::MAX));
            if self.glide.is_some() {
                run = run.min(self.countdown as usize);
            }
            let (Some(buffer), Some(l), Some(r)) = (
                scratch.get_mut(..run),
                left.get_mut(done..done + run),
                right.get_mut(done..done + run),
            ) else {
                break;
            };
            if run == 0 {
                self.active = false;
                break;
            }
            match (&mut self.source, &mut self.filter) {
                // One pass, so the oscillator's arithmetic overlaps the filter's (PB-03).
                (Source::Wave(osc), Some(filter)) => osc.fill_through(buffer, filter),
                (Source::Buffer { index, pos, step }, _) => {
                    let samples = track
                        .buffers
                        .get(*index as usize)
                        .map_or(&[][..], |b| &b[..]);
                    for sample in buffer.iter_mut() {
                        *sample = read_buffer(samples, *pos);
                        *pos += *step;
                    }
                }
                (source, filter) => {
                    source.fill(buffer);
                    if let Some(filter) = filter {
                        filter.process_buffer(buffer);
                    }
                }
            }
            if let Some(fade) = &mut self.fade {
                for sample in buffer.iter_mut() {
                    let mut level = self.env.next_level();
                    let x = f64::from(fade.remaining) / f64::from(fade.total);
                    let s = sine_turns(x / 4.0);
                    level *= to_f32(s * s);
                    fade.remaining = fade.remaining.saturating_sub(1);
                    *sample *= level;
                }
            } else {
                self.env.apply(buffer);
            }
            if self.glide.is_some() {
                for ((l, r), sample) in l.iter_mut().zip(r.iter_mut()).zip(buffer.iter()) {
                    self.left += self.step_left;
                    self.right += self.step_right;
                    *l += sample * self.left;
                    *r += sample * self.right;
                }
                self.countdown -= u32::try_from(run).unwrap_or(u32::MAX).min(self.countdown);
            } else {
                let (gain_left, gain_right) = (self.left, self.right);
                for ((l, r), sample) in l.iter_mut().zip(r.iter_mut()).zip(buffer.iter()) {
                    *l += sample * gain_left;
                    *r += sample * gain_right;
                }
            }
            self.pos += run as u64;
            done += run;
            let faded = self.fade.is_some_and(|fade| fade.remaining == 0);
            if self.pos >= self.len || !self.env.is_active() || faded {
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
        let brightness = mix(a.brightness, b.brightness);
        self.source.set_brightness(brightness);
        if let Some(filter) = &mut self.filter {
            let cutoff = brightness_cutoff(frequency.0, brightness, self.instrument, rate);
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
                source: if tone.instrument == Instrument::Click {
                    Source::click(seed, rate)
                } else {
                    Source::new(tone.instrument)
                },
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
                    source: if glide.instrument == Instrument::Click {
                        Source::click(seed, rate)
                    } else {
                        Source::new(glide.instrument)
                    },
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
            // SDR-22's minimum ramps at both ends; centred, so nothing is in pan (A11Y-14).
            EventKind::Audify { buffer, gain } => Self {
                instrument: Instrument::Sine,
                ramp: ramp_frames(MIN_RAMP, rate),
                gain,
                pan: PanGains::constant_power(Pan::CENTRE),
                frequency: 0.0,
                brightness: 1.0,
                source: Source::Buffer {
                    index: buffer,
                    pos: 0.0,
                    step: 1.0,
                },
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

/// The buffer at a fractional position, linearly interpolated; 0 outside it.
fn read_buffer(samples: &[f32], pos: f64) -> f32 {
    if pos.is_nan() || pos < 0.0 {
        return 0.0;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "checked non-negative; positions below 2^53"
    )]
    let index = pos.floor() as usize;
    let Some(&a) = samples.get(index) else {
        return 0.0;
    };
    let b = samples.get(index + 1).copied().unwrap_or(0.0);
    #[expect(clippy::cast_precision_loss, reason = "positions below 2^53")]
    let frac = to_f32(pos - index as f64);
    a + (b - a) * frac
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
            buffers: Vec::new(),
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

    /// PB-03 (#185): the mallet's chunked fill gives exactly the samples of its per-sample
    /// path, for any buffer size, and a mallet outside the fast path's range still plays
    /// sample by sample.
    #[test]
    fn the_mallet_fill_matches_its_per_sample_path_bit_for_bit() {
        for hz in [400.0, 1_400.0, 9_000.0, 30_000.0] {
            for brightness in [0.0, 0.5, 1.0] {
                let mallet = || {
                    let mut source = Source::new(Instrument::Mallet);
                    source.set_brightness(brightness);
                    source.set_frequency(Hz(hz), SampleRate::HZ_48000);
                    source
                };
                let mut by_sample = mallet();
                let expected: Vec<u32> = (0..3_000)
                    .map(|_| by_sample.next_sample().to_bits())
                    .collect();
                let mut by_fill = mallet();
                let mut actual = vec![0.0_f32; 3_000];
                for chunk in actual.chunks_mut(83) {
                    by_fill.fill(chunk);
                }
                let actual: Vec<u32> = actual.iter().map(|s| s.to_bits()).collect();
                assert_eq!(actual, expected, "{hz} Hz, brightness {brightness}");
            }
        }
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
        voice.render(
            &mut left,
            &mut right,
            &track,
            SampleRate::HZ_48000,
            &mut vec![0.0; 256],
        );
        assert!(!voice.is_active());
        assert!(
            left[4_799].abs() < 1e-6,
            "released to silence by the last frame"
        );
        assert!(left[4_800..].iter().all(|s| *s == 0.0));
    }
}
