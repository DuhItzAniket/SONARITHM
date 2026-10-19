// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GEN-001: trend reversals × data density, after Nees & Walker (2008)
//! (`nees2008density`, ACM TAP 5(3), Art. 13).
//!
//! The study crossed four data densities with four numbers of trend reversals. Each graph
//! was a fictitious stock price over a ten-hour trading day, one second per hour, with
//! every graph reaching both ends of the price range.
//!
//! | Parameter | Published (section, page) | GEN-001 |
//! |---|---|---|
//! | Density | 1, 2, 4, 8 points/s (§2.1.3, p. 13:8) | `rate`, required; others allowed, above 8 flagged experimental (SDR-20, H-02) |
//! | Reversals | 0, 1, 2, 3 (§2.1.3, p. 13:8) | `reversals`, required; above 3 flagged (SDR-21) |
//! | Initial direction | Up or down, counterbalanced (p. 13:9) | `start`: `up`, `down` or `random` (default) |
//! | Day | 8 a.m. to 6 p.m., 1 s per hour (pp. 13:8–13:10) | `hours` = 10 by default; columns `t` (s) and `hour` (clock) |
//! | Points per graph | Not stated | `hours · rate + 1` (11, 21, 41, 81), inferred from the 10.125 s graphs and Fig. 1 |
//! | Range | $6 to $106, both reached by every graph (pp. 13:8, 13:10) | `range` = [6, 106]; the trend is min–max scaled onto it |
//! | Series shape | Not published; Fig. 1 is "schematic only" | `shape`: `schematic` (default) reads Fig. 1's shapes, `random` draws piecewise-linear trends |
//! | Point-estimation query | Each hour, 8 a.m. to 6 p.m., in random order (p. 13:11) | `queries` hours drawn without replacement in seeded order (default all 11) |
//! | Trend-identification query | Each of the 10 hourly intervals: increased, decreased or stayed the same (§3.1.2, p. 13:15) | `trend-queries` intervals in seeded order (default all 10) |
//! | Tones, ramps, pitch, timbre | 125 ms, 10 ms ramps, G2–B6 exact scale, piano (§2.1.4) | Not data: the gallery spec sets them (T02) |
//!
//! **Schematic shapes** (figure-read from Fig. 1 for the initially rising case, with
//! `u = t / hours` in [0, 1], then min–max scaled to the range): 0 reversals `u`;
//! 1 reversal `(1 − cos 2πu) / 2`; 2 reversals `sin 2πu`; 3 reversals `sin 3πu`. An
//! initially falling series mirrors the rising one. These shapes are a reading of a
//! schematic, not published functions, so every schematic stimulus carries a
//! [`Note::Deviation`] that says so. Schematic shapes exist for 0 to 3 reversals only.
//!
//! **Random shapes** split the series into `reversals + 1` monotone segments whose
//! directions alternate. Segment lengths are a uniform random composition with at least
//! `min-segment` steps each; peaks are drawn from the top 40% and troughs from the bottom
//! 40% of the unit range before scaling, so every segment moves by at least a fifth of
//! the range. Optional Gaussian `noise` (in value units) is added after scaling; the
//! ground truth describes the noise-free trend except for the queried values, which are
//! the values as played.

use serde::{Deserialize, Serialize};
use sonarithm_core::math;

use super::{GeneratorError, Params, Rng, composition, draw, invalid, round, whole};
use crate::harness::stimulus::{Column, ColumnKind, GeneratorId, Note, Query, Stimulus, Truth};

const ID: GeneratorId = GeneratorId::TrendReversals;

/// The data densities Nees & Walker (2008) tested, in points per second (SDR-19).
pub const STUDIED_RATES: [f64; 4] = [1.0, 2.0, 4.0, 8.0];

/// The trend-reversal levels Nees & Walker (2008) tested (SDR-21).
pub const STUDIED_REVERSALS: [u32; 4] = [0, 1, 2, 3];

/// The direction of a monotone segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    /// Values rise.
    Up,
    /// Values fall.
    Down,
}

impl Direction {
    const fn flip(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

/// The local trend between two queried times: the three answers of Nees & Walker's
/// trend-identification task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Trend {
    /// The value rose.
    Increased,
    /// The value fell.
    Decreased,
    /// The value is the same at both times.
    Same,
}

impl Trend {
    /// The answer's position in the protocol's three options: increased, decreased,
    /// stayed the same.
    #[must_use]
    pub const fn option(self) -> usize {
        match self {
            Self::Increased => 0,
            Self::Decreased => 1,
            Self::Same => 2,
        }
    }
}

/// Which way the first segment goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Start {
    /// Up or down with equal probability, from the seed (the study counterbalanced both).
    #[default]
    Random,
    /// The series starts rising.
    Up,
    /// The series starts falling.
    Down,
}

