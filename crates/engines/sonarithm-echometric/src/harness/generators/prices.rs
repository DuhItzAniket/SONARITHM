// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GEN-002: price series with auditory context cues, after Smith & Walker (2005)
//! (`smith2005effects`, Applied Cognitive Psychology 19(8):1065–1087).
//!
//! Both studies used one series: a stock over a ten-hour trading day, two values per
//! hour, pseudo-sinusoidal, opening at $50, rising to a daily maximum of $84, falling to
//! a minimum of $10 and closing at the opening price (p. 1069). The paper does not
//! tabulate the values in between.
//!
//! | Parameter | Published (page) | GEN-002 |
//! |---|---|---|
//! | Series | One; $50 open → $84 max → $10 min → $50 close (p. 1069) | `series = "reconstructed"` (default) rebuilds it; `"random"` draws new series with the same anchors |
//! | Values | Two per hour; not tabulated. Fig. 2 plots the hourly answers within about ±$2 (p. 1074) | Hourly values read from Fig. 2: 50, 65, 80, 76, 61, 44, 28, 10, 23, 40, 50; half hours interpolated, except the $84 maximum at 10:30 |
//! | Tones | Twenty 0.5 s tones, back to back, 10 s (Study 1); Study 2 glides through 21 values (p. 1076) | 21 rows, 8:00 to 18:00, so the 18:00 close can be queried and sounded; the gallery's Study 1 item drops the last row |
//! | Clicks | 10 ms broadband, 1/s, on every other data point (p. 1069) | Column `click`: 1 on whole hours, 0 on half hours |
//! | Static reference | Opening price, the whole graph (p. 1069) | Column `ref-static`: the opening price on every row |
//! | Dynamic reference | Day's maximum while the price rises, minimum while it falls; starts at the maximum (p. 1069) | Column `ref-dynamic` (SDR-17), from the direction of the step into each row |
//! | Reference beeps | Triangle wave; beep rate, length and level not published (p. 1069) | Not data: the gallery spec sounds one triangle tone per data point |
//! | Mapping | $ = c · f^0.9256, c = 0.0765 ($50 → 1100 Hz); "200–1900 Hz" (p. 1069) | Not data: the gallery spec's psychophysical scale (SDR-11) |
//! | Query | Each hour from 8 a.m. to 6 p.m., random order, 11 per test (p. 1070) | `queries` hours drawn without replacement in seeded order (default 11) |
//!
//! With `series = "reconstructed"` the values are fixed; the seed orders the queries. With
//! `series = "random"`, the open and close are $50 and the extremes $84 and $10 as
//! published; the maximum falls on a seeded half hour from 9:00 to 11:30 and the minimum
//! from 13:30 to 16:00, and the values between anchors follow raised-cosine curves (the
//! "pseudo-sinusoidal" shape) rounded to cents.

use serde::{Deserialize, Serialize};
use sonarithm_core::math;

use super::{GeneratorError, Params, Rng, invalid, round};
use crate::harness::stimulus::{Column, ColumnKind, GeneratorId, Note, Query, Stimulus, Truth};

const ID: GeneratorId = GeneratorId::PriceSeries;

/// The hourly values read from Smith & Walker (2005) Fig. 2, 8:00 to 18:00, in dollars.
/// Figure-read, within about ±$2; the $10 minimum at 15:00 and the $50 open and close are
/// stated in the text.
pub const FIG2_HOURLY: [f64; 11] = [
    50.0, 65.0, 80.0, 76.0, 61.0, 44.0, 28.0, 10.0, 23.0, 40.0, 50.0,
];

/// The published anchors: opening (and closing) price, daily maximum, daily minimum.
pub const OPEN: f64 = 50.0;
/// The published daily maximum.
pub const MAXIMUM: f64 = 84.0;
/// The published daily minimum.
pub const MINIMUM: f64 = 10.0;

/// Which series to generate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Series {
    /// The published series, reconstructed from its anchors and Fig. 2.
    #[default]
    Reconstructed,
    /// A new seeded series with the published anchors.
    Random,
}

