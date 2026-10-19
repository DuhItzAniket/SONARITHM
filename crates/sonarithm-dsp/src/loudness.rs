// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Equal-loudness compensation from ISO 226:2023 (SDR-24, DEFAULT).
//!
//! SDR-24: ISO 226:2023 compensation is on by default for perceptual consistency, so tones
//! of different pitch sound equally salient. It is not claimed to improve accuracy
//! (Fitzpatrick & Neff 2023).
//!
//! [`spl_for_loudness_level`] is Formula (1) of ISO 226:2023 with the parameters of its
//! Table 1, and [`loudness_level`] is its inverse, Formula (2). Between the table's
//! third-octave frequencies the parameters are interpolated linearly in log frequency;
//! outside 20 Hz–12.5 kHz they are held at the end values. [`Compensation`] turns the
//! contours into gains for a pitch range, never boosting: the least sensitive frequency in
//! the range plays at 0 dB, and the others are attenuated to match it. All maths uses
//! `libm`, so plans are identical on every platform (ADR-0013).

use sonarithm_core::{Db, Hz, math};

/// ISO 226:2023 Table 1: frequency in Hz, exponent `α_f`, transfer function `L_U` in dB and
/// hearing threshold `T_f` in dB, for the preferred third-octave frequencies of ISO 266.
/// Checked against the table as printed in the standard's official preview
/// (standards.iteh.ai, ISO 226:2023(E), p. 4).
pub const TABLE_1: [(f64, f64, f64, f64); 29] = [
    (20.0, 0.635, -31.5, 78.1),
    (25.0, 0.602, -27.2, 68.7),
    (31.5, 0.569, -23.1, 59.5),
    (40.0, 0.537, -19.3, 51.1),
    (50.0, 0.509, -16.1, 44.0),
    (63.0, 0.482, -13.1, 37.5),
    (80.0, 0.456, -10.4, 31.5),
    (100.0, 0.433, -8.2, 26.5),
    (125.0, 0.412, -6.3, 22.1),
    (160.0, 0.391, -4.6, 17.9),
    (200.0, 0.373, -3.2, 14.4),
    (250.0, 0.357, -2.1, 11.4),
    (315.0, 0.343, -1.2, 8.6),
    (400.0, 0.330, -0.5, 6.2),
    (500.0, 0.320, 0.0, 4.4),
    (630.0, 0.311, 0.4, 3.0),
    (800.0, 0.303, 0.5, 2.2),
    (1_000.0, 0.300, 0.0, 2.4),
    (1_250.0, 0.295, -2.7, 3.5),
    (1_600.0, 0.292, -4.2, 1.7),
    (2_000.0, 0.290, -1.2, -1.3),
    (2_500.0, 0.290, 1.4, -4.2),
    (3_150.0, 0.289, 2.3, -6.0),
    (4_000.0, 0.289, 1.0, -5.4),
    (5_000.0, 0.289, -2.3, -1.5),
    (6_300.0, 0.293, -7.2, 6.0),
    (8_000.0, 0.303, -11.2, 12.6),
    (10_000.0, 0.323, -10.9, 13.9),
    (12_500.0, 0.354, -3.5, 12.3),
];

/// The loudness level the compensation assumes: 60 phon, a comfortable listening level.
/// A05 provisional value: the playback level is unknown to the renderer, and no design rule
/// fixes it yet (TODO(verify) with the sound-design review).
pub const REFERENCE_PHON: f64 = 60.0;

/// The parameters `(α_f, L_U, T_f)` at `frequency`, interpolated in log frequency.
fn parameters(frequency: f64) -> (f64, f64, f64) {
    let first = TABLE_1[0];
    let last = TABLE_1[TABLE_1.len() - 1];
    if frequency.is_nan() || frequency <= first.0 {
        return (first.1, first.2, first.3);
    }
    if frequency >= last.0 {
        return (last.1, last.2, last.3);
    }
    let upper = TABLE_1
        .iter()
        .position(|row| row.0 >= frequency)
        .unwrap_or(TABLE_1.len() - 1);
    let (a, b) = (TABLE_1[upper.saturating_sub(1)], TABLE_1[upper]);
    let w = math::log2(frequency / a.0) / math::log2(b.0 / a.0);
    let mix = |x: f64, y: f64| x + (y - x) * w;
    (mix(a.1, b.1), mix(a.2, b.2), mix(a.3, b.3))
}

/// ISO 226:2023 Formula (1): the sound pressure level in dB of a pure tone at `frequency`
/// that has the loudness level `phon`.
#[must_use]
pub fn spl_for_loudness_level(frequency: Hz, phon: f64) -> Db {
    let (alpha, lu, tf) = parameters(frequency.0);
    let inner = math::pow(4e-10, 0.3 - alpha)
        * (math::pow(10.0, 0.03 * phon) - math::pow(10.0, 0.072))
        + math::pow(10.0, alpha * (tf + lu) / 10.0);
    Db(10.0 / alpha * math::log10(inner) - lu)
}

