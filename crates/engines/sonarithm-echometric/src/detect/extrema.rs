// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `extrema` detector (C05-T02; FR-080, A11Y-23): the global minimum and maximum, and the
//! local extrema at each trend reversal.
//!
//! A reversal counts only when the series moves back from a running extreme by more than a
//! hysteresis band, a fraction of its robust value range (the 5th to 95th percentile), so
//! noise does not make every wiggle an extremum and one spike does not widen the band until
//! it hides every other reversal (ADR-0031, F-209). The first and last values are never
//! turns: a turn needs values on both sides. The same reversals give SDR-21's count and the trend segments
//! (C05-T03), so `extrema`, `trend-segments` and the lint agree on them.

use super::{Finding, Param, Params, Series};

/// The hysteresis band, as a fraction of the value range: a move back from a running extreme
/// smaller than this is not a reversal. INFER (ADR-0031): no design rule sets it; chosen so
/// the A08 reversal generator's (GEN-001) counts are found exactly, TODO(verify) with
/// C05-T03's precision and recall.
pub const HYSTERESIS: f64 = 0.1;

/// The percentiles whose distance is the range the band is a fraction of: the 5th and
/// 95th, so a few spikes (up to 5 % of the values at each end) do not set the band. When
/// they are equal (a flat series with a spike), the full range is used, so the spike is
/// still a turn. INFER (ADR-0031, F-209; #506 review).
pub const RANGE_PERCENTILES: (f64, f64) = (0.05, 0.95);

/// The parameters recorded with every `extrema` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("reversal-hysteresis")),
        ("hysteresis", Param::Number(HYSTERESIS)),
        ("rangeLow", Param::Number(RANGE_PERCENTILES.0)),
        ("rangeHigh", Param::Number(RANGE_PERCENTILES.1)),
    ])
}

/// Where the series turns: the index of each local extreme and whether it is a maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turn {
    /// The index into the series.
    pub index: usize,
    /// A local maximum (else a local minimum).
    pub maximum: bool,
}

/// The `at`-th quantile (0 to 1) of `sorted`, which must not be empty, interpolated linearly
/// between the closest ranks.
fn quantile(sorted: &[f64], at: f64) -> f64 {
    let last = sorted.len().saturating_sub(1);
    #[expect(
        clippy::cast_precision_loss,
        reason = "series lengths are far below 2^52"
    )]
    let position = at * last as f64;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "position is in [0, last]"
    )]
    let below = (position.floor() as usize).min(last);
    let above = (below + 1).min(last);
    #[expect(clippy::cast_precision_loss, reason = "as above")]
    let fraction = position - below as f64;
    let (Some(&low), Some(&high)) = (sorted.get(below), sorted.get(above)) else {
        return f64::NAN;
    };
    low + fraction * (high - low)
}

/// The range the band is a fraction of: from the 5th to the 95th percentile
/// ([`RANGE_PERCENTILES`]), or the full range when those are equal. `None` for no values.
#[must_use]
pub fn robust_range(series: &Series) -> Option<f64> {
    let (lo, hi) = series.bounds()?;
    let mut sorted = series.values().to_vec();
    sorted.sort_by(f64::total_cmp);
    let (low, high) = RANGE_PERCENTILES;
    let robust = quantile(&sorted, high) - quantile(&sorted, low);
    Some(if robust > 0.0 { robust } else { hi - lo })
}

/// The reversals of `series` with a band of `hysteresis` times its robust value range
/// ([`robust_range`]), in order. A flat series has none. The first and last values are never
/// turns.
#[must_use]
pub fn turns(series: &Series, hysteresis: f64) -> Vec<Turn> {
    let values = series.values();
    let Some(range) = robust_range(series) else {
        return Vec::new();
    };
    let band = hysteresis * range;
    if band.is_nan() || band <= 0.0 {
        return Vec::new();
    }
    let mut turns = Vec::new();
    // The running extreme since the last turn, and the direction away from that turn:
    // `None` until the series has moved by more than the band from its start.
    let mut extreme = 0usize;
    let mut rising: Option<bool> = None;
    for (index, &value) in values.iter().enumerate().skip(1) {
        let at = values.get(extreme).copied().unwrap_or(value);
        match rising {
            None => {
                let start = values.first().copied().unwrap_or(value);
                if value - start > band {
                    rising = Some(true);
                    extreme = index;
                } else if start - value > band {
                    rising = Some(false);
                    extreme = index;
                } else if (value > at && value > start) || (value < at && value < start) {
                    extreme = index;
                }
            }
            Some(true) => {
                if value > at {
                    extreme = index;
                } else if at - value > band {
                    turns.push(Turn {
                        index: extreme,
                        maximum: true,
                    });
                    rising = Some(false);
                    extreme = index;
                }
            }
            Some(false) => {
                if value < at {
                    extreme = index;
                } else if value - at > band {
                    turns.push(Turn {
                        index: extreme,
                        maximum: false,
                    });
                    rising = Some(true);
                    extreme = index;
                }
            }
        }
    }
    turns
}

