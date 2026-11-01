// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Critical points of a function (C03-T03, A11Y-28): zeros, local extrema, inflection
//! points and horizontal asymptotes, beside the poles, holes, jumps and undefined spans of
//! [`scan`](crate::scan).
//!
//! The derivatives are symbolic ([`Expr::derivative`]), else central differences (07
//! CURVASONA). Roots of `f`, `f'` and `f''` are found between consecutive points that
//! are not separated by a singular point or an undefined span: sign changes are bisected
//! to the last bit, and dips that do not change sign are refined by golden-section search,
//! which finds double zeros (`x^2`). An extremum is a sign change of `f'` (a kink such as
//! `abs(x)` included); a zero of `f'` without one (`x^3` at 0) is not. An inflection is a
//! sign change of `f''`. Holes are left out of the extrema: `sin(x)/x` has no maximum at 0,
//! where it is undefined. Horizontal asymptotes are limits of `f` far beyond the domain.

use crate::expr::{Expr, NoDerivative, numeric_derivative};
use crate::scan::{Scan, SingularityKind};

/// What a feature is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum FeatureKind {
    /// `f(x) = 0`.
    Zero,
    /// A local maximum.
    Maximum,
    /// A local minimum.
    Minimum,
    /// An inflection point: `f''` changes sign.
    Inflection,
    /// A vertical asymptote (a pole).
    VerticalAsymptote,
    /// A horizontal asymptote as `x` decreases (`right` false) or increases.
    HorizontalAsymptote {
        /// Towards plus infinity.
        right: bool,
    },
    /// A removable singularity.
    Hole,
    /// A jump discontinuity.
    Jump,
    /// A span where the function is undefined.
    Undefined,
    /// A span above (`above`) or below the y-window.
    Offscreen {
        /// Above the window.
        above: bool,
    },
}

impl FeatureKind {
    /// The text equivalent of the feature (A11Y-24), as the insight's label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::Maximum => "local maximum",
            Self::Minimum => "local minimum",
            Self::Inflection => "inflection point",
            Self::VerticalAsymptote => "vertical asymptote",
            Self::HorizontalAsymptote { right: false } => "horizontal asymptote to the left",
            Self::HorizontalAsymptote { right: true } => "horizontal asymptote to the right",
            Self::Hole => "hole",
            Self::Jump => "jump",
            Self::Undefined => "undefined",
            Self::Offscreen { above: true } => "above the window",
            Self::Offscreen { above: false } => "below the window",
        }
    }
}

/// A feature of the function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Feature {
    /// What it is.
    pub kind: FeatureKind,
    /// Where it is, or where its span starts. A horizontal asymptote is at the domain's end
    /// on its side.
    pub x: f64,
    /// Where its span ends; equal to `x` for a point.
    pub end: f64,
    /// Its value: `f(x)` at a zero, extremum or inflection, the limit at a hole or a
    /// horizontal asymptote; `None` for poles, jumps and spans.
    pub y: Option<f64>,
}

impl Feature {
    const fn point(kind: FeatureKind, x: f64, y: Option<f64>) -> Self {
        Self { kind, x, end: x, y }
    }

    /// Whether it is a single point rather than a span.
    #[must_use]
    pub fn is_point(&self) -> bool {
        self.x.total_cmp(&self.end).is_eq()
    }
}

/// `f` with its first and second derivatives, symbolic where they exist.
#[derive(Clone, Debug)]
pub struct Calculus {
    f: Expr,
    d1: Option<Expr>,
    d2: Option<Expr>,
}

impl Calculus {
    /// The derivatives of `f`.
    #[must_use]
    pub fn new(f: Expr) -> Self {
        let d1 = f.derivative().ok();
        let d2 = d1.as_ref().and_then(|d| d.derivative().ok());
        Self { f, d1, d2 }
    }

    /// The function.
    #[must_use]
    pub const fn f(&self) -> &Expr {
        &self.f
    }

    /// Why `f'` is numerical, if it is.
    #[must_use]
    pub fn first_numeric(&self) -> Option<NoDerivative> {
        match &self.d1 {
            Some(_) => None,
            None => self.f.derivative().err(),
        }
    }

