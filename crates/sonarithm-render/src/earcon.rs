// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Earcon synthesis (keyboard-and-navigation §11; sound-design §8).
//!
//! The B03 set (B03-T06) follows the catalogue's acoustic descriptions and the earcon
//! rules of sound-design §8: noise-based or inharmonic families no data voice uses
//! (ED-01), at most about 150 ms except `E-ALARM` (ED-02), rising for on and enter and
//! falling for off and exit (ED-03), main energy below about 4 kHz (ED-06). The
//! frequencies and lengths are proposals awaiting the listening check (HP-11). Noise is
//! seeded (SDR-02), so every render of the same Score sounds the same.
//!
//! [`EarconSound`] covers the earcons a Score can contain ([`EarconKind`]) and the
//! interactive ones the app plays live with [`crate::Command::Earcon`]: invalid action,
//! mode and level changes, and the gap of a missing value.

use sonarithm_core::{EarconKind, Hz, SampleRate, Seconds};
use sonarithm_dsp::env::{MIN_RAMP, ramp_frames};
use sonarithm_dsp::{AdsrParams, Biquad, BiquadKind, sine_turns};

/// An earcon of the catalogue (keyboard §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EarconSound {
    /// `E-BOUNDARY`: a dull low "bonk" of band-limited noise near 250 Hz, about 60 ms.
    Boundary,
    /// `E-INVALID`: two soft blips falling a minor third, about 40 ms each.
    Invalid,
    /// `E-LANDMARK`: a bell-like inharmonic ping, about 120 ms, decaying.
    Landmark,
    /// `E-THRESHOLD`: a mid-register wood-block "tock", about 30 ms.
    Threshold,
    /// `E-MODE` on or enter: a rising two-note dyad, about 60 ms per note.
    ModeOn,
    /// `E-MODE` off or exit: the falling dyad.
    ModeOff,
    /// `E-STREAM`: a soft chime, never urgent.
    Stream,
    /// `E-ALARM`: a repeating pulsed pattern, only for user-configured alarms (SDR-35).
    Alarm,
    /// `E-LEVEL` into a child level: a short upward glide, about 50 ms.
    LevelUp,
    /// `E-LEVEL` out to the parent: a short downward glide.
    LevelDown,
    /// `E-GAP`: two soft noise taps, the missing value at the cursor (03 §5).
    Gap,
    /// `E-ZERO`: a dry inharmonic tick, about 40 ms, every partial below 4 kHz (RFC-0007).
    Zero,
    /// `E-ASYMPTOTE`: a swelling "whoosh" of band-passed noise, about 120 ms (RFC-0007).
    Asymptote,
    /// `E-UNDEFINED`: a dull hush of low-passed noise, about 100 ms (RFC-0007).
    Undefined,
    /// `E-OFFSCREEN`: two quick taps of band-passed noise near 2 kHz, about 80 ms (RFC-0007;
    /// band-passed, not high-passed, so most energy stays below 4 kHz, ED-06).
    Offscreen,
}

impl From<EarconKind> for EarconSound {
    fn from(kind: EarconKind) -> Self {
        match kind {
            EarconKind::Boundary => Self::Boundary,
            EarconKind::Landmark => Self::Landmark,
            EarconKind::Threshold => Self::Threshold,
            EarconKind::Alarm => Self::Alarm,
            EarconKind::Zero => Self::Zero,
            EarconKind::Asymptote => Self::Asymptote,
            EarconKind::Undefined => Self::Undefined,
            EarconKind::Offscreen => Self::Offscreen,
            _ => Self::Stream,
        }
    }
}

impl EarconSound {
    /// Every earcon sound, in catalogue order.
    pub const ALL: [Self; 15] = [
        Self::Boundary,
        Self::Invalid,
        Self::Landmark,
        Self::Threshold,
        Self::ModeOn,
        Self::ModeOff,
        Self::Stream,
        Self::Alarm,
        Self::LevelUp,
        Self::LevelDown,
        Self::Gap,
        Self::Zero,
        Self::Asymptote,
        Self::Undefined,
        Self::Offscreen,
    ];

    /// How long the earcon sounds, ramps included.
    #[must_use]
    pub fn duration(self) -> Seconds {
        Seconds::from_millis(match self {
            Self::Boundary => 60.0,
            Self::Invalid => 80.0,
            Self::Landmark | Self::ModeOn | Self::ModeOff => 120.0,
            Self::Threshold => 30.0,
            Self::Stream => 150.0,
            Self::Alarm => 600.0,
            Self::LevelUp | Self::LevelDown => 50.0,
            Self::Gap => 70.0,
            Self::Zero => 40.0,
            Self::Asymptote => 120.0,
            Self::Undefined => 100.0,
            Self::Offscreen => 80.0,
        })
    }
}

