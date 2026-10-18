// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `tracks`: marks and channels (RFC-0001 §6, 04 §5.2–5.3).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sonarithm_core::engine::MarkKind;

use crate::values::{ChannelRange, ChannelValue, UnitRange};

/// One track: a stream of auditory marks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct TrackSpec {
    /// The track ID, unique in the spec (`^[A-Za-z][A-Za-z0-9_-]{0,63}$`).
    pub id: String,
    /// The encoder engine; default: the first encoder registered for the mark.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// The mark. Required: `auto` depends on SDR-20, an experimental rule (RFC-0001 Q16).
    pub mark: Mark,
    /// The instrument preset; default: the encoder's (SDR-29).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrument: Option<String>,
    /// The channels.
    #[serde(default)]
    pub channels: Channels,
    /// Redundant encodings of the pitch field (A11Y-13).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redundant: Vec<Redundant>,
    /// A row filter for this track.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
    /// Audification speed for the `audify` mark (SDR-31).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audify: Option<Audify>,
}

/// Mark kinds (04 §5.3) with their technique labels (SDR-06).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Mark {
    /// One discrete note per datum (`PMSon`).
    Tone,
    /// A continuous contour (`PMSon`).
    Glide,
    /// A grain cloud (`PMSon`, granular).
    Grain,
    /// The series as a waveform (Audification).
    Audify,
    /// A recorded sound per event (Auditory icon).
    Sample,
    /// An excitable data model (MBS), Stage D.
    Model,
    /// Tone, glide or audification chosen from density (SDR-20, experimental).
    Auto,
}

impl From<Mark> for MarkKind {
    fn from(mark: Mark) -> Self {
        match mark {
            Mark::Tone => Self::Tone,
            Mark::Glide => Self::Glide,
            Mark::Grain => Self::Grain,
            Mark::Audify => Self::Audify,
            Mark::Sample => Self::Sample,
            Mark::Model => Self::Model,
            Mark::Auto => Self::Auto,
        }
    }
}

/// The channels of a track. Which channels a mark accepts is checked by validation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Channels {
    /// Pitch (SDR-07..11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch: Option<Channel>,
    /// Note duration (A11Y-13).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<Channel>,
    /// Tap count, 1–8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taps: Option<Channel>,
    /// Loudness in relative dB; constant by default (SDR-23).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loudness: Option<Channel>,
    /// Stereo position, −1 to 1; never the sole carrier (A11Y-14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan: Option<Channel>,
    /// Timbre preset, or a nominal field (SDR-25).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timbre: Option<Channel>,
    /// Brightness, 0 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brightness: Option<Channel>,
    /// Modulation or event rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<Channel>,
    /// Grain density, grains/s.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density: Option<Channel>,
}

impl Channels {
    /// The channels that are present, with their spec names, in a fixed order.
    #[must_use]
    pub fn present(&self) -> Vec<(ChannelName, &Channel)> {
        [
            (ChannelName::Pitch, &self.pitch),
            (ChannelName::Duration, &self.duration),
            (ChannelName::Taps, &self.taps),
            (ChannelName::Loudness, &self.loudness),
            (ChannelName::Pan, &self.pan),
            (ChannelName::Timbre, &self.timbre),
            (ChannelName::Brightness, &self.brightness),
            (ChannelName::Rate, &self.rate),
            (ChannelName::Density, &self.density),
        ]
        .into_iter()
        .filter_map(|(name, channel)| channel.as_ref().map(|channel| (name, channel)))
        .collect()
    }
}

/// Channel names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[expect(missing_docs, reason = "the variant names are the channel names")]
pub enum ChannelName {
    Pitch,
    Duration,
    Taps,
    Loudness,
    Pan,
    Timbre,
    Brightness,
    Rate,
    Density,
}

impl ChannelName {
    /// The channel's key in the spec.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Pitch => "pitch",
            Self::Duration => "duration",
            Self::Taps => "taps",
            Self::Loudness => "loudness",
            Self::Pan => "pan",
            Self::Timbre => "timbre",
            Self::Brightness => "brightness",
            Self::Rate => "rate",
            Self::Density => "density",
        }
    }
}

/// One channel: either `field` (with mapping options) or a constant `value`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Channel {
    /// The data field mapped to this channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// A constant value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<ChannelValue>,
    /// How data values map onto the range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Scale>,
    /// The output range, low to high.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<ChannelRange>,
    /// The pitch space (pitch only; SDR-09).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space: Option<PitchSpace>,
    /// Mapping direction (SDR-12..15).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polarity: Option<Polarity>,
}

/// A scale from data values to the channel range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Scale {
    /// The scale type; default `linear`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub scale_type: Option<ScaleType>,
    /// The data domain; default `"auto"` (the data extent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,
    /// Round the domain to nice values; default `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nice: Option<bool>,
    /// Exponent for `pow`, and m for `psychophysical`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exponent: Option<f64>,
    /// Reference point for `psychophysical` (SDR-11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<ScaleReference>,
}

/// Scale types (04 §5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
#[allow(
    missing_docs,
    reason = "the variant names are the scale types of 04 §5.3"
)]
pub enum ScaleType {
    Linear,
    Log,
    Symlog,
    Sqrt,
    Pow,
    Psychophysical,
    Quantile,
    Band,
    Ordinal,
}

/// A scale domain: a keyword or `[min, max]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Domain {
    /// `"auto"` or `"zero-based"`.
    Keyword(DomainKeyword),
    /// An explicit `[min, max]`.
    Range([f64; 2]),
}

/// Domain keywords.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DomainKeyword {
    /// The data extent.
    Auto,
    /// From zero to the data extent.
    ZeroBased,
}

/// The reference point `f_ref` at data value `d_ref` of a psychophysical scale (SDR-11).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct ScaleReference {
    /// The data value `d_ref`.
    pub value: f64,
    /// The frequency `f_ref` in hertz.
    pub hz: f64,
}

/// Pitch spaces (SDR-09).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum PitchSpace {
    /// Semitones or cents (default).
    LogFrequency,
    /// Hertz.
    LinearFrequency,
    /// Mel.
    Mel,
    /// ERB-rate.
    Erb,
    /// Bark.
    Bark,
}

/// Mapping direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Polarity {
    /// From the field's concept (SDR-12..14).
    Auto,
    /// Higher value, higher pitch.
    Positive,
    /// Higher value, lower pitch.
    Negative,
}

/// Redundant encodings (A11Y-13).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Redundant {
    /// Tap count.
    Taps,
    /// Note duration.
    Duration,
}

/// A row filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Filter {
    /// The predicate.
    pub expr: String,
}

/// Audification speed (SDR-31): a speed-up factor or a target band.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Audify {
    /// The speed-up factor k.
    K(f64),
    /// The target audio band.
    Band(UnitRange),
}