    /// `f'(x)`.
    #[must_use]
    pub fn d1(&self, x: f64) -> f64 {
        self.d1
            .as_ref()
            .map_or_else(|| numeric_derivative(&self.f, x), |d| d.eval(x))
    }

    /// `f''(x)`.
    #[must_use]
    #[expect(
        clippy::many_single_char_names,
        reason = "the usual names of a second difference"
    )]
    pub fn d2(&self, x: f64) -> f64 {
        if let Some(d2) = &self.d2 {
            return d2.eval(x);
        }
        if let Some(d1) = &self.d1 {
            return numeric_derivative(d1, x);
        }
        // A second difference with the step that balances truncation and rounding error.
        // EPSILON^(1/4) = 2^-13, written out: no platform `powf` (ADR-0013 §1).
        let h = (1.0 / 8192.0) * x.abs().max(1.0);
        let (l, m, r) = (self.f.eval(x - h), self.f.eval(x), self.f.eval(x + h));
        (r - 2.0 * m + l) / (h * h)
    }
}

/// Finds the features of `calculus.f()` on the increasing points `xs`, with `scan`'s
/// singular points and spans (from [`scan::scan`](crate::scan::scan) over the same
/// points), at most `max_features` of them.
#[must_use]
pub fn features(calculus: &Calculus, xs: &[f64], scan: &Scan, max_features: usize) -> Vec<Feature> {
    let (Some(&min), Some(&max)) = (xs.first(), xs.last()) else {
        return Vec::new();
    };
    let tolerance = 1e-9 * (max - min).max(f64::MIN_POSITIVE);
    let f = |x: f64| calculus.f().eval(x);
    let bridgeable = |p: f64, q: f64| {
        f(p).is_finite()
            && f(q).is_finite()
            && !scan
                .singularities
                .iter()
                .any(|s| s.x > p - tolerance && s.x < q + tolerance)
            && !scan
                .undefined
                .iter()
                .any(|s| s.start < q + tolerance && s.end > p - tolerance)
    };
    let pairs: Vec<bool> = xs.windows(2).map(|w| bridgeable(w[0], w[1])).collect();
    let mut out = Vec::new();
    // The typical size of f: the median, because near a pole f is huge. Noise floors for f,
    // f' and f'' scale from it and the domain's width: below them a derivative's value is
    // rounding noise (`(x^2 - 1)/(x - 1)` has f'' = 0 up to rounding).
    let scale = median(xs.iter().map(|&x| f(x)));
    let width = (max - min).max(f64::MIN_POSITIVE);
    let floor1 = 1e-8 * scale / width;
    let floor2 = 1e-8 * scale / (width * width);
    for (x, _, _) in roots(&f, xs, &pairs, 1e-12 * scale, true) {
        out.push(Feature::point(FeatureKind::Zero, x, Some(0.0)));
    }
    for span in &scan.undefined {
        for edge in [span.start, span.end] {
            if edge > min && edge < max && f(edge).abs() <= 1e-12 * scale.max(1.0) {
                out.push(Feature::point(FeatureKind::Zero, edge, Some(0.0)));
            }
        }
    }
    // Extrema: sign changes of f'.
    let d1 = |x: f64| {
        let v = calculus.d1(x);
        if v.abs() <= floor1 { 0.0 } else { v }
    };
    for (x, before, after) in roots(&d1, xs, &pairs, 0.0, false) {
        let kind = if before > 0.0 && after < 0.0 {
            FeatureKind::Maximum
        } else {
            FeatureKind::Minimum
        };
        out.push(Feature::point(kind, x, Some(f(x))));
    }
    // Inflections: sign changes of f'', ignoring values indistinguishable from 0.
    let d2 = |x: f64| calculus.d2(x);
    let floor = floor2.max(1e-9 * median(xs.iter().map(|&x| d2(x))));
    let quiet = |x: f64| {
        let v = d2(x);
        if v.abs() <= floor { 0.0 } else { v }
    };
    for (x, _, _) in roots(&quiet, xs, &pairs, 0.0, false) {
        out.push(Feature::point(FeatureKind::Inflection, x, Some(f(x))));
    }
    for singularity in &scan.singularities {
        let (kind, y) = match singularity.kind {
            SingularityKind::Pole => (FeatureKind::VerticalAsymptote, None),
            SingularityKind::Hole => (
                FeatureKind::Hole,
                match singularity.left {
                    crate::scan::Approach::Value(v) => Some(v),
                    _ => None,
                },
            ),
            _ => (FeatureKind::Jump, None),
        };
        out.push(Feature::point(kind, singularity.x, y));
    }
    for span in &scan.undefined {
        out.push(Feature {
            kind: FeatureKind::Undefined,
            x: span.start,
            end: span.end,
            y: None,
        });
    }
    for span in &scan.offscreen {
        out.push(Feature {
            kind: FeatureKind::Offscreen { above: span.above },
            x: span.span.start,
            end: span.span.end,
            y: None,
        });
    }
    for right in [false, true] {
        if let Some(limit) = limit_at_infinity(calculus.f(), if right { max } else { min }, right) {
            let at = if right { max } else { min };
            out.push(Feature::point(
                FeatureKind::HorizontalAsymptote { right },
                at,
                Some(limit),
            ));
        }
    }
    // Duplicates of one feature (a zero found from both sides of a grid point) are merged
    // within each kind, then the features are put in x order.
    out.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.x.total_cmp(&b.x)));
    out.dedup_by(|b, a| a.kind == b.kind && (a.x - b.x).abs() <= tolerance);
    out.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.kind.cmp(&b.kind)));
    out.truncate(max_features);
    out
}

