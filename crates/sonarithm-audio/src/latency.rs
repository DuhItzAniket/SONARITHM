// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Key-to-onset latency measurement on Windows (testing-strategy §14, ADR-0016).
//!
//! Each trial sends a cue through the engine's command path, the way a key press will
//! (`Command::Cue`), and records the performance-counter time of the send. A capture stream
//! listens either to the output's own loopback (software path only, the self-test) or to a
//! physical input wired to the output by cable (sign-off). The latency of a trial is the
//! capture timestamp of the cue's onset minus the send time. The calibration offset is the
//! median of the onset minus the time the engine predicted for it (the host time of the
//! block that started the cue), which is what playhead anchors must add (06 §9).
//!
//! Trials are spaced 300–600 ms apart from a seeded generator, so triggers do not lock to
//! block boundaries. Cues are 50 ms square tones at −30 dBFS or lower (hearing safety: the
//! default RAW mode bypasses OEM processing that would add gain, F-032).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use sonarithm_core::{Db, Hz, Mode, Pan, SampleRate, Score, ScoreHeader, Seconds, SpecVersion};
use sonarithm_render::graph::{Command, Cue};
use sonarithm_render::{Instrument, PlanConfig};
use wasapi::{DeviceEnumerator, Direction, SampleType, StreamMode};

use crate::backend::{AudioBackend, AudioError, StreamConfig};
use crate::player::{Player, PlayerError};
use crate::wasapi_backend::{WasapiBackend, WasapiMode, WasapiStreamInfo};

/// The loudest cue the tool plays when the RAW option is in effect: −30 dBFS.
pub const MAX_LEVEL: Db = Db(-30.0);

/// The loudest cue without RAW: −50 dBFS. OEM enhancement processing can add about 20 dB
/// after SONARITHM's limiter (F-032), which keeps the result near −30 dBFS.
pub const MAX_LEVEL_PROCESSED: Db = Db(-50.0);

/// What the capture stream listens to.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CaptureSource {
    /// The output device's own loopback: the software path only (self-test).
    Loopback,
    /// A capture device by identifier, wired to the output with a cable (sign-off).
    Device(String),
}

/// One measurement campaign.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct LatencyRun {
    /// How the output stream shares the device.
    pub mode: WasapiMode,
    /// What to capture.
    pub capture: CaptureSource,
    /// Number of cues; testing-strategy §14 asks for at least 200.
    pub trials: usize,
    /// Cue level, limited to [`MAX_LEVEL`] with RAW and to [`MAX_LEVEL_PROCESSED`]
    /// without it.
    pub level: Db,
    /// Seed for the trial spacing.
    pub seed: u64,
}

impl LatencyRun {
    /// 200 trials in RAW shared mode through the loopback, at −30 dBFS.
    #[must_use]
    pub fn new(capture: CaptureSource) -> Self {
        Self {
            mode: WasapiMode::SharedRaw,
            capture,
            trials: 200,
            level: MAX_LEVEL,
            seed: 1729,
        }
    }
}

/// The results, in milliseconds.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct LatencyReport {
    /// The output stream as opened.
    pub stream: WasapiStreamInfo,
    /// The output device's name.
    pub device: String,
    /// Cues sent.
    pub trials: usize,
    /// Cues whose onset was found.
    pub detected: usize,
    /// Send-to-onset percentiles and maximum.
    pub p50: f64,
    /// 95th percentile.
    pub p95: f64,
    /// 99th percentile.
    pub p99: f64,
    /// Maximum.
    pub max: f64,
    /// Median of onset minus the engine's predicted time: the calibration offset.
    pub offset: f64,
    /// Underruns the output reported during the run.
    pub xruns: u64,
}

