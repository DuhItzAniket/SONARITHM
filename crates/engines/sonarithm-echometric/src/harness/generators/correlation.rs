// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GEN-004: correlation triplets, after Elmquist et al. (2024) (`elmquist2024parallel`,
//! Personal and Ubiquitous Computing 28(5):657–676, "Parallel Chords").
//!
//! The study's task is a three-alternative forced choice: which of axes B, C and D
//! correlates most strongly with axis A? One of the three shares A's noise level; the
//! other two have extra noise Δσ, and a staircase makes Δσ smaller as the listener
//! succeeds.
//!
//! | Parameter | Published (section, page) | GEN-004 |
//! |---|---|---|
//! | Items | 300 polylines (§5.2, p. 664) | `items` = 300 |
//! | Equations | v1, v2 ~ N(i/300, σ²); v3, v4 ~ N(i/300, (σ + Δσ)²); i = 0..299 (Eqs. 1–4, p. 665) | The same, with `i / items` |
//! | Reference noise | σ = 0.07 (p. 665) | `sigma` = 0.07 |
//! | Added noise | 20 levels from 20% down to 0.2% of the range; the equation legend says "(0.02 − 20)", which contradicts the prose (p. 665) | `delta`, required, in range units (0.02 = 2%); outside 0.002–0.2 noted |
//! | Axes | v1 on A; v2–v4 on B–D at random (p. 665) | Seeded assignment; the truth names the axis with v2 |
//! | Range | Values outside [0, 1] redrawn with the same mean; data normalized to a common range last (p. 665) | Redrawn; then each axis min–max scaled onto [0, 1] (the normalization method is not stated) |
//! | Sign | Positive and negative linear correlations; negative datasets invert the selectable axes (§4, §5.2) | `sign`: `positive` (default) or `negative`, which inverts B, C and D |
//! | Target strength | r ≈ 0.95 for the correct axis (§5.2) | Reported per stimulus in the truth |
//! | Regeneration | A staircase's datasets were regenerated when r did not fall monotonically with difficulty (p. 665) | A triplet is redrawn, up to 100 times, until the target has the strongest correlation |
//! | Sonification | Onset from A, pitch MIDI 55–91, mallet timbre, 1–2 s (§3.3) | Not data: the gallery spec sets it |

use serde::{Deserialize, Serialize};

use super::{GeneratorError, Params, Rng, invalid, round};
use crate::harness::stimulus::{Column, ColumnKind, GeneratorId, Note, Stimulus, Truth};

const ID: GeneratorId = GeneratorId::CorrelationTriplets;

/// The added-noise range Elmquist et al. (2024) used, in range units (0.2% to 20%).
pub const STUDIED_DELTA: [f64; 2] = [0.002, 0.2];

/// The axes a listener chooses from.
pub const CHOICES: [&str; 3] = ["b", "c", "d"];

/// The sign of the correlations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sign {
    /// B, C and D rise with A.
    #[default]
    Positive,
    /// B, C and D fall as A rises.
    Negative,
}

/// GEN-004 parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CorrelationParams {
    /// The extra noise Δσ on the two distractor axes, in range units; studied: 0.002 to
    /// 0.2.
    pub delta: f64,
    /// The sign of the correlations.
    #[serde(default)]
    pub sign: Sign,
    /// Items per axis (Elmquist et al. 2024: 300).
    #[serde(default = "default_items")]
    pub items: u32,
    /// The reference noise σ (2024: 0.07).
    #[serde(default = "default_sigma")]
    pub sigma: f64,
}

const fn default_items() -> u32 {
    300
}

const fn default_sigma() -> f64 {
    0.07
}

impl CorrelationParams {
    /// The published design at added noise `delta`.
    #[must_use]
    pub const fn level(delta: f64, sign: Sign) -> Self {
        Self {
            delta,
            sign,
            items: default_items(),
            sigma: default_sigma(),
        }
    }
}

/// GEN-004 ground truth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CorrelationTruth {
    /// Which of B, C, D (0, 1, 2) holds the target: the correct answer.
    pub strongest: usize,
    /// Pearson r of A with B, C and D, as written (rounded to 4 decimals).
    pub correlations: [f64; 3],
    /// How many triplets were drawn until the target was the strongest.
    pub attempts: u32,
}

/// The most triplets drawn before giving up.
const MAX_ATTEMPTS: u32 = 100;

