// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Versioned settings with migrations and crash-safe writes (12 §3, 09 §5, NFR-11).
//!
//! Settings live in `settings.json` in the folder the host names (the OS app-config
//! folder). Every write goes to `settings.json.tmp`, is flushed to the disk, then replaces
//! `settings.json` in one rename, after the previous file is copied to `settings.json.bak`.
//! A crash at any point leaves a readable `settings.json` or `settings.json.bak`; a stray
//! temporary file is ignored and removed by the next write.
//!
//! Reading migrates older versions step by step to [`SETTINGS_VERSION`]. A file from a
//! newer version is never overwritten: the defaults are used and the notice says why.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ts_rs::TS;

use sonarithm_speech::WordsPerMinute;

use crate::codes;
use crate::ipc::IpcError;
pub use crate::speech::SpeechRoute;

/// The current settings format version.
pub const SETTINGS_VERSION: u32 = 1;

/// The settings file's name.
pub const FILE: &str = "settings.json";
const BACKUP: &str = "settings.json.bak";

/// The sonification volume range in dB (A11Y-15). 0 dB is the level the Score was
/// mastered at; the volume only ever lowers it.
pub const VOLUME_RANGE_DB: (f64, f64) = (-60.0, 0.0);

/// The speaking rates a user may set, in words per minute: a sanity bound on the setting,
/// not a research value.
pub const SPEECH_RATE_RANGE: (f64, f64) = (WordsPerMinute::MIN.0, 1_000.0);

/// The buffer sizes a user may request, in frames.
pub const BUFFER_RANGE: (u32, u32) = (16, 8192);

/// Every user setting (12 §3). Unknown fields are refused, so a typo is an error and not a
/// silently ignored setting.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Settings {
    /// The format version, [`SETTINGS_VERSION`].
    pub settings_version: u32,
    /// Audio output.
    pub audio: AudioSettings,
    /// The UI theme.
    pub theme: Theme,
    /// Announcement verbosity (08 §4).
    pub verbosity: Verbosity,
    /// Which output modalities are on; they share one cursor (A11Y-10).
    pub modalities: Modalities,
    /// Keymap overrides: command ID → key combination (keyboard-and-navigation §13).
    pub keymap: BTreeMap<String, String>,
    /// Consent for features that use the network (ADR-0012). `None` means not asked yet.
    pub consent: Consent,
    /// How announcements are spoken and sequenced with sonification (08 §4).
    #[serde(default)]
    pub speech: SpeechSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: SETTINGS_VERSION,
            audio: AudioSettings::default(),
            theme: Theme::System,
            verbosity: Verbosity::Terse,
            modalities: Modalities::default(),
            keymap: BTreeMap::new(),
            consent: Consent::default(),
            speech: SpeechSettings::default(),
        }
    }
}

impl Settings {
    /// Checks every value's range.
    ///
    /// # Errors
    ///
    /// `SNR-APP-013` naming the first invalid value.
    pub fn validate(&self) -> Result<(), IpcError> {
        let (low, high) = VOLUME_RANGE_DB;
        let volume = self.audio.volume_db;
        if !(low..=high).contains(&volume) {
            return Err(IpcError::new(
                &codes::SETTINGS_VALUE,
                format!("The sonification volume {volume} dB is outside −60 to 0 dB."),
            ));
        }
        if let Some(frames) = self.audio.buffer_frames
            && !(BUFFER_RANGE.0..=BUFFER_RANGE.1).contains(&frames)
        {
            return Err(IpcError::new(
                &codes::SETTINGS_VALUE,
                format!("A buffer of {frames} frames is outside 16 to 8192 frames."),
            ));
        }
        let rate = self.speech.rate_wpm;
        if !(SPEECH_RATE_RANGE.0..=SPEECH_RATE_RANGE.1).contains(&rate) {
            return Err(IpcError::new(
                &codes::SETTINGS_VALUE,
                format!("A speaking rate of {rate} words per minute is outside 30 to 1000."),
            ));
        }
        if self.settings_version != SETTINGS_VERSION {
            return Err(IpcError::new(
                &codes::SETTINGS_VALUE,
                format!(
                    "Settings version {} is not the current version {SETTINGS_VERSION}.",
                    self.settings_version
                ),
            ));
        }
        Ok(())
    }
}

/// Audio output settings.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct AudioSettings {
    /// The output device's ID, or `None` for the system default.
    pub device: Option<String>,
    /// The buffer size to request, or `None` for the device's default.
    pub buffer_frames: Option<u32>,
    /// The sonification volume in dB, independent of the system volume (A11Y-15).
    pub volume_db: f64,
}