/// Up to three sine partials: frequency in Hz and amplitude.
type Partials = [(f64, f64); 3];

/// Harmonic partials at `f`, softened (2f and 3f at a third and a twelfth).
const fn soft(f: f64, amplitude: f64) -> Partials {
    [
        (f, amplitude),
        (2.0 * f, amplitude / 3.0),
        (3.0 * f, amplitude / 12.0),
    ]
}

/// A minor third down, as a frequency ratio (2^(−3/12)).
const MINOR_THIRD_DOWN: f64 = 0.840_896_415_253_714_5;
/// A major third up (2^(4/12)).
const MAJOR_THIRD_UP: f64 = 1.259_921_049_894_873_2;
/// E-LEVEL glides by a fifth (3:2).
const LEVEL_RATIO: f64 = 1.5;

/// E-LANDMARK: a bell-like inharmonic ping (ratios 1, 2.76, 5.40).
const LANDMARK: Partials = [(880.0, 0.6), (2_428.8, 0.3), (4_752.0, 0.1)];
/// E-THRESHOLD: a mid-register wood-block tock (inharmonic 1 : 2.5).
const THRESHOLD: Partials = [(1_000.0, 0.8), (2_500.0, 0.2), (0.0, 0.0)];
/// E-STREAM: a soft chime.
const STREAM: Partials = [(1_320.0, 0.7), (2_640.0, 0.2), (0.0, 0.0)];
/// E-ALARM: a pulsed tone.
const ALARM: Partials = [(1_000.0, 0.7), (1_500.0, 0.2), (0.0, 0.0)];
/// E-ZERO: a struck-bar tick (ratios 1 : 2.76 : 5.4 from 700 Hz), every partial below
/// 4 kHz (ED-06, RFC-0007 amendment 2). A proposal until the listening check (HP-21).
const ZERO: Partials = [(700.0, 0.6), (1_932.0, 0.3), (3_780.0, 0.1)];
/// E-ALARM pulses: 100 ms on, 100 ms off.
const ALARM_PULSE: Seconds = Seconds(0.100);

/// What a part of an earcon is made of.
#[derive(Clone, Copy, Debug)]
enum Body {
    /// Sine partials, gliding by `ratio` over the part.
    Tone { partials: Partials, ratio: f64 },
    /// The earcon's noise source through its filter.
    Noise,
}

/// One part of an earcon: when it starts and how long it lasts, in seconds.
#[derive(Clone, Copy, Debug)]
struct Part {
    start: f64,
    length: f64,
    body: Body,
}

/// The earcon's noise filter: kind, centre or cutoff, Q and output gain.
type NoiseSpec = (BiquadKind, f64, f64, f32);

/// The parts of an earcon and its noise filter, if it uses noise.
fn recipe(sound: EarconSound) -> ([Option<Part>; 2], Option<NoiseSpec>) {
    let tone = |start, length, partials| {
        Some(Part {
            start,
            length,
            body: Body::Tone {
                partials,
                ratio: 1.0,
            },
        })
    };
    let noise = |start, length| {
        Some(Part {
            start,
            length,
            body: Body::Noise,
        })
    };
    match sound {
        EarconSound::Boundary => (
            [noise(0.0, 0.060), None],
            Some((BiquadKind::BandPass, 250.0, 1.5, 6.0)),
        ),
        EarconSound::Invalid => (
            [
                tone(0.0, 0.040, soft(660.0, 0.5)),
                tone(0.040, 0.040, soft(660.0 * MINOR_THIRD_DOWN, 0.5)),
            ],
            None,
        ),
        EarconSound::Landmark => ([tone(0.0, 0.120, LANDMARK), None], None),
        EarconSound::Threshold => ([tone(0.0, 0.030, THRESHOLD), None], None),
        EarconSound::ModeOn => (
            [
                tone(0.0, 0.060, soft(523.25, 0.6)),
                tone(0.060, 0.060, soft(523.25 * MAJOR_THIRD_UP, 0.6)),
            ],
            None,
        ),
        EarconSound::ModeOff => (
            [
                tone(0.0, 0.060, soft(523.25 * MAJOR_THIRD_UP, 0.6)),
                tone(0.060, 0.060, soft(523.25, 0.6)),
            ],
            None,
        ),
        EarconSound::Stream => ([tone(0.0, 0.150, STREAM), None], None),
        EarconSound::Alarm => ([tone(0.0, 0.600, ALARM), None], None),
        EarconSound::LevelUp | EarconSound::LevelDown => {
            let (from, ratio) = if sound == EarconSound::LevelUp {
                (440.0, LEVEL_RATIO)
            } else {
                (660.0, 1.0 / LEVEL_RATIO)
            };
            let part = Part {
                start: 0.0,
                length: 0.050,
                body: Body::Tone {
                    partials: soft(from, 0.6),
                    ratio,
                },
            };
            ([Some(part), None], None)
        }
        EarconSound::Gap => (
            [noise(0.0, 0.010), noise(0.060, 0.010)],
            Some((BiquadKind::LowPass, 3_000.0, 0.707, 1.0)),
        ),
        // The RFC-0007 set (C03-T04), proposals until HP-21: noise-based or inharmonic
        // (ED-01), short (ED-02), main energy below 4 kHz (ED-06).
        EarconSound::Zero => ([tone(0.0, 0.040, ZERO), None], None),
        EarconSound::Asymptote => (
            [noise(0.0, 0.120), None],
            Some((BiquadKind::BandPass, 1_000.0, 0.8, 5.0)),
        ),
        EarconSound::Undefined => (
            [noise(0.0, 0.100), None],
            Some((BiquadKind::LowPass, 200.0, 0.707, 3.0)),
        ),
        EarconSound::Offscreen => (
            [noise(0.0, 0.025), noise(0.055, 0.025)],
            Some((BiquadKind::BandPass, 2_000.0, 1.2, 4.0)),
        ),
    }
}

