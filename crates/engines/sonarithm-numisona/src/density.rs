// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Event density and the `auto` mark (SDR-20, INFER: **experimental**).
//!
//! Density studies stop at 8 points per second (Nees & Walker 2008), and metrical grouping
//! has a floor of about 100 ms between onsets (London 2012; Repp 2005). Together they
//! suggest that above about 10 events per second discrete notes stop being heard as
//! separate events and a continuous mark should take over. That threshold is an untested
//! inference: hypothesis H-02 (EXP-002) tests it, and until then `auto` stays opt-in (the
//! spec's `mark` has no default, RFC-0001 Q16) and every use is reported as experimental.
//!
//! `auto` chooses between `tone` and `glide`. Audification would suit very dense series,
//! but it belongs to CURVASONA (C03), and grain textures to TONALENS (D01); the encoder says
//! so when it switches.

/// SDR-20 (INFER, experimental): above this many events per second (an inter-onset interval
/// under 100 ms), `auto` plays a glide instead of tones.
pub const GLIDE_ABOVE_EVENTS_PER_S: f64 = 10.0;

/// The mark `auto` resolves to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoMark {
    /// Discrete tones.
    Tone,
    /// A continuous glide.
    Glide,
}

/// The mean event rate of onsets in seconds: `(n − 1) / (last − first)`, or `None` with
/// fewer than two onsets or no time between them.
#[must_use]
pub fn events_per_second(onsets: &[f64]) -> Option<f64> {
    let (first, last) = (onsets.first()?, onsets.last()?);
    let span = last - first;
    #[expect(
        clippy::cast_precision_loss,
        reason = "event counts stay far below 2^52"
    )]
    let gaps = (onsets.len() - 1) as f64;
    (onsets.len() >= 2 && span > 0.0).then(|| gaps / span)
}

/// SDR-20 (INFER, experimental): the mark for an event rate. Rates up to and including
/// [`GLIDE_ABOVE_EVENTS_PER_S`] keep tones; faster ones glide. Simultaneous onsets (no time
/// between the first and last) also glide, since no tone could be told apart.
#[must_use]
pub fn choose(onsets: &[f64]) -> AutoMark {
    match events_per_second(onsets) {
        Some(rate) if rate <= GLIDE_ABOVE_EVENTS_PER_S => AutoMark::Tone,
        None if onsets.len() < 2 => AutoMark::Tone,
        _ => AutoMark::Glide,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, reason = "rates of evenly spaced onsets are exact")]

    use super::*;

    fn even(rate: f64, n: u32) -> Vec<f64> {
        (0..n).map(|k| f64::from(k) / rate).collect()
    }

    /// SDR-20 (INFER, experimental): `auto` keeps tones up to 10 events per second and glides
    /// above; the threshold is the documented inference.
    #[test]
    fn sdr_20_auto_glides_above_10_events_per_second() {
        assert_eq!(GLIDE_ABOVE_EVENTS_PER_S, 10.0);
        assert_eq!(choose(&even(3.0, 30)), AutoMark::Tone);
        assert_eq!(choose(&even(10.0, 30)), AutoMark::Tone);
        assert_eq!(choose(&even(10.5, 30)), AutoMark::Glide);
        assert_eq!(choose(&even(100.0, 2000)), AutoMark::Glide);
    }

    #[test]
    fn rates_come_from_the_onsets() {
        assert_eq!(events_per_second(&even(4.0, 9)), Some(4.0));
        assert_eq!(events_per_second(&[1.0]), None);
        assert_eq!(events_per_second(&[]), None);
        assert_eq!(events_per_second(&[2.0, 2.0]), None);
        assert_eq!(choose(&[1.0]), AutoMark::Tone);
        assert_eq!(choose(&[2.0, 2.0, 2.0]), AutoMark::Glide);
    }
}
