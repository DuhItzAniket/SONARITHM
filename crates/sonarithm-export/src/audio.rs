// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Audio file writers (06 §7): WAV through hound and FLAC through flacenc, fed block by
//! block from the offline renderer, so no export holds the whole render in memory as PCM.
//!
//! Samples are quantized to 24 bits by scaling with `2^23 − 1`, rounding half to even and
//! clamping; there is no dither (24-bit quantization noise is near −144 dBFS). A 32-bit
//! float WAV stores the render's samples unchanged.

use std::io::{Seek, Write};

use flacenc::bitsink::ByteSink;
use flacenc::component::BitRepr;
use flacenc::error::{SourceError, Verify};
use flacenc::source::{Fill, Source};
use sonarithm_core::SampleRate;
use sonarithm_render::OfflineRender;

/// Output channels: stereo (06 §3 spatializer output).
pub const CHANNELS: u16 = 2;
/// Bits per sample of integer output.
pub const PCM_BITS: u16 = 24;
const PCM_SCALE: f64 = 8_388_607.0;

/// A WAV sample encoding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WavEncoding {
    /// 24-bit integer PCM, the default (06 §7).
    #[default]
    Pcm24,
    /// 32-bit IEEE float: the render's samples bit for bit.
    Float32,
}

/// An audio file format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AudioFormat {
    /// RIFF WAVE.
    Wav(WavEncoding),
    /// FLAC, 24-bit.
    Flac,
}

impl AudioFormat {
    /// The usual file extension, without the dot.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Wav(_) => "wav",
            Self::Flac => "flac",
        }
    }

    /// Bits per sample in the file.
    #[must_use]
    pub fn bits_per_sample(self) -> u16 {
        match self {
            Self::Wav(WavEncoding::Float32) => 32,
            _ => PCM_BITS,
        }
    }
}

/// Why an audio file could not be written.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExportError {
    /// Writing to the output failed.
    #[error("writing the audio output failed")]
    Io(#[from] std::io::Error),
    /// The WAV writer failed.
    #[error("WAV encoding failed")]
    Wav(#[from] hound::Error),
    /// The FLAC encoder failed. Its error types are not `Send`, so only the stage is kept.
    #[error("FLAC encoding failed at the {0} stage")]
    Flac(FlacStage),
}

/// Where FLAC encoding failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FlacStage {
    /// Checking the encoder configuration.
    Config,
    /// Encoding frames.
    Encode,
    /// Serializing the stream.
    Write,
}

impl std::fmt::Display for FlacStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Config => "configuration",
            Self::Encode => "encoding",
            Self::Write => "serialization",
        })
    }
}

/// What was written.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct AudioReport {
    /// Frames per channel.
    pub frames: u64,
    /// The sample rate.
    pub sample_rate: SampleRate,
    /// Channels.
    pub channels: u16,
    /// Bits per sample in the file.
    pub bits_per_sample: u16,
    /// The largest absolute sample before quantization.
    pub peak: f32,
    /// Samples outside [−1, 1] that were clamped (integer formats only).
    pub clipped: u64,
}

/// A sample quantized to 24 bits; NaN becomes 0.
#[must_use]
pub fn to_pcm24(sample: f32) -> i32 {
    if sample.is_nan() {
        return 0;
    }
    let scaled = (f64::from(sample) * PCM_SCALE).round_ties_even();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped to the 24-bit range first"
    )]
    let quantized = scaled.clamp(-PCM_SCALE - 1.0, PCM_SCALE) as i32;
    quantized
}

#[derive(Default)]
struct Meter {
    peak: f32,
    clipped: u64,
}

impl Meter {
    fn sample(&mut self, sample: f32) {
        self.peak = self.peak.max(sample.abs());
        self.clipped += u64::from(sample.abs() > 1.0);
    }
}