/// The envelope of an earcon: the minimum onset ramp (SDR-22), then an exponential decay
/// for the single percussive sounds, or a plateau for the two-part ones, whose parts are
/// gated individually.
pub(crate) fn envelope(sound: EarconSound) -> AdsrParams {
    let (decay, sustain) = match sound {
        EarconSound::Boundary => (0.045, 0.0),
        EarconSound::Landmark => (0.100, 0.0),
        EarconSound::Threshold => (0.025, 0.0),
        EarconSound::Stream => (0.060, 0.0),
        EarconSound::Zero => (0.035, 0.0),
        _ => (MIN_RAMP.0, 1.0),
    };
    // E-ASYMPTOTE swells into its whoosh; E-UNDEFINED fades in softly.
    let attack = match sound {
        EarconSound::Asymptote => Seconds(0.060),
        EarconSound::Undefined => Seconds(0.020),
        _ => MIN_RAMP,
    };
    let release = match sound {
        EarconSound::Asymptote => Seconds(0.050),
        EarconSound::Undefined => Seconds(0.030),
        _ => MIN_RAMP,
    };
    AdsrParams {
        attack,
        decay: Seconds(decay),
        sustain,
        release,
    }
}

/// A part in frames.
#[derive(Clone, Copy, Debug)]
struct FramePart {
    start: u32,
    end: u32,
    tone: bool,
    /// Per-frame multiplier of the partial increments (a glide).
    glide: f64,
}

/// An earcon generator. RT-safe after construction: arithmetic and `libm` only.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Earcon {
    phases: [f64; 3],
    increments: [f64; 3],
    amplitudes: [f64; 3],
    /// The second part's partials, when it is a different tone.
    next: Option<([f64; 3], [f64; 3])>,
    parts: [Option<FramePart>; 2],
    /// Raised-cosine edges of each part, in frames, when there are two parts.
    edge: Option<u32>,
    /// Noise source, filter and gain.
    noise: Option<(u64, Biquad, f32)>,
    /// The E-ALARM pulse gate.
    pulse: Option<Pulse>,
    pos: u32,
}

