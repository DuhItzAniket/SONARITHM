// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The query engine's statistics against statrs (C05-T06; FR-082, ADR-0031): the same mean,
//! median and sample standard deviation, within rounding, on any values.

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    reason = "integration test; indices become rows"
)]

use proptest::prelude::*;
use sonarithm_core::ids::{Generation, TableId};
use sonarithm_core::query::Statistic;
use sonarithm_echometric::query::{Scale, Scope, Value, answer};
use statrs::statistics::{Data, Distribution, Median};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()))
}

proptest! {
    #[test]
    fn mean_median_and_sd_match_statrs(xs in prop::collection::vec(-1e6f64..1e6, 2..120)) {
        let values: Vec<Value> = xs
            .iter()
            .enumerate()
            .map(|(i, &x)| Value { row: i as u64, value: Some(x), estimated: false })
            .collect();
        let scope = Scope {
            table: TableId(0),
            generation: Generation(0),
            scale: Scale::Continuous,
            values: &values,
        };
        let data = Data::new(xs.clone());
        let mean = answer(&scope, Statistic::Mean).value.unwrap();
        prop_assert!(close(mean, data.mean().unwrap()), "{mean}");
        let median = answer(&scope, Statistic::Median).value.unwrap();
        prop_assert!(close(median, data.median()), "{median}");
        let sd = answer(&scope, Statistic::StandardDeviation).value.unwrap();
        prop_assert!(close(sd, data.std_dev().unwrap()), "{sd}");
        let sum = answer(&scope, Statistic::Sum).value.unwrap();
        prop_assert!(close(sum, xs.iter().sum::<f64>()), "{sum}");
    }
}
