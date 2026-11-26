// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The answer to a statistics query (C05-T06; A11Y-22, A11Y-24, keyboard-and-navigation §7):
//! the core types ECHOMETRIC computes and AUDAXIS words, so neither engine depends on the
//! other (AGENTS rule 4). The app core's `nav_query` passes one from the first to the second.

use serde::{Deserialize, Serialize};

use crate::ids::RowRange;

/// A statistic of the query menu's "Statistics" section (keyboard §7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Statistic {
    /// The arithmetic mean.
    Mean,
    /// The middle value, or the mean of the two middle values.
    Median,
    /// The most frequent value, for discrete values only.
    Mode,
    /// The sample standard deviation (n − 1).
    StandardDeviation,
    /// The sum.
    Sum,
    /// How many values there are.
    Count,
}

/// Why a statistic has no value; each has its own message (keyboard §7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Unavailable {
    /// No usable values: all are missing, or the scope is empty.
    NoValues,
    /// The statistic does not apply: the mode of continuous values, the mode of values that
    /// never repeat, the standard deviation of a single value.
    NotMeaningful,
}

/// A statistic over a field's values, with what it cites (A11Y-24) and how much of it is
/// estimated (AGENTS rule 7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatAnswer {
    /// The statistic asked for.
    pub statistic: Statistic,
    /// Its value, or `None` with the reason in [`StatAnswer::unavailable`].
    pub value: Option<f64>,
    /// Why there is no value, when there is none.
    pub unavailable: Option<Unavailable>,
    /// The rows it cites: the median's middle row or rows, the mode's rows, or the rows it was
    /// computed over (as ranges).
    pub rows: Vec<RowRange>,
    /// The values it used.
    pub count: u64,
    /// The rows in scope without a usable value ("365 points (2 missing)").
    pub missing: u64,
    /// Of the values used, how many are estimates (not `exact`, `vector` or `user_edited`,
    /// 03 §4), so the answer can be qualified.
    pub estimated: u64,
    /// The mode's other values when several are as frequent, increasing.
    pub ties: Vec<f64>,
}

impl StatAnswer {
    /// An answer of `value`, citing `rows`.
    #[must_use]
    pub fn value(statistic: Statistic, value: f64, rows: Vec<RowRange>) -> Self {
        Self {
            statistic,
            value: Some(value),
            unavailable: None,
            rows,
            count: 0,
            missing: 0,
            estimated: 0,
            ties: Vec::new(),
        }
    }

    /// An answer with no value, for `reason`.
    #[must_use]
    pub fn unavailable(statistic: Statistic, reason: Unavailable) -> Self {
        Self {
            statistic,
            value: None,
            unavailable: Some(reason),
            rows: Vec::new(),
            count: 0,
            missing: 0,
            estimated: 0,
            ties: Vec::new(),
        }
    }
}
