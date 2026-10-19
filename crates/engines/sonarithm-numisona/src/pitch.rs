// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pitch spaces and pitch ranges (SDR-08, SDR-09; 04 §5.3, RFC-0001 §6).
//!
//! A pitch channel maps a scale position `u` in [0, 1] to a pitch between the ends of its
//! range. The **pitch space** decides what "evenly spaced" means: equal steps in `u` are
//! equal steps in the space's coordinate.
//!
//! | Space | Coordinate of a frequency `f` (Hz) | Source |
//! |---|---|---|
//! | `log-frequency` (default, SDR-09) | cents, `1200 · log2(f / 440)` | equal-tempered semitones |
//! | `linear-frequency` | `f` | — |
//! | `mel` | `2595 · log10(1 + f / 700)` | the engineering form popularised by O'Shaughnessy (1987) for the scale of Stevens, Volkmann & Newman (1937) |
//! | `erb` | `21.4 · log10(1 + 0.00437 · f)`, the ERB-number | Glasberg & Moore (1990) |
//! | `bark` | `26.81 · f / (1960 + f) − 0.53` | Traunmüller (1990), analytical form without his end corrections |
//!
//! Sources are summarised in docs/research/notes/02-theory-perception.md §2d.
//!
//! The range defaults to 400–1400 Hz ([`DEFAULT_RANGE_HZ`], SDR-08). Pitches are returned
//! in cents relative to A4, the Score IR's unit, and always lie inside the range's ends in
//! cents, so 05 §4.2 invariant 3 holds whatever the space.

use sonarithm_core::math;
use sonarithm_core::score::PitchRange;
use sonarithm_core::units::{Cents, Hz, Midi};

/// SDR-08 (DEFAULT): the default pitch range, 400–1400 Hz (about MIDI 67–89).
///
/// Evidence: in a study of three groups of 16 listeners, a 400–1400 Hz range performed as
/// well as a 65–1480 Hz range on line-chart tasks (Fitzpatrick & Neff 2023; one study,
/// n = 48, Moderate).
pub const DEFAULT_RANGE_HZ: [f64; 2] = [400.0, 1400.0];

/// SDR-08 (DEFAULT): the expanded range the default can be widened to, 65–1480 Hz (the
/// wide range of Fitzpatrick & Neff 2023).
pub const EXPANDED_RANGE_HZ: [f64; 2] = [65.0, 1480.0];

/// The bounds any pitch range must stay within, 20–20 000 Hz (RFC-0001 §6, the `pitch`
/// channel's bounds).
pub const AUDIBLE_RANGE_HZ: [f64; 2] = [20.0, 20_000.0];

/// A pitch space (SDR-09). Equal steps of the scale position are equal steps of the
/// space's coordinate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PitchSpace {
    /// SDR-09 (DEFAULT): log-frequency, in cents (equal semitones).
    #[default]
    LogFrequency,
    /// Linear frequency, in hertz.
    LinearFrequency,
    /// Mel.
    Mel,
    /// ERB-rate (ERB-number).
    Erb,
    /// Bark (critical-band rate).
    Bark,
}

impl PitchSpace {
    /// Every space, default first.
    pub const ALL: [Self; 5] = [
        Self::LogFrequency,
        Self::LinearFrequency,
        Self::Mel,
        Self::Erb,
        Self::Bark,
    ];

    /// The space's name in the spec (`space`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LogFrequency => "log-frequency",
            Self::LinearFrequency => "linear-frequency",
            Self::Mel => "mel",
            Self::Erb => "erb",
            Self::Bark => "bark",
        }
    }

    /// The coordinate of `hz` in this space.
    #[must_use]
    pub fn coordinate(self, hz: Hz) -> f64 {
        let f = hz.0;
        match self {
            Self::LogFrequency => hz.to_cents().0,
            Self::LinearFrequency => f,
            Self::Mel => 2595.0 * math::log10(1.0 + f / 700.0),
            Self::Erb => 21.4 * math::log10(1.0 + 0.00437 * f),
            Self::Bark => 26.81 * f / (1960.0 + f) - 0.53,
        }
    }

    /// The frequency at `coordinate`; the inverse of [`PitchSpace::coordinate`].
    #[must_use]
    pub fn frequency(self, coordinate: f64) -> Hz {
        let x = coordinate;
        match self {
            Self::LogFrequency => Cents(x).to_hz(),
            Self::LinearFrequency => Hz(x),
            Self::Mel => Hz(700.0 * (math::pow(10.0, x / 2595.0) - 1.0)),
            Self::Erb => Hz((math::pow(10.0, x / 21.4) - 1.0) / 0.00437),
            Self::Bark => Hz(1960.0 * (x + 0.53) / (26.28 - x)),
        }
    }
}

