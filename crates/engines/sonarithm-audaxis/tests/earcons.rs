// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! B03-T06: the earcon catalogue with a text equivalent for every earcon (A11Y-08,
//! A11Y-24), its defaults, and the gap earcon for `missing: "earcon"`.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use std::collections::BTreeSet;

use serde_json::json;
use sonarithm_audaxis::earcons::{CATALOGUE, EarconId, EarconSettings, for_outcome};
use sonarithm_audaxis::nav::{Cursor, Verb, XTarget};
use sonarithm_core::score::EarconKind;
use sonarithm_i18n::{Args, Localizer, PSEUDO_LOCALE, check_pseudo};
use support::tree_with;

/// Sample values for every argument an earcon text takes.
fn sample(arg: &str) -> &'static str {
    match arg {
        "edge" => "last",
        "reason" => "one-series",
        "kind" => "segment",
        "state" => "on",
        _ => "7",
    }
}

/// B03-T06 acceptance: a test enumerates the catalogue and finds a text equivalent for
/// every earcon, in English and in the pseudo-locale (A11Y-08, A11Y-24, ED-05).
#[test]
fn a11y_08_every_earcon_has_a_text_equivalent() {
    let english = Localizer::new("en-US").unwrap();
    let pseudo = Localizer::new(PSEUDO_LOCALE).unwrap();
    let mut codes = BTreeSet::new();
    for info in &CATALOGUE {
        assert!(codes.insert(info.code), "{} is listed twice", info.code);
        assert_eq!(info.id.info().code, info.code);
        assert!(info.meaning.len() > 3, "{}", info.code);
        let args = info
            .args
            .iter()
            .fold(Args::new(), |args, arg| args.text(*arg, sample(arg)));
        let text = english
            .format(info.text, &args)
            .unwrap_or_else(|e| panic!("{}: {e}", info.code));
        assert!(!text.trim().is_empty(), "{}", info.code);
        let marked = pseudo.format(info.text, &args).unwrap();
        check_pseudo(&marked).unwrap_or_else(|e| panic!("{}: {e}", info.code));
    }
    assert_eq!(codes.len(), 9, "keyboard §11's eight earcons and E-GAP");
}

/// Keyboard §11: E-ALARM and E-LEVEL are off by default, every other earcon on.
#[test]
fn a11y_08_alarm_and_level_are_off_by_default() {
    for info in &CATALOGUE {
        let off = matches!(info.id, EarconId::Alarm | EarconId::Level);
        assert_eq!(info.default_on, !off, "{}", info.code);
    }
}

/// Every earcon kind a Score can contain has a catalogue entry.
#[test]
fn every_score_earcon_kind_is_in_the_catalogue() {
    for kind in [
        EarconKind::Boundary,
        EarconKind::Landmark,
        EarconKind::Threshold,
        EarconKind::Stream,
        EarconKind::Alarm,
    ] {
        assert!(
            CATALOGUE.iter().any(|info| info.compiled == Some(kind)),
            "{kind:?}"
        );
    }
}

fn spec(missing: &str) -> serde_json::Value {
    json!({
        "specVersion": "0.1",
        "data": {"source": {"kind": "inline", "values": [
            {"x": 0, "y": 1.0}, {"x": 1, "y": null}, {"x": 2, "y": 3.0}
        ]}, "fields": {"x": {"role": "x"}}, "missing": missing},
        "tracks": [{"id": "y", "mark": "tone", "channels": {"pitch": {"field": "y"}}}]
    })
}

/// 03 §5, B03-T06: with `missing: "earcon"`, landing on a missing value plays E-GAP; with
/// the default, it does not.
#[test]
fn a11y_08_gap_earcon_marks_missing_values_when_asked() {
    for (missing, expected) in [("earcon", true), ("silence", false)] {
        let tree = tree_with(&spec(missing), &[]);
        let settings = EarconSettings::for_tree(&tree);
        assert_eq!(settings.gap, expected);
        let mut cursor = Cursor::at(&tree, tree.points(tree.series().next().unwrap())[0]).unwrap();
        let outcome = cursor.apply(&tree, &Verb::GoTo(XTarget::Number(1.0)));
        let gap = for_outcome(&tree, &outcome, &settings)
            .iter()
            .any(|cue| cue.id == EarconId::Gap);
        assert_eq!(gap, expected, "missing: {missing}");
    }
}

/// E-BOUNDARY plays at an edge; E-LEVEL, when turned on, rises into a child level and
/// falls back out.
#[test]
fn boundary_and_level_earcons_follow_the_cursor() {
    let tree = tree_with(&spec("silence"), &[]);
    let mut settings = EarconSettings::for_tree(&tree);
    let mut cursor = Cursor::new();
    let boundary = cursor.apply(&tree, &Verb::Out);
    let cues = for_outcome(&tree, &boundary, &settings);
    assert_eq!(
        cues.iter().map(|c| c.id).collect::<Vec<_>>(),
        [EarconId::Boundary]
    );

    let into = cursor.apply(&tree, &Verb::Into);
    assert!(
        for_outcome(&tree, &into, &settings).is_empty(),
        "E-LEVEL is off by default"
    );
    settings.set(EarconId::Level, true);
    let deeper = cursor.apply(&tree, &Verb::Into);
    let cues = for_outcome(&tree, &deeper, &settings);
    assert!(cues.iter().any(|c| c.id == EarconId::Level && c.up));
    let back = cursor.apply(&tree, &Verb::Out);
    let cues = for_outcome(&tree, &back, &settings);
    assert!(cues.iter().any(|c| c.id == EarconId::Level && !c.up));
}
