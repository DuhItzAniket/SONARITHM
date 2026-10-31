// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `experimental`: opt-in switches for sound designs that evidence does not yet back
//! (spec v0.4, RFC-0006, ADR-0027).
//!
//! Every switch is off unless the spec turns it on; templates, profiles and presets never
//! set one. A switch that is on is reported (`SNR-COMP-037`), named in the text
//! description and recorded in the render manifest. The block is separate from
//! `lint.experimental`, which only turns on experimental lint rules (RFC-0004 §4).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::json::JsonValue;

/// The key of the low-confidence timbre switch (RFC-0006 §3, SDR-43).
pub const LOW_CONFIDENCE_TIMBRE: &str = "lowConfidenceTimbre";

/// A switch this version knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Experiment {
    /// The key in the `experimental` block.
    pub key: &'static str,
    /// The engine that implements it.
    pub engine: &'static str,
    /// The design rules and hypotheses it rests on.
    pub evidence: &'static [&'static str],
    /// What it does, for reports and descriptions: "Experimental: {summary}."
    pub summary: &'static str,
}

/// The switches this version knows (RFC-0006 §1).
pub const EXPERIMENTS: &[Experiment] = &[Experiment {
    key: LOW_CONFIDENCE_TIMBRE,
    engine: "NUMISONA",
    evidence: &["SDR-43", "H-06"],
    summary: "low-confidence timbre is on. Estimated values sound darker, more so at lower confidence; speech still says which values are estimates",
}];

/// The switch `key`, when this version knows it.
#[must_use]
pub fn experiment(key: &str) -> Option<&'static Experiment> {
    EXPERIMENTS.iter().find(|experiment| experiment.key == key)
}

/// `experimental` (RFC-0006 §1): named switches, each `true` or `false`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Experimental {
    /// Estimated values play in a darker timbre (SDR-43, `INFER`; H-06). Default `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low_confidence_timbre: Option<bool>,
    /// Switches this version does not know: kept, warned about (`SNR-SPEC-047`) and
    /// ignored.
    #[serde(flatten)]
    pub other: BTreeMap<String, JsonValue>,
}

impl Experimental {
    /// The keys of the known switches that are on, sorted.
    #[must_use]
    pub fn on(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        if self.low_confidence_timbre == Some(true) {
            keys.push(LOW_CONFIDENCE_TIMBRE);
        }
        keys.sort_unstable();
        keys
    }

    /// The block as the canonical form keeps it (RFC-0006 §2): switches set to `false`
    /// dropped, and `None` when nothing is left, so `{}`, all-`false` and no block share
    /// one spec hash. Unknown switches are kept.
    #[must_use]
    pub fn canonical(&self) -> Option<Self> {
        let mut block = self.clone();
        if block.low_confidence_timbre == Some(false) {
            block.low_confidence_timbre = None;
        }
        block
            .other
            .retain(|_, value| *value != JsonValue::Bool(false));
        (block.low_confidence_timbre.is_some() || !block.other.is_empty()).then_some(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(json: &str) -> Experimental {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn known_switches_that_are_on_are_listed() {
        assert_eq!(
            block(r#"{"lowConfidenceTimbre": true}"#).on(),
            [LOW_CONFIDENCE_TIMBRE]
        );
        assert_eq!(
            block(r#"{"lowConfidenceTimbre": false}"#).on(),
            Vec::<&str>::new()
        );
        assert_eq!(block(r#"{"someday": true}"#).on(), Vec::<&str>::new());
        assert_eq!(
            experiment(LOW_CONFIDENCE_TIMBRE).unwrap().engine,
            "NUMISONA"
        );
    }

    #[test]
    fn the_canonical_block_drops_switches_that_are_off() {
        assert_eq!(block("{}").canonical(), None);
        assert_eq!(
            block(r#"{"lowConfidenceTimbre": false, "someday": false}"#).canonical(),
            None
        );
        let kept = block(r#"{"lowConfidenceTimbre": false, "someday": true}"#)
            .canonical()
            .unwrap();
        assert_eq!(kept.low_confidence_timbre, None);
        assert_eq!(kept.other.get("someday"), Some(&JsonValue::Bool(true)));
    }
}
