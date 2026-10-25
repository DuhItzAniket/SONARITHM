// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `substrate`, `context`, `analysis`, `music`, `playback` and `output`
//! (RFC-0001 §5, §7–11).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::values::{DurationValue, Toggle};

/// `substrate`: the temporal canvas (Enge et al., SDR-04).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Substrate {
    /// The time substrate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeSubstrate>,
}

/// The time substrate.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct TimeSubstrate {
    /// The ordering field; default: the field with `role=x`, else row order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// The pace; default 3 points/s (SDR-19).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace: Option<Pace>,
}

/// The pace: exactly one of three keys.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Pace {
    /// Data points per second (SDR-19: 2–4 for overviews).
    PointsPerSecond(f64),
    /// The duration of the whole series.
    Duration(DurationValue),
    /// Seconds per unit of the time field.
    SecondsPerUnit(f64),
}

/// `context` (AUDAXIS): ticks, reference tone, legend, earcons, spatial mode.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Context {
    /// The x-axis tick track (SDR-16).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticks: Option<Ticks>,
    /// The y-axis reference tone (SDR-17).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<Reference>,
    /// The spoken legend (SDR-18).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legend: Option<Legend>,
    /// Earcons (A11Y-08).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub earcons: Option<Earcons>,
    /// Spatial mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial: Option<Spatial>,
}

/// The tick track (SDR-16).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Ticks {
    /// The axis; only `x` in v0.1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<Axis>,
    /// Major tick interval in units of the time field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<f64>,
    /// Minor ticks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minor: Option<bool>,
    /// `"auto"` (on for discrete marks, off for glides), `true` or `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<Toggle>,
}

/// Axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Axis {
    /// The time axis.
    X,
}

/// The reference tone (SDR-17).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Reference {
    /// The reference kind.
    pub kind: ReferenceKind,
    /// The track it refers to; default: the first data track.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    /// Its timbre, distinct from the data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timbre: Option<String>,
    /// The value for `static-value`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

/// Reference kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ReferenceKind {
    /// No reference tone.
    None,
    /// The running extreme (SDR-17; default).
    DynamicExtreme,
    /// A fixed value (lint warns, SDR-17).
    StaticValue,
    /// The zero line.
    ZeroLine,
}

/// The spoken legend (SDR-18).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Legend {
    /// When it is spoken; default `before-play`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speak: Option<LegendSpeak>,
    /// What it says; default: all parts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<LegendPart>>,
}

/// When the legend is spoken (RFC-0001 Q14).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum LegendSpeak {
    /// Before playback starts.
    BeforePlay,
    /// Only on request.
    OnDemand,
    /// Never spoken automatically; still in the navigation tree.
    Off,
}

/// Legend parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the legend parts")]
pub enum LegendPart {
    Title,
    XRange,
    YRange,
    Units,
    Polarity,
}

/// Earcons (A11Y-08).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Earcons {
    /// Boundary earcons; default on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<bool>,
    /// Landmark earcons; default on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landmarks: Option<bool>,
    /// Threshold crossings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thresholds: Vec<Threshold>,
}

/// A threshold whose crossing plays an earcon.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Threshold {
    /// The track.
    pub track: String,
    /// The value, in data units.
    pub value: f64,
}

/// Spatial settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Spatial {
    /// The spatial mode.
    pub mode: SpatialMode,
}

/// Spatial modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum SpatialMode {
    /// Stereo (default).
    Stereo,
    /// Binaural (D03).
    Binaural,
    /// Multichannel (D03).
    Multichannel,
}

/// `analysis` (ECHOMETRIC, CURVASONA).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Analysis {
    /// Detectors to run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detect: Vec<Detector>,
    /// How detections are cued; default `earcon`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cue: Option<Cue>,
    /// Generated descriptions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub describe: Option<Describe>,
}

/// Detectors.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[expect(missing_docs, reason = "the variant names are the detector names")]
pub enum Detector {
    Extrema,
    Outliers,
    Changepoints,
    Periodicity,
    TrendSegments,
    CriticalPoints,
}

/// Cue kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Cue {
    /// An earcon (default).
    Earcon,
    /// Speech.
    Speech,
    /// No cue.
    None,
}

/// Description settings (A11Y-02).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Describe {
    /// Description levels; default L1 and L2. L4 is rejected by validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<DescribeLevel>>,
}

/// Description levels (Lundgard and Satyanarayan).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum DescribeLevel {
    /// Construction: chart type, axes, encodings.
    L1,
    /// Statistics: extrema, ranges, averages.
    L2,
    /// Trends and patterns (opt-in phrasing).
    L3,
    /// Interpretation; never automatic (A11Y-02), rejected by validation.
    L4,
}

/// `music` (AXONOTE), Presentation mode only (SDR-10).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Music {
    /// Pitch quantization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantize: Option<Quantize>,
    /// Tempo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tempo: Option<Tempo>,
    /// Voicing; default `none`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voicing: Option<Voicing>,
}

/// Quantization to a scale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Quantize {
    /// The scale: a name such as `C major pentatonic`, or custom cents (RFC-0004 §7).
    #[schemars(with = "crate::v02::ScaleSpec")]
    pub scale: crate::json::JsonValue,
    /// The tuning: `12-TET` (default for named scales), `just`, or custom cents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<crate::v02::TuningSpec>")]
    pub tuning: Option<crate::json::JsonValue>,
}

/// Tempo.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Tempo {
    /// Beats per minute.
    pub bpm: f64,
}

/// Voicing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Voicing {
    /// No chords (default).
    None,
}

/// `playback` (SONOTRACE).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Playback {
    /// Playback speed; default 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// Loop; default `false`.
    #[serde(rename = "loop", default, skip_serializing_if = "Option::is_none")]
    pub looping: Option<bool>,
    /// Monitoring settings (SDR-35).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor: Option<Monitor>,
}

/// Monitoring settings (SDR-35).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Monitor {
    /// The ambient soundscape, such as `forest`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soundscape: Option<String>,
    /// Alarms; reserved for true alarms (SDR-35).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alarms: Vec<Alarm>,
}

/// An alarm on a detector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Alarm {
    /// The detector.
    pub detector: Detector,
    /// How salient the alarm is.
    pub salience: Salience,
}

/// Alarm salience.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[expect(missing_docs, reason = "the variant names are the salience levels")]
pub enum Salience {
    Low,
    Medium,
    High,
}

/// `output`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Output {
    /// Sample rate in Hz; default 48000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    /// Channel count; default 2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<u16>,
    /// Block size in frames; default 256.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_size: Option<u32>,
    /// Export loudness targets (proposed defaults, sound-design §6.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loudness: Option<OutputLoudness>,
    /// Sinks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sinks: Vec<Sink>,
}

/// Export loudness targets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct OutputLoudness {
    /// Integrated loudness target; default −16 LUFS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_lufs: Option<f64>,
    /// True-peak limit; default −1 dBTP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub true_peak_dbtp: Option<f64>,
}

/// A sink, tagged by `kind` (RFC-0001 Q15).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum Sink {
    /// A standard MIDI file.
    MidiFile {
        /// Path relative to the spec.
        path: String,
    },
    /// Open Sound Control messages.
    Osc {
        /// `host:port`.
        address: String,
    },
}
