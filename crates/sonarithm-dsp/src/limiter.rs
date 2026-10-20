// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The master true-peak limiter (06 §10, sound-design §12 HS-02): look-ahead, stereo-linked,
//! ceiling −1 dBTP.
//!
//! The limiter estimates the true peak between samples with a 4× polyphase interpolator,
//! as ITU-R BS.1770 does for metering. It turns that into the gain each sample needs,
//! takes the minimum over the look-ahead window, and smooths it with a moving average over
//! the same window. Delaying the signal by the window plus the interpolator delay then
//! guarantees that the gain has fallen far enough by the time each peak leaves the delay
//! line. The gain recovers with a one-pole release. Everything is preallocated at
//! construction; [`TruePeakLimiter::process`] is RT-safe.

use sonarithm_core::{Db, SampleRate, Seconds};

use crate::env::ramp_frames;
use crate::osc::sine_turns;

/// The output ceiling: −1 dBTP (06 §10; sound-design §12 HS-02 proposal).
pub const CEILING: Db = Db(-1.0);
/// The ceiling the gain computer aims at: 0.2 dB under [`CEILING`], to absorb the
/// interpolator's estimation error and the inter-sample overshoot of the gain changes.
const TARGET: Db = Db(-1.2);
/// Look-ahead: 2 ms. A05 provisional value.
pub const LOOKAHEAD: Seconds = Seconds(0.002);
/// Release time constant: 100 ms. A05 provisional value.
pub const RELEASE: Seconds = Seconds(0.100);

/// Interpolator taps per phase; four phases give 4× oversampling.
const TAPS: usize = 12;
const PHASES: usize = 4;
/// Input samples between the newest sample and the start of the interpolated interval.
const INTERPOLATOR_DELAY: usize = TAPS / 2;

/// Gain below 1 means the limiter is reducing the level.
const ENGAGED: f32 = 0.999;

/// A stereo-linked look-ahead true-peak limiter.
#[derive(Clone, Debug)]
pub struct TruePeakLimiter {
    target: f32,
    window: usize,
    release: f32,
    /// Polyphase interpolator coefficients, `[phase][tap]`.
    coefficients: [[f32; TAPS]; PHASES],
    /// The largest sum of absolute coefficients over the phases: no interpolated value
    /// exceeds this times the largest input sample in the window.
    overshoot: f32,
    /// The input history per channel, written twice (at `head` and `head + TAPS`) so the
    /// last `TAPS` samples are always one contiguous slice, newest first.
    history: [[f32; 2 * TAPS]; 2],
    head: usize,
    /// Delay lines per channel, `latency + 1` long, written circularly.
    delay: [Vec<f32>; 2],
    /// Monotonic queue of (sample index, required gain) for the window minimum.
    queue: Vec<(u64, f32)>,
    queue_head: usize,
    queue_len: usize,
    /// The last `window` minima, for the moving average.
    minima: Vec<f32>,
    sum: f64,
    /// Next slot in `minima`, and next write position in the delay lines.
    slot: usize,
    write: usize,
    gain: f32,
    index: u64,
    engaged: u64,
    least_gain: f32,
}

impl TruePeakLimiter {
    /// A limiter for `rate`. Allocates its delay lines: call it off the audio thread.
    #[must_use]
    pub fn new(rate: SampleRate) -> Self {
        let window = ramp_frames(LOOKAHEAD, rate) as usize;
        let latency = window - 1 + INTERPOLATOR_DELAY;
        let release_frames = RELEASE.0 * f64::from(rate.get());
        #[expect(clippy::cast_possible_truncation, reason = "gains are f32")]
        let (target, release) = (
            TARGET.to_gain() as f32,
            libm::exp(-1.0 / release_frames) as f32,
        );
        Self {
            target,
            window,
            release,
            coefficients: interpolator(),
            overshoot: interpolator()
                .iter()
                .map(|phase| phase.iter().map(|h| h.abs()).sum::<f32>())
                .fold(1.0, f32::max),
            history: [[0.0; 2 * TAPS]; 2],
            head: 0,
            delay: [vec![0.0; latency + 1], vec![0.0; latency + 1]],
            queue: vec![(0, 1.0); window + 1],
            queue_head: 0,
            queue_len: 0,
            minima: vec![1.0; window],
            sum: window as f64,
            slot: 0,
            write: 0,
            gain: 1.0,
            index: 0,
            engaged: 0,
            least_gain: 1.0,
        }
    }

