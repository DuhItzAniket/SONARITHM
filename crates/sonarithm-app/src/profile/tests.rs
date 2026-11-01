// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Profile documents, layers and the effective spec (ADR-0020; B03-T08).

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::*;
use crate::settings::Settings;

fn doc(extra: &Value) -> Vec<u8> {
    let mut value = json!({ "format": FORMAT, "formatVersion": "1.0", "name": "Mine" });
    if let (Some(base), Some(extra)) = (value.as_object_mut(), extra.as_object()) {
        for (key, v) in extra {
            base.insert(key.clone(), v.clone());
        }
    }
    serde_json::to_vec(&value).unwrap_or_default()
}

fn code(result: Result<(Profile, Vec<IpcError>), IpcError>) -> String {
    match result {
        Ok(_) => "ok".to_owned(),
        Err(err) => err.code,
    }
}

#[test]
fn every_preset_is_a_valid_profile() {
    let presets = presets();
    let ids: Vec<&str> = presets.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "preset:blv-chart-reading",
            "preset:analyst",
            "preset:classroom",
            "preset:presentation",
            "preset:monitoring"
        ]
    );
    for (_, profile) in &presets {
        assert_eq!(validate(profile), Ok(()));
    }
}

#[test]
fn a_profile_round_trips_through_its_canonical_bytes() {
    let (profile, warnings) = parse(&doc(&json!({
        "sound": { "pitchRange": { "hz": [300, 1200] }, "polarity": { "flippedFields": ["temp"] } },
        "interaction": { "keymap": { "nav.whereAmI": "W" } }
    })))
    .unwrap();
    assert_eq!(warnings, Vec::new());
    let bytes = write(&profile).unwrap_or_default();
    let (again, _) = parse(&bytes).unwrap();
    assert_eq!(again, profile);
    assert_eq!(write(&again).unwrap_or_default(), bytes, "byte-identical");
}

