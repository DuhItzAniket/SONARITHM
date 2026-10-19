// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scale tests: worked values, error cases, and the monotonicity and range properties of
//! A06-T01 (testing-strategy §3).

#![allow(
    clippy::float_cmp,
    reason = "exact end positions (0, 0.5, 1) are part of the scale contract"
)]

use proptest::prelude::*;

use super::*;

fn u(scale: &ContinuousScale, value: f64) -> f64 {
    scale.position(value).unwrap().u
}

#[test]
fn linear_log_sqrt_and_pow_place_worked_values() {
    let linear = ContinuousScale::new(Transform::Linear, [0.0, 10.0]).unwrap();
    assert_eq!(u(&linear, 5.0), 0.5);
    assert_eq!(u(&linear, 0.0), 0.0);
    assert_eq!(u(&linear, 10.0), 1.0);

    let log = ContinuousScale::new(Transform::Log, [1.0, 100.0]).unwrap();
    assert!((u(&log, 10.0) - 0.5).abs() < 1e-15);

    let sqrt = ContinuousScale::new(Transform::SQRT, [0.0, 100.0]).unwrap();
    assert_eq!(u(&sqrt, 25.0), 0.5);

    let square = ContinuousScale::new(Transform::Pow(2.0), [0.0, 10.0]).unwrap();
    assert_eq!(u(&square, 5.0), 0.25);
}

#[test]
fn symlog_is_symmetric_about_zero_and_handles_both_signs() {
    let symlog = ContinuousScale::new(Transform::Symlog, [-1000.0, 1000.0]).unwrap();
    assert_eq!(u(&symlog, 0.0), 0.5);
    for x in [0.5, 1.0, 10.0, 999.0] {
        let sum = u(&symlog, x) + u(&symlog, -x);
        assert!((sum - 1.0).abs() < 1e-15, "{x}: {sum}");
    }
    // Logarithmic far from zero: equal ratios give nearly equal steps.
    let step_low = u(&symlog, 100.0) - u(&symlog, 10.0);
    let step_high = u(&symlog, 1000.0) - u(&symlog, 100.0);
    assert!((step_low - step_high).abs() < 0.01);
}

#[test]
fn domains_come_from_the_data_or_include_zero() {
    let values = [3.0, f64::NAN, 7.0, f64::INFINITY, 5.0];
    let auto = ContinuousScale::fit(Transform::Linear, Domain::Auto, false, &values).unwrap();
    assert_eq!(auto.domain(), [3.0, 7.0]);
    let zero = ContinuousScale::fit(Transform::Linear, Domain::ZeroBased, false, &values).unwrap();
    assert_eq!(zero.domain(), [0.0, 7.0]);
    let negative = [-4.0, -2.0];
    let zero = ContinuousScale::fit(Transform::Linear, Domain::ZeroBased, false, &negative);
    assert_eq!(zero.unwrap().domain(), [-4.0, 0.0]);
    let explicit = ContinuousScale::fit(
        Transform::Linear,
        Domain::Explicit([0.0, 100.0]),
        false,
        &values,
    );
    assert_eq!(explicit.unwrap().domain(), [0.0, 100.0]);
}

#[test]
fn values_outside_the_domain_are_clamped_and_reported() {
    let scale = ContinuousScale::new(Transform::Linear, [0.0, 10.0]).unwrap();
    assert_eq!(
        scale.position(-1.0),
        Some(Position {
            u: 0.0,
            off_scale: OffScale::Below
        })
    );
    assert_eq!(
        scale.position(f64::INFINITY),
        Some(Position {
            u: 1.0,
            off_scale: OffScale::Above
        })
    );
    assert_eq!(scale.position(f64::NAN), None);
    let log = ContinuousScale::new(Transform::Log, [1.0, 10.0]).unwrap();
    assert_eq!(log.position(0.0).unwrap().off_scale, OffScale::Below);
    assert_eq!(log.position(-5.0).unwrap().u, 0.0);
}

