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

use crate::filter::Biquad;

/// The largest phase increment, in turns per sample: just under the Nyquist frequency.
/// PolyBLEP correction needs an increment below 0.5.
const MAX_INCREMENT: f64 = 0.499;

/// Samples per chunk when [`Oscillator::fill`] computes phases ahead of the waveform, so the
/// waveform arithmetic can run on several samples per instruction.
const CHUNK: usize = 64;

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
    sine_folded(turns - turns.round())
}

/// [`sine_turns`] for `turns` in [0, 1.5): the same result, bit for bit, with a cheaper
/// reduction. Oscillator phases (in [0, 1), or [0.25, 1.25) for a quarter-turn offset) and
/// envelope ramps (in [0, 1/4]) are in this range. The renderer uses it on its hot paths
/// (PB-03, #185).
///
/// Outside [0, 1.5) the result is not the sine; debug builds check the range. NaN gives
/// NaN.
///
/// # Real-time safety
///
/// RT-safe: arithmetic only.
#[must_use]
#[inline]
pub fn sine_turns_unit(turns: f64) -> f64 {
    debug_assert!(
        !(turns < 0.0 || turns >= 1.5),
        "sine_turns_unit needs turns in [0, 1.5), got {turns}"
    );
    // `turns.round()` is 0 below 1/2 and 1 from 1/2 (halves round away from zero), so the
    // reduction is one comparison. `+ 0.0` turns −0.0 into +0.0, as `−0.0 − round(−0.0)`
    // does.
    sine_folded(if turns >= 0.5 {
        turns - 1.0
    } else {
        turns + 0.0
    })
}

/// The sine of `r` turns for `r` in [−1/2, 1/2]: folds onto [−1/4, 1/4], then evaluates
/// the polynomial.
#[inline]
fn sine_folded(r: f64) -> f64 {
    let r = if r > 0.25 {
        0.5 - r
    } else if r < -0.25 {
        -0.5 - r
    } else {
        r
    };
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
        let value = match self.waveform {
            Waveform::Sine => sine(self.phase, self.increment),
            Waveform::Triangle => triangle(self.phase, self.increment),
            Waveform::Saw => saw(self.phase, self.increment),
            Waveform::Square => square(self.phase, self.increment),
        };
        self.phase = advance(self.phase, self.increment);
        to_sample(value)
    }

    /// Writes the next `out.len()` samples into `out`: the same samples as calling
    /// [`Oscillator::next_sample`] for each.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: bounded by `out.len()`; a fixed buffer on the stack.
    pub fn fill(&mut self, out: &mut [f32]) {
        match self.waveform {
            Waveform::Sine => self.fill_with(out, sine),
            Waveform::Triangle => self.fill_with(out, triangle),
            Waveform::Saw => self.fill_with(out, saw),
            Waveform::Square => self.fill_with(out, square),
        }
    }

    /// Writes the next `out.len()` samples through `filter`: the same result as
    /// [`Oscillator::fill`] followed by [`Biquad::process_buffer`], in one pass, so the
    /// waveform arithmetic overlaps the filter's chain of dependent operations (PB-03).
    ///
    /// # Real-time safety
    ///
    /// RT-safe: bounded by `out.len()`.
    pub fn fill_through(&mut self, out: &mut [f32], filter: &mut Biquad) {
        match self.waveform {
            Waveform::Sine => self.fill_through_with(out, filter, sine),
            Waveform::Triangle => self.fill_through_with(out, filter, triangle),
            Waveform::Saw => self.fill_through_with(out, filter, saw),
            Waveform::Square => self.fill_through_with(out, filter, square),
        }
    }

    /// [`Oscillator::fill`] for one waveform: the phases of a chunk first, then the
    /// waveform for every phase, which the compiler can vectorize.
    #[inline]
    fn fill_with(&mut self, out: &mut [f32], wave: fn(f64, f64) -> f64) {
        let dt = self.increment;
        let mut phase = self.phase;
        let mut phases = [0.0; CHUNK];
        for chunk in out.chunks_mut(CHUNK) {
            for slot in phases.iter_mut().take(chunk.len()) {
                *slot = phase;
                phase = advance(phase, dt);
            }
            for (sample, &p) in chunk.iter_mut().zip(phases.iter()) {
                *sample = to_sample(wave(p, dt));
            }
        }
        self.phase = phase;
    }

    #[inline]
    fn fill_through_with(
        &mut self,
        out: &mut [f32],
        filter: &mut Biquad,
        wave: fn(f64, f64) -> f64,
    ) {
        let dt = self.increment;
        let mut phase = self.phase;
        for sample in out {
            let x = to_sample(wave(phase, dt));
            phase = advance(phase, dt);
            *sample = filter.process(x);
        }
        self.phase = phase;
    }
}

// The waveforms, for a phase `p` in [0, 1) and an increment `dt` in [0, 0.499]: the
// oscillator keeps its phase in that range, so the reductions below are exact.

/// The phase after one sample.
#[inline]
fn advance(p: f64, dt: f64) -> f64 {
    let next = p + dt;
    if next >= 1.0 { next - 1.0 } else { next }
}

