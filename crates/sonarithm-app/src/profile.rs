// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Listener profiles (ADR-0020; B03-T08; SDR-15, SDR-38, A11Y-10, A11Y-12).
//!
//! A profile is one portable JSON document of preferences, every one optional: user
//! profiles in the app-config folder, built-in presets, and a project's
//! `profiles/project-profile.json`. Settings come from six layers (12 §3):
//!
//! **built-in defaults < user settings < active profile < project profile < spec < explicit
//! UI action**
//!
//! Interaction preferences (verbosity, part order, modalities, speech, keymap) are resolved
//! here field by field ([`effective`]). Sound preferences reach the compiler only through
//! the spec: [`effective_spec`] fills what the spec leaves unset, and writes every polarity
//! a profile decides as an explicit value, so the compile never sees a profile and a Score
//! re-renders from its manifest's spec alone (ADR-0013).

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sonarithm_core::SpecPointer;
use sonarithm_data::DataBundle;
use sonarithm_numisona::polarity::{
    ConceptOrigin, ConceptTable, Polarity, PolarityChoice, PolarityProfile, PolarityRequest,
    resolve,
};
use ts_rs::TS;

use crate::codes;
use crate::ipc::IpcError;
use crate::settings::{Settings, SpeechRoute, Verbosity};

/// The `format` of every profile document.
pub const FORMAT: &str = "sonarithm-profile";
/// The version this build writes and reads: major, minor (ADR-0020 §1).
pub const FORMAT_VERSION: (u32, u32) = (1, 0);
/// The largest profile file accepted (ADR-0020 §1).
pub const MAX_BYTES: usize = 1 << 20;
/// Where the generated schema is committed, relative to the workspace root.
pub const SCHEMA_PATH: &str = "schemas/sonarithm-profile-1.schema.json";
/// The ID that names the project's own profile in the profile commands.
pub const PROJECT_ID: &str = "project";
/// The prefix of the built-in presets' IDs.
pub const PRESET_PREFIX: &str = "preset:";

/// A listener profile (ADR-0020 §1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(title = "SONARITHM profile")]
#[non_exhaustive]
pub struct Profile {
    /// Always `sonarithm-profile`.
    pub format: String,
    /// `MAJOR.MINOR`; this build writes `1.0`.
    pub format_version: String,
    /// The name the listener sees.
    pub name: String,
    /// How data sounds; each value fills only what a spec leaves unset.
    #[serde(default)]
    pub sound: SoundPrefs,
    /// How the listener works with the app.
    #[serde(default)]
    pub interaction: InteractionPrefs,
}

impl Profile {
    /// An empty profile called `name`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            format: FORMAT.to_owned(),
            format_version: format!("{}.{}", FORMAT_VERSION.0, FORMAT_VERSION.1),
            name: name.into(),
            sound: SoundPrefs::default(),
            interaction: InteractionPrefs::default(),
        }
    }
}

/// Sound preferences (SDR-38).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct SoundPrefs {
    /// The pitch range of every pitch channel (SDR-08).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch_range: Option<PitchRange>,
    /// The instrument of every track, such as `mallet` or `square` (SDR-29).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timbre: Option<String>,
    /// Discrete or continuous marks where a spec leaves the choice (quick and templates).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marks: Option<MarkPreference>,
    /// The overview rate (SDR-19).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace: Option<Pace>,
    /// Polarity by concept, and the one-key flips (SDR-15).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polarity: Option<PolarityPrefs>,
    /// The context cues (SDR-16..18).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextPrefs>,
}

/// A pitch range in hertz, low to high.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PitchRange {
    /// Low and high, in Hz.
    pub hz: [f64; 2],
}

/// The overview rate.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pace {
    /// Data points per second.
    pub points_per_second: f64,
}

/// Discrete notes or continuous contours (SDR-38).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum MarkPreference {
    /// One note per datum (`tone`).
    Discrete,
    /// A continuous contour (`glide`).
    Continuous,
}

/// A polarity direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    /// Higher values sound higher.
    Positive,
    /// Higher values sound lower.
    Negative,
}

/// Polarity preferences (SDR-15).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct PolarityPrefs {
    /// The direction `auto` takes for a concept, such as `cost`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub concepts: BTreeMap<String, Direction>,
    /// Fields whose polarity the one-key flip inverts.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub flipped_fields: BTreeSet<String>,
}