/// The median magnitude of the finite values.
fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut magnitudes: Vec<f64> = values.filter(|v| v.is_finite()).map(f64::abs).collect();
    magnitudes.sort_by(f64::total_cmp);
    magnitudes.get(magnitudes.len() / 2).copied().unwrap_or(0.0)
}

/// The roots of `g` between consecutive points of `xs` where `pairs` allows, as
/// `(x, sign before, sign after)`. With `dips`, local minima of `|g|` that do not change
/// sign count as (double) roots when `|g|` falls to `tiny`; otherwise only sign changes
/// count, and a point where `g` is exactly 0 counts when its neighbours have opposite signs.
#[expect(
    clippy::many_single_char_names,
    reason = "the usual names of the numerical method"
)]
fn roots(
    g: &dyn Fn(f64) -> f64,
    xs: &[f64],
    pairs: &[bool],
    tiny: f64,
    dips: bool,
) -> Vec<(f64, f64, f64)> {
    let values: Vec<f64> = xs.iter().map(|&x| g(x)).collect();
    let mut out = Vec::new();
    let ok = |i: usize| pairs.get(i).copied().unwrap_or(false);
    for i in 0..xs.len() {
        let (Some(&x), Some(&v)) = (xs.get(i), values.get(i)) else {
            continue;
        };
        let before = i
            .checked_sub(1)
            .filter(|&j| ok(j))
            .and_then(|j| values.get(j))
            .copied();
        let after = values.get(i + 1).copied().filter(|_| ok(i));
        // An exact zero at a point.
        if v == 0.0 {
            match (before, after) {
                (Some(b), Some(a)) if b != 0.0 && a != 0.0 && (b < 0.0) != (a < 0.0) => {
                    out.push((x, b, a));
                }
                (Some(b), Some(a)) if dips && b != 0.0 && a != 0.0 => out.push((x, b, a)),
                (None, Some(a)) if dips && a != 0.0 => out.push((x, 0.0, a)),
                (Some(b), None) if dips && b != 0.0 => out.push((x, b, 0.0)),
                _ => {}
            }
        }
        // A sign change to the next point.
        if let (Some(a), Some(&next)) = (after, xs.get(i + 1))
            && v != 0.0
            && a != 0.0
            && (v < 0.0) != (a < 0.0)
        {
            out.push((bisect(g, x, next, v < 0.0), v, a));
        }
    }
    if dips {
        // Dips of |g| that do not change sign, inside allowed pairs only.
        let allowed = |lo: f64, hi: f64| {
            xs.windows(2)
                .zip(pairs)
                .filter(|(w, _)| w[0] >= lo && w[1] <= hi)
                .all(|(_, &ok)| ok)
        };
        for (lo, hi, negative) in crate::scan::dips(xs, &values) {
            if !allowed(lo, hi) {
                continue;
            }
            let toward = if negative { -1.0 } else { 1.0 };
            let xm = golden(g, lo, hi, toward);
            let vm = g(xm);
            let (b, a) = (g(lo), g(hi));
            if vm.is_finite() && vm != 0.0 && (vm < 0.0) != negative {
                out.push((bisect(g, lo, xm, negative), b, vm));
                out.push((bisect(g, xm, hi, vm < 0.0), vm, a));
            } else if vm.abs() <= tiny {
                out.push((xm, b, a));
            }
        }
    }
    out
}

