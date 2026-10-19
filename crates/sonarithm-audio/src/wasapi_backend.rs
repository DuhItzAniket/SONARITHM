// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Windows WASAPI backend (ADR-0016), on the `wasapi` crate.
//!
//! - **Shared mode with the RAW stream option by default.** RAW bypasses the OEM enhancement
//!   APOs, which on the development laptop add about 38 ms and +20.6 dB after SONARITHM's
//!   limiter (F-032). When the driver refuses RAW, the stream falls back to default
//!   processing and [`WasapiStreamInfo::raw`] says so.
//! - **Exclusive mode as an opt-in low-latency setting.** It takes the device away from
//!   every other application, so a screen reader speaking on the same device goes silent
//!   (ADR-0016). Callers must say so before enabling it.
//!
//! The device thread runs event-driven at "Pro Audio" priority. Underruns are detected from
//! the device clock: when the device has played every frame written so far, it ran dry.
//! When the default output device changes, a stream opened on the default device reports
//! itself lost, so the [`Player`](crate::Player) rebuilds it on the new default and pauses
//! (sound-design §12 HS-05).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use sonarithm_render::Processor;
use wasapi::{
    AudioClientProperties, DeviceEnumerator, DeviceEventCallbacks, Direction, SampleType,
    StreamMode, StreamOption, WaveFormat,
};

use crate::backend::{AudioBackend, AudioError, DeviceInfo, Stream, StreamConfig, StreamStatus};
use crate::callback::{Callback, MAX_CHUNK};

/// How the WASAPI stream shares the device.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum WasapiMode {
    /// Shared with other applications, RAW processing (ADR-0016 default).
    #[default]
    SharedRaw,
    /// Shared, with the device's default processing (enhancements on).
    Shared,
    /// Exclusive: lowest latency, but silences every other application on the device,
    /// including a screen reader's speech (ADR-0016). Opt-in only.
    Exclusive,
}

/// What a WASAPI stream ended up with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WasapiStreamInfo {
    /// The mode asked for.
    pub mode: WasapiMode,
    /// Whether the RAW option is in effect.
    pub raw: bool,
    /// The device buffer in frames.
    pub buffer_frames: u32,
    /// The device period in 100 ns units (default for shared, minimum for exclusive).
    pub period_hns: i64,
}

/// The WASAPI backend.
#[derive(Debug, Default)]
pub struct WasapiBackend {
    mode: WasapiMode,
    last: Option<WasapiStreamInfo>,
}

impl WasapiBackend {
    /// A backend opening streams in `mode`.
    #[must_use]
    pub fn new(mode: WasapiMode) -> Self {
        Self { mode, last: None }
    }

    /// Details of the last stream opened.
    #[must_use]
    pub fn last_stream(&self) -> Option<WasapiStreamInfo> {
        self.last
    }
}

fn backend(error: impl std::fmt::Display) -> AudioError {
    AudioError::Backend(error.to_string())
}

fn device(enumerator: &DeviceEnumerator, id: Option<&str>) -> Result<wasapi::Device, AudioError> {
    match id {
        None => enumerator
            .get_default_device(&Direction::Render)
            .map_err(|_| AudioError::NoDevice),
        Some(id) => enumerator
            .get_device(id)
            .map_err(|_| AudioError::DeviceNotFound),
    }
}

/// COM must be initialized on every thread that calls WASAPI. Repeated calls are harmless.
fn com() {
    let _ = wasapi::initialize_mta();
}

