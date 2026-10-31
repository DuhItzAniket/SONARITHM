// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Resampling for audification (RFC-0007 §3, SDR-31): a series played at its own `rate` is
//! resampled to the output rate at plan time, on a worker thread, so audification changes
//! speed by resampling, never by pitch shifting.
//!
//! A windowed-sinc interpolator (Blackman window), computed with `libm` and IEEE
//! arithmetic only, so a plan is bit-identical on every platform (ADR-0013, F-039). RFC-0007
//! named rubato; its FFT path runs through rustfft's per-CPU SIMD code, whose rounding can
//! differ between x86 and ARM, which golden audio's cross-platform hashes do not allow
//! (F-195). Downsampling widens the kernel to low-pass at the output's Nyquist frequency.

use sonarithm_core::math;

/// Taps on each side of the kernel at unity ratio.
pub const HALF_TAPS: usize = 16;

/// The widest kernel, in taps on each side, when downsampling by a large factor.
pub const MAX_HALF_TAPS: usize = 4096;

/// `input` at `from` samples per second, resampled to `to`: `floor(len · to / from)`
/// samples, each clamped to [−1, 1]. An empty or unusable input gives no samples.
#[must_use]
pub fn resample(input: &[f32], from: f64, to: f64) -> Vec<f32> {
    if input.is_empty() || !(from > 0.0 && to > 0.0 && from.is_finite() && to.is_finite()) {
        return Vec::new();
    }
    let ratio = to / from;
    #[expect(
        clippy::cast_precision_loss,
        reason = "at most 2^22 input samples, exact in f64"
    )]
    let len = input.len() as f64;
    let out_len = (len * ratio).floor();
    if !(out_len >= 1.0 && out_len < 1e9) {
        return Vec::new();
    }
    // Low-pass at the lower of the two Nyquist frequencies, relative to the input's.
    let cutoff = ratio.min(1.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "a tap count between HALF_TAPS and MAX_HALF_TAPS"
    )]
    let half = ((HALF_TAPS as f64 / cutoff).ceil() as usize).clamp(HALF_TAPS, MAX_HALF_TAPS);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "checked above: below 1e9"
    )]
    let count = out_len as usize;
    let mut out = Vec::with_capacity(count);
    #[expect(clippy::cast_precision_loss, reason = "tap counts are small")]
    let width = half as f64;
    for i in 0..count {
        #[expect(clippy::cast_precision_loss, reason = "output indices below 1e9")]
        let position = i as f64 / ratio;
        let centre = position.floor();
        let (mut sum, mut weights) = (0.0_f64, 0.0_f64);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            reason = "input positions below 2^22"
        )]
        let first = centre as i64 - half as i64 + 1;
        for offset in 0..2 * half {
            #[expect(clippy::cast_possible_wrap, reason = "tap counts are small")]
            let n = first + offset as i64;
            let Ok(index) = usize::try_from(n) else {
                continue;
            };
            let Some(&x) = input.get(index) else {
                continue;
            };
            #[expect(clippy::cast_precision_loss, reason = "input positions below 2^22")]
            let distance = position - n as f64;
            let w = cutoff * sinc(cutoff * distance) * blackman(distance / width);
            sum += f64::from(x) * w;
            weights += w;
        }
        let value = if weights.abs() > 1e-12 {
            sum / weights
        } else {
            0.0
        };
        #[expect(clippy::cast_possible_truncation, reason = "samples are f32")]
        out.push(value.clamp(-1.0, 1.0) as f32);
    }
    out
}

/// sin(πx)/(πx), 1 at 0.
fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        let px = std::f64::consts::PI * x;
        math::sin(px) / px
    }
}

/// The Blackman window on [−1, 1], 0 outside.
fn blackman(x: f64) -> f64 {
    if x.abs() >= 1.0 {
        return 0.0;
    }
    let a = std::f64::consts::PI * x;
    0.42 + 0.5 * math::cos(a) + 0.08 * math::cos(2.0 * a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frequency: f64, rate: f64, count: usize) -> Vec<f32> {
        (0..count)
            .map(|i| {
                #[expect(clippy::cast_precision_loss, reason = "test sizes")]
                let t = i as f64 / rate;
                #[expect(clippy::cast_possible_truncation, reason = "samples are f32")]
                let s = (0.5 * math::sin(std::f64::consts::TAU * frequency * t)) as f32;
                s
            })
            .collect()
    }

    /// Zero crossings per second, from rising crossings.
    fn frequency(samples: &[f32], rate: f64) -> f64 {
        let rising = samples
            .windows(2)
            .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
            .count();
        #[expect(clippy::cast_precision_loss, reason = "test sizes")]
        let seconds = samples.len() as f64 / rate;
        #[expect(clippy::cast_precision_loss, reason = "test sizes")]
        let per_second = rising as f64 / seconds;
        per_second
    }

    /// SDR-31: resampling keeps the waveform; a tone keeps its frequency in seconds of
    /// output when the rate changes.
    #[test]
    fn sdr_31_resampling_keeps_the_tone() {
        // One second of input at each rate.
        for (from, to, count) in [
            (8_000.0, 48_000.0, 8_000),
            (96_000.0, 48_000.0, 96_000),
            (44_100.0, 48_000.0, 44_100),
        ] {
            let input = sine(440.0, from, count);
            let output = resample(&input, from, to);
            #[expect(clippy::cast_precision_loss, reason = "test sizes")]
            let expected = (input.len() as f64 * to / from).floor();
            #[expect(clippy::cast_precision_loss, reason = "test sizes")]
            let got = output.len() as f64;
            assert!(
                (got - expected).abs() < 1.0,
                "{from} -> {to}: {got} vs {expected}"
            );
            let f = frequency(&output, to);
            assert!((f - 440.0).abs() < 5.0, "{from} -> {to}: {f} Hz");
        }
    }

    #[test]
    fn downsampling_removes_what_the_output_cannot_hold() {
        // 20 kHz at 96 kHz, down to 16 kHz (Nyquist 8 kHz): filtered out.
        let input = sine(20_000.0, 96_000.0, 9_600);
        let output = resample(&input, 96_000.0, 16_000.0);
        let middle = &output[200..output.len() - 200];
        let peak = middle.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
        assert!(peak < 0.05, "{peak}");
    }

    #[test]
    fn unusable_inputs_give_nothing() {
        assert!(resample(&[], 1.0, 2.0).is_empty());
        assert!(resample(&[0.5], 0.0, 2.0).is_empty());
        assert!(resample(&[0.5], f64::NAN, 2.0).is_empty());
        assert!(resample(&[0.5; 10], 48_000.0, 1.0).is_empty());
    }

    #[test]
    fn resampling_is_deterministic() {
        let input = sine(123.0, 7_000.0, 3_000);
        assert_eq!(
            resample(&input, 7_000.0, 48_000.0),
            resample(&input, 7_000.0, 48_000.0)
        );
    }
}
