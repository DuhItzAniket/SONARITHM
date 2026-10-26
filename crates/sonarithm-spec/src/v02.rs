// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The sections spec v0.2 adds (RFC-0004): `template`, `instruments`, `lint`, a typed chart
//! `calibration`, chart `corrections`, and the `music.quantize` vocabularies.
//!
//! `calibration`, `corrections`, `scale` and `tuning` are kept as JSON in [`crate::Spec`]
//! and read into the types here by validation, because v0.1 accepted any value there:
//! a v0.1 document with a value v0.2 rejects still reads, and fails validation with the
//! RFC's code at the exact pointer (RFC-0004 §9, Q14). The schema shows the typed shapes.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::json::JsonValue;

/// `template`: a reusable design bound to a dataset's fields (RFC-0004 §2, SDR-03).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct TemplateRef {
    /// The template ID, lowercase `kebab-case`, as an engine registers it.
    #[schemars(extend("pattern" = "^[a-z][a-z0-9]*(-[a-z0-9]+)*$"))]
    pub id: String,
    /// Slot name → field name; slots not named bind by role, then by kind.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub bind: BTreeMap<String, String>,
}

impl TemplateRef {
    /// A template reference without bindings.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bind: BTreeMap::new(),
        }
    }
}

/// An instrument definition, tagged by `kind` (RFC-0004 §3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum Instrument {
    /// A sound font (SF2) file the user supplies.
    Soundfont {
        /// The SF2 file, relative to the spec.
        path: String,
        /// The bank, 0 … 16383; default 0.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(max = 16383))]
        bank: Option<u16>,
        /// The program, 0 … 127; default 0.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(max = 127))]
        program: Option<u8>,
    },
}

/// `lint`: rule levels and suppressions (RFC-0004 §4).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Lint {
    /// Design-rule ID → level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<BTreeMap<String, LintLevel>>,
    /// Rules turned off, for the spec or one track, each with a reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppress: Option<Vec<Suppression>>,
    /// Turns on the experimental (`INFER`) rules; default `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<bool>,
}

/// A lint rule level.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[expect(missing_docs, reason = "the variant names are the levels")]
pub enum LintLevel {
    Off,
    Info,
    Warning,
    Error,
}

/// One suppression.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Suppression {
    /// The design-rule ID, such as `SDR-17`.
    pub rule: String,
    /// The track it applies to; the whole spec when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    /// Why; required and not empty.
    pub reason: String,
}

/// The lint rules ECHOMETRIC implements (RFC-0004 §4; C05-T07 moves them from NUMISONA's
/// encoder, C05-T08 adds the experimental ones). An ID outside this list gives
/// `SNR-SPEC-045`, a warning, so a spec written for a newer lint still compiles.
pub const LINT_RULES: &[&str] = &[
    "A11Y-14", "SDR-14", "SDR-17", "SDR-19", "SDR-21", "SDR-23", "SDR-26", "SDR-27", "SDR-28",
    "SDR-39",
];

/// `data.source.calibration` for chart sources (RFC-0004 §5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Calibration {
    /// The coordinate system; only `cartesian` in v0.2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<CoordinateSystem>,
    /// The page (PDF only), from 1; default 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1))]
    pub page: Option<u32>,
    /// The plot area in source coordinates; found by the extractor when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plot_area: Option<PlotArea>,
    /// The x and y axes.
    pub axes: CalibrationAxes,
}

/// A calibration's coordinate system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum CoordinateSystem {
    /// x and y axes at right angles.
    Cartesian,
}

/// A rectangle in source coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct PlotArea {
    /// The corner with the smaller coordinates.
    pub min: [f64; 2],
    /// The corner with the larger coordinates.
    pub max: [f64; 2],
}

/// The two axes of a calibration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct CalibrationAxes {
    /// The horizontal axis.
    pub x: CalibrationAxis,
    /// The vertical axis.
    pub y: CalibrationAxis,
}

/// One axis: its kind and at least two anchors.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct CalibrationAxis {
    /// The scale kind.
    pub kind: AxisKind,
    /// Points on the source and the values they stand for; at least two.
    #[schemars(length(min = 2))]
    pub anchors: Vec<CalibrationAnchor>,
}

