// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The render manifest, `<name>.render.json` (ADR-0013, 06 §7): everything needed to
//! reproduce and attribute an export.
//!
//! It records the engine and crate versions, the spec version and hash (and optionally the
//! canonical spec itself), the data fingerprint and provenance mix, the seed, sample rate,
//! block size and mode, the output file's hash and loudness, and the OS and architecture.
//! It holds no wall-clock time. **Paths are never absolute** (privacy, ADR-0012): file and
//! data-source names keep only their last component.
//!
//! The JSON Schema is generated from these types (`cargo xtask codegen`, schema at
//! [`MANIFEST_SCHEMA_PATH`]); manifests carry their [`MANIFEST_VERSION`].

use std::collections::BTreeMap;

use schemars::JsonSchema;
use schemars::generate::SchemaSettings;
use serde::{Deserialize, Serialize};
use sonarithm_core::{Mode, Sha256Digest};

/// The name of a mode as the spec writes it.
#[must_use]
pub fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Analysis => "analysis",
        Mode::Presentation => "presentation",
        Mode::Monitoring => "monitoring",
    }
}

/// The manifest format version. Bump it on any change to the serialized form.
///
/// Version 2 adds `spec.experimental` (RFC-0006 §4). Version 1 manifests still read: the
/// field is optional and left out when no experiment is on.
pub const MANIFEST_VERSION: u32 = 2;

/// Where the generated schema is committed, relative to the workspace root. The version 1
/// schema stays as written.
pub const MANIFEST_SCHEMA_PATH: &str = "schemas/sonarithm-render-manifest-2.schema.json";

/// A render manifest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct RenderManifest {
    /// The manifest format version (`MANIFEST_VERSION`, currently 2; 1 still reads).
    pub manifest_version: u32,
    /// The software that rendered.
    pub sonarithm: Software,
    /// The Score that was rendered.
    pub score: ScoreInfo,
    /// The spec it was compiled from.
    pub spec: SpecInfo,
    /// The input data.
    pub data: DataInfo,
    /// Render settings.
    pub render: RenderSettings,
    /// The audio file.
    pub output: OutputInfo,
    /// The cue sidecar, if one was written.
    pub cues: Option<CueInfo>,
    /// Where it ran.
    pub platform: Platform,
}

/// Versions of the software.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Software {
    /// The SONARITHM version (of `sonarithm-export`, which shares the workspace version).
    pub version: String,
    /// The git commit of the build, when the caller knows it.
    pub git: Option<String>,
    /// The engines that contributed, with their versions, from the Score header (context
    /// providers only when they added a track; shapers whenever they ran; 05 §4, #434).
    pub engines: BTreeMap<String, String>,
}

/// The rendered Score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct ScoreInfo {
    /// The Score IR version.
    pub ir_version: u32,
    /// SHA-256 of the canonical Score, `sha256:` and 64 hex digits (05 §4.3).
    #[schemars(regex(pattern = r"^sha256:[0-9a-f]{64}$"))]
    pub hash: String,
}

/// The spec.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct SpecInfo {
    /// The spec language version, such as `0.1`.
    pub version: String,
    /// SHA-256 of the RFC 8785 canonical spec with defaults applied.
    #[schemars(regex(pattern = r"^sha256:[0-9a-f]{64}$"))]
    pub hash: String,
    /// The canonical spec itself, when embedded.
    pub embedded: Option<serde_json::Value>,
    /// The experimental switches that were on (RFC-0006), sorted; left out when none was
    /// (manifest version 2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub experimental: Vec<String>,
}

/// The input data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct DataInfo {
    /// SHA-256 fingerprint of the data (03 §2).
    #[schemars(regex(pattern = r"^sha256:[0-9a-f]{64}$"))]
    pub fingerprint: String,
    /// Where it came from, without any directory.
    pub source: Option<DataSource>,
    /// The share of values in each provenance class (03 §4), for example `exact: 1.0`.
    pub provenance: BTreeMap<String, f64>,
}