#[test]
fn a_single_value_sits_at_the_centre() {
    let scale = ContinuousScale::fit(Transform::Log, Domain::Auto, false, &[4.0, 4.0]).unwrap();
    assert!(scale.is_degenerate());
    assert_eq!(u(&scale, 4.0), 0.5);
    assert_eq!(u(&scale, 3.0), 0.0);
    assert_eq!(scale.invert(0.7), 4.0);
}

#[test]
fn invalid_scales_are_rejected_with_a_reason() {
    assert_eq!(
        ContinuousScale::fit(Transform::Linear, Domain::Auto, false, &[f64::NAN]),
        Err(ScaleError::NoValues)
    );
    assert_eq!(
        ContinuousScale::fit(Transform::Log, Domain::Auto, false, &[0.0, 5.0]),
        Err(ScaleError::NonPositiveLogDomain { min: 0.0 })
    );
    assert_eq!(
        ContinuousScale::fit(Transform::Log, Domain::ZeroBased, false, &[2.0, 5.0]),
        Err(ScaleError::NonPositiveLogDomain { min: 0.0 })
    );
    assert_eq!(
        ContinuousScale::fit(Transform::Linear, Domain::Explicit([5.0, 5.0]), false, &[]),
        Err(ScaleError::InvalidDomain { min: 5.0, max: 5.0 })
    );
    assert_eq!(
        ContinuousScale::new(Transform::Pow(0.0), [0.0, 1.0]),
        Err(ScaleError::InvalidExponent(0.0))
    );
    assert_eq!(
        ContinuousScale::new(Transform::Pow(f64::NAN), [0.0, 1.0])
            .err()
            .map(|err| matches!(err, ScaleError::InvalidExponent(_))),
        Some(true)
    );
    assert_eq!(
        DiscreteScale::new(DiscreteKind::Band, []),
        Err(ScaleError::NoCategories)
    );
}

#[test]
fn nice_rounds_domains_outward_to_round_numbers() {
    let fit = |transform, values: &[f64]| {
        ContinuousScale::fit(transform, Domain::Auto, true, values)
            .unwrap()
            .domain()
    };
    assert_eq!(fit(Transform::Linear, &[0.123, 9.87]), [0.0, 10.0]);
    assert_eq!(fit(Transform::Linear, &[3.0, 97.0]), [0.0, 100.0]);
    assert_eq!(fit(Transform::Linear, &[-0.42, 0.37]), [-0.5, 0.4]);
    assert_eq!(fit(Transform::Log, &[3.0, 470.0]), [1.0, 1000.0]);
    // A degenerate domain stays as it is.
    assert_eq!(fit(Transform::Linear, &[2.5]), [2.5, 2.5]);
}

#[test]
fn quantile_positions_follow_ranks() {
    let scale = QuantileScale::fit(&[10.0, 1.0, 1000.0, 100.0, f64::NAN]).unwrap();
    let positions: Vec<f64> = [1.0, 10.0, 100.0, 1000.0]
        .iter()
        .map(|&v| scale.position(v).unwrap().u)
        .collect();
    let thirds = [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];
    for (got, want) in positions.iter().zip(thirds) {
        assert!((got - want).abs() < 1e-15);
    }
    // Between two data values, between their ranks; ties share the mean rank.
    assert!((scale.position(5.0).unwrap().u - 1.0 / 6.0).abs() < 1e-15);
    let tied = QuantileScale::fit(&[1.0, 1.0, 2.0]).unwrap();
    assert_eq!(tied.position(1.0).unwrap().u, 0.25);
    assert_eq!(scale.position(0.0).unwrap().off_scale, OffScale::Below);
    assert_eq!(QuantileScale::fit(&[]), Err(ScaleError::NoValues));
    assert_eq!(
        QuantileScale::fit(&[7.0]).unwrap().position(7.0),
        Some(Position::within(0.5))
    );
}