/// Context cue preferences (SDR-16..18).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct ContextPrefs {
    /// The tick track.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticks: Option<bool>,
    /// The dynamic reference tone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_tone: Option<bool>,
    /// The spoken legend before playback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spoken_legend: Option<bool>,
}

/// Interaction preferences (A11Y-10, A11Y-12).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct InteractionPrefs {
    /// Terse or verbose announcements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
    /// The order of announcement parts per verbosity (A11Y-12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_order: Option<PartOrder>,
    /// Which output modalities are on (A11Y-10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<ModalityPrefs>,
    /// How announcements are spoken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech: Option<SpeechPrefs>,
    /// Keymap overrides: command ID → keys (keyboard-and-navigation §13).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub keymap: BTreeMap<String, String>,
}

/// Announcement parts (AUDAXIS, A11Y-12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum PartName {
    /// The series name.
    Series,
    /// The x value.
    X,
    /// The data value.
    Value,
    /// "3 of 12".
    Position,
    /// The level, such as "segment".
    Level,
    /// Tags such as "maximum".
    Tags,
    /// The provenance qualifier.
    Provenance,
    /// The key hint.
    Hint,
}

/// The part order per verbosity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PartOrder {
    /// The terse order.
    pub terse: Vec<PartName>,
    /// The verbose order.
    pub verbose: Vec<PartName>,
}

/// Modality switches; unset ones come from the layer below.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct ModalityPrefs {
    /// Sonification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sonification: Option<bool>,
    /// Speech and text announcements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech: Option<bool>,
    /// Braille output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub braille: Option<bool>,
    /// The visual plot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual: Option<bool>,
}

/// Speech preferences.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub struct SpeechPrefs {
    /// The screen reader or self-voicing (A11Y-19).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<SpeechRoute>,
    /// Duck instead of pausing for speech during playback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duck_instead_of_pause: Option<bool>,
    /// The speaking rate sequencing assumes, in words per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_wpm: Option<f64>,
}

fn invalid(detail: impl std::fmt::Display) -> IpcError {
    IpcError::new(
        &codes::PROFILE_INVALID,
        format!("The profile is not valid: {detail}."),
    )
}

/// Reads a profile document: the format, the version (a newer major is refused, a newer
/// minor loses what this version does not know, with a warning), the schema and the
/// value checks. Returns the profile and its warnings.
///
/// # Errors
///
/// `SNR-APP-080` for anything that is not a valid profile, `SNR-APP-081` for a newer major
/// version.
pub fn parse(bytes: &[u8]) -> Result<(Profile, Vec<IpcError>), IpcError> {
    if bytes.len() > MAX_BYTES {
        return Err(invalid("it is larger than 1 MiB"));
    }
    let mut value: Value = serde_json::from_slice(bytes).map_err(|_| invalid("it is not JSON"))?;
    if value["format"] != FORMAT {
        return Err(invalid("it is not a SONARITHM profile"));
    }
    let version = value["formatVersion"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let (major, minor) = version
        .split_once('.')
        .and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?)))
        .ok_or_else(|| invalid("the format version is not MAJOR.MINOR"))?;
    if major > FORMAT_VERSION.0 {
        return Err(IpcError::new(
            &codes::PROFILE_MAJOR,
            format!(
                "The profile uses format version {version}, made by a newer SONARITHM; this \
                 version reads format {}.",
                FORMAT_VERSION.0
            ),
        ));
    }
    if major < FORMAT_VERSION.0 {
        return Err(invalid(
            "its format version is older than any this build reads",
        ));
    }
    let mut warnings = Vec::new();
    if minor > FORMAT_VERSION.1 {
        let schema = schema_value();
        if strip_unknown(&mut value, &schema, &schema) {
            warnings.push(IpcError::new(
                &codes::PROFILE_NEWER,
                format!(
                    "The profile uses format version {version}; preferences this version does \
                     not know were left out, and saving drops them."
                ),
            ));
        }
        value["formatVersion"] = json!(format!("{}.{}", FORMAT_VERSION.0, FORMAT_VERSION.1));
    }
    let profile: Profile = serde_json::from_value(value).map_err(invalid)?;
    validate(&profile)?;
    Ok((profile, warnings))
}

