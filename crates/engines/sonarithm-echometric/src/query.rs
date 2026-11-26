// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The query engine's statistics (C05-T06; A11Y-22, A11Y-24, FR-082; keyboard §7): mean,
//! median, mode, standard deviation, sum and count over one field's values in a scope of
//! rows: the series under the cursor today, a selection later (A11Y-22). Each answer cites
//! its rows and says how many of its values are estimates, so the wording can qualify it
//! (AGENTS rule 7). AUDAXIS words the answer; the app core's `nav_query` joins the two.
//!
//! Sums run in row order on IEEE arithmetic, so an answer is the same on every platform
//! (ADR-0013, ADR-0031).

use std::collections::BTreeMap;

use sonarithm_core::ids::{Generation, RowRange, TableId};
use sonarithm_core::query::{StatAnswer, Statistic, Unavailable};

use crate::detect::median;

/// Whether a field's values are measurements on a continuum or take repeated values, which
/// decides whether a mode means anything (keyboard §7: "Not meaningful for continuous
/// values"). The app core decides from the field's semantic type: nominal, ordinal, count
/// and binned fields are discrete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scale {
    /// Continuous measurements.
    Continuous,
    /// Categories, ranks, counts or bins.
    Discrete,
}

/// One row in a query's scope: its value (`None` when missing) and whether that value is an
/// estimate (not `exact`, `vector` or `user_edited`, 03 §4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Value {
    /// The row.
    pub row: u64,
    /// The value, `None` or non-finite when missing.
    pub value: Option<f64>,
    /// Whether the value is an estimate.
    pub estimated: bool,
}

/// What a query is asked over: one field's values in a scope of rows of one table.
#[derive(Clone, Copy, Debug)]
pub struct Scope<'a> {
    /// The table the rows belong to.
    pub table: TableId,
    /// Its generation.
    pub generation: Generation,
    /// Whether a mode applies.
    pub scale: Scale,
    /// The rows in scope, in row order.
    pub values: &'a [Value],
}

impl Scope<'_> {
    fn range(&self, start: u64, end: u64) -> RowRange {
        RowRange {
            table: self.table,
            generation: self.generation,
            start,
            end,
        }
    }

    /// `rows` (increasing) as the fewest ranges.
    fn ranges(&self, rows: impl IntoIterator<Item = u64>) -> Vec<RowRange> {
        let mut out: Vec<RowRange> = Vec::new();
        for row in rows {
            match out.last_mut() {
                Some(last) if last.end == row => last.end = row + 1,
                _ => out.push(self.range(row, row + 1)),
            }
        }
        out
    }
}

/// `statistic` over `scope`.
#[must_use]
pub fn answer(scope: &Scope<'_>, statistic: Statistic) -> StatAnswer {
    let used: Vec<&Value> = scope
        .values
        .iter()
        .filter(|v| v.value.is_some_and(f64::is_finite))
        .collect();
    let missing = (scope.values.len() - used.len()) as u64;
    let estimated = used.iter().filter(|v| v.estimated).count() as u64;
    let numbers: Vec<f64> = used.iter().filter_map(|v| v.value).collect();
    let all_rows = || scope.ranges(used.iter().map(|v| v.row));
    let mut out = if statistic == Statistic::Count {
        StatAnswer::value(statistic, numbers.len() as f64, all_rows())
    } else if numbers.is_empty() {
        StatAnswer::unavailable(statistic, Unavailable::NoValues)
    } else {
        match statistic {
            Statistic::Mean => StatAnswer::value(statistic, mean(&numbers), all_rows()),
            Statistic::Sum => StatAnswer::value(statistic, numbers.iter().sum(), all_rows()),
            Statistic::StandardDeviation => match standard_deviation(&numbers) {
                Some(sd) => StatAnswer::value(statistic, sd, all_rows()),
                None => StatAnswer::unavailable(statistic, Unavailable::NotMeaningful),
            },
            Statistic::Median => {
                let centre = median(&numbers);
                let rows = middle_rows(&used);
                StatAnswer::value(statistic, centre, scope.ranges(rows))
            }
            Statistic::Mode => mode(scope, &used),
            _ => StatAnswer::unavailable(statistic, Unavailable::NotMeaningful),
        }
    };
    out.count = numbers.len() as u64;
    out.missing = missing;
    out.estimated = estimated;
    out
}

fn mean(numbers: &[f64]) -> f64 {
    numbers.iter().sum::<f64>() / numbers.len() as f64
}

/// The sample standard deviation (n − 1), or `None` for fewer than two values.
fn standard_deviation(numbers: &[f64]) -> Option<f64> {
    if numbers.len() < 2 {
        return None;
    }
    let centre = mean(numbers);
    let squares: f64 = numbers.iter().map(|v| (v - centre) * (v - centre)).sum();
    Some((squares / (numbers.len() - 1) as f64).sqrt())
}

/// The rows of the middle value or values, in row order: one for an odd count, two for an
/// even one (ties broken by row).
fn middle_rows(used: &[&Value]) -> Vec<u64> {
    let mut sorted: Vec<(f64, u64)> = used
        .iter()
        .filter_map(|v| v.value.map(|x| (x, v.row)))
        .collect();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mid = sorted.len() / 2;
    let mut rows: Vec<u64> = if sorted.len() % 2 == 1 {
        sorted.get(mid).map(|&(_, r)| r).into_iter().collect()
    } else {
        [mid.checked_sub(1), Some(mid)]
            .into_iter()
            .flatten()
            .filter_map(|at| sorted.get(at).map(|&(_, r)| r))
            .collect()
    };
    rows.sort_unstable();
    rows
}