/// A pitch given in one of the spec's pitch units (RFC-0001 §3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PitchValue {
    /// Hertz.
    Hz(f64),
    /// A MIDI note number, 69 = A4.
    Midi(f64),
    /// Cents relative to A4 = 440 Hz.
    Cents(f64),
}

impl PitchValue {
    /// The pitch in cents, exactly for MIDI and cents input.
    #[must_use]
    pub fn cents(self) -> Cents {
        match self {
            Self::Hz(hz) => Hz(hz).to_cents(),
            Self::Midi(note) => Midi(note).to_cents(),
            Self::Cents(cents) => Cents(cents),
        }
    }
}

/// Why a pitch mapping cannot be built.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum PitchError {
    /// The range is not finite and increasing.
    #[error("the pitch range must go from low to high; use polarity to invert a mapping")]
    NotIncreasing,
    /// The range leaves 20–20 000 Hz.
    #[error("the pitch range {low:.1}–{high:.1} Hz must stay within 20–20000 Hz")]
    OutOfBounds {
        /// The low end in hertz.
        low: f64,
        /// The high end in hertz.
        high: f64,
    },
}

/// A resolved pitch mapping: a space and a range, from scale positions to pitches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchMap {
    space: PitchSpace,
    range: PitchRange,
    coordinates: [f64; 2],
}

impl PitchMap {
    /// The mapping over `[low, high]` in `space`.
    ///
    /// # Errors
    ///
    /// [`PitchError::NotIncreasing`] unless `low < high`; [`PitchError::OutOfBounds`] if the
    /// range leaves [`AUDIBLE_RANGE_HZ`] (with a small tolerance for unit rounding).
    pub fn new(space: PitchSpace, low: PitchValue, high: PitchValue) -> Result<Self, PitchError> {
        let range = PitchRange {
            low: low.cents(),
            high: high.cents(),
        };
        if !(range.low.is_finite() && range.high.is_finite() && range.low.0 < range.high.0) {
            return Err(PitchError::NotIncreasing);
        }
        let [low_hz, high_hz] = [range.low.to_hz().0, range.high.to_hz().0];
        // 0.00001 cent of slack, so bounds written in MIDI notes or cents that convert to
        // 20 Hz or 20 kHz only up to rounding are accepted.
        let slack = 1e-5 / 1200.0 * math::ln(2.0);
        let [min, max] = AUDIBLE_RANGE_HZ;
        if low_hz < min * (1.0 - slack) || high_hz > max * (1.0 + slack) {
            return Err(PitchError::OutOfBounds {
                low: low_hz,
                high: high_hz,
            });
        }
        Ok(Self::from_range(space, range))
    }

    /// SDR-08, SDR-09 (DEFAULT): the default mapping, 400–1400 Hz in log-frequency.
    #[must_use]
    pub fn default_range() -> Self {
        let [low, high] = DEFAULT_RANGE_HZ;
        Self::from_range(
            PitchSpace::default(),
            PitchRange {
                low: Hz(low).to_cents(),
                high: Hz(high).to_cents(),
            },
        )
    }

    /// The mapping over a range already checked to be increasing and audible.
    fn from_range(space: PitchSpace, range: PitchRange) -> Self {
        let coordinates = match space {
            PitchSpace::LogFrequency => [range.low.0, range.high.0],
            space => [
                space.coordinate(range.low.to_hz()),
                space.coordinate(range.high.to_hz()),
            ],
        };
        Self {
            space,
            range,
            coordinates,
        }
    }

    /// The pitch space.
    #[must_use]
    pub const fn space(&self) -> PitchSpace {
        self.space
    }

    /// The range in cents, low to high, as the Score records it.
    #[must_use]
    pub const fn range(&self) -> PitchRange {
        self.range
    }

    /// The pitch at scale position `u` (clamped to [0, 1]): `u = 0` is the low end and
    /// `u = 1` the high end, and equal steps of `u` are equal steps in the space.
    #[must_use]
    pub fn pitch(&self, u: f64) -> Cents {
        let u = if u.is_nan() { 0.5 } else { u.clamp(0.0, 1.0) };
        let PitchRange { low, high } = self.range;
        // The ends are exact, so a datum at the edge of the domain sounds the range's end.
        if u <= 0.0 {
            return low;
        }
        if u >= 1.0 {
            return high;
        }
        let cents = match self.space {
            // No round trip through hertz.
            PitchSpace::LogFrequency => low.0 + u * (high.0 - low.0),
            space => {
                let [x0, x1] = self.coordinates;
                space.frequency(x0 + u * (x1 - x0)).to_cents().0
            }
        };
        Cents(cents.clamp(low.0, high.0))
    }

