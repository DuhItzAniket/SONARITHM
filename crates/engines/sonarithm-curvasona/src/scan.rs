// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Singular points and spans of a sampled function (C03-T02, A11Y-28): poles, holes and
//! jumps, regions where the function is undefined, and spans off a y-window.
//!
//! The grid alone misses a pole or a hole between two samples, so the scan works from the
//! expression's [guards](crate::expr::Expr::guards): every discontinuity of a function in
//! the language sits at a zero of one of them. Each guard is sampled on the grid; sign
//! changes are bisected to the last bit, and a local minimum of its magnitude is refined by
//! golden-section search, which finds double zeros (`1/x^2`) and pairs of zeros inside one
//! grid step (a narrow undefined region). Each zero is then classified from the function's
//! one-sided limits, read from samples ever closer to it. All of this is deterministic and
//! counted against the evaluation budget; a scan that runs out says so ([`Scan::truncated`]).

use crate::expr::{Expr, GuardKind};

/// How the function behaves on one side of a point.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Approach {
    /// It tends to this value.
    Value(f64),
    /// It grows without bound, upwards.
    PlusInfinity,
    /// It grows without bound, downwards.
    MinusInfinity,
    /// It is undefined there, or oscillates without a limit.
    Undefined,
}

impl Approach {
    const fn is_infinite(self) -> bool {
        matches!(self, Self::PlusInfinity | Self::MinusInfinity)
    }
}

/// What kind of singular point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SingularityKind {
    /// A vertical asymptote: infinite on at least one side.
    Pole,
    /// A removable singularity: undefined at the point, with the same finite limit on both
    /// sides.
    Hole,
    /// A jump discontinuity: different finite limits on the two sides.
    Jump,
}

/// A pole, hole or jump.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Singularity {
    /// Where it is.
    pub x: f64,
    /// What it is.
    pub kind: SingularityKind,
    /// The limit from the left.
    pub left: Approach,
    /// The limit from the right.
    pub right: Approach,
}

/// An interval of `x`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    /// The start.
    pub start: f64,
    /// The end.
    pub end: f64,
}

/// A span where the function is defined but outside the y-window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offscreen {
    /// The span.
    pub span: Span,
    /// Above the window (otherwise below).
    pub above: bool,
}

/// What a scan found, in increasing `x`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct Scan {
    /// Poles, holes and jumps.
    pub singularities: Vec<Singularity>,
    /// Spans where the function is undefined, apart from single points.
    pub undefined: Vec<Span>,
    /// Spans off the y-window, when one was given.
    pub offscreen: Vec<Offscreen>,
    /// Whether the scan stopped at a budget, so features may be missing.
    pub truncated: bool,
    /// The evaluations the scan used.
    pub evaluations: u64,
}

/// The scan's limits. Engineering safety limits, not research defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanBudget {
    /// The most evaluations of the function and its guards.
    pub max_evaluations: u64,
    /// The most singular points and spans reported.
    pub max_features: usize,
}

impl ScanBudget {
    /// 1 000 000 evaluations and 1000 features.
    pub const DEFAULT: Self = Self {
        max_evaluations: 1_000_000,
        max_features: 1000,
    };
}

/// Counts evaluations against the budget; past it every value is `NaN`.
struct Counter {
    used: u64,
    max: u64,
    exhausted: bool,
}

impl Counter {
    fn eval(&mut self, expr: &Expr, x: f64) -> f64 {
        if self.used >= self.max {
            self.exhausted = true;
            return f64::NAN;
        }
        self.used += 1;
        expr.eval(x)
    }
}

/// A zero of a guard, before classification.
#[derive(Clone, Copy, Debug)]
struct Candidate {
    x: f64,
    kind: GuardKind,
}

