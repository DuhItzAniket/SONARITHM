// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The constant-power stereo panner (06 §3).
//!
//! Pan is never the only carrier of essential information (A11Y-14): a mono downmix or
//! NVDA's sound split must lose nothing. The panner only places a voice.

use sonarithm_core::Pan;

use crate::osc::sine_turns;

/// Left and right channel gains.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanGains {
    /// Left channel gain.
    pub left: f32,
    /// Right channel gain.
    pub right: f32,
}

impl PanGains {
    /// Constant-power gains for `pan`: with `θ = (pan + 1) · π/4`, left is `cos θ` and right
    /// is `sin θ`, so `left² + right² = 1` and the centre is −3 dB on each side. `pan` is
    /// limited to [−1, 1]; NaN is centred.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic only.
    #[must_use]
    pub fn constant_power(pan: Pan) -> Self {
        let turns = (Pan::clamped(pan.0).0 + 1.0) / 8.0;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "gains are in [0, 1]; samples are f32 (coding-standards §10)"
        )]
        Self {
            left: sine_turns(turns + 0.25) as f32,
            right: sine_turns(turns) as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_is_constant_across_the_field() {
        for i in -100..=100 {
            let g = PanGains::constant_power(Pan(f64::from(i) / 100.0));
            let power = g.left * g.left + g.right * g.right;
            assert!((power - 1.0).abs() < 1e-6, "pan {i}: {power}");
        }
    }

    #[test]
    fn centre_is_minus_3_db_and_the_extremes_are_one_sided() {
        let centre = PanGains::constant_power(Pan::CENTRE);
        assert_eq!(centre.left, centre.right);
        assert!((centre.left - core::f32::consts::FRAC_1_SQRT_2).abs() < 1e-7);
        assert_eq!(
            PanGains::constant_power(Pan(-1.0)),
            PanGains {
                left: 1.0,
                right: 0.0
            }
        );
        assert_eq!(
            PanGains::constant_power(Pan(1.0)),
            PanGains {
                left: 0.0,
                right: 1.0
            }
        );
        assert_eq!(PanGains::constant_power(Pan(f64::NAN)), centre);
        assert_eq!(
            PanGains::constant_power(Pan(7.0)),
            PanGains::constant_power(Pan(1.0))
        );
    }
}
