// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC's detectors (C05-T01 to T03; 07 ECHOMETRIC, ADR-0031): extrema and outliers
//! so far; change points, periodicity and trend segments follow in C05-T03.
//!
//! Each detector is a pure function of a [`Series`]: the finite values of one field, in row
//! order, with the rows they came from. It uses IEEE arithmetic and [`sonarithm_core::math`]
//! only, sorts with `total_cmp` and sums in row order, so it gives the same findings on every
//! platform (ADR-0013, ADR-0031). Every threshold is recorded in the finding's parameters
//! (RFC-0009 §1) and is INFER: no design rule sets it (ADR-0031).

pub mod extrema;
pub mod outliers;

use std::collections::BTreeMap;

/// The values of one field to analyze: `values[i]` is the value of row `rows[i]`. Missing
/// and non-finite values are left out by [`Series::new`], so a detector never sees them.
#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    rows: Vec<u64>,
    values: Vec<f64>,
}

impl Series {
    /// The finite values among `points`, which are `(row, value)` pairs in row order.
    #[must_use]
    pub fn new(points: impl IntoIterator<Item = (u64, Option<f64>)>) -> Self {
        let (rows, values) = points
            .into_iter()
            .filter_map(|(row, value)| value.filter(|v| v.is_finite()).map(|v| (row, v)))
            .unzip();
        Self { rows, values }
    }

    /// The rows, in order.
    #[must_use]
    pub fn rows(&self) -> &[u64] {
        &self.rows
    }

    /// The values, one per row.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// How many values there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The smallest and largest value, or `None` when empty.
    #[must_use]
    pub fn bounds(&self) -> Option<(f64, f64)> {
        let mut values = self.values.iter().copied();
        let first = values.next()?;
        Some(values.fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v))))
    }
}

/// A detector parameter's value, as recorded in an annotation (RFC-0009 §1).
#[derive(Clone, Debug, PartialEq)]
pub enum Param {
    /// A number, such as a threshold.
    Number(f64),
    /// A name, such as the method.
    Text(&'static str),
}

/// The parameters a detector used, by name; ordered, so they serialize the same way.
pub type Params = BTreeMap<&'static str, Param>;

/// What a detector found: a row range and the value it reports.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    /// The stable English label, also the catalogue key's last part (`insight-<analyzer>-<label>`,
    /// F-191): `maximum`, `local minimum`, `outlier`, …
    pub label: &'static str,
    /// The first row it covers.
    pub first: u64,
    /// The last row it covers (equal to `first` for a point).
    pub last: u64,
    /// The value it reports: the extreme value, or the outlier's value.
    pub value: f64,
    /// A score where the detector has one: the outlier's robust z-score.
    pub score: Option<f64>,
}

/// The median of `values`, which must not be empty: the middle value, or the mean of the two
/// middle values for an even count. Sorts a copy with `total_cmp`.
#[must_use]
pub(crate) fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    match (
        sorted.len() % 2,
        sorted.get(mid),
        mid.checked_sub(1).and_then(|m| sorted.get(m)),
    ) {
        (1, Some(&middle), _) => middle,
        (_, Some(&upper), Some(&lower)) => f64::midpoint(lower, upper),
        _ => f64::NAN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_series_keeps_only_finite_values_with_their_rows() {
        let series = Series::new([
            (0, Some(1.0)),
            (1, None),
            (2, Some(f64::NAN)),
            (3, Some(f64::INFINITY)),
            (4, Some(-2.0)),
        ]);
        assert_eq!(series.rows(), [0, 4]);
        assert_eq!(series.values(), [1.0, -2.0]);
        assert_eq!(series.bounds(), Some((-2.0, 1.0)));
        assert_eq!(Series::new([]).bounds(), None);
    }

    #[test]
    fn the_median_of_an_even_count_is_the_mean_of_the_middle_two() {
        assert!((median(&[3.0, 1.0, 2.0]) - 2.0).abs() < f64::EPSILON);
        assert!((median(&[4.0, 1.0, 3.0, 2.0]) - 2.5).abs() < f64::EPSILON);
        assert!((median(&[7.0]) - 7.0).abs() < f64::EPSILON);
    }
}
