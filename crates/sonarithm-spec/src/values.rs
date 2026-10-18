// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared value shapes: durations, unit-tagged values and ranges (RFC-0001 §3).
//!
//! A unit-tagged value is an object with exactly one unit key, such as `{ "hz": 440 }`,
//! so a number is never ambiguous between hertz, MIDI notes and cents.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A duration: `{ "s": n }` or `{ "ms": n }`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum DurationValue {
    /// Seconds.
    S(f64),
    /// Milliseconds.
    Ms(f64),
}

impl DurationValue {
    /// The duration in seconds.
    #[must_use]
    pub fn seconds(self) -> f64 {
        match self {
            Self::S(s) => s,
            Self::Ms(ms) => ms / 1000.0,
        }
    }
}

/// A unit and its spec key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unit {
    /// `hz`: hertz.
    Hz,
    /// `midi`: MIDI note number (69 = A4).
    Midi,
    /// `cents`: cents relative to A4 = 440 Hz.
    Cents,
    /// `s`: seconds.
    S,
    /// `ms`: milliseconds.
    Ms,
    /// `db`: decibels, relative.
    Db,
}

impl Unit {
    /// The unit's key in the spec.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Hz => "hz",
            Self::Midi => "midi",
            Self::Cents => "cents",
            Self::S => "s",
            Self::Ms => "ms",
            Self::Db => "db",
        }
    }
}

/// A value with a unit key.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
#[expect(missing_docs, reason = "each variant is the unit of the same name")]
pub enum UnitValue {
    Hz(f64),
    Midi(f64),
    Cents(f64),
    S(f64),
    Ms(f64),
    Db(f64),
}

impl UnitValue {
    /// The unit and the number.
    #[must_use]
    pub const fn parts(self) -> (Unit, f64) {
        match self {
            Self::Hz(x) => (Unit::Hz, x),
            Self::Midi(x) => (Unit::Midi, x),
            Self::Cents(x) => (Unit::Cents, x),
            Self::S(x) => (Unit::S, x),
            Self::Ms(x) => (Unit::Ms, x),
            Self::Db(x) => (Unit::Db, x),
        }
    }
}

/// A two-element range with a unit key, such as `{ "hz": [400, 1400] }`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
#[expect(missing_docs, reason = "each variant is the unit of the same name")]
pub enum UnitRange {
    Hz([f64; 2]),
    Midi([f64; 2]),
    Cents([f64; 2]),
    S([f64; 2]),
    Ms([f64; 2]),
    Db([f64; 2]),
}

impl UnitRange {
    /// The unit and the two bounds as written.
    #[must_use]
    pub const fn parts(self) -> (Unit, [f64; 2]) {
        match self {
            Self::Hz(r) => (Unit::Hz, r),
            Self::Midi(r) => (Unit::Midi, r),
            Self::Cents(r) => (Unit::Cents, r),
            Self::S(r) => (Unit::S, r),
            Self::Ms(r) => (Unit::Ms, r),
            Self::Db(r) => (Unit::Db, r),
        }
    }
}

/// A constant channel value: a plain number, a preset name, or a unit-tagged value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ChannelValue {
    /// A plain number (pan, brightness, taps, density, rate in events/s).
    Number(f64),
    /// A preset name (timbre).
    Name(String),
    /// A unit-tagged value.
    Unit(UnitValue),
}

/// A channel range: a plain `[lo, hi]` or a unit-tagged range.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ChannelRange {
    /// A plain range for unitless channels.
    Plain([f64; 2]),
    /// A unit-tagged range.
    Unit(UnitRange),
}

impl ChannelRange {
    /// The unit (`None` for a plain range) and the two bounds as written.
    #[must_use]
    pub const fn parts(self) -> (Option<Unit>, [f64; 2]) {
        match self {
            Self::Plain(r) => (None, r),
            Self::Unit(range) => {
                let (unit, r) = range.parts();
                (Some(unit), r)
            }
        }
    }
}

/// `"auto"`, the only keyword of [`Toggle`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AutoKeyword {
    /// Let the compiler decide.
    Auto,
}

/// `"auto"`, `true` or `false` (RFC-0001 Q4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Toggle {
    /// Decided by the compiler from the data and marks.
    Auto(AutoKeyword),
    /// On or off.
    Set(bool),
}
