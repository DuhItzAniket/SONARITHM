// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `outliers` detector (C05-T02; FR-080, A11Y-23): values far from the series' centre by
//! the robust z-score of Iglewicz and Hoaglin (1993), from the median and the median absolute
//! deviation (MAD). It does not depend on the series' mean or standard deviation, which the
//! outliers themselves would pull (ADR-0031).

use super::{median, Finding, Param, Params, Series};

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

/// The parameters recorded with every `outliers` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("robust-z")),
        ("threshold", Param::Number(THRESHOLD)),
        ("minValues", Param::Number(MIN_VALUES as f64)),
    ])
}

/// Each value's robust z-score, or `None` when the series is too short or has no spread.
#[must_use]
pub fn scores(series: &Series) -> Option<Vec<f64>> {
    let values = series.values();
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
    if scale.is_nan() || scale <= 0.0 {
        return None;
    }
    Some(values.iter().map(|v| (v - centre) / scale).collect())
}

/// The outliers of `series`, in row order: each value whose robust z-score exceeds
/// [`THRESHOLD`] in size, with the score.
#[must_use]
pub fn detect(series: &Series) -> Vec<Finding> {
    let Some(scores) = scores(series) else {
        return Vec::new();
    };
    series
        .rows()
        .iter()
        .zip(series.values())
        .zip(scores)
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
        assert!(detect(&series(&[1.0, 100.0, 1.0])).is_empty(), "too short");
        assert!(detect(&series(&[2.0; 8])).is_empty(), "no spread");
    }

    #[test]
    fn a_zero_mad_falls_back_to_the_mean_absolute_deviation() {
        // Most values equal: the MAD is 0, but the one far value is still found.
        let found = detect(&series(&[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 50.0]));
        assert_eq!(found.iter().map(|f| f.first).collect::<Vec<_>>(), [6]);
    }
}