/// Scans `f` over the increasing grid `xs` (at least two points), with an optional
/// y-window `[low, high]` for off-screen spans.
#[must_use]
pub fn scan(f: &Expr, xs: &[f64], window: Option<[f64; 2]>, budget: &ScanBudget) -> Scan {
    let mut counter = Counter {
        used: 0,
        max: budget.max_evaluations,
        exhausted: false,
    };
    let (Some(&min), Some(&max)) = (xs.first(), xs.last()) else {
        return Scan::default();
    };
    if xs.len() < 2 || max <= min {
        return Scan::default();
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "grids are far below 2^52 points"
    )]
    let step = (max - min) / (xs.len() - 1) as f64;
    let mut candidates = Vec::new();
    for guard in f.guards() {
        zeros(&guard.expr, guard.kind, xs, &mut counter, &mut candidates);
        if counter.exhausted {
            break;
        }
    }
    candidates.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.kind.cmp(&b.kind)));
    // One candidate per place, the strongest kind first (Denominator < Pole < Step < Domain).
    let tolerance = 1e-9 * (max - min);
    let mut unique: Vec<Candidate> = Vec::new();
    for candidate in candidates {
        match unique.last_mut() {
            Some(last) if (candidate.x - last.x).abs() <= tolerance => {
                if candidate.kind < last.kind {
                    last.kind = candidate.kind;
                }
            }
            _ => unique.push(candidate),
        }
    }
    let mut singularities = Vec::new();
    for candidate in &unique {
        if candidate.x <= min || candidate.x >= max {
            continue;
        }
        if let Some(found) = classify(f, *candidate, step, &mut counter) {
            singularities.push(found);
        }
    }
    let mut points: Vec<f64> = xs.to_vec();
    points.extend(unique.iter().map(|c| c.x).filter(|&x| x > min && x < max));
    points.sort_by(f64::total_cmp);
    points.dedup();
    let undefined = undefined_spans(f, &points, &singularities, &mut counter);
    let offscreen = window.map_or_else(Vec::new, |window| {
        offscreen_spans(f, &points, window, &mut counter)
    });
    let mut scan = Scan {
        singularities,
        undefined,
        offscreen,
        truncated: counter.exhausted,
        evaluations: counter.used,
    };
    if scan.singularities.len() + scan.undefined.len() + scan.offscreen.len() > budget.max_features
    {
        scan.singularities.truncate(budget.max_features);
        scan.undefined.truncate(budget.max_features);
        scan.offscreen.truncate(budget.max_features);
        scan.truncated = true;
    }
    scan
}

/// The zeros of `g` on the grid: grid points where it is zero, bisected sign changes, and
/// refined local minima of its magnitude.
fn zeros(g: &Expr, kind: GuardKind, xs: &[f64], counter: &mut Counter, out: &mut Vec<Candidate>) {
    let values: Vec<f64> = xs.iter().map(|&x| counter.eval(g, x)).collect();
    // The typical size of g (the median: near a pole of g it is huge).
    let mut magnitudes: Vec<f64> = values
        .iter()
        .filter(|v| v.is_finite())
        .map(|v| v.abs())
        .collect();
    magnitudes.sort_by(f64::total_cmp);
    let scale = magnitudes.get(magnitudes.len() / 2).copied().unwrap_or(0.0);
    let tiny = 1e-12 * scale.max(f64::MIN_POSITIVE);
    let mut push = |x: f64| out.push(Candidate { x, kind });
    for (i, (&x, &v)) in xs.iter().zip(&values).enumerate() {
        if v == 0.0 {
            push(x);
        }
        let (Some(&x1), Some(&v1)) = (xs.get(i + 1), values.get(i + 1)) else {
            continue;
        };
        if v.is_finite() && v1.is_finite() && v != 0.0 && v1 != 0.0 && (v < 0.0) != (v1 < 0.0) {
            push(bisect(g, x, x1, v < 0.0, counter));
        }
    }
    // Dips of |g| that do not change sign on the grid: refine each and look for zeros.
    for (lo, hi, negative) in dips(xs, &values) {
        // Towards zero: the lowest point of a positive dip, the highest of a negative one.
        let toward = if negative { -1.0 } else { 1.0 };
        let xm = golden_minimum(g, lo, hi, toward, counter);
        let vm = counter.eval(g, xm);
        if vm.is_finite() && vm != 0.0 && (vm < 0.0) != negative {
            push(bisect(g, lo, xm, negative, counter));
            push(bisect(g, xm, hi, vm < 0.0, counter));
        } else if vm.abs() <= tiny {
            push(xm);
        }
    }
}

