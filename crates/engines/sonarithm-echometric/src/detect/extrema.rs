// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `extrema` detector (C05-T02; FR-080, A11Y-23): the global minimum and maximum, and the
//! local extrema at each trend reversal.
//!
//! A reversal counts only when the series moves back from a running extreme by more than a
//! hysteresis band, a fraction of its value range, so noise does not make every wiggle an
//! extremum (ADR-0031). The same reversals give SDR-21's count and the trend segments
//! (C05-T03), so `extrema`, `trend-segments` and the lint agree on them.

use super::{Finding, Param, Params, Series};

/// The hysteresis band, as a fraction of the value range: a move back from a running extreme
/// smaller than this is not a reversal. INFER (ADR-0031): no design rule sets it; chosen so
/// the A08 reversal generator's (GEN-001) counts are found exactly, TODO(verify) with
/// C05-T03's precision and recall.
pub const HYSTERESIS: f64 = 0.1;

/// The parameters recorded with every `extrema` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("reversal-hysteresis")),
        ("hysteresis", Param::Number(HYSTERESIS)),
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

/// The reversals of `series` with a band of `hysteresis` times its value range, in order. A
/// flat series has none. The first and last values are never turns.
#[must_use]
pub fn turns(series: &Series, hysteresis: f64) -> Vec<Turn> {
    let values = series.values();
    let Some((lo, hi)) = series.bounds() else {
        return Vec::new();
    };
    let band = hysteresis * (hi - lo);
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
/// local extrema at its turns, in row order. Empty for an empty series; a constant series
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
    for turn in turns(series, HYSTERESIS) {
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
