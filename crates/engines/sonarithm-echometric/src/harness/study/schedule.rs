// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Trial sequencing: a protocol and an order seed give one fixed list of trials.
//!
//! Each block draws from its own seeded stream, keyed by the protocol seed, the order seed
//! and the block ID: first the shuffle (for `order = "shuffle"`), then one stimulus seed
//! per trial. Two sessions with the same order seed get the same trials in the same order;
//! a different order seed counterbalances (EXP-000 §3).

use serde::{Deserialize, Serialize};
use sonarithm_core::engine::TrackRng;
use sonarithm_core::ids::TrackId;

use super::protocol::{Order, Protocol};
use crate::harness::generators::{GeneratorError, Params, generate};
use crate::harness::stimulus::Stimulus;

/// One trial of a session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Trial {
    /// The position in the session, from 0.
    pub index: usize,
    /// The block ID.
    pub block: String,
    /// The position in the block, from 0.
    pub block_trial: usize,
    /// Whether the block is practice.
    pub practice: bool,
    /// Whether the listener hears the correct answer after responding.
    pub feedback: bool,
    /// The condition ID.
    pub condition: String,
    /// The stimulus seed.
    pub stimulus_seed: u64,
}

/// The trials of one session, in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    /// The order seed the schedule was built with.
    pub order_seed: u64,
    /// The trials.
    pub trials: Vec<Trial>,
}

impl Schedule {
    /// Builds the schedule of `protocol` for `order_seed`.
    #[must_use]
    pub fn build(protocol: &Protocol, order_seed: u64) -> Self {
        let mut trials = Vec::new();
        for block in &protocol.blocks {
            let mut rng = TrackRng::new(
                protocol.seed ^ order_seed.rotate_left(32),
                &TrackId::new(format!("study/{}/{}", protocol.exp_id, block.id)),
            );
            let mut conditions: Vec<&str> = Vec::new();
            for condition in &block.conditions {
                for _ in 0..block.repeats {
                    conditions.push(condition.as_str());
                }
            }
            if block.order == Order::Shuffle {
                for last in (1..conditions.len()).rev() {
                    let pick = usize::try_from(rng.next_u64() % (last as u64 + 1)).unwrap_or(0);
                    conditions.swap(last, pick);
                }
            }
            for (block_trial, condition) in conditions.into_iter().enumerate() {
                trials.push(Trial {
                    index: trials.len(),
                    block: block.id.clone(),
                    block_trial,
                    practice: block.practice,
                    feedback: block.feedback,
                    condition: condition.to_owned(),
                    stimulus_seed: rng.next_u64(),
                });
            }
        }
        Self { order_seed, trials }
    }

    /// The number of trials.
    #[must_use]
    pub fn len(&self) -> usize {
        self.trials.len()
    }

    /// Whether the schedule has no trials.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.trials.is_empty()
    }
}

/// The stimulus parameters of `trial`'s block.
#[must_use]
pub fn params_of<'p>(protocol: &'p Protocol, trial: &Trial) -> Option<&'p Params> {
    protocol
        .blocks
        .iter()
        .find(|block| block.id == trial.block)
        .map(|block| &block.stimulus)
}

/// Generates `trial`'s stimulus.
///
/// # Errors
///
/// A [`GeneratorError`] if the block's parameters are invalid; [`Protocol::check`] reports
/// that before a session starts.
pub fn stimulus_of(protocol: &Protocol, trial: &Trial) -> Result<Option<Stimulus>, GeneratorError> {
    params_of(protocol, trial)
        .map(|params| generate(params, trial.stimulus_seed))
        .transpose()
}
