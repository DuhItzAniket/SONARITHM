// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC's robust z-scores against statrs's median (ADR-0031): the same median and
//! median absolute deviation, so the same outliers, within rounding.

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    reason = "integration test; indices become rows"
)]

use proptest::prelude::*;
use sonarithm_echometric::detect::{Series, outliers};
use statrs::statistics::{Data, Median};

fn series(values: &[f64]) -> Series {
    Series::new(values.iter().enumerate().map(|(i, &v)| (i as u64, Some(v))))
}

/// The robust z-scores from statrs's median and MAD (Iglewicz & Hoaglin 1993), with the
/// same zero-MAD rule as ECHOMETRIC's (the scaled mean absolute deviation).
fn reference_scores(values: &[f64]) -> Option<Vec<f64>> {
    if values.len() < outliers::MIN_VALUES {
        return None;
    }
    let centre = Data::new(values.to_vec()).median();
    let deviations: Vec<f64> = values.iter().map(|v| (v - centre).abs()).collect();
    let mad = Data::new(deviations.clone()).median();
    let scale = if mad > 0.0 {
        mad / 0.6745
    } else {
        deviations.iter().sum::<f64>() / deviations.len() as f64 * 1.253_314
    };
    (scale > 0.0).then(|| values.iter().map(|v| (v - centre) / scale).collect())
}

proptest! {
    #[test]
    fn robust_z_scores_match_statrs(values in prop::collection::vec(-1e4f64..1e4, 0..80)) {
        let ours = outliers::scores(&series(&values));
        let theirs = reference_scores(&values);
        prop_assert_eq!(ours.is_some(), theirs.is_some());
        if let (Some(ours), Some(theirs)) = (ours, theirs) {
            for (a, b) in ours.iter().zip(&theirs) {
                prop_assert!((a - b).abs() <= 1e-9 * (1.0 + b.abs()), "{a} vs {b}");
            }
        }
    }
}
