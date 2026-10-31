// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The 40-curve suite (C03 exit): classic curves of school and first-year maths with their
//! singular points, undefined spans and critical points known analytically. Tests check the
//! scan (C03-T02) and the critical points (C03-T03) against it, and the curve gallery
//! renders it (C03-T06). Values without a closed form (the extrema of `sin(x)/x`, roots of
//! `tan x = x`) are given to 16 digits from the literature.

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
    /// Zeros (points only; a function that is 0 on a whole interval has none).
    pub zeros: &'static [f64],
    /// Local maxima, endpoints and holes excluded.
    pub maxima: &'static [f64],
    /// Local minima, endpoints and holes excluded.
    pub minima: &'static [f64],
    /// Inflection points; `None` where the suite does not check them (no closed form).
    pub inflections: Option<&'static [f64]>,
    /// The limit as `x` goes to minus infinity, when it exists and `f` is not constant.
    pub left: Option<f64>,
    /// The limit as `x` goes to plus infinity.
    pub right: Option<f64>,
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
        zeros: &[],
        maxima: &[],
        minima: &[],
        inflections: Some(&[]),
        left: None,
        right: None,
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

    const fn zeros(mut self, zeros: &'static [f64]) -> Self {
        self.zeros = zeros;
        self
    }

    const fn maxima(mut self, maxima: &'static [f64]) -> Self {
        self.maxima = maxima;
        self
    }

    const fn minima(mut self, minima: &'static [f64]) -> Self {
        self.minima = minima;
        self
    }

    const fn inflections(mut self, inflections: Option<&'static [f64]>) -> Self {
        self.inflections = inflections;
        self
    }

    const fn limits(mut self, left: Option<f64>, right: Option<f64>) -> Self {
        self.left = left;
        self.right = right;
        self
    }
}

const SQRT_2: f64 = std::f64::consts::SQRT_2;
const SQRT_3: f64 = 1.732_050_807_568_877_2;
const FRAC_1_SQRT_2: f64 = std::f64::consts::FRAC_1_SQRT_2;
/// The positive roots of `tan x = x` below 20.
const TAN_X_X: [f64; 5] = [
    4.493_409_457_909_064,
    7.725_251_836_937_707,
    10.904_121_659_428_899,
    14.066_193_912_831_473,
    17.220_755_271_930_768,
];
const BOTH_ZERO: (Option<f64>, Option<f64>) = (Some(0.0), Some(0.0));

