// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `psychophysical` pitch scale (SDR-11): `f = f_ref · (d / d_ref)^(1/m)`.
//!
//! Magnitude estimation (Walker 2002, 2007) finds, for a data concept, the exponent `m` of
//! the power law between a sound dimension and the perceived data magnitude. Inverting it
//! gives a pitch mapping in which equal data *ratios* sound as equal perceived steps. Smith
//! & Walker (2005) mapped dollars to frequency this way, `$ = c · f^0.9256` with
//! `c = 0.0765`, so that $50 sounded at 1100 Hz (docs/research/notes/02-theory-perception.md
//! §2b).
//!
//! Exponents are **guidance, not law** (Walker & Nees 2011: "they should be treated as
//! guidance"); every diagnostic and legend that names one says so ([`GUIDANCE`]).
//!
//! In cents the law is a straight line in `log2(d)`:
//! `pitch = pitch_ref + (1200 / m) · log2(d / d_ref)`, so it is a log scale whose ends are
//! set by the law instead of by a range. Without a `range`, the track's pitch range is the
//! image of the data domain, kept within 20–20 000 Hz; with one, pitches are clamped to it
//! and the clamped data are reported as off-scale.

use sonarithm_core::math;
use sonarithm_core::score::PitchRange;
use sonarithm_core::units::{CENTS_PER_OCTAVE, Cents, Hz};

use super::{OffScale, ScaleError};
use crate::pitch::AUDIBLE_RANGE_HZ;

/// SDR-11: how every message about a psychophysical exponent labels it.
pub const GUIDANCE: &str =
    "psychophysical exponents are guidance from magnitude-estimation studies, not laws (SDR-11)";

/// A psychophysical pitch scale: an exponent `m` and a reference point `(d_ref, f_ref)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Psychophysical {
    exponent: f64,
    reference_value: f64,
    reference_pitch: Cents,
}

/// A datum's pitch on a psychophysical scale, clamped to a range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PsychoPitch {
    /// The pitch, inside the range.
    pub pitch: Cents,
    /// Whether the law's pitch fell outside the range and was clamped, or the datum has a
    /// different sign from `d_ref` (the law is undefined there; it sounds the low end).
    pub off_scale: OffScale,
}

impl Psychophysical {
    /// The scale `f = reference_hz · (d / reference_value)^(1/exponent)`.
    ///
    /// # Errors
    ///
    /// [`ScaleError::InvalidExponent`] unless `exponent` is finite and positive;
    /// [`ScaleError::InvalidReference`] unless `reference_value` is finite and non-zero and
    /// `reference_hz` lies within 20–20 000 Hz.
    pub fn new(exponent: f64, reference_value: f64, reference_hz: f64) -> Result<Self, ScaleError> {
        if !(exponent.is_finite() && exponent > 0.0) {
            return Err(ScaleError::InvalidExponent(exponent));
        }
        let [min, max] = AUDIBLE_RANGE_HZ;
        let usable = reference_value.is_finite()
            && reference_value != 0.0
            && (min..=max).contains(&reference_hz);
        if !usable {
            return Err(ScaleError::InvalidReference {
                value: reference_value,
                hz: reference_hz,
            });
        }
        Ok(Self {
            exponent,
            reference_value,
            reference_pitch: Hz(reference_hz).to_cents(),
        })
    }

    /// The exponent `m`.
    #[must_use]
    pub const fn exponent(&self) -> f64 {
        self.exponent
    }

    /// The law's pitch for `value`, unclamped, or `None` when `value / d_ref` is not
    /// positive (zero, a different sign from `d_ref`, or NaN).
    #[must_use]
    pub fn law(&self, value: f64) -> Option<Cents> {
        let ratio = value / self.reference_value;
        (ratio > 0.0 && ratio.is_finite()).then(|| {
            Cents(self.reference_pitch.0 + CENTS_PER_OCTAVE / self.exponent * math::log2(ratio))
        })
    }

    /// The data value whose law pitch is `pitch`: the inverse of [`Psychophysical::law`].
    #[must_use]
    pub fn value_at(&self, pitch: Cents) -> f64 {
        let octaves = (pitch.0 - self.reference_pitch.0) / CENTS_PER_OCTAVE;
        self.reference_value * math::exp2(octaves * self.exponent)
    }

