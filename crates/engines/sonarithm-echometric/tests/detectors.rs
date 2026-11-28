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
use sonarithm_echometric::detect::{Series, changepoints, extrema, outliers, periodicity, trends};
use sonarithm_echometric::harness::generators::outliers::OutlierParams;
use sonarithm_echometric::harness::generators::reversals::{ReversalParams, STUDIED_RATES};
use sonarithm_echometric::harness::generators::shifts::ShiftParams;
use sonarithm_echometric::harness::{Params, Stimulus, Truth, generate};

fn series(stimulus: &Stimulus, column: &str) -> Series {
    let values = &stimulus.column(column).expect(column).values;
    Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
}

fn of(values: &[f64]) -> Series {
    Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
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

/// GEN-001's schematic trends (no noise): the local extrema are its turning rows, for every
/// studied rate and reversal count (a tie between two rows aside), so SDR-21's count is the
/// generator's.
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
                let truth: Vec<usize> = truth.turning_rows.clone();
                let s = series(&stimulus, "price");
                let values = s.values();
                let found: Vec<usize> = extrema::turns(&s, extrema::HYSTERESIS)
                    .iter()
                    .map(|t| t.index)
                    .collect();
                // SDR-21's count is the generator's, exactly. A turn may sit one row from
                // the generator's when the two rows tie (a peak between two hourly points:
                // the generator rounds half up, the detector keeps the first).
                assert_eq!(
                    found.len(),
                    truth.len(),
                    "rate {rate}, {reversals} reversals, seed {seed}: {found:?} vs {truth:?}"
                );
                for (&got, &want) in found.iter().zip(&truth) {
                    assert!(
                        got == want || (got.abs_diff(want) == 1 && values[got] == values[want]),
                        "rate {rate}, {reversals} reversals, seed {seed}: turn at {got}, truth {want}"
                    );
                }
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
    insta::assert_debug_snapshot!("changepoints_params", shown(changepoints::params()));
    insta::assert_debug_snapshot!("periodicity_params", shown(periodicity::params()));
    insta::assert_debug_snapshot!("trend_segments_params", shown(trends::params()));
}

/// GEN-012's level shifts over 100 seeds: a change point within one minimum segment of a true
/// shift is a hit, each truth matched once. The measured rates are printed for C05-T03's
/// record; the floors catch a regression.
#[test]
fn c05_t03_changepoints_precision_and_recall_on_gen_012() {
    let (mut hits, mut found_all, mut truth_all) = (0usize, 0usize, 0usize);
    for seed in 0..100 {
        let stimulus = generate(&Params::Shifts(ShiftParams::default()), seed).unwrap();
        let Truth::Shifts(truth) = &stimulus.truth else {
            panic!("GEN-012 has shift truth");
        };
        let found: Vec<u64> = changepoints::detect(&series(&stimulus, "value"))
            .iter()
            .map(|f| f.first)
            .collect();
        let mut unmatched: Vec<u64> = truth.change_rows.iter().map(|&r| r as u64).collect();
        for row in &found {
            if let Some(at) = unmatched
                .iter()
                .position(|t| t.abs_diff(*row) <= changepoints::MIN_SEGMENT as u64)
            {
                unmatched.remove(at);
                hits += 1;
            }
        }
        found_all += found.len();
        truth_all += truth.change_rows.len();
    }
    let precision = hits as f64 / found_all.max(1) as f64;
    let recall = hits as f64 / truth_all.max(1) as f64;
    println!(
        "changepoints on GEN-012 (100 seeds, defaults): precision {precision:.4}, recall {recall:.4}"
    );
    assert!(precision >= 0.9, "precision {precision}");
    assert!(recall >= 0.9, "recall {recall}");
}

/// GEN-012 with a 25-point sine and no shifts: the period is found within a point; with
/// noise alone, none is reported.
#[test]
fn c05_t03_periodicity_on_gen_012() {
    let periodic = ShiftParams {
        shifts: 0,
        period: Some(25),
        ..ShiftParams::default()
    };
    let noise = ShiftParams {
        shifts: 0,
        ..ShiftParams::default()
    };
    let (mut right, mut false_alarms) = (0usize, 0usize);
    for seed in 0..100 {
        let stimulus = generate(&Params::Shifts(periodic.clone()), seed).unwrap();
        let found = periodicity::period(series(&stimulus, "value").values());
        if found.is_some_and(|(lag, _)| lag.abs_diff(25) <= 1) {
            right += 1;
        }
        let stimulus = generate(&Params::Shifts(noise.clone()), seed).unwrap();
        if periodicity::period(series(&stimulus, "value").values()).is_some() {
            false_alarms += 1;
        }
    }
    // The realistic false alarm (#520 review, F-260): a noisy trend, 0.1 per row plus noise
    // of σ 1 over 100 points (GEN-011 without spikes or sine).
    let ramp = OutlierParams {
        points: 100,
        outliers: 0,
        amplitude: 0.0,
        slope: 0.1,
        ..OutlierParams::default()
    };
    let mut trend_alarms = 0usize;
    for seed in 0..100 {
        let stimulus = generate(&Params::Outliers(ramp.clone()), seed).unwrap();
        if periodicity::period(series(&stimulus, "value").values()).is_some() {
            trend_alarms += 1;
        }
    }
    println!(
        "periodicity on GEN-012 (100 seeds): period found {right}/100, false alarms {false_alarms}/100; noisy trend (GEN-011, slope 0.1): false alarms {trend_alarms}/100"
    );
    assert!(right >= 95, "{right}/100");
    assert!(false_alarms <= 5, "{false_alarms}/100");
    assert!(trend_alarms <= 5, "{trend_alarms}/100 noisy trends called periodic");
}