/// The value checks of ADR-0020 §1: ranges, names, and no paths.
///
/// # Errors
///
/// `SNR-APP-080` naming the first problem.
pub fn validate(profile: &Profile) -> Result<(), IpcError> {
    let name = profile.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(invalid("the name must have 1 to 100 characters"));
    }
    if let Some(PitchRange { hz: [low, high] }) = profile.sound.pitch_range {
        let audible = |hz: f64| hz.is_finite() && (20.0..=20_000.0).contains(&hz);
        if !(audible(low) && audible(high) && low < high) {
            return Err(invalid("the pitch range must rise within 20 to 20000 Hz"));
        }
    }
    if let Some(Pace { points_per_second }) = profile.sound.pace
        && !(points_per_second.is_finite() && points_per_second > 0.0 && points_per_second <= 100.0)
    {
        return Err(invalid(
            "the pace must be above 0 and at most 100 points per second",
        ));
    }
    if let Some(timbre) = &profile.sound.timbre {
        let ok = !timbre.is_empty()
            && timbre.len() <= 40
            && timbre
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !ok {
            return Err(invalid("a timbre is a short lowercase instrument name"));
        }
    }
    if let Some(rate) = profile.interaction.speech.and_then(|s| s.rate_wpm) {
        let (low, high) = crate::settings::SPEECH_RATE_RANGE;
        if !(low..=high).contains(&rate) {
            return Err(invalid(
                "the speaking rate must be 30 to 1000 words per minute",
            ));
        }
    }
    if profile.interaction.keymap.len() > 1_000 {
        return Err(invalid("the keymap has more than 1000 entries"));
    }
    let value = serde_json::to_value(profile).map_err(invalid)?;
    if has_path(&value) {
        return Err(invalid("a profile never holds a file path"));
    }
    Ok(())
}

/// Whether any string or key looks like an absolute path (ADR-0012, ADR-0020 §1).
fn has_path(value: &Value) -> bool {
    let absolute = |text: &str| {
        text.starts_with('/')
            || text.starts_with('\\')
            || text.get(1..3).is_some_and(|s| s == ":\\" || s == ":/")
    };
    match value {
        Value::String(text) => absolute(text),
        Value::Array(items) => items.iter().any(has_path),
        Value::Object(map) => map.iter().any(|(key, v)| absolute(key) || has_path(v)),
        _ => false,
    }
}

/// The canonical bytes of `profile` (RFC 8785), with a final newline.
///
/// # Errors
///
/// `SNR-APP-080` if it cannot be written, which only an invalid value causes.
pub fn write(profile: &Profile) -> Result<Vec<u8>, IpcError> {
    let mut text = sonarithm_spec::canonical::to_canonical(profile).map_err(invalid)?;
    text.push('\n');
    Ok(text.into_bytes())
}

/// The built-in presets (sound-design §13): ID and profile. Their values are proposals
/// pending co-design in B07 (ADR-0020 §2), so every name says so in the list reply.
#[must_use]
pub fn presets() -> Vec<(String, Profile)> {
    [
        (
            "blv-chart-reading",
            include_str!("../presets/blv-chart-reading.json"),
        ),
        ("analyst", include_str!("../presets/analyst.json")),
        ("classroom", include_str!("../presets/classroom.json")),
        ("presentation", include_str!("../presets/presentation.json")),
        ("monitoring", include_str!("../presets/monitoring.json")),
    ]
    .into_iter()
    .filter_map(|(id, text)| {
        let (profile, _) = parse(text.as_bytes()).ok()?;
        Some((format!("{PRESET_PREFIX}{id}"), profile))
    })
    .collect()
}

/// Where a setting's value came from (12 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Layer {
    /// The built-in default.
    Default,
    /// The user's settings.
    Settings,
    /// The active profile.
    Profile,
    /// The project's profile.
    Project,
    /// The spec.
    Spec,
    /// An explicit action in this session.
    Session,
}

/// One setting in the effective-settings view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EffectiveSetting {
    /// Its key, such as `interaction.verbosity` or `sound.polarity.temp`.
    pub key: String,
    /// Its value (`null` when the compiler's default applies).
    #[ts(type = "unknown")]
    pub value: Value,
    /// The layer it came from.
    pub layer: Layer,
}

