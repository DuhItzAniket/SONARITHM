// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `changepoints` detector (C05-T03; FR-080, A11Y-23): where a series' level shifts.
//! PELT (Killick, Fearnhead & Eckley 2012) finds the segmentation that minimises the summed
//! squared deviations from each segment's mean plus a penalty per change, exactly, pruning
//! the candidates that can no longer win (ADR-0031). The penalty scales with the noise,
//! estimated robustly from the first differences, so the same shift is found in any unit.

use sonarithm_core::math;

use super::{Finding, Param, Params, Series, median};

/// The penalty per change, in units of the noise variance times ln n (a BIC-style penalty
/// for a change in mean). INFER (ADR-0031): no design rule sets it.
pub const PENALTY: f64 = 2.0;

/// The fewest values a segment may have. INFER (ADR-0031).
pub const MIN_SEGMENT: usize = 5;

/// The normal distribution's 75th percentile, which scales a MAD to a standard deviation.
const MAD_SCALE: f64 = 0.6745;

/// The parameters recorded with every `changepoints` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("pelt-l2")),
        ("penalty", Param::Number(PENALTY)),
        ("minSegment", Param::Number(MIN_SEGMENT as f64)),
    ])
}

/// Prefix sums of the values and their squares, for a segment's cost in constant time. The
/// values are centred on their mean first: at a large offset (about 1e8) with small noise,
/// `q − s²/n` on the raw values loses its digits and a flat series grew false changes
/// (#520 review).
struct Sums {
    sum: Vec<f64>,
    squares: Vec<f64>,
    offset: f64,
}

impl Sums {
    fn new(values: &[f64]) -> Self {
        let offset = if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        };
        let mut sum = Vec::with_capacity(values.len() + 1);
        let mut squares = Vec::with_capacity(values.len() + 1);
        let (mut s, mut q) = (0.0, 0.0);
        sum.push(s);
        squares.push(q);
        for &v in values {
            let v = v - offset;
            s += v;
            q += v * v;
            sum.push(s);
            squares.push(q);
        }
        Self {
            sum,
            squares,
            offset,
        }
    }

    /// The squared deviations from the mean of `values[start..end]`.
    fn cost(&self, start: usize, end: usize) -> f64 {
        let n = (end - start) as f64;
        let s =
            self.sum.get(end).copied().unwrap_or(0.0) - self.sum.get(start).copied().unwrap_or(0.0);
        let q = self.squares.get(end).copied().unwrap_or(0.0)
            - self.squares.get(start).copied().unwrap_or(0.0);
        (q - s * s / n).max(0.0)
    }

    /// The mean of `values[start..end]`.
    fn mean(&self, start: usize, end: usize) -> f64 {
        let s =
            self.sum.get(end).copied().unwrap_or(0.0) - self.sum.get(start).copied().unwrap_or(0.0);
        self.offset + s / (end - start) as f64
    }
}

/// The noise variance, from the MAD of the first differences: a level shift moves one
/// difference and does not inflate it (σ² = (MAD / 0.6745)² / 2).
fn noise_variance(values: &[f64]) -> f64 {
    let differences: Vec<f64> = values
        .windows(2)
        .map(|pair| match pair {
            [a, b] => b - a,
            _ => 0.0,
        })
        .collect();
    if differences.is_empty() {
        return 0.0;
    }
    let centre = median(&differences);
    let deviations: Vec<f64> = differences.iter().map(|d| (d - centre).abs()).collect();
    let sigma = median(&deviations) / MAD_SCALE;
    if sigma > 0.0 {
        return sigma * sigma / 2.0;
    }
    // No noise to measure: a noise-free or integer-plateau series, whose differences are
    // mostly exactly 0. The smallest step it takes stands in for the noise, so any sustained
    // shift between segments of at least MIN_SEGMENT values is a change, and a series that
    // never moves has none (#520 review). INFER (ADR-0031).
    let smallest = differences
        .iter()
        .map(|d| d.abs())
        .filter(|d| *d > 0.0)
        .fold(f64::INFINITY, f64::min);
    if smallest.is_finite() {
        smallest * smallest / 2.0
    } else {
        0.0
    }
}