/// How the trend is shaped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    /// The shapes read from the study's schematic Fig. 1 (0 to 3 reversals).
    #[default]
    Schematic,
    /// Seeded piecewise-linear trends, for new stimuli with any number of reversals.
    Random,
}

/// GEN-001 parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ReversalParams {
    /// Data density in points per second; studied: 1, 2, 4, 8 (SDR-19).
    pub rate: f64,
    /// The number of trend reversals; studied: 0 to 3 (SDR-21).
    pub reversals: u32,
    /// The length of the trading day in hours, one second each (Nees & Walker 2008: 10).
    #[serde(default = "default_hours")]
    pub hours: u32,
    /// The value range `[low, high]` every series reaches (Nees & Walker 2008: $6–$106).
    #[serde(default = "default_range")]
    pub range: [f64; 2],
    /// The shape of the trend.
    #[serde(default)]
    pub shape: Shape,
    /// The direction of the first segment.
    #[serde(default)]
    pub start: Start,
    /// The smallest number of steps in a monotone segment, for `random` shapes.
    #[serde(default = "default_min_segment")]
    pub min_segment: u32,
    /// The standard deviation of Gaussian noise added to every point, in value units
    /// (the study used none).
    #[serde(default)]
    pub noise: f64,
    /// How many hours to query for point estimation, drawn without replacement in seeded
    /// order (the study queried all 11 hours).
    #[serde(default = "default_queries")]
    pub queries: u32,
    /// How many hourly intervals to query for trend identification, in seeded order (the
    /// study queried all 10).
    #[serde(default = "default_trend_queries")]
    pub trend_queries: u32,
}

const fn default_hours() -> u32 {
    10
}

const fn default_range() -> [f64; 2] {
    [6.0, 106.0]
}

const fn default_min_segment() -> u32 {
    2
}

const fn default_queries() -> u32 {
    11
}

const fn default_trend_queries() -> u32 {
    10
}

impl ReversalParams {
    /// The published design for one cell: `rate` points per second and `reversals`
    /// reversals, every other parameter at its default.
    #[must_use]
    pub const fn cell(rate: f64, reversals: u32) -> Self {
        Self {
            rate,
            reversals,
            hours: default_hours(),
            range: default_range(),
            shape: Shape::Schematic,
            start: Start::Random,
            min_segment: default_min_segment(),
            noise: 0.0,
            queries: default_queries(),
            trend_queries: default_trend_queries(),
        }
    }
}

/// A trend-identification query: did the value rise, fall or stay between two hours?
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TrendQuery {
    /// The earlier hour, as a clock hour.
    pub from: f64,
    /// The later hour.
    pub to: f64,
    /// The correct answer.
    pub answer: Trend,
}

/// GEN-001 ground truth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ReversalTruth {
    /// The number of reversals in the trend.
    pub reversals: u32,
    /// The direction of each monotone segment, in order.
    pub segments: Vec<Direction>,
    /// The rows where the trend changes direction.
    pub turning_rows: Vec<usize>,
    /// Point-estimation queries (x is the clock hour), with the values as played.
    pub queries: Vec<Query>,
    /// Trend-identification queries over hourly intervals.
    pub trend_queries: Vec<TrendQuery>,
}

/// Generates a GEN-001 series.
pub(crate) fn generate(params: &ReversalParams, seed: u64) -> Result<Stimulus, GeneratorError> {
    let points = check(params)?;
    let mut rng = Rng::new(seed, ID);
    let first = match params.start {
        Start::Up => Direction::Up,
        Start::Down => Direction::Down,
        Start::Random => {
            if rng.below(2) == 0 {
                Direction::Up
            } else {
                Direction::Down
            }
        }
    };
    let (unit, turning_rows, segments) = match params.shape {
        Shape::Schematic => schematic(params, points, first),
        Shape::Random => random(&mut rng, params, points, first),
    };
    let [low, high] = params.range;
    let values: Vec<f64> = unit
        .iter()
        .map(|&u| {
            let value = low + (high - low) * u;
            let noisy = if params.noise > 0.0 {
                value + params.noise * rng.normal()
            } else {
                value
            };
            round(noisy, 2)
        })
        .collect();
    let times: Vec<f64> = (0..points)
        .map(|i| round(i as f64 / params.rate, 6))
        .collect();
    let hours: Vec<f64> = times.iter().map(|t| round(8.0 + t, 6)).collect();
    let hourly = hourly_rows(params, points);
    let queries = draw(&mut rng, &hourly, params.queries as usize)
        .into_iter()
        .map(|row| Query {
            row,
            x: hours[row],
            value: values[row],
        })
        .collect();
    let intervals: Vec<usize> = (0..hourly.len().saturating_sub(1)).collect();
    let trend_queries = draw(&mut rng, &intervals, params.trend_queries as usize)
        .into_iter()
        .map(|interval| {
            let (from, to) = (hourly[interval], hourly[interval + 1]);
            TrendQuery {
                from: hours[from],
                to: hours[to],
                answer: trend(values[from], values[to]),
            }
        })
        .collect();
    let truth = ReversalTruth {
        reversals: params.reversals,
        segments,
        turning_rows,
        queries,
        trend_queries,
    };
    Ok(Stimulus {
        generator: ID,
        seed,
        params: Params::TrendReversals(params.clone()),
        columns: vec![
            Column::new("t", ColumnKind::Time, "Time", times).with_unit("s"),
            Column::new("hour", ColumnKind::Value, "Hour of the day", hours).with_unit("h"),
            Column::new("price", ColumnKind::Value, "Price", values)
                .with_unit("$")
                .with_concept("money"),
        ],
        truth: Truth::Reversals(truth),
        notes: notes(params),
    })
}

