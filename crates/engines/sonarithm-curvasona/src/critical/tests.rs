// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeMap;

use super::*;
use crate::expr::Limits;
use crate::scan::{ScanBudget, scan};
use crate::suite::{CURVES, Curve};
use crate::{Budgets, Grid};

fn features_of(text: &str, domain: [f64; 2], samples: u32) -> Vec<Feature> {
    let f = Expr::parse(text, &BTreeMap::new(), &Limits::DEFAULT).unwrap();
    let xs: Vec<f64> = Grid::new(domain, Some(samples), None, &Budgets::DEFAULT)
        .unwrap()
        .points()
        .collect();
    let found = scan(&f, &xs, None, &ScanBudget::DEFAULT);
    features(&Calculus::new(f), &xs, &found, 1000)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + b.abs())
}

fn xs_of(features: &[Feature], kind: FeatureKind) -> Vec<f64> {
    features
        .iter()
        .filter(|f| f.kind == kind)
        .map(|f| f.x)
        .collect()
}

fn limit(features: &[Feature], right: bool) -> Option<f64> {
    features
        .iter()
        .find(|f| f.kind == FeatureKind::HorizontalAsymptote { right })
        .and_then(|f| f.y)
}

fn check(curve: &Curve, samples: u32) -> Vec<String> {
    let found = features_of(curve.expr, curve.domain, samples);
    let mut failures = Vec::new();
    let mut compare = |what: &str, got: Vec<f64>, want: &[f64]| {
        let same = got.len() == want.len() && got.iter().zip(want).all(|(&g, &w)| near(g, w));
        if !same {
            failures.push(format!(
                "{} ({}), {samples} samples: {what} {got:?}, expected {want:?}",
                curve.name, curve.expr
            ));
        }
    };
    compare("zeros", xs_of(&found, FeatureKind::Zero), curve.zeros);
    compare("maxima", xs_of(&found, FeatureKind::Maximum), curve.maxima);
    compare("minima", xs_of(&found, FeatureKind::Minimum), curve.minima);
    if let Some(inflections) = curve.inflections {
        compare(
            "inflections",
            xs_of(&found, FeatureKind::Inflection),
            inflections,
        );
    }
    compare(
        "vertical asymptotes",
        xs_of(&found, FeatureKind::VerticalAsymptote),
        curve.poles,
    );
    compare("holes", xs_of(&found, FeatureKind::Hole), curve.holes);
    compare("jumps", xs_of(&found, FeatureKind::Jump), curve.jumps);
    for (right, want) in [(false, curve.left), (true, curve.right)] {
        let got = limit(&found, right);
        let same = match (got, want) {
            (Some(g), Some(w)) => near(g, w),
            (None, None) => true,
            _ => false,
        };
        if !same {
            failures.push(format!(
                "{} ({}), {samples} samples: limit {} {got:?}, expected {want:?}",
                curve.name,
                curve.expr,
                if right { "right" } else { "left" }
            ));
        }
    }
    failures
}

/// C03 exit (C03-T03): the 40-curve suite's analytically known zeros, extrema,
/// inflections, asymptotes, holes and jumps are found within 1e-9 (relative to 1 + |x|),
/// and nothing else is reported, at 1000 and 101 samples.
#[test]
fn a11y_28_the_suite_finds_every_critical_point() {
    let mut failures = Vec::new();
    for curve in &CURVES {
        for samples in [1000, 101] {
            failures.extend(check(curve, samples));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn extrema_and_inflections_carry_their_values() {
    let found = features_of("x^3 - 3*x", [-3.0, 3.0], 1000);
    let max = found
        .iter()
        .find(|f| f.kind == FeatureKind::Maximum)
        .unwrap();
    assert!(near(max.y.unwrap(), 2.0));
    let inflection = found
        .iter()
        .find(|f| f.kind == FeatureKind::Inflection)
        .unwrap();
    assert!(near(inflection.y.unwrap(), 0.0));
    let hole = features_of("sin(x)/x", [-1.0, 1.0], 100);
    let hole = hole.iter().find(|f| f.kind == FeatureKind::Hole).unwrap();
    assert!(near(hole.y.unwrap(), 1.0));
}

#[test]
fn every_label_is_a_short_text_equivalent() {
    for kind in [
        FeatureKind::Zero,
        FeatureKind::Maximum,
        FeatureKind::Minimum,
        FeatureKind::Inflection,
        FeatureKind::VerticalAsymptote,
        FeatureKind::HorizontalAsymptote { right: false },
        FeatureKind::HorizontalAsymptote { right: true },
        FeatureKind::Hole,
        FeatureKind::Jump,
        FeatureKind::Undefined,
        FeatureKind::Offscreen { above: true },
        FeatureKind::Offscreen { above: false },
    ] {
        let label = kind.label();
        assert!(!label.is_empty() && label.len() <= 40, "{label}");
    }
}