    /// The delay the limiter adds, in frames.
    #[must_use]
    pub fn latency(&self) -> usize {
        self.delay[0].len() - 1
    }

    /// Frames processed with the gain below unity (limiter engaged), since construction.
    #[must_use]
    pub fn engaged_frames(&self) -> u64 {
        self.engaged
    }

    /// The deepest gain reduction so far, in dB (0 or negative).
    #[must_use]
    pub fn max_reduction(&self) -> Db {
        Db::from_gain(f64::from(self.least_gain))
    }

    /// Limits `left` and `right` in place, delayed by [`TruePeakLimiter::latency`] frames.
    /// Non-finite input samples are treated as silence.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: no allocation; bounded by the buffer length and the window size.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let input = [finite(*l), finite(*r)];
            let peak = self.true_peak(input);
            let required = if peak > self.target {
                self.target / peak
            } else {
                1.0
            };
            let minimum = self.window_minimum(required);
            let old = self
                .minima
                .get_mut(self.slot)
                .map_or(1.0, |m| core::mem::replace(m, minimum));
            self.slot = (self.slot + 1) % self.window;
            self.sum += f64::from(minimum) - f64::from(old);
            #[expect(clippy::cast_possible_truncation, reason = "gains are f32")]
            let smoothed = (self.sum / self.window as f64) as f32;
            self.gain = if smoothed < self.gain {
                smoothed
            } else {
                smoothed + (self.gain - smoothed) * self.release
            };
            let length = self.delay[0].len();
            let read = (self.write + 1) % length;
            for (line, (out, sample)) in self.delay.iter_mut().zip([l, r].into_iter().zip(input)) {
                if let Some(slot) = line.get_mut(self.write) {
                    *slot = sample;
                }
                *out = line.get(read).copied().unwrap_or(0.0) * self.gain;
            }
            self.write = read;
            if self.gain < ENGAGED {
                self.engaged += 1;
            }
            self.least_gain = self.least_gain.min(self.gain);
            self.index += 1;
        }
    }

    /// Pushes one frame into the interpolator and returns the stereo true peak of the
    /// interval between the two samples `INTERPOLATOR_DELAY` frames back.
    fn true_peak(&mut self, input: [f32; 2]) -> f32 {
        // Move the head back one place; the newest sample goes at `head` and `head + TAPS`.
        self.head = (self.head + TAPS - 1) % TAPS;
        let head = self.head;
        let mut peak = 0.0_f32;
        for (history, sample) in self.history.iter_mut().zip(input) {
            if let Some(slot) = history.get_mut(head) {
                *slot = sample;
            }
            if let Some(slot) = history.get_mut(head + TAPS) {
                *slot = sample;
            }
            let Some(window) = history.get(head..head + TAPS) else {
                continue;
            };
            peak = peak
                .max(window.get(INTERPOLATOR_DELAY).map_or(0.0, |x| x.abs()))
                .max(window.get(INTERPOLATOR_DELAY - 1).map_or(0.0, |x| x.abs()));
            // When even the worst-case overshoot stays under the target, the interpolated
            // values cannot matter: skip them. The gain is the same either way.
            let largest = window.iter().fold(0.0_f32, |m, x| m.max(x.abs()));
            if largest * self.overshoot <= self.target {
                continue;
            }
            for phase in &self.coefficients {
                let value: f32 = phase.iter().zip(window).map(|(h, x)| h * x).sum();
                peak = peak.max(value.abs());
            }
        }
        peak
    }

    /// The minimum required gain over the last `window` samples (a monotonic queue).
    fn window_minimum(&mut self, required: f32) -> f32 {
        let capacity = self.queue.len();
        // Drop entries from the back that are no smaller than the new one.
        while self.queue_len > 0 {
            let back = (self.queue_head + self.queue_len - 1) % capacity;
            if self.queue[back].1 >= required {
                self.queue_len -= 1;
            } else {
                break;
            }
        }
        let back = (self.queue_head + self.queue_len) % capacity;
        self.queue[back] = (self.index, required);
        self.queue_len += 1;
        // Drop the front once it is older than the window.
        let oldest = self.index.saturating_sub(self.window as u64 - 1);
        while self.queue_len > 1 && self.queue[self.queue_head].0 < oldest {
            self.queue_head = (self.queue_head + 1) % capacity;
            self.queue_len -= 1;
        }
        self.queue[self.queue_head].1
    }
}