/// The layers below the spec.
#[derive(Clone, Copy, Debug)]
pub struct Layers<'a> {
    /// The user's settings.
    pub settings: &'a Settings,
    /// The active profile.
    pub profile: Option<&'a Profile>,
    /// The project's profile.
    pub project: Option<&'a Profile>,
    /// Explicit actions in this session.
    pub session: &'a InteractionPrefs,
}

/// The effective interaction preferences.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Interaction {
    /// Verbosity.
    pub verbosity: Verbosity,
    /// The part order, if any layer sets one.
    pub part_order: Option<PartOrder>,
    /// Modalities: sonification, speech, braille, visual.
    pub modalities: [bool; 4],
    /// The speech route.
    pub route: SpeechRoute,
    /// Duck instead of pause.
    pub duck_instead_of_pause: bool,
    /// The speaking rate in words per minute.
    pub rate_wpm: f64,
    /// The keymap overrides.
    pub keymap: BTreeMap<String, String>,
    /// Every value with its layer.
    pub view: Vec<EffectiveSetting>,
}

/// Picks, field by field, the highest layer that sets a value.
struct Pick<'a> {
    view: &'a mut Vec<EffectiveSetting>,
}

impl Pick<'_> {
    fn one<T: Serialize + PartialEq + Clone>(
        &mut self,
        key: &str,
        default: T,
        settings: T,
        upper: [(Layer, Option<T>); 3],
    ) -> T {
        let (mut value, mut layer) = if settings == default {
            (default, Layer::Default)
        } else {
            (settings, Layer::Settings)
        };
        for (upper_layer, upper_value) in upper {
            if let Some(v) = upper_value {
                value = v;
                layer = upper_layer;
            }
        }
        self.view.push(EffectiveSetting {
            key: key.to_owned(),
            value: serde_json::to_value(&value).unwrap_or(Value::Null),
            layer,
        });
        value
    }
}

fn as_value<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

/// The effective interaction preferences of `layers` (12 §3, ADR-0020 §3).
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one pick per setting, in the order of the effective-settings view"
)]
pub fn effective(layers: &Layers<'_>) -> Interaction {
    let defaults = Settings::default();
    let interaction = |p: Option<&Profile>| p.map(|p| p.interaction.clone());
    let (profile, project) = (interaction(layers.profile), interaction(layers.project));
    let session = Some(layers.session.clone());
    let upper = |f: &dyn Fn(&InteractionPrefs) -> Option<Value>| {
        [
            (Layer::Profile, profile.as_ref().and_then(f)),
            (Layer::Project, project.as_ref().and_then(f)),
            (Layer::Session, session.as_ref().and_then(f)),
        ]
    };
    let mut view = Vec::new();
    let mut pick = Pick { view: &mut view };
    let verbosity = pick.one(
        "interaction.verbosity",
        as_value(defaults.verbosity),
        as_value(layers.settings.verbosity),
        upper(&|p| p.verbosity.map(as_value)),
    );
    let part_order = pick.one(
        "interaction.partOrder",
        Value::Null,
        Value::Null,
        upper(&|p| p.part_order.as_ref().map(as_value)),
    );
    let modality = |get: fn(&ModalityPrefs) -> Option<bool>| {
        move |p: &InteractionPrefs| p.modalities.as_ref().and_then(get).map(as_value)
    };
    let d = &defaults.modalities;
    let s = &layers.settings.modalities;
    let sonification = pick.one(
        "interaction.modalities.sonification",
        as_value(d.sonification),
        as_value(s.sonification),
        upper(&modality(|m| m.sonification)),
    );
    let speech = pick.one(
        "interaction.modalities.speech",
        as_value(d.speech),
        as_value(s.speech),
        upper(&modality(|m| m.speech)),
    );
    let braille = pick.one(
        "interaction.modalities.braille",
        as_value(d.braille),
        as_value(s.braille),
        upper(&modality(|m| m.braille)),
    );
    let visual = pick.one(
        "interaction.modalities.visual",
        as_value(d.visual),
        as_value(s.visual),
        upper(&modality(|m| m.visual)),
    );
    let ds = &defaults.speech;
    let ss = &layers.settings.speech;
    let route = pick.one(
        "interaction.speech.route",
        as_value(ds.route),
        as_value(ss.route),
        upper(&|p| p.speech.and_then(|s| s.route).map(as_value)),
    );
    let duck = pick.one(
        "interaction.speech.duckInsteadOfPause",
        as_value(ds.duck_instead_of_pause),
        as_value(ss.duck_instead_of_pause),
        upper(&|p| p.speech.and_then(|s| s.duck_instead_of_pause).map(as_value)),
    );
    let rate = pick.one(
        "interaction.speech.rateWpm",
        as_value(ds.rate_wpm),
        as_value(ss.rate_wpm),
        upper(&|p| p.speech.and_then(|s| s.rate_wpm).map(as_value)),
    );
    // The keymap merges key by key.
    let mut keymap: BTreeMap<String, (String, Layer)> = layers
        .settings
        .keymap
        .iter()
        .map(|(k, v)| (k.clone(), (v.clone(), Layer::Settings)))
        .collect();
    for (layer, prefs) in [
        (Layer::Profile, profile.as_ref()),
        (Layer::Project, project.as_ref()),
        (Layer::Session, session.as_ref()),
    ] {
        for (command, keys) in prefs.map(|p| &p.keymap).into_iter().flatten() {
            keymap.insert(command.clone(), (keys.clone(), layer));
        }
    }
    for (command, (keys, layer)) in &keymap {
        view.push(EffectiveSetting {
            key: format!("interaction.keymap.{command}"),
            value: json!(keys),
            layer: *layer,
        });
    }
    let flag = |v: &Value| v.as_bool().unwrap_or(true);
    Interaction {
        verbosity: serde_json::from_value(verbosity).unwrap_or_default(),
        part_order: serde_json::from_value(part_order).ok(),
        modalities: [
            flag(&sonification),
            flag(&speech),
            flag(&braille),
            flag(&visual),
        ],
        route: serde_json::from_value(route).unwrap_or_default(),
        duck_instead_of_pause: duck.as_bool().unwrap_or(false),
        rate_wpm: rate.as_f64().unwrap_or(ds.rate_wpm),
        keymap: keymap.into_iter().map(|(k, (v, _))| (k, v)).collect(),
        view,
    }
}

