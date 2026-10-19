// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Oscillators: sine, triangle and band-limited (PolyBLEP) saw and square (06 §3).
//!
//! Every sample is computed with IEEE 754 arithmetic only: no call into the platform maths
//! library. A02 found that offline renders are bit-identical across Windows, Linux and
//! macOS only under that condition (F-039, ADR-0013), so the sine is a polynomial
//! ([`sine_turns`]) rather than `f32::sin`. The phase is an `f64` in turns (cycles), so
//! long notes do not drift; samples are `f32`.

use sonarithm_core::{Hz, SampleRate};

/// The largest phase increment, in turns per sample: just under the Nyquist frequency.
/// PolyBLEP correction needs an increment below 0.5.
const MAX_INCREMENT: f64 = 0.499;

/// Taylor coefficients of `sin(2πr)` in powers of `r`: `(−1)^k (2π)^(2k+1) / (2k+1)!`,
/// for `k = 0..8`. On `|r| ≤ 1/4` the truncation error is below 1e-11.
const SINE_COEFFICIENTS: [f64; 8] = sine_coefficients();

const fn sine_coefficients() -> [f64; 8] {
    let tau = core::f64::consts::TAU;
    let mut coefficients = [0.0; 8];
    let mut term = tau; // (2π)^(2k+1) / (2k+1)!, with its sign
    let mut k = 0;
    while k < 8 {
        coefficients[k] = term;
        let n = (2 * k + 2) as f64;
        term = -term * tau * tau / (n * (n + 1.0));
        k += 1;
    }
    coefficients
}

/// `sin(2π · turns)`: the sine of a phase given in turns, for any finite input.
///
/// A polynomial in IEEE 754 arithmetic, so the result is the same on every platform
/// (F-039). The absolute error against the exact sine is below 1e-10. Non-finite input
/// gives NaN.
///
/// # Real-time safety
///
/// RT-safe: arithmetic only.
#[must_use]
#[inline]
pub fn sine_turns(turns: f64) -> f64 {
    // Reduce to [−1/2, 1/2], then fold onto [−1/4, 1/4] with sin(π − x) = sin(x).
    let mut r = turns - turns.round();
    if r > 0.25 {
        r = 0.5 - r;
    } else if r < -0.25 {
        r = -0.5 - r;
    }
    let r2 = r * r;
    let mut sum = 0.0;
    let mut k = SINE_COEFFICIENTS.len();
    while k > 0 {
        k -= 1;
        sum = sum * r2 + SINE_COEFFICIENTS[k];
    }
    sum * r
}

/// An oscillator waveform. The square wave is the "discrete square" preset for BLV chart
/// reading (SDR-29).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Waveform {
    /// A pure sine (SDR-29: an option for the plainest tone).
    #[default]
    Sine,
    /// A triangle. Its harmonics fall at 12 dB per octave, so it is not band-limited.
    Triangle,
    /// A rising sawtooth, band-limited with PolyBLEP.
    Saw,
    /// A square wave with a 50% duty cycle, band-limited with PolyBLEP.
    Square,
}

/// A phase-accumulating oscillator. Starts at phase 0, where every waveform rises from 0
/// except the square wave, which steps to +1 (the envelope's onset ramp removes the step,
/// SDR-22).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oscillator {
    waveform: Waveform,
    /// Phase in turns, in [0, 1).
    phase: f64,
    /// Phase increment in turns per sample, in [0, 0.499].
    increment: f64,
}

impl Oscillator {
    /// A silent oscillator (frequency 0) of `waveform` at phase 0.
    #[must_use]
    pub fn new(waveform: Waveform) -> Self {
        Self {
            waveform,
            phase: 0.0,
            increment: 0.0,
        }
    }

    /// The waveform.
    #[must_use]
    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    /// Sets the frequency. Negative, non-finite and zero frequencies give silence; a
    /// frequency at or above Nyquist is limited to just below it.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic only.
    pub fn set_frequency(&mut self, frequency: Hz, rate: SampleRate) {
        let increment = frequency.0 / f64::from(rate.get());
        self.increment = if increment.is_finite() && increment > 0.0 {
            increment.min(MAX_INCREMENT)
        } else {
            0.0
        };
    }

    /// Restarts the waveform at phase 0.
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// The next sample, in [−1, 1] up to the PolyBLEP overshoot.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic only.
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let p = self.phase;
        let dt = self.increment;
        let value = match self.waveform {
            Waveform::Sine => sine_turns(p),
            Waveform::Triangle => {
                // Rises from 0 at p = 0 to 1 at p = 1/4, like the sine.
                let shifted = p + 0.25;
                let shifted = shifted - shifted.floor();
                1.0 - 4.0 * (shifted - 0.5).abs()
            }
            Waveform::Saw => 2.0 * p - 1.0 - poly_blep(p, dt),
            Waveform::Square => {
                let naive = if p < 0.5 { 1.0 } else { -1.0 };
                let half = if p < 0.5 { p + 0.5 } else { p - 0.5 };
                naive + poly_blep(p, dt) - poly_blep(half, dt)
            }
        };
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "samples are f32 (coding-standards §10); the value is in [−1.1, 1.1]"
        )]
        let sample = value as f32;
        sample
    }

    /// Writes the next `out.len()` samples into `out`.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: bounded by `out.len()`.
    pub fn fill(&mut self, out: &mut [f32]) {
        for sample in out {
            *sample = self.next_sample();
        }
    }
}

/// The PolyBLEP residual for a unit step at phase 0, with phase `t` and increment `dt` in
/// turns (Välimäki and Huovilainen 2007). Zero away from the discontinuity.
#[inline]
fn poly_blep(t: f64, dt: f64) -> f64 {
    if dt <= 0.0 {
        0.0
    } else if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

#[cfg(test)]
#[allow(
    clippy::cast_possible_truncation,
    reason = "test signals are computed in f64 and stored as f32 samples"
)]
mod tests {
    use super::*;
    use crate::testing::{magnitude_db, spectrum, windowed_spectrum};