/// SDR-21: trend segments are the reversals plus one, and the reversal count is GEN-001's.
#[test]
fn sdr_21_trend_segments_follow_the_reversals_of_gen_001() {
    for rate in STUDIED_RATES {
        for reversals in 0..=3 {
            for seed in 0..10 {
                let params = ReversalParams::cell(rate, reversals);
                let stimulus = generate(&Params::TrendReversals(params), seed).unwrap();
                let s = series(&stimulus, "price");
                assert_eq!(
                    trends::reversals(&s),
                    reversals as usize,
                    "rate {rate}, seed {seed}"
                );
                let segments = trends::detect(&s);
                assert_eq!(segments.len(), reversals as usize + 1);
                for pair in segments.windows(2) {
                    assert_ne!(pair[0].label, pair[1].label, "directions alternate");
                }
            }
        }
    }
}

/// #507 (F-207): the outliers' ends against the interior, on GEN-011 with spikes forced at
/// both ends, over flat, sloped and periodic bases, with the ends skipped and with Theil–Sen
/// end lines. The measured rates are printed for F-207; only the interior has floors here,
/// so the ends are reported as measured, not tuned to pass.
#[test]
fn f_207_outlier_ends_against_the_interior() {
    let bases = [
        ("flat", 0.0, 0.0),
        ("sloped", 0.0, 0.05),
        ("periodic", 2.0, 0.0),
    ];
    for (base, amplitude, slope) in bases {
        for ends in [outliers::EndFit::Skip, outliers::EndFit::TheilSen] {
            let (mut end_hits, mut end_found, mut end_truth) = (0usize, 0usize, 0usize);
            let (mut mid_hits, mut mid_found, mut mid_truth) = (0usize, 0usize, 0usize);
            for seed in 0..100 {
                let params = OutlierParams {
                    amplitude,
                    slope,
                    ends: true,
                    ..OutlierParams::default()
                };
                let stimulus = generate(&Params::Outliers(params), seed).unwrap();
                let Truth::Outliers(truth) = &stimulus.truth else {
                    panic!("GEN-011 has outlier truth");
                };
                let last = (stimulus.rows() - 1) as u64;
                let is_end = |row: u64| row == 0 || row == last;
                let truth: BTreeSet<u64> = truth.rows.iter().map(|&r| r as u64).collect();
                let found: BTreeSet<u64> = outliers::detect_with(&series(&stimulus, "value"), ends)
                    .iter()
                    .map(|f| f.first)
                    .collect();
                for row in &found {
                    let hit = truth.contains(row);
                    if is_end(*row) {
                        end_found += 1;
                        end_hits += usize::from(hit);
                    } else {
                        mid_found += 1;
                        mid_hits += usize::from(hit);
                    }
                }
                end_truth += truth.iter().filter(|r| is_end(**r)).count();
                mid_truth += truth.iter().filter(|r| !is_end(**r)).count();
            }
            let rate = |hits: usize, of: usize| hits as f64 / of.max(1) as f64;
            println!(
                "F-207 {base:>8} {ends:?}: ends precision {:.4} recall {:.4}; interior precision {:.4} recall {:.4}",
                rate(end_hits, end_found),
                rate(end_hits, end_truth),
                rate(mid_hits, mid_found),
                rate(mid_hits, mid_truth),
            );
            assert!(
                rate(mid_hits, mid_truth) >= 0.95,
                "{base} {ends:?} interior recall"
            );
            if ends == outliers::EndFit::Skip {
                assert_eq!(end_found, 0, "skipped ends are never reported");
            }
            // The other half: with spikes forced at both ends, an end finding is always a
            // hit, so the precision above cannot show false alarms. The same bases and seeds
            // with no end spikes give the false-alarm rate of the ordinary end values.
            let mut false_alarms = 0usize;
            for seed in 0..100 {
                let params = OutlierParams {
                    amplitude,
                    slope,
                    ..OutlierParams::default()
                };
                let stimulus = generate(&Params::Outliers(params), seed).unwrap();
                let last = (stimulus.rows() - 1) as u64;
                false_alarms += outliers::detect_with(&series(&stimulus, "value"), ends)
                    .iter()
                    .filter(|f| f.first == 0 || f.first == last)
                    .count();
            }
            println!(
                "F-207 {base:>8} {ends:?}: false alarms at ordinary ends {false_alarms} of 200 ({:.4})",
                rate(false_alarms, 200),
            );
        }
    }
}

proptest! {
    /// Change points never make a segment shorter than the minimum, and the same series gives
    /// the same ones (ADR-0013).
    #[test]
    fn changepoints_keep_the_minimum_segment(values in prop::collection::vec(-100.0f64..100.0, 0..150)) {
        let found = changepoints::starts(&values);
        prop_assert_eq!(&found, &changepoints::starts(&values));
        let mut bounds = vec![0];
        bounds.extend(&found);
        bounds.push(values.len());
        for pair in bounds.windows(2) {
            prop_assert!(found.is_empty() || pair[1] - pair[0] >= changepoints::MIN_SEGMENT);
        }
    }

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
        prop_assert_eq!(found[0].first, values.iter().position(|&v| v == min).unwrap() as u64);
        prop_assert_eq!(found[1].first, values.iter().position(|&v| v == max).unwrap() as u64);
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