/// The UI theme (design-language §5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Theme {
    /// Follow the operating system.
    #[default]
    System,
    /// Dark.
    Dark,
    /// Light.
    Light,
    /// High contrast, dark.
    HighContrastDark,
    /// High contrast, light.
    HighContrastLight,
}

/// Announcement verbosity (08 §4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Verbosity {
    /// Values only.
    #[default]
    Terse,
    /// Values with units and context.
    Verbose,
}

/// Output modalities (A11Y-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each modality is an independent switch (A11Y-10)"
)]
pub struct Modalities {
    /// Sonification.
    pub sonification: bool,
    /// Speech and text announcements.
    pub speech: bool,
    /// Braille output.
    pub braille: bool,
    /// The visual plot.
    pub visual: bool,
}

impl Default for Modalities {
    fn default() -> Self {
        Self {
            sonification: true,
            speech: true,
            braille: true,
            visual: true,
        }
    }
}

/// How announcements are spoken (08 §4, keyboard-and-navigation §9.2; B03-T07).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct SpeechSettings {
    /// Where announcements are spoken: the screen reader (the default, A11Y-19) or
    /// SONARITHM's own voice.
    pub route: SpeechRoute,
    /// A speaking command during playback ducks sonification 20 dB instead of pausing it
    /// (keyboard §9.2 item 4; A11Y-20).
    pub duck_instead_of_pause: bool,
    /// The speaking rate that sequencing estimates assume, in words per minute.
    pub rate_wpm: f64,
}

impl Default for SpeechSettings {
    fn default() -> Self {
        Self {
            route: SpeechRoute::ScreenReader,
            duck_instead_of_pause: false,
            rate_wpm: WordsPerMinute::DEFAULT.0,
        }
    }
}

/// Consent to network features (ADR-0012). Each is `None` until the user answers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct Consent {
    /// The anonymous update check (12 §7).
    pub update_check: Option<bool>,
}

/// What [`SettingsStore::load`] found.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Loaded {
    /// The settings in effect.
    pub settings: Settings,
    /// The version the file had before migration, if a file was read.
    pub migrated_from: Option<u32>,
    /// Notices for the UI: a recovery, or a newer file left alone.
    pub notices: Vec<IpcError>,
}

/// Where a write is interrupted, for fault-injection tests of crash safety.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Write half of the temporary file, then stop as if the process died.
    PartialTemp,
    /// Write the temporary file completely, then stop before the rename.
    BeforeRename,
}

/// The settings file in one folder.
#[derive(Debug)]
pub struct SettingsStore {
    dir: PathBuf,
    /// Set when the file on disk is from a newer version: never overwrite it.
    read_only: bool,
    fault: Option<Fault>,
}

