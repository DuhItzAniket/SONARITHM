// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Platform-independent maths for every value that reaches the Score IR.
//!
//! The `std` float functions (`f64::powf`, `exp`, `ln`, `sin`, …) call the platform's maths
//! library, whose results can differ in the last bits between operating systems and CPUs.
//! The Score IR must be byte-identical everywhere (ADR-0013, 05 §4.2 invariant 6), so every
//! Score-affecting transcendental function goes through this module, which wraps the
//! pure-Rust [`libm`](https://crates.io/crates/libm) crate. Basic arithmetic (`+`, `-`, `*`,
//! `/`), `sqrt` and rounding are exact in IEEE 754 and need no wrapper.
//!
//! The crates whose output reaches the Score deny the `std` equivalents with clippy's
//! `disallowed-methods` (their `clippy.toml`).

/// `2^x`.
#[must_use]
#[inline]
pub fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}

/// `e^x`.
#[must_use]
#[inline]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// Natural logarithm.
#[must_use]
#[inline]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

/// Base-2 logarithm.
#[must_use]
#[inline]
pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}

/// Base-10 logarithm.
#[must_use]
#[inline]
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}

/// `base^exponent`.
#[must_use]
#[inline]
pub fn pow(base: f64, exponent: f64) -> f64 {
    libm::pow(base, exponent)
}

/// Sine of `x` radians.
#[must_use]
#[inline]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

/// Cosine of `x` radians.
#[must_use]
#[inline]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exact results that any correct implementation must give; they pin the wrapper to
    /// the right libm function.
    #[test]
    fn wrappers_call_the_matching_function() {
        assert_eq!(exp2(10.0), 1024.0);
        assert_eq!(exp(0.0), 1.0);
        assert_eq!(ln(1.0), 0.0);
        assert_eq!(log2(1024.0), 10.0);
        assert_eq!(log10(1000.0), 3.0);
        assert_eq!(pow(10.0, 2.0), 100.0);
        assert_eq!(sin(0.0), 0.0);
        assert_eq!(cos(0.0), 1.0);
    }

    /// The bit patterns below are the correctly rounded results (checked against 40-digit
    /// decimal arithmetic). They must not change on any platform; the test runs on every
    /// CI OS (ADR-0013).
    #[test]
    fn results_are_bit_identical_across_platforms() {
        assert_eq!(log2(1.5).to_bits(), 0x3fe2_b803_473f_7ad1);
        assert_eq!(exp2(0.1).to_bits(), 0x3ff1_25fb_ee25_0664);
        assert_eq!(pow(10.0, -0.9).to_bits(), 0x3fc0_1d3f_2d96_84d0);
        assert_eq!(sin(1.0).to_bits(), 0x3fea_ed54_8f09_0cee);
    }
}