/// Why a measurement failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LatencyError {
    /// The output stream could not be opened.
    #[error(transparent)]
    Player(#[from] PlayerError),
    /// The capture stream failed.
    #[error(transparent)]
    Audio(#[from] AudioError),
    /// No onset was found for any cue: check the cable, the input level and the device.
    #[error("no cue onset was captured")]
    NothingCaptured,
}

/// Runs `run` and reports the latencies.
///
/// # Errors
///
/// [`LatencyError`] if a stream cannot be opened or nothing is captured.
pub fn measure(run: &LatencyRun) -> Result<LatencyReport, LatencyError> {
    let backend = WasapiBackend::new(run.mode);
    let rate = backend.sample_rate(None)?;
    let rate = SampleRate::new(rate).ok_or(PlayerError::NoSampleRate)?;
    let mut header = ScoreHeader::new(SpecVersion::V0_1, Mode::Analysis, 0);
    header.duration = Seconds(1.0);
    let mut player = Player::open(
        backend,
        Score::new(header),
        PlanConfig::new(rate),
        StreamConfig::default(),
        256,
    )?;
    // Hearing safety: without RAW, OEM processing may add gain after the limiter.
    let raw = player.backend_mut().last_stream().is_some_and(|s| s.raw);
    let ceiling = if raw { MAX_LEVEL } else { MAX_LEVEL_PROCESSED };
    let level = Db(run.level.0.min(ceiling.0));
    let stop = Arc::new(AtomicBool::new(false));
    let (onsets_tx, onsets) = mpsc::channel();
    let (ready_tx, ready) = mpsc::channel();
    let capture_stop = Arc::clone(&stop);
    let source = run.capture.clone();
    // Onsets are above a quarter of the cue's peak (centre pan: −3 dB).
    let threshold = 0.25 * 0.707 * level.to_gain();
    let capture = std::thread::spawn(move || {
        capture(&source, threshold, &capture_stop, &onsets_tx, &ready_tx);
    });
    ready
        .recv()
        .map_err(|_| AudioError::Backend("the capture thread ended".to_owned()))??;
    // Let the start-up ramp (500 ms) finish before the first cue.
    std::thread::sleep(Duration::from_millis(800));
    let mut spacing = run.seed | 1;
    let mut trials = Vec::with_capacity(run.trials);
    for _ in 0..run.trials {
        spacing ^= spacing << 13;
        spacing ^= spacing >> 7;
        spacing ^= spacing << 17;
        std::thread::sleep(Duration::from_millis(300 + spacing % 300));
        let blocks = player.state().blocks;
        let sent = qpc::now();
        player.send(Command::Cue(Cue {
            frequency: Hz(1_000.0),
            gain: level,
            pan: Pan::CENTRE,
            instrument: Instrument::Square,
            duration: Seconds(0.05),
        }));
        let predicted = predicted_onset(&mut player, blocks);
        trials.push((sent, predicted));
        let _ = player.tick();
    }
    std::thread::sleep(Duration::from_millis(500));
    stop.store(true, Ordering::Release);
    let _ = capture.join();
    let captured: Vec<Duration> = onsets.try_iter().collect();
    let stream = player.backend_mut().last_stream();
    let device = player
        .stream()
        .map(|s| s.device().to_owned())
        .unwrap_or_default();
    let xruns = player.stream().map_or(0, |s| s.status().xruns());
    report(&trials, &captured, stream, device, xruns)
}

/// The host time the engine published for the first block after `blocks`: the block that
/// applied the cue command, whose first frame is the cue's onset.
fn predicted_onset(player: &mut Player<WasapiBackend>, blocks: u64) -> Option<Duration> {
    for _ in 0..200 {
        let state = player.state();
        if state.blocks > blocks {
            return state.host_time;
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    None
}

#[expect(
    clippy::cast_precision_loss,
    reason = "durations of seconds, in milliseconds"
)]
fn millis(d: Duration) -> f64 {
    d.as_nanos() as f64 / 1e6
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "an index within the slice"
    )]
    let index = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted.get(index).copied().unwrap_or(f64::NAN)
}