    /// The scale position of `pitch` in this mapping; the inverse of [`PitchMap::pitch`]
    /// inside the range, used by legends, announcements and the polarity mirror.
    #[must_use]
    pub fn position(&self, pitch: Cents) -> f64 {
        let PitchRange { low, high } = self.range;
        let pitch = Cents(pitch.0.clamp(low.0, high.0));
        let [x0, x1] = self.coordinates;
        let x = match self.space {
            PitchSpace::LogFrequency => pitch.0,
            space => space.coordinate(pitch.to_hz()),
        };
        ((x - x0) / (x1 - x0)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::float_cmp,
        reason = "range ends are exact by construction and are tested exactly"
    )]

    use proptest::prelude::*;

    use super::*;

    fn hz(pitch: Cents) -> f64 {
        pitch.to_hz().0
    }

    /// SDR-08 (DEFAULT): without a `range`, pitch spans 400–1400 Hz, about MIDI 67–89, and
    /// the documented expanded range is 65–1480 Hz.
    #[test]
    fn sdr_08_default_range_is_400_to_1400_hz() {
        let map = PitchMap::default_range();
        assert_eq!(map.space(), PitchSpace::LogFrequency);
        assert!((hz(map.pitch(0.0)) - 400.0).abs() < 1e-9);
        assert!((hz(map.pitch(1.0)) - 1400.0).abs() < 1e-9);
        let [low, high] = [map.range().low.to_midi().0, map.range().high.to_midi().0];
        assert_eq!((low.round(), high.round()), (67.0, 89.0));
        assert_eq!(DEFAULT_RANGE_HZ, [400.0, 1400.0]);
        assert_eq!(EXPANDED_RANGE_HZ, [65.0, 1480.0]);
        let expanded = PitchMap::new(
            PitchSpace::LogFrequency,
            PitchValue::Hz(EXPANDED_RANGE_HZ[0]),
            PitchValue::Hz(EXPANDED_RANGE_HZ[1]),
        );
        assert!(expanded.is_ok());
    }

    /// SDR-08: ranges given in hertz, MIDI notes or cents resolve to the same cents, and a
    /// reversed or out-of-bounds range is refused (polarity inverts a mapping instead).
    #[test]
    fn sdr_08_ranges_accept_every_pitch_unit() {
        let by_hz = PitchMap::new(
            PitchSpace::LogFrequency,
            PitchValue::Hz(220.0),
            PitchValue::Hz(880.0),
        )
        .unwrap();
        let by_midi = PitchMap::new(
            PitchSpace::LogFrequency,
            PitchValue::Midi(57.0),
            PitchValue::Midi(81.0),
        )
        .unwrap();
        let by_cents = PitchMap::new(
            PitchSpace::LogFrequency,
            PitchValue::Cents(-1200.0),
            PitchValue::Cents(1200.0),
        )
        .unwrap();
        assert_eq!(by_midi.range(), by_cents.range());
        assert!((by_hz.range().low.0 - by_cents.range().low.0).abs() < 1e-9);
        assert_eq!(
            PitchMap::new(
                PitchSpace::Mel,
                PitchValue::Hz(1400.0),
                PitchValue::Hz(400.0)
            ),
            Err(PitchError::NotIncreasing)
        );
        assert!(matches!(
            PitchMap::new(PitchSpace::Mel, PitchValue::Hz(10.0), PitchValue::Hz(400.0)),
            Err(PitchError::OutOfBounds { .. })
        ));
        let audible = PitchMap::new(
            PitchSpace::Erb,
            PitchValue::Hz(20.0),
            PitchValue::Hz(20_000.0),
        );
        assert!(audible.is_ok());
    }

    /// SDR-09 (DEFAULT): the default space is log-frequency, where equal scale steps are
    /// equal steps in cents.
    #[test]
    fn sdr_09_default_space_is_log_frequency() {
        assert_eq!(PitchSpace::default(), PitchSpace::LogFrequency);
        let map = PitchMap::default_range();
        let steps: Vec<f64> = (0..=4)
            .map(|i| map.pitch(f64::from(i) / 4.0).0)
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect();
        for step in &steps {
            assert!((step - steps[0]).abs() < 1e-9, "{steps:?}");
        }
        // A quarter of 400–1400 Hz in log-frequency is a quarter of 2168.8 cents.
        assert!((steps[0] - 1200.0 * math::log2(3.5) / 4.0).abs() < 1e-9);
    }

