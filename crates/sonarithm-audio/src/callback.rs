// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The device callback adapter: everything every backend does on the audio thread around
//! [`Processor::process`] (06 §11, coding-standards §11).
//!
//! It renders stereo into a preallocated scratch buffer in pieces of at most
//! [`MAX_CHUNK`] frames, spreads it over the device's channels, counts late callbacks from
//! the device's own timestamps, and catches a panic as a last resort: the callback then
//! outputs silence and raises the fault flag for the control thread (06 §11). Everything
//! here is RT-safe once constructed.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::time::Duration;

use sonarithm_render::Processor;

use crate::backend::StreamStatus;

/// The most frames rendered in one piece; larger device buffers are rendered in pieces.
pub(crate) const MAX_CHUNK: usize = 4_096;

/// A callback is late when it starts this many periods after the previous one.
const LATE_FACTOR: f64 = 1.5;

/// The state that moves to the audio thread with the processor.
pub(crate) struct Callback {
    processor: Processor,
    scratch: Vec<f32>,
    status: Arc<StreamStatus>,
    rate: f64,
    rate_hz: u64,
    last: Option<(Duration, usize)>,
    faulted: bool,
}

impl Callback {
    /// Allocates the scratch buffer: call it off the audio thread.
    pub(crate) fn new(processor: Processor, status: Arc<StreamStatus>) -> Self {
        let processor_rate = processor.sample_rate().get();
        let rate = f64::from(processor_rate);
        Self {
            processor,
            scratch: vec![0.0; 2 * MAX_CHUNK],
            status,
            rate,
            rate_hz: u64::from(processor_rate),
            last: None,
            faulted: false,
        }
    }

    /// Fills interleaved `out` with `channels` channels. `callback_time` is the device's
    /// timestamp for this callback and `playback_time` its estimate of when the first frame
    /// is heard; both come from the backend, never from a clock read here.
    ///
    /// # Real-time safety
    ///
    /// RT-safe: no allocation, locks or I/O; a panic is caught, not propagated.
    pub(crate) fn fill(
        &mut self,
        out: &mut [f32],
        channels: usize,
        callback_time: Option<Duration>,
        playback_time: Option<Duration>,
    ) {
        self.status.callback();
        let frames = out.len() / channels.max(1);
        if let (Some(now), Some((before, before_frames))) = (callback_time, self.last) {
            #[expect(clippy::cast_precision_loss, reason = "buffer sizes are small")]
            let period = before_frames as f64 / self.rate;
            if now.saturating_sub(before).as_secs_f64() > LATE_FACTOR * period {
                self.status.late_callback();
            }
        }
        if let Some(now) = callback_time {
            self.last = Some((now, frames));
            if let Some(heard) = playback_time {
                self.status.report_latency(heard.saturating_sub(now));
            }
        }
        if self.faulted {
            out.fill(0.0);
            return;
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.render(out, channels, playback_time);
        }));
        if result.is_err() {
            // Last resort (06 §11): silence, and the control thread tears the stream down.
            self.faulted = true;
            self.status.set_fault();
            out.fill(0.0);
        }
    }

    fn render(&mut self, out: &mut [f32], channels: usize, playback_time: Option<Duration>) {
        let channels = channels.max(1);
        let mut done = 0_u32;
        for chunk in out.chunks_mut(MAX_CHUNK * channels) {
            let frames = chunk.len() / channels;
            let Some(stereo) = self.scratch.get_mut(..2 * frames) else {
                chunk.fill(0.0);
                continue;
            };
            // Each piece is heard `done` frames after the callback's first frame.
            let offset =
                Duration::from_nanos(u64::from(done) * 1_000_000_000 / self.rate_hz.max(1));
            let time = playback_time.map(|t| t + offset);
            self.processor.process(stereo, time);
            spread(stereo, chunk, channels);
            done = done.saturating_add(u32::try_from(frames).unwrap_or(u32::MAX));
        }
    }
}

/// Writes stereo frames to a device buffer: mono gets the average of both channels, stereo
/// is copied, and channels beyond the second are silent.
fn spread(stereo: &[f32], out: &mut [f32], channels: usize) {
    for (frame, &[l, r]) in out
        .chunks_exact_mut(channels)
        .zip(stereo.as_chunks::<2>().0)
    {
        match frame {
            [mono] => *mono = f32::midpoint(l, r),
            [left, right, rest @ ..] => {
                *left = l;
                *right = r;
                rest.fill(0.0);
            }
            [] => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_spreads_over_any_channel_count() {
        let stereo = [0.5, -0.5, 1.0, 0.0];
        let mut mono = [9.0; 2];
        spread(&stereo, &mut mono, 1);
        assert_eq!(mono, [0.0, 0.5]);
        let mut quad = [9.0; 8];
        spread(&stereo, &mut quad, 4);
        assert_eq!(quad, [0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
    }
}