impl SettingsStore {
    /// The store in `dir`, the OS app-config folder the host chose. The folder is created
    /// on the first write.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            read_only: false,
            fault: None,
        }
    }

    /// Interrupts the next writes at `fault` (fault-injection tests).
    #[doc(hidden)]
    pub fn inject_fault(&mut self, fault: Option<Fault>) {
        self.fault = fault;
    }

    /// The settings file's path.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.dir.join(FILE)
    }

    /// Reads the settings: the file, else its backup, else the defaults. Older versions
    /// are migrated (and written back in the current version).
    #[must_use]
    pub fn load(&mut self) -> Loaded {
        let mut notices = Vec::new();
        let main = read_value(&self.dir.join(FILE));
        let (value, recovered) = match main {
            Read::Missing => (None, false),
            Read::Value(value) => (Some(value), false),
            Read::Damaged => match read_value(&self.dir.join(BACKUP)) {
                Read::Value(value) => (Some(value), true),
                Read::Missing | Read::Damaged => (None, true),
            },
        };
        let Some(value) = value else {
            if recovered {
                notices.push(IpcError::new(
                    &codes::SETTINGS_RECOVERED,
                    "The settings file is damaged and has no readable backup, so the defaults are in use.",
                ));
            }
            return Loaded {
                settings: Settings::default(),
                migrated_from: None,
                notices,
            };
        };
        let version = version_of(&value);
        if version > SETTINGS_VERSION {
            self.read_only = true;
            notices.push(IpcError::new(
                &codes::SETTINGS_NEWER,
                format!(
                    "The settings file has version {version}; this version of SONARITHM reads up to version {SETTINGS_VERSION}, so it uses its defaults."
                ),
            ));
            return Loaded {
                settings: Settings::default(),
                migrated_from: Some(version),
                notices,
            };
        }
        let migrated = migrate(value, version);
        let settings = migrated
            .and_then(|value| serde_json::from_value::<Settings>(value).ok())
            .filter(|settings| settings.validate().is_ok());
        let Some(settings) = settings else {
            notices.push(IpcError::new(
                &codes::SETTINGS_RECOVERED,
                "The settings file has values this version cannot use, so the defaults are in use.",
            ));
            return Loaded {
                settings: Settings::default(),
                migrated_from: Some(version),
                notices,
            };
        };
        if recovered {
            notices.push(IpcError::new(
                &codes::SETTINGS_RECOVERED,
                "The settings file is damaged, so the last good copy is in use.",
            ));
        }
        if (version < SETTINGS_VERSION || recovered)
            && let Err(error) = self.save(&settings)
        {
            notices.push(error);
        }
        Loaded {
            settings,
            migrated_from: Some(version),
            notices,
        }
    }

    /// Validates and writes `settings`, crash-safely.
    ///
    /// # Errors
    ///
    /// `SNR-APP-013` for an invalid value, `SNR-APP-012` when the file on disk is from a
    /// newer version, `SNR-APP-010` when the file cannot be written; the previous file stays.
    pub fn save(&self, settings: &Settings) -> Result<(), IpcError> {
        settings.validate()?;
        if self.read_only {
            return Err(IpcError::new(
                &codes::SETTINGS_NEWER,
                "The settings file is from a newer version of SONARITHM, so it is not overwritten.",
            ));
        }
        let mut text =
            serde_json::to_string_pretty(settings).map_err(|err| write_error(&err.to_string()))?;
        text.push('\n');
        self.write_atomic(text.as_bytes())
    }

    fn write_atomic(&self, bytes: &[u8]) -> Result<(), IpcError> {
        let io = |err: std::io::Error| write_error(&err.to_string());
        std::fs::create_dir_all(&self.dir).map_err(io)?;
        let target = self.dir.join(FILE);
        let backup = || -> Result<(), String> {
            // Keep the last good copy; a damaged main file falls back to it.
            if target.exists() && matches!(read_value(&target), Read::Value(_)) {
                std::fs::copy(&target, self.dir.join(BACKUP)).map_err(|err| err.to_string())?;
            }
            Ok(())
        };
        write_atomically_with(&target, bytes, self.fault, backup)
            .map_err(|detail| write_error(&detail))
    }
}

/// Writes `bytes` to `target` crash-safely: `<target>.tmp`, flushed, then renamed over the
/// target. A crash at any point leaves the previous file whole (NFR-11). `fault` interrupts
/// the write, for fault-injection tests.
///
/// # Errors
///
/// The file system's error, in words; the previous file stays.
pub(crate) fn write_atomically(
    target: &Path,
    bytes: &[u8],
    fault: Option<Fault>,
) -> Result<(), String> {
    write_atomically_with(target, bytes, fault, || Ok(()))
}

fn write_atomically_with(
    target: &Path,
    bytes: &[u8],
    fault: Option<Fault>,
    before_rename: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let io = |err: std::io::Error| err.to_string();
    let mut temp = target.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    {
        let mut file = std::fs::File::create(&temp).map_err(io)?;
        if fault == Some(Fault::PartialTemp) {
            let half = bytes.get(..bytes.len() / 2).unwrap_or_default();
            file.write_all(half).map_err(io)?;
            return Err("interrupted (fault injection)".to_owned());
        }
        file.write_all(bytes).map_err(io)?;
        file.sync_all().map_err(io)?;
    }
    if fault == Some(Fault::BeforeRename) {
        return Err("interrupted (fault injection)".to_owned());
    }
    before_rename()?;
    std::fs::rename(&temp, target).map_err(io)?;
    if let Some(dir) = target.parent() {
        sync_dir(dir);
    }
    Ok(())
}

fn write_error(detail: &str) -> IpcError {
    IpcError::new(
        &codes::SETTINGS_WRITE,
        format!("The settings file could not be written: {detail}."),
    )
}

/// Makes the rename durable where the OS needs the folder flushed too.
fn sync_dir(dir: &Path) {
    #[cfg(unix)]
    if let Ok(handle) = std::fs::File::open(dir) {
        let _ = handle.sync_all();
    }
    #[cfg(not(unix))]
    let _ = dir;
}

enum Read {
    Missing,
    Damaged,
    Value(Value),
}

fn read_value(path: &Path) -> Read {
    match std::fs::read(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Read::Missing,
        Err(_) => Read::Damaged,
        Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
            Ok(value @ Value::Object(_)) => Read::Value(value),
            _ => Read::Damaged,
        },
    }
}