    /// SDR-09: mel, ERB-rate and Bark are options; each matches its published formula at
    /// reference frequencies.
    #[test]
    fn sdr_09_mel_erb_and_bark_follow_their_published_formulas() {
        // Mel: 1000 Hz is about 1000 mel in the O'Shaughnessy form.
        let mel = PitchSpace::Mel.coordinate(Hz(1000.0));
        assert!((mel - 999.985_537_14).abs() < 1e-6, "{mel}");
        // ERB-number at 1 kHz: 21.4 · log10(5.37) ≈ 15.62 (Glasberg & Moore 1990).
        let erb = PitchSpace::Erb.coordinate(Hz(1000.0));
        assert!((erb - 21.4 * math::log10(5.37)).abs() < 1e-12);
        assert!((erb - 15.621).abs() < 1e-3, "{erb}");
        // Bark at 1 kHz: 26.81 · 1000 / 2960 − 0.53 ≈ 8.53 (Traunmüller 1990).
        let bark = PitchSpace::Bark.coordinate(Hz(1000.0));
        assert!((bark - (26.81 * 1000.0 / 2960.0 - 0.53)).abs() < 1e-12);
        assert!((bark - 8.527).abs() < 1e-3, "{bark}");
        assert_eq!(PitchSpace::LinearFrequency.coordinate(Hz(1000.0)), 1000.0);
        let names: Vec<&str> = PitchSpace::ALL.iter().map(|s| s.name()).collect();
        assert_eq!(
            names,
            ["log-frequency", "linear-frequency", "mel", "erb", "bark"]
        );
    }

    /// SDR-09: in every space, equal scale steps are equal steps of that space's coordinate,
    /// and the ends of the range are reached exactly.
    #[test]
    fn sdr_09_every_space_spaces_steps_evenly_in_its_own_coordinate() {
        for space in PitchSpace::ALL {
            let map = PitchMap::new(space, PitchValue::Hz(400.0), PitchValue::Hz(1400.0)).unwrap();
            let coords: Vec<f64> = (0..=8)
                .map(|i| space.coordinate(map.pitch(f64::from(i) / 8.0).to_hz()))
                .collect();
            let first = coords[1] - coords[0];
            for pair in coords.windows(2) {
                let step = pair[1] - pair[0];
                assert!(
                    (step - first).abs() <= 1e-9 * first.abs().max(1.0),
                    "{space:?}"
                );
            }
            assert_eq!(map.pitch(0.0), map.range().low, "{space:?}");
            assert_eq!(map.pitch(1.0), map.range().high, "{space:?}");
        }
        // The spaces differ: the midpoint of 400–1400 Hz lies lowest in log-frequency and
        // highest in linear frequency.
        let mid = |space| {
            let map = PitchMap::new(space, PitchValue::Hz(400.0), PitchValue::Hz(1400.0)).unwrap();
            hz(map.pitch(0.5))
        };
        assert!((mid(PitchSpace::LogFrequency) - 748.33).abs() < 0.01);
        assert!((mid(PitchSpace::LinearFrequency) - 900.0).abs() < 1e-6);
        assert!(mid(PitchSpace::LogFrequency) < mid(PitchSpace::Erb));
        assert!(mid(PitchSpace::Erb) < mid(PitchSpace::LinearFrequency));
    }

    fn space() -> impl Strategy<Value = PitchSpace> {
        prop::sample::select(PitchSpace::ALL.to_vec())
    }

    proptest! {
        /// Each space's coordinate rises with frequency and its inverse undoes it over the
        /// audible range.
        #[test]
        fn coordinates_rise_and_invert(space in space(), a in 20.0..20_000.0_f64, b in 20.0..20_000.0_f64) {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(space.coordinate(Hz(lo)) <= space.coordinate(Hz(hi)));
            let back = space.frequency(space.coordinate(Hz(a))).0;
            prop_assert!((back - a).abs() <= 1e-9 * a, "{a} → {back}");
        }

        /// SDR-08: every mapped pitch lies within the resolved range (05 §4.2 invariant 3),
        /// rises with the scale position, and `position` inverts `pitch`.
        #[test]
        fn sdr_08_pitches_stay_in_range_and_rise(
            space in space(),
            low in 20.0..10_000.0_f64,
            ratio in 1.001..100.0_f64,
            a in -0.5..1.5_f64,
            b in -0.5..1.5_f64,
        ) {
            let high = (low * ratio).min(20_000.0);
            let map = PitchMap::new(space, PitchValue::Hz(low), PitchValue::Hz(high)).unwrap();
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            let (p, q) = (map.pitch(lo), map.pitch(hi));
            prop_assert!(map.range().contains(p) && map.range().contains(q));
            prop_assert!(p.0 <= q.0);
            let u = lo.clamp(0.0, 1.0);
            prop_assert!((map.position(map.pitch(u)) - u).abs() < 1e-9);
        }
    }
}
