// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC's detectors on labelled synthetic data (C05-T02; FR-080, A11Y-23, the C05 exit
//! criterion): precision and recall per detector against the A08 generators' ground truth,
//! and their invariants as property tests (ADR-0031).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    reason = "integration test; clippy treats its helpers as non-test code, and counts become rates"
)]

use std::collections::BTreeSet;

use proptest::prelude::*;
use sonarithm_echometric::detect::{Series, extrema, outliers};
use sonarithm_echometric::harness::generators::outliers::OutlierParams;
use sonarithm_echometric::harness::generators::reversals::{ReversalParams, STUDIED_RATES};
use sonarithm_echometric::harness::{Params, Stimulus, Truth, generate};

fn series(stimulus: &Stimulus, column: &str) -> Series {
    let values = &stimulus.column(column).expect(column).values;
    Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
}

fn of(values: &[f64]) -> Series {
    Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
}

/// Precision and recall of `found` against `truth`.
fn rates(found: &BTreeSet<u64>, truth: &BTreeSet<u64>) -> (f64, f64) {
    let hits = found.intersection(truth).count() as f64;
    let precision = if found.is_empty() {
        1.0
    } else {
        hits / found.len() as f64
    };
    let recall = if truth.is_empty() {
        1.0
    } else {
        hits / truth.len() as f64
    };
    (precision, recall)
}

/// The labelled outliers of GEN-011 over 100 seeds: the measured precision and recall, which
/// C05-T02 records, and floors that a regression would break.
#[test]
fn c05_t02_outliers_precision_and_recall_on_gen_011() {
    let (mut found_all, mut truth_all) = (0usize, 0usize);
    let (mut hits_all, mut extra_all) = (0usize, 0usize);
    for seed in 0..100 {
        let stimulus = generate(&Params::Outliers(OutlierParams::default()), seed).unwrap();
        let Truth::Outliers(truth) = &stimulus.truth else {
            panic!("GEN-011 has outlier truth");
        };
        let truth: BTreeSet<u64> = truth.rows.iter().map(|&r| r as u64).collect();
        let found: BTreeSet<u64> = outliers::detect(&series(&stimulus, "value"))
            .iter()
            .map(|f| f.first)
            .collect();
        hits_all += found.intersection(&truth).count();
        extra_all += found.difference(&truth).count();
        found_all += found.len();
        truth_all += truth.len();
    }
    let precision = hits_all as f64 / found_all.max(1) as f64;
    let recall = hits_all as f64 / truth_all.max(1) as f64;
    println!(
        "outliers on GEN-011 (100 seeds, defaults): precision {precision:.4}, recall {recall:.4}, {extra_all} false positives"
    );
    assert!(precision >= 0.95, "precision {precision}");
    assert!(recall >= 0.95, "recall {recall}");
}

/// GEN-001's schematic trends (no noise): the local extrema are exactly its turning rows, for
/// every studied rate and reversal count, so SDR-21's count is the generator's.
#[test]
fn sdr_21_local_extrema_are_the_turning_rows_of_gen_001() {
    for rate in STUDIED_RATES {
        for reversals in 0..=3 {
            for seed in 0..10 {
                let params = ReversalParams::cell(rate, reversals);
                let stimulus = generate(&Params::TrendReversals(params), seed).unwrap();
                let Truth::Reversals(truth) = &stimulus.truth else {
                    panic!("GEN-001 has reversal truth");
                };
                let truth: BTreeSet<u64> =
                    truth.turning_rows.iter().map(|&r| r as u64).collect();
                let s = series(&stimulus, "price");
                let found: BTreeSet<u64> = extrema::turns(&s, extrema::HYSTERESIS)
                    .iter()
                    .map(|t| t.index as u64)
                    .collect();
                assert_eq!(
                    found, truth,
                    "rate {rate}, {reversals} reversals, seed {seed}"
                );
                assert_eq!(rates(&found, &truth), (1.0, 1.0));
            }
        }
    }
}

/// Every annotation parameter is recorded (RFC-0009 §1), in a stable order.
#[test]
fn c05_t01_detector_parameters_are_pinned() {
    let shown = |params: sonarithm_echometric::detect::Params| -> Vec<String> {
        params
            .iter()
            .map(|(name, value)| format!("{name}={value:?}"))
            .collect()
    };
    insta::assert_debug_snapshot!("extrema_params", shown(extrema::params()));
    insta::assert_debug_snapshot!("outliers_params", shown(outliers::params()));
}

proptest! {
    /// The robust z-score is unchanged by shifting and by positive scaling, so the outliers
    /// do not depend on units (ADR-0031).
    #[test]
    fn outliers_ignore_shift_and_scale(
        values in prop::collection::vec(-1000.0f64..1000.0, 5..60),
        shift in -1e3f64..1e3,
        scale in 0.01f64..100.0,
    ) {
        let rows = |vs: &[f64]| outliers::detect(&of(vs)).iter().map(|f| f.first).collect::<Vec<_>>();
        let moved: Vec<f64> = values.iter().map(|v| v * scale + shift).collect();
        let (a, b) = (rows(&values), rows(&moved));
        // Rounding can move a value that sits on the threshold; allow that, never more.
        prop_assert!(a.iter().filter(|r| !b.contains(r)).count() <= 1, "{a:?} vs {b:?}");
    }

    /// The global extremes are the series' minimum and maximum, at their first rows.
    #[test]
    fn global_extrema_are_the_minimum_and_maximum(
        values in prop::collection::vec(-1e6f64..1e6, 1..200),
    ) {
        let found = extrema::detect(&of(&values));
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        prop_assert_eq!(found[0].value, min);
        prop_assert_eq!(found[1].value, max);
        prop_assert_eq!(found[0].first as usize, values.iter().position(|&v| v == min).unwrap());
        prop_assert_eq!(found[1].first as usize, values.iter().position(|&v| v == max).unwrap());
    }

    /// Turns alternate between maxima and minima, in increasing order.
    #[test]
    fn turns_alternate(values in prop::collection::vec(-100.0f64..100.0, 2..200)) {
        let turns = extrema::turns(&of(&values), extrema::HYSTERESIS);
        for pair in turns.windows(2) {
            prop_assert_ne!(pair[0].maximum, pair[1].maximum);
            prop_assert!(pair[0].index < pair[1].index);
        }
    }

    /// The same series gives the same findings, byte for byte (ADR-0013).
    #[test]
    fn detectors_are_deterministic(values in prop::collection::vec(-1e3f64..1e3, 0..100)) {
        let s = of(&values);
        prop_assert_eq!(format!("{:?}", extrema::detect(&s)), format!("{:?}", extrema::detect(&s)));
        prop_assert_eq!(format!("{:?}", outliers::detect(&s)), format!("{:?}", outliers::detect(&s)));
    }
}
