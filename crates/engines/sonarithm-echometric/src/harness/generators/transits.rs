// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GEN-003: SNR-controlled transit series, after Tucker Brown, Harrison, Zanella &
//! Trayford (2022) (`tuckerbrown2022evaluating`, MNRAS 516(4):5674–5683; read from the
//! accepted version, arXiv:2209.04465v1).
//!
//! Listeners counted transit-like dips in evenly sampled light curves: a flat baseline
//! with box-shaped dips and Gaussian noise, made with astronify's `simulated_lc`.
//!
//! | Parameter | Published (arXiv v1 section) | GEN-003 |
//! |---|---|---|
//! | Length | `t_l` = 1000 (§2.1) | `points` = 1000 |
//! | Transit shape | Boxcar dips of set width and depth (§2.1) | Box dips |
//! | Width | `w_f` = 0.1 of the length (§2.1) | `width` = 0.1, so `N_t` = 100 points per transit |
//! | Depth | "Fractional transit depth" `t_d` = 25 (§2.1) | `depth` = 25% of `baseline` = 100, so 25 flux units (inferred from astronify's percent depth and default offset; Fig. 1 matches) |
//! | Noise | Gaussian (§2.1) | Gaussian, σ from the SNR |
//! | SNR | SNR = `t_d · √N_t / σ`; levels 3, 5, 7, 10, 30, 100 (§2.1) | `snr`, required; σ = `depth · √N_t / SNR` (250 / SNR by default) |
//! | Transits per curve | 0, 1 or 2, set through the period (§2.1) | `transits`, required; above 2 noted |
//! | Transit positions | Not stated (Fig. 1 shows one near 420–520, two near 150–250 and 640–740) | Seeded, non-overlapping, at least half a width from the ends and from each other |
//! | Sonification | Linear 440–880 Hz, one tone per point, 0.01 s spacing, 0.2 s notes (§2.2) | Not data: the gallery spec sets it |
//!
//! The ground truth reports the SNR the noise actually reached, from the sample standard
//! deviation of the drawn noise, next to the nominal one.

use serde::{Deserialize, Serialize};

use super::{GeneratorError, Params, Rng, composition, invalid, round, whole};
use crate::harness::stimulus::{Column, ColumnKind, GeneratorId, Note, Stimulus, Truth};

const ID: GeneratorId = GeneratorId::Transits;

/// The SNR levels Tucker Brown et al. (2022) tested (SDR-40, signal detection by SNR).
pub const STUDIED_SNR: [f64; 6] = [3.0, 5.0, 7.0, 10.0, 30.0, 100.0];

/// GEN-003 parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TransitParams {
    /// The signal-to-noise ratio, `t_d · √N_t / σ`; studied: 3, 5, 7, 10, 30, 100.
    pub snr: f64,
    /// The number of transits; studied: 0, 1, 2.
    pub transits: u32,
    /// The number of points (Tucker Brown et al. 2022: 1000).
    #[serde(default = "default_points")]
    pub points: u32,
    /// The transit width as a fraction of the length (2022: 0.1).
    #[serde(default = "default_width")]
    pub width: f64,
    /// The transit depth in percent of the baseline (2022: 25).
    #[serde(default = "default_depth")]
    pub depth: f64,
    /// The baseline flux (astronify's default offset, 100).
    #[serde(default = "default_baseline")]
    pub baseline: f64,
}

const fn default_points() -> u32 {
    1000
}

const fn default_width() -> f64 {
    0.1
}

const fn default_depth() -> f64 {
    25.0
}

const fn default_baseline() -> f64 {
    100.0
}

impl TransitParams {
    /// The published design for one light curve: `transits` dips at `snr`.
    #[must_use]
    pub const fn curve(snr: f64, transits: u32) -> Self {
        Self {
            snr,
            transits,
            points: default_points(),
            width: default_width(),
            depth: default_depth(),
            baseline: default_baseline(),
        }
    }
}

/// GEN-003 ground truth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TransitTruth {
    /// The number of transits: the answer to the counting task.
    pub transits: u32,
    /// The first row of each transit.
    pub starts: Vec<usize>,
    /// Points per transit, `N_t`.
    pub width_points: usize,
    /// The depth in flux units.
    pub depth_flux: f64,
    /// The noise standard deviation used, σ.
    pub sigma: f64,
    /// The nominal SNR.
    pub snr: f64,
    /// The SNR with the sample standard deviation of the drawn noise.
    pub snr_achieved: f64,
}

