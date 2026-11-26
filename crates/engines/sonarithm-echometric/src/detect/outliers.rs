// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `outliers` detector (C05-T02; FR-080, A11Y-23): values far from their neighbours. Each
//! value's residual is its distance from the median of a window around it (a moving median,
//! which a single spike does not move); the robust z-score of Iglewicz and Hoaglin (1993) is
//! taken over all residuals, from their median and median absolute deviation (MAD). Neither
//! step uses the mean or standard deviation, which outliers would pull (ADR-0031).
//!
//! Both halves matter (F-205, GEN-011's sine base over 100 seeds): a robust z over the raw
//! values let the base inflate the MAD and missed spikes that ran against it (recall 0.66);
//! a robust z inside each small window estimated the MAD from too few values and called
//! noise an outlier (precision 0.87). Residuals keep the base out and use every value.

use super::{Finding, Param, Params, Series, median};

/// The robust z-score above which a value is an outlier (Iglewicz & Hoaglin 1993's
/// recommendation). INFER (ADR-0031): no design rule sets it for sonification.
pub const THRESHOLD: f64 = 3.5;

/// The normal distribution's 75th percentile, which scales the MAD to the standard deviation
/// for normal data (Iglewicz & Hoaglin 1993).
const MAD_SCALE: f64 = 0.6745;

/// When the MAD is zero (more than half the values are equal), the mean absolute deviation
/// times this factor, √(π/2), stands in for it (Iglewicz & Hoaglin 1993).
const MEAN_AD_SCALE: f64 = 1.253_314;

/// The fewest values a series needs before any is called an outlier: with fewer, the median
/// and MAD say little. INFER (ADR-0031).
pub const MIN_VALUES: usize = 5;

/// The values on each side of a value whose median it is measured from (21 in all, fewer at
/// the ends). INFER (ADR-0031, F-205).
pub const HALF_WINDOW: usize = 10;

/// The parameters recorded with every `outliers` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("moving-median-robust-z")),
        ("threshold", Param::Number(THRESHOLD)),
        ("minValues", Param::Number(MIN_VALUES as f64)),
        ("window", Param::Number((2 * HALF_WINDOW + 1) as f64)),
    ])
}

/// Each value's distance from the median of its window: [`HALF_WINDOW`] values on each side,
/// fewer near the ends, always the same number on both sides. A window cut short on one side
/// would not be centred, and on a slope its median would leave every value near the end
/// looking like an outlier; so the first and last values, whose windows are only themselves,
/// are never judged.
#[must_use]
pub fn residuals(values: &[f64]) -> Vec<f64> {
    (0..values.len())
        .map(|at| {
            let half = HALF_WINDOW.min(at).min(values.len() - 1 - at);
            let start = at - half;
            let end = at + half + 1;
            let centre = values.get(start..end).map_or(f64::NAN, median);
            values.get(at).map_or(f64::NAN, |v| v - centre)
        })
        .collect()
}

/// The robust z-scores of `values` (Iglewicz & Hoaglin 1993), or `None` when there are
/// fewer than [`MIN_VALUES`] or they have no spread.
#[must_use]
pub fn robust_z(values: &[f64]) -> Option<Vec<f64>> {
    if values.len() < MIN_VALUES {
        return None;
    }
    let centre = median(values);
    let deviations: Vec<f64> = values.iter().map(|v| (v - centre).abs()).collect();
    let mad = median(&deviations);
    let scale = if mad > 0.0 {
        mad / MAD_SCALE
    } else {
        let mean_ad = deviations.iter().sum::<f64>() / deviations.len() as f64;
        mean_ad * MEAN_AD_SCALE
    };
    (scale.is_finite() && scale > 0.0)
        .then(|| values.iter().map(|v| (v - centre) / scale).collect())
}

/// Each value's robust z-score among the residuals from the moving median, or `None` when
/// the series is too short or has no spread.
#[must_use]
pub fn scores(series: &Series) -> Option<Vec<f64>> {
    robust_z(&residuals(series.values()))
}

/// The outliers of `series`, in row order: each value whose robust z-score exceeds
/// [`THRESHOLD`] in size, with the score.
#[must_use]
pub fn detect(series: &Series) -> Vec<Finding> {
    series
        .rows()
        .iter()
        .zip(series.values())
        .zip(scores(series).unwrap_or_default())
        .filter(|(_, z)| z.abs() > THRESHOLD)
        .map(|((&row, &value), z)| Finding {
            label: "outlier",
            first: row,
            last: row,
            value,
            score: Some(z),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(values: &[f64]) -> Series {
        Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
    }

    #[test]
    fn a_spike_is_an_outlier_and_the_rest_are_not() {
        let found = detect(&series(&[10.0, 11.0, 9.0, 10.5, 9.5, 40.0, 10.2]));
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].first, found[0].value), (5, 40.0));
        assert!(found[0].score.is_some_and(|z| z > THRESHOLD));
    }

    #[test]
    fn a_short_or_constant_series_has_no_outliers() {
        assert_eq!(detect(&series(&[1.0, 100.0, 1.0])), [], "too short");
        assert_eq!(detect(&series(&[2.0; 8])), [], "no spread");
    }

    #[test]
    fn a_zero_mad_falls_back_to_the_mean_absolute_deviation() {
        // Most values equal: the MAD is 0, but the one far value is still found.
        let found = detect(&series(&[5.0, 5.0, 5.0, 50.0, 5.0, 5.0, 5.0]));
        assert_eq!(found.iter().map(|f| f.first).collect::<Vec<_>>(), [3]);
    }

    #[test]
    fn a_spike_against_a_slope_is_found() {
        // A steady rise with one value dropped well below it: over the raw values the slope
        // would inflate the MAD (F-205).
        let mut values: Vec<f64> = (0..60).map(|i| f64::from(i) * 2.0).collect();
        values[30] = 20.0;
        let found = detect(&series(&values));
        assert_eq!(found.iter().map(|f| f.first).collect::<Vec<_>>(), [30]);
    }
}
