// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The facade end to end: specs, compiles, the quick builder, summaries and descriptions.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::PathBuf;

use sonarithm::{
    Datasets, Error, Format, Mark, ReadOptions, SchemaKind, Severity, Sonarithm, Spec,
};
use sonarithm_core::engine::CancelToken;

const TEMPS: &str = r#"{
    "specVersion": "0.1",
    "name": "Temperatures",
    "description": "Three days of noon temperatures.",
    "data": {
        "source": { "kind": "inline", "values": [
            { "day": 1, "temp": 8.5 }, { "day": 2, "temp": 9.25 }, { "day": 3, "temp": 7.0 }
        ] },
        "fields": { "temp": { "unit": "°C", "concept": "temperature" } }
    },
    "substrate": { "time": { "field": "day" } },
    "tracks": [{ "id": "temps", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }]
}"#;

fn temps() -> Spec {
    Spec::from_json_str(TEMPS)
        .unwrap()
        .with_name("temps.spec.json")
}

fn codes(diagnostics: &sonarithm::Diagnostics) -> Vec<String> {
    diagnostics.iter().map(|d| d.code.to_string()).collect()
}

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("facade")
        .join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// With navigation, a compile also returns AUDAXIS's navigation tree, and AUDAXIS's
/// shaping (stereo placement, A11Y-14) is recorded in the Score header; the data events
/// are the same.
#[test]
fn navigation_builds_the_tree_beside_the_score() {
    let plain = Sonarithm::builder().with_default_engines().build().unwrap();
    let navigating = Sonarithm::builder()
        .with_default_engines()
        .with_navigation()
        .build()
        .unwrap();
    let without = plain.compile(&temps()).unwrap();
    let with = navigating.compile(&temps()).unwrap();
    assert!(without.nav().is_none());
    let tree = with.nav().unwrap();
    assert!(tree.len() > 3, "dataset, series, axes and three points");
    assert_eq!(
        with.score().tracks[0].events,
        without.score().tracks[0].events
    );
    assert!(
        navigating
            .engines()
            .iter()
            .any(|engine| engine.id.as_str() == "audaxis")
    );
}

#[test]
fn a_spec_compiles_with_the_default_engines() {
    let snr = Sonarithm::builder().with_default_engines().build().unwrap();
    let compiled = snr.compile(&temps()).unwrap();
    assert_eq!(compiled.score().tracks.len(), 1);
    assert_eq!(compiled.score().tracks[0].events.len(), 3);
    assert!(!compiled.diagnostics().has_errors());
    assert!(
        compiled
            .diagnostics()
            .iter()
            .any(|d| d.severity == Severity::Info && d.rules.iter().any(|r| r.as_str() == "SDR-08")),
        "applied defaults are reported with their rules"
    );
    let engines: Vec<String> = snr.engines().iter().map(|e| e.id.to_string()).collect();
    assert_eq!(engines, ["platform", "numisona"]);
}

#[test]
fn without_engines_a_tone_track_reports_the_missing_engine() {
    let snr = Sonarithm::builder().build().unwrap();
    let Err(Error::Spec(diagnostics)) = snr.compile(&temps()) else {
        panic!("expected an error")
    };
    assert!(codes(&diagnostics).contains(&"SNR-COMP-010".to_owned()));
}

#[test]
fn the_facade_compiles_the_same_score_as_the_compiler() {
    let snr = Sonarithm::new();
    let first = snr.compile(&temps()).unwrap();
    let second = snr.compile(&temps()).unwrap();
    assert_eq!(first.score_hash().unwrap(), second.score_hash().unwrap());

    let numisona = sonarithm_numisona::Numisona::new();
    let platform = sonarithm_compiler::Platform::new();
    let direct = sonarithm_compiler::Compiler::new(&[&platform, &numisona])
        .unwrap()
        .compile(TEMPS, &CancelToken::new())
        .unwrap();
    assert_eq!(
        first.score().canonical_json().unwrap(),
        direct.score.canonical_json().unwrap(),
        "byte-identical canonical Score"
    );
    assert_eq!(first.canonical_spec(), direct.canonical_spec);
}

#[test]
fn summaries_and_descriptions_are_stable() {
    let compiled = Sonarithm::new().compile(&temps()).unwrap();
    // Engine versions, and the Score hash that covers them, change with every release.
    let mut summary = compiled.summary();
    for version in summary.engines.values_mut() {
        "[version]".clone_into(version);
    }
    summary.score_hash = Some("[score hash]".to_owned());
    insta::assert_json_snapshot!("temps_summary", summary);
    insta::assert_snapshot!("temps_description", compiled.describe().to_string());
}