/// GEN-002 parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PriceParams {
    /// Which series.
    #[serde(default)]
    pub series: Series,
    /// How many hours to query, drawn without replacement in seeded order (the study
    /// queried all 11, 8:00 to 18:00).
    #[serde(default = "default_queries")]
    pub queries: u32,
}

const fn default_queries() -> u32 {
    11
}

impl Default for PriceParams {
    /// The published design: the reconstructed series, all 11 hours queried.
    fn default() -> Self {
        Self {
            series: Series::Reconstructed,
            queries: default_queries(),
        }
    }
}

/// GEN-002 ground truth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PriceTruth {
    /// The opening price.
    pub open: f64,
    /// The daily maximum and its clock hour.
    pub maximum: (f64, f64),
    /// The daily minimum and its clock hour.
    pub minimum: (f64, f64),
    /// The closing price.
    pub close: f64,
    /// Point-estimation queries; x is the clock hour.
    pub queries: Vec<Query>,
}

/// Generates a GEN-002 series.
pub(crate) fn generate(params: &PriceParams, seed: u64) -> Result<Stimulus, GeneratorError> {
    let queries = params.queries;
    if queries > 11 {
        return Err(invalid(
            ID,
            "queries",
            "the day has 11 whole hours, 8:00 to 18:00",
        ));
    }
    let mut rng = Rng::new(seed, ID);
    let prices = match params.series {
        Series::Reconstructed => reconstructed(),
        Series::Random => random(&mut rng),
    };
    let rows = prices.len();
    let hours: Vec<f64> = (0..rows).map(|i| 8.0 + i as f64 / 2.0).collect();
    let times: Vec<f64> = (0..rows).map(|i| i as f64 / 2.0).collect();
    let clicks: Vec<f64> = (0..rows)
        .map(|i| if i % 2 == 0 { 1.0 } else { 0.0 })
        .collect();
    let (high_row, low_row) = extremes(&prices);
    let reference: Vec<f64> = dynamic_reference(&prices)
        .into_iter()
        .map(|rising| {
            if rising {
                prices[high_row]
            } else {
                prices[low_row]
            }
        })
        .collect();
    let hourly: Vec<usize> = (0..rows).step_by(2).collect();
    let mut pool = hourly;
    let count = (queries as usize).min(pool.len());
    for index in 0..count {
        let pick = index + rng.below(pool.len() - index);
        pool.swap(index, pick);
    }
    let queries = pool
        .into_iter()
        .take(count)
        .map(|row| Query {
            row,
            x: hours[row],
            value: prices[row],
        })
        .collect();
    let truth = PriceTruth {
        open: prices[0],
        maximum: (prices[high_row], hours[high_row]),
        minimum: (prices[low_row], hours[low_row]),
        close: prices[rows - 1],
        queries,
    };
    let notes = match params.series {
        Series::Reconstructed => vec![Note::Deviation {
            detail: "Smith & Walker (2005) do not tabulate the series; hourly values are read from their Fig. 2 (about ±$2) and half hours interpolated, with the $84 maximum at 10:30".to_owned(),
        }],
        Series::Random => vec![Note::Deviation {
            detail: "a new series with the published anchors ($50, $84, $10, $50), not the one the study used".to_owned(),
        }],
    };
    Ok(Stimulus {
        generator: ID,
        seed,
        params: Params::PriceSeries(params.clone()),
        columns: vec![
            Column::new("t", ColumnKind::Time, "Time", times).with_unit("s"),
            Column::new("hour", ColumnKind::Value, "Hour of the day", hours).with_unit("h"),
            Column::new("price", ColumnKind::Value, "Price", prices)
                .with_unit("$")
                .with_concept("money"),
            Column::new("click", ColumnKind::Value, "Hour click", clicks),
            Column::new(
                "ref-static",
                ColumnKind::Value,
                "Static reference",
                vec![OPEN; rows],
            )
            .with_unit("$")
            .with_concept("money"),
            Column::new(
                "ref-dynamic",
                ColumnKind::Value,
                "Dynamic reference",
                reference,
            )
            .with_unit("$")
            .with_concept("money"),
        ],
        truth: Truth::Prices(truth),
        notes,
    })
}