impl Earcon {
    pub(crate) fn new(sound: EarconSound, rate: SampleRate, seed: u64) -> Self {
        let sr = f64::from(rate.get());
        let frames = |seconds: f64| ramp_frames(Seconds(seconds), rate);
        let increments = |partials: Partials| {
            partials.map(|(hz, _)| {
                if hz > 0.0 && hz < 0.45 * sr {
                    hz / sr
                } else {
                    0.0
                }
            })
        };
        let (parts, noise) = recipe(sound);
        let mut earcon = Self {
            phases: [0.0; 3],
            increments: [0.0; 3],
            amplitudes: [0.0; 3],
            next: None,
            parts: [None; 2],
            edge: parts[1].is_some().then(|| frames(MIN_RAMP.0)),
            noise: noise.map(|(kind, hz, q, gain)| {
                // xorshift needs a non-zero state.
                (splitmix(seed) | 1, Biquad::new(kind, Hz(hz), q, rate), gain)
            }),
            pulse: (sound == EarconSound::Alarm).then(|| Pulse {
                on: frames(ALARM_PULSE.0),
                period: 2 * frames(ALARM_PULSE.0),
                edge: frames(MIN_RAMP.0),
                pos: 0,
            }),
            pos: 0,
        };
        // Part boundaries round to the nearest frame, so consecutive parts meet exactly.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "earcon parts last well under a second"
        )]
        let at = |seconds: f64| (seconds * sr).round().max(0.0) as u32;
        for (slot, part) in parts.into_iter().enumerate() {
            let Some(part) = part else { continue };
            let start = at(part.start);
            let end = start + at(part.length).max(1);
            let (tone, glide) = match part.body {
                Body::Tone { partials, ratio } => {
                    let incs = increments(partials);
                    let amps = partials.map(|(_, a)| a);
                    if slot == 0 {
                        earcon.increments = incs;
                        earcon.amplitudes = amps;
                    } else {
                        earcon.next = Some((incs, amps));
                    }
                    let length = f64::from(end - start);
                    (true, sonarithm_core::math::pow(ratio, 1.0 / length))
                }
                Body::Noise => (false, 1.0),
            };
            earcon.parts[slot] = Some(FramePart {
                start,
                end,
                tone,
                glide,
            });
        }
        earcon
    }

    /// The part sounding at frame `pos`, and its gate.
    fn part_at(&self, pos: u32) -> Option<(usize, FramePart, f32)> {
        self.parts.iter().enumerate().find_map(|(index, part)| {
            let part = (*part)?;
            if pos < part.start || pos >= part.end {
                return None;
            }
            let gate = self.edge.map_or(1.0, |edge| {
                let edge = edge.max(1);
                let level = (pos - part.start).min(part.end - 1 - pos).min(edge);
                let s = sine_turns(f64::from(level) / f64::from(edge) / 4.0);
                to_f32(s * s)
            });
            Some((index, part, gate))
        })
    }

    #[inline]
    pub(crate) fn next_sample(&mut self) -> f32 {
        let pos = self.pos;
        self.pos = self.pos.saturating_add(1);
        let Some((index, part, gate)) = self.part_at(pos) else {
            return 0.0;
        };
        if index == 1
            && pos == part.start
            && let Some((increments, amplitudes)) = self.next.take()
        {
            self.increments = increments;
            self.amplitudes = amplitudes;
            self.phases = [0.0; 3];
        }
        let mut sample = 0.0_f32;
        if part.tone {
            let mut sum = 0.0;
            for ((phase, increment), amplitude) in self
                .phases
                .iter_mut()
                .zip(self.increments.iter_mut())
                .zip(self.amplitudes)
            {
                sum += amplitude * sine_turns(*phase);
                *phase += *increment;
                if *phase >= 1.0 {
                    *phase -= 1.0;
                }
                *increment *= part.glide;
            }
            sample = to_f32(sum);
        } else if let Some((state, filter, gain)) = &mut self.noise {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            // The top 24 bits as a uniform value in [−1, 1).
            #[expect(clippy::cast_precision_loss, reason = "24 bits fit an f32 exactly")]
            let uniform = (*state >> 40) as f32 / 8_388_608.0 - 1.0;
            sample = *gain * filter.process(uniform);
        }
        if let Some(pulse) = &mut self.pulse {
            sample *= pulse.next_gain();
        }
        sample * gate
    }
}

#[expect(clippy::cast_possible_truncation, reason = "samples and gains are f32")]
#[inline]
fn to_f32(x: f64) -> f32 {
    x as f32
}

/// A pulse gate in frames: on for `on` of every `period`, with raised-cosine edges of the
/// minimum ramp, so pulses never step (SDR-22).
#[derive(Clone, Copy, Debug)]
struct Pulse {
    on: u32,
    period: u32,
    edge: u32,
    pos: u32,
}

impl Pulse {
    fn next_gain(&mut self) -> f32 {
        let (pos, on, edge) = (self.pos, self.on, self.edge.max(1));
        let level = if pos >= on {
            0.0
        } else if pos < edge {
            f64::from(pos) / f64::from(edge)
        } else if on - pos < edge {
            f64::from(on - pos) / f64::from(edge)
        } else {
            1.0
        };
        self.pos = (pos + 1) % self.period.max(1);
        let s = sine_turns(level / 4.0);
        to_f32(s * s)
    }
}

/// `SplitMix64`: spreads a seed over 64 bits.
pub(crate) fn splitmix(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