/// The intervals where `|g|` dips without a sign change on the grid, with the sign of `g`
/// there: around each interior local minimum of `|g|`, the first or last interval when
/// `|g|` falls towards that end of the domain, where a dip shows only at the end point, and
/// both intervals next to a grid point where `g` is exactly 0.
pub(crate) fn dips(xs: &[f64], values: &[f64]) -> Vec<(f64, f64, bool)> {
    let usable = |v: f64| v.is_finite() && v != 0.0;
    let same = |a: f64, b: f64| (a < 0.0) == (b < 0.0);
    let mut out = Vec::new();
    let n = values.len().min(xs.len());
    for i in 0..n {
        let (Some(&x), Some(&v)) = (xs.get(i), values.get(i)) else {
            continue;
        };
        // g is exactly 0 at a grid point: a second zero may hide in either neighbouring
        // step without a sign change on the grid (x(x - b) on the grid -4, 0, 4).
        if v == 0.0 {
            for j in [i.checked_sub(1), Some(i + 1)].into_iter().flatten() {
                if let (Some(&xj), Some(&vj)) = (xs.get(j), values.get(j))
                    && usable(vj)
                {
                    out.push((x.min(xj), x.max(xj), vj < 0.0));
                }
            }
            continue;
        }
        if !usable(v) {
            continue;
        }
        let before = i
            .checked_sub(1)
            .and_then(|j| Some((*xs.get(j)?, *values.get(j)?)));
        let after = xs.get(i + 1).copied().zip(values.get(i + 1).copied());
        match (before, after) {
            (Some((x0, v0)), Some((x1, v1)))
                if usable(v0)
                    && usable(v1)
                    && same(v0, v)
                    && same(v, v1)
                    && v.abs() < v0.abs()
                    && v.abs() <= v1.abs() =>
            {
                out.push((x0, x1, v < 0.0));
            }
            (None, Some((x1, v1))) if usable(v1) && same(v, v1) && v.abs() < v1.abs() => {
                out.push((x, x1, v < 0.0));
            }
            (Some((x0, v0)), None) if usable(v0) && same(v0, v) && v.abs() < v0.abs() => {
                out.push((x0, x, v < 0.0));
            }
            _ => {}
        }
    }
    out
}

/// The point between `a` and `b` where `g` changes sign, to the last representable bit.
/// `a_negative` is the sign of `g(a)`.
#[expect(
    clippy::many_single_char_names,
    reason = "the usual names of the numerical method"
)]
fn bisect(g: &Expr, mut a: f64, mut b: f64, a_negative: bool, counter: &mut Counter) -> f64 {
    for _ in 0..200 {
        let m = f64::midpoint(a, b);
        if m <= a || m >= b {
            break;
        }
        let v = counter.eval(g, m);
        if v == 0.0 {
            return m;
        }
        if v.is_nan() {
            break;
        }
        if (v < 0.0) == a_negative {
            a = m;
        } else {
            b = m;
        }
    }
    if counter.eval(g, b).abs() < counter.eval(g, a).abs() {
        b
    } else {
        a
    }
}

/// The point between `a` and `b` where `sign * g` is smallest, by golden-section search.
#[expect(
    clippy::many_single_char_names,
    reason = "the usual names of the numerical method"
)]
fn golden_minimum(g: &Expr, mut a: f64, mut b: f64, sign: f64, counter: &mut Counter) -> f64 {
    const R: f64 = 0.618_033_988_749_894_9;
    let mut c = b - R * (b - a);
    let mut d = a + R * (b - a);
    let mut fc = sign * counter.eval(g, c);
    let mut fd = sign * counter.eval(g, d);
    for _ in 0..120 {
        if (b - a).abs() <= 4.0 * f64::EPSILON * a.abs().max(b.abs()).max(f64::MIN_POSITIVE) {
            break;
        }
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - R * (b - a);
            fc = sign * counter.eval(g, c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + R * (b - a);
            fd = sign * counter.eval(g, d);
        }
    }
    if fc < fd { c } else { d }
}

/// `f` at `x + sign·h` for `h` = 10^-2 to 10^-8 grid steps: the probes of one side.
fn probe(f: &Expr, x: f64, step: f64, sign: f64, counter: &mut Counter) -> [f64; 7] {
    let mut values = [0.0; 7];
    for (k, value) in values.iter_mut().enumerate() {
        let h = step * 10_f64.powi(-2 - i32::try_from(k).unwrap_or(0));
        *value = counter.eval(f, x + sign * h);
    }
    values
}