/// Checks the parameters; returns the number of points.
fn check(params: &ReversalParams) -> Result<usize, GeneratorError> {
    if !(params.rate.is_finite() && params.rate > 0.0 && params.rate <= 1000.0) {
        return Err(invalid(
            ID,
            "rate",
            "must be a number of points per second in (0, 1000]",
        ));
    }
    if params.hours == 0 || params.hours > 1000 {
        return Err(invalid(ID, "hours", "must be between 1 and 1000"));
    }
    let [low, high] = params.range;
    if !(low.is_finite() && high.is_finite() && low < high) {
        return Err(invalid(
            ID,
            "range",
            "must be two finite numbers with low < high",
        ));
    }
    if !(params.noise.is_finite() && params.noise >= 0.0) {
        return Err(invalid(
            ID,
            "noise",
            "must be zero or a positive standard deviation",
        ));
    }
    let steps = f64::from(params.hours) * params.rate;
    if steps.fract() != 0.0 || steps > 100_000.0 {
        return Err(invalid(
            ID,
            "rate",
            "hours × rate must be a whole number of steps, at most 100000, so every hour falls on a point",
        ));
    }
    let points = whole(steps) + 1;
    if params.shape == Shape::Schematic && params.reversals > 3 {
        return Err(invalid(
            ID,
            "reversals",
            "schematic shapes exist for 0 to 3 reversals; use shape = \"random\" for more",
        ));
    }
    if params.min_segment == 0 {
        return Err(invalid(ID, "min-segment", "must be at least 1 step"));
    }
    let needed = (params.reversals as usize + 1) * params.min_segment as usize + 1;
    if params.shape == Shape::Random && points < needed {
        return Err(invalid(
            ID,
            "reversals",
            format!(
                "{} reversals need {needed} points with min-segment {}, but the series has {points}",
                params.reversals, params.min_segment
            ),
        ));
    }
    if params.queries > params.hours + 1 {
        return Err(invalid(
            ID,
            "queries",
            "cannot exceed the number of hours plus one",
        ));
    }
    if params.trend_queries > params.hours {
        return Err(invalid(
            ID,
            "trend-queries",
            "cannot exceed the number of hours",
        ));
    }
    Ok(points)
}

fn notes(params: &ReversalParams) -> Vec<Note> {
    let mut notes = Vec::new();
    if params.shape == Shape::Schematic {
        notes.push(Note::Deviation {
            detail: "Nees & Walker (2008) publish no generating function; the shape is read from their schematic Fig. 1".to_owned(),
        });
    }
    if params.rate > 8.0 {
        notes.push(Note::Experimental {
            rule: "SDR-20 / H-02".to_owned(),
            detail: format!(
                "{} points/s is above the 8 points/s Nees & Walker (2008) tested",
                params.rate
            ),
        });
    } else if !STUDIED_RATES.contains(&params.rate) {
        notes.push(Note::Deviation {
            detail: format!(
                "{} points/s is not one of the studied densities 1, 2, 4, 8",
                params.rate
            ),
        });
    }
    if params.reversals > 3 {
        notes.push(Note::Experimental {
            rule: "SDR-21 / H-02".to_owned(),
            detail: format!(
                "{} reversals is above the 3 Nees & Walker (2008) tested",
                params.reversals
            ),
        });
    }
    if params.noise > 0.0 {
        notes.push(Note::Deviation {
            detail: "noise was not part of the published stimuli and can add local reversals"
                .to_owned(),
        });
    }
    notes
}

