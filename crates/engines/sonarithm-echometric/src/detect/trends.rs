// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `trend-segments` detector (C05-T03; FR-080, A11Y-23, SDR-21): the runs between a
//! series' reversals, each rising or falling. The reversals are [`super::extrema::turns`]
//! with the same hysteresis band, so the segments, the local extrema and SDR-21's reversal
//! count always agree (ADR-0031).

use super::extrema::{self, HYSTERESIS};
use super::{Finding, Param, Params, Series};

/// The parameters recorded with every `trend-segments` finding (RFC-0009 §1).
#[must_use]
pub fn params() -> Params {
    Params::from([
        ("method", Param::Text("reversal-hysteresis")),
        ("hysteresis", Param::Number(HYSTERESIS)),
    ])
}

/// How many times `series` reverses (SDR-21): its turns beyond the hysteresis band.
#[must_use]
pub fn reversals(series: &Series) -> usize {
    extrema::turns(series, HYSTERESIS).len()
}

/// The trend segments of `series`, in play order: from its first value to each turn and on to
/// its last, labelled `rising` or `falling` by the change to the next turn, which is the
/// value. Segments partition the series: each covers its points up to the one before the
/// next turn, and the last runs to the end, so no point belongs to two (AUDAXIS uses them as
/// the navigation tree's segments). A series that never moves beyond the band is one `flat`
/// segment.
#[must_use]
pub fn detect(series: &Series) -> Vec<Finding> {
    let rows = series.rows();
    let values = series.values();
    if values.len() < 2 {
        return Vec::new();
    }
    let mut bounds = vec![0usize];
    bounds.extend(extrema::turns(series, HYSTERESIS).iter().map(|t| t.index));
    bounds.push(values.len() - 1);
    let band = series
        .bounds()
        .map_or(0.0, |(lo, hi)| HYSTERESIS * (hi - lo));
    let last_index = values.len() - 1;
    bounds
        .windows(2)
        .filter_map(|pair| {
            let (&start, &end) = (pair.first()?, pair.get(1)?);
            let change = values.get(end)? - values.get(start)?;
            // The turn starts the next segment; the last segment keeps the final point.
            let through = if end == last_index { end } else { end - 1 };
            let label = if change > band {
                "rising"
            } else if change < -band {
                "falling"
            } else {
                "flat"
            };
            Some(Finding {
                label,
                first: *rows.get(start)?,
                last: *rows.get(through.max(start))?,
                value: change,
                score: None,
            })
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
    fn a_zigzag_is_rising_and_falling_segments_between_its_turns() {
        let found = detect(&series(&[0.0, 10.0, 0.0, 10.0]));
        let shown: Vec<(&str, u64, u64)> =
            found.iter().map(|f| (f.label, f.first, f.last)).collect();
        assert_eq!(
            shown,
            [("rising", 0, 0), ("falling", 1, 1), ("rising", 2, 3)]
        );
        assert_eq!(reversals(&series(&[0.0, 10.0, 0.0, 10.0])), 2);
    }

    #[test]
    fn segments_cover_the_series_end_to_end() {
        let values: Vec<f64> = (0..50).map(|i| f64::from(i % 10)).collect();
        let found = detect(&series(&values));
        assert_eq!(found.first().map(|f| f.first), Some(0));
        assert_eq!(found.last().map(|f| f.last), Some(49));
        for pair in found.windows(2) {
            assert_eq!(
                pair[0].last + 1,
                pair[1].first,
                "a partition: contiguous, no overlap"
            );
        }
    }

    #[test]
    fn a_flat_series_is_one_flat_segment() {
        let found = detect(&series(&[3.0, 3.0, 3.0]));
        assert_eq!(found.iter().map(|f| f.label).collect::<Vec<_>>(), ["flat"]);
    }
}
