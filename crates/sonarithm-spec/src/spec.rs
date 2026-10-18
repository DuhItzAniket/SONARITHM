// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The top-level spec document (RFC-0001 §2).

use std::collections::BTreeMap;

use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use sonarithm_core::score::SpecVersion;

use crate::data::{DataSpec, Transform};
use crate::json::JsonValue;
use crate::sections::{Analysis, Context, Music, Output, Playback, Substrate};
use crate::track::TrackSpec;

/// The spec version this crate reads and writes.
pub const SPEC_VERSION: SpecVersion = SpecVersion::V0_1;

/// A SONARITHM sonification spec, version 0.1: data plus how it becomes sound (RFC-0001).
///
/// Optional keys stay unset when the author omits them, so the spec records what was
/// written; defaults are applied separately and reported as Info diagnostics. Unknown
/// top-level keys are kept in `extensions`, and validation accepts only `x-` keys there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(title = "SONARITHM spec", transform = top_level_keys)]
#[non_exhaustive]
pub struct Spec {
    /// The spec language version, `"0.1"`.
    #[schemars(with = "String", extend("pattern" = "^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)$"))]
    pub spec_version: SpecVersion,
    /// A title, spoken by the legend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Free text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The mode; default `analysis` (SDR-36).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<Mode>,
    /// The seed every random stream derives from; default 0 (SDR-02). At most 2^53 − 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub seed: Option<u64>,
    /// The data.
    pub data: DataSpec,
    /// Transforms, applied in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<Transform>,
    /// The temporal substrate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substrate: Option<Substrate>,
    /// The tracks; at least one.
    pub tracks: Vec<TrackSpec>,
    /// Context cues (AUDAXIS).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Detectors and descriptions (ECHOMETRIC).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<Analysis>,
    /// Musical presentation (AXONOTE), Presentation mode only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<Music>,
    /// Playback (SONOTRACE).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playback: Option<Playback>,
    /// Output settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Output>,
    /// Other top-level keys. Only `x-` vendor keys are valid; they are preserved and
    /// ignored by the compiler.
    #[serde(flatten)]
    pub extensions: BTreeMap<String, JsonValue>,
}

/// The mode (SDR-36).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// Analytic listening (default).
    Analysis,
    /// Musical presentation.
    Presentation,
    /// Peripheral monitoring.
    Monitoring,
}

impl From<Mode> for sonarithm_core::score::Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Analysis => Self::Analysis,
            Mode::Presentation => Self::Presentation,
            Mode::Monitoring => Self::Monitoring,
        }
    }
}

/// In the schema, only `x-` keys may appear beside the listed ones (RFC-0001 §1).
fn top_level_keys(schema: &mut Schema) {
    schema.insert("additionalProperties".to_owned(), false.into());
    schema.insert(
        "patternProperties".to_owned(),
        serde_json::json!({ "^x-": {} }),
    );
}