/// The sound preferences of the profile and project layers, the project's winning.
#[must_use]
pub fn sound(profile: Option<&Profile>, project: Option<&Profile>) -> Vec<(Layer, SoundPrefs)> {
    [(Layer::Profile, profile), (Layer::Project, project)]
        .into_iter()
        .filter_map(|(layer, p)| p.map(|p| (layer, p.sound.clone())))
        .collect()
}

/// The spec actually compiled: `spec` with what it leaves unset filled from the sound
/// layers (higher layers last), and every polarity a profile decides written as an
/// explicit value (ADR-0020 §3). Also the sound part of the effective-settings view.
///
/// # Errors
///
/// `SNR-APP-080` only if `spec` is not a JSON object.
pub fn effective_spec(
    spec: &str,
    layers: &[(Layer, SoundPrefs)],
    data: &BTreeMap<String, DataBundle>,
) -> Result<(String, Vec<EffectiveSetting>), IpcError> {
    let mut value: Value = serde_json::from_str(spec).map_err(invalid)?;
    if !value.is_object() {
        return Err(invalid("the spec is not a JSON object"));
    }
    let mut view = Vec::new();
    let mut fill = |value: &mut Value, pointer: &str, key: &str, new: Option<(Layer, Value)>| {
        let existing = value.pointer(pointer).cloned();
        let (layer, chosen) = match (existing, new) {
            (Some(spec), _) => (Layer::Spec, spec),
            (None, Some((layer, new))) => {
                set(value, pointer, new.clone());
                (layer, new)
            }
            (None, None) => (Layer::Default, Value::Null),
        };
        view.push(EffectiveSetting {
            key: key.to_owned(),
            value: chosen,
            layer,
        });
    };
    let top = |get: &dyn Fn(&SoundPrefs) -> Option<Value>| {
        layers
            .iter()
            .filter_map(|(layer, prefs)| get(prefs).map(|v| (*layer, v)))
            .next_back()
    };
    let tracks = value["tracks"].as_array().map_or(0, Vec::len);
    for index in 0..tracks {
        let id = value["tracks"][index]["id"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if value["tracks"][index]["channels"]["pitch"].is_object() {
            fill(
                &mut value,
                &format!("/tracks/{index}/channels/pitch/range"),
                &format!("sound.pitchRange.{id}"),
                top(&|s| s.pitch_range.map(|r| json!({ "hz": r.hz }))),
            );
        }
        fill(
            &mut value,
            &format!("/tracks/{index}/instrument"),
            &format!("sound.timbre.{id}"),
            top(&|s| s.timbre.clone().map(Value::String)),
        );
    }
    fill(
        &mut value,
        "/substrate/time/pace",
        "sound.pace",
        top(&|s| {
            s.pace
                .map(|p| json!({ "pointsPerSecond": p.points_per_second }))
        }),
    );
    let context = |get: fn(&ContextPrefs) -> Option<bool>, on: Value, off: Value| {
        top(&|s| {
            s.context
                .as_ref()
                .and_then(get)
                .map(|b| if b { on.clone() } else { off.clone() })
        })
    };
    fill(
        &mut value,
        "/context/ticks/enabled",
        "sound.context.ticks",
        context(|c| c.ticks, json!(true), json!(false)),
    );
    fill(
        &mut value,
        "/context/reference/kind",
        "sound.context.referenceTone",
        context(
            |c| c.reference_tone,
            json!("dynamic-extreme"),
            json!("none"),
        ),
    );
    fill(
        &mut value,
        "/context/legend/speak",
        "sound.context.spokenLegend",
        context(|c| c.spoken_legend, json!("before-play"), json!("off")),
    );
    polarities(&mut value, layers, data, &mut view);
    let text = sonarithm_spec::canonical::to_canonical(&value).map_err(invalid)?;
    Ok((text, view))
}

/// Writes every polarity the profile layers decide as an explicit value (ADR-0020 §3):
/// NUMISONA's resolution (spec explicit > profile concept > concept table > generic), then
/// the one-key flip.
fn polarities(
    value: &mut Value,
    layers: &[(Layer, SoundPrefs)],
    data: &BTreeMap<String, DataBundle>,
    view: &mut Vec<EffectiveSetting>,
) {
    let mut profile = PolarityProfile::default();
    let mut decided_by: BTreeMap<String, Layer> = BTreeMap::new();
    let mut flip_layer: BTreeMap<String, Layer> = BTreeMap::new();
    for (layer, prefs) in layers {
        let Some(polarity) = &prefs.polarity else {
            continue;
        };
        for (concept, direction) in &polarity.concepts {
            let direction = match direction {
                Direction::Positive => Polarity::Positive,
                Direction::Negative => Polarity::Negative,
            };
            profile.concepts.insert(concept.clone(), direction);
            decided_by.insert(concept.clone(), *layer);
        }
        for field in &polarity.flipped_fields {
            profile.flipped_fields.insert(field.clone());
            flip_layer.insert(field.clone(), *layer);
        }
    }
    let Ok(table) = ConceptTable::builtin() else {
        return;
    };
    let source = value["data"]["source"]["id"].as_str().map(str::to_owned);
    let tracks = value["tracks"].as_array().map_or(0, Vec::len);
    for index in 0..tracks {
        let pitch = &value["tracks"][index]["channels"]["pitch"];
        let Some(field) = pitch["field"].as_str().map(str::to_owned) else {
            continue;
        };
        let choice = match pitch["polarity"].as_str() {
            Some("positive") => PolarityChoice::Positive,
            Some("negative") => PolarityChoice::Negative,
            _ => PolarityChoice::Auto,
        };
        let spec_concept = value["data"]["fields"][&field]["concept"]
            .as_str()
            .map(str::to_owned);
        let data_concept = source
            .as_deref()
            .and_then(|id| data.get(id))
            .and_then(DataBundle::primary)
            .and_then(|table| table.field_meta(&field).ok())
            .and_then(|meta| meta.concept.map(|c| c.as_str().to_owned()));
        let concept = spec_concept
            .as_deref()
            .map(|c| (c, ConceptOrigin::Spec))
            .or_else(|| data_concept.as_deref().map(|c| (c, ConceptOrigin::Data)));
        let profile_concept = concept
            .and_then(|(c, _)| decided_by.get(c))
            .filter(|_| choice == PolarityChoice::Auto);
        let flipped = flip_layer.get(&field);
        let key = format!("sound.polarity.{field}");
        if profile_concept.is_none() && flipped.is_none() {
            let (layer, shown) = match choice {
                PolarityChoice::Auto => (Layer::Default, Value::Null),
                _ => (Layer::Spec, pitch["polarity"].clone()),
            };
            view.push(EffectiveSetting {
                key,
                value: shown,
                layer,
            });
            continue;
        }
        let request = PolarityRequest {
            field: &field,
            concept,
            choice,
        };
        let (resolved, _) = resolve(table, &profile, &request, &SpecPointer::root());
        let explicit = match resolved.polarity {
            Polarity::Positive => "positive",
            Polarity::Negative => "negative",
        };
        value["tracks"][index]["channels"]["pitch"]["polarity"] = json!(explicit);
        let layer = flipped
            .or(profile_concept)
            .copied()
            .unwrap_or(Layer::Profile);
        view.push(EffectiveSetting {
            key,
            value: json!(explicit),
            layer,
        });
    }
}

/// Sets `pointer` in `value`, creating objects on the way.
fn set(value: &mut Value, pointer: &str, new: Value) {
    let mut at = value;
    let parts: Vec<&str> = pointer.trim_start_matches('/').split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        let last = i + 1 == parts.len();
        let next = match at {
            Value::Array(items) => {
                match part.parse::<usize>().ok().and_then(|n| items.get_mut(n)) {
                    Some(item) => item,
                    None => return,
                }
            }
            Value::Object(map) => {
                if last {
                    map.insert((*part).to_owned(), new);
                    return;
                }
                map.entry((*part).to_owned())
                    .or_insert_with(|| Value::Object(Map::new()))
            }
            _ => return,
        };
        at = next;
    }
}