/// The suite.
pub const CURVES: [Curve; 40] = [
    curve("parabola", "x^2", [-3.0, 3.0])
        .zeros(&[0.0])
        .minima(&[0.0]),
    curve("cubic", "x^3 - 3*x", [-3.0, 3.0])
        .zeros(&[-SQRT_3, 0.0, SQRT_3])
        .maxima(&[-1.0])
        .minima(&[1.0])
        .inflections(Some(&[0.0])),
    curve("sine", "sin(x)", [-10.0, 10.0])
        .zeros(&[-3.0 * PI, -2.0 * PI, -PI, 0.0, PI, 2.0 * PI, 3.0 * PI])
        .maxima(&[-1.5 * PI, 0.5 * PI, 2.5 * PI])
        .minima(&[-2.5 * PI, -0.5 * PI, 1.5 * PI])
        .inflections(Some(&[
            -3.0 * PI,
            -2.0 * PI,
            -PI,
            0.0,
            PI,
            2.0 * PI,
            3.0 * PI,
        ])),
    curve("cosine", "cos(x)", [-10.0, 10.0])
        .zeros(&[
            -2.5 * PI,
            -1.5 * PI,
            -0.5 * PI,
            0.5 * PI,
            1.5 * PI,
            2.5 * PI,
        ])
        .maxima(&[-2.0 * PI, 0.0, 2.0 * PI])
        .minima(&[-3.0 * PI, -PI, PI, 3.0 * PI])
        .inflections(Some(&[
            -2.5 * PI,
            -1.5 * PI,
            -0.5 * PI,
            0.5 * PI,
            1.5 * PI,
            2.5 * PI,
        ])),
    curve("tangent", "tan(x)", [-5.0, 5.0])
        .poles(&[-1.5 * PI, -0.5 * PI, 0.5 * PI, 1.5 * PI])
        .zeros(&[-PI, 0.0, PI])
        .inflections(Some(&[-PI, 0.0, PI])),
    curve("reciprocal", "1/x", [-5.0, 5.0])
        .poles(&[0.0])
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("shifted reciprocal", "1/(x - 1)", [-4.0, 4.0])
        .poles(&[1.0])
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("inverse square", "1/x^2", [-3.0, 3.0])
        .poles(&[0.0])
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("line with a hole", "(x^2 - 1)/(x - 1)", [-3.0, 3.0])
        .holes(&[1.0])
        .zeros(&[-1.0]),
    curve("sinc", "sin(x)/x", [-20.0, 20.0])
        .holes(&[0.0])
        .zeros(&[
            -6.0 * PI,
            -5.0 * PI,
            -4.0 * PI,
            -3.0 * PI,
            -2.0 * PI,
            -PI,
            PI,
            2.0 * PI,
            3.0 * PI,
            4.0 * PI,
            5.0 * PI,
            6.0 * PI,
        ])
        .maxima(&[-TAN_X_X[3], -TAN_X_X[1], TAN_X_X[1], TAN_X_X[3]])
        .minima(&[
            -TAN_X_X[4],
            -TAN_X_X[2],
            -TAN_X_X[0],
            TAN_X_X[0],
            TAN_X_X[2],
            TAN_X_X[4],
        ])
        .inflections(None)
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("natural logarithm", "ln(x)", [-2.0, 5.0])
        .poles(&[0.0])
        .undefined(&[[-2.0, 0.0]])
        .zeros(&[1.0]),
    curve("square root", "sqrt(x)", [-4.0, 4.0])
        .undefined(&[[-4.0, 0.0]])
        .zeros(&[0.0]),
    curve("semicircle", "sqrt(1 - x^2)", [-2.0, 2.0])
        .undefined(&[[-2.0, -1.0], [1.0, 2.0]])
        .zeros(&[-1.0, 1.0])
        .maxima(&[0.0]),
    curve("narrow gap", "sqrt(x^2 - 0.000001)", [-1.0, 1.0])
        .undefined(&[[-0.001, 0.001]])
        .zeros(&[-0.001, 0.001]),
    curve("two poles", "1/(x^2 - 4)", [-5.0, 5.0])
        .poles(&[-2.0, 2.0])
        .maxima(&[0.0])
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("bounded rational", "x/(x^2 + 1)", [-5.0, 5.0])
        .zeros(&[0.0])
        .maxima(&[1.0])
        .minima(&[-1.0])
        .inflections(Some(&[-SQRT_3, 0.0, SQRT_3]))
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("exponential", "exp(x)", [-3.0, 3.0]).limits(Some(0.0), None),
    curve("bell", "exp(-x^2)", [-3.0, 3.0])
        .maxima(&[0.0])
        .inflections(Some(&[-FRAC_1_SQRT_2, FRAC_1_SQRT_2]))
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("floor", "floor(x)", [-2.5, 2.5]).jumps(&[-2.0, -1.0, 0.0, 1.0, 2.0]),
    curve("sign by division", "x/abs(x)", [-2.0, 2.0]).jumps(&[0.0]),
    curve("absolute value", "abs(x)", [-2.0, 2.0])
        .zeros(&[0.0])
        .minima(&[0.0]),
    curve("secant", "sec(x)", [-4.0, 4.0])
        .poles(&[-0.5 * PI, 0.5 * PI])
        .maxima(&[-PI, PI])
        .minima(&[0.0]),
    curve("cotangent", "cot(x)", [-3.0, 3.0])
        .poles(&[0.0])
        .zeros(&[-0.5 * PI, 0.5 * PI])
        .inflections(Some(&[-0.5 * PI, 0.5 * PI])),
    curve("cosecant", "csc(x)", [0.5, 6.0])
        .poles(&[PI])
        .maxima(&[1.5 * PI])
        .minima(&[0.5 * PI]),
    curve("inverse hyperbolic tangent", "atanh(x)", [-2.0, 2.0])
        .poles(&[-1.0, 1.0])
        .undefined(&[[-2.0, -1.0], [1.0, 2.0]])
        .zeros(&[0.0])
        .inflections(Some(&[0.0])),
    curve("arcsine", "asin(x)", [-2.0, 2.0])
        .undefined(&[[-2.0, -1.0], [1.0, 2.0]])
        .zeros(&[0.0])
        .inflections(Some(&[0.0])),
    curve("log of the absolute value", "ln(abs(x))", [-3.0, 3.0])
        .poles(&[0.0])
        .zeros(&[-1.0, 1.0]),
    curve("cubic over a line", "(x^3 - 8)/(x - 2)", [0.0, 4.0]).holes(&[2.0]),
    curve("logistic", "1/(1 + exp(-x))", [-6.0, 6.0])
        .inflections(Some(&[0.0]))
        .limits(Some(0.0), Some(1.0)),
    curve("fractional power", "x^(1/3)", [-8.0, 8.0])
        .undefined(&[[-8.0, 0.0]])
        .zeros(&[0.0]),
    curve("cube root", "cbrt(x)", [-8.0, 8.0])
        .zeros(&[0.0])
        .inflections(Some(&[0.0])),
    curve("hole and pole", "(x + 1)/(x^2 - 1)", [-3.0, 3.0])
        .poles(&[1.0])
        .holes(&[-1.0])
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("round", "round(x)", [-2.0, 2.0]).jumps(&[-1.5, -0.5, 0.5, 1.5]),
    curve("two linear poles", "1/((x - 1)*(x + 2))", [-4.0, 4.0])
        .poles(&[-2.0, 1.0])
        .maxima(&[-0.5])
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("one-sided pole", "exp(1/x)", [-2.0, 2.0])
        .poles(&[0.0])
        .inflections(Some(&[-0.5]))
        .limits(Some(1.0), Some(1.0)),
    curve("half-angle tangent", "tan(x/2)", [-10.0, 10.0])
        .poles(&[-3.0 * PI, -PI, PI, 3.0 * PI])
        .zeros(&[-2.0 * PI, 0.0, 2.0 * PI])
        .inflections(Some(&[-2.0 * PI, 0.0, 2.0 * PI])),
    curve("oblique asymptote", "x^2/(x - 3)", [0.0, 5.0])
        .poles(&[3.0])
        .zeros(&[0.0]),
    curve("log of a parabola", "ln(x^2 - 1)", [-3.0, 3.0])
        .poles(&[-1.0, 1.0])
        .undefined(&[[-1.0, 1.0]])
        .zeros(&[-SQRT_2, SQRT_2]),
    curve("shifted sinc", "sin(x)/(x - pi)", [0.0, 6.0])
        .holes(&[PI])
        .zeros(&[0.0])
        .inflections(None)
        .limits(BOTH_ZERO.0, BOTH_ZERO.1),
    curve("arccosine", "acos(x/2)", [-3.0, 3.0])
        .undefined(&[[-3.0, -2.0], [2.0, 3.0]])
        .zeros(&[2.0])
        .inflections(Some(&[0.0])),
];
