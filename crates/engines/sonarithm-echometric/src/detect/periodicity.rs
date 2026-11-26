// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `periodicity` detector (C05-T03; FR-080, A11Y-23): whether a series repeats, and its
//! period in rows. The normalised autocorrelation of the series (its mean removed) is computed
//! for every lag up to half its length; the period is the lag of the first local peak that
//! reaches [`MIN_CORRELATION`] and is the highest such peak within a tolerance, so a period's
//! multiples (which correlate as well) are not reported instead of it (ADR-0031).

use super::{Finding, Param, Params, Series};

/// The correlation a peak must reach to count as a period. INFER (ADR-0031).
pub const MIN_CORRELATION: f64 = 0.5;

/// The shortest period, in rows: a lag of 1 is the series' smoothness, not a repetition.
pub const MIN_LAG: usize = 2;

/// The fewest full periods the series must hold. INFER (ADR-0031).
pub const MIN_PERIODS: usize = 2;

/// How much lower than the highest peak a shorter lag may be and still be the period: a
/// period's multiples correlate almost as well, so the shortest strong peak wins.
const PEAK_TOLERANCE: f64 = 0.1;

/// The parameters recorded with every `periodicity` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("autocorrelation")),
        ("minCorrelation", Param::Number(MIN_CORRELATION)),
        ("minLag", Param::Number(MIN_LAG as f64)),
        ("minPeriods", Param::Number(MIN_PERIODS as f64)),
    ])
}

/// The normalised autocorrelation of `values` at lags 0 to `max_lag`, or `None` when the
/// values have no variance.
#[must_use]
pub fn autocorrelation(values: &[f64], max_lag: usize) -> Option<Vec<f64>> {
    let n = values.len();
    if n == 0 {
        return None;
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let centred: Vec<f64> = values.iter().map(|v| v - mean).collect();
    let variance: f64 = centred.iter().map(|v| v * v).sum();
    if !variance.is_finite() || variance <= 0.0 {
        return None;
    }
    Some(
        (0..=max_lag.min(n.saturating_sub(1)))
            .map(|lag| {
                let products: f64 = centred
                    .iter()
                    .zip(centred.iter().skip(lag))
                    .map(|(a, b)| a * b)
                    .sum();
                products / variance
            })
            .collect(),
    )
}

/// The period of `values` in rows and its correlation, or `None` when no lag reaches
/// [`MIN_CORRELATION`] with at least [`MIN_PERIODS`] periods in the series.
#[must_use]
pub fn period(values: &[f64]) -> Option<(usize, f64)> {
    let max_lag = values.len() / MIN_PERIODS;
    let r = autocorrelation(values, max_lag)?;
    let peaks: Vec<(usize, f64)> = (MIN_LAG..r.len())
        .filter_map(|lag| {
            let here = *r.get(lag)?;
            let before = *r.get(lag - 1)?;
            let after = r.get(lag + 1).copied().unwrap_or(f64::NEG_INFINITY);
            (here >= MIN_CORRELATION && here > before && here >= after).then_some((lag, here))
        })
        .collect();
    let highest = peaks.iter().map(|&(_, r)| r).fold(f64::NEG_INFINITY, f64::max);
    peaks
        .into_iter()
        .find(|&(_, r)| r >= highest - PEAK_TOLERANCE)
}

/// The period of `series`, as one finding over all its rows: the value is the period in
/// rows, the score its autocorrelation.
#[must_use]
pub fn detect(series: &Series) -> Vec<Finding> {
    let rows = series.rows();
    let (Some(&first), Some(&last)) = (rows.first(), rows.last()) else {
        return Vec::new();
    };
    period(series.values())
        .map(|(lag, r)| Finding {
            label: "period",
            first,
            last,
            value: lag as f64,
            score: Some(r),
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use sonarithm_core::math;

    use super::*;

    fn wave(period: f64, n: usize) -> Vec<f64> {
        (0..n)
            .map(|i| math::sin(std::f64::consts::TAU * i as f64 / period))
            .collect()
    }

    #[test]
    fn a_sine_has_its_period() {
        let (lag, r) = period(&wave(12.0, 120)).expect("periodic");
        assert_eq!(lag, 12);
        assert!(r > 0.8, "{r}");
    }

    #[test]
    fn the_shortest_strong_peak_wins_over_its_multiples() {
        // Two periods of 10 and 20 rows: the 20-row lag correlates as well, but 10 is the period.
        let values: Vec<f64> = (0..200)
            .map(|i| {
                let t = std::f64::consts::TAU * i as f64;
                math::sin(t / 10.0) + 0.2 * math::sin(t / 20.0)
            })
            .collect();
        assert_eq!(period(&values).map(|(lag, _)| lag), Some(10));
    }

    #[test]
    fn a_ramp_or_a_constant_is_not_periodic() {
        let ramp: Vec<f64> = (0..100).map(f64::from).collect();
        assert_eq!(period(&ramp), None);
        assert_eq!(period(&[4.0; 50]), None);
        assert_eq!(detect(&Series::new([])), []);
    }
}
