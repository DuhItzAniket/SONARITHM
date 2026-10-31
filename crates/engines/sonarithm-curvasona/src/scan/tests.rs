// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeMap;

use proptest::prelude::*;

use super::*;
use crate::expr::Limits;
use crate::suite::{CURVES, Curve};
use crate::{Budgets, Grid};

fn parse(text: &str) -> Expr {
    Expr::parse(text, &BTreeMap::new(), &Limits::DEFAULT).unwrap()
}

fn grid(domain: [f64; 2], samples: u32) -> Vec<f64> {
    Grid::new(domain, Some(samples), None, &Budgets::DEFAULT)
        .unwrap()
        .points()
        .collect()
}

fn run(text: &str, domain: [f64; 2], samples: u32) -> Scan {
    scan(
        &parse(text),
        &grid(domain, samples),
        None,
        &ScanBudget::DEFAULT,
    )
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + b.abs())
}

fn of_kind(scan: &Scan, kind: SingularityKind) -> Vec<f64> {
    scan.singularities
        .iter()
        .filter(|s| s.kind == kind)
        .map(|s| s.x)
        .collect()
}

fn matches(found: &[f64], expected: &[f64]) -> bool {
    found.len() == expected.len() && found.iter().zip(expected).all(|(&f, &e)| near(f, e))
}

fn check(curve: &Curve, samples: u32) -> Result<(), String> {
    let scan = run(curve.expr, curve.domain, samples);
    let fail = |what: &str, found: &dyn std::fmt::Debug, expected: &dyn std::fmt::Debug| {
        Err(format!(
            "{} ({}), {samples} samples: {what} {found:?}, expected {expected:?}",
            curve.name, curve.expr
        ))
    };
    if scan.truncated {
        return fail("truncated", &scan.evaluations, &"within budget");
    }
    for (kind, expected, what) in [
        (SingularityKind::Pole, curve.poles, "poles"),
        (SingularityKind::Hole, curve.holes, "holes"),
        (SingularityKind::Jump, curve.jumps, "jumps"),
    ] {
        let found = of_kind(&scan, kind);
        if !matches(&found, expected) {
            return fail(what, &found, &expected);
        }
    }
    let spans: Vec<[f64; 2]> = scan.undefined.iter().map(|s| [s.start, s.end]).collect();
    let same = spans.len() == curve.undefined.len()
        && spans
            .iter()
            .zip(curve.undefined)
            .all(|(f, e)| near(f[0], e[0]) && near(f[1], e[1]));
    if !same {
        return fail("undefined spans", &spans, &curve.undefined);
    }
    Ok(())
}