/// The one-sided limit the probes of [`probe`] show.
fn approach_of(values: &[f64; 7]) -> Approach {
    let Some(&last) = values.last() else {
        return Approach::Undefined;
    };
    if last.is_nan() {
        return Approach::Undefined;
    }
    if last.is_infinite() {
        return if last > 0.0 {
            Approach::PlusInfinity
        } else {
            Approach::MinusInfinity
        };
    }
    if values.iter().any(|v| !v.is_finite()) {
        return Approach::Undefined;
    }
    let first_step = (values[1] - values[0]).abs();
    let last_step = (values[6] - values[5]).abs();
    let scale = 1e-12 * (1.0 + last.abs());
    // A limit: the steps shrink, or the values barely move at all. The second case covers
    // rounding near a hole, such as sin(x)/(x - pi), whose float pi is not exactly pi.
    let total = (last - values[0]).abs();
    if last_step <= scale || last_step <= 0.1 * first_step || total <= 1e-4 * (1.0 + last.abs()) {
        // The most settled sample: closer ones lose digits to rounding.
        let settled = values
            .windows(2)
            .min_by(|a, b| (a[1] - a[0]).abs().total_cmp(&(b[1] - b[0]).abs()))
            .map_or(last, |w| w[1]);
        return Approach::Value(settled);
    }
    let growing = values.windows(2).all(|w| w[1].abs() >= w[0].abs());
    if growing && last_step >= 0.5 * first_step {
        if last > 0.0 {
            Approach::PlusInfinity
        } else {
            Approach::MinusInfinity
        }
    } else {
        Approach::Undefined
    }
}

