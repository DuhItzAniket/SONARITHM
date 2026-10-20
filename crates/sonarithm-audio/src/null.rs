// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A timer-driven null device (testing-strategy §9.3): a thread that calls the callback in
//! fixed blocks, paced like a real device or as fast as possible, with no audio hardware.
//! It drives the allocation soak, the real-time-path equivalence test, and tests of device
//! loss, so they run on CI machines without sound cards.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rtrb::{Consumer, Producer, RingBuffer};
use sonarithm_render::Processor;

use crate::backend::{AudioBackend, AudioError, DeviceInfo, Stream, StreamConfig, StreamStatus};
use crate::callback::Callback;

/// The null device's identifier and name.
pub const NULL_DEVICE: &str = "null";

/// How the null device runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NullConfig {
    /// Frames per callback.
    pub block: usize,
    /// The device's sample rate.
    pub sample_rate: u32,
    /// Channels per frame.
    pub channels: u16,
    /// Sleep between callbacks to match real time; otherwise run as fast as possible.
    pub paced: bool,
    /// Interleaved samples to keep for inspection, at most.
    pub capture: usize,
}

impl NullConfig {
    /// Stereo at `sample_rate` in blocks of `block`, paced, capturing nothing.
    #[must_use]
    pub fn new(sample_rate: u32, block: usize) -> Self {
        Self {
            block: block.max(1),
            sample_rate,
            channels: 2,
            paced: true,
            capture: 0,
        }
    }
}

/// A backend whose one device is a thread.
pub struct NullBackend {
    config: NullConfig,
    captured: Option<Consumer<f32>>,
    current: Option<Arc<NullControl>>,
}

/// Control over an open null stream, for tests: stop it or make its device disappear.
#[derive(Debug)]
pub struct NullControl {
    stop: AtomicBool,
    status: Arc<StreamStatus>,
}

impl NullControl {
    /// Makes the device disappear, as an unplugged headset does.
    pub fn unplug(&self) {
        self.status.lose_device();
        self.stop.store(true, Ordering::Release);
    }
}

impl NullBackend {
    /// A null backend with `config`.
    #[must_use]
    pub fn new(config: NullConfig) -> Self {
        Self {
            config,
            captured: None,
            current: None,
        }
    }

    /// Takes the samples captured so far by the last opened stream (interleaved).
    pub fn take_captured(&mut self) -> Vec<f32> {
        let Some(consumer) = &mut self.captured else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(consumer.slots());
        while let Ok(sample) = consumer.pop() {
            out.push(sample);
        }
        out
    }

    /// Control over the last opened stream.
    #[must_use]
    pub fn control(&self) -> Option<Arc<NullControl>> {
        self.current.clone()
    }
}

/// Stops the device thread when the stream is dropped.
struct NullStream {
    control: Arc<NullControl>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for NullStream {
    fn drop(&mut self) {
        self.control.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl AudioBackend for NullBackend {
    fn name(&self) -> &'static str {
        "null"
    }

    fn devices(&self) -> Result<Vec<DeviceInfo>, AudioError> {
        Ok(vec![DeviceInfo {
            id: NULL_DEVICE.to_owned(),
            name: NULL_DEVICE.to_owned(),
            is_default: true,
            sample_rate: Some(self.config.sample_rate),
            channels: Some(self.config.channels),
        }])
    }

    fn sample_rate(&self, _device: Option<&str>) -> Result<u32, AudioError> {
        Ok(self.config.sample_rate)
    }

    fn open(&mut self, config: &StreamConfig, processor: Processor) -> Result<Stream, AudioError> {
        if config.device.as_deref().is_some_and(|id| id != NULL_DEVICE) {
            return Err(AudioError::DeviceNotFound);
        }
        let device = self.config;
        if processor.sample_rate().get() != device.sample_rate {
            return Err(AudioError::SampleRate {
                plan_rate: processor.sample_rate(),
                device_rate: device.sample_rate,
            });
        }
        let status = Arc::new(StreamStatus::default());
        let control = Arc::new(NullControl {
            stop: AtomicBool::new(false),
            status: Arc::clone(&status),
        });
        let (producer, consumer) = RingBuffer::new(device.capture.max(1));
        self.captured = Some(consumer);
        self.current = Some(Arc::clone(&control));
        let callback = Callback::new(processor, Arc::clone(&status));
        let thread_control = Arc::clone(&control);
        let thread = std::thread::Builder::new()
            .name("sonarithm-null-device".to_owned())
            .spawn(move || run(callback, device, producer, &thread_control))
            .map_err(|error| AudioError::Backend(error.to_string()))?;
        Ok(Stream {
            status,
            buffer_frames: u32::try_from(device.block).ok(),
            device: NULL_DEVICE.to_owned(),
            sample_rate: device.sample_rate,
            channels: device.channels,
            _keep: Box::new(NullStream {
                control,
                thread: Some(thread),
            }),
        })
    }
}

/// The device thread: one callback per block until stopped.
fn run(
    mut callback: Callback,
    device: NullConfig,
    mut capture: Producer<f32>,
    control: &NullControl,
) {
    let channels = usize::from(device.channels);
    let mut buffer = vec![0.0; device.block * channels];
    let period = Duration::from_nanos(
        u64::try_from(device.block).unwrap_or(u64::MAX) * 1_000_000_000
            / u64::from(device.sample_rate.max(1)),
    );
    let start = Instant::now();
    let mut blocks = 0_u32;
    while !control.stop.load(Ordering::Acquire) {
        // The device's clock: block n starts at n periods.
        let now = period * blocks;
        callback.fill(&mut buffer, channels, Some(now), Some(now + period));
        for &sample in &buffer {
            if capture.push(sample).is_err() {
                break;
            }
        }
        blocks = blocks.saturating_add(1);
        if device.paced {
            let due = start + period * blocks;
            if let Some(wait) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
        }
    }
}
