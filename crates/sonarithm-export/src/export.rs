// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! One export: the audio file, its render manifest and its cue sidecar (06 §7, FR-100,
//! FR-101).
//!
//! Loudness normalization (sound-design §6.3, proposal; RFC-0001 `render.loudness`) is a
//! single static gain, never compression. The render is deterministic, so the first pass
//! renders and measures, and the second renders again with the gain and writes. When the
//! target loudness would push the true peak over its limit, the export uses the highest
//! gain that stays within it, and the manifest records the result.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;

use sha2::{Digest, Sha256};
use sonarithm_core::{Db, Frames, Score};
use sonarithm_dsp::meter::{LoudnessMeter, MeterError};
use sonarithm_render::{OfflineRender, PlanConfig, PlanError, RenderPlan};

use crate::audio::{AudioFormat, ExportError, write_audio_observed};
use crate::cues::{cues, webvtt};
use crate::manifest::{
    CueInfo, DataInfo, DataSource, MANIFEST_VERSION, OutputInfo, Platform, RenderManifest,
    RenderSettings, ScoreInfo, Software, SpecInfo, file_name, mode_name, sha256_text,
};

/// How the export level is set.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Normalization {
    /// Keep the rendered level (research exports, sound-design §6.3).
    Preserve,
    /// One static gain to reach an integrated loudness, within a true-peak limit.
    Target {
        /// Integrated loudness to aim for, in LUFS.
        lufs: f64,
        /// The true-peak limit, in dBTP.
        true_peak: f64,
    },
}

impl Normalization {
    /// RFC-0001's `render.loudness` defaults: −16 LUFS integrated, −1 dBTP (proposed export
    /// defaults, sound-design §6.3).
    pub const SPEC_DEFAULT: Self = Self::Target {
        lufs: -16.0,
        true_peak: -1.0,
    };
}

/// What to export and how.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ExportOptions {
    /// The audio format.
    pub format: AudioFormat,
    /// Plan settings: sample rate and loudness compensation.
    pub plan: PlanConfig,
    /// The offline block size in frames.
    pub block_size: usize,
    /// The level.
    pub normalization: Normalization,
    /// Whether to write the WebVTT cue sidecar.
    pub cues: bool,
}

impl ExportOptions {
    /// `format` at 48 kHz, 256-frame blocks, spec-default normalization, with cues.
    #[must_use]
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            plan: PlanConfig::new(sonarithm_core::SampleRate::HZ_48000),
            block_size: sonarithm_render::offline::DEFAULT_BLOCK,
            normalization: Normalization::SPEC_DEFAULT,
            cues: true,
        }
    }
}

/// Provenance the Score does not carry, supplied by the caller (A07 CLI, desktop app).
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ManifestInputs {
    /// The build's git commit.
    pub git: Option<String>,
    /// The canonical spec with defaults applied, to embed in the manifest.
    pub spec: Option<serde_json::Value>,
    /// Where the data came from.
    pub data_source: Option<DataSource>,
    /// The share of values in each provenance class (03 §4).
    pub provenance: BTreeMap<String, f64>,
    /// The experimental switches that were on (RFC-0006), sorted.
    pub experimental: Vec<String>,
}