#[test]
fn band_and_ordinal_scales_place_categories() {
    let text = |s: &str| Category::Text(s.to_owned());
    let regions = categories_in_order_of_appearance(["north", "south", "north", "east"].map(text));
    assert_eq!(regions, ["north", "south", "east"].map(text));
    let band = DiscreteScale::new(DiscreteKind::Band, regions.clone()).unwrap();
    let positions: Vec<f64> = regions
        .iter()
        .map(|c| band.position(c).unwrap().u)
        .collect();
    assert_eq!(positions, [1.0 / 6.0, 0.5, 5.0 / 6.0]);
    assert_eq!(band.position(&text("west")), None);

    let ordinal = DiscreteScale::new(DiscreteKind::Ordinal, sorted_categories(regions)).unwrap();
    assert_eq!(ordinal.categories(), ["east", "north", "south"].map(text));
    assert_eq!(ordinal.position(&text("east")).unwrap().u, 0.0);
    assert_eq!(ordinal.position(&text("north")).unwrap().u, 0.5);
    assert_eq!(ordinal.position(&text("south")).unwrap().u, 1.0);
    let one = DiscreteScale::new(DiscreteKind::Ordinal, [text("only")]).unwrap();
    assert_eq!(one.position_of_index(0), 0.5);
}

#[test]
fn categories_order_numbers_before_text() {
    let sorted = sorted_categories([
        Category::Text("b".into()),
        Category::Number(2.0),
        Category::Text("a".into()),
        Category::Number(-1.0),
        Category::Number(2.0),
    ]);
    assert_eq!(
        sorted,
        [
            Category::Number(-1.0),
            Category::Number(2.0),
            Category::Text("a".into()),
            Category::Text("b".into()),
        ]
    );
}

#[test]
fn numeric_fits_cover_every_scale_kind() {
    let values = [2.0, 8.0, 4.0, 4.0];
    for kind in [
        ScaleKind::Linear,
        ScaleKind::Log,
        ScaleKind::Symlog,
        ScaleKind::Sqrt,
        ScaleKind::Pow(3.0),
        ScaleKind::Quantile,
        ScaleKind::Band,
        ScaleKind::Ordinal,
    ] {
        let scale = Scale::fit_numeric(kind, Domain::Auto, false, &values).unwrap();
        assert_eq!(scale.position(f64::NAN), None, "{kind:?}");
        let low = scale.position(2.0).unwrap().u;
        let high = scale.position(8.0).unwrap().u;
        assert!(low < high, "{kind:?}");
        assert_eq!(
            kind.is_discrete(),
            kind.transform().is_none() && kind != ScaleKind::Quantile
        );
    }
    let ordinal = Scale::fit_numeric(ScaleKind::Ordinal, Domain::Auto, false, &values).unwrap();
    assert_eq!(ordinal.position(4.0).unwrap().u, 0.5);
    assert_eq!(ordinal.position(5.0), None);
}

/// Continuous transforms with valid parameters.
fn transform() -> impl Strategy<Value = Transform> {
    prop_oneof![
        Just(Transform::Linear),
        Just(Transform::Symlog),
        Just(Transform::SQRT),
        (0.05..5.0_f64).prop_map(Transform::Pow),
    ]
}

/// A domain `[d0, d1]` with `d0 < d1`, over magnitudes typical of data.
fn domain() -> impl Strategy<Value = [f64; 2]> {
    (-1e6..1e6_f64, 1e-3..1e6_f64).prop_map(|(d0, span)| [d0, d0 + span])
}

/// A positive domain for log scales.
fn positive_domain() -> impl Strategy<Value = [f64; 2]> {
    (1e-6..1e6_f64, 1.0001..1e6_f64).prop_map(|(d0, ratio)| [d0, d0 * ratio])
}

/// A value anywhere, including off-scale magnitudes and infinities.
fn value() -> impl Strategy<Value = f64> {
    prop_oneof![
        8 => -3e6..3e6_f64,
        1 => Just(f64::INFINITY),
        1 => Just(f64::NEG_INFINITY),
        1 => Just(0.0),
    ]
}

fn check_monotonic(scale: &ContinuousScale, a: f64, b: f64) -> Result<(), TestCaseError> {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let (p, q) = (scale.position(lo).unwrap(), scale.position(hi).unwrap());
    prop_assert!((0.0..=1.0).contains(&p.u) && (0.0..=1.0).contains(&q.u));
    prop_assert!(p.u <= q.u, "{lo} → {} but {hi} → {}", p.u, q.u);
    let [d0, d1] = scale.domain();
    prop_assert_eq!(scale.position(d0).unwrap().u, 0.0);
    prop_assert_eq!(scale.position(d1).unwrap().u, 1.0);
    Ok(())
}