/// The mode of discrete values: the most frequent value (the smallest, when several are as
/// frequent, the rest in `ties`), citing its rows. Continuous values have no meaningful mode
/// (keyboard §7); discrete values that never repeat have none.
fn mode(scope: &Scope<'_>, used: &[&Value]) -> StatAnswer {
    if scope.scale == Scale::Continuous {
        return StatAnswer::unavailable(Statistic::Mode, Unavailable::NotMeaningful);
    }
    // Keyed by the value's bits: discrete values are exact, and `total_cmp` order is the
    // numeric order for them (-0.0 and 0.0 aside, which count as one).
    let mut counts: BTreeMap<u64, (f64, Vec<u64>)> = BTreeMap::new();
    for v in used {
        if let Some(x) = v.value {
            let x = if x == 0.0 { 0.0 } else { x };
            counts
                .entry(x.to_bits())
                .or_insert((x, Vec::new()))
                .1
                .push(v.row);
        }
    }
    let most = counts
        .values()
        .map(|(_, rows)| rows.len())
        .max()
        .unwrap_or(0);
    if most < 2 {
        return StatAnswer::unavailable(Statistic::Mode, Unavailable::NoRepeat);
    }
    let mut modes: Vec<(f64, &Vec<u64>)> = counts
        .values()
        .filter(|(_, rows)| rows.len() == most)
        .map(|(x, rows)| (*x, rows))
        .collect();
    modes.sort_by(|a, b| a.0.total_cmp(&b.0));
    let Some(&(value, rows)) = modes.first() else {
        return StatAnswer::unavailable(Statistic::Mode, Unavailable::NotMeaningful);
    };
    let mut out = StatAnswer::value(Statistic::Mode, value, scope.ranges(rows.iter().copied()));
    out.ties = modes.iter().skip(1).map(|&(x, _)| x).collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(xs: &[Option<f64>]) -> Vec<Value> {
        xs.iter()
            .enumerate()
            .map(|(i, &value)| Value {
                row: i as u64,
                value,
                estimated: i % 3 == 2,
            })
            .collect()
    }

    fn scope(values: &[Value], scale: Scale) -> Scope<'_> {
        Scope {
            table: TableId(1),
            generation: Generation(0),
            scale,
            values,
        }
    }

    #[test]
    fn count_reports_the_missing_values() {
        let v = values(&[Some(1.0), None, Some(3.0), Some(f64::NAN), Some(5.0)]);
        let a = answer(&scope(&v, Scale::Continuous), Statistic::Count);
        assert_eq!((a.value, a.count, a.missing), (Some(3.0), 3, 2));
    }

    #[test]
    fn the_median_of_an_even_count_cites_both_middle_rows() {
        let v = values(&[Some(4.0), Some(1.0), Some(3.0), Some(2.0)]);
        let a = answer(&scope(&v, Scale::Continuous), Statistic::Median);
        assert_eq!(a.value, Some(2.5));
        let rows: Vec<(u64, u64)> = a.rows.iter().map(|r| (r.start, r.end)).collect();
        assert_eq!(rows, [(2, 4)], "rows 2 and 3 hold 3 and 2");
    }

    #[test]
    fn the_mode_of_continuous_values_is_not_meaningful() {
        let v = values(&[Some(1.5), Some(1.5), Some(2.0)]);
        let a = answer(&scope(&v, Scale::Continuous), Statistic::Mode);
        assert_eq!(a.unavailable, Some(Unavailable::NotMeaningful));
        let d = answer(&scope(&v, Scale::Discrete), Statistic::Mode);
        assert_eq!(d.value, Some(1.5));
        assert_eq!(d.rows.len(), 1);
    }

    #[test]
    fn discrete_values_that_never_repeat_have_no_mode() {
        let v = values(&[Some(1.0), Some(2.0), Some(3.0)]);
        let a = answer(&scope(&v, Scale::Discrete), Statistic::Mode);
        assert_eq!(
            (a.value, a.unavailable),
            (None, Some(Unavailable::NoRepeat))
        );
        // The continuous case keeps keyboard §7's own reason.
        let c = answer(&scope(&v, Scale::Continuous), Statistic::Mode);
        assert_eq!(c.unavailable, Some(Unavailable::NotMeaningful));
    }

    #[test]
    fn a_tied_mode_reports_its_ties() {
        let v = values(&[Some(2.0), Some(1.0), Some(2.0), Some(1.0), Some(3.0)]);
        let a = answer(&scope(&v, Scale::Discrete), Statistic::Mode);
        assert_eq!((a.value, a.ties.clone()), (Some(1.0), vec![2.0]));
    }

    #[test]
    fn one_value_has_no_standard_deviation_and_none_has_no_mean() {
        let one = values(&[Some(7.0)]);
        let sd = answer(
            &scope(&one, Scale::Continuous),
            Statistic::StandardDeviation,
        );
        assert_eq!(sd.unavailable, Some(Unavailable::NotMeaningful));
        let none = values(&[None, None]);
        let m = answer(&scope(&none, Scale::Continuous), Statistic::Mean);
        assert_eq!((m.unavailable, m.missing), (Some(Unavailable::NoValues), 2));
    }

    #[test]
    fn estimates_are_counted_for_qualifying() {
        let v = values(&[Some(1.0), Some(2.0), Some(3.0), Some(4.0)]);
        let a = answer(&scope(&v, Scale::Continuous), Statistic::Mean);
        assert_eq!(
            (a.value, a.estimated),
            (Some(2.5), 1),
            "row 2 is an estimate"
        );
    }
}
