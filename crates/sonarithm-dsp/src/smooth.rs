// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Parameter smoothing (coding-standards §11.3): every parameter that reaches the audio
//! thread from outside (gain, pan, speed, brightness) moves along a linear ramp instead of
//! jumping, which avoids zipper noise and clicks (sound-design §12 HS-01).

use sonarithm_core::{SampleRate, Seconds};

use crate::env::ramp_frames;

/// The default smoothing time: 10 ms (sound-design §5, proposal: "continuous parameter and
/// level changes are smoothed").
pub const SMOOTHING_TIME: Seconds = Seconds(0.010);

/// A value that moves linearly to each new target over a fixed number of frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearSmoother {
    current: f64,
    target: f64,
    step: f64,
    remaining: u32,
    frames: u32,
}

impl LinearSmoother {
    /// A smoother resting at `initial` (0 if not finite) that ramps over `time`, at least
    /// one frame.
    #[must_use]
    pub fn new(initial: f32, time: Seconds, rate: SampleRate) -> Self {
        let initial = if initial.is_finite() {
            f64::from(initial)
        } else {
            0.0
        };
        Self {
            current: initial,
            target: initial,
            step: 0.0,
            remaining: 0,
            frames: ramp_frames(time, rate),
        }
    }

    /// Starts a ramp from the current value to `target`. A non-finite target is ignored.
    ///
    /// # Real-time safety
    ///
    /// RT-safe.
    pub fn set_target(&mut self, target: f32) {
        if !target.is_finite() {
            return;
        }
        self.target = f64::from(target);
        self.remaining = self.frames;
        self.step = (self.target - self.current) / f64::from(self.frames);
    }

    /// The value for the next frame. The last frame of a ramp lands exactly on the target.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: arithmetic only.
    #[inline]
    pub fn next_value(&mut self) -> f32 {
        if self.remaining > 0 {
            self.remaining -= 1;
            self.current = if self.remaining == 0 {
                self.target
            } else {
                self.current + self.step
            };
        }
        self.value()
    }

    /// The current value.
    #[must_use]
    pub fn value(&self) -> f32 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "parameter values are f32 on the audio thread (coding-standards §10)"
        )]
        let value = self.current as f32;
        value
    }

    /// The value the ramp is heading to.
    #[must_use]
    pub fn target(&self) -> f32 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "targets come from f32 values"
        )]
        let target = self.target as f32;
        target
    }

    /// Whether the value has reached its target.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        self.remaining == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: SampleRate = SampleRate::HZ_48000;

    #[test]
    fn a_ramp_is_linear_and_lands_exactly_on_the_target() {
        let mut s = LinearSmoother::new(0.0, SMOOTHING_TIME, RATE);
        s.set_target(1.0);
        let values: Vec<f32> = (0..480).map(|_| s.next_value()).collect();
        assert!(values.windows(2).all(|w| w[1] > w[0]));
        assert_eq!(values[479], 1.0);
        assert!(s.is_settled());
        assert_eq!(s.next_value(), 1.0);
    }

    #[test]
    fn retargeting_mid_ramp_continues_from_the_current_value() {
        let mut s = LinearSmoother::new(0.0, SMOOTHING_TIME, RATE);
        s.set_target(1.0);
        for _ in 0..240 {
            s.next_value();
        }
        let before = s.value();
        s.set_target(0.0);
        let after = s.next_value();
        assert!((before - after).abs() < 0.002, "{before} → {after}");
        for _ in 0..479 {
            s.next_value();
        }
        assert_eq!(s.value(), 0.0);
    }

    #[test]
    fn non_finite_values_are_ignored() {
        let mut s = LinearSmoother::new(f32::NAN, Seconds(0.0), RATE);
        assert_eq!(s.value(), 0.0);
        s.set_target(f32::INFINITY);
        assert!(s.is_settled());
        s.set_target(0.5);
        assert_eq!(
            s.next_value(),
            0.5,
            "a zero-length ramp still takes one frame"
        );
    }
}