impl AudioBackend for WasapiBackend {
    fn name(&self) -> &'static str {
        "wasapi"
    }

    fn devices(&self) -> Result<Vec<DeviceInfo>, AudioError> {
        com();
        let enumerator = DeviceEnumerator::new().map_err(backend)?;
        let default = enumerator
            .get_default_device(&Direction::Render)
            .and_then(|d| d.get_id())
            .ok();
        let collection = enumerator
            .get_device_collection(&Direction::Render)
            .map_err(backend)?;
        let count = collection.get_nbr_devices().map_err(backend)?;
        let mut devices = Vec::new();
        for index in 0..count {
            let Ok(device) = collection.get_device_at_index(index) else {
                continue;
            };
            let Ok(id) = device.get_id() else { continue };
            let format = device
                .get_iaudioclient()
                .and_then(|c| c.get_mixformat())
                .ok();
            devices.push(DeviceInfo {
                is_default: default.as_deref() == Some(id.as_str()),
                name: device.get_friendlyname().unwrap_or_else(|_| id.clone()),
                sample_rate: format.as_ref().map(WaveFormat::get_samplespersec),
                channels: format.as_ref().map(WaveFormat::get_nchannels),
                id,
            });
        }
        Ok(devices)
    }

    fn sample_rate(&self, device_id: Option<&str>) -> Result<u32, AudioError> {
        com();
        let enumerator = DeviceEnumerator::new().map_err(backend)?;
        let client = device(&enumerator, device_id)?
            .get_iaudioclient()
            .map_err(backend)?;
        Ok(client.get_mixformat().map_err(backend)?.get_samplespersec())
    }

    fn open(&mut self, config: &StreamConfig, processor: Processor) -> Result<Stream, AudioError> {
        com();
        let rate = processor.sample_rate();
        let mode = self.mode;
        let device_id = config.device.clone();
        let status = Arc::new(StreamStatus::default());
        let stop = Arc::new(AtomicBool::new(false));
        let callback = Callback::new(processor, Arc::clone(&status));
        let (ready_tx, ready) = mpsc::channel();
        let (thread_status, thread_stop) = (Arc::clone(&status), Arc::clone(&stop));
        let thread = std::thread::Builder::new()
            .name("sonarithm-wasapi".to_owned())
            .spawn(move || {
                let id = device_id.as_deref();
                match prepare(id, mode, rate.get()) {
                    Ok(prepared) => {
                        let _ = ready_tx.send(Ok((
                            prepared.info,
                            prepared.name.clone(),
                            prepared.channels,
                        )));
                        run(&prepared, callback, &thread_status, &thread_stop);
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                    }
                }
            })
            .map_err(backend)?;
        let (info, name, channels) = ready
            .recv()
            .map_err(|_| AudioError::Backend("the WASAPI thread ended".to_owned()))??;
        self.last = Some(info);
        // A stream on the default device follows the default: when it changes, report the
        // stream lost so the player rebuilds it on the new default (06 §11).
        let notifications = if config.device.is_none() {
            let changed = Arc::clone(&status);
            let mut callbacks = DeviceEventCallbacks::new();
            callbacks.set_default_device_callback(move |direction, _role, _id| {
                if direction == Direction::Render {
                    changed.change_device();
                    changed.lose_device();
                }
            });
            DeviceEnumerator::new()
                .and_then(|enumerator| enumerator.register_notification_callback(callbacks))
                .ok()
        } else {
            None
        };
        Ok(Stream {
            status,
            buffer_frames: Some(info.buffer_frames),
            device: name,
            sample_rate: rate.get(),
            channels,
            _keep: Box::new(WasapiStream {
                stop,
                thread: Some(thread),
                _notifications: notifications.map(|n| Box::new(n) as Box<dyn std::any::Any>),
            }),
        })
    }
}

/// Stops the device thread when the stream is dropped.
struct WasapiStream {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    _notifications: Option<Box<dyn std::any::Any>>,
}

impl Drop for WasapiStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// How samples are stored in the device buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Encoding {
    F32,
    /// 32-bit container, left-justified (24 or 32 valid bits).
    I32,
    /// Packed 24-bit.
    I24,
    I16,
}

struct Prepared {
    client: wasapi::AudioClient,
    render: wasapi::AudioRenderClient,
    clock: wasapi::AudioClock,
    event: wasapi::Handle,
    info: WasapiStreamInfo,
    name: String,
    channels: u16,
    encoding: Encoding,
    block_align: usize,
    rate: u32,
}

fn encoding_of(format: &WaveFormat) -> Option<Encoding> {
    match (format.get_subformat().ok()?, format.get_bitspersample()) {
        (SampleType::Float, 32) => Some(Encoding::F32),
        (SampleType::Int, 32) => Some(Encoding::I32),
        (SampleType::Int, 24) => Some(Encoding::I24),
        (SampleType::Int, 16) => Some(Encoding::I16),
        _ => None,
    }
}

