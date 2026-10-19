// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Filters: a one-pole low-pass and second-order sections after the RBJ Audio EQ Cookbook
//! (06 §3: brightness, and the comfort low-pass of the square preset, sound-design §11).
//!
//! Coefficients are computed with `libm` when a cutoff changes, which is RT-safe (pure
//! arithmetic) and identical on every platform (ADR-0013). Per-sample processing is
//! arithmetic only. The state is `f64` and is flushed to zero below [`DENORMAL_FLOOR`], so a
//! decaying tail never reaches subnormal numbers, whatever the CPU's FTZ setting
//! (coding-standards §11.4).

use sonarithm_core::{Hz, SampleRate, math};

/// Filter state below this magnitude is set to zero (about −400 dBFS).
pub const DENORMAL_FLOOR: f64 = 1e-20;

/// The quality factor of a Butterworth second-order section, `1/√2`.
pub const BUTTERWORTH_Q: f64 = core::f64::consts::FRAC_1_SQRT_2;

/// The cutoff limited to (1 Hz, 0.49 · rate); NaN becomes 1 kHz. Returns the cutoff as a
/// fraction of the sample rate.
fn normalized_cutoff(cutoff: Hz, rate: SampleRate) -> f64 {
    let sr = f64::from(rate.get());
    let hz = if cutoff.0.is_nan() { 1_000.0 } else { cutoff.0 };
    hz.clamp(1.0, 0.49 * sr) / sr
}

#[inline]
fn flush(x: f64) -> f64 {
    if x.abs() < DENORMAL_FLOOR { 0.0 } else { x }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "samples are f32 (coding-standards §10)"
)]
#[inline]
fn to_sample(x: f64) -> f32 {
    x as f32
}

/// A one-pole low-pass: `y += a · (x − y)`, with `a = 1 − e^(−2π fc / fs)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnePole {
    coefficient: f64,
    state: f64,
}

impl OnePole {
    /// A low-pass at `cutoff`, with silent state.
    #[must_use]
    pub fn low_pass(cutoff: Hz, rate: SampleRate) -> Self {
        let mut filter = Self {
            coefficient: 1.0,
            state: 0.0,
        };
        filter.set_cutoff(cutoff, rate);
        filter
    }

    /// Changes the cutoff and keeps the state.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic and `libm` only.
    pub fn set_cutoff(&mut self, cutoff: Hz, rate: SampleRate) {
        let x = normalized_cutoff(cutoff, rate);
        self.coefficient = 1.0 - math::exp(-core::f64::consts::TAU * x);
    }

    /// Clears the state.
    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    /// Filters one sample.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic only.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.state = flush(self.state + self.coefficient * (f64::from(x) - self.state));
        to_sample(self.state)
    }

    /// Filters `buffer` in place.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: bounded by `buffer.len()`.
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer {
            *sample = self.process(*sample);
        }
    }
}

/// The response of a [`Biquad`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BiquadKind {
    /// Low-pass, 12 dB per octave.
    LowPass,
    /// High-pass, 12 dB per octave.
    HighPass,
    /// Band-pass with 0 dB gain at the centre frequency.
    BandPass,
}