    const RATE: SampleRate = SampleRate::HZ_48000;

    fn render(waveform: Waveform, hz: f64, frames: usize) -> Vec<f32> {
        let mut osc = Oscillator::new(waveform);
        osc.set_frequency(Hz(hz), RATE);
        let mut out = vec![0.0; frames];
        osc.fill(&mut out);
        out
    }

    #[test]
    fn sine_turns_matches_libm_within_1e_10() {
        for i in -4_000..=4_000 {
            let turns = f64::from(i) / 997.0;
            let exact = libm::sin(core::f64::consts::TAU * turns);
            let error = (sine_turns(turns) - exact).abs();
            assert!(error < 1e-10, "turns {turns}: error {error}");
        }
        assert_eq!(sine_turns(0.0), 0.0);
        assert!((sine_turns(0.25) - 1.0).abs() < 1e-11);
        assert!(sine_turns(f64::NAN).is_nan());
    }

    /// The bit patterns below were produced on Windows x86-64. The polynomial uses only
    /// IEEE 754 arithmetic, so every CI OS must reproduce them exactly (F-039, ADR-0013).
    #[test]
    fn sine_turns_is_bit_identical_across_platforms() {
        assert_eq!(sine_turns(0.1).to_bits(), 0x3fe2_cf23_0475_5a5e);
        assert_eq!(sine_turns(0.37).to_bits(), 0x3fe7_53b6_03d2_b815);
        assert_eq!(sine_turns(-1.73).to_bits(), 0x3fef_bf67_5480_a599);
    }

    #[test]
    fn the_sine_oscillator_is_pure() {
        // A frequency centred on a bin, so the rectangular window leaks nothing.
        let bins = 8_192;
        let hz = 48_000.0 / bins as f64 * 171.0; // ≈ 1 001.95 Hz, exactly bin 171
        let signal = render(Waveform::Sine, hz, bins);
        let mags = spectrum(&signal);
        let fundamental = magnitude_db(mags[171]);
        let worst_spur = mags
            .iter()
            .enumerate()
            .filter(|(bin, _)| bin.abs_diff(171) > 2)
            .map(|(_, m)| magnitude_db(*m))
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(fundamental - worst_spur > 120.0, "spur at {worst_spur} dB");
    }

    #[test]
    fn saw_and_square_have_the_ideal_harmonic_amplitudes() {
        let bins = 8_192;
        let hz = 48_000.0 / bins as f64 * 40.0; // ≈ 234 Hz, bin 40
        let saw = spectrum(&render(Waveform::Saw, hz, bins));
        let square = spectrum(&render(Waveform::Square, hz, bins));
        for n in 1..=7_usize {
            let ideal_saw = 2.0 / (core::f64::consts::PI * n as f64);
            let measured = saw[40 * n];
            assert!(
                (measured / ideal_saw - 1.0).abs() < 0.01,
                "saw harmonic {n}"
            );
            if n % 2 == 1 {
                let ideal_square = 4.0 / (core::f64::consts::PI * n as f64);
                assert!(
                    (square[40 * n] / ideal_square - 1.0).abs() < 0.01,
                    "square harmonic {n}"
                );
            } else {
                assert!(
                    magnitude_db(square[40 * n]) < -80.0,
                    "square has even harmonic {n}"
                );
            }
        }
    }

    /// PolyBLEP removes most aliasing: at 1 480 Hz (the top of the expanded SDR-08 range)
    /// the strongest alias of the band-limited saw is at least 15 dB below the naive saw's.
    #[test]
    fn polyblep_suppresses_aliasing() {
        let bins = 16_384;
        let hz = 1_480.0;
        let blep = windowed_spectrum(&render(Waveform::Saw, hz, bins));
        let naive: Vec<f32> = (0..bins)
            .map(|i| {
                let p = (i as f64 * hz / 48_000.0).fract();
                (2.0 * p - 1.0) as f32
            })
            .collect();
        let naive = windowed_spectrum(&naive);
        let bin_hz = 48_000.0 / bins as f64;
        // Aliases fold back between harmonics; look below 10 kHz, away from harmonics.
        let worst_alias = |mags: &[f64]| {
            mags.iter()
                .enumerate()
                .filter(|(bin, _)| {
                    let f = *bin as f64 * bin_hz;
                    let to_harmonic = (f / hz - (f / hz).round()).abs() * hz;
                    f < 10_000.0 && to_harmonic > 60.0
                })
                .map(|(_, m)| magnitude_db(*m))
                .fold(f64::NEG_INFINITY, f64::max)
        };
        assert!(worst_alias(&naive) - worst_alias(&blep) > 15.0);
    }

    #[test]
    fn triangle_rises_like_the_sine() {
        let mut osc = Oscillator::new(Waveform::Triangle);
        osc.set_frequency(Hz(48_000.0 / 8.0), RATE);
        let mut out = [0.0; 8];
        osc.fill(&mut out);
        assert_eq!(out, [0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5]);
    }

    #[test]
    fn invalid_frequencies_are_silent_and_nyquist_is_limited() {
        for hz in [0.0, -5.0, f64::NAN, f64::INFINITY] {
            assert!(
                render(Waveform::Sine, hz, 64).iter().all(|s| *s == 0.0),
                "{hz}"
            );
        }
        let near_nyquist = render(Waveform::Square, 30_000.0, 4_800);
        assert!(near_nyquist.iter().all(|s| s.is_finite() && s.abs() < 1.5));
    }
}
