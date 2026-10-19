// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Earcon synthesis (keyboard-and-navigation §11; sound-design §8).
//!
//! **Provisional.** These follow the catalogue's acoustic descriptions closely enough to be
//! distinguishable, but they are A05 placeholders: AUDAXIS designs and tests the earcon set
//! with listeners in B03 (sound-design §15 proposal process). Noise is seeded (SDR-02), so
//! every render of the same Score sounds the same.

use sonarithm_core::{EarconKind, Hz, SampleRate, Seconds};
use sonarithm_dsp::env::MIN_RAMP;
use sonarithm_dsp::{AdsrParams, Biquad, BiquadKind, sine_turns};

/// Up to three sine partials: frequency in Hz and amplitude.
type Partials = [(f64, f64); 3];

/// E-LANDMARK: a bell-like inharmonic ping (ratios 1, 2.76, 5.40).
const LANDMARK: Partials = [(880.0, 0.6), (2_428.8, 0.3), (4_752.0, 0.1)];
/// E-THRESHOLD: a mid-register wood-block tock.
const THRESHOLD: Partials = [(1_000.0, 0.8), (2_500.0, 0.2), (0.0, 0.0)];
/// E-STREAM: a soft chime.
const STREAM: Partials = [(1_320.0, 0.7), (2_640.0, 0.2), (0.0, 0.0)];
/// E-ALARM: a pulsed tone.
const ALARM: Partials = [(1_000.0, 0.7), (1_500.0, 0.2), (0.0, 0.0)];
/// E-BOUNDARY: noise band-passed around 250 Hz, scaled to a level near the tonal earcons.
const BOUNDARY_CENTRE: Hz = Hz(250.0);
const BOUNDARY_GAIN: f32 = 6.0;
/// E-ALARM pulses: 100 ms on, 100 ms off.
const ALARM_PULSE: Seconds = Seconds(0.100);

/// The envelope of an earcon: the minimum onset ramp (SDR-22), then an exponential decay.
pub(crate) fn envelope(kind: EarconKind) -> AdsrParams {
    let (decay, sustain) = match kind {
        EarconKind::Boundary => (0.015, 0.0),
        EarconKind::Landmark => (0.040, 0.0),
        EarconKind::Threshold => (0.008, 0.0),
        EarconKind::Alarm => (1.0, 1.0),
        _ => (0.060, 0.0),
    };
    AdsrParams {
        attack: MIN_RAMP,
        decay: Seconds(decay),
        sustain,
        release: MIN_RAMP,
    }
}

/// An earcon generator. RT-safe after construction.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Earcon {
    phases: [f64; 3],
    increments: [f64; 3],
    amplitudes: [f64; 3],
    /// Noise source and its band-pass, for E-BOUNDARY.
    noise: Option<(u64, Biquad)>,
    /// The E-ALARM pulse gate.
    pulse: Option<Pulse>,
}

impl Earcon {
    pub(crate) fn new(kind: EarconKind, rate: SampleRate, seed: u64) -> Self {
        let sr = f64::from(rate.get());
        let partials = match kind {
            EarconKind::Boundary => [(0.0, 0.0); 3],
            EarconKind::Landmark => LANDMARK,
            EarconKind::Threshold => THRESHOLD,
            EarconKind::Alarm => ALARM,
            _ => STREAM,
        };
        let on = sonarithm_dsp::env::ramp_frames(ALARM_PULSE, rate);
        Self {
            phases: [0.0; 3],
            increments: partials.map(|(hz, _)| if hz < 0.45 * sr { hz / sr } else { 0.0 }),
            amplitudes: partials.map(|(_, amplitude)| amplitude),
            noise: (kind == EarconKind::Boundary).then(|| {
                // xorshift needs a non-zero state.
                let state = splitmix(seed) | 1;
                (
                    state,
                    Biquad::new(BiquadKind::BandPass, BOUNDARY_CENTRE, 1.5, rate),
                )
            }),
            pulse: (kind == EarconKind::Alarm).then(|| Pulse {
                on,
                period: 2 * on,
                edge: sonarithm_dsp::env::ramp_frames(MIN_RAMP, rate),
                pos: 0,
            }),
        }
    }

    #[inline]
    pub(crate) fn next_sample(&mut self) -> f32 {
        let mut sum = 0.0;
        for ((phase, increment), amplitude) in self
            .phases
            .iter_mut()
            .zip(self.increments)
            .zip(self.amplitudes)
        {
            sum += amplitude * sine_turns(*phase);
            *phase += increment;
            if *phase >= 1.0 {
                *phase -= 1.0;
            }
        }
        #[expect(clippy::cast_possible_truncation, reason = "samples are f32")]
        let mut sample = sum as f32;
        if let Some((state, filter)) = &mut self.noise {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            // The top 24 bits as a uniform value in [−1, 1).
            #[expect(clippy::cast_precision_loss, reason = "24 bits fit an f32 exactly")]
            let uniform = (*state >> 40) as f32 / 8_388_608.0 - 1.0;
            sample += BOUNDARY_GAIN * filter.process(uniform);
        }
        if let Some(pulse) = &mut self.pulse {
            sample *= pulse.next_gain();
        }
        sample
    }
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
        #[expect(clippy::cast_possible_truncation, reason = "gains are f32")]
        let gain = (s * s) as f32;
        gain
    }
}

/// `SplitMix64`: spreads a seed over 64 bits.
pub(crate) fn splitmix(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
