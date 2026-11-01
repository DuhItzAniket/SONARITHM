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

/// The calibrated output latencies a user may store, in seconds: a sanity bound on the
/// setting (an output a second late is a broken device), not a research value.
pub const LATENCY_RANGE_S: (f64, f64) = (0.0, 1.0);

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
    /// Keymap overrides: command ID → keys (keyboard-and-navigation §13): chords separated
    /// by spaces, or the empty string to unbind the command. Reserved chords are refused
    /// (`SNR-APP-091`, §13 rule 6).
    pub keymap: BTreeMap<String, String>,
    /// Keyboard choices beyond single bindings (B03-T09).
    #[serde(default)]
    pub keyboard: KeyboardSettings,
    /// Consent for features that use the network (ADR-0012). `None` means not asked yet.
    pub consent: Consent,
    /// How announcements are spoken and sequenced with sonification (08 §4).
    #[serde(default)]
    pub speech: SpeechSettings,
    /// The active profile's ID (ADR-0020), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_profile: Option<String>,
    /// Which GPU the webview draws plots with (B04-T02, F-044). Read by the host when it
    /// starts, so a change applies after a restart.
    #[serde(default)]
    pub gpu: GpuPreference,
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
            keyboard: KeyboardSettings::default(),
            consent: Consent::default(),
            speech: SpeechSettings::default(),
            active_profile: None,
            gpu: GpuPreference::default(),
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
        let (low, high) = LATENCY_RANGE_S;
        if let Some(latency) = self
            .audio
            .calibrated_latency_s
            .values()
            .find(|latency| !(low..=high).contains(*latency))
        {
            // The device is not named: its ID can contain a person's name (rule 9).
            return Err(IpcError::new(
                &codes::SETTINGS_VALUE,
                format!("A calibrated output latency of {latency} s is outside 0 to 1 s."),
            ));
        }
        let rate = self.speech.rate_wpm;
        if !(SPEECH_RATE_RANGE.0..=SPEECH_RATE_RANGE.1).contains(&rate) {
            return Err(IpcError::new(
                &codes::SETTINGS_VALUE,
                format!("A speaking rate of {rate} words per minute is outside 30 to 1000."),
            ));
        }
        if let Some((command, chord)) = crate::keymap::first_reserved(&self.keymap) {
            return Err(IpcError::new(
                &codes::KEYMAP_REFUSED,
                format!(
                    "{chord} is reserved and cannot be bound to {command} (keyboard §13 rule 6)."
                ),
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

/// Keyboard choices beyond single bindings (keyboard-and-navigation §13).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct KeyboardSettings {
    /// The group switch of KB-02 and §13 rule 3: "Single-key commands in the chart
    /// explorer". Off unbinds every single-character binding there (letters, Shift+letters,
    /// punctuation); arrows, Home, End, Page Up, Page Down, Space and Escape stay. On by
    /// default: focus scoping already satisfies WCAG 2.1.4.
    pub single_key_commands: bool,
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            single_key_commands: true,
        }
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
    /// The measured output latency of each device, in seconds, by device ID (ADR-0016,
    /// per-device latency calibration; B04-T04). For a device listed here, clock anchors
    /// carry this instead of the backend's reported latency (ADR-0017 "Clock anchors").
    ///
    /// Privacy (ADR-0012, AGENTS rule 9): a device ID or name can contain a person's name
    /// (a headset named after its owner), so this map lives only in the local settings
    /// file. It is never written to logs, manifests, projects or exported diagnostics, and
    /// no error message names a device from it.
    #[serde(default)]
    pub calibrated_latency_s: BTreeMap<String, f64>,
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
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS, schemars::JsonSchema,
)]
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
    /// How the host hands announcements to the screen reader (OD-05, B03-T04). Added after
    /// settings version 1 was written, so files without it read the default.
    #[serde(default)]
    pub announcer: AnnouncerRoute,
}

impl Default for SpeechSettings {
    fn default() -> Self {
        Self {
            route: SpeechRoute::ScreenReader,
            duck_instead_of_pause: false,
            rate_wpm: WordsPerMinute::DEFAULT.0,
            announcer: AnnouncerRoute::default(),
        }
    }
}

/// How the host passes announcements to the screen reader when [`SpeechRoute::ScreenReader`]
/// is the route (08 §4; OD-05, decided provisionally on 2026-10-29).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum AnnouncerRoute {
    /// The managed ARIA live-region pair in the webview: the default, spoken by NVDA in the
    /// A02 walkthrough.
    #[default]
    LiveRegion,
    /// The platform's notification API (UI Automation on Windows). NVDA did not speak it in
    /// the A02 walkthrough (#382); hosts without one use live regions.
    Native,
}

