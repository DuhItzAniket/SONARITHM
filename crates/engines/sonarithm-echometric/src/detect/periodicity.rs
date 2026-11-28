// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `periodicity` detector (C05-T03; FR-080, A11Y-23): whether a series repeats, and its
//! period in rows. The normalised autocorrelation of the series, with its least-squares line
//! removed, is computed for every lag up to half its length. Removing only the mean left a
//! trend's slow decay in the autocorrelation, whose noise peaks passed the threshold: a
//! noisy ramp reported a period in about half of all seeds (#520 review, F-260). the period is the lag of the first local peak that
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
        ("method", Param::Text("detrended-autocorrelation")),
        ("minCorrelation", Param::Number(MIN_CORRELATION)),
        ("minLag", Param::Number(MIN_LAG as f64)),
        ("minPeriods", Param::Number(MIN_PERIODS as f64)),
    ])
}

/// `values` less their least-squares line over the row index: a trend's slow rise or fall
/// is removed, and a periodic series keeps its swings (a sine's line is nearly flat).
#[must_use]
pub fn detrended(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    if n < 2 {
        return values.to_vec();
    }
    let count = n as f64;
    let x_mean = (count - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / count;
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (i, &y) in values.iter().enumerate() {
        let dx = i as f64 - x_mean;
        sxy += dx * (y - y_mean);
        sxx += dx * dx;
    }
    let slope = if sxx > 0.0 { sxy / sxx } else { 0.0 };
    values
        .iter()
        .enumerate()
        .map(|(i, &y)| y - y_mean - slope * (i as f64 - x_mean))
        .collect()
}

/// The normalised autocorrelation of `values`, with their least-squares line removed
/// ([`detrended`]), at lags 0 to `max_lag`, or `None` when nothing is left once the line is
/// removed (a constant or a straight line).
#[must_use]
pub fn autocorrelation(values: &[f64], max_lag: usize) -> Option<Vec<f64>> {
    let n = values.len();
    if n == 0 {
        return None;
    }
    let centred = detrended(values);
    let variance: f64 = centred.iter().map(|v| v * v).sum();
    // What the line leaves of a straight series is rounding residue: below a 1e-12 share of
    // the spread about the mean, there is nothing to correlate.
    let mean = values.iter().sum::<f64>() / n as f64;
    let spread: f64 = values.iter().map(|v| (v - mean) * (v - mean)).sum();
    if !variance.is_finite() || variance <= 1e-12 * spread || variance <= 0.0 {
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
    let highest = peaks
        .iter()
        .map(|&(_, r)| r)
        .fold(f64::NEG_INFINITY, f64::max);
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
                let t = std::f64::consts::TAU * f64::from(i);
                math::sin(t / 10.0) + 0.2 * math::sin(t / 20.0)
            })
            .collect();
        assert_eq!(period(&values).map(|(lag, _)| lag), Some(10));
    }

    #[test]
    fn a_ramp_or_a_constant_is_not_periodic() {
        let ramp: Vec<f64> = (0..100).map(f64::from).collect();
        assert_eq!(period(&ramp), None);
        // A straight line leaves only rounding residue once its line is removed, which
        // must not read as a period.
        assert_eq!(period(&[4.0; 50]), None);
        assert_eq!(detect(&Series::new([])), []);
    }

    /// #520 review (F-260): a noisy trend, 0.1 per row plus noise of σ 1 over 100 points
    /// (GEN-011 without spikes or sine), is not periodic. With only the mean removed, about
    /// half the seeds reported a period.
    #[test]
    fn f_260_a_noisy_ramp_is_not_periodic() {
        use crate::harness::generators::outliers::OutlierParams;
        use crate::harness::{Params, generate};
        let ramp = OutlierParams {
            points: 100,
            outliers: 0,
            amplitude: 0.0,
            slope: 0.1,
            ..OutlierParams::default()
        };
        let mut false_alarms = 0;
        for seed in 0..200 {
            let stimulus = generate(&Params::Outliers(ramp.clone()), seed).unwrap();
            let values = &stimulus.column("value").unwrap().values;
            if period(values).is_some() {
                false_alarms += 1;
            }
        }
        assert!(
            false_alarms <= 10,
            "{false_alarms}/200 noisy ramps called periodic"
        );
    }

    #[test]
    fn detrending_keeps_a_sine_and_flattens_a_line() {
        let line: Vec<f64> = (0..50).map(|i| 3.0 + 0.5 * f64::from(i)).collect();
        assert!(detrended(&line).iter().all(|r| r.abs() < 1e-9));
        let sine = wave(10.0, 100);
        let kept = detrended(&sine);
        let most = kept.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        assert!(most > 0.9, "{most}");
    }
}
