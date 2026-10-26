// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use proptest::prelude::*;

use super::ticks::civil_ms;
use super::*;

/// Anchors on an exact scale: `values` placed at `slope · t(value) + intercept`.
fn exact(kind: ScaleKind, values: &[f64], slope: f64, intercept: f64) -> Vec<Anchor> {
    values
        .iter()
        .map(|v| {
            let t = kind.transform(*v).unwrap_or(f64::NAN);
            Anchor::new(slope * t + intercept, *v)
        })
        .collect()
}

fn codes_of(diagnostics: &[Diagnostic]) -> Vec<&str> {
    diagnostics.iter().map(|d| d.code.as_str()).collect()
}

fn relative_error(found: f64, expected: f64, span: f64) -> f64 {
    (found - expected).abs() / expected.abs().max(span)
}

fn distinct_sorted(mut values: Vec<f64>) -> Vec<f64> {
    values.sort_by(f64::total_cmp);
    values.dedup_by(|a, b| (*a - *b).abs() < 1e-6 * b.abs().max(1.0));
    values
}

proptest! {
    /// A fit through exact anchors inverts within 1e-9 relative error (C01-T04).
    #[test]
    fn exact_linear_anchors_invert(
        values in prop::collection::vec(-1e6_f64..1e6, 2..12),
        slope in prop_oneof![-500.0_f64..-0.01, 0.01_f64..500.0],
        intercept in -1e4_f64..1e4,
    ) {
        let values = distinct_sorted(values);
        prop_assume!(values.len() >= 2);
        let span = values.last().unwrap() - values.first().unwrap();
        let anchors = exact(ScaleKind::Linear, &values, slope, intercept);
        let fit = Calibration::fit(ScaleKind::Linear, &anchors).unwrap();
        for anchor in &anchors {
            prop_assert!(relative_error(fit.value_at(anchor.position), anchor.value, span) < 1e-9);
        }
        prop_assert!(fit.residual() < 1e-9);
    }

    #[test]
    fn exact_log_anchors_invert(
        exponents in prop::collection::vec(-6.0_f64..9.0, 2..10),
        slope in 1.0_f64..400.0,
        intercept in -1e3_f64..1e3,
    ) {
        let values = distinct_sorted(exponents.iter().map(|e| libm::pow(10.0, *e)).collect());
        prop_assume!(values.len() >= 2);
        let anchors = exact(ScaleKind::Log, &values, slope, intercept);
        let fit = Calibration::fit(ScaleKind::Log, &anchors).unwrap();
        for anchor in &anchors {
            prop_assert!(relative_error(fit.value_at(anchor.position), anchor.value, 0.0) < 1e-9);
        }
    }

    #[test]
    fn exact_date_anchors_invert(
        days in prop::collection::vec(0_i64..40_000, 2..10),
        slope in 1e-9_f64..1e-5,
    ) {
        #[expect(clippy::cast_precision_loss, reason = "test dates")]
        let values = distinct_sorted(days.iter().map(|d| (d * 86_400_000) as f64).collect());
        prop_assume!(values.len() >= 2);
        let span = values.last().unwrap() - values.first().unwrap();
        let anchors = exact(ScaleKind::Date, &values, slope, 40.0);
        let fit = Calibration::fit(ScaleKind::Date, &anchors).unwrap();
        for anchor in &anchors {
            prop_assert!(relative_error(fit.value_at(anchor.position), anchor.value, span) < 1e-9);
        }
    }

    /// Residuals grow with injected jitter (C01-T04).
    #[test]
    fn residuals_grow_with_jitter(
        count in 3_usize..12,
        small in 0.001_f64..1.0,
        factor in 1.5_f64..20.0,
    ) {
        #[expect(clippy::cast_precision_loss, reason = "small counts")]
        let values: Vec<f64> = (0..count).map(|i| i as f64 * 10.0).collect();
        let jittered = |amplitude: f64| -> f64 {
            let anchors: Vec<Anchor> = exact(ScaleKind::Linear, &values, 4.0, 50.0)
                .into_iter()
                .enumerate()
                .map(|(i, a)| {
                    let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                    Anchor::new(a.position + sign * amplitude, a.value)
                })
                .collect();
            Calibration::fit(ScaleKind::Linear, &anchors).unwrap().residual()
        };
        let (a, b) = (jittered(small), jittered(small * factor));
        prop_assert!(a > 0.0);
        prop_assert!(b > a, "{b} should exceed {a}");
    }

    /// A log axis read as linear, and a linear axis read as log, are caught (C01-T04).
    #[test]
    fn scale_mismatches_are_caught(count in 3_usize..10, step in 5.0_f64..80.0) {
        #[expect(clippy::cast_precision_loss, reason = "small counts")]
        let positions: Vec<f64> = (0..count).map(|i| i as f64 * step).collect();
        let decades: Vec<Anchor> = positions
            .iter()
            .zip(0..)
            .map(|(p, i)| Anchor::new(*p, libm::pow(10.0, f64::from(i))))
            .collect();
        let found = check_scale(ScaleKind::Linear, &decades, "y").map(|d| d.code);
        prop_assert_eq!(found.as_ref().map(sonarithm_core::Code::as_str), Some("SNR-ENG-323"));
        prop_assert!(check_scale(ScaleKind::Log, &decades, "y").is_none());
        let steps: Vec<Anchor> = positions
            .iter()
            .zip(1..)
            .map(|(p, i)| Anchor::new(*p, f64::from(i) * 100.0))
            .collect();
        let found = check_scale(ScaleKind::Log, &steps, "y").map(|d| d.code);
        prop_assert_eq!(found.as_ref().map(sonarithm_core::Code::as_str), Some("SNR-ENG-323"));
        prop_assert!(check_scale(ScaleKind::Linear, &steps, "y").is_none());
        let auto = Calibration::fit_auto(&decades).unwrap();
        prop_assert_eq!(auto.kind(), ScaleKind::Log);
        prop_assert_eq!(Calibration::fit_auto(&steps).unwrap().kind(), ScaleKind::Linear);
    }
}