/// Opens and initializes the client on the device thread.
fn prepare(id: Option<&str>, mode: WasapiMode, rate: u32) -> Result<Prepared, AudioError> {
    com();
    let enumerator = DeviceEnumerator::new().map_err(backend)?;
    let device = device(&enumerator, id)?;
    let name = device
        .get_friendlyname()
        .unwrap_or_else(|_| "WASAPI device".to_owned());
    let mut client = device.get_iaudioclient().map_err(backend)?;
    let mut raw = false;
    if mode == WasapiMode::SharedRaw {
        let properties = AudioClientProperties::new().set_option(StreamOption::Raw);
        raw = client.set_properties(properties).is_ok();
    }
    let (default_period, min_period) = client.get_device_period().map_err(backend)?;
    let (format, stream_mode, period_hns) = if mode == WasapiMode::Exclusive {
        let format = exclusive_format(&client, rate)?;
        let period = client
            .calculate_aligned_period_near(min_period, Some(128), &format)
            .unwrap_or(min_period);
        (
            format,
            StreamMode::EventsExclusive { period_hns: period },
            period,
        )
    } else {
        let format = client.get_mixformat().map_err(backend)?;
        (
            format,
            StreamMode::EventsShared {
                autoconvert: false,
                buffer_duration_hns: default_period,
            },
            default_period,
        )
    };
    if format.get_samplespersec() != rate {
        return Err(AudioError::SampleRate {
            plan_rate: sonarithm_core::SampleRate::new(rate)
                .unwrap_or(sonarithm_core::SampleRate::HZ_48000),
            device_rate: format.get_samplespersec(),
        });
    }
    let encoding = encoding_of(&format).ok_or_else(|| {
        AudioError::Backend("the device's sample format is not supported".to_owned())
    })?;
    client
        .initialize_client(&format, &Direction::Render, &stream_mode)
        .map_err(backend)?;
    let event = client.set_get_eventhandle().map_err(backend)?;
    let render = client.get_audiorenderclient().map_err(backend)?;
    let clock = client.get_audioclock().map_err(backend)?;
    let buffer_frames = client.get_buffer_size().map_err(backend)?;
    Ok(Prepared {
        client,
        render,
        clock,
        event,
        info: WasapiStreamInfo {
            mode,
            raw,
            buffer_frames,
            period_hns,
        },
        name,
        channels: format.get_nchannels(),
        encoding,
        block_align: format.get_blockalign() as usize,
        rate,
    })
}

/// The first exclusive format the device accepts at `rate`, in order of preference.
fn exclusive_format(client: &wasapi::AudioClient, rate: u32) -> Result<WaveFormat, AudioError> {
    let candidates = [
        (32, 32, SampleType::Float),
        (32, 24, SampleType::Int),
        (24, 24, SampleType::Int),
        (16, 16, SampleType::Int),
    ];
    candidates
        .iter()
        .find_map(|(store, valid, kind)| {
            let format = WaveFormat::new(*store, *valid, kind, rate as usize, 2, None);
            client.is_supported_exclusive_with_quirks(&format).ok()
        })
        .ok_or_else(|| AudioError::Backend("no exclusive-mode format at this rate".to_owned()))
}

/// The device loop. Any WASAPI error (such as an invalidated device) ends it and flags the
/// stream lost, so the control thread rebuilds it.
fn run(device: &Prepared, mut callback: Callback, status: &StreamStatus, stop: &AtomicBool) {
    // Real-time priority for the device thread (cpal does the same through its `realtime`
    // feature). Failure only costs robustness, so it is not an error.
    let _priority = audio_thread_priority::promote_current_thread_to_real_time(
        device.info.buffer_frames,
        device.rate,
    );
    let channels = usize::from(device.channels);
    let capacity = device.info.buffer_frames as usize;
    let mut samples = vec![0.0_f32; capacity.max(MAX_CHUNK) * channels];
    let mut bytes = vec![0_u8; capacity.max(MAX_CHUNK) * device.block_align];
    let frequency = device.clock.get_frequency().unwrap_or(1).max(1);
    let mut state = Feed {
        samples: &mut samples,
        bytes: &mut bytes,
        channels,
        capacity,
        frequency,
        written: 0,
        underrun_at: u64::MAX,
    };
    // Fill the buffer before starting, so playback starts with rendered audio (WASAPI's
    // "Rendering a Stream" procedure).
    if state.service(device, &mut callback, status).is_err()
        || device.client.start_stream().is_err()
    {
        status.lose_device();
        return;
    }
    while !stop.load(Ordering::Acquire) {
        if device.event.wait_for_event(1_000).is_err() {
            continue;
        }
        if state.service(device, &mut callback, status).is_err() {
            status.lose_device();
            break;
        }
    }
    let _ = device.client.stop_stream();
}

