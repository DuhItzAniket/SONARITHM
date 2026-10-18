// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Physical and musical quantities as newtypes (coding-standards §6).
//!
//! Score-level time and pitch are `f64`; absolute sample positions are integer [`Frames`]
//! (05 §4.1). A bare `f64` meaning seconds or decibels never crosses a public API.
//! Conversions are explicit methods that state their rounding rule, and they use
//! [`crate::math`] so their results are identical on every platform (ADR-0013).

use core::cmp::Ordering;
use core::fmt;
use core::num::NonZeroU32;
use core::ops::{Add, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

use crate::math;

/// The reference pitch A4 in hertz: 0 cents and MIDI note 69 (05 §4.1).
pub const A4_HZ: f64 = 440.0;
/// The MIDI note number of A4.
pub const A4_MIDI: f64 = 69.0;
/// Cents per equal-tempered semitone (one MIDI note step).
pub const CENTS_PER_SEMITONE: f64 = 100.0;
/// Cents per octave.
pub const CENTS_PER_OCTAVE: f64 = 1200.0;

/// Upper bound (exclusive) of `u64` as an `f64`: 2^64.
const U64_LIMIT: f64 = 18_446_744_073_709_551_616.0;

/// Shared trait implementations for the `f64` newtypes.
macro_rules! float_unit {
    ($name:ident, $suffix:literal) => {
        impl $name {
            /// Compares two values with IEEE 754 `totalOrder`, for sorting (coding-standards §10).
            #[must_use]
            pub fn total_cmp(&self, other: &Self) -> Ordering {
                self.0.total_cmp(&other.0)
            }

            /// Whether the value is neither infinite nor NaN.
            #[must_use]
            pub fn is_finite(self) -> bool {
                self.0.is_finite()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!("{}", $suffix), self.0)
            }
        }
    };
}

/// Additive arithmetic for quantities where sums are meaningful.
macro_rules! additive_unit {
    ($name:ident) => {
        impl Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }
    };
}

/// A frequency in hertz.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hz(pub f64);

impl Hz {
    /// The pitch in cents relative to A4: `1200 · log2(f / 440 Hz)`.
    ///
    /// A non-positive frequency gives `-inf` or NaN; validate ranges before converting.
    #[must_use]
    pub fn to_cents(self) -> Cents {
        Cents(CENTS_PER_OCTAVE * math::log2(self.0 / A4_HZ))
    }

    /// The fractional MIDI note number: `69 + 12 · log2(f / 440 Hz)`.
    #[must_use]
    pub fn to_midi(self) -> Midi {
        self.to_cents().to_midi()
    }
}

float_unit!(Hz, " Hz");

/// A pitch in cents relative to A4 = 440 Hz, the Score IR's pitch unit (05 §4.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cents(pub f64);

impl Cents {
    /// The frequency: `440 Hz · 2^(c / 1200)`.
    #[must_use]
    pub fn to_hz(self) -> Hz {
        Hz(A4_HZ * math::exp2(self.0 / CENTS_PER_OCTAVE))
    }

    /// The fractional MIDI note number: `69 + c / 100`.
    #[must_use]
    pub fn to_midi(self) -> Midi {
        Midi(A4_MIDI + self.0 / CENTS_PER_SEMITONE)
    }
}

float_unit!(Cents, " cents");
additive_unit!(Cents);

/// A fractional MIDI note number (69 = A4 = 440 Hz).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Midi(pub f64);

impl Midi {
    /// The pitch in cents relative to A4: `(n − 69) · 100`.
    #[must_use]
    pub fn to_cents(self) -> Cents {
        Cents((self.0 - A4_MIDI) * CENTS_PER_SEMITONE)
    }

    /// The frequency in hertz.
    #[must_use]
    pub fn to_hz(self) -> Hz {
        self.to_cents().to_hz()
    }
}

float_unit!(Midi, "");

/// A level in decibels, relative to a stated reference (full scale, the data bus, …).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Db(pub f64);

impl Db {
    /// 0 dB: unity gain.
    pub const ZERO: Self = Self(0.0);

    /// The level of a linear amplitude ratio: `20 · log10(gain)`.
    ///
    /// A gain of zero gives `-inf`; a negative gain gives NaN.
    #[must_use]
    pub fn from_gain(gain: f64) -> Self {
        Self(20.0 * math::log10(gain))
    }