/// Where `g` changes sign between `a` and `b`, to the last bit; `a_negative` is the sign of
/// `g(a)`.
#[expect(
    clippy::many_single_char_names,
    reason = "the usual names of the numerical method"
)]
fn bisect(g: &dyn Fn(f64) -> f64, mut a: f64, mut b: f64, a_negative: bool) -> f64 {
    for _ in 0..200 {
        let m = f64::midpoint(a, b);
        if m <= a || m >= b {
            break;
        }
        let v = g(m);
        // Exactly zero, or undefined: the kink of `abs` has no derivative at the root.
        if v == 0.0 || v.is_nan() {
            return m;
        }
        if (v < 0.0) == a_negative {
            a = m;
        } else {
            b = m;
        }
    }
    if g(b).abs() < g(a).abs() { b } else { a }
}

/// The point between `a` and `b` where `sign * g` is smallest (golden-section search).
#[expect(
    clippy::many_single_char_names,
    reason = "the usual names of the numerical method"
)]
fn golden(g: &dyn Fn(f64) -> f64, mut a: f64, mut b: f64, sign: f64) -> f64 {
    const R: f64 = 0.618_033_988_749_894_9;
    let mut c = b - R * (b - a);
    let mut d = a + R * (b - a);
    let mut fc = sign * g(c);
    let mut fd = sign * g(d);
    for _ in 0..120 {
        if (b - a).abs() <= 4.0 * f64::EPSILON * a.abs().max(b.abs()).max(f64::MIN_POSITIVE) {
            break;
        }
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - R * (b - a);
            fc = sign * g(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + R * (b - a);
            fd = sign * g(d);
        }
    }
    if fc < fd { c } else { d }
}

/// The limit of `f` as `x` goes to infinity beyond `edge` (`right`) or to minus infinity,
/// when it exists and `f` is not constant there; read at `edge · 4^k`, `k` up to 30.
fn limit_at_infinity(f: &Expr, edge: f64, right: bool) -> Option<f64> {
    let start = edge.abs().max(1.0);
    let sign = if right { 1.0 } else { -1.0 };
    let mut values = Vec::with_capacity(31);
    for k in 0..=30 {
        let v = f.eval(sign * start * 4_f64.powi(k));
        if !v.is_finite() {
            return None;
        }
        values.push(v);
    }
    let (first, last) = (values.first()?, values.last()?);
    let first_step = (values.get(1)? - first).abs();
    let last_step = (last - values.get(29)?).abs();
    let spread = values.iter().fold(0.0_f64, |m, v| m.max((v - last).abs()));
    if spread == 0.0 {
        // Constant out there: no asymptote to speak of.
        return None;
    }
    let settled = last_step <= 1e-6 * first_step || last_step <= 1e-15 * (1.0 + last.abs());
    if !settled {
        return None;
    }
    Some(if last.abs() <= 1e-12 * (1.0 + spread) {
        0.0
    } else {
        *last
    })
}

#[cfg(test)]
mod tests;