fn report(
    trials: &[(Duration, Option<Duration>)],
    onsets: &[Duration],
    stream: Option<WasapiStreamInfo>,
    device: String,
    xruns: u64,
) -> Result<LatencyReport, LatencyError> {
    let mut latencies = Vec::new();
    let mut offsets = Vec::new();
    for &(sent, predicted) in trials {
        // The first onset within 500 ms of the send belongs to this trial.
        let Some(onset) = onsets
            .iter()
            .copied()
            .find(|onset| *onset >= sent && *onset < sent + Duration::from_millis(500))
        else {
            continue;
        };
        latencies.push(millis(onset.saturating_sub(sent)));
        if let Some(predicted) = predicted {
            offsets.push(millis(onset) - millis(predicted));
        }
    }
    if latencies.is_empty() {
        return Err(LatencyError::NothingCaptured);
    }
    latencies.sort_by(f64::total_cmp);
    offsets.sort_by(f64::total_cmp);
    Ok(LatencyReport {
        stream: stream.ok_or(LatencyError::NothingCaptured)?,
        device,
        trials: trials.len(),
        detected: latencies.len(),
        p50: percentile(&latencies, 0.5),
        p95: percentile(&latencies, 0.95),
        p99: percentile(&latencies, 0.99),
        max: latencies.last().copied().unwrap_or(f64::NAN),
        offset: percentile(&offsets, 0.5),
        xruns,
    })
}

/// The capture thread: reports each onset's performance-counter time.
fn capture(
    source: &CaptureSource,
    threshold: f64,
    stop: &AtomicBool,
    onsets: &mpsc::Sender<Duration>,
    ready: &mpsc::Sender<Result<(), AudioError>>,
) {
    let opened = open_capture(source);
    let (client, capture, rate, channels, block_align, encoding) = match opened {
        Ok(parts) => {
            let _ = ready.send(Ok(()));
            parts
        }
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    let mut bytes = vec![0_u8; 2 * rate as usize * block_align];
    // Re-arm after 50 ms below the threshold.
    let quiet_needed = u64::from(rate / 20);
    let (mut armed, mut quiet) = (false, 0_u64);
    while !stop.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(2));
        while let Ok(Some(frames)) = capture.get_next_packet_size() {
            if frames == 0 {
                break;
            }
            let Ok((read, info)) = capture.read_from_device(&mut bytes) else {
                break;
            };
            for (index, frame) in bytes
                .chunks_exact(block_align)
                .take(read as usize)
                .enumerate()
            {
                let peak = frame_peak(frame, channels, encoding);
                if f64::from(peak) > threshold {
                    if armed {
                        let offset = (index as u64) * 1_000_000_000 / u64::from(rate.max(1));
                        let at = Duration::from_nanos(info.timestamp.saturating_mul(100) + offset);
                        let _ = onsets.send(at);
                        armed = false;
                    }
                    quiet = 0;
                } else {
                    quiet += 1;
                    armed |= quiet >= quiet_needed;
                }
            }
        }
    }
    let _ = client.stop_stream();
}

/// Sample encodings the capture side reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Encoding {
    F32,
    I32,
    I16,
}

type Opened = (
    wasapi::AudioClient,
    wasapi::AudioCaptureClient,
    u32,
    usize,
    usize,
    Encoding,
);

fn open_capture(source: &CaptureSource) -> Result<Opened, AudioError> {
    let backend = |e: wasapi::WasapiError| AudioError::Backend(e.to_string());
    let _ = wasapi::initialize_mta();
    let enumerator = DeviceEnumerator::new().map_err(backend)?;
    let device = match source {
        CaptureSource::Loopback => enumerator.get_default_device(&Direction::Render),
        CaptureSource::Device(id) => enumerator.get_device(id),
    }
    .map_err(|_| AudioError::DeviceNotFound)?;
    let mut client = device.get_iaudioclient().map_err(backend)?;
    let format = client.get_mixformat().map_err(backend)?;
    let encoding = match (
        format.get_subformat().map_err(backend)?,
        format.get_bitspersample(),
    ) {
        (SampleType::Float, 32) => Encoding::F32,
        (SampleType::Int, 32) => Encoding::I32,
        (SampleType::Int, 16) => Encoding::I16,
        _ => return Err(AudioError::Backend("unsupported capture format".to_owned())),
    };
    // Loopback is a capture stream on a render device.
    client
        .initialize_client(
            &format,
            &Direction::Capture,
            &StreamMode::PollingShared {
                autoconvert: true,
                buffer_duration_hns: 1_000_000,
            },
        )
        .map_err(backend)?;
    let capture = client.get_audiocaptureclient().map_err(backend)?;
    client.start_stream().map_err(backend)?;
    Ok((
        client,
        capture,
        format.get_samplespersec(),
        usize::from(format.get_nchannels()),
        format.get_blockalign() as usize,
        encoding,
    ))
}

