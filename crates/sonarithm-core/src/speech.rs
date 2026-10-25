// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Estimated speech durations (08 §4, keyboard-and-navigation §9.1).
//!
//! Screen readers do not report when they finish speaking (macOS gives no completion
//! callback), so sequencing speech with sonification (A11Y-20) and timing `WebVTT` cues use
//! an estimate from the text: its words at a speaking rate. The estimate decides how long
//! sonification waits or stays ducked; it never cuts speech short.

use serde::{Deserialize, Serialize};

use crate::units::Seconds;

/// A speaking rate in words per minute.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WordsPerMinute(pub f64);

impl WordsPerMinute {
    /// The rate estimates assume unless the listener sets their own: 180 words per minute,
    /// an unhurried synthetic voice. TODO(verify): a placeholder until co-design (B07)
    /// measures the rates SONARITHM's listeners use. Screen-reader users often listen much
    /// faster, so at this rate estimates err long: sonification waits a little more rather
    /// than talking over speech.
    pub const DEFAULT: Self = Self(180.0);

    /// The slowest rate an estimate uses, so a zero or negative setting cannot make an
    /// estimate unbounded.
    pub const MIN: Self = Self(30.0);
}

impl Default for WordsPerMinute {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// How long `text` takes to speak at `rate`: its whitespace-separated words at that rate.
/// Empty text takes no time.
#[must_use]
pub fn spoken_duration(text: &str, rate: WordsPerMinute) -> Seconds {
    let words = text.split_whitespace().count();
    let rate = if rate.0.is_finite() {
        rate.0.max(WordsPerMinute::MIN.0)
    } else {
        WordsPerMinute::DEFAULT.0
    };
    #[expect(clippy::cast_precision_loss, reason = "a word count is far below 2^52")]
    let words = words as f64;
    Seconds(words * 60.0 / rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_at_the_rate() {
        let text = "Temperature 12.5 degrees Celsius, 3 of 12";
        assert_eq!(
            spoken_duration(text, WordsPerMinute(120.0)),
            Seconds(3.5),
            "7 words at 2 words per second"
        );
        assert_eq!(spoken_duration("  ", WordsPerMinute::DEFAULT), Seconds(0.0));
    }

    #[test]
    fn a_bad_rate_never_makes_an_unbounded_estimate() {
        let one = |rate| spoken_duration("word", WordsPerMinute(rate)).0;
        assert!((one(0.0) - 2.0).abs() < 1e-12, "the minimum rate");
        assert!((one(-5.0) - 2.0).abs() < 1e-12);
        assert!((one(f64::NAN) - 60.0 / 180.0).abs() < 1e-12, "the default");
        assert!(one(f64::INFINITY).is_finite());
    }
}
