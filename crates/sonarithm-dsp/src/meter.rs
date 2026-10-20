// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! EBU R128 loudness metering (ITU-R BS.1770) through the `ebur128` crate: integrated
//! loudness, loudness range and true peak for exports, render manifests and the golden
//! tolerance ΔLUFS ≤ 0.1 (06 §7–8, testing-strategy §5.2).
//!
//! **Not for the audio thread:** the meter allocates its history as it measures. Real-time
//! level meters (peak per block) come from the render graph's statistics.

use sonarithm_core::{Db, SampleRate};

/// Why the meter failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MeterError {
    /// The meter rejected the sample rate.
    #[error("the loudness meter does not support {0}")]
    SampleRate(SampleRate),
    /// The meter could not take more audio (out of memory).
    #[error("the loudness meter could not take more audio")]
    Measure,
}

/// The results of a measurement. Loudness values are in LUFS (dB relative to full scale,
/// K-weighted) and true peak in dBTP.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Loudness {
    /// Integrated loudness, or `None` if every gating block was silent.
    pub integrated: Option<Db>,
    /// Loudness range in LU, or `None` if too short to measure.
    pub range: Option<Db>,
    /// True peak over both channels.
    pub true_peak: Db,
}

/// A stereo EBU R128 meter.
pub struct LoudnessMeter {
    inner: ebur128::EbuR128,
}

impl LoudnessMeter {
    /// A meter for interleaved stereo at `rate`.
    ///
    /// # Errors
    ///
    /// [`MeterError::SampleRate`] for rates the meter cannot handle.
    pub fn new(rate: SampleRate) -> Result<Self, MeterError> {
        let mode = ebur128::Mode::I | ebur128::Mode::LRA | ebur128::Mode::TRUE_PEAK;
        let inner =
            ebur128::EbuR128::new(2, rate.get(), mode).map_err(|_| MeterError::SampleRate(rate))?;
        Ok(Self { inner })
    }

    /// Adds interleaved stereo samples.
    ///
    /// # Errors
    ///
    /// [`MeterError::Measure`] if the meter cannot store more history.
    pub fn add(&mut self, interleaved: &[f32]) -> Result<(), MeterError> {
        self.inner
            .add_frames_f32(interleaved)
            .map_err(|_| MeterError::Measure)
    }

    /// The measurement so far.
    #[must_use]
    pub fn loudness(&self) -> Loudness {
        let finite =
            |value: Result<f64, ebur128::Error>| value.ok().filter(|v| v.is_finite()).map(Db);
        let peak = (0..2)
            .filter_map(|channel| self.inner.true_peak(channel).ok())
            .fold(0.0, f64::max);
        Loudness {
            integrated: finite(self.inner.loudness_global()),
            range: finite(self.inner.loudness_range()),
            true_peak: Db::from_gain(peak),
        }
    }
}

/// Measures interleaved stereo in one go.
///
/// # Errors
///
/// As [`LoudnessMeter::new`] and [`LoudnessMeter::add`].
pub fn measure(interleaved: &[f32], rate: SampleRate) -> Result<Loudness, MeterError> {
    let mut meter = LoudnessMeter::new(rate)?;
    meter.add(interleaved)?;
    Ok(meter.loudness())
}

#[cfg(test)]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "test signals are stored as f32; segment lengths are positive"
)]
mod tests {
    use super::*;
    use crate::osc::sine_turns;

    const RATE: SampleRate = SampleRate::HZ_48000;

    /// A stereo 1 kHz sine: each `(seconds, peak dBFS)` segment in turn, phase-continuous.
    fn segments(parts: &[(f64, f64)]) -> Vec<f32> {
        let mut out = Vec::new();
        let mut frame = 0_u64;
        for &(seconds, dbfs) in parts {
            let amplitude = Db(dbfs).to_gain();
            for _ in 0..(seconds * 48_000.0) as u64 {
                let s = (amplitude * sine_turns(frame as f64 * 1_000.0 / 48_000.0)) as f32;
                out.extend([s, s]);
                frame += 1;
            }
        }
        out
    }

    fn integrated(parts: &[(f64, f64)]) -> f64 {
        measure(&segments(parts), RATE)
            .unwrap()
            .integrated
            .unwrap()
            .0
    }

    /// EBU Tech 3341 (V4) minimum-requirement cases 1–5, synthesized: the integrated
    /// loudness is within ±0.1 LU of the expected value.
    #[test]
    fn ebu_tech_3341_cases_1_to_5() {
        let cases: [(&[(f64, f64)], f64); 5] = [
            (&[(20.0, -23.0)], -23.0),
            (&[(20.0, -33.0)], -33.0),
            (&[(10.0, -36.0), (60.0, -23.0), (10.0, -36.0)], -23.0),
            (
                &[
                    (10.0, -72.0),
                    (10.0, -36.0),
                    (60.0, -23.0),
                    (10.0, -36.0),
                    (10.0, -72.0),
                ],
                -23.0,
            ),
            (&[(20.0, -26.0), (20.1, -20.0), (20.0, -26.0)], -23.0),
        ];
        for (index, (parts, expected)) in cases.iter().enumerate() {
            let lufs = integrated(parts);
            assert!(
                (lufs - expected).abs() <= 0.1,
                "case {}: {lufs:.3} LUFS",
                index + 1
            );
        }
    }

    #[test]
    fn silence_has_no_integrated_loudness() {
        let result = measure(&vec![0.0; 2 * 48_000], RATE).unwrap();
        assert_eq!(result.integrated, None);
    }

    #[test]
    fn a_full_scale_sine_peaks_near_0_dbtp() {
        let peak = measure(&segments(&[(1.0, 0.0)]), RATE).unwrap().true_peak.0;
        assert!((peak - 0.0).abs() < 0.1, "{peak} dBTP");
    }
}