#[inline]
fn sine(p: f64, _dt: f64) -> f64 {
    sine_turns_unit(p)
}

/// Rises from 0 at p = 0 to 1 at p = 1/4, like the sine.
#[inline]
fn triangle(p: f64, _dt: f64) -> f64 {
    // `shifted` is in [1/4, 5/4], so its floor is 0 or 1.
    let shifted = p + 0.25;
    let wrapped = if shifted >= 1.0 {
        shifted - 1.0
    } else {
        shifted
    };
    1.0 - 4.0 * (wrapped - 0.5).abs()
}

#[inline]
fn saw(p: f64, dt: f64) -> f64 {
    2.0 * p - 1.0 - poly_blep(p, dt)
}

#[inline]
fn square(p: f64, dt: f64) -> f64 {
    let naive = if p < 0.5 { 1.0 } else { -1.0 };
    let half = if p < 0.5 { p + 0.5 } else { p - 0.5 };
    naive + poly_blep(p, dt) - poly_blep(half, dt)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "samples are f32 (coding-standards §10); the value is in [−1.1, 1.1]"
)]
#[inline]
fn to_sample(value: f64) -> f32 {
    value as f32
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

    /// The reference: `sine_turns` with the general reduction, as before PB-03's fast path.
    fn reference_sine(turns: f64) -> f64 {
        sine_turns(turns)
    }

    /// PB-03 (#185): the fast reduction gives the same bits as the general one everywhere in
    /// its range, including the halves where `round` breaks ties and the fold's edges.
    #[test]
    fn sine_turns_unit_matches_sine_turns_bit_for_bit() {
        let mut inputs = vec![
            0.0,
            -0.0,
            0.25,
            0.5,
            0.75,
            1.0,
            1.25,
            1.499_999_999_999_999_8,
        ];
        for edge in [0.25_f64, 0.5, 0.75, 1.0, 1.25] {
            inputs.push(f64::from_bits(edge.to_bits() - 1));
            inputs.push(f64::from_bits(edge.to_bits() + 1));
        }
        inputs.extend((0..1_500_000).map(|i| f64::from(i) / 1_000_000.0));
        inputs.extend((0..100_000).map(|i| f64::from(i) * 0.000_014_999_7));
        for turns in inputs {
            assert_eq!(
                sine_turns_unit(turns).to_bits(),
                reference_sine(turns).to_bits(),
                "turns {turns}"
            );
        }
        assert!(sine_turns_unit(f64::NAN).is_nan());
    }

    /// The per-sample oscillator as it was before PB-03, for comparison.
    fn reference_samples(waveform: Waveform, hz: f64, frames: usize) -> Vec<f32> {
        let dt = (hz / 48_000.0).min(MAX_INCREMENT);
        let mut p = 0.0_f64;
        (0..frames)
            .map(|_| {
                let value = match waveform {
                    Waveform::Sine => reference_sine(p),
                    Waveform::Triangle => {
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
                p += dt;
                if p >= 1.0 {
                    p -= 1.0;
                }
                value as f32
            })
            .collect()
    }

    const WAVEFORMS: [Waveform; 4] = [
        Waveform::Sine,
        Waveform::Triangle,
        Waveform::Saw,
        Waveform::Square,
    ];

    fn bits(samples: &[f32]) -> Vec<u32> {
        samples.iter().map(|s| s.to_bits()).collect()
    }

    /// PB-03 (#185): `fill`, `fill_through` and `next_sample` give the samples of the
    /// per-sample oscillator before the speed work, whatever the buffer sizes.
    #[test]
    fn fills_match_the_per_sample_oscillator_bit_for_bit() {
        for waveform in WAVEFORMS {
            for hz in [0.0, 61.7, 437.3, 1_480.0, 9_999.9, 23_000.0] {
                let frames = 5_000;
                let reference = reference_samples(waveform, hz, frames);
                let mut osc = Oscillator::new(waveform);
                osc.set_frequency(Hz(hz), RATE);
                let mut filled = vec![0.0; frames];
                for chunk in filled.chunks_mut(97) {
                    osc.fill(chunk);
                }
                assert_eq!(bits(&filled), bits(&reference), "fill {waveform:?} {hz}");
                let mut osc = Oscillator::new(waveform);
                osc.set_frequency(Hz(hz), RATE);
                let single: Vec<f32> = (0..frames).map(|_| osc.next_sample()).collect();
                assert_eq!(bits(&single), bits(&reference), "next {waveform:?} {hz}");

                let filter = Biquad::new(
                    crate::BiquadKind::LowPass,
                    Hz(2_000.0),
                    crate::filter::BUTTERWORTH_Q,
                    RATE,
                );
                let mut expected = reference.clone();
                filter.clone().process_buffer(&mut expected);
                let mut osc = Oscillator::new(waveform);
                osc.set_frequency(Hz(hz), RATE);
                let mut through = vec![0.0; frames];
                let mut fused = filter;
                for chunk in through.chunks_mut(61) {
                    osc.fill_through(chunk, &mut fused);
                }
                assert_eq!(bits(&through), bits(&expected), "through {waveform:?} {hz}");
            }
        }
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