/// Why an export failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExportFailure {
    /// The Score could not be planned.
    #[error("the Score cannot be rendered")]
    Plan(#[from] PlanError),
    /// The Score has no canonical form, so it has no hash.
    #[error("the Score has no canonical form")]
    Canonical(#[from] sonarithm_core::score::canonical::CanonicalError),
    /// Measuring loudness failed.
    #[error("measuring the export's loudness failed")]
    Meter(#[from] MeterError),
    /// Writing audio failed.
    #[error(transparent)]
    Audio(#[from] ExportError),
    /// A file could not be written or read back.
    #[error("writing the export files failed")]
    Io(#[from] std::io::Error),
    /// The manifest could not be serialized.
    #[error("writing the render manifest failed")]
    Manifest(#[from] serde_json::Error),
}

/// What an export wrote.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Exported {
    /// The manifest, as written next to the audio.
    pub manifest: RenderManifest,
    /// Diagnostics from planning, such as unknown instruments.
    pub diagnostics: sonarithm_core::Diagnostics,
}

/// Renders `score` into `directory` as `<stem>.<ext>`, `<stem>.render.json` and, if asked,
/// `<stem>.cues.vtt`.
///
/// # Errors
///
/// [`ExportFailure`] if the Score cannot be planned or hashed, or a file cannot be written.
pub fn export(
    score: &Score,
    options: &ExportOptions,
    inputs: &ManifestInputs,
    directory: &Path,
    stem: &str,
) -> Result<Exported, ExportFailure> {
    let stem = file_name(stem);
    let planned = RenderPlan::build(score, &options.plan)?;
    let plan = planned.plan;
    let rate = plan.sample_rate;
    let gain = match options.normalization {
        Normalization::Preserve => Db::ZERO,
        Normalization::Target { lufs, true_peak } => {
            let measured = measure(&plan, options.block_size)?;
            normalization_gain(measured, lufs, true_peak)
        }
    };
    #[expect(clippy::cast_possible_truncation, reason = "gains are f32")]
    let linear = gain.to_gain() as f32;

    let audio_name = format!("{stem}.{}", options.format.extension());
    let audio_path = directory.join(&audio_name);
    let mut render = OfflineRender::new(plan, options.block_size);
    // The manifest's loudness is measured on the samples as written, before quantization.
    let mut meter = LoudnessMeter::new(rate)?;
    let mut metered = Ok(());
    let report = write_audio_observed(
        &mut render,
        options.format,
        linear,
        BufWriter::new(File::create(&audio_path)?),
        &mut |samples| {
            if metered.is_ok() {
                metered = meter.add(samples);
            }
        },
    )?;
    metered?;
    let loudness = meter.loudness();
    let limited_frames = render.limited_frames();
    let file_hash = hash_file(&audio_path)?;

    let cue_info = if options.cues {
        let list = cues(score);
        let name = format!("{stem}.cues.vtt");
        std::fs::write(directory.join(&name), webvtt(&list))?;
        Some(CueInfo {
            file: name,
            count: u32::try_from(list.len()).unwrap_or(u32::MAX),
        })
    } else {
        None
    };

    let manifest = manifest(
        score,
        options,
        inputs,
        Written {
            audio: audio_name,
            hash: file_hash,
            frames: report.frames,
            bits_per_sample: report.bits_per_sample,
            channels: report.channels,
            loudness,
            gain,
            limited_frames,
            cues: cue_info,
        },
    )?;
    std::fs::write(
        directory.join(format!("{stem}.render.json")),
        manifest.to_json()?,
    )?;
    Ok(Exported {
        manifest,
        diagnostics: planned.diagnostics,
    })
}

/// What the export wrote, for the manifest.
struct Written {
    audio: String,
    hash: String,
    frames: u64,
    bits_per_sample: u16,
    channels: u16,
    loudness: sonarithm_dsp::Loudness,
    gain: Db,
    limited_frames: u64,
    cues: Option<CueInfo>,
}

fn manifest(
    score: &Score,
    options: &ExportOptions,
    inputs: &ManifestInputs,
    written: Written,
) -> Result<RenderManifest, ExportFailure> {
    let header = &score.header;
    let rate = options.plan.sample_rate;
    Ok(RenderManifest {
        manifest_version: MANIFEST_VERSION,
        sonarithm: Software {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            git: inputs.git.clone(),
            engines: header
                .engines
                .iter()
                .map(|(id, version)| (id.to_string(), version.0.clone()))
                .collect(),
        },
        score: ScoreInfo {
            ir_version: header.ir_version,
            hash: sha256_text(&score.canonical_hash()?),
        },
        spec: SpecInfo {
            version: header.spec_version.to_string(),
            hash: sha256_text(&header.spec_hash),
            embedded: inputs.spec.clone(),
            experimental: inputs.experimental.clone(),
        },
        data: DataInfo {
            fingerprint: sha256_text(&header.data_fingerprint),
            source: inputs.data_source.clone(),
            provenance: inputs.provenance.clone(),
        },
        render: RenderSettings {
            sample_rate: rate.get(),
            block_size: u32::try_from(options.block_size).unwrap_or(u32::MAX),
            seed: header.seed,
            mode: mode_name(header.mode).to_owned(),
            duration_s: Frames(written.frames).to_seconds(rate).0,
            loudness_compensation_phon: options.plan.loudness_compensation,
        },
        output: OutputInfo {
            file: written.audio,
            format: options.format.extension().to_owned(),
            bits_per_sample: written.bits_per_sample,
            channels: written.channels,
            frames: written.frames,
            sha256: written.hash,
            integrated_lufs: written.loudness.integrated.map(|db| round(db.0, 100.0)),
            true_peak_dbtp: round(written.loudness.true_peak.0, 100.0),
            gain_db: round(written.gain.0, 100.0),
            limited_frames: written.limited_frames,
        },
        cues: written.cues,
        platform: Platform::current(),
    })
}

/// The integrated loudness and true peak of the unnormalized render.
fn measure(plan: &RenderPlan, block_size: usize) -> Result<(Option<Db>, Db), MeterError> {
    let mut meter = LoudnessMeter::new(plan.sample_rate)?;
    let mut render = OfflineRender::new(plan.clone(), block_size);
    let mut block = vec![0.0; 2 * render.block_size()];
    loop {
        let samples = render.next_block(&mut block);
        if samples.is_empty() {
            break;
        }
        meter.add(samples)?;
    }
    let loudness = meter.loudness();
    Ok((loudness.integrated, loudness.true_peak))
}

/// The static gain that brings `measured` loudness to `lufs` without the true peak passing
/// `true_peak`. Silence gets 0 dB.
#[must_use]
pub fn normalization_gain(measured: (Option<Db>, Db), lufs: f64, true_peak: f64) -> Db {
    let (integrated, peak) = measured;
    let Some(integrated) = integrated else {
        return Db::ZERO;
    };
    let wanted = lufs - integrated.0;
    let headroom = if peak.0.is_finite() {
        true_peak - peak.0
    } else {
        wanted
    };
    Db(wanted.min(headroom))
}

/// Rounds for the manifest: 0.01 dB is far below the golden tolerances.
fn round(x: f64, scale: f64) -> f64 {
    (x * scale).round_ties_even() / scale
}

/// `sha256:` and the hex digest of a file's bytes, read in 64 KiB pieces.
fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 16];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let mut text = String::from("sha256:");
    for byte in hasher.finalize() {
        let _ = write!(text, "{byte:02x}");
    }
    Ok(text)
}