#[test]
fn descriptions_name_estimates() {
    let estimated = sonarithm_numisona::examples::estimated().unwrap();
    let compiled = Sonarithm::new()
        .quick()
        .data(estimated)
        .x("x")
        .y("y")
        .compile()
        .unwrap();
    let description = compiled.describe();
    assert!(
        description
            .l2
            .iter()
            .any(|line| line.contains("3 of 10 tones are estimates")),
        "{description}"
    );
    assert_eq!(compiled.summary().tracks[0].estimated, 3);
}

#[test]
fn invalid_json_is_reported_with_its_position() {
    let Err(Error::Spec(diagnostics)) = Spec::from_json_str("{ \"specVersion\": ") else {
        panic!("expected an error")
    };
    assert_eq!(codes(&diagnostics), ["SNR-SPEC-001"]);
    assert!(
        diagnostics
            .iter()
            .next()
            .unwrap()
            .message
            .contains("line 1")
    );
}

#[test]
fn validation_reports_problems_and_defaults() {
    let defaults = temps().validate();
    assert!(!defaults.has_errors());
    assert!(codes(&defaults).contains(&"SNR-SPEC-080".to_owned()));
    let unknown_key =
        Spec::from_json_str(r#"{ "specVersion": "0.1", "tracks": [], "data": {}, "colour": 1 }"#)
            .unwrap()
            .validate();
    assert!(unknown_key.has_errors());
}

#[test]
fn migration_changes_only_the_version_and_a_current_spec_not_at_all() {
    // RFC-0004 §9: 0.1 → 0.2 rewrites `specVersion` and nothing else.
    let migration = temps().migrate().unwrap();
    assert!(migration.changed);
    assert_eq!(migration.from, sonarithm::SpecVersion::V0_1);
    let migrated: serde_json::Value = serde_json::from_str(&migration.text).unwrap();
    let mut expected: serde_json::Value = serde_json::from_str(TEMPS).unwrap();
    expected["specVersion"] = serde_json::json!("0.2");
    assert_eq!(migrated, expected);
    let again = Spec::from_json_str(&migration.text)
        .unwrap()
        .migrate()
        .unwrap();
    assert!(!again.changed);
    assert_eq!(again.from, sonarithm::SPEC_VERSION);
    let Err(Error::Spec(diagnostics)) = Spec::from_json_str("{}").unwrap().migrate() else {
        panic!("expected an error")
    };
    assert_eq!(codes(&diagnostics), ["SNR-SPEC-003"]);
    let future = Spec::from_json_str(r#"{ "specVersion": "9.0" }"#).unwrap();
    let Err(Error::Spec(diagnostics)) = future.migrate() else {
        panic!("expected an error")
    };
    assert_eq!(codes(&diagnostics), ["SNR-SPEC-005"]);
}

#[test]
fn spec_files_resolve_their_data_next_to_them() {
    let dir = tmp("spec-files");
    std::fs::write(dir.join("series.csv"), "t,v\n0,1\n1,3\n2,2\n3,5\n").unwrap();
    std::fs::write(
        dir.join("series.spec.json"),
        r#"{ "specVersion": "0.1", "data": { "source": { "kind": "file", "path": "series.csv" } },
             "substrate": { "time": { "field": "t" } },
             "tracks": [{ "id": "v", "mark": "tone", "channels": { "pitch": { "field": "v" } } }] }"#,
    )
    .unwrap();
    let spec = Spec::from_file(dir.join("series.spec.json")).unwrap();
    assert_eq!(spec.name(), "series.spec.json");
    let compiled = Sonarithm::new().compile(&spec).unwrap();
    assert_eq!(compiled.score().tracks[0].events.len(), 4);

    let Err(err @ Error::Read { .. }) = Spec::from_file(dir.join("absent.spec.json")) else {
        panic!("expected a read error")
    };
    let message = err.to_string();
    assert!(message.contains("absent.spec.json"));
    assert!(
        !message.contains("facade"),
        "no directories in messages: {message}"
    );
}

#[test]
fn host_datasets_compile_by_id() {
    let bundle = sonarithm::read_bytes(
        b"x,y\n1,2\n2,4\n3,3\n",
        Format::Csv,
        "frame.csv",
        &ReadOptions::new(),
    )
    .unwrap()
    .bundle;
    let spec = Spec::from_json_str(
        r#"{ "specVersion": "0.1", "data": { "source": { "kind": "dataset", "id": "frame" } },
             "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "y" } } }] }"#,
    )
    .unwrap();
    let datasets = Datasets::new().with("frame", bundle);
    assert_eq!(datasets.ids().collect::<Vec<_>>(), ["frame"]);
    let compiled = Sonarithm::new().compile_with(&spec, &datasets).unwrap();
    assert_eq!(compiled.score().tracks[0].events.len(), 3);
    let Err(Error::Spec(diagnostics)) = Sonarithm::new().compile_with(
        &spec,
        &Datasets::new().with("other", datasets.get("frame").unwrap().clone()),
    ) else {
        panic!("expected an error")
    };
    assert!(codes(&diagnostics).contains(&"SNR-COMP-026".to_owned()));
}

#[test]
fn a_cancelled_compile_stops() {
    let cancel = CancelToken::new();
    cancel.cancel();
    let result = Sonarithm::new().compile_cancellable(&temps(), &Datasets::new(), &cancel);
    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
}

#[test]
fn quick_writes_a_portable_spec_for_a_file() {
    let dir = tmp("quick-file");
    let csv = dir.join("temps.csv");
    std::fs::write(
        &csv,
        "year,anomaly,station\n2000,0.39,a\n2001,0.54,a\n2002,0.63,a\n",
    )
    .unwrap();
    let quick = Sonarithm::new()
        .quick()
        .data_file(&csv)
        .unwrap()
        .x("year")
        .y("anomaly");
    let spec = quick.spec().unwrap();
    assert_eq!(spec.name(), "temps.spec.json");
    let value: serde_json::Value = serde_json::from_str(spec.text()).unwrap();
    assert_eq!(
        value["data"]["source"],
        serde_json::json!({ "kind": "file", "path": "temps.csv" })
    );
    assert_eq!(value["name"], "temps");
    assert!(
        !spec.text().contains("quick-file"),
        "no directories in the spec (P8)"
    );
    let compiled = quick.compile().unwrap();
    assert_eq!(compiled.score().tracks[0].events.len(), 3);
    // Without x and y, the template binds by role and kind.
    let bound = Sonarithm::new()
        .quick()
        .data_file(&csv)
        .unwrap()
        .compile()
        .unwrap();
    assert_eq!(bound.score_hash().unwrap(), compiled.score_hash().unwrap());
}

#[test]
fn quick_options_shape_the_spec() {
    let temps = sonarithm_numisona::examples::temperature().unwrap();
    let quick = Sonarithm::new()
        .quick()
        .data(temps)
        .x("day")
        .y("temp")
        .mark(Mark::Glide)
        .range_hz(300.0, 900.0)
        .concept("temperature")
        .name("Noon temperatures");
    let value: serde_json::Value = serde_json::from_str(quick.spec().unwrap().text()).unwrap();
    assert_eq!(value["tracks"][0]["mark"], "glide");
    assert_eq!(
        value["tracks"][0]["channels"]["pitch"]["range"],
        serde_json::json!({ "hz": [300.0, 900.0] })
    );
    assert_eq!(value["data"]["fields"]["temp"]["concept"], "temperature");
    assert_eq!(
        value["data"]["source"],
        serde_json::json!({ "kind": "dataset", "id": "main" })
    );
    let compiled = quick.compile().unwrap();
    let range = compiled.summary().tracks[0].pitch_range_hz.unwrap();
    assert_eq!(range, [300.0, 900.0]);
    assert_eq!(compiled.summary().tracks[0].glides, 1);
}

#[test]
fn quick_explains_what_it_cannot_map() {
    let snr = Sonarithm::new();
    assert!(matches!(snr.quick().compile(), Err(Error::NoData)));
    let fruit = sonarithm_numisona::examples::fruit().unwrap();
    let same = snr.quick().data(fruit.clone()).x("sold").y("sold").spec();
    assert!(matches!(same, Err(Error::Template(_))), "{same:?}");
    let text = snr.quick().data(fruit).x("fruit").y("sold").spec();
    let Err(err) = text else {
        panic!("a text field cannot be x")
    };
    assert!(err.to_string().contains("fruit"), "{err}");
}

#[test]
fn the_schema_is_the_published_one() {
    let published = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../schemas/sonarithm-spec-0.2.schema.json"),
    )
    .unwrap();
    assert_eq!(
        sonarithm::schema(SchemaKind::Spec).trim_end(),
        published.trim_end()
    );
}