/// Generates a GEN-004 triplet.
pub(crate) fn generate(params: &CorrelationParams, seed: u64) -> Result<Stimulus, GeneratorError> {
    check(params)?;
    let mut rng = Rng::new(seed, ID);
    let items = params.items as usize;
    for attempt in 1..=MAX_ATTEMPTS {
        let a = axis(&mut rng, items, params.sigma)?;
        let target = axis(&mut rng, items, params.sigma)?;
        let first = axis(&mut rng, items, params.sigma + params.delta)?;
        let second = axis(&mut rng, items, params.sigma + params.delta)?;
        let mut order = [0_usize, 1, 2];
        rng.shuffle(&mut order);
        let pool = [target, first, second];
        let mut axes: Vec<Vec<f64>> = order.iter().map(|&index| pool[index].clone()).collect();
        let strongest = order.iter().position(|&index| index == 0).unwrap_or(0);
        let a = normalize(&a);
        for values in &mut axes {
            let scaled = normalize(values);
            *values = match params.sign {
                Sign::Positive => scaled,
                Sign::Negative => scaled.iter().map(|v| round(1.0 - v, 4)).collect(),
            };
        }
        let correlations = [
            round(pearson(&a, &axes[0]), 4),
            round(pearson(&a, &axes[1]), 4),
            round(pearson(&a, &axes[2]), 4),
        ];
        let target_r = correlations[strongest].abs();
        let clear = correlations
            .iter()
            .enumerate()
            .all(|(index, r)| index == strongest || r.abs() < target_r);
        if !clear {
            continue;
        }
        let [b, c, d]: [Vec<f64>; 3] = match axes.try_into() {
            Ok(axes) => axes,
            Err(_) => break,
        };
        let index: Vec<f64> = (0..items).map(|i| i as f64).collect();
        return Ok(Stimulus {
            generator: ID,
            seed,
            params: Params::CorrelationTriplets(params.clone()),
            columns: vec![
                Column::new("i", ColumnKind::Time, "Item", index),
                Column::new("a", ColumnKind::Value, "Axis A", a),
                Column::new("b", ColumnKind::Value, "Axis B", b),
                Column::new("c", ColumnKind::Value, "Axis C", c),
                Column::new("d", ColumnKind::Value, "Axis D", d),
            ],
            truth: Truth::Correlations(CorrelationTruth {
                strongest,
                correlations,
                attempts: attempt,
            }),
            notes: notes(params),
        });
    }
    Err(invalid(
        ID,
        "delta",
        format!(
            "no triplet in {MAX_ATTEMPTS} draws had the target as the strongest correlation; increase delta"
        ),
    ))
}

fn check(params: &CorrelationParams) -> Result<(), GeneratorError> {
    if !(params.delta.is_finite() && params.delta > 0.0 && params.delta <= 1.0) {
        return Err(invalid(
            ID,
            "delta",
            "must be in (0, 1], in range units (0.02 = 2%)",
        ));
    }
    if !(params.sigma.is_finite() && params.sigma > 0.0 && params.sigma + params.delta <= 1.0) {
        return Err(invalid(
            ID,
            "sigma",
            "must be positive, with sigma + delta at most 1",
        ));
    }
    if !(10..=100_000).contains(&params.items) {
        return Err(invalid(ID, "items", "must be between 10 and 100000"));
    }
    Ok(())
}

fn notes(params: &CorrelationParams) -> Vec<Note> {
    let mut notes = vec![Note::Deviation {
        detail: "Elmquist et al. (2024) do not state how data were normalized; each axis is min-max scaled onto [0, 1]".to_owned(),
    }];
    let [low, high] = STUDIED_DELTA;
    if params.delta < low || params.delta > high {
        notes.push(Note::Deviation {
            detail: format!(
                "delta {} is outside the studied 0.002–0.2 (0.2% to 20%)",
                params.delta
            ),
        });
    }
    notes
}

/// One axis: N(i / items, sd²) per item, redrawn while outside [0, 1].
fn axis(rng: &mut Rng, items: usize, sd: f64) -> Result<Vec<f64>, GeneratorError> {
    let mut values = Vec::with_capacity(items);
    for i in 0..items {
        let mean = i as f64 / items as f64;
        let mut drawn = None;
        for _ in 0..1000 {
            let value = mean + sd * rng.normal();
            if (0.0..=1.0).contains(&value) {
                drawn = Some(value);
                break;
            }
        }
        values.push(drawn.ok_or_else(|| {
            invalid(
                ID,
                "sigma",
                "values keep falling outside [0, 1]; lower sigma or delta",
            )
        })?);
    }
    Ok(values)
}

/// Min–max scales onto [0, 1] and rounds to 4 decimals.
fn normalize(values: &[f64]) -> Vec<f64> {
    let low = values.iter().copied().fold(f64::INFINITY, f64::min);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = high - low;
    values
        .iter()
        .map(|v| round(if span > 0.0 { (v - low) / span } else { 0.0 }, 4))
        .collect()
}

/// Pearson's r, summed in row order (deterministic).
#[must_use]
pub fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len()) as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        let (dx, dy) = (a - mean_x, b - mean_y);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx == 0.0 || syy == 0.0 {
        return 0.0;
    }
    sxy / (sxx * syy).sqrt()
}