proptest! {
    /// A06-T01: every continuous scale is monotonic non-decreasing, stays in [0, 1] and maps
    /// the domain ends to 0 and 1.
    #[test]
    fn continuous_scales_are_monotonic_and_bounded(
        transform in transform(),
        domain in domain(),
        a in value(),
        b in value(),
    ) {
        let scale = ContinuousScale::new(transform, domain).unwrap();
        check_monotonic(&scale, a, b)?;
    }

    /// A06-T01: the same for log scales on positive domains.
    #[test]
    fn log_scales_are_monotonic_and_bounded(domain in positive_domain(), a in value(), b in value()) {
        let scale = ContinuousScale::new(Transform::Log, domain).unwrap();
        check_monotonic(&scale, a, b)?;
    }

    /// A06-T01: a scale fitted to the data spans exactly the data, and nice domains contain
    /// the data extent.
    #[test]
    fn fitted_domains_contain_the_data(
        transform in transform(),
        values in prop::collection::vec(-1e6..1e6_f64, 1..64),
        nice in any::<bool>(),
    ) {
        let scale = ContinuousScale::fit(transform, Domain::Auto, nice, &values).unwrap();
        let [d0, d1] = scale.domain();
        for &v in &values {
            prop_assert!(d0 <= v && v <= d1);
            prop_assert_eq!(scale.position(v).unwrap().off_scale, OffScale::Within);
        }
        if !nice {
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            prop_assert_eq!([d0, d1], [min, max]);
        }
    }

    /// A06-T01: `position` undoes `invert` up to rounding, so legends and announcements
    /// name the value a position stands for. (The other direction is ill-conditioned for
    /// steep `pow` scales near zero, where many values share one position.)
    #[test]
    fn position_undoes_invert(transform in transform(), domain in domain(), u in 0.0..=1.0_f64) {
        let scale = ContinuousScale::new(transform, domain).unwrap();
        let back = scale.position(scale.invert(u)).unwrap();
        prop_assert_eq!(back.off_scale, OffScale::Within);
        prop_assert!((back.u - u).abs() <= 1e-9, "{u} → {}", back.u);
    }

    /// A06-T01: quantile positions are monotonic, in [0, 1], and span it for distinct data.
    #[test]
    fn quantile_scales_are_monotonic_and_bounded(
        values in prop::collection::vec(-1e3..1e3_f64, 2..64),
        a in value(),
        b in value(),
    ) {
        let scale = QuantileScale::fit(&values).unwrap();
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let (p, q) = (scale.position(lo).unwrap().u, scale.position(hi).unwrap().u);
        prop_assert!((0.0..=1.0).contains(&p) && (0.0..=1.0).contains(&q));
        prop_assert!(p <= q);
        let [min, max] = scale.domain();
        let unique = |x: f64| values.iter().filter(|&&v| v == x).count() == 1;
        if unique(min) {
            prop_assert_eq!(scale.position(min).unwrap().u, 0.0);
        }
        if unique(max) {
            prop_assert_eq!(scale.position(max).unwrap().u, 1.0);
        }
    }

    /// A06-T01: band and ordinal positions rise strictly in scale order and stay in [0, 1];
    /// ordinal scales reach both ends, band scales neither.
    #[test]
    fn discrete_positions_rise_in_scale_order(
        labels in prop::collection::btree_set("[a-z]{1,6}", 2..20),
        ordinal in any::<bool>(),
    ) {
        let kind = if ordinal { DiscreteKind::Ordinal } else { DiscreteKind::Band };
        let categories: Vec<Category> = labels.into_iter().map(Category::Text).collect();
        let scale = DiscreteScale::new(kind, categories.clone()).unwrap();
        let positions: Vec<f64> = categories
            .iter()
            .map(|c| scale.position(c).unwrap().u)
            .collect();
        prop_assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        let (first, last) = (positions[0], positions[positions.len() - 1]);
        if ordinal {
            prop_assert_eq!((first, last), (0.0, 1.0));
        } else {
            prop_assert!(first > 0.0 && last < 1.0);
        }
    }
}
