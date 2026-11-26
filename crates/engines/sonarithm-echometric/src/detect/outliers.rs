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
//!
//! **The ends.** The windows are symmetric and shrink toward the ends, so the first and last
//! values' windows are only themselves. With [`EndFit::Skip`] they are never judged and a
//! spike in either is not reported (F-205); descriptions say so (C05-T05). With
//! [`EndFit::TheilSen`] (#507, experimental) each end is predicted from the robust line
//! through its next [`END_WINDOW`] values, and its residual is judged with the rest.
//! [`END_FIT`] is the default, chosen by measurement (F-207).

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

/// How the first and last values are judged (#507).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndFit {
    /// Not at all: their windows are only themselves.
    Skip,
    /// Against the Theil–Sen line through the next [`END_WINDOW`] values (the median of
    /// their pairwise slopes, then the median intercept), extrapolated to the end.
    /// Experimental: INFER.
    TheilSen,
}

impl EndFit {
    const fn name(self) -> &'static str {
        match self {
            Self::Skip => "none",
            Self::TheilSen => "theil-sen",
        }
    }
}

/// The end treatment the detector uses. INFER (ADR-0031): it stays [`EndFit::Skip`] unless
/// the ends-and-interior measurement supports a change (F-207, rule 5).
pub const END_FIT: EndFit = EndFit::Skip;

/// How many values next to an end its line is fitted to. INFER (ADR-0031, #507).
pub const END_WINDOW: usize = 10;

/// The parameters recorded with every `outliers` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("moving-median-robust-z")),
        ("threshold", Param::Number(THRESHOLD)),
        ("minValues", Param::Number(MIN_VALUES as f64)),
        ("window", Param::Number((2 * HALF_WINDOW + 1) as f64)),
        ("endFit", Param::Text(END_FIT.name())),
        ("endWindow", Param::Number(END_WINDOW as f64)),
    ])
}

/// Each value's distance from the median of its window: [`HALF_WINDOW`] values on each side,
/// fewer near the ends, always the same number on both sides. A window cut short on one side
/// would not be centred, and on a slope its median would leave every value near the end
/// looking like an outlier; so the first and last values, whose windows are only themselves,
/// are never judged.
#[must_use]
pub fn residuals(values: &[f64]) -> Vec<f64> {
    residuals_with(values, END_FIT)
}

/// [`residuals`] with the end treatment `ends`.
#[must_use]
pub fn residuals_with(values: &[f64], ends: EndFit) -> Vec<f64> {
    let n = values.len();
    (0..n)
        .map(|at| {
            if ends == EndFit::TheilSen && n > END_WINDOW && (at == 0 || at == n - 1) {
                // The end's neighbours, by their distance from it: 1, 2, … END_WINDOW.
                let neighbours: Vec<(f64, f64)> = (1..=END_WINDOW)
                    .filter_map(|step| {
                        let index = if at == 0 { step } else { n - 1 - step };
                        values.get(index).map(|&v| (step as f64, v))
                    })
                    .collect();
                if let (Some(line), Some(&value)) = (theil_sen(&neighbours), values.get(at)) {
                    return value - line.1;
                }
            }
            let half = HALF_WINDOW.min(at).min(values.len() - 1 - at);
            let start = at - half;
            let end = at + half + 1;
            let centre = values.get(start..end).map_or(f64::NAN, median);
            values.get(at).map_or(f64::NAN, |v| v - centre)
        })
        .collect()
}

/// The Theil–Sen line through `points` (x, y): the median of the pairwise slopes, then the
/// median of y − slope·x, as `(slope, intercept)`; the intercept is the prediction at x = 0.
/// `None` for fewer than two points.
#[must_use]
pub fn theil_sen(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let mut slopes = Vec::with_capacity(points.len() * points.len() / 2);
    for (i, &(x1, y1)) in points.iter().enumerate() {
        for &(x2, y2) in points.iter().skip(i + 1) {
            if (x2 - x1).abs() > 0.0 {
                slopes.push((y2 - y1) / (x2 - x1));
            }
        }
    }
    if slopes.is_empty() {
        return None;
    }
    let slope = median(&slopes);
    let intercepts: Vec<f64> = points.iter().map(|&(x, y)| y - slope * x).collect();
    Some((slope, median(&intercepts)))
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
    scores_with(series, END_FIT)
}

/// [`scores`] with the end treatment `ends`.
#[must_use]
pub fn scores_with(series: &Series, ends: EndFit) -> Option<Vec<f64>> {
    robust_z(&residuals_with(series.values(), ends))
}

/// The outliers of `series`, in row order: each value whose robust z-score exceeds
/// [`THRESHOLD`] in size, with the score.
#[must_use]
pub fn detect(series: &Series) -> Vec<Finding> {
    detect_with(series, END_FIT)
}

/// [`detect`] with the end treatment `ends`, for measuring it (F-207).
#[must_use]
pub fn detect_with(series: &Series, ends: EndFit) -> Vec<Finding> {
    series
        .rows()
        .iter()
        .zip(series.values())
        .zip(scores_with(series, ends).unwrap_or_default())
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
    fn theil_sen_ends_find_an_end_spike_on_a_slope_and_leave_a_plain_slope_alone() {
        let mut values: Vec<f64> = (0..40).map(|i| f64::from(i) * 2.0).collect();
        // A small deterministic wiggle, so the residuals have a spread.
        for (i, v) in values.iter_mut().enumerate() {
            *v += [0.3, -0.2, 0.1, -0.4][i % 4];
        }
        let plain = detect_with(&series(&values), EndFit::TheilSen);
        assert_eq!(plain, [], "a slope's ends are on its line");
        values[0] = 40.0;
        let found = detect_with(&series(&values), EndFit::TheilSen);
        assert_eq!(found.iter().map(|f| f.first).collect::<Vec<_>>(), [0]);
        assert_eq!(detect_with(&series(&values), EndFit::Skip), [], "skipped ends");
    }

    #[test]
    fn theil_sen_ignores_one_spike_among_its_points() {
        let mut points: Vec<(f64, f64)> = (1..=10).map(|i| (f64::from(i), 3.0 * f64::from(i) + 1.0)).collect();
        points[4].1 = 500.0;
        let (slope, intercept) = theil_sen(&points).unwrap();
        assert!((slope - 3.0).abs() < 1e-9 && (intercept - 1.0).abs() < 1e-9, "{slope} {intercept}");
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