/// ISO 226:2023 Formula (2): the loudness level in phon of a pure tone at `frequency` and
/// `spl` dB. NaN below the hearing threshold.
#[must_use]
pub fn loudness_level(frequency: Hz, spl: Db) -> f64 {
    let (alpha, lu, tf) = parameters(frequency.0);
    let excess =
        math::pow(10.0, alpha * (spl.0 + lu) / 10.0) - math::pow(10.0, alpha * (tf + lu) / 10.0);
    100.0 / 3.0 * math::log10(excess / math::pow(4e-10, 0.3 - alpha) + math::pow(10.0, 0.072))
}

/// Equal-loudness gains for one pitch range at one loudness level (SDR-24).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Compensation {
    phon: f64,
    /// The highest level the contour needs inside the range; it maps to 0 dB.
    ceiling: f64,
}

/// Points sampled across a range to find the contour's maximum.
const RANGE_SAMPLES: u32 = 64;

impl Compensation {
    /// Compensation for tones from `low` to `high` at `phon`. The contour's highest level
    /// inside the range (sampled at 64 log-spaced points and both ends) plays at 0 dB.
    #[must_use]
    pub fn for_range(low: Hz, high: Hz, phon: f64) -> Self {
        let (low, high) = if low.0 <= high.0 {
            (low.0, high.0)
        } else {
            (high.0, low.0)
        };
        let low = low.max(1.0);
        let high = high.max(low);
        let ceiling = (0..=RANGE_SAMPLES)
            .map(|i| {
                let w = f64::from(i) / f64::from(RANGE_SAMPLES);
                let f = low * math::pow(high / low, w);
                spl_for_loudness_level(Hz(f), phon).0
            })
            .fold(f64::NEG_INFINITY, f64::max);
        Self { phon, ceiling }
    }

    /// The gain for a tone at `frequency`: 0 dB at the least sensitive point of the range,
    /// negative elsewhere. Frequencies outside the range get the gain of the nearest table
    /// parameters and are not limited to 0 dB.
    #[must_use]
    pub fn gain(&self, frequency: Hz) -> Db {
        Db(spl_for_loudness_level(frequency, self.phon).0 - self.ceiling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// At 1 kHz the formula is the identity: `L_f = L_N`.
    #[test]
    fn sdr_24_one_kilohertz_is_the_reference() {
        for phon in [20.0, 40.0, 60.0, 80.0, 90.0] {
            let spl = spl_for_loudness_level(Hz(1_000.0), phon).0;
            assert!((spl - phon).abs() < 1e-9, "{phon} phon: {spl} dB");
        }
    }

    /// Formula (2) inverts Formula (1) everywhere in the valid range.
    #[test]
    fn sdr_24_formulas_1_and_2_are_inverses() {
        for &(f, ..) in &TABLE_1 {
            for phon in [20.0, 40.0, 60.0, 80.0] {
                let spl = spl_for_loudness_level(Hz(f), phon);
                let back = loudness_level(Hz(f), spl);
                assert!((back - phon).abs() < 1e-6, "{f} Hz, {phon} phon: {back}");
            }
        }
    }

    /// The contours have their textbook shape: low frequencies need far more level, and the
    /// ear is most sensitive near 3–4 kHz.
    #[test]
    fn sdr_24_contours_have_the_textbook_shape() {
        let at = |f: f64| spl_for_loudness_level(Hz(f), 60.0).0;
        assert!(at(100.0) > at(1_000.0) + 10.0);
        assert!(at(3_150.0) < at(1_000.0));
        assert!(at(20.0) > at(100.0));
        // Interpolated values lie between their neighbours.
        let between = at(1_100.0);
        assert!((at(1_000.0).min(at(1_250.0))..=at(1_000.0).max(at(1_250.0))).contains(&between));
    }

    /// SDR-24: over the SDR-08 default range (400–1400 Hz) the gains never boost, the least
    /// sensitive pitch plays at 0 dB, and a lower, less sensitive pitch gets the most gain.
    #[test]
    fn sdr_24_compensation_never_boosts() {
        let comp = Compensation::for_range(Hz(400.0), Hz(1_400.0), REFERENCE_PHON);
        let gains: Vec<f64> = (0..=50)
            .map(|i| comp.gain(Hz(400.0 * math::pow(3.5, f64::from(i) / 50.0))).0)
            .collect();
        assert!(gains.iter().all(|g| *g <= 1e-9));
        assert!(gains.iter().any(|g| g.abs() < 1e-9));
        assert!(gains[0] > gains[25], "400 Hz needs more level than ~750 Hz");
        assert!(
            gains.iter().all(|g| *g > -6.0),
            "the contour is flat to within 6 dB here"
        );
    }

    #[test]
    fn out_of_range_frequencies_hold_the_end_parameters() {
        assert_eq!(parameters(5.0), parameters(20.0));
        assert_eq!(parameters(f64::NAN), parameters(20.0));
        assert_eq!(parameters(20_000.0), parameters(12_500.0));
    }
}
