// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GEN-012: labelled level shifts and periods (C05-T03; DS-010). Piecewise-constant levels
//! with a known number of shifts at known rows, an optional sine of a known period, and
//! Gaussian noise, so the `changepoints` and `periodicity` detectors' precision and recall can
//! be measured against ground truth. It reproduces no study: it is a detector test set.

use serde::{Deserialize, Serialize};
use sonarithm_core::math;

use super::{GeneratorError, Params, Rng, invalid, round};
use crate::harness::stimulus::{Column, ColumnKind, GeneratorId, Note, Stimulus, Truth};

const ID: GeneratorId = GeneratorId::Shifts;

/// GEN-012's parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ShiftParams {
    /// How many points, at least 40.
    #[serde(default = "default_points")]
    pub points: u32,
    /// How many level shifts.
    #[serde(default = "default_shifts")]
    pub shifts: u32,
    /// The fewest points between two shifts, and between a shift and either end (at least 5).
    #[serde(default = "default_spacing")]
    pub spacing: u32,
    /// Each shift's size, in noise standard deviations (above 1); its sign is random.
    #[serde(default = "default_magnitude")]
    pub magnitude: f64,
    /// The noise's standard deviation.
    #[serde(default = "default_noise")]
    pub noise: f64,
    /// The period of an added sine, in points; none by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<u32>,
    /// The sine's amplitude, in noise standard deviations.
    #[serde(default = "default_amplitude")]
    pub amplitude: f64,
}

const fn default_points() -> u32 {
    300
}

const fn default_shifts() -> u32 {
    3
}

const fn default_spacing() -> u32 {
    30
}

const fn default_magnitude() -> f64 {
    6.0
}

const fn default_noise() -> f64 {
    1.0
}

const fn default_amplitude() -> f64 {
    3.0
}

impl Default for ShiftParams {
    fn default() -> Self {
        Self {
            points: default_points(),
            shifts: default_shifts(),
            spacing: default_spacing(),
            magnitude: default_magnitude(),
            noise: default_noise(),
            period: None,
            amplitude: default_amplitude(),
        }
    }
}

/// GEN-012's ground truth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ShiftTruth {
    /// The first row of each new level, increasing.
    pub change_rows: Vec<usize>,
    /// The sine's period in points, if one was added.
    pub period: Option<u32>,
}

fn check(params: &ShiftParams) -> Result<usize, GeneratorError> {
    if params.points < 40 {
        return Err(invalid(ID, "points", "at least 40"));
    }
    if params.spacing < 5 {
        return Err(invalid(ID, "spacing", "at least 5"));
    }
    let needed = u64::from(params.shifts + 1) * u64::from(params.spacing);
    if needed > u64::from(params.points) {
        return Err(invalid(
            ID,
            "shifts",
            "too many for the points and spacing: (shifts + 1) × spacing must fit",
        ));
    }
    if !params.magnitude.is_finite() || params.magnitude <= 1.0 {
        return Err(invalid(ID, "magnitude", "above 1 noise standard deviation"));
    }
    if !params.noise.is_finite() || params.noise <= 0.0 {
        return Err(invalid(ID, "noise", "a positive standard deviation"));
    }
    if let Some(period) = params.period {
        if period < 4 || period.saturating_mul(4) > params.points {
            return Err(invalid(ID, "period", "4 or more points, with at least 4 periods"));
        }
    }
    if !params.amplitude.is_finite() || params.amplitude < 0.0 {
        return Err(invalid(ID, "amplitude", "zero or more"));
    }
    Ok(params.points as usize)
}

/// `count` change rows in `[spacing, points − spacing]`, at least `spacing` apart.
fn change_rows(rng: &mut Rng, points: usize, count: usize, spacing: usize) -> Vec<usize> {
    // Lay out the free room left after the minimum gaps, then place the shifts in it: a
    // uniform draw of `count` sorted offsets in the slack, each pushed by its gaps.
    let slack = points - (count + 1) * spacing;
    let mut offsets: Vec<usize> = (0..count).map(|_| rng.below(slack + 1)).collect();
    offsets.sort_unstable();
    offsets
        .iter()
        .enumerate()
        .map(|(index, offset)| offset + (index + 1) * spacing)
        .collect()
}

pub(crate) fn generate(params: &ShiftParams, seed: u64) -> Result<Stimulus, GeneratorError> {
    let points = check(params)?;
    let mut rng = Rng::new(seed, ID);
    let rows = change_rows(&mut rng, points, params.shifts as usize, params.spacing as usize);
    let mut level = 0.0;
    let mut next = rows.iter().peekable();
    let values: Vec<f64> = (0..points)
        .map(|row| {
            if next.peek().is_some_and(|&&at| at == row) {
                next.next();
                let sign = if rng.below(2) == 0 { 1.0 } else { -1.0 };
                level += sign * params.magnitude * params.noise;
            }
            let wave = params.period.map_or(0.0, |period| {
                let phase = std::f64::consts::TAU * row as f64 / f64::from(period);
                params.amplitude * params.noise * math::sin(phase)
            });
            round(level + wave + params.noise * rng.normal(), 4)
        })
        .collect();
    let times: Vec<f64> = (0..points).map(|row| row as f64).collect();
    Ok(Stimulus {
        generator: ID,
        seed,
        params: Params::Shifts(params.clone()),
        columns: vec![
            Column::new("t", ColumnKind::Time, "Sample", times),
            Column::new("value", ColumnKind::Value, "Value", values),
        ],
        truth: Truth::Shifts(ShiftTruth {
            change_rows: rows,
            period: params.period,
        }),
        notes: vec![Note::Deviation {
            detail: "A labelled detector test set (DS-010), not a study's stimulus.".to_owned(),
        }],
    })
}
