// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Offline rendering: a compiled spec to an audio file, its render manifest and its cue
//! sidecar (06 §7, ADR-0013), or to WAV bytes in memory for notebook playback.
//!
//! Both paths use the export crate's render and level: the spec's `output` section sets the
//! sample rate, block size and loudness target (−16 LUFS within −1 dBTP by default), and
//! the same Score, spec and seed always give the same bytes.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sonarithm_core::{Diagnostics, SampleRate};
use sonarithm_dsp::meter::LoudnessMeter;
use sonarithm_export::export::normalization_gain;
pub use sonarithm_export::{AudioFormat, Normalization, RenderManifest, WavEncoding};
use sonarithm_export::{DataSource, ExportOptions, ManifestInputs, write_audio};
use sonarithm_render::{OfflineRender, PlanConfig, RenderPlan};

use crate::api::Sonarithm;
use crate::compiled::Compiled;
use crate::error::Error;

/// How to render. Every unset option comes from the spec's `output` section.
///
/// ```
/// use sonarithm::{AudioFormat, RenderOptions};
///
/// let flac = RenderOptions::new().with_format(AudioFormat::Flac).with_sample_rate(44_100);
/// let raw = RenderOptions::new().with_normalization(sonarithm::Normalization::Preserve);
/// # let _ = (flac, raw);
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RenderOptions {
    /// The audio format; by default from the file extension (`.wav`, `.flac`).
    pub format: Option<AudioFormat>,
    /// The sample rate in hertz; by default the spec's `output.sampleRate` (48 kHz).
    pub sample_rate: Option<u32>,
    /// The level; by default the spec's `output.loudness` target.
    pub normalization: Option<Normalization>,
    /// Whether to write the `WebVTT` cue sidecar (default yes).
    pub cues: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            format: None,
            sample_rate: None,
            normalization: None,
            cues: true,
        }
    }
}

impl RenderOptions {
    /// The defaults: everything from the spec, with cues.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same options with a format.
    #[must_use]
    pub fn with_format(mut self, format: AudioFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// The same options with a sample rate in hertz.
    #[must_use]
    pub fn with_sample_rate(mut self, hz: u32) -> Self {
        self.sample_rate = Some(hz);
        self
    }

    /// The same options with a level policy.
    #[must_use]
    pub fn with_normalization(mut self, normalization: Normalization) -> Self {
        self.normalization = Some(normalization);
        self
    }

    /// The same options without the cue sidecar.
    #[must_use]
    pub fn without_cues(mut self) -> Self {
        self.cues = false;
        self
    }
}

/// What [`Sonarithm::render_to_file`] wrote.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Rendered {
    /// The audio file.
    pub audio: PathBuf,
    /// The render manifest, `<stem>.render.json`.
    pub manifest_path: PathBuf,
    /// The cue sidecar, `<stem>.cues.vtt`, when written.
    pub cues: Option<PathBuf>,
    /// The manifest as written.
    pub manifest: RenderManifest,
    /// Render diagnostics, such as an unknown instrument replaced by the default.
    pub diagnostics: Diagnostics,
}

/// The settings a render uses: the options over the spec's `output` section.
struct Settings {
    options: ExportOptions,
}

fn settings(
    compiled: &Compiled,
    options: &RenderOptions,
    format: AudioFormat,
) -> Result<Settings, Error> {
    let output = sonarithm_spec::parse(compiled.canonical_spec())
        .ok()
        .and_then(|spec| spec.output);
    let hz = options
        .sample_rate
        .or_else(|| output.as_ref().and_then(|output| output.sample_rate))
        .unwrap_or(sonarithm_spec::defaults::DEFAULT_SAMPLE_RATE);
    let rate = SampleRate::new(hz)
        .ok_or_else(|| Error::Render("the sample rate must be above 0 Hz".to_owned()))?;
    let block = output
        .as_ref()
        .and_then(|output| output.block_size)
        .and_then(|frames| usize::try_from(frames).ok())
        .filter(|frames| *frames > 0)
        .unwrap_or(sonarithm_render::offline::DEFAULT_BLOCK);
    let loudness = output.as_ref().and_then(|output| output.loudness);
    let normalization = options.normalization.unwrap_or(Normalization::Target {
        lufs: loudness
            .and_then(|l| l.target_lufs)
            .unwrap_or(sonarithm_spec::defaults::DEFAULT_TARGET_LUFS),
        true_peak: loudness
            .and_then(|l| l.true_peak_dbtp)
            .unwrap_or(sonarithm_spec::defaults::DEFAULT_TRUE_PEAK_DBTP),
    });
    let mut export = ExportOptions::new(format);
    export.plan = PlanConfig::new(rate);
    export.block_size = block;
    export.normalization = normalization;
    export.cues = options.cues;
    Ok(Settings { options: export })
}

/// The format a path asks for, or `explicit` when it agrees with the extension.
fn format_for(path: &Path, explicit: Option<AudioFormat>) -> Result<AudioFormat, Error> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    let from_extension = match extension.as_deref() {
        Some("wav") => Some(AudioFormat::Wav(WavEncoding::Pcm24)),
        Some("flac") => Some(AudioFormat::Flac),
        _ => None,
    };
    match (explicit, from_extension) {
        (Some(format), Some(found)) if format.extension() != found.extension() => {
            Err(Error::Render(format!(
                "`{}` names a .{} file, but the format is {}",
                crate::error::file_name(path),
                found.extension(),
                format.extension()
            )))
        }
        (Some(format), _) => Ok(format),
        (None, Some(found)) => Ok(found),
        (None, None) => Err(Error::Render(format!(
            "`{}` needs a .wav or .flac extension, or an explicit format",
            crate::error::file_name(path)
        ))),
    }
}