/// C03-T02 acceptance: on the 40-curve suite no known pole, hole, jump or undefined span is
/// missed, and nothing else is reported, at the default 1000 samples and at coarse grids.
#[test]
fn a11y_28_the_suite_finds_every_singularity_and_undefined_span() {
    let mut failures = Vec::new();
    for curve in &CURVES {
        for samples in [1000, 101, 37] {
            if let Err(message) = check(curve, samples) {
                failures.push(message);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn poles_say_how_the_function_approaches_them() {
    let tangent = run("tan(x)", [0.0, 3.0], 100);
    let pole = tangent.singularities[0];
    assert_eq!(
        (pole.left, pole.right),
        (Approach::PlusInfinity, Approach::MinusInfinity)
    );
    let log = run("ln(x)", [-1.0, 1.0], 100).singularities[0];
    assert_eq!(
        (log.left, log.right),
        (Approach::Undefined, Approach::MinusInfinity)
    );
    let one_sided = run("exp(1/x)", [-1.0, 1.0], 100).singularities[0];
    assert_eq!(one_sided.left, Approach::Value(0.0));
    assert_eq!(one_sided.right, Approach::PlusInfinity);
    let hole = run("sin(x)/x", [-1.0, 1.0], 100).singularities[0];
    assert!(matches!(hole.left, Approach::Value(v) if (v - 1.0).abs() < 1e-9));
    let jump = run("floor(x)", [0.5, 1.5], 100).singularities[0];
    assert_eq!(
        (jump.left, jump.right),
        (Approach::Value(0.0), Approach::Value(1.0))
    );
}

#[test]
fn offscreen_spans_follow_the_window() {
    let f = parse("x^2");
    let scan = scan(
        &f,
        &grid([-3.0, 3.0], 61),
        Some([0.0, 4.0]),
        &ScanBudget::DEFAULT,
    );
    let spans: Vec<(f64, f64, bool)> = scan
        .offscreen
        .iter()
        .map(|o| (o.span.start, o.span.end, o.above))
        .collect();
    assert_eq!(spans.len(), 2, "{spans:?}");
    assert!(near(spans[0].0, -3.0) && near(spans[0].1, -2.0) && spans[0].2);
    assert!(near(spans[1].0, 2.0) && near(spans[1].1, 3.0) && spans[1].2);
    let pole = scan_with_window("1/x", [-2.0, 2.0], [-1.0, 1.0]);
    let sides: Vec<bool> = pole.offscreen.iter().map(|o| o.above).collect();
    assert_eq!(sides, [false, true], "{:?}", pole.offscreen);
    assert!(near(pole.offscreen[0].span.start, -1.0));
    assert!(near(pole.offscreen[1].span.end, 1.0));
}

fn scan_with_window(text: &str, domain: [f64; 2], window: [f64; 2]) -> Scan {
    scan(
        &parse(text),
        &grid(domain, 200),
        Some(window),
        &ScanBudget::DEFAULT,
    )
}

#[test]
fn the_budget_holds_and_is_reported() {
    let budget = ScanBudget {
        max_evaluations: 500,
        max_features: 1000,
    };
    let scan = scan(
        &parse("tan(x) + 1/(x - 1)"),
        &grid([-5.0, 5.0], 400),
        None,
        &budget,
    );
    assert!(scan.truncated);
    assert!(scan.evaluations <= 500);
    let few = ScanBudget {
        max_evaluations: 1_000_000,
        max_features: 3,
    };
    let floor = scan_budget("floor(x)", [-10.5, 10.5], &few);
    assert!(floor.truncated);
    assert!(floor.singularities.len() <= 3);
}

fn scan_budget(text: &str, domain: [f64; 2], budget: &ScanBudget) -> Scan {
    scan(&parse(text), &grid(domain, 500), None, budget)
}

proptest! {
    /// A pole and a hole anywhere in the domain are found at any grid size, and the scan is
    /// the same every time (C03-T02).
    #[test]
    fn poles_and_holes_are_never_missed(a in -4.0..4.0_f64, b in -4.0..4.0_f64, samples in 2_u32..1500) {
        prop_assume!((a - b).abs() > 1e-3 && a.abs() < 3.99 && b.abs() < 3.99);
        let text = format!("(x - ({b}))/((x - ({a}))*(x - ({b})))");
        let f = parse(&text);
        let xs = grid([-4.0, 4.0], samples.max(3));
        let first = scan(&f, &xs, None, &ScanBudget::DEFAULT);
        prop_assert_eq!(&first, &scan(&f, &xs, None, &ScanBudget::DEFAULT));
        prop_assert!(!first.truncated);
        prop_assert!(matches(&of_kind(&first, SingularityKind::Pole), &[a]), "{:?}", first);
        prop_assert!(matches(&of_kind(&first, SingularityKind::Hole), &[b]), "{:?}", first);
        prop_assert!(first.undefined.is_empty());
    }

    /// A narrow undefined span anywhere is found with its edges.
    #[test]
    fn narrow_undefined_spans_are_never_missed(c in -3.0..3.0_f64, width in 1e-4..0.5_f64, samples in 3_u32..400) {
        let text = format!("sqrt((x - ({c}))^2 - {})", width * width);
        let scan = run(&text, [-4.0, 4.0], samples);
        prop_assert_eq!(scan.undefined.len(), 1, "{:?}", scan);
        let span = scan.undefined[0];
        prop_assert!((span.start - (c - width)).abs() < 1e-7 && (span.end - (c + width)).abs() < 1e-7, "{:?}", span);
    }
}