/// An axis kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum AxisKind {
    /// Numbers at equal distances per equal step.
    Linear,
    /// Positive numbers at equal distances per equal ratio.
    Log,
    /// ISO 8601 dates or date-times.
    Date,
    /// Labels.
    Category,
}

/// A source point and its value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct CalibrationAnchor {
    /// The point in the source's own coordinates: pixels (y down, a pixel's centre at
    /// +0.5), SVG user units, or PDF default user space.
    pub at: [f64; 2],
    /// A number (`linear`, `log`), an ISO 8601 date (`date`) or a label (`category`).
    pub value: AnchorValue,
}

/// An anchor's value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[expect(missing_docs, reason = "the variants are the JSON types")]
pub enum AnchorValue {
    Number(f64),
    Text(String),
}

/// `data.source.corrections` (RFC-0004 §6): inline edits; the file form is reserved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Corrections {
    /// The edits, applied in order after extraction and before `transform`.
    pub edits: Vec<Edit>,
}

/// One correction, tagged by `op`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum Edit {
    /// A point the extractor missed.
    Add {
        /// The series name as extracted.
        series: String,
        /// The new point, in data units.
        to: PointValue,
    },
    /// A point at the wrong value.
    Move {
        /// The series name as extracted.
        series: String,
        /// The 0-based index in the extractor's order.
        point: u32,
        /// The extracted value, a guard.
        was: PointValue,
        /// The new value: `x`, `y` or both.
        to: PointValue,
    },
    /// A point that is not data.
    Delete {
        /// The series name as extracted.
        series: String,
        /// The 0-based index in the extractor's order.
        point: u32,
        /// The extracted value, a guard.
        was: PointValue,
    },
    /// A point given to the wrong series.
    Retype {
        /// The series name as extracted.
        series: String,
        /// The 0-based index in the extractor's order.
        point: u32,
        /// The extracted value, a guard.
        was: PointValue,
        /// The series it belongs to.
        to: SeriesTarget,
    },
}

/// A point value in data units; either part may be absent where an edit allows it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct PointValue {
    /// The x value (a number, or a label or date as text).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<AnchorValue>,
    /// The y value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

/// The series a `retype` edit moves a point to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct SeriesTarget {
    /// The series name.
    pub series: String,
}

/// `music.quantize.scale`: a name such as `"D minor pentatonic"`, or custom degrees
/// (RFC-0004 §7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ScaleSpec {
    /// `"<tonic> <mode>"`.
    Name(String),
    /// Degrees in cents above the tonic, strictly increasing from 0, below 1200.
    Custom(CustomScale),
}

/// A custom scale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct CustomScale {
    /// The tonic, `A`–`G` with an optional `#` or `b`.
    pub tonic: String,
    /// The degrees in cents above the tonic.
    pub cents: Vec<f64>,
}

/// `music.quantize.tuning` (RFC-0004 §7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[non_exhaustive]
pub enum TuningSpec {
    /// `12-TET` (`just` is reserved, RFC-0004 Q13).
    Name(String),
    /// The twelve pitch classes in cents above the tonic.
    Custom(CustomTuning),
}

/// A custom twelve-note tuning.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct CustomTuning {
    /// `c0` … `c11`, strictly increasing, `c0` = 0, all below 1200.
    pub cents: Vec<f64>,
}

/// The modes of a scale name and their degrees in semitones above the tonic (RFC-0004 §7).
pub const MODES: &[(&str, &[u8])] = &[
    ("major", &[0, 2, 4, 5, 7, 9, 11]),
    ("minor", &[0, 2, 3, 5, 7, 8, 10]),
    ("harmonic minor", &[0, 2, 3, 5, 7, 8, 11]),
    ("melodic minor", &[0, 2, 3, 5, 7, 9, 11]),
    ("dorian", &[0, 2, 3, 5, 7, 9, 10]),
    ("phrygian", &[0, 1, 3, 5, 7, 8, 10]),
    ("lydian", &[0, 2, 4, 6, 7, 9, 11]),
    ("mixolydian", &[0, 2, 4, 5, 7, 9, 10]),
    ("locrian", &[0, 1, 3, 5, 6, 8, 10]),
    ("major pentatonic", &[0, 2, 4, 7, 9]),
    ("minor pentatonic", &[0, 3, 5, 7, 10]),
    ("blues", &[0, 3, 5, 6, 7, 10]),
    ("whole tone", &[0, 2, 4, 6, 8, 10]),
    ("chromatic", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
];

/// The tuning names this version accepts (RFC-0004 §7). `just` is reserved: it ships only
/// with a cited 5-limit ratio table (RFC-0004 Q13, C04-T02), and until then it is
/// `SNR-SPEC-044` with a hint.
pub const TUNINGS: &[&str] = &["12-TET"];

/// A scale name read: the tonic's pitch class (0 = C) and the mode's degrees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaleName {
    /// The tonic as a pitch class, 0 (C) … 11 (B).
    pub tonic: u8,
    /// The mode's degrees in semitones above the tonic.
    pub degrees: &'static [u8],
}

