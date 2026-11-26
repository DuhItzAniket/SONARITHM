// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC's change points and periods against augurs (C05-T03, ADR-0031): different
//! methods (PELT against Bayesian online change-point detection; autocorrelation against a
//! periodogram), so they are compared on GEN-012's labelled series by what they find: change
//! points within one minimum segment of each other, periods within one point.
//!
//! augurs-changepoint 0.10.3's results (`NormalGammaDetector::default()`, change points after
//! row 0) are pinned in `fixtures/augurs-changepoint-0.10.3.json` for GEN-012 v1's seeds 0 to
//! 49, written once on 2026-11-26: its dependencies carry unmaintained-crate advisories that
//! `cargo deny` refuses (ADR-0031's fallback for a reference that cannot be a dependency).
//! augurs-seasons is still a live dev-dependency.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::cast_precision_loss,
    reason = "integration test; counts become rates"
)]

use std::collections::BTreeMap;

use augurs_seasons::{Detector as _, PeriodogramDetector};
use sonarithm_echometric::detect::{changepoints, periodicity};
use sonarithm_echometric::harness::generators::shifts::ShiftParams;
use sonarithm_echometric::harness::{Params, generate};

fn values(params: &ShiftParams, seed: u64) -> Vec<f64> {
    let stimulus = generate(&Params::Shifts(params.clone()), seed).unwrap();
    stimulus.column("value").unwrap().values.clone()
}

/// Whether every point of `a` has one of `b` within `tolerance`.
fn covered(a: &[usize], b: &[usize], tolerance: usize) -> bool {
    a.iter()
        .all(|x| b.iter().any(|y| x.abs_diff(*y) <= tolerance))
}

#[test]
fn changepoints_agree_with_augurs_bocpd() {
    let pinned: BTreeMap<String, Vec<usize>> =
        serde_json::from_str(include_str!("fixtures/augurs-changepoint-0.10.3.json")).unwrap();
    let mut agree = 0;
    for seed in 0..50u64 {
        let xs = values(&ShiftParams::default(), seed);
        let ours = changepoints::starts(&xs);
        let theirs = pinned.get(&seed.to_string()).unwrap();
        if covered(&ours, theirs, changepoints::MIN_SEGMENT)
            && covered(theirs, &ours, changepoints::MIN_SEGMENT)
        {
            agree += 1;
        }
    }
    println!("changepoints: agreement with augurs BOCPD on {agree}/50 GEN-012 series");
    assert!(agree >= 40, "{agree}/50");
}

#[test]
fn periods_agree_with_augurs_periodogram() {
    let params = ShiftParams {
        shifts: 0,
        period: Some(25),
        ..ShiftParams::default()
    };
    let mut agree = 0;
    for seed in 0..50 {
        let xs = values(&params, seed);
        let ours = periodicity::period(&xs).map(|(lag, _)| lag);
        let theirs = PeriodogramDetector::default().detect(&xs);
        if ours.is_some_and(|lag| theirs.iter().any(|&p| (p as usize).abs_diff(lag) <= 1)) {
            agree += 1;
        }
    }
    println!("periodicity: agreement with augurs's periodogram on {agree}/50 GEN-012 series");
    assert!(agree >= 45, "{agree}/50");
}