/// Classifies the candidate at `x` from its one-sided limits; `None` when it is not a
/// singular point (a smooth point, or the edge of an undefined region).
fn classify(
    f: &Expr,
    candidate: Candidate,
    step: f64,
    counter: &mut Counter,
) -> Option<Singularity> {
    let x = candidate.x;
    let below = probe(f, x, step, -1.0, counter);
    let above = probe(f, x, step, 1.0, counter);
    let (left, right) = (approach_of(&below), approach_of(&above));
    let kind = match (left, right) {
        _ if left.is_infinite() || right.is_infinite() => SingularityKind::Pole,
        (Approach::Value(l), Approach::Value(r)) => {
            let same = (l - r).abs() <= 1e-6 * (1.0 + l.abs().max(r.abs()))
                || closing(&below, &above);
            if !same {
                SingularityKind::Jump
            } else if candidate.kind == GuardKind::Denominator || !counter.eval(f, x).is_finite() {
                SingularityKind::Hole
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(Singularity {
        x,
        kind,
        left,
        right,
    })
}

/// Whether the gap between the two sides closes as the probes come closer: at a hole on a
/// steep slope (beside a pole, F-201) the sides differ by about the slope times the probe
/// distance, so the gap shrinks tenfold per decade, while at a jump it stays the jump's
/// size. Judged on the three closest probes, which lie nearest the point and so on its own
/// slope: over two decades the gap must fall at least fivefold, or reach rounding.
fn closing(below: &[f64; 7], above: &[f64; 7]) -> bool {
    let gap = |k: usize| match (below.get(k), above.get(k)) {
        (Some(l), Some(r)) => (l - r).abs(),
        _ => f64::NAN,
    };
    let (far, near) = (gap(4), gap(6));
    if !(far.is_finite() && near.is_finite()) {
        return false;
    }
    let scale = below.get(6).map_or(0.0, |v| v.abs()).max(above.get(6).map_or(0.0, |v| v.abs()));
    near <= 0.2 * far || near <= 1e-12 * (1.0 + scale)
}

/// The last point between `defined` and `undefined` where `f` is not `NaN`.
fn edge(f: &Expr, mut defined: f64, mut undefined: f64, counter: &mut Counter) -> f64 {
    for _ in 0..200 {
        let m = f64::midpoint(defined, undefined);
        if m.to_bits() == defined.to_bits() || m.to_bits() == undefined.to_bits() {
            break;
        }
        if counter.eval(f, m).is_nan() {
            undefined = m;
        } else {
            defined = m;
        }
    }
    defined
}

/// Spans where `f` is `NaN`, found on `points` and the midpoints between them, with their
/// edges bisected. A single undefined point at a singularity is not a span.
fn undefined_spans(
    f: &Expr,
    points: &[f64],
    singularities: &[Singularity],
    counter: &mut Counter,
) -> Vec<Span> {
    let mut dense = Vec::with_capacity(points.len() * 2);
    for (i, &p) in points.iter().enumerate() {
        dense.push(p);
        if let Some(&q) = points.get(i + 1) {
            dense.push(f64::midpoint(p, q));
        }
    }
    let nan: Vec<bool> = dense.iter().map(|&x| counter.eval(f, x).is_nan()).collect();
    let (Some(&min), Some(&max)) = (dense.first(), dense.last()) else {
        return Vec::new();
    };
    let tolerance = 1e-9 * (max - min);
    let mut spans = Vec::new();
    let mut i = 0;
    while i < dense.len() {
        if !nan.get(i).copied().unwrap_or(false) {
            i += 1;
            continue;
        }
        let mut j = i;
        while nan.get(j + 1).copied().unwrap_or(false) {
            j += 1;
        }
        let (Some(&first), Some(&last)) = (dense.get(i), dense.get(j)) else {
            break;
        };
        // A run that is only the point of a pole or hole (perhaps several samples within a
        // float step of it) is that point, not a span.
        let at_singularity = singularities
            .iter()
            .any(|s| (s.x - first).abs() <= tolerance && (s.x - last).abs() <= tolerance);
        if !at_singularity {
            let start = match i.checked_sub(1).and_then(|k| dense.get(k)) {
                Some(&before) => edge(f, before, first, counter),
                None => min,
            };
            let end = match dense.get(j + 1) {
                Some(&after) => edge(f, after, last, counter),
                None => max,
            };
            spans.push(Span { start, end });
        }
        i = j + 1;
    }
    spans
}

/// Spans of `points` where `f` is defined but above or below `window`, with their edges
/// bisected against the window's bounds.
#[expect(
    clippy::many_single_char_names,
    reason = "the usual names of the numerical method"
)]
fn offscreen_spans(
    f: &Expr,
    points: &[f64],
    window: [f64; 2],
    counter: &mut Counter,
) -> Vec<Offscreen> {
    let [low, high] = window;
    let side = |v: f64| -> Option<bool> {
        if v.is_nan() {
            None
        } else if v > high {
            Some(true)
        } else if v < low {
            Some(false)
        } else {
            None
        }
    };
    let values: Vec<f64> = points.iter().map(|&x| counter.eval(f, x)).collect();
    let mut spans: Vec<Offscreen> = Vec::new();
    let mut i = 0;
    while i < points.len() {
        let Some(above) = values.get(i).copied().and_then(side) else {
            i += 1;
            continue;
        };
        let mut j = i;
        while values.get(j + 1).copied().and_then(side) == Some(above) {
            j += 1;
        }
        let cross = |inside: f64, outside: f64, counter: &mut Counter| {
            let mut a = inside;
            let mut b = outside;
            for _ in 0..200 {
                let m = f64::midpoint(a, b);
                if m.to_bits() == a.to_bits() || m.to_bits() == b.to_bits() {
                    break;
                }
                let v = counter.eval(f, m);
                if side(v) == Some(above) {
                    b = m;
                } else if v.is_nan() {
                    break;
                } else {
                    a = m;
                }
            }
            b
        };
        let (Some(&first), Some(&last)) = (points.get(i), points.get(j)) else {
            break;
        };
        let start = match i.checked_sub(1) {
            Some(k) if values.get(k).is_some_and(|v| v.is_finite()) => {
                cross(points.get(k).copied().unwrap_or(first), first, counter)
            }
            _ => first,
        };
        let end = match values.get(j + 1) {
            Some(v) if v.is_finite() => {
                cross(points.get(j + 1).copied().unwrap_or(last), last, counter)
            }
            _ => last,
        };
        spans.push(Offscreen {
            span: Span { start, end },
            above,
        });
        i = j + 1;
    }
    spans
}

#[cfg(test)]
mod tests;
