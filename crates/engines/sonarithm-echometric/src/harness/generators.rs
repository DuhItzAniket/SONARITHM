// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stimulus generators GEN-001 to GEN-004 (docs/research-paper/datasets.md).
//!
//! Each generator re-implements a published stimulus design from the paper's description,
//! never from the authors' code. Its module documents the published parameters next to
//! SONARITHM's and justifies every difference; a difference that matters for a replication
//! is also attached to the stimulus as a [`Note`](super::Note).
//!
//! | ID | Module | Published basis |
//! |---|---|---|
//! | GEN-001 | [`reversals`] | Nees & Walker 2008 (`nees2008density`) |
//! | GEN-002 | [`prices`] | Smith & Walker 2005 (`smith2005effects`) |
//! | GEN-003 | [`transits`] | Tucker Brown et al. 2022 (`tuckerbrown2022evaluating`) |
//! | GEN-004 | [`correlation`] | Elmquist et al. 2024 (`elmquist2024parallel`) |
//!
//! **Determinism (ADR-0013, SDR-02).** A generator draws from one `ChaCha8` stream keyed
//! by the seed and its ID ([`TrackRng`]), in a fixed order, and computes with
//! [`sonarithm_core::math`] and IEEE-exact operations only. The same parameters, seed and
//! [generator version](super::GeneratorId::version) give the same bytes on every platform.

pub mod correlation;
pub mod prices;
pub mod reversals;
pub mod transits;

use serde::{Deserialize, Serialize};
use sonarithm_core::engine::TrackRng;
use sonarithm_core::ids::TrackId;
use sonarithm_core::math;

use super::stimulus::{GeneratorId, Stimulus};

/// The parameters of one generator, tagged with its ID, as written in gallery items and
/// study protocols: `{ "generator": "GEN-001", "rate": 2, ... }`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "generator")]
pub enum Params {
    /// GEN-001.
    #[serde(rename = "GEN-001")]
    TrendReversals(reversals::ReversalParams),
    /// GEN-002.
    #[serde(rename = "GEN-002")]
    PriceSeries(prices::PriceParams),
    /// GEN-003.
    #[serde(rename = "GEN-003")]
    Transits(transits::TransitParams),
    /// GEN-004.
    #[serde(rename = "GEN-004")]
    CorrelationTriplets(correlation::CorrelationParams),
}

impl Params {
    /// The generator these parameters belong to.
    #[must_use]
    pub const fn generator(&self) -> GeneratorId {
        match self {
            Self::TrendReversals(_) => GeneratorId::TrendReversals,
            Self::PriceSeries(_) => GeneratorId::PriceSeries,
            Self::Transits(_) => GeneratorId::Transits,
            Self::CorrelationTriplets(_) => GeneratorId::CorrelationTriplets,
        }
    }
}

/// Why a generator refused its parameters.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum GeneratorError {
    /// A parameter is out of range or inconsistent with another.
    #[error("{generator:?} parameter `{parameter}`: {reason}")]
    InvalidParameter {
        /// The generator.
        generator: GeneratorId,
        /// The parameter's name, as written in a protocol.
        parameter: &'static str,
        /// What is wrong, and the accepted range.
        reason: String,
    },
}

/// Generates the stimulus that `params` and `seed` describe.
///
/// # Errors
///
/// [`GeneratorError::InvalidParameter`] when a parameter is out of range.
pub fn generate(params: &Params, seed: u64) -> Result<Stimulus, GeneratorError> {
    match params {
        Params::TrendReversals(params) => reversals::generate(params, seed),
        Params::PriceSeries(params) => prices::generate(params, seed),
        Params::Transits(params) => transits::generate(params, seed),
        Params::CorrelationTriplets(params) => correlation::generate(params, seed),
    }
}

/// The seeded stream of one generator.
pub(crate) struct Rng(TrackRng);

impl Rng {
    /// The stream for `generator` under `seed`; the generator ID keys it, so two generators
    /// with the same seed draw independent numbers.
    pub(crate) fn new(seed: u64, generator: GeneratorId) -> Self {
        Self(TrackRng::new(seed, &TrackId::new(generator.as_str())))
    }

    /// A uniform number in [0, 1).
    pub(crate) fn uniform(&mut self) -> f64 {
        self.0.next_f64()
    }