/// A second-order section in transposed direct form II, with RBJ cookbook coefficients.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Biquad {
    kind: BiquadKind,
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl Biquad {
    /// A section of `kind` at `cutoff` with quality factor `q`, with silent state.
    #[must_use]
    pub fn new(kind: BiquadKind, cutoff: Hz, q: f64, rate: SampleRate) -> Self {
        let mut filter = Self {
            kind,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        };
        filter.set(cutoff, q, rate);
        filter
    }

    /// Changes the cutoff (or centre frequency) and `q`, and keeps the state. `q` is
    /// limited to [0.1, 100]; NaN becomes [`BUTTERWORTH_Q`].
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic and `libm` only.
    pub fn set(&mut self, cutoff: Hz, q: f64, rate: SampleRate) {
        let w0 = core::f64::consts::TAU * normalized_cutoff(cutoff, rate);
        let q = if q.is_nan() {
            BUTTERWORTH_Q
        } else {
            q.clamp(0.1, 100.0)
        };
        let (sin, cos) = (math::sin(w0), math::cos(w0));
        let alpha = sin / (2.0 * q);
        let (b0, b1, b2) = match self.kind {
            BiquadKind::LowPass => ((1.0 - cos) / 2.0, 1.0 - cos, (1.0 - cos) / 2.0),
            BiquadKind::HighPass => (
                f64::midpoint(1.0, cos),
                -(1.0 + cos),
                f64::midpoint(1.0, cos),
            ),
            BiquadKind::BandPass => (alpha, 0.0, -alpha),
        };
        let a0 = 1.0 + alpha;
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = -2.0 * cos / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Clears the state.
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    /// Filters one sample.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic only.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let x = f64::from(x);
        let y = self.b0 * x + self.z1;
        self.z1 = flush(self.b1 * x - self.a1 * y + self.z2);
        self.z2 = flush(self.b2 * x - self.a2 * y);
        to_sample(y)
    }

    /// Filters `buffer` in place.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: bounded by `buffer.len()`.
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer {
            *sample = self.process(*sample);
        }
    }

    /// The magnitude response at `frequency`, in dB, from the coefficients.
    #[must_use]
    pub fn response_db(&self, frequency: Hz, rate: SampleRate) -> f64 {
        let w = core::f64::consts::TAU * frequency.0 / f64::from(rate.get());
        let (c1, s1, c2, s2) = (
            math::cos(w),
            math::sin(w),
            math::cos(2.0 * w),
            math::sin(2.0 * w),
        );
        let num = (
            self.b0 + self.b1 * c1 + self.b2 * c2,
            -(self.b1 * s1 + self.b2 * s2),
        );
        let den = (
            1.0 + self.a1 * c1 + self.a2 * c2,
            -(self.a1 * s1 + self.a2 * s2),
        );
        let power = (num.0 * num.0 + num.1 * num.1) / (den.0 * den.0 + den.1 * den.1);
        10.0 * math::log10(power)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: SampleRate = SampleRate::HZ_48000;

    #[test]
    fn a_butterworth_low_pass_has_the_textbook_response() {
        let lp = Biquad::new(BiquadKind::LowPass, Hz(1_000.0), BUTTERWORTH_Q, RATE);
        assert!(lp.response_db(Hz(10.0), RATE).abs() < 0.01);
        assert!((lp.response_db(Hz(1_000.0), RATE) + 3.01).abs() < 0.01);
        // The bilinear transform adds attenuation above the analogue −40 dB per decade.
        assert!(lp.response_db(Hz(10_000.0), RATE) < -40.0);
    }

    #[test]
    fn a_high_pass_blocks_dc_and_a_band_pass_peaks_at_0_db() {
        let mut hp = Biquad::new(BiquadKind::HighPass, Hz(200.0), BUTTERWORTH_Q, RATE);
        let mut last = 1.0;
        for _ in 0..48_000 {
            last = hp.process(1.0);
        }
        assert!(last.abs() < 1e-6);
        let bp = Biquad::new(BiquadKind::BandPass, Hz(2_000.0), 4.0, RATE);
        assert!(bp.response_db(Hz(2_000.0), RATE).abs() < 1e-9);
        assert!(bp.response_db(Hz(200.0), RATE) < -20.0);
    }

    #[test]
    fn the_time_domain_matches_the_response() {
        let mut lp = Biquad::new(BiquadKind::LowPass, Hz(1_000.0), BUTTERWORTH_Q, RATE);
        let expected = lp.response_db(Hz(3_000.0), RATE);
        let mut peak = 0.0_f32;
        for i in 0..48_000 {
            let x = crate::osc::sine_turns(3_000.0 * f64::from(i) / 48_000.0);
            let y = lp.process(to_sample(x));
            if i > 24_000 {
                peak = peak.max(y.abs());
            }
        }
        let measured = 20.0 * math::log10(f64::from(peak));
        assert!(
            (measured - expected).abs() < 0.05,
            "{measured} vs {expected}"
        );
    }

    /// coding-standards §11.4: a decaying tail reaches exact zero, never a subnormal, and
    /// never NaN.
    #[test]
    fn silent_input_decays_to_exact_zero_without_subnormals() {
        let mut biquad = Biquad::new(BiquadKind::LowPass, Hz(50.0), 10.0, RATE);
        let mut one_pole = OnePole::low_pass(Hz(20.0), RATE);
        let (mut b, mut o) = (biquad.process(1.0), one_pole.process(1.0));
        for _ in 0..20 * 48_000 {
            b = biquad.process(0.0);
            o = one_pole.process(0.0);
            assert!(b == 0.0 || b.is_normal(), "{b}");
            assert!(o == 0.0 || o.is_normal(), "{o}");
        }
        assert_eq!((b, o), (0.0, 0.0));
    }

    #[test]
    fn a_one_pole_passes_dc_and_attenuates_high_frequencies() {
        let mut lp = OnePole::low_pass(Hz(500.0), RATE);
        let mut dc = 0.0;
        for _ in 0..48_000 {
            dc = lp.process(1.0);
        }
        assert!((dc - 1.0).abs() < 1e-6);
        lp.reset();
        let mut buffer: Vec<f32> = (0..48_000)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        lp.process_buffer(&mut buffer);
        assert!(
            buffer[24_000..].iter().all(|y| y.abs() < 0.07),
            "Nyquist leaks"
        );
    }

    #[test]
    fn out_of_range_settings_are_limited() {
        for (hz, q) in [(f64::NAN, f64::NAN), (-1.0, 0.0), (1e9, 1e9)] {
            let mut lp = Biquad::new(BiquadKind::LowPass, Hz(hz), q, RATE);
            let y: Vec<f32> = (0..4_800)
                .map(|i| lp.process(if i == 0 { 1.0 } else { 0.0 }))
                .collect();
            assert!(y.iter().all(|s| s.is_finite()), "{hz} Hz, q {q}");
        }
    }
}
