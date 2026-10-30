// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Level metering for the UI (B01-T04, 09 §2): the master's peak over fixed windows.
//!
//! The control thread reads the published [`GraphStats`](crate::GraphStats) about 30 times
//! a second, and the engine publishes after every block. A meter that showed only the last
//! block's peak would miss most of the audio between reads. So the graph keeps the peak of
//! each [`METER_WINDOW`] and publishes the last complete window: a window is published for
//! as long as the next one takes to fill, which is longer than the time between two reads,
//! so every window is read at least once and no peak is lost.

use sonarithm_core::{SampleRate, Seconds};

/// The meter window: 50 ms. Longer than the 33 ms between two 30 Hz meter frames plus one
/// 10 ms control tick of jitter, so every window is read; short enough that a meter shows
/// a peak within 100 ms.
pub const METER_WINDOW: Seconds = Seconds(0.05);

/// The master's level over the last complete [`METER_WINDOW`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct MeterLevels {
    /// Peak absolute sample, left and right, after the limiter (1.0 is full scale).
    pub peak: [f32; 2],
    /// Whether the master limiter engaged during the window (06 §10).
    pub limiting: bool,
}

/// Accumulates block peaks into windows.
///
/// # Real-time safety
///
/// RT-safe: fixed-size state, arithmetic only.
#[derive(Clone, Debug)]
pub(crate) struct MeterWindow {
    length: u64,
    frames: u64,
    peak: [f32; 2],
    limited_at_start: u64,
    last: MeterLevels,
}

impl MeterWindow {
    pub(crate) fn new(rate: SampleRate) -> Self {
        Self {
            length: METER_WINDOW
                .to_frames(rate)
                .map_or(1, |frames| frames.0.max(1)),
            frames: 0,
            peak: [0.0; 2],
            limited_at_start: 0,
            last: MeterLevels::default(),
        }
    }

    /// Adds a block of `frames` with peak `peak`; `limited` is the limiter's running count
    /// of engaged frames. Returns the last complete window.
    pub(crate) fn push(&mut self, frames: u64, peak: [f32; 2], limited: u64) -> MeterLevels {
        self.peak = [self.peak[0].max(peak[0]), self.peak[1].max(peak[1])];
        self.frames += frames;
        if self.frames >= self.length {
            self.last = MeterLevels {
                peak: self.peak,
                limiting: limited > self.limited_at_start,
            };
            self.frames = 0;
            self.peak = [0.0; 2];
            self.limited_at_start = limited;
        }
        self.last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> MeterWindow {
        // 50 ms at 48 kHz: 2 400 frames.
        MeterWindow::new(SampleRate::HZ_48000)
    }

    #[test]
    fn a_window_is_published_only_once_it_is_full() {
        let mut meter = window();
        assert_eq!(meter.push(1_200, [0.5, 0.25], 0).peak, [0.0, 0.0]);
        assert_eq!(meter.push(1_200, [0.1, 0.1], 0).peak, [0.5, 0.25]);
    }

    #[test]
    fn a_short_peak_inside_a_window_is_kept() {
        let mut meter = window();
        // One loud 256-frame block among quiet ones.
        let blocks = [0.1, 0.9, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1];
        let mut levels = MeterLevels::default();
        for peak in blocks {
            levels = meter.push(256, [peak, peak], 0);
        }
        assert!((levels.peak[0] - 0.9).abs() < f32::EPSILON, "{levels:?}");
        // The next window forgets it.
        for _ in 0..10 {
            levels = meter.push(256, [0.1, 0.1], 0);
        }
        assert!((levels.peak[0] - 0.1).abs() < f32::EPSILON, "{levels:?}");
    }

    #[test]
    fn limiting_is_reported_for_the_window_it_happened_in() {
        let mut meter = window();
        assert!(meter.push(2_400, [1.0, 1.0], 30).limiting);
        assert!(!meter.push(2_400, [0.5, 0.5], 30).limiting);
    }
}