    /// The pitch range the law gives a data domain: the law's pitches at the domain's ends,
    /// kept within 20–20 000 Hz.
    ///
    /// # Errors
    ///
    /// [`ScaleError::InvalidDomain`] when the domain has no value of `d_ref`'s sign or is
    /// not increasing, so the law has no pitch for it.
    pub fn range_for(&self, [d0, d1]: [f64; 2]) -> Result<PitchRange, ScaleError> {
        let invalid = ScaleError::InvalidDomain { min: d0, max: d1 };
        if d0.partial_cmp(&d1).is_none_or(std::cmp::Ordering::is_gt) {
            return Err(invalid);
        }
        let ends = (self.law(d0), self.law(d1));
        let (low, high) = match ends {
            (Some(a), Some(b)) => (a.0.min(b.0), a.0.max(b.0)),
            _ => return Err(invalid),
        };
        let [min, max] = AUDIBLE_RANGE_HZ.map(|hz| Hz(hz).to_cents().0);
        let (low, high) = (low.clamp(min, max), high.clamp(min, max));
        Ok(PitchRange {
            low: Cents(low),
            high: Cents(high),
        })
    }

    /// The pitch of `value` clamped to `range`, or `None` for NaN (a missing value).
    #[must_use]
    pub fn pitch(&self, value: f64, range: PitchRange) -> Option<PsychoPitch> {
        if value.is_nan() {
            return None;
        }
        let Some(law) = self.law(value) else {
            return Some(PsychoPitch {
                pitch: range.low,
                off_scale: OffScale::Below,
            });
        };
        let off_scale = if law.0 < range.low.0 {
            OffScale::Below
        } else if law.0 > range.high.0 {
            OffScale::Above
        } else {
            OffScale::Within
        };
        Some(PsychoPitch {
            pitch: Cents(law.0.clamp(range.low.0, range.high.0)),
            off_scale,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::float_cmp,
        reason = "clamped pitches equal the range ends exactly"
    )]

    use proptest::prelude::*;

    use super::*;

    fn hz(pitch: Cents) -> f64 {
        pitch.to_hz().0
    }

    /// SDR-11 (SHOULD): the scale follows `f = f_ref · (d / d_ref)^(1/m)` exactly (to
    /// rounding) and passes through its reference point.
    #[test]
    fn sdr_11_psychophysical_scaling_follows_the_power_law() {
        let scale = Psychophysical::new(0.65, 20.0, 600.0).unwrap();
        assert!((hz(scale.law(20.0).unwrap()) - 600.0).abs() < 1e-9);
        for d in [1.0, 5.0, 37.5, 120.0] {
            let expected = 600.0 * math::pow(d / 20.0, 1.0 / 0.65);
            let got = hz(scale.law(d).unwrap());
            assert!(
                (got - expected).abs() <= 1e-9 * expected,
                "{d}: {got} vs {expected}"
            );
        }
        // Equal data ratios give equal pitch steps of 1200 / m cents per doubling.
        let step = scale.law(40.0).unwrap().0 - scale.law(20.0).unwrap().0;
        assert!((step - 1200.0 / 0.65).abs() < 1e-9);
        assert!(GUIDANCE.contains("guidance") && GUIDANCE.contains("SDR-11"));
    }

    /// SDR-11: the Smith & Walker (2005) dollar mapping, `$ = 0.0765 · f^0.9256`, is the
    /// scale with m = 0.9256 through ($50, 1100 Hz); their 200–1900 Hz range is the image of
    /// $10.32 to $82.89.
    #[test]
    fn sdr_11_reproduces_the_smith_and_walker_dollar_mapping() {
        let m = 0.9256;
        let scale = Psychophysical::new(m, 50.0, 1100.0).unwrap();
        let theirs = |dollars: f64| math::pow(dollars / 0.0765, 1.0 / m);
        // Their constant puts $50 at 1100.5 Hz rather than exactly 1100 Hz; the ratio between
        // the two mappings is the same at every dollar value, so the exponent matches.
        let ratios: Vec<f64> = [12.0, 30.0, 50.0, 80.0]
            .iter()
            .map(|&d| hz(scale.law(d).unwrap()) / theirs(d))
            .collect();
        for ratio in &ratios {
            assert!((ratio - ratios[0]).abs() < 1e-12);
            assert!((ratio - 1.0).abs() < 2e-3, "{ratio}");
        }
        let low_dollars = 0.0765 * math::pow(200.0, m);
        let high_dollars = 0.0765 * math::pow(1900.0, m);
        assert!((low_dollars - 10.316).abs() < 1e-3 && (high_dollars - 82.885).abs() < 1e-3);
        let range = scale.range_for([low_dollars, high_dollars]).unwrap();
        assert!((hz(range.low) - 200.0).abs() < 0.5 && (hz(range.high) - 1900.0).abs() < 3.0);
    }

    /// SDR-11: without a range, the track's range is the law's image of the data domain,
    /// kept within 20–20 000 Hz; with one, pitches are clamped to it and reported.
    #[test]
    fn sdr_11_ranges_come_from_the_law_or_clamp_to_the_given_range() {
        let scale = Psychophysical::new(0.5, 1.0, 440.0).unwrap();
        let range = scale.range_for([1.0, 2.0]).unwrap();
        assert!((range.low.0 - 0.0).abs() < 1e-9 && (range.high.0 - 2400.0).abs() < 1e-9);
        // 2^(1/0.5) per doubling: 1 → 1000 spans 20 octaves, more than the audible range.
        let wide = scale.range_for([1e-3, 1e3]).unwrap();
        assert!((hz(wide.low) - 20.0).abs() < 1e-6 && (hz(wide.high) - 20_000.0).abs() < 1e-3);
        let given = PitchRange {
            low: Hz(400.0).to_cents(),
            high: Hz(1400.0).to_cents(),
        };
        let high = scale.pitch(100.0, given).unwrap();
        assert_eq!((high.pitch, high.off_scale), (given.high, OffScale::Above));
        let inside = scale.pitch(1.5, given).unwrap();
        assert_eq!(inside.off_scale, OffScale::Within);
        // The law is undefined for values of the other sign; they sound the low end.
        let negative = scale.pitch(-3.0, given).unwrap();
        assert_eq!(
            (negative.pitch, negative.off_scale),
            (given.low, OffScale::Below)
        );
        assert_eq!(scale.pitch(f64::NAN, given), None);
        assert_eq!(
            scale.range_for([-2.0, -1.0]),
            Err(ScaleError::InvalidDomain {
                min: -2.0,
                max: -1.0
            })
        );
    }

    /// Negative data work with a negative reference value: the ratio stays positive.
    #[test]
    fn negative_data_use_a_negative_reference() {
        let scale = Psychophysical::new(1.0, -10.0, 440.0).unwrap();
        assert!((hz(scale.law(-20.0).unwrap()) - 880.0).abs() < 1e-9);
        assert_eq!(scale.law(5.0), None);
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        assert_eq!(
            Psychophysical::new(0.0, 1.0, 440.0),
            Err(ScaleError::InvalidExponent(0.0))
        );
        assert_eq!(
            Psychophysical::new(0.9, 0.0, 440.0),
            Err(ScaleError::InvalidReference {
                value: 0.0,
                hz: 440.0
            })
        );
        assert!(Psychophysical::new(0.9, 1.0, 19.0).is_err());
        assert!(Psychophysical::new(0.9, 1.0, 30_000.0).is_err());
    }

    proptest! {
        /// SDR-11: for a positive reference the law rises with the data, and `value_at`
        /// inverts it.
        #[test]
        fn sdr_11_law_is_monotonic_and_invertible(
            m in 0.2..3.0_f64,
            reference in 0.01..1e4_f64,
            hz in 50.0..5000.0_f64,
            a in 1e-3..1e5_f64,
            b in 1e-3..1e5_f64,
        ) {
            let scale = Psychophysical::new(m, reference, hz).unwrap();
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(scale.law(lo).unwrap().0 <= scale.law(hi).unwrap().0);
            let back = scale.value_at(scale.law(a).unwrap());
            prop_assert!((back - a).abs() <= 1e-9 * a, "{a} → {back}");
        }
    }
}