    /// The linear amplitude ratio: `10^(dB / 20)`.
    #[must_use]
    pub fn to_gain(self) -> f64 {
        math::pow(10.0, self.0 / 20.0)
    }
}

float_unit!(Db, " dB");
additive_unit!(Db);

/// A time or duration in seconds, the Score IR's time unit (05 §4.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Seconds(pub f64);

impl Seconds {
    /// Zero seconds.
    pub const ZERO: Self = Self(0.0);

    /// A duration given in milliseconds.
    #[must_use]
    pub fn from_millis(ms: f64) -> Self {
        Self(ms / 1000.0)
    }

    /// The nearest frame at `rate`, rounding half to even (05 §4.1).
    ///
    /// Returns `None` when the time is negative, not finite, or beyond `u64::MAX` frames.
    #[must_use]
    pub fn to_frames(self, rate: SampleRate) -> Option<Frames> {
        let exact = self.0 * f64::from(rate.get());
        if !exact.is_finite() || exact < 0.0 {
            return None;
        }
        let rounded = exact.round_ties_even();
        if rounded >= U64_LIMIT {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "rounded is an integer in [0, 2^64), checked above"
        )]
        Some(Frames(rounded as u64))
    }
}

float_unit!(Seconds, " s");
additive_unit!(Seconds);

impl Mul<f64> for Seconds {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self(self.0 * rhs)
    }
}

/// An absolute sample position or a length in samples (frames per channel).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Frames(pub u64);

impl Frames {
    /// The time of this frame at `rate`: `frames / rate`.
    ///
    /// Exact to the nearest `f64` for positions below 2^53 frames (about 5.9 million
    /// years at 48 kHz).
    #[must_use]
    pub fn to_seconds(self, rate: SampleRate) -> Seconds {
        #[expect(
            clippy::cast_precision_loss,
            reason = "exact below 2^53 frames, documented above"
        )]
        let frames = self.0 as f64;
        Seconds(frames / f64::from(rate.get()))
    }
}

impl fmt::Display for Frames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} frames", self.0)
    }
}

/// A sample rate in frames per second; never zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SampleRate(NonZeroU32);

impl SampleRate {
    /// 48 kHz, the default output and golden-test rate.
    pub const HZ_48000: Self = Self(NonZeroU32::MIN.saturating_add(47_999));

    /// A sample rate of `hz` frames per second, or `None` for zero.
    #[must_use]
    pub const fn new(hz: u32) -> Option<Self> {
        match NonZeroU32::new(hz) {
            Some(rate) => Some(Self(rate)),
            None => None,
        }
    }

    /// The rate in frames per second.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Hz", self.0)
    }
}

/// A stereo position: −1 is hard left, 0 is centre, 1 is hard right.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pan(pub f64);

impl Pan {
    /// The centre position.
    pub const CENTRE: Self = Self(0.0);

    /// `position` limited to [−1, 1]; NaN becomes the centre.
    #[must_use]
    pub fn clamped(position: f64) -> Self {
        if position.is_nan() {
            Self::CENTRE
        } else {
            Self(position.clamp(-1.0, 1.0))
        }
    }
}

