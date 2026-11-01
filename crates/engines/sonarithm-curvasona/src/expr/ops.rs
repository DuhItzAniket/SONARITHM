// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The operators exmex evaluates with: its default set, but every function computed by the
//! pure-Rust `libm` crate (ADR-0013 §1). exmex's own operators call the `std` float
//! functions, which differ in the last bits between platforms, so a function sampled on
//! Linux, macOS and Windows gave three tables, three data fingerprints and three Scores
//! (F-204). Only the operators the facade's lowered form uses are defined: anything else
//! is not a name exmex can read.

use exmex::{BinOp, MakeOperators, Operator};

/// exmex's operator set with `libm` maths.
#[derive(Clone, Debug)]
pub(crate) struct LibmOps;

/// The unary functions the lowered form calls: every [`super::Func1`] name except `sec`,
/// `csc` and `cot`, which lower to `1/cos`, `1/sin` and `1/tan`.
pub(crate) const UNARY: [(&str, fn(f64) -> f64); 23] = [
    ("abs", libm::fabs),
    ("acos", libm::acos),
    ("acosh", libm::acosh),
    ("asin", libm::asin),
    ("asinh", libm::asinh),
    ("atan", libm::atan),
    ("atanh", libm::atanh),
    ("cbrt", libm::cbrt),
    ("ceil", libm::ceil),
    ("cos", libm::cos),
    ("cosh", libm::cosh),
    ("exp", libm::exp),
    ("floor", libm::floor),
    ("ln", libm::log),
    ("log10", libm::log10),
    ("log2", libm::log2),
    ("round", libm::round),
    ("sin", libm::sin),
    ("sinh", libm::sinh),
    ("sqrt", libm::sqrt),
    ("tan", libm::tan),
    ("tanh", libm::tanh),
    ("trunc", libm::trunc),
];

/// The two-argument functions, as [`super::Func2`] names them.
pub(crate) const BINARY: [(&str, fn(f64, f64) -> f64); 3] = [
    ("atan2", libm::atan2),
    ("max", libm::fmax),
    ("min", libm::fmin),
];

const fn bin(apply: fn(f64, f64) -> f64, prio: i64, is_commutative: bool) -> BinOp<f64> {
    BinOp {
        apply,
        prio,
        is_commutative,
    }
}

impl MakeOperators<f64> for LibmOps {
    fn make<'a>() -> Vec<Operator<'a, f64>> {
        // The arithmetic operators and their priorities are exmex's own; IEEE 754 makes
        // `+ - * /` the same everywhere, and `^` is libm's `pow`.
        let mut ops = vec![
            Operator::make_bin("^", bin(libm::pow, 4, false)),
            Operator::make_bin("*", bin(|a, b| a * b, 2, true)),
            Operator::make_bin("/", bin(|a, b| a / b, 3, false)),
            Operator::make_bin_unary("+", bin(|a, b| a + b, 0, true), |a| a),
            Operator::make_bin_unary("-", bin(|a, b| a - b, 1, false), |a| -a),
        ];
        ops.extend(
            BINARY
                .iter()
                .map(|&(name, apply)| Operator::make_bin(name, bin(apply, 0, false))),
        );
        ops.extend(
            UNARY
                .iter()
                .map(|&(name, apply)| Operator::make_unary(name, apply)),
        );
        ops
    }
}