/// The published series: Fig. 2's hourly values, half hours halfway between, and the
/// $84 maximum at 10:30 (between the $80 at 10:00 and the $76 at 11:00, the only place
/// Fig. 2 leaves room for it).
fn reconstructed() -> Vec<f64> {
    let mut prices = Vec::with_capacity(21);
    for pair in FIG2_HOURLY.windows(2) {
        prices.push(pair[0]);
        prices.push(round(f64::midpoint(pair[0], pair[1]), 2));
    }
    prices.push(FIG2_HOURLY[10]);
    // Row 5 is 10:30.
    prices[5] = MAXIMUM;
    prices
}

/// A seeded series through the published anchors on raised-cosine curves.
fn random(rng: &mut Rng) -> Vec<f64> {
    // Half-hour rows: the maximum at 9:00–11:30 (rows 2–7), the minimum at 13:30–16:00
    // (rows 11–16).
    let high_row = 2 + rng.below(6);
    let low_row = 11 + rng.below(6);
    let anchors = [
        (0, OPEN),
        (high_row, MAXIMUM),
        (low_row, MINIMUM),
        (20, OPEN),
    ];
    let mut prices = vec![0.0; 21];
    for pair in anchors.windows(2) {
        let ((start, from), (end, to)) = (pair[0], pair[1]);
        for (step, price) in prices[start..=end].iter_mut().enumerate() {
            let u = step as f64 / (end - start) as f64;
            let eased = (1.0 - math::cos(std::f64::consts::PI * u)) / 2.0;
            *price = round(from + (to - from) * eased, 2);
        }
    }
    prices
}

/// The rows of the daily maximum and minimum (the first of equal values).
fn extremes(prices: &[f64]) -> (usize, usize) {
    let mut high = 0;
    let mut low = 0;
    for (row, &price) in prices.iter().enumerate() {
        if price > prices[high] {
            high = row;
        }
        if price < prices[low] {
            low = row;
        }
    }
    (high, low)
}

/// Whether the price is rising at each row, for the dynamic reference (SDR-17): the
/// direction of the step into the row, so the peak itself still sounds the maximum and
/// the trough the minimum. The first row takes the direction of the first step that
/// moves, and a flat step keeps the previous direction. The series rises first, so the
/// reference starts at the maximum, as published. The paper does not say on which tone
/// the reference turns; this choice is SONARITHM's.
#[must_use]
pub fn dynamic_reference(prices: &[f64]) -> Vec<bool> {
    let first = prices
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .find(|step| *step != 0.0)
        .unwrap_or(0.0);
    let mut rising = first >= 0.0;
    prices
        .iter()
        .enumerate()
        .map(|(row, &price)| {
            if let Some(&previous) = row.checked_sub(1).and_then(|row| prices.get(row)) {
                if price > previous {
                    rising = true;
                } else if price < previous {
                    rising = false;
                }
            }
            rising
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reconstruction_keeps_the_published_anchors() {
        let prices = reconstructed();
        assert_eq!(prices.len(), 21);
        assert_eq!(prices[0], OPEN);
        assert_eq!(prices[20], OPEN);
        assert_eq!(prices.iter().copied().fold(f64::MIN, f64::max), MAXIMUM);
        assert_eq!(prices.iter().copied().fold(f64::MAX, f64::min), MINIMUM);
        for (hour, &value) in FIG2_HOURLY.iter().enumerate() {
            assert_eq!(prices[2 * hour], value);
        }
    }

    #[test]
    fn random_series_keep_the_anchors_for_any_seed() {
        for seed in 0..50 {
            let prices = random(&mut Rng::new(seed, ID));
            assert_eq!(prices[0], OPEN);
            assert_eq!(prices[20], OPEN);
            assert_eq!(prices.iter().copied().fold(f64::MIN, f64::max), MAXIMUM);
            assert_eq!(prices.iter().copied().fold(f64::MAX, f64::min), MINIMUM);
        }
    }
}