/// Generates a GEN-003 light curve.
pub(crate) fn generate(params: &TransitParams, seed: u64) -> Result<Stimulus, GeneratorError> {
    let (points, width) = check(params)?;
    let transits = params.transits as usize;
    let mut rng = Rng::new(seed, ID);
    let gap = width.div_ceil(2);
    // Free points split into transits + 1 gaps of at least half a width each.
    let gaps = composition(&mut rng, points - transits * width, transits + 1, gap);
    let mut starts = Vec::with_capacity(transits);
    let mut row = 0;
    for &free in gaps.iter().take(transits) {
        row += free;
        starts.push(row);
        row += width;
    }
    let depth_flux = params.baseline * params.depth / 100.0;
    let sigma = depth_flux * (width as f64).sqrt() / params.snr;
    let noise: Vec<f64> = (0..points).map(|_| sigma * rng.normal()).collect();
    let mut flux = Vec::with_capacity(points);
    for (index, &jitter) in noise.iter().enumerate() {
        let inside = starts
            .iter()
            .any(|&start| (start..start + width).contains(&index));
        let level = if inside {
            params.baseline - depth_flux
        } else {
            params.baseline
        };
        flux.push(round(level + jitter, 4));
    }
    let mean = noise.iter().sum::<f64>() / points as f64;
    let variance = noise.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (points - 1) as f64;
    let snr_achieved = round(depth_flux * (width as f64).sqrt() / variance.sqrt(), 4);
    let times: Vec<f64> = (0..points).map(|i| i as f64).collect();
    let truth = TransitTruth {
        transits: params.transits,
        starts,
        width_points: width,
        depth_flux,
        sigma,
        snr: params.snr,
        snr_achieved,
    };
    Ok(Stimulus {
        generator: ID,
        seed,
        params: Params::Transits(params.clone()),
        columns: vec![
            Column::new("t", ColumnKind::Time, "Time", times),
            Column::new("flux", ColumnKind::Value, "Brightness", flux),
        ],
        truth: Truth::Transits(truth),
        notes: notes(params),
    })
}

/// Checks the parameters; returns the number of points and the transit width in points.
fn check(params: &TransitParams) -> Result<(usize, usize), GeneratorError> {
    if !(params.snr.is_finite() && params.snr > 0.0) {
        return Err(invalid(ID, "snr", "must be a positive number"));
    }
    if !(params.depth.is_finite() && params.depth > 0.0 && params.depth <= 100.0) {
        return Err(invalid(ID, "depth", "must be a percentage in (0, 100]"));
    }
    if !(params.baseline.is_finite() && params.baseline > 0.0) {
        return Err(invalid(ID, "baseline", "must be a positive flux"));
    }
    if !(params.width.is_finite() && params.width > 0.0 && params.width < 1.0) {
        return Err(invalid(
            ID,
            "width",
            "must be a fraction of the length in (0, 1)",
        ));
    }
    if !(10..=1_000_000).contains(&params.points) {
        return Err(invalid(ID, "points", "must be between 10 and 1000000"));
    }
    let points = params.points as usize;
    let width = whole((params.width * points as f64).round());
    if width == 0 {
        return Err(invalid(
            ID,
            "width",
            "covers no whole point; widen it or add points",
        ));
    }
    let transits = params.transits as usize;
    let needed = transits * width + (transits + 1) * width.div_ceil(2);
    if transits > 20 || needed > points {
        return Err(invalid(
            ID,
            "transits",
            format!(
                "{transits} transits of {width} points do not fit in {points} points with half a width around each"
            ),
        ));
    }
    Ok((points, width))
}

fn notes(params: &TransitParams) -> Vec<Note> {
    let mut notes = vec![Note::Deviation {
        detail: "Tucker Brown et al. (2022) do not publish transit positions; they are drawn from the seed".to_owned(),
    }];
    if !STUDIED_SNR.contains(&params.snr) {
        notes.push(Note::Deviation {
            detail: format!(
                "SNR {} is not one of the studied levels 3, 5, 7, 10, 30, 100",
                params.snr
            ),
        });
    }
    if params.transits > 2 {
        notes.push(Note::Deviation {
            detail: format!(
                "{} transits; no studied curve had more than 2",
                params.transits
            ),
        });
    }
    notes
}