/// The device-side buffers and counters of the loop.
struct Feed<'a> {
    samples: &'a mut [f32],
    bytes: &'a mut [u8],
    channels: usize,
    capacity: usize,
    frequency: u64,
    written: u64,
    underrun_at: u64,
}

impl Feed<'_> {
    /// Renders and writes as many frames as the device has room for.
    fn service(
        &mut self,
        device: &Prepared,
        callback: &mut Callback,
        status: &StreamStatus,
    ) -> Result<(), wasapi::WasapiError> {
        let available = device.client.get_available_space_in_frames()?;
        let (position, qpc) = device.clock.get_position()?;
        // Frames the device has played, from its clock.
        let played = u64::try_from(
            u128::from(position) * u128::from(device.rate) / u128::from(self.frequency),
        )
        .unwrap_or(u64::MAX);
        // An underrun: the device has played everything written (exclusive), or the shared
        // buffer is empty when the engine asks for more.
        let dry = if device.info.mode == WasapiMode::Exclusive {
            played >= self.written
        } else {
            available >= device.info.buffer_frames
        };
        if self.written > 0 && dry && self.underrun_at != self.written {
            status.xrun();
            self.underrun_at = self.written;
        }
        let frames = (available as usize).min(self.capacity);
        if frames == 0 {
            return Ok(());
        }
        // Frames queued ahead of the device: in shared mode the buffer's padding; in exclusive
        // mode the device clock, because some drivers (HW-01's Realtek) signal several events
        // per hardware period and take more than one buffer (F-051).
        let queued = if device.info.mode == WasapiMode::Exclusive {
            self.written.saturating_sub(played)
        } else {
            u64::from(device.info.buffer_frames.saturating_sub(available))
        };
        let now = Duration::from_nanos(qpc.saturating_mul(100));
        let ahead = queued.saturating_mul(1_000_000_000) / u64::from(device.rate.max(1));
        let heard = now + Duration::from_nanos(ahead);
        let (Some(out), Some(raw)) = (
            self.samples.get_mut(..frames * self.channels),
            self.bytes.get_mut(..frames * device.block_align),
        ) else {
            return Ok(());
        };
        callback.fill(out, self.channels, Some(now), Some(heard));
        encode(out, raw, device.encoding);
        device.render.write_to_device(frames, raw, None)?;
        self.written += frames as u64;
        Ok(())
    }
}

/// Writes f32 samples into the device's encoding, little-endian, clamped.
fn encode(samples: &[f32], bytes: &mut [u8], encoding: Encoding) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "samples are clamped to [−1, 1] before scaling"
    )]
    match encoding {
        Encoding::F32 => {
            for (s, b) in samples.iter().zip(bytes.as_chunks_mut::<4>().0) {
                b.copy_from_slice(&s.to_le_bytes());
            }
        }
        Encoding::I32 => {
            for (s, b) in samples.iter().zip(bytes.as_chunks_mut::<4>().0) {
                let v = (f64::from(s.clamp(-1.0, 1.0)) * 2_147_483_647.0) as i32;
                b.copy_from_slice(&v.to_le_bytes());
            }
        }
        Encoding::I24 => {
            for (s, b) in samples.iter().zip(bytes.as_chunks_mut::<3>().0) {
                let v = (f64::from(s.clamp(-1.0, 1.0)) * 8_388_607.0) as i32;
                if let Some(low) = v.to_le_bytes().get(..3) {
                    b.copy_from_slice(low);
                }
            }
        }
        Encoding::I16 => {
            for (s, b) in samples.iter().zip(bytes.as_chunks_mut::<2>().0) {
                let v = (s.clamp(-1.0, 1.0) * 32_767.0) as i16;
                b.copy_from_slice(&v.to_le_bytes());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_encode_little_endian_and_clamped() {
        let samples = [0.5_f32, -2.0];
        let mut bytes = [0_u8; 8];
        encode(&samples, &mut bytes, Encoding::F32);
        assert_eq!(&bytes[..4], &0.5_f32.to_le_bytes());
        let mut bytes = [0_u8; 4];
        encode(&samples, &mut bytes, Encoding::I16);
        assert_eq!(bytes, [0xff, 0x3f, 0x01, 0x80]);
        let mut bytes = [0_u8; 6];
        encode(&samples, &mut bytes, Encoding::I24);
        assert_eq!(bytes, [0xff, 0xff, 0x3f, 0x01, 0x00, 0x80]);
    }
}
