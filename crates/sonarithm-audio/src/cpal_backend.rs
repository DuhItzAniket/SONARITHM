// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The cpal backend (06 §6, ADR-0004): WASAPI shared mode on Windows, CoreAudio on macOS,
//! ALSA (and PipeWire or PulseAudio through it) on Linux.
//!
//! cpal cannot request WASAPI RAW or exclusive mode (cpal #1173, #1343), so on Windows the
//! OEM enhancement processing applies after SONARITHM's limiter (F-032); the WASAPI
//! backend of ADR-0016 avoids it. A stream on the default device follows default-device
//! changes where cpal supports it; a stream on a named device reports its loss through
//! [`StreamStatus::device_lost`](crate::StreamStatus::device_lost).

use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, ErrorKind, SampleFormat};
use sonarithm_render::Processor;

use crate::backend::{AudioBackend, AudioError, DeviceInfo, Stream, StreamConfig, StreamStatus};
use crate::callback::Callback;

/// Audio output through cpal's default host.
pub struct CpalBackend {
    host: cpal::Host,
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CpalBackend {
    /// The platform's default host.
    #[must_use]
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    fn device(&self, id: Option<&str>) -> Result<cpal::Device, AudioError> {
        match id {
            None => self
                .host
                .default_output_device()
                .ok_or(AudioError::NoDevice),
            Some(id) => self
                .host
                .output_devices()
                .map_err(backend)?
                .find(|device| device_id(device).as_deref() == Some(id))
                .ok_or(AudioError::DeviceNotFound),
        }
    }
}

fn backend(error: impl std::fmt::Display) -> AudioError {
    AudioError::Backend(error.to_string())
}

fn device_id(device: &cpal::Device) -> Option<String> {
    device.id().ok().map(|id| id.id().to_owned())
}

fn device_name(device: &cpal::Device) -> String {
    device.description().map_or_else(
        |_| "Unknown device".to_owned(),
        |description| description.name().to_owned(),
    )
}

impl AudioBackend for CpalBackend {
    fn name(&self) -> &'static str {
        "cpal"
    }

    fn devices(&self) -> Result<Vec<DeviceInfo>, AudioError> {
        let default = self
            .host
            .default_output_device()
            .and_then(|d| device_id(&d));
        let devices = self.host.output_devices().map_err(backend)?;
        Ok(devices
            .filter_map(|device| {
                let id = device_id(&device)?;
                let config = device.default_output_config().ok();
                Some(DeviceInfo {
                    is_default: default.as_deref() == Some(id.as_str()),
                    name: device_name(&device),
                    sample_rate: config
                        .as_ref()
                        .map(cpal::SupportedStreamConfig::sample_rate),
                    channels: config.as_ref().map(cpal::SupportedStreamConfig::channels),
                    id,
                })
            })
            .collect())
    }

    fn sample_rate(&self, device: Option<&str>) -> Result<u32, AudioError> {
        let config = self
            .device(device)?
            .default_output_config()
            .map_err(backend)?;
        Ok(config.sample_rate())
    }

    fn open(&mut self, config: &StreamConfig, processor: Processor) -> Result<Stream, AudioError> {
        let device = self.device(config.device.as_deref())?;
        let default = device.default_output_config().map_err(backend)?;
        let rate = processor.sample_rate();
        if default.sample_rate() != rate.get() {
            return Err(AudioError::SampleRate {
                plan_rate: rate,
                device_rate: default.sample_rate(),
            });
        }
        if default.sample_format() != SampleFormat::F32 {
            return Err(AudioError::Backend(format!(
                "the device's default sample format is {}, not 32-bit float",
                default.sample_format()
            )));
        }
        let channels = default.channels();
        let stream_config = cpal::StreamConfig {
            channels,
            sample_rate: rate.get(),
            buffer_size: config
                .buffer_frames
                .map_or(BufferSize::Default, BufferSize::Fixed),
        };
        let status = Arc::new(StreamStatus::default());
        let mut callback = Callback::new(processor, Arc::clone(&status));
        let errors = Arc::clone(&status);
        let stream = device
            .build_output_stream::<f32, _, _>(
                stream_config,
                move |data, info| {
                    let ts = info.timestamp();
                    let callback_time = Duration::from_nanos(nanos(ts.callback.as_nanos()));
                    let playback_time = Duration::from_nanos(nanos(ts.playback.as_nanos()));
                    callback.fill(
                        data,
                        usize::from(channels),
                        Some(callback_time),
                        Some(playback_time),
                    );
                },
                move |error| match error.kind() {
                    ErrorKind::Xrun => errors.xrun(),
                    ErrorKind::DeviceNotAvailable | ErrorKind::StreamInvalidated => {
                        errors.lose_device();
                    }
                    ErrorKind::DeviceChanged => errors.change_device(),
                    _ => {}
                },
                None,
            )
            .map_err(backend)?;
        stream.play().map_err(backend)?;
        Ok(Stream {
            status,
            buffer_frames: stream.buffer_size().ok(),
            device: device_name(&device),
            sample_rate: rate.get(),
            channels,
            _keep: Box::new(stream),
        })
    }
}

fn nanos(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