#[test]
fn bad_profiles_are_refused() {
    assert_eq!(code(parse(b"not json")), "SNR-APP-080");
    assert_eq!(code(parse(br#"{"format":"x"}"#)), "SNR-APP-080");
    let newer_major = br#"{"format":"sonarithm-profile","formatVersion":"2.0","name":"x"}"#;
    assert_eq!(code(parse(newer_major)), "SNR-APP-081");
    for bad in [
        json!({ "sound": { "pitchRange": { "hz": [1400, 400] } } }),
        json!({ "sound": { "pace": { "pointsPerSecond": 0 } } }),
        json!({ "sound": { "timbre": "Grand Piano" } }),
        json!({ "interaction": { "speech": { "rateWpm": 5 } } }),
        json!({ "interaction": { "keymap": { "nav.open": "C:\\Users\\me" } } }),
        json!({ "unknown": true }),
        json!({ "name": "" }),
    ] {
        assert_eq!(code(parse(&doc(&bad))), "SNR-APP-080", "{bad}");
    }
    let mut big = doc(&json!({}));
    big.resize(MAX_BYTES + 1, b' ');
    assert_eq!(code(parse(&big)), "SNR-APP-080");
}

#[test]
fn a_newer_minor_version_drops_what_it_does_not_know() {
    let newer = doc(&json!({
        "formatVersion": "1.4",
        "haptics": { "on": true },
        "sound": { "timbre": "mallet", "shimmer": 3 },
        "interaction": { "modalities": { "speech": false, "smell": true } }
    }));
    let (profile, warnings) = parse(&newer).unwrap();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code, "SNR-APP-082");
    assert_eq!(profile.format_version, "1.0");
    assert_eq!(profile.sound.timbre.as_deref(), Some("mallet"));
    assert_eq!(
        profile.interaction.modalities.and_then(|m| m.speech),
        Some(false)
    );
}

fn with(interaction: &Value) -> Profile {
    let (profile, _) = parse(&doc(&json!({ "interaction": interaction }))).unwrap();
    profile
}

/// 12 §3: built-in defaults < user settings < active profile < project profile < explicit
/// UI action, field by field.
#[test]
fn interaction_layers_follow_the_precedence_of_12_3() {
    let mut settings = Settings::default();
    let none = InteractionPrefs::default();
    let layers = |settings: &Settings, profile, project, session| {
        effective(&Layers {
            settings,
            profile,
            project,
            session,
        })
    };
    let base = layers(&settings, None, None, &none);
    assert_eq!(base.verbosity, Verbosity::Terse);
    let layer = |e: &Interaction, key: &str| {
        e.view
            .iter()
            .find(|s| s.key == key)
            .map_or(Layer::Default, |s| s.layer)
    };
    assert_eq!(layer(&base, "interaction.verbosity"), Layer::Default);

    settings.verbosity = Verbosity::Verbose;
    let from_settings = layers(&settings, None, None, &none);
    assert_eq!(from_settings.verbosity, Verbosity::Verbose);
    assert_eq!(
        layer(&from_settings, "interaction.verbosity"),
        Layer::Settings
    );

    let profile = with(&json!({ "verbosity": "terse", "modalities": { "braille": false } }));
    let from_profile = layers(&settings, Some(&profile), None, &none);
    assert_eq!(from_profile.verbosity, Verbosity::Terse);
    assert_eq!(
        layer(&from_profile, "interaction.verbosity"),
        Layer::Profile
    );
    assert_eq!(from_profile.modalities, [true, true, false, true]);

    let project = with(&json!({ "verbosity": "verbose" }));
    let from_project = layers(&settings, Some(&profile), Some(&project), &none);
    assert_eq!(from_project.verbosity, Verbosity::Verbose);
    assert_eq!(
        layer(&from_project, "interaction.verbosity"),
        Layer::Project
    );
    assert_eq!(
        layer(&from_project, "interaction.modalities.braille"),
        Layer::Profile,
        "fields a higher layer leaves unset keep the lower layer's value"
    );

    let session: InteractionPrefs = serde_json::from_value(json!({
        "verbosity": "terse", "keymap": { "nav.whereAmI": "Q" }
    }))
    .unwrap_or_default();
    let from_session = layers(&settings, Some(&profile), Some(&project), &session);
    assert_eq!(from_session.verbosity, Verbosity::Terse);
    assert_eq!(
        layer(&from_session, "interaction.verbosity"),
        Layer::Session
    );
    assert_eq!(from_session.keymap["nav.whereAmI"], "Q");
}

/// Keyboard §13 (B03-T09): a project's profile never sets keys, since a shared file must not
/// rebind someone's keyboard; the user's own profile and settings do.
#[test]
fn keyboard_13_a_project_profile_never_sets_keys() {
    let mut settings = Settings::default();
    settings
        .keymap
        .insert("nav.whereAmI".to_owned(), "Ctrl+W".to_owned());
    let profile = with(&json!({ "keymap": { "nav.repeat": "Ctrl+R" } }));
    let project = with(&json!({ "keymap": { "nav.whereAmI": "K", "nav.next": "L" } }));
    let none = InteractionPrefs::default();
    let merged = effective(&Layers {
        settings: &settings,
        profile: Some(&profile),
        project: Some(&project),
        session: &none,
    });
    assert_eq!(
        merged.keymap.get("nav.whereAmI").map(String::as_str),
        Some("Ctrl+W")
    );
    assert_eq!(
        merged.keymap.get("nav.repeat").map(String::as_str),
        Some("Ctrl+R")
    );
    assert_eq!(merged.keymap.get("nav.next"), None);
    assert!(
        merged
            .view
            .iter()
            .all(|s| !(s.key.starts_with("interaction.keymap.") && s.layer == Layer::Project))
    );
}

const SPEC: &str = r#"{
    "specVersion": "0.1",
    "data": { "source": { "kind": "dataset", "id": "d1" } },
    "tracks": [{ "id": "t", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
}"#;

fn sound_layers(sound: &Value) -> Vec<(Layer, SoundPrefs)> {
    vec![(
        Layer::Profile,
        serde_json::from_value(sound.clone()).unwrap_or_default(),
    )]
}

fn effective_value(spec: &str, layers: &[(Layer, SoundPrefs)]) -> (Value, Vec<EffectiveSetting>) {
    let (text, view) = effective_spec(spec, layers, &BTreeMap::new()).unwrap();
    (serde_json::from_str(&text).unwrap_or_default(), view)
}

/// SDR-38: each sound preference fills only what the spec leaves unset.
#[test]
fn sdr_38_sound_preferences_fill_what_the_spec_leaves_unset() {
    let layers = sound_layers(&json!({
        "pitchRange": { "hz": [300, 1200] },
        "timbre": "square",
        "pace": { "pointsPerSecond": 2 },
        "context": { "ticks": false, "referenceTone": false, "spokenLegend": false }
    }));
    let (value, view) = effective_value(SPEC, &layers);
    assert_eq!(
        value["tracks"][0]["channels"]["pitch"]["range"],
        json!({ "hz": [300, 1200] })
    );
    assert_eq!(value["tracks"][0]["instrument"], "square");
    assert_eq!(
        value["substrate"]["time"]["pace"],
        json!({ "pointsPerSecond": 2 })
    );
    assert_eq!(value["context"]["ticks"]["enabled"], false);
    assert_eq!(value["context"]["reference"]["kind"], "none");
    assert_eq!(value["context"]["legend"]["speak"], "off");
    assert!(
        view.iter()
            .any(|s| s.key == "sound.timbre.t" && s.layer == Layer::Profile)
    );

    // A spec value always wins.
    let explicit = SPEC.replace(
        r#""mark": "tone""#,
        r#""mark": "tone", "instrument": "mallet""#,
    );
    let (value, view) = effective_value(&explicit, &layers);
    assert_eq!(value["tracks"][0]["instrument"], "mallet");
    assert!(
        view.iter()
            .any(|s| s.key == "sound.timbre.t" && s.layer == Layer::Spec)
    );
}

/// SDR-15 and ADR-0020 §3: a polarity a profile decides is written into the spec as an
/// explicit value; the flip inverts even an explicit spec value.
#[test]
fn sdr_15_profile_polarity_becomes_an_explicit_spec_value() {
    let flipped = sound_layers(&json!({ "polarity": { "flippedFields": ["temp"] } }));
    let (value, view) = effective_value(SPEC, &flipped);
    assert_eq!(
        value["tracks"][0]["channels"]["pitch"]["polarity"],
        "negative"
    );
    assert!(
        view.iter()
            .any(|s| s.key == "sound.polarity.temp" && s.layer == Layer::Profile)
    );
    let explicit = SPEC.replace(
        r#""field": "temp""#,
        r#""field": "temp", "polarity": "negative""#,
    );
    let (value, _) = effective_value(&explicit, &flipped);
    assert_eq!(
        value["tracks"][0]["channels"]["pitch"]["polarity"],
        "positive"
    );

    let with_concept = SPEC.replace(
        r#""data": { "source": { "kind": "dataset", "id": "d1" } }"#,
        r#""data": { "source": { "kind": "dataset", "id": "d1" }, "fields": { "temp": { "concept": "cost" } } }"#,
    );
    let by_concept = sound_layers(&json!({ "polarity": { "concepts": { "cost": "negative" } } }));
    let (value, _) = effective_value(&with_concept, &by_concept);
    assert_eq!(
        value["tracks"][0]["channels"]["pitch"]["polarity"],
        "negative"
    );

    // No profile decision: the spec is left for NUMISONA.
    let (value, _) = effective_value(SPEC, &[]);
    assert_eq!(
        value["tracks"][0]["channels"]["pitch"].get("polarity"),
        None
    );
}

#[test]
fn the_project_layer_wins_over_the_profile() {
    let layers = vec![
        (
            Layer::Profile,
            serde_json::from_value(json!({ "timbre": "square" })).unwrap_or_default(),
        ),
        (
            Layer::Project,
            serde_json::from_value(json!({ "timbre": "sine" })).unwrap_or_default(),
        ),
    ];
    let (value, view) = effective_value(SPEC, &layers);
    assert_eq!(value["tracks"][0]["instrument"], "sine");
    assert!(
        view.iter()
            .any(|s| s.key == "sound.timbre.t" && s.layer == Layer::Project)
    );
}

#[test]
fn the_schema_describes_the_document() {
    let schema: Value = serde_json::from_str(&json_schema()).unwrap_or_default();
    assert_eq!(schema["title"], "SONARITHM profile");
    assert_eq!(schema["additionalProperties"], false);
    for key in ["format", "formatVersion", "name"] {
        assert!(
            schema["required"]
                .as_array()
                .is_some_and(|r| r.iter().any(|k| k == key)),
            "{key}"
        );
    }
}