/// The rows that fall on whole hours.
fn hourly_rows(params: &ReversalParams, points: usize) -> Vec<usize> {
    let per_hour = whole(params.rate.trunc());
    if per_hour == 0 || params.rate.fract() != 0.0 {
        // A rate below one point per hour or a fractional rate: every point whose time
        // is a whole hour.
        return (0..points)
            .filter(|&i| (i as f64 / params.rate).fract() == 0.0)
            .collect();
    }
    (0..points).step_by(per_hour).collect()
}

fn trend(from: f64, to: f64) -> Trend {
    if to > from {
        Trend::Increased
    } else if to < from {
        Trend::Decreased
    } else {
        Trend::Same
    }
}

/// Fig. 1's shapes in [0, 1], with their turning rows and segment directions.
fn schematic(
    params: &ReversalParams,
    points: usize,
    first: Direction,
) -> (Vec<f64>, Vec<usize>, Vec<Direction>) {
    let pi = std::f64::consts::PI;
    let last = (points - 1) as f64;
    let rising: Vec<f64> = (0..points)
        .map(|i| {
            let u = i as f64 / last;
            match params.reversals {
                0 => u,
                1 => (1.0 - math::cos(2.0 * pi * u)) / 2.0,
                2 => math::sin(2.0 * pi * u),
                _ => math::sin(3.0 * pi * u),
            }
        })
        .collect();
    let unit = normalize(&rising);
    let unit: Vec<f64> = match first {
        Direction::Up => unit,
        Direction::Down => unit.iter().map(|u| 1.0 - u).collect(),
    };
    // Turning points of the shapes, as fractions of the day: 1 reversal at 1/2,
    // 2 at 1/4 and 3/4, 3 at 1/6, 1/2 and 5/6.
    let fractions: &[f64] = match params.reversals {
        0 => &[],
        1 => &[0.5],
        2 => &[0.25, 0.75],
        _ => &[1.0 / 6.0, 0.5, 5.0 / 6.0],
    };
    let turning_rows = fractions
        .iter()
        .map(|f| whole((f * last).round()))
        .collect();
    (
        unit,
        turning_rows,
        directions(first, params.reversals as usize + 1),
    )
}

/// A seeded piecewise-linear trend in [0, 1].
fn random(
    rng: &mut Rng,
    params: &ReversalParams,
    points: usize,
    first: Direction,
) -> (Vec<f64>, Vec<usize>, Vec<Direction>) {
    let segments = params.reversals as usize + 1;
    let lengths = composition(rng, points - 1, segments, params.min_segment as usize);
    let segment_directions = directions(first, segments);
    let mut ends = Vec::with_capacity(segments + 1);
    ends.push(match first {
        Direction::Up => 0.4 * rng.uniform(),
        Direction::Down => 0.6 + 0.4 * rng.uniform(),
    });
    for direction in &segment_directions {
        ends.push(match direction {
            Direction::Up => 0.6 + 0.4 * rng.uniform(),
            Direction::Down => 0.4 * rng.uniform(),
        });
    }
    let mut trend = Vec::with_capacity(points);
    let mut turning_rows = Vec::with_capacity(segments - 1);
    let mut row = 0;
    for (segment, &length) in lengths.iter().enumerate() {
        let (from, to) = (ends[segment], ends[segment + 1]);
        for step in 0..length {
            trend.push(from + (to - from) * step as f64 / length as f64);
        }
        row += length;
        if segment + 1 < segments {
            turning_rows.push(row);
        }
    }
    trend.push(ends[segments]);
    (normalize(&trend), turning_rows, segment_directions)
}

fn directions(first: Direction, segments: usize) -> Vec<Direction> {
    let mut direction = first;
    (0..segments)
        .map(|_| {
            let current = direction;
            direction = direction.flip();
            current
        })
        .collect()
}

/// Min–max scales `values` onto [0, 1], so the series reaches both ends of the range.
fn normalize(values: &[f64]) -> Vec<f64> {
    let low = values.iter().copied().fold(f64::INFINITY, f64::min);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = high - low;
    values
        .iter()
        .map(|v| if span > 0.0 { (v - low) / span } else { 0.0 })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hourly_rows_follow_the_rate() {
        let params = ReversalParams::cell(4.0, 0);
        assert_eq!(
            hourly_rows(&params, 41),
            (0..41).step_by(4).collect::<Vec<_>>()
        );
        let params = ReversalParams {
            rate: 0.5,
            hours: 4,
            ..ReversalParams::cell(0.5, 0)
        };
        assert_eq!(hourly_rows(&params, 3), vec![0, 1, 2]);
    }
}