#[test]
fn the_monotonic_check_passes_and_fails() {
    let rising = [
        Anchor::new(0.0, 0.0),
        Anchor::new(10.0, 5.0),
        Anchor::new(20.0, 10.0),
    ];
    assert!(check_monotonic(&rising, "x").is_ok());
    // An SVG y axis: values fall as positions grow downwards. Still monotonic.
    let falling = [
        Anchor::new(300.0, 0.0),
        Anchor::new(200.0, 50.0),
        Anchor::new(100.0, 100.0),
    ];
    assert!(check_monotonic(&falling, "y").is_ok());
    let zigzag = [
        Anchor::new(0.0, 0.0),
        Anchor::new(10.0, 5.0),
        Anchor::new(20.0, 3.0),
    ];
    assert_eq!(
        check_monotonic(&zigzag, "x").unwrap_err().code.as_str(),
        "SNR-ENG-322"
    );
    let shared = [Anchor::new(0.0, 0.0), Anchor::new(0.0, 5.0)];
    assert_eq!(
        check_monotonic(&shared, "x").unwrap_err().code.as_str(),
        "SNR-ENG-322"
    );
}

#[test]
fn the_residual_check_passes_and_fails() {
    let values = [0.0, 10.0, 20.0, 30.0, 40.0];
    let clean = exact(ScaleKind::Linear, &values, 5.0, 0.0);
    let fit = Calibration::fit(ScaleKind::Linear, &clean).unwrap();
    assert_eq!(fit.check(&clean, "y"), Vec::new());
    assert_eq!(fit.confidence(), Confidence::CERTAIN);
    let mut rough = clean.clone();
    if let Some(a) = rough.get_mut(2) {
        a.position += 8.0; // 4 % of a 200-unit axis
    }
    let fit = Calibration::fit(ScaleKind::Linear, &rough).unwrap();
    assert_eq!(codes_of(&fit.check(&rough, "y")), ["SNR-ENG-325"]);
    assert!(f32::from(fit.confidence()) < 0.99);
}

#[test]
fn the_bounds_check_passes_and_fails() {
    let fit = Calibration::fit(
        ScaleKind::Linear,
        &[Anchor::new(100.0, 0.0), Anchor::new(500.0, 100.0)],
    )
    .unwrap();
    assert!(
        fit.check_bounds(&[100.0, 300.0, 501.0], (100.0, 500.0), 0.01, "x")
            .is_none()
    );
    let outside = fit
        .check_bounds(&[50.0, 300.0, 600.0], (500.0, 100.0), 0.01, "x")
        .unwrap();
    assert_eq!(outside.code.as_str(), "SNR-ENG-324");
    assert!(outside.message.starts_with("2 point(s)"));
}

#[test]
fn fits_that_cannot_be_made_are_refused() {
    let one = [Anchor::new(0.0, 1.0)];
    assert_eq!(
        Calibration::fit(ScaleKind::Linear, &one)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-320"
    );
    let same_value = [Anchor::new(0.0, 1.0), Anchor::new(5.0, 1.0)];
    assert_eq!(
        Calibration::fit(ScaleKind::Linear, &same_value)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-320"
    );
    let zero = [Anchor::new(0.0, 0.0), Anchor::new(5.0, 10.0)];
    assert_eq!(
        Calibration::fit(ScaleKind::Log, &zero)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-321"
    );
    let infinite = [Anchor::new(0.0, f64::INFINITY), Anchor::new(5.0, 10.0)];
    assert_eq!(
        Calibration::fit(ScaleKind::Linear, &infinite)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-320"
    );
}

#[test]
fn ticks_become_a_calibrated_date_axis() {
    use super::ticks::{Tick, date_anchors};
    let ticks: Vec<Tick> = (0..4)
        .map(|i| Tick::new(f64::from(i) * 100.0, format!("{}-01-01", 2018 + i)))
        .collect();
    let (anchors, unread) = date_anchors(&ticks, "x");
    assert!(unread.is_none());
    check_monotonic(&anchors, "x").unwrap();
    let fit = Calibration::fit(ScaleKind::Date, &anchors).unwrap();
    // Leap year 2020 makes the yearly steps uneven by one day in 366: a tiny residual.
    assert!(fit.residual() < 0.002, "{}", fit.residual());
    #[expect(clippy::cast_precision_loss, reason = "test dates")]
    let mid_2019 = civil_ms(2019, 7, 2) as f64;
    let at = fit.position_of(mid_2019).unwrap();
    assert!((at - 150.0).abs() < 0.5, "{at}");
}

#[test]
fn category_axes_read_the_nearest_label_within_its_band() {
    let axis = CategoryAxis::new(vec![
        (150.0, "B".to_owned()),
        (50.0, "A".to_owned()),
        (250.0, "C".to_owned()),
    ]);
    assert_eq!(axis.labels().collect::<Vec<_>>(), ["A", "B", "C"]);
    assert_eq!(axis.label_at(140.0), Some("B"));
    assert_eq!(axis.label_at(99.0), Some("A"));
    assert_eq!(axis.label_at(260.0), Some("C"));
    assert_eq!(axis.label_at(400.0), None);
    let single = CategoryAxis::new(vec![(10.0, "only".to_owned())]);
    assert_eq!(single.label_at(1e6), Some("only"));
}