/// Reads `"<tonic> <mode>"` (words separated by single spaces), or `None`.
#[must_use]
pub fn parse_scale_name(name: &str) -> Option<ScaleName> {
    let (tonic, mode) = name.split_once(' ')?;
    let tonic = tonic_class(tonic)?;
    let degrees = MODES.iter().find(|(m, _)| *m == mode).map(|(_, d)| *d)?;
    Some(ScaleName { tonic, degrees })
}

/// A tonic `A`–`G` with an optional `#` or `b`, as a pitch class (0 = C).
#[must_use]
pub fn tonic_class(tonic: &str) -> Option<u8> {
    let mut chars = tonic.chars();
    let base: i8 = match chars.next()? {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let shift: i8 = match chars.next() {
        None => 0,
        Some('#') => 1,
        Some('b') => -1,
        Some(_) => return None,
    };
    if chars.next().is_some() {
        return None;
    }
    u8::try_from((base + shift).rem_euclid(12)).ok()
}

/// Whether cents degrees are strictly increasing, start at 0 and stay below 1200.
#[must_use]
pub fn valid_cents(cents: &[f64]) -> bool {
    cents.first() == Some(&0.0)
        && cents.iter().all(|c| c.is_finite() && *c < 1200.0)
        && cents.windows(2).all(|w| w[0] < w[1])
}

/// Every section v0.2 adds as raw JSON, read as its type, for hosts and engines.
///
/// # Errors
///
/// The `serde_json` error when `value` does not have the type's shape.
pub fn read<T: serde::de::DeserializeOwned>(value: &JsonValue) -> Result<T, serde_json::Error> {
    serde_json::from_value(to_json(value))
}

/// The JSON of a [`JsonValue`], with whole numbers written as integers: the spec keeps
/// every number as `f64`, and an index such as `"point": 12` must read back as an integer.
#[must_use]
pub fn to_json(value: &JsonValue) -> serde_json::Value {
    use serde_json::Value;
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "checked to be whole and in range"
            )]
            let whole =
                (n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0).then_some(*n as i64);
            match whole {
                Some(i) => Value::from(i),
                None => serde_json::Number::from_f64(*n).map_or(Value::Null, Value::Number),
            }
        }
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(items) => Value::Array(items.iter().map(to_json).collect()),
        JsonValue::Object(map) => {
            Value::Object(map.iter().map(|(k, v)| (k.clone(), to_json(v))).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_names_read_as_rfc_0004_section_7_lists_them() {
        let pentatonic = parse_scale_name("C major pentatonic").unwrap();
        assert_eq!(
            (pentatonic.tonic, pentatonic.degrees),
            (0, &[0, 2, 4, 7, 9][..])
        );
        assert_eq!(
            parse_scale_name("D minor pentatonic").map(|s| s.tonic),
            Some(2)
        );
        assert_eq!(
            parse_scale_name("Bb harmonic minor").map(|s| s.tonic),
            Some(10)
        );
        assert_eq!(parse_scale_name("Cb major").map(|s| s.tonic), Some(11));
        for bad in [
            "H major",
            "C  major",
            "C Major",
            "C",
            "C# minor  ",
            "Cx major",
            "major",
        ] {
            assert_eq!(parse_scale_name(bad), None, "{bad}");
        }
        assert_eq!(MODES.len(), 14);
        assert!(valid_cents(&[0.0, 150.0, 350.0]));
        assert!(!valid_cents(&[10.0, 150.0]));
        assert!(!valid_cents(&[0.0, 0.0]));
        assert!(!valid_cents(&[0.0, 1200.0]));
    }
}