/// Renders `render` to the end and writes it to `writer` in `format`, scaling every
/// sample by `gain` first (1 for none).
///
/// # Errors
///
/// [`ExportError`] if encoding or writing fails.
pub fn write_audio<W: Write + Seek>(
    render: &mut OfflineRender,
    format: AudioFormat,
    gain: f32,
    writer: W,
) -> Result<AudioReport, ExportError> {
    let sample_rate = render.sample_rate();
    let frames = render.remaining();
    let mut meter = Meter::default();
    match format {
        AudioFormat::Wav(encoding) => write_wav(render, encoding, gain, writer, &mut meter)?,
        AudioFormat::Flac => write_flac(render, gain, writer, &mut meter)?,
    }
    Ok(AudioReport {
        frames,
        sample_rate,
        channels: CHANNELS,
        bits_per_sample: format.bits_per_sample(),
        peak: meter.peak,
        clipped: if format == AudioFormat::Wav(WavEncoding::Float32) {
            0
        } else {
            meter.clipped
        },
    })
}

fn write_wav<W: Write + Seek>(
    render: &mut OfflineRender,
    encoding: WavEncoding,
    gain: f32,
    writer: W,
    meter: &mut Meter,
) -> Result<(), ExportError> {
    let float = encoding == WavEncoding::Float32;
    let spec = hound::WavSpec {
        channels: CHANNELS,
        sample_rate: render.sample_rate().get(),
        bits_per_sample: if float { 32 } else { PCM_BITS },
        sample_format: if float {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    };
    let mut wav = hound::WavWriter::new(writer, spec)?;
    let mut block = vec![0.0; 2 * render.block_size()];
    loop {
        let samples = render.next_block(&mut block);
        if samples.is_empty() {
            break;
        }
        for &sample in samples {
            let sample = sample * gain;
            meter.sample(sample);
            if float {
                wav.write_sample(sample)?;
            } else {
                wav.write_sample(to_pcm24(sample))?;
            }
        }
    }
    wav.finalize()?;
    Ok(())
}

/// Feeds the encoder from the renderer. A render block can cross an encoder block
/// boundary; the samples past it wait in `carry` for the next encoder block.
struct RenderSource<'a> {
    render: &'a mut OfflineRender,
    gain: f32,
    block: Vec<f32>,
    pcm: Vec<i32>,
    carry: Vec<i32>,
    meter: &'a mut Meter,
    total: usize,
}

impl Source for RenderSource<'_> {
    fn channels(&self) -> usize {
        usize::from(CHANNELS)
    }

    fn bits_per_sample(&self) -> usize {
        usize::from(PCM_BITS)
    }

    fn sample_rate(&self) -> usize {
        self.render.sample_rate().get() as usize
    }

    fn read_samples<F: Fill>(
        &mut self,
        block_size: usize,
        dest: &mut F,
    ) -> Result<usize, SourceError> {
        let wanted = 2 * block_size;
        self.pcm.clear();
        self.pcm.append(&mut self.carry);
        while self.pcm.len() < wanted {
            let samples = self.render.next_block(&mut self.block);
            if samples.is_empty() {
                break;
            }
            for &sample in samples {
                let sample = sample * self.gain;
                self.meter.sample(sample);
                self.pcm.push(to_pcm24(sample));
            }
        }
        if self.pcm.len() > wanted {
            self.carry.extend_from_slice(&self.pcm[wanted..]);
            self.pcm.truncate(wanted);
        }
        dest.fill_interleaved(&self.pcm)?;
        Ok(self.pcm.len() / 2)
    }

    fn len_hint(&self) -> Option<usize> {
        Some(self.total)
    }
}

fn write_flac<W: Write>(
    render: &mut OfflineRender,
    gain: f32,
    mut writer: W,
    meter: &mut Meter,
) -> Result<(), ExportError> {
    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|_| ExportError::Flac(FlacStage::Config))?;
    let block_size = config.block_size;
    let source = RenderSource {
        total: usize::try_from(render.remaining()).unwrap_or(usize::MAX),
        block: vec![0.0; 2 * render.block_size()],
        pcm: Vec::with_capacity(2 * block_size),
        carry: Vec::new(),
        render,
        gain,
        meter,
    };
    let stream = flacenc::encode_with_fixed_block_size(&config, source, block_size)
        .map_err(|_| ExportError::Flac(FlacStage::Encode))?;
    let mut sink = ByteSink::new();
    stream
        .write(&mut sink)
        .map_err(|_| ExportError::Flac(FlacStage::Write))?;
    writer.write_all(sink.as_slice())?;
    writer.flush()?;
    Ok(())
}