    /// A uniform integer in `0..n`, without modulo bias; 0 when `n` is 0.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        let Ok(n) = u64::try_from(n) else {
            return 0;
        };
        if n == 0 {
            return 0;
        }
        // Rejection sampling: accept only draws below the largest multiple of n.
        let zone = u64::MAX - u64::MAX % n;
        loop {
            let draw = self.0.next_u64();
            if draw < zone {
                return usize::try_from(draw % n).unwrap_or(0);
            }
        }
    }

    /// A standard normal number (Box–Muller, one value per pair of uniforms).
    pub(crate) fn normal(&mut self) -> f64 {
        // 1 − U lies in (0, 1], so the logarithm is finite.
        let radius = (-2.0 * math::ln(1.0 - self.uniform())).sqrt();
        radius * math::cos(std::f64::consts::TAU * self.uniform())
    }

    /// Shuffles `items` in place (Fisher–Yates).
    pub(crate) fn shuffle<T>(&mut self, items: &mut [T]) {
        for last in (1..items.len()).rev() {
            let pick = self.below(last + 1);
            items.swap(last, pick);
        }
    }
}

/// Draws `count` items without replacement, in the seeded order of the draw.
pub(crate) fn draw(rng: &mut Rng, items: &[usize], count: usize) -> Vec<usize> {
    let mut pool = items.to_vec();
    let count = count.min(pool.len());
    for index in 0..count {
        let pick = index + rng.below(pool.len() - index);
        pool.swap(index, pick);
    }
    pool.truncate(count);
    pool
}

/// A uniform random composition of `steps` into `parts` lengths of at least `min` each
/// (stars and bars over the steps left after the minimums).
pub(crate) fn composition(rng: &mut Rng, steps: usize, parts: usize, min: usize) -> Vec<usize> {
    let extra = steps - parts * min;
    let slots = extra + parts - 1;
    let mut bars = draw(rng, &(0..slots).collect::<Vec<_>>(), parts - 1);
    bars.sort_unstable();
    let mut lengths = Vec::with_capacity(parts);
    let mut placed = 0;
    for (index, &bar) in bars.iter().enumerate() {
        // The stars before this bar are its slot minus the bars before it.
        let stars = bar - index - placed;
        lengths.push(min + stars);
        placed += stars;
    }
    lengths.push(min + extra - placed);
    lengths
}

/// Converts a whole, non-negative number below 2^53 to an index; callers check the range
/// first.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "callers pass whole numbers in [0, 2^53), which convert exactly"
)]
pub(crate) const fn whole(value: f64) -> usize {
    value as usize
}

/// Rounds to `digits` decimals with IEEE-exact operations, so the result is the same on
/// every platform.
pub(crate) fn round(value: f64, digits: i32) -> f64 {
    let scale = math::pow(10.0, f64::from(digits));
    (value * scale).round() / scale
}

/// The error for parameter `parameter` of `generator`.
pub(crate) fn invalid(
    generator: GeneratorId,
    parameter: &'static str,
    reason: impl Into<String>,
) -> GeneratorError {
    GeneratorError::InvalidParameter {
        generator,
        parameter,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_stays_in_range_and_covers_it() {
        let mut rng = Rng::new(7, GeneratorId::TrendReversals);
        let mut seen = [false; 5];
        for _ in 0..200 {
            let draw = rng.below(5);
            assert!(draw < 5);
            seen[draw] = true;
        }
        assert!(seen.iter().all(|&hit| hit));
        assert_eq!(rng.below(0), 0);
    }

    #[test]
    fn normal_draws_have_unit_scale() {
        let mut rng = Rng::new(7, GeneratorId::Transits);
        let draws: Vec<f64> = (0..20_000).map(|_| rng.normal()).collect();
        let n = draws.len() as f64;
        let mean = draws.iter().sum::<f64>() / n;
        let var = draws.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n;
        assert!(mean.abs() < 0.03, "mean {mean}");
        assert!((var - 1.0).abs() < 0.05, "variance {var}");
    }

    #[test]
    fn streams_differ_between_generators_with_one_seed() {
        let mut a = Rng::new(1, GeneratorId::TrendReversals);
        let mut b = Rng::new(1, GeneratorId::PriceSeries);
        assert_ne!(a.uniform().to_bits(), b.uniform().to_bits());
    }

    #[test]
    fn compositions_sum_to_the_steps_and_respect_the_minimum() {
        let mut rng = Rng::new(5, GeneratorId::TrendReversals);
        for _ in 0..200 {
            let lengths = composition(&mut rng, 40, 4, 3);
            assert_eq!(lengths.len(), 4);
            assert_eq!(lengths.iter().sum::<usize>(), 40);
            assert!(lengths.iter().all(|&length| length >= 3));
        }
        assert_eq!(composition(&mut rng, 10, 1, 2), vec![10]);
    }

    #[test]
    fn draws_are_distinct_and_capped() {
        let mut rng = Rng::new(9, GeneratorId::Transits);
        let picked = draw(&mut rng, &[3, 5, 7, 9], 10);
        assert_eq!(picked.len(), 4);
        let mut sorted = picked.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![3, 5, 7, 9]);
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut rng = Rng::new(3, GeneratorId::CorrelationTriplets);
        let mut items: Vec<usize> = (0..10).collect();
        rng.shuffle(&mut items);
        let mut sorted = items.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    }
}
