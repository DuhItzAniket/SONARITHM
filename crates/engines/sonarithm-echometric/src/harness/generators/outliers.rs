// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GEN-011: labelled outliers (C05-T02; DS-010). A smooth series with Gaussian noise and a
//! known number of spikes at known rows, so a detector's precision and recall can be measured
//! against ground truth. It reproduces no study: it is a test set for ECHOMETRIC's `outliers`
//! detector, in the robust-z terms of Iglewicz and Hoaglin (1993).

use serde::{Deserialize, Serialize};
use sonarithm_core::math;

use super::{GeneratorError, Params, Rng, draw, invalid, round};
use crate::harness::stimulus::{Column, ColumnKind, GeneratorId, Note, Stimulus, Truth};

const ID: GeneratorId = GeneratorId::Outliers;

/// GEN-011's parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct OutlierParams {
    /// How many points, at least 20.
    #[serde(default = "default_points")]
    pub points: u32,
    /// How many spikes to add, fewer than a tenth of the points.
    #[serde(default = "default_outliers")]
    pub outliers: u32,
    /// Each spike's size, in noise standard deviations (above 3).
    #[serde(default = "default_magnitude")]
    pub magnitude: f64,
    /// The noise's standard deviation.
    #[serde(default = "default_noise")]
    pub noise: f64,
    /// The underlying sine's amplitude; 0 for a flat series.
    #[serde(default = "default_amplitude")]
    pub amplitude: f64,
    /// How many periods of the sine the series spans.
    #[serde(default = "default_periods")]
    pub periods: f64,
}

const fn default_points() -> u32 {
    200
}

const fn default_outliers() -> u32 {
    5
}

const fn default_magnitude() -> f64 {
    8.0
}

const fn default_noise() -> f64 {
    1.0
}

const fn default_amplitude() -> f64 {
    2.0
}

const fn default_periods() -> f64 {
    2.0
}

impl Default for OutlierParams {
    fn default() -> Self {
        Self {
            points: default_points(),
            outliers: default_outliers(),
            magnitude: default_magnitude(),
            noise: default_noise(),
            amplitude: default_amplitude(),
            periods: default_periods(),
        }
    }
}

/// GEN-011's ground truth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OutlierTruth {
    /// The rows that hold a spike, increasing.
    pub rows: Vec<usize>,
    /// Each spike's signed size, in noise standard deviations, by row.
    pub sizes: Vec<f64>,
}

fn check(params: &OutlierParams) -> Result<usize, GeneratorError> {
    if params.points < 20 {
        return Err(invalid(ID, "points", "at least 20"));
    }
    if params.outliers.saturating_mul(10) >= params.points {
        return Err(invalid(ID, "outliers", "fewer than a tenth of the points"));
    }
    if !params.magnitude.is_finite() || params.magnitude <= 3.0 {
        return Err(invalid(
            ID,
            "magnitude",
            "above 3 noise standard deviations",
        ));
    }
    if !params.noise.is_finite() || params.noise <= 0.0 {
        return Err(invalid(ID, "noise", "a positive standard deviation"));
    }
    if !params.amplitude.is_finite() || params.amplitude < 0.0 {
        return Err(invalid(ID, "amplitude", "zero or more"));
    }
    if !params.periods.is_finite() || params.periods < 0.0 {
        return Err(invalid(ID, "periods", "zero or more"));
    }
    Ok(params.points as usize)
}

pub(crate) fn generate(params: &OutlierParams, seed: u64) -> Result<Stimulus, GeneratorError> {
    let points = check(params)?;
    let mut rng = Rng::new(seed, ID);
    // Spikes go only where both neighbours are ordinary points, so no two spikes touch and
    // the first and last rows stay ordinary.
    let candidates: Vec<usize> = (1..points - 1).filter(|row| row % 3 == 1).collect();
    let mut rows = draw(&mut rng, &candidates, params.outliers as usize);
    rows.sort_unstable();
    let mut sizes = Vec::with_capacity(rows.len());
    let values: Vec<f64> = (0..points)
        .map(|row| {
            let phase = std::f64::consts::TAU * params.periods * row as f64 / points as f64;
            let base = params.amplitude * math::sin(phase);
            let noise = params.noise * rng.normal();
            let spike = if rows.binary_search(&row).is_ok() {
                let sign = if rng.below(2) == 0 { 1.0 } else { -1.0 };
                sizes.push(sign * params.magnitude);
                sign * params.magnitude * params.noise
            } else {
                0.0
            };
            round(base + noise + spike, 4)
        })
        .collect();
    let times: Vec<f64> = (0..points).map(|row| row as f64).collect();
    Ok(Stimulus {
        generator: ID,
        seed,
        params: Params::Outliers(params.clone()),
        columns: vec![
            Column::new("t", ColumnKind::Time, "Sample", times),
            Column::new("value", ColumnKind::Value, "Value", values),
        ],
        truth: Truth::Outliers(OutlierTruth { rows, sizes }),
        notes: vec![Note::Deviation {
            detail: "A labelled detector test set (DS-010), not a study's stimulus.".to_owned(),
        }],
    })
}