/// The JSON Schema (draft 2020-12) of [`Profile`], pretty-printed with a trailing newline.
#[must_use]
pub fn json_schema() -> String {
    let mut text = serde_json::to_string_pretty(&schema_value()).unwrap_or_default();
    text.push('\n');
    text
}

fn schema_value() -> Value {
    let generator = schemars::generate::SchemaSettings::draft2020_12()
        .with(|settings| {
            settings
                .transforms
                .push(Box::new(schemars::transform::RecursiveTransform(
                    not_nullable,
                )));
        })
        .into_generator();
    let mut schema = generator.into_root_schema_for::<Profile>();
    schema.insert(
        "$comment".to_owned(),
        "Generated by `cargo xtask codegen` from the sonarithm-app profile types; do not edit."
            .into(),
    );
    schema.to_value()
}

/// Optional keys are optional, not nullable (as the spec schema, A03-T06).
fn not_nullable(schema: &mut schemars::Schema) {
    let Some(object) = schema.as_object_mut() else {
        return;
    };
    if let Some(Value::Array(types)) = object.get_mut("type") {
        types.retain(|t| t != "null");
        if let [single] = types.as_slice() {
            let single = single.clone();
            object.insert("type".to_owned(), single);
        }
    }
    let null = json!({ "type": "null" });
    let other = match object.get("anyOf") {
        Some(Value::Array(any)) if any.len() == 2 && any.contains(&null) => {
            any.iter().find(|branch| **branch != null).cloned()
        }
        _ => None,
    };
    if let Some(Value::Object(branch)) = other {
        object.remove("anyOf");
        for (key, value) in branch {
            object.entry(key).or_insert(value);
        }
    }
}

/// Removes keys the schema does not know, recursively; returns whether any went.
fn strip_unknown(value: &mut Value, schema: &Value, root: &Value) -> bool {
    let schema = match schema.get("$ref").and_then(Value::as_str) {
        Some(reference) => reference
            .strip_prefix("#/$defs/")
            .and_then(|name| root["$defs"].get(name))
            .unwrap_or(&Value::Null),
        None => schema,
    };
    let mut removed = false;
    match value {
        Value::Object(map) => {
            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                let before = map.len();
                map.retain(|key, _| properties.contains_key(key));
                removed |= map.len() != before;
                for (key, child) in map.iter_mut() {
                    if let Some(child_schema) = properties.get(key) {
                        removed |= strip_unknown(child, child_schema, root);
                    }
                }
            } else if let Some(values) =
                schema.get("additionalProperties").filter(|v| v.is_object())
            {
                for child in map.values_mut() {
                    removed |= strip_unknown(child, values, root);
                }
            }
        }
        Value::Array(items) => {
            if let Some(item) = schema.get("items") {
                for child in items {
                    removed |= strip_unknown(child, item, root);
                }
            }
        }
        _ => {}
    }
    removed
}

#[cfg(test)]
#[path = "profile/tests.rs"]
mod tests;