/// The largest absolute sample of one frame.
fn frame_peak(frame: &[u8], channels: usize, encoding: Encoding) -> f32 {
    let width = frame.len() / channels.max(1);
    frame
        .chunks_exact(width.max(1))
        .map(|sample| match (encoding, sample) {
            (Encoding::F32, &[a, b, c, d]) => f32::from_le_bytes([a, b, c, d]).abs(),
            #[expect(clippy::cast_precision_loss, reason = "a level, not a value")]
            (Encoding::I32, &[a, b, c, d]) => {
                (i32::from_le_bytes([a, b, c, d]) as f32 / 2_147_483_648.0).abs()
            }
            (Encoding::I16, &[a, b]) => (f32::from(i16::from_le_bytes([a, b])) / 32_768.0).abs(),
            _ => 0.0,
        })
        .fold(0.0, f32::max)
}

/// The Windows performance counter, the clock WASAPI timestamps use.
mod qpc {
    #![allow(
        unsafe_code,
        reason = "platform FFI on the unsafe register (coding-standards §5)"
    )]

    use std::time::Duration;

    use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

    /// The current performance-counter time.
    pub(super) fn now() -> Duration {
        let (mut counter, mut frequency) = (0_i64, 0_i64);
        // SAFETY: both functions only write the integer they are given, which lives for the
        // call; they cannot fail on Windows XP or later.
        unsafe {
            let _ = QueryPerformanceCounter(&raw mut counter);
            let _ = QueryPerformanceFrequency(&raw mut frequency);
        }
        let (counter, frequency) = (
            u128::try_from(counter).unwrap_or(0),
            u128::try_from(frequency).unwrap_or(1).max(1),
        );
        Duration::from_nanos(u64::try_from(counter * 1_000_000_000 / frequency).unwrap_or(u64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_match_onsets_to_trials() {
        let ms = Duration::from_millis;
        let trials = [
            (ms(1_000), Some(ms(1_010))),
            (ms(2_000), Some(ms(2_012))),
            (ms(3_000), None),
        ];
        let onsets = [ms(1_080), ms(2_090), ms(9_000)];
        let stream = WasapiStreamInfo {
            mode: WasapiMode::SharedRaw,
            raw: true,
            buffer_frames: 480,
            period_hns: 100_000,
        };
        let report = report(&trials, &onsets, Some(stream), "dev".to_owned(), 0).unwrap();
        assert_eq!((report.trials, report.detected), (3, 2));
        assert!((report.p50 - 80.0).abs() < 1e-9 || (report.p50 - 90.0).abs() < 1e-9);
        assert!((report.max - 90.0).abs() < 1e-9);
        assert!((report.offset - 70.0).abs() < 1e-9 || (report.offset - 78.0).abs() < 1e-9);
        assert!(matches!(report_empty(), Err(LatencyError::NothingCaptured)));
    }

    fn report_empty() -> Result<LatencyReport, LatencyError> {
        report(&[(Duration::ZERO, None)], &[], None, String::new(), 0)
    }

    #[test]
    fn frame_peaks_read_every_encoding() {
        let f = [0.25_f32.to_le_bytes(), (-0.5_f32).to_le_bytes()].concat();
        assert!((frame_peak(&f, 2, Encoding::F32) - 0.5).abs() < 1e-6);
        let i = [i16::MIN.to_le_bytes(), 0_i16.to_le_bytes()].concat();
        assert!((frame_peak(&i, 2, Encoding::I16) - 1.0).abs() < 1e-6);
    }
}
