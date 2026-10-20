// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `AudioBackend` contract (06 §6, ADR-0004): every device API sits behind it, so cpal,
//! WASAPI and the test null device are interchangeable.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use sonarithm_core::SampleRate;
use sonarithm_render::Processor;

/// An output device.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DeviceInfo {
    /// A stable identifier for [`StreamConfig::device`].
    pub id: String,
    /// The name to show and announce.
    pub name: String,
    /// Whether this is the system default output.
    pub is_default: bool,
    /// The device's default sample rate, if known.
    pub sample_rate: Option<u32>,
    /// The device's default channel count, if known.
    pub channels: Option<u16>,
}

/// What to open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct StreamConfig {
    /// The device identifier, or `None` for the system default (which the backend may follow
    /// when the default changes).
    pub device: Option<String>,
    /// The buffer size to request in frames, or `None` for the device default. Drivers may
    /// ignore it (A02 found WASAPI shared mode fixed at 480 frames, F-030).
    pub buffer_frames: Option<u32>,
}

impl StreamConfig {
    /// The same configuration on the device `id`.
    #[must_use]
    pub fn on_device(self, id: impl Into<String>) -> Self {
        Self {
            device: Some(id.into()),
            ..self
        }
    }

    /// The same configuration requesting `frames` per callback.
    #[must_use]
    pub fn with_buffer(self, frames: u32) -> Self {
        Self {
            buffer_frames: Some(frames),
            ..self
        }
    }
}

/// Output latency of an open stream.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LatencyInfo {
    /// The callback buffer size the stream actually uses, in frames.
    pub buffer_frames: Option<u32>,
    /// The backend's own estimate of output latency. Only a fallback: A02 found cpal's
    /// estimate about 80 ms early on Windows (F-033); per-device calibration replaces it
    /// (ADR-0016).
    pub reported: Option<Duration>,
}

/// Why a backend failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AudioError {
    /// No output device is available.
    #[error("no audio output device is available")]
    NoDevice,
    /// The named device does not exist (any more).
    #[error("the audio device is not available")]
    DeviceNotFound,
    /// The device cannot play at the plan's sample rate; rebuild the plan at
    /// `device_rate` and try again.
    #[error("the audio device plays at {device_rate} Hz, not {plan_rate}")]
    SampleRate {
        /// The plan's rate.
        plan_rate: SampleRate,
        /// The device's rate.
        device_rate: u32,
    },
    /// The device API reported an error.
    #[error("the audio device failed: {0}")]
    Backend(String),
}

/// Counters and flags an open stream shares with the control thread. The audio callback
/// and the device's error callback write them with relaxed atomics: RT-safe.
#[derive(Debug, Default)]
pub struct StreamStatus {
    callbacks: AtomicU64,
    late: AtomicU64,
    xruns: AtomicU64,
    device_lost: AtomicBool,
    device_changed: AtomicBool,
    fault: AtomicBool,
    /// The backend's latest `playback − callback` estimate, in nanoseconds.
    reported_ns: AtomicU64,
    allocations: AtomicU64,
}

impl StreamStatus {
    pub(crate) fn callback(&self) {
        self.callbacks.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn late_callback(&self) {
        self.late.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn xrun(&self) {
        self.xruns.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn lose_device(&self) {
        self.device_lost.store(true, Ordering::Release);
    }

    pub(crate) fn change_device(&self) {
        self.device_changed.store(true, Ordering::Release);
    }

    pub(crate) fn set_fault(&self) {
        self.fault.store(true, Ordering::Release);
    }

    #[cfg_attr(
        not(feature = "count-allocations"),
        expect(dead_code, reason = "only the count-allocations feature counts")
    )]
    pub(crate) fn allocation(&self) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn report_latency(&self, latency: Duration) {
        let nanos = u64::try_from(latency.as_nanos()).unwrap_or(u64::MAX);
        self.reported_ns.store(nanos, Ordering::Relaxed);
    }

    /// Callbacks so far.
    #[must_use]
    pub fn callbacks(&self) -> u64 {
        self.callbacks.load(Ordering::Relaxed)
    }

    /// Callbacks that started more than 1.5 periods after the previous one: a stand-in for
    /// underruns where the backend does not report them (cpal on WASAPI, A02).
    #[must_use]
    pub fn late_callbacks(&self) -> u64 {
        self.late.load(Ordering::Relaxed)
    }

    /// Underruns the device API reported.
    #[must_use]
    pub fn xruns(&self) -> u64 {
        self.xruns.load(Ordering::Relaxed)
    }

    /// Callbacks that allocated or freed memory. Counted only with the `count-allocations`
    /// feature (coding-standards §11.5); always 0 without it.
    #[must_use]
    pub fn allocations(&self) -> u64 {
        self.allocations.load(Ordering::Relaxed)
    }

    /// Whether the device went away; the stream must be rebuilt (06 §11).
    #[must_use]
    pub fn device_lost(&self) -> bool {
        self.device_lost.load(Ordering::Acquire)
    }

    /// Whether the backend moved the stream to another device since the last call (a stream
    /// on the default device following a new default), and clears the flag.
    pub fn take_device_changed(&self) -> bool {
        self.device_changed.swap(false, Ordering::AcqRel)
    }

    /// Whether the callback panicked. It now outputs silence; tear the stream down (06 §11).
    #[must_use]
    pub fn fault(&self) -> bool {
        self.fault.load(Ordering::Acquire)
    }
}

/// An open output stream. Dropping it stops the device and the processor with it.
pub struct Stream {
    pub(crate) status: Arc<StreamStatus>,
    pub(crate) buffer_frames: Option<u32>,
    pub(crate) device: String,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
    /// Keeps the device stream alive (not `Send`: cpal streams are not on every OS).
    pub(crate) _keep: Box<dyn std::any::Any>,
}

impl Stream {
    /// The shared counters and flags.
    #[must_use]
    pub fn status(&self) -> &StreamStatus {
        &self.status
    }

    /// The stream's latency figures.
    #[must_use]
    pub fn latency(&self) -> LatencyInfo {
        let nanos = self.status.reported_ns.load(Ordering::Relaxed);
        LatencyInfo {
            buffer_frames: self.buffer_frames,
            reported: (nanos > 0).then(|| Duration::from_nanos(nanos)),
        }
    }

    /// The device's name.
    #[must_use]
    pub fn device(&self) -> &str {
        &self.device
    }

    /// The stream's sample rate.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// The device's channel count.
    #[must_use]
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

/// An audio output API.
pub trait AudioBackend {
    /// A short name, such as `cpal`.
    fn name(&self) -> &'static str;

    /// The output devices.
    ///
    /// # Errors
    ///
    /// [`AudioError::Backend`] if the device API fails.
    fn devices(&self) -> Result<Vec<DeviceInfo>, AudioError>;

    /// The sample rate a stream on `device` (the default if `None`) will run at, so the
    /// caller can build its plan at that rate first.
    ///
    /// # Errors
    ///
    /// [`AudioError::NoDevice`] or [`AudioError::DeviceNotFound`].
    fn sample_rate(&self, device: Option<&str>) -> Result<u32, AudioError>;

    /// Opens an output stream that plays `processor`, which moves to the audio thread.
    ///
    /// # Errors
    ///
    /// [`AudioError::SampleRate`] if the device rate differs from the plan's; others if the
    /// device cannot be opened.
    fn open(&mut self, config: &StreamConfig, processor: Processor) -> Result<Stream, AudioError>;
}