/// Which GPU the webview's plots use (B04-T02, F-044; approved by the coordinator on
/// 2026-10-30). Only WebView2 (Windows) can be asked: the host passes
/// `--force_high_performance_gpu` on top of its default browser arguments. Other webviews
/// ignore it, and the UI says so. There is no detection and no telemetry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum GpuPreference {
    /// The webview's own choice: on a hybrid laptop, usually the integrated GPU.
    #[default]
    Default,
    /// The discrete, high-performance GPU of a hybrid laptop, for dense plots (F-044).
    HighPerformance,
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
        let mut cleaned = false;
        let settings = migrated
            .and_then(|value| serde_json::from_value::<Settings>(value).ok())
            .map(|mut settings| {
                while let Some((command, chord)) = crate::keymap::first_reserved(&settings.keymap)
                {
                    notices.push(IpcError::new(
                        &codes::KEYMAP_REFUSED,
                        format!(
                            "{chord} is reserved, so the settings file's binding for {command} \
                             was left out (keyboard §13 rule 6)."
                        ),
                    ));
                    let command = command.to_owned();
                    settings.keymap.remove(&command);
                    cleaned = true;
                }
                settings
            })
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
        // A file that lost a reserved binding is written back, so the notice is said once.
        if (version < SETTINGS_VERSION || recovered || cleaned)
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
    fn od_05_speech_settings_without_the_announcer_route_read_live_regions() {
        // A file written before B03-T04 has no `announcer`.
        let text = r#"{"route":"screenReader","duckInsteadOfPause":false,"rateWpm":180.0}"#;
        let speech: SpeechSettings = serde_json::from_str(text).unwrap_or_default();
        assert_eq!(speech.announcer, AnnouncerRoute::LiveRegion);
        let native = r#"{"route":"screenReader","duckInsteadOfPause":false,"rateWpm":180.0,
            "announcer":"native"}"#;
        let speech: SpeechSettings = serde_json::from_str(native).unwrap_or_default();
        assert_eq!(speech.announcer, AnnouncerRoute::Native);
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
    fn b04_t04_a_latency_outside_range_is_refused_without_naming_the_device() {
        let mut settings = Settings::default();
        settings
            .audio
            .calibrated_latency_s
            .insert("Ada's AirPods".to_owned(), 0.142);
        assert!(settings.validate().is_ok());
        settings
            .audio
            .calibrated_latency_s
            .insert("Ada's AirPods".to_owned(), 2.5);
        let error = settings.validate().err();
        assert_eq!(error.as_ref().map(|e| e.code.as_str()), Some("SNR-APP-013"));
        assert!(error.is_some_and(|e| !e.message.contains("Ada")));
    }

    #[test]
    fn b04_t04_settings_without_latencies_still_load() {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        value["audio"]
            .as_object_mut()
            .unwrap()
            .remove("calibratedLatencyS");
        let settings: Settings = serde_json::from_value(value).unwrap();
        assert!(settings.audio.calibrated_latency_s.is_empty());
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

    /// Keyboard §13 rule 6 (B03-T09): saving a reserved chord is refused with
    /// `SNR-APP-091`, and nothing is written.
    #[test]
    fn keyboard_13_a_reserved_chord_is_refused_on_save() {
        let dir = TempDir::new("settings-reserved");
        let store = SettingsStore::new(dir.path());
        let mut settings = Settings::default();
        settings.keymap.insert("nav.next".to_owned(), "Shift+Tab".to_owned());
        let error = store.save(&settings).err();
        assert_eq!(error.map(|e| e.code), Some("SNR-APP-091".to_owned()));
        assert!(!store.path().exists());
    }

    /// B03-T09: keys and the group switch persist, and a hand-edited file with a reserved
    /// chord loses only that binding, with a notice, not every setting.
    #[test]
    fn keyboard_13_keys_persist_and_a_hand_edited_reserved_chord_is_left_out() {
        let dir = TempDir::new("settings-keys");
        let mut store = SettingsStore::new(dir.path());
        let mut settings = Settings::default();
        settings.keymap.insert("nav.next".to_owned(), "L".to_owned());
        settings.keyboard.single_key_commands = false;
        settings.verbosity = Verbosity::Verbose;
        assert!(store.save(&settings).is_ok());
        assert_eq!(store.load().settings, settings);

        let mut value = serde_json::to_value(&settings).unwrap();
        value["keymap"]["nav.previous"] = serde_json::json!("Tab");
        std::fs::write(store.path(), value.to_string()).unwrap();
        let loaded = store.load();
        assert_eq!(loaded.settings, settings, "only the reserved binding is gone");
        assert_eq!(loaded.notices.len(), 1);
        assert_eq!(loaded.notices[0].code, "SNR-APP-091");
        assert_eq!(store.load().notices, Vec::new(), "written back, so said once");
    }

    #[test]
    fn b03_t09_settings_without_the_keyboard_section_still_load() {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        value.as_object_mut().unwrap().remove("keyboard");
        let settings: Settings = serde_json::from_value(value).unwrap();
        assert!(settings.keyboard.single_key_commands);
    }
}