/// A data source: its kind and a name with no directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct DataSource {
    /// The source kind, such as `file`.
    pub kind: String,
    /// The name, with any directory removed.
    pub name: String,
}

impl DataSource {
    /// A source of `kind` named `name`. Any directory in `name` is dropped, so an absolute
    /// path never reaches the manifest.
    #[must_use]
    pub fn new(kind: impl Into<String>, name: &str) -> Self {
        Self {
            kind: kind.into(),
            name: file_name(name),
        }
    }
}

/// Render settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct RenderSettings {
    /// Frames per second.
    pub sample_rate: u32,
    /// The offline block size in frames.
    pub block_size: u32,
    /// The Score seed (SDR-02).
    pub seed: u64,
    /// The mode (SDR-36): `analysis`, `presentation` or `monitoring`.
    pub mode: String,
    /// The length in seconds.
    pub duration_s: f64,
    /// The ISO 226:2023 loudness level used for compensation (SDR-24), or `None` if off.
    pub loudness_compensation_phon: Option<f64>,
}

/// The audio file and its measured loudness.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct OutputInfo {
    /// The file name, with no directory.
    pub file: String,
    /// `wav` or `flac`.
    pub format: String,
    /// Bits per sample.
    pub bits_per_sample: u16,
    /// Channels.
    pub channels: u16,
    /// Frames per channel.
    pub frames: u64,
    /// SHA-256 of the file's bytes.
    #[schemars(regex(pattern = r"^sha256:[0-9a-f]{64}$"))]
    pub sha256: String,
    /// Integrated loudness in LUFS (EBU R128), or `None` for silence.
    pub integrated_lufs: Option<f64>,
    /// True peak in dBTP.
    pub true_peak_dbtp: f64,
    /// The static gain applied for loudness normalization, in dB (sound-design §6.3).
    pub gain_db: f64,
    /// Frames during which the master limiter reduced the level.
    pub limited_frames: u64,
}

/// The WebVTT cue sidecar.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct CueInfo {
    /// The file name, with no directory.
    pub file: String,
    /// The number of cues.
    pub count: u32,
}

/// The OS and CPU architecture of the build.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Platform {
    /// `windows`, `linux`, `macos`, …
    pub os: String,
    /// `x86_64`, `aarch64`, …
    pub arch: String,
}

impl Platform {
    /// The platform this code was compiled for (compile-time constants, not environment
    /// reads).
    #[must_use]
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_owned(),
            arch: std::env::consts::ARCH.to_owned(),
        }
    }
}

/// `sha256:` followed by the digest's hex digits.
#[must_use]
pub fn sha256_text(digest: &Sha256Digest) -> String {
    format!("sha256:{digest}")
}

/// The last component of `name`, after any `/`, `\` or drive `:`, so no directory, drive
/// or user name is kept.
#[must_use]
pub fn file_name(name: &str) -> String {
    name.rsplit(['/', '\\', ':'])
        .next()
        .unwrap_or_default()
        .to_owned()
}

impl RenderManifest {
    /// The manifest as pretty-printed JSON with a trailing newline.
    ///
    /// # Errors
    ///
    /// A `serde_json` error if a value cannot be written, such as a non-finite number.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let mut text = serde_json::to_string_pretty(self)?;
        text.push('\n');
        Ok(text)
    }
}

/// The JSON Schema (draft 2020-12) of [`RenderManifest`], pretty-printed with a trailing
/// newline. `cargo xtask codegen` writes it to [`MANIFEST_SCHEMA_PATH`].
#[must_use]
pub fn json_schema() -> String {
    let generator = SchemaSettings::draft2020_12().into_generator();
    let mut schema = generator.into_root_schema_for::<RenderManifest>();
    schema.insert(
        "$comment".to_owned(),
        "Generated by `cargo xtask codegen` from the sonarithm-export types; do not edit.".into(),
    );
    let mut text = serde_json::to_string_pretty(&schema).unwrap_or_default();
    text.push('\n');
    text
}