float_unit!(Pan, "");

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn rate(hz: u32) -> SampleRate {
        SampleRate::new(hz).unwrap()
    }

    #[test]
    fn a4_is_440_hz_0_cents_and_midi_69() {
        assert_eq!(Hz(440.0).to_cents(), Cents(0.0));
        assert_eq!(Cents(0.0).to_hz(), Hz(440.0));
        assert_eq!(Cents(0.0).to_midi(), Midi(69.0));
        assert_eq!(Midi(81.0).to_hz(), Hz(880.0));
        assert_eq!(Hz(220.0).to_cents(), Cents(-1200.0));
    }

    #[test]
    fn decibels_convert_to_amplitude_ratios() {
        assert_eq!(Db(0.0).to_gain(), 1.0);
        assert_eq!(Db(-20.0).to_gain(), 0.1);
        assert_eq!(Db::from_gain(10.0), Db(20.0));
        assert_eq!(Db::from_gain(0.0), Db(f64::NEG_INFINITY));
    }

    #[test]
    fn seconds_to_frames_rounds_half_to_even() {
        // 0.5 / 1 Hz = 0.5 frames → 0; 1.5 → 2; 2.5 → 2.
        let one = rate(1);
        assert_eq!(Seconds(0.5).to_frames(one), Some(Frames(0)));
        assert_eq!(Seconds(1.5).to_frames(one), Some(Frames(2)));
        assert_eq!(Seconds(2.5).to_frames(one), Some(Frames(2)));
        assert_eq!(
            Seconds(1.0).to_frames(SampleRate::HZ_48000),
            Some(Frames(48_000))
        );
    }

    #[test]
    fn seconds_to_frames_rejects_negative_infinite_and_huge_times() {
        let r = SampleRate::HZ_48000;
        assert_eq!(Seconds(-0.001).to_frames(r), None);
        assert_eq!(Seconds(f64::NAN).to_frames(r), None);
        assert_eq!(Seconds(f64::INFINITY).to_frames(r), None);
        assert_eq!(Seconds(1e300).to_frames(r), None);
        assert_eq!(Seconds(-0.0).to_frames(r), Some(Frames(0)));
    }

    #[test]
    fn sample_rate_is_never_zero() {
        assert_eq!(SampleRate::new(0), None);
        assert_eq!(SampleRate::HZ_48000.get(), 48_000);
    }

    #[test]
    fn pan_clamps_to_the_stereo_field() {
        assert_eq!(Pan::clamped(-3.0), Pan(-1.0));
        assert_eq!(Pan::clamped(0.25), Pan(0.25));
        assert_eq!(Pan::clamped(f64::NAN), Pan::CENTRE);
    }

    #[test]
    fn units_serialize_as_plain_numbers() {
        let json = serde_json::to_string(&(Hz(440.0), Seconds(0.25), Frames(3))).unwrap();
        assert_eq!(json, "[440.0,0.25,3]");
        let rate: SampleRate = serde_json::from_str("44100").unwrap();
        assert_eq!(rate.get(), 44_100);
        assert!(serde_json::from_str::<SampleRate>("0").is_err());
    }

    #[test]
    fn display_names_the_unit() {
        assert_eq!(Hz(440.0).to_string(), "440 Hz");
        assert_eq!(Db(-18.0).to_string(), "-18 dB");
        assert_eq!(Seconds(0.25).to_string(), "0.25 s");
        assert_eq!(Frames(256).to_string(), "256 frames");
    }

    fn relative_error(a: f64, b: f64) -> f64 {
        (a - b).abs() / a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
    }

    proptest! {
        /// A03-T01 acceptance: Hz ↔ cents ↔ MIDI round trips stay within 1e-9.
        #[test]
        fn hz_cents_midi_round_trip_within_1e_9(f in 1.0_f64..30_000.0) {
            let hz = Hz(f);
            prop_assert!(relative_error(hz.to_cents().to_hz().0, f) < 1e-9);
            prop_assert!(relative_error(hz.to_midi().to_hz().0, f) < 1e-9);
            let cents = hz.to_cents();
            prop_assert!((cents.to_midi().to_cents().0 - cents.0).abs() < 1e-9);
        }

        #[test]
        fn midi_cents_round_trip_within_1e_9(n in -20.0_f64..160.0) {
            let midi = Midi(n);
            prop_assert!((midi.to_cents().to_midi().0 - n).abs() < 1e-9);
            prop_assert!((midi.to_hz().to_midi().0 - n).abs() < 1e-9);
        }

        #[test]
        fn db_gain_round_trip_within_1e_9(db in -120.0_f64..24.0) {
            prop_assert!((Db::from_gain(Db(db).to_gain()).0 - db).abs() < 1e-9);
        }

        /// Seconds → frames → seconds stays within half a frame (testing-strategy §3).
        #[test]
        fn frames_round_trip_within_half_a_frame(
            t in 0.0_f64..86_400.0,
            hz in prop::sample::select(vec![8_000_u32, 22_050, 44_100, 48_000, 96_000, 192_000]),
        ) {
            let r = rate(hz);
            let frames = Seconds(t).to_frames(r).unwrap();
            let back = frames.to_seconds(r).0;
            prop_assert!((back - t).abs() <= 0.5 / f64::from(hz) + 1e-12);
        }

        /// Whole frames survive frames → seconds → frames exactly.
        #[test]
        fn whole_frames_round_trip_exactly(n in 0_u64..(1_u64 << 40), hz in 1_u32..400_000) {
            let r = rate(hz);
            prop_assert_eq!(Frames(n).to_seconds(r).to_frames(r), Some(Frames(n)));
        }
    }
}