/// The version of a settings document: `settingsVersion`, or 0 for the unversioned layout.
fn version_of(value: &Value) -> u32 {
    value
        .get("settingsVersion")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0)
}

/// Migrates a document from `version` to [`SETTINGS_VERSION`], one step at a time.
fn migrate(mut value: Value, version: u32) -> Option<Value> {
    let mut at = version;
    while at < SETTINGS_VERSION {
        value = match at {
            0 => v0_to_v1(value)?,
            _ => return None,
        };
        at += 1;
    }
    Some(value)
}

/// Version 0 is the flat, unversioned layout (`theme`, `outputDevice`, `bufferSize`,
/// `volume`, `verbose`, `keymap`). No release wrote it; it keeps the migration chain
/// exercised from the first version on.
fn v0_to_v1(value: Value) -> Option<Value> {
    let Value::Object(old) = value else {
        return None;
    };
    let text = |key: &str| old.get(key).and_then(Value::as_str).map(str::to_owned);
    let mut audio = Map::new();
    audio.insert(
        "device".into(),
        text("outputDevice").map_or(Value::Null, Value::String),
    );
    audio.insert(
        "bufferFrames".into(),
        old.get("bufferSize")
            .and_then(Value::as_u64)
            .map_or(Value::Null, Value::from),
    );
    audio.insert(
        "volumeDb".into(),
        Value::from(old.get("volume").and_then(Value::as_f64).unwrap_or(0.0)),
    );
    let theme = match text("theme").as_deref() {
        Some("dark") => "dark",
        Some("light") => "light",
        Some("contrast") => "highContrastDark",
        _ => "system",
    };
    let verbose = old.get("verbose").and_then(Value::as_bool).unwrap_or(false);
    let mut new = Map::new();
    new.insert("settingsVersion".into(), Value::from(1));
    new.insert("audio".into(), Value::Object(audio));
    new.insert("theme".into(), Value::from(theme));
    new.insert(
        "verbosity".into(),
        Value::from(if verbose { "verbose" } else { "terse" }),
    );
    new.insert(
        "modalities".into(),
        serde_json::to_value(Modalities::default()).ok()?,
    );
    new.insert(
        "keymap".into(),
        old.get("keymap")
            .cloned()
            .filter(Value::is_object)
            .unwrap_or_else(|| Value::Object(Map::new())),
    );
    new.insert(
        "consent".into(),
        serde_json::to_value(Consent::default()).ok()?,
    );
    Some(Value::Object(new))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TempDir;

    #[test]
    fn defaults_round_trip_through_json() {
        let settings = Settings::default();
        let text = serde_json::to_string(&settings).unwrap_or_default();
        let back: Settings = serde_json::from_str(&text).unwrap_or_default();
        assert_eq!(back, settings);
    }

    #[test]
    fn unknown_fields_are_refused() {
        let text = r#"{"settingsVersion":1,"audio":{"device":null,"bufferFrames":null,"volumeDb":0.0},
            "theme":"dark","verbosity":"terse","modalities":{"sonification":true,"speech":true,"braille":true,"visual":true},
            "keymap":{},"consent":{"updateCheck":null},"telemetry":true}"#;
        assert!(serde_json::from_str::<Settings>(text).is_err());
    }

    #[test]
    fn missing_file_gives_defaults_without_notice() {
        let dir = TempDir::new("settings-missing");
        let loaded = SettingsStore::new(dir.path()).load();
        assert_eq!(loaded.settings, Settings::default());
        assert_eq!(loaded.notices, Vec::new());
        assert_eq!(loaded.migrated_from, None);
    }

    #[test]
    fn a11y_15_volume_outside_range_is_refused() {
        let dir = TempDir::new("settings-volume");
        let store = SettingsStore::new(dir.path());
        let mut settings = Settings::default();
        settings.audio.volume_db = 6.0;
        let error = store.save(&settings).err();
        assert_eq!(error.map(|e| e.code), Some("SNR-APP-013".to_owned()));
        assert!(!store.path().exists());
    }

    #[test]
    fn newer_file_is_never_overwritten() {
        let dir = TempDir::new("settings-newer");
        let path = dir.path().join(FILE);
        let newer = r#"{"settingsVersion": 99, "future": true}"#;
        std::fs::write(&path, newer).unwrap_or_default();
        let mut store = SettingsStore::new(dir.path());
        let loaded = store.load();
        assert_eq!(loaded.settings, Settings::default());
        assert_eq!(loaded.notices[0].code, "SNR-APP-012");
        assert!(store.save(&Settings::default()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap_or_default(), newer);
    }
}
