// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The 40-curve suite (C03 exit): classic curves of school and first-year maths with their
//! singular points and undefined spans known analytically. Tests check the scan against it
//! (C03-T02); the critical points join in C03-T03, and the curve gallery renders it
//! (C03-T06).

use std::f64::consts::PI;

/// One curve and what is known about it on its domain.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Curve {
    /// A short name.
    pub name: &'static str,
    /// The expression (ADR-0028).
    pub expr: &'static str,
    /// The domain.
    pub domain: [f64; 2],
    /// Vertical asymptotes.
    pub poles: &'static [f64],
    /// Removable singularities.
    pub holes: &'static [f64],
    /// Jump discontinuities.
    pub jumps: &'static [f64],
    /// Spans where the function is undefined, as `[start, end]`.
    pub undefined: &'static [[f64; 2]],
}

const fn curve(name: &'static str, expr: &'static str, domain: [f64; 2]) -> Curve {
    Curve {
        name,
        expr,
        domain,
        poles: &[],
        holes: &[],
        jumps: &[],
        undefined: &[],
    }
}

impl Curve {
    const fn poles(mut self, poles: &'static [f64]) -> Self {
        self.poles = poles;
        self
    }

    const fn holes(mut self, holes: &'static [f64]) -> Self {
        self.holes = holes;
        self
    }

    const fn jumps(mut self, jumps: &'static [f64]) -> Self {
        self.jumps = jumps;
        self
    }

    const fn undefined(mut self, undefined: &'static [[f64; 2]]) -> Self {
        self.undefined = undefined;
        self
    }
}

/// The suite.
pub const CURVES: [Curve; 40] = [
    curve("parabola", "x^2", [-3.0, 3.0]),
    curve("cubic", "x^3 - 3*x", [-3.0, 3.0]),
    curve("sine", "sin(x)", [-10.0, 10.0]),
    curve("cosine", "cos(x)", [-10.0, 10.0]),
    curve("tangent", "tan(x)", [-5.0, 5.0]).poles(&[-1.5 * PI, -0.5 * PI, 0.5 * PI, 1.5 * PI]),
    curve("reciprocal", "1/x", [-5.0, 5.0]).poles(&[0.0]),
    curve("shifted reciprocal", "1/(x - 1)", [-4.0, 4.0]).poles(&[1.0]),
    curve("inverse square", "1/x^2", [-3.0, 3.0]).poles(&[0.0]),
    curve("line with a hole", "(x^2 - 1)/(x - 1)", [-3.0, 3.0]).holes(&[1.0]),
    curve("sinc", "sin(x)/x", [-20.0, 20.0]).holes(&[0.0]),
    curve("natural logarithm", "ln(x)", [-2.0, 5.0])
        .poles(&[0.0])
        .undefined(&[[-2.0, 0.0]]),
    curve("square root", "sqrt(x)", [-4.0, 4.0]).undefined(&[[-4.0, 0.0]]),
    curve("semicircle", "sqrt(1 - x^2)", [-2.0, 2.0]).undefined(&[[-2.0, -1.0], [1.0, 2.0]]),
    curve("narrow gap", "sqrt(x^2 - 0.000001)", [-1.0, 1.0]).undefined(&[[-0.001, 0.001]]),
    curve("two poles", "1/(x^2 - 4)", [-5.0, 5.0]).poles(&[-2.0, 2.0]),
    curve("bounded rational", "x/(x^2 + 1)", [-5.0, 5.0]),
    curve("exponential", "exp(x)", [-3.0, 3.0]),
    curve("bell", "exp(-x^2)", [-3.0, 3.0]),
    curve("floor", "floor(x)", [-2.5, 2.5]).jumps(&[-2.0, -1.0, 0.0, 1.0, 2.0]),
    curve("sign by division", "x/abs(x)", [-2.0, 2.0]).jumps(&[0.0]),
    curve("absolute value", "abs(x)", [-2.0, 2.0]),
    curve("secant", "sec(x)", [-4.0, 4.0]).poles(&[-0.5 * PI, 0.5 * PI]),
    curve("cotangent", "cot(x)", [-3.0, 3.0]).poles(&[0.0]),
    curve("cosecant", "csc(x)", [0.5, 6.0]).poles(&[PI]),
    curve("inverse hyperbolic tangent", "atanh(x)", [-2.0, 2.0])
        .poles(&[-1.0, 1.0])
        .undefined(&[[-2.0, -1.0], [1.0, 2.0]]),
    curve("arcsine", "asin(x)", [-2.0, 2.0]).undefined(&[[-2.0, -1.0], [1.0, 2.0]]),
    curve("log of the absolute value", "ln(abs(x))", [-3.0, 3.0]).poles(&[0.0]),
    curve("cubic over a line", "(x^3 - 8)/(x - 2)", [0.0, 4.0]).holes(&[2.0]),
    curve("logistic", "1/(1 + exp(-x))", [-6.0, 6.0]),
    curve("fractional power", "x^(1/3)", [-8.0, 8.0]).undefined(&[[-8.0, 0.0]]),
    curve("cube root", "cbrt(x)", [-8.0, 8.0]),
    curve("hole and pole", "(x + 1)/(x^2 - 1)", [-3.0, 3.0])
        .poles(&[1.0])
        .holes(&[-1.0]),
    curve("round", "round(x)", [-2.0, 2.0]).jumps(&[-1.5, -0.5, 0.5, 1.5]),
    curve("two linear poles", "1/((x - 1)*(x + 2))", [-4.0, 4.0]).poles(&[-2.0, 1.0]),
    curve("one-sided pole", "exp(1/x)", [-2.0, 2.0]).poles(&[0.0]),
    curve("half-angle tangent", "tan(x/2)", [-10.0, 10.0]).poles(&[-3.0 * PI, -PI, PI, 3.0 * PI]),
    curve("oblique asymptote", "x^2/(x - 3)", [0.0, 5.0]).poles(&[3.0]),
    curve("log of a parabola", "ln(x^2 - 1)", [-3.0, 3.0])
        .poles(&[-1.0, 1.0])
        .undefined(&[[-1.0, 1.0]]),
    curve("shifted sinc", "sin(x)/(x - pi)", [0.0, 6.0]).holes(&[PI]),
    curve("arccosine", "acos(x/2)", [-3.0, 3.0]).undefined(&[[-3.0, -2.0], [2.0, 3.0]]),
];
