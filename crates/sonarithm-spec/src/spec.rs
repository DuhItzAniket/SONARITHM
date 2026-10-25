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
use crate::v02::{Instrument, Lint, TemplateRef};

/// The spec version this crate writes, and the newest it reads.
pub const SPEC_VERSION: SpecVersion = SpecVersion::V0_2;

/// The versions this crate reads as they are, without migrating them: 0.1 reads directly
/// because the 0.1 → 0.2 migration is the identity (RFC-0004 §9).
pub const READABLE_VERSIONS: &[SpecVersion] = &[SpecVersion::V0_1, SpecVersion::V0_2];

/// A SONARITHM sonification spec: data plus how it becomes sound (RFC-0001, RFC-0004).
///
/// Optional keys stay unset when the author omits them, so the spec records what was
/// written; defaults are applied separately and reported as Info diagnostics. Unknown
/// top-level keys are kept in `extensions`, and validation accepts only `x-` keys there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(title = "SONARITHM spec", transform = top_level_keys)]
#[non_exhaustive]
pub struct Spec {
    /// The spec language version, `"0.2"` (or `"0.1"`, read as it is).
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
    /// The tracks; at least one, unless `template` gives them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
    /// A template that designs the tracks and other sections (v0.2, RFC-0004 §2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<TemplateRef>,
    /// Named instrument definitions (v0.2, RFC-0004 §3).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub instruments: BTreeMap<String, Instrument>,
    /// Lint rule levels and suppressions (v0.2, RFC-0004 §4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lint: Option<Lint>,
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
    // A spec has tracks, or a template that designs them (RFC-0004 §1).
    schema.insert(
        "anyOf".to_owned(),
        serde_json::json!([{ "required": ["tracks"] }, { "required": ["template"] }]),
    );
    schema.insert(
        "patternProperties".to_owned(),
        serde_json::json!({ "^x-": {} }),
    );
}