/// The global minimum and maximum of `series` (the first row of each, on ties), then the
/// local extrema at its turns, in row order. A turn at the global extreme's own row is not
/// listed again as a local one (#506 review). Empty for an empty series; a constant series
/// has its one value as both minimum and maximum.
#[must_use]
pub fn detect(series: &Series) -> Vec<Finding> {
    let rows = series.rows();
    let values = series.values();
    let mut found = Vec::new();
    let mut lowest: Option<(usize, f64)> = None;
    let mut highest: Option<(usize, f64)> = None;
    for (index, &value) in values.iter().enumerate() {
        if lowest.is_none_or(|(_, v)| value < v) {
            lowest = Some((index, value));
        }
        if highest.is_none_or(|(_, v)| value > v) {
            highest = Some((index, value));
        }
    }
    let point = |label, index: usize, value| {
        rows.get(index).map(|&row| Finding {
            label,
            first: row,
            last: row,
            value,
            score: None,
        })
    };
    found.extend(lowest.and_then(|(i, v)| point("minimum", i, v)));
    found.extend(highest.and_then(|(i, v)| point("maximum", i, v)));
    let global = [lowest, highest].map(|extreme| extreme.map(|(index, _)| index));
    for turn in turns(series, HYSTERESIS) {
        if global.contains(&Some(turn.index)) {
            continue;
        }
        let label = if turn.maximum {
            "local maximum"
        } else {
            "local minimum"
        };
        if let Some(&value) = values.get(turn.index) {
            found.extend(point(label, turn.index, value));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(values: &[f64]) -> Series {
        Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
    }

    #[test]
    fn the_global_extremes_are_the_first_rows_that_reach_them() {
        let found = detect(&series(&[2.0, 5.0, 1.0, 5.0, 1.0]));
        assert_eq!(found[0].label, "minimum");
        assert_eq!((found[0].first, found[0].value), (2, 1.0));
        assert_eq!(found[1].label, "maximum");
        assert_eq!((found[1].first, found[1].value), (1, 5.0));
    }

    #[test]
    fn a_wiggle_inside_the_band_is_not_a_turn() {
        // Range 10, band 1: the 0.5 dip at row 2 is noise; the fall after row 4 is a turn.
        let s = series(&[0.0, 5.0, 4.5, 6.0, 10.0, 3.0, 2.0]);
        let found = turns(&s, HYSTERESIS);
        assert_eq!(
            found,
            [Turn {
                index: 4,
                maximum: true
            }]
        );
    }

    #[test]
    fn a_zigzag_turns_at_each_peak_and_trough() {
        let s = series(&[0.0, 10.0, 0.0, 10.0, 0.0]);
        let found: Vec<(usize, bool)> = turns(&s, HYSTERESIS)
            .into_iter()
            .map(|t| (t.index, t.maximum))
            .collect();
        assert_eq!(found, [(1, true), (2, false), (3, true)]);
    }

    /// #506 review, F-209: one spike does not widen the band until it hides the reversals.
    /// With the full range, the spike of 100 makes the band 10 and the swings of 5 vanish.
    #[test]
    fn f_209_one_spike_does_not_hide_the_reversals() {
        let mut values: Vec<f64> = (0..40)
            .map(|i| if (i / 5) % 2 == 0 { 0.0 } else { 5.0 })
            .collect();
        values[22] = 100.0;
        let found = turns(&series(&values), HYSTERESIS);
        assert!(found.len() >= 6, "{found:?}");
        // A flat series with one spike still turns at the spike.
        let mut flat = vec![1.0; 30];
        flat[12] = 9.0;
        let found = turns(&series(&flat), HYSTERESIS);
        assert_eq!(found.iter().map(|t| t.index).collect::<Vec<_>>(), [12]);
    }

    #[test]
    fn the_robust_range_ignores_a_few_spikes() {
        let mut values: Vec<f64> = (0..100).map(f64::from).collect();
        values[50] = 10_000.0;
        let range = robust_range(&series(&values)).unwrap();
        assert!((range - 90.0).abs() < 1.0, "{range}");
        assert_eq!(robust_range(&series(&[])), None);
    }

    /// #506 review: the global maximum's own turn is not listed again as a local maximum,
    /// and the other peak still is.
    #[test]
    fn a_global_extreme_is_listed_once() {
        let found: Vec<(&str, u64)> = detect(&series(&[0.0, 10.0, 0.0, 8.0, 0.0]))
            .iter()
            .map(|f| (f.label, f.first))
            .collect();
        assert_eq!(
            found,
            [
                ("minimum", 0),
                ("maximum", 1),
                ("local minimum", 2),
                ("local maximum", 3)
            ]
        );
    }

    #[test]
    fn a_flat_or_empty_series_has_no_turns() {
        assert_eq!(turns(&series(&[3.0, 3.0, 3.0]), HYSTERESIS), []);
        assert_eq!(turns(&series(&[]), HYSTERESIS), []);
        let found = detect(&series(&[3.0, 3.0]));
        assert_eq!(
            found.len(),
            2,
            "one value is both the minimum and the maximum"
        );
    }
}