/// The indices where a new segment starts, increasing: PELT over `values` with the
/// penalty [`PENALTY`] · σ² · ln n and segments of at least [`MIN_SEGMENT`] values. Empty for
/// a series too short to split or that never changes. Without measurable noise (the
/// differences' MAD is 0), σ² comes from the series' smallest step.
#[must_use]
pub fn starts(values: &[f64]) -> Vec<usize> {
    let n = values.len();
    if n < 2 * MIN_SEGMENT {
        return Vec::new();
    }
    let variance = noise_variance(values);
    if !variance.is_finite() || variance <= 0.0 {
        return Vec::new();
    }
    let beta = PENALTY * variance * math::ln(n as f64);
    let sums = Sums::new(values);
    // best[t]: the least cost of values[..t]; last[t]: where its final segment starts.
    let mut best = vec![f64::INFINITY; n + 1];
    let mut last = vec![0usize; n + 1];
    if let Some(first) = best.first_mut() {
        *first = -beta;
    }
    let mut candidates: Vec<usize> = vec![0];
    for t in MIN_SEGMENT..=n {
        let mut chosen: Option<(f64, usize)> = None;
        for &tau in &candidates {
            if t - tau < MIN_SEGMENT {
                continue;
            }
            let total = best.get(tau).copied().unwrap_or(f64::INFINITY) + sums.cost(tau, t) + beta;
            // Candidates are visited in increasing start order, so a strict `<` keeps the
            // earliest start on a tie: the result is the same however costs round.
            if chosen.is_none_or(|(cost, _)| total < cost) {
                chosen = Some((total, tau));
            }
        }
        if let Some((cost, tau)) = chosen
            && let (Some(b), Some(l)) = (best.get_mut(t), last.get_mut(t))
        {
            *b = cost;
            *l = tau;
        }
        let bound = best.get(t).copied().unwrap_or(f64::INFINITY);
        // PELT's pruning: a start whose cost without the penalty already exceeds the best
        // can never be optimal later (Killick et al. 2012, with K = 0 for this cost).
        candidates.retain(|&tau| {
            t - tau < MIN_SEGMENT
                || best.get(tau).copied().unwrap_or(f64::INFINITY) + sums.cost(tau, t) <= bound
        });
        if t + MIN_SEGMENT <= n {
            candidates.push(t);
        }
    }
    let mut found = Vec::new();
    let mut at = n;
    while at > 0 {
        let start = last.get(at).copied().unwrap_or(0);
        if start > 0 {
            found.push(start);
        }
        at = start;
    }
    found.reverse();
    found
}

/// The change points of `series`, in row order: the first row of each new segment, with
/// the new segment's mean as the value and the shift from the segment before as the score.
#[must_use]
pub fn detect(series: &Series) -> Vec<Finding> {
    let values = series.values();
    let found = starts(values);
    let sums = Sums::new(values);
    let mut bounds = found.clone();
    bounds.push(values.len());
    let mut previous = 0usize;
    let mut out = Vec::with_capacity(found.len());
    for (index, &start) in found.iter().enumerate() {
        let end = bounds.get(index + 1).copied().unwrap_or(values.len());
        let before = sums.mean(previous, start);
        let after = sums.mean(start, end);
        if let Some(&row) = series.rows().get(start) {
            out.push(Finding {
                label: "change point",
                first: row,
                last: row,
                value: after,
                score: Some(after - before),
            });
        }
        previous = start;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(values: &[f64]) -> Series {
        Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
    }

    /// #520 review: a noise-free step and an integer plateau are changes, though the MAD of
    /// their differences is 0; a constant series has none.
    #[test]
    fn a_noise_free_step_is_a_change() {
        let step: Vec<f64> = [vec![0.0; 20], vec![10.0; 20]].concat();
        assert_eq!(starts(&step), [20]);
        let plateau: Vec<f64> = [vec![3.0; 15], vec![7.0; 15], vec![3.0; 15]].concat();
        assert_eq!(starts(&plateau), [15, 30]);
        assert_eq!(starts(&[5.0; 40]), Vec::<usize>::new());
    }

    /// #520 review: centring keeps the cost precise at a large offset, so a flat series has
    /// no change and a step has exactly one, as without the offset.
    #[test]
    fn a_large_offset_changes_nothing() {
        let offset = 1e8;
        let flat: Vec<f64> = (0..60).map(|i| offset + wiggle(i) * 0.01).collect();
        assert_eq!(starts(&flat), Vec::<usize>::new());
        let step: Vec<f64> = (0..60)
            .map(|i| offset + if i < 30 { 0.0 } else { 1.0 } + wiggle(i) * 0.01)
            .collect();
        assert_eq!(starts(&step), [30]);
        let found = detect(&series(&step));
        assert!(
            (found[0].value - (offset + 1.0)).abs() < 0.01,
            "{:?}",
            found[0]
        );
    }

    /// A small deterministic wiggle, so the noise estimate is not zero.
    fn wiggle(i: usize) -> f64 {
        [0.3, -0.2, 0.1, -0.4, 0.2, -0.1, 0.4, -0.3][i % 8]
    }

    #[test]
    fn one_level_shift_is_found_at_its_row() {
        let values: Vec<f64> = (0..40)
            .map(|i| if i < 20 { 0.0 } else { 10.0 } + wiggle(i))
            .collect();
        let found = detect(&series(&values));
        assert_eq!(found.iter().map(|f| f.first).collect::<Vec<_>>(), [20]);
        assert!(
            found[0]
                .score
                .is_some_and(|shift| (shift - 10.0).abs() < 0.5)
        );
    }

    #[test]
    fn a_noisy_flat_series_has_no_change_points() {
        let values: Vec<f64> = (0..60).map(wiggle).collect();
        assert_eq!(detect(&series(&values)), []);
    }

    #[test]
    fn segments_are_never_shorter_than_the_minimum() {
        let mut values: Vec<f64> = (0..40).map(wiggle).collect();
        values[2] = 50.0;
        let found = starts(&values);
        let mut bounds = vec![0];
        bounds.extend(&found);
        bounds.push(values.len());
        for pair in bounds.windows(2) {
            assert!(pair[1] - pair[0] >= MIN_SEGMENT, "{found:?}");
        }
    }

    #[test]
    fn a_short_series_is_not_split() {
        assert_eq!(
            starts(&[0.0, 1.0, 0.0, 10.0, 10.0, 11.0]),
            Vec::<usize>::new()
        );
    }
}