/// Where the data came from, for the manifest: the source kind and a name without
/// directories (P8).
fn data_source(compiled: &Compiled) -> Option<DataSource> {
    let spec: Value = serde_json::from_str(compiled.canonical_spec()).ok()?;
    let source = spec.pointer("/data/source")?;
    let kind = source.get("kind")?.as_str()?;
    let name = source
        .get("path")
        .or_else(|| source.get("id"))
        .and_then(Value::as_str)
        .unwrap_or(kind);
    Some(DataSource::new(kind, name))
}

fn manifest_inputs(compiled: &Compiled) -> ManifestInputs {
    let mut inputs = ManifestInputs::default();
    inputs.spec = serde_json::from_str(compiled.canonical_spec()).ok();
    inputs.data_source = data_source(compiled);
    inputs.provenance = compiled.provenance().clone();
    inputs
}

impl Sonarithm {
    /// Renders `compiled` to `path` (`.wav` or `.flac`), with `<stem>.render.json` (the
    /// render manifest: versions, hashes, settings, loudness; no timestamps or user paths)
    /// and, unless turned off, `<stem>.cues.vtt` next to it.
    ///
    /// ```no_run
    /// # use sonarithm::{RenderOptions, Sonarithm, Spec};
    /// let snr = Sonarithm::new();
    /// let compiled = snr.compile(&Spec::from_file("temps.spec.json")?)?;
    /// let rendered = snr.render_to_file(&compiled, "temps.flac", &RenderOptions::default())?;
    /// println!("{}", rendered.manifest.output.file);
    /// # Ok::<(), sonarithm::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::Render`] when the path has no supported extension or a file cannot be
    /// written.
    pub fn render_to_file(
        &self,
        compiled: &Compiled,
        path: impl AsRef<Path>,
        options: &RenderOptions,
    ) -> Result<Rendered, Error> {
        let path = path.as_ref();
        let format = format_for(path, options.format)?;
        let settings = settings(compiled, options, format)?;
        let directory = match path.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let stem = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .ok_or_else(|| Error::Render("the output path has no file name".to_owned()))?;
        let exported = sonarithm_export::export(
            compiled.score(),
            &settings.options,
            &manifest_inputs(compiled),
            &directory,
            &stem,
        )
        .map_err(|err| Error::Render(render_message(&err)))?;
        let cues = exported
            .manifest
            .cues
            .as_ref()
            .map(|cues| directory.join(&cues.file));
        Ok(Rendered {
            audio: directory.join(&exported.manifest.output.file),
            manifest_path: directory.join(format!("{stem}.render.json")),
            cues,
            manifest: exported.manifest,
            diagnostics: exported.diagnostics,
        })
    }

    /// Renders `compiled` to 24-bit WAV bytes in memory, at the level and rate a file
    /// render would use: the bytes equal those of [`Sonarithm::render_to_file`] for a
    /// `.wav` path. Returns the bytes and the sample rate.
    ///
    /// # Errors
    ///
    /// [`Error::Render`] when the Score cannot be planned or measured.
    pub fn render_wav(
        &self,
        compiled: &Compiled,
        options: &RenderOptions,
    ) -> Result<(Vec<u8>, u32), Error> {
        let format = AudioFormat::Wav(WavEncoding::Pcm24);
        let settings = settings(compiled, options, format)?.options;
        let failed = |message: String| Error::Render(message);
        let plan = RenderPlan::build(compiled.score(), &settings.plan)
            .map_err(|err| failed(err.to_string()))?
            .plan;
        let gain = match settings.normalization {
            Normalization::Target { lufs, true_peak } => {
                let mut meter =
                    LoudnessMeter::new(plan.sample_rate).map_err(|err| failed(err.to_string()))?;
                let mut render = OfflineRender::new(plan.clone(), settings.block_size);
                let mut block = vec![0.0; 2 * render.block_size()];
                loop {
                    let samples = render.next_block(&mut block);
                    if samples.is_empty() {
                        break;
                    }
                    meter.add(samples).map_err(|err| failed(err.to_string()))?;
                }
                let loudness = meter.loudness();
                normalization_gain((loudness.integrated, loudness.true_peak), lufs, true_peak)
            }
            _ => sonarithm_core::Db::ZERO,
        };
        #[expect(clippy::cast_possible_truncation, reason = "gains are f32")]
        let linear = gain.to_gain() as f32;
        let rate = plan.sample_rate.get();
        let mut render = OfflineRender::new(plan, settings.block_size);
        let mut bytes = Cursor::new(Vec::new());
        write_audio(&mut render, format, linear, &mut bytes)
            .map_err(|err| failed(err.to_string()))?;
        Ok((bytes.into_inner(), rate))
    }
}

/// The export error with its causes, for one message.
fn render_message(err: &sonarithm_export::ExportFailure) -> String {
    let mut message = err.to_string();
    let mut source = std::error::Error::source(err);
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}