#[inline]
fn finite(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}

/// A 4× interpolator: a Blackman-windowed sinc of `PHASES × TAPS` taps, split into phases
/// that each sum to 1. Phase `p` estimates the signal `(TAPS − 1)/2 − p/4 + 3/8` frames back,
/// all four inside the interval between the samples `TAPS/2` and `TAPS/2 − 1` frames back.
fn interpolator() -> [[f32; TAPS]; PHASES] {
    let n = PHASES * TAPS;
    let centre = (n as f64 - 1.0) / 2.0;
    let mut phases = [[0.0_f64; TAPS]; PHASES];
    for (i, h) in (0..n).map(|i| {
        let x = (i as f64 - centre) / PHASES as f64;
        let sinc = if x == 0.0 {
            1.0
        } else {
            sine_turns(x / 2.0) / (core::f64::consts::PI * x)
        };
        let w = i as f64 / (n as f64 - 1.0);
        let blackman = 0.42 - 0.5 * sine_turns(w + 0.25) + 0.08 * sine_turns(2.0 * w + 0.25);
        (i, sinc * blackman)
    }) {
        phases[i % PHASES][i / PHASES] = h;
    }
    phases.map(|phase| {
        let sum: f64 = phase.iter().sum();
        #[expect(clippy::cast_possible_truncation, reason = "coefficients are f32")]
        phase.map(|h| (h / sum) as f32)
    })
}

#[cfg(test)]
#[allow(
    clippy::cast_possible_truncation,
    reason = "test signals are stored as f32"
)]
mod tests {
    use super::*;

    const RATE: SampleRate = SampleRate::HZ_48000;

    fn run(limiter: &mut TruePeakLimiter, signal: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let (mut left, mut right) = (signal.to_vec(), signal.to_vec());
        for (l, r) in left.chunks_mut(256).zip(right.chunks_mut(256)) {
            limiter.process(l, r);
        }
        (left, right)
    }

    #[test]
    fn quiet_signals_pass_unchanged_after_the_latency() {
        let mut limiter = TruePeakLimiter::new(RATE);
        let latency = limiter.latency();
        assert_eq!(latency, 95 + 6);
        let signal: Vec<f32> = (0..4_800)
            .map(|i| 0.5 * sine_turns(f64::from(i) * 440.0 / 48_000.0) as f32)
            .collect();
        let (left, _) = run(&mut limiter, &signal);
        assert!(left[..latency].iter().all(|s| *s == 0.0));
        assert!(
            left[latency..]
                .iter()
                .zip(&signal)
                .all(|(y, x)| y.to_bits() == x.to_bits())
        );
        assert_eq!(limiter.engaged_frames(), 0);
    }

    #[test]
    fn a_full_scale_step_never_passes_the_target() {
        let mut limiter = TruePeakLimiter::new(RATE);
        let mut signal = vec![0.0_f32; 9_600];
        for (i, s) in signal.iter_mut().enumerate().skip(1_000) {
            *s = if (i / 20) % 2 == 0 { 4.0 } else { -4.0 };
        }
        let (left, right) = run(&mut limiter, &signal);
        let peak = left
            .iter()
            .chain(&right)
            .fold(0.0_f32, |m, s| m.max(s.abs()));
        assert!(
            peak <= TARGET.to_gain() as f32 * 1.000_1,
            "sample peak {peak}"
        );
        assert!(limiter.engaged_frames() > 0);
        assert!(limiter.max_reduction().0 < -12.0);
    }

    #[test]
    fn non_finite_samples_become_silence() {
        let mut limiter = TruePeakLimiter::new(RATE);
        let (left, _) = run(
            &mut limiter,
            &[f32::NAN, f32::INFINITY, 0.1, 0.0].repeat(100),
        );
        assert!(left.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn each_interpolator_phase_has_unity_dc_gain() {
        for phase in interpolator() {
            let sum: f32 = phase.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6);
        }
    }
}
