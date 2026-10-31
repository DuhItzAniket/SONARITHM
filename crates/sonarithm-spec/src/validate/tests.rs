// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A03-T07 acceptance: every validation rule has a failing case (the code is reported at
//! the expected pointer) and a passing case (the spec is fully valid).

use sonarithm_core::diagnostic::Severity;

use super::*;
use crate::parse::parse;

const SOURCE: &str = r#"{ "kind": "inline", "values": [{ "t": 1, "v": 2.5, "g": "a", "m": 3 }] }"#;
const TONE: &str = r#"{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "v" } } }"#;

/// A spec with `tracks` and extra top-level members (`extra` starts with a comma).
fn spec_json(tracks: &str, extra: &str) -> String {
    format!(
        r#"{{ "specVersion": "0.1", "data": {{ "source": {SOURCE}, "fields": {{ "m": {{ "concept": "money" }} }} }},
             "tracks": [{tracks}] {extra} }}"#
    )
}

/// A spec whose only track is `track`.
fn with_track(track: &str) -> String {
    spec_json(track, "")
}

/// A spec with the tone track and extra top-level members.
fn with_extra(extra: &str) -> String {
    spec_json(TONE, extra)
}

/// A tone track with `channels`.
fn tone(channels: &str) -> String {
    format!(r#"{{ "id": "y", "mark": "tone", "channels": {channels} }}"#)
}

fn diagnose(json: &str) -> Diagnostics {
    let parsed = parse(json);
    assert!(parsed.is_ok(), "does not parse: {parsed:?}\n{json}");
    let spec = parsed.unwrap();
    let catalog = FieldCatalog::from_spec(&spec);
    validate(&spec, catalog.as_ref())
}

/// `failing` reports `code` at `pointer`; `passing` has no diagnostics at all.
fn check(code: &str, pointer: &str, failing: &str, passing: &str) {
    let found = diagnose(failing);
    assert!(
        found.iter().any(|d| d.code.as_str() == code
            && d.location.as_ref().map(SpecPointer::as_str) == Some(pointer)),
        "expected {code} at {pointer}, got {found:#?}"
    );
    let clean = diagnose(passing);
    assert!(clean.is_empty(), "passing case reported {clean:#?}");
}

#[test]
fn the_base_spec_is_valid() {
    assert!(diagnose(&with_extra("")).is_empty());
}

#[test]
fn snr_spec_004_only_x_keys_at_the_top_level() {
    check(
        "SNR-SPEC-004",
        "/colour",
        &with_extra(r#", "colour": 1"#),
        &with_extra(r#", "x-tool": 1"#),
    );
}

#[test]
fn snr_spec_002_seed_is_at_most_2_pow_53_minus_1() {
    check(
        "SNR-SPEC-002",
        "/seed",
        &with_extra(r#", "seed": 9007199254740992"#),
        &with_extra(r#", "seed": 9007199254740991"#),
    );
}

#[test]
fn snr_spec_010_011_012_track_ids() {
    check(
        "SNR-SPEC-010",
        "/tracks",
        &spec_json("", ""),
        &with_extra(""),
    );
    let twice = format!("{TONE}, {TONE}");
    let distinct = format!("{TONE}, {}", TONE.replace(r#""y""#, r#""z""#));
    check(
        "SNR-SPEC-011",
        "/tracks/1/id",
        &spec_json(&twice, ""),
        &spec_json(&distinct, ""),
    );
    check(
        "SNR-SPEC-012",
        "/tracks/0/id",
        &with_track(&TONE.replace(r#""y""#, r#""1y""#)),
        &with_track(&TONE.replace(r#""y""#, r#""y1""#)),
    );
}

#[test]
fn snr_spec_013_and_003_context_references() {
    check(
        "SNR-SPEC-013",
        "/context/reference/track",
        &with_extra(
            r#", "context": { "reference": { "kind": "dynamic-extreme", "track": "nope" } }"#,
        ),
        &with_extra(r#", "context": { "reference": { "kind": "dynamic-extreme", "track": "y" } }"#),
    );
    check(
        "SNR-SPEC-013",
        "/context/earcons/thresholds/0/track",
        &with_extra(
            r#", "context": { "earcons": { "thresholds": [{ "track": "nope", "value": 0 }] } }"#,
        ),
        &with_extra(
            r#", "context": { "earcons": { "thresholds": [{ "track": "y", "value": 0 }] } }"#,
        ),
    );
    check(
        "SNR-SPEC-003",
        "/context/reference",
        &with_extra(r#", "context": { "reference": { "kind": "static-value" } }"#),
        &with_extra(r#", "context": { "reference": { "kind": "static-value", "value": 0 } }"#),
    );
}

#[test]
fn snr_spec_020_021_field_or_value() {
    check(
        "SNR-SPEC-020",
        "/tracks/0/channels/pitch",
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "value": { "hz": 440 } } }"#,
        )),
        &with_track(&tone(r#"{ "pitch": { "value": { "hz": 440 } } }"#)),
    );
    check(
        "SNR-SPEC-021",
        "/tracks/0/channels/pan",
        &with_track(&tone(r#"{ "pan": {} }"#)),
        &with_track(&tone(r#"{ "pan": { "value": 0 } }"#)),
    );
}

#[test]
fn snr_spec_022_channels_and_audify_belong_to_marks() {
    check(
        "SNR-SPEC-022",
        "/tracks/0/channels/density",
        &with_track(&tone(r#"{ "density": { "value": 100 } }"#)),
        &with_track(
            r#"{ "id": "y", "mark": "grain", "channels": { "density": { "value": 100 } } }"#,
        ),
    );
    check(
        "SNR-SPEC-022",
        "/tracks/0/audify",
        &with_track(r#"{ "id": "y", "mark": "tone", "audify": { "k": 2200 } }"#),
        &with_track(r#"{ "id": "y", "mark": "audify", "audify": { "k": 2200 } }"#),
    );
}

#[test]
fn snr_spec_023_units_accepted_by_each_channel() {
    check(
        "SNR-SPEC-023",
        "/tracks/0/channels/pitch/value",
        &with_track(&tone(r#"{ "pitch": { "value": { "db": 3 } } }"#)),
        &with_track(&tone(r#"{ "pitch": { "value": { "midi": 69 } } }"#)),
    );
    check(
        "SNR-SPEC-023",
        "/tracks/0/channels/timbre/value",
        &with_track(&tone(r#"{ "timbre": { "value": 3 } }"#)),
        &with_track(&tone(r#"{ "timbre": { "value": "mallet" } }"#)),
    );
    check(
        "SNR-SPEC-023",
        "/tracks/0/channels/pitch/range",
        &with_track(&tone(r#"{ "pitch": { "field": "v", "range": [1, 2] } }"#)),
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "range": { "cents": [-200, 500] } } }"#,
        )),
    );
}

#[test]
fn snr_spec_024_values_inside_channel_bounds() {
    check(
        "SNR-SPEC-024",
        "/tracks/0/channels/pan/value",
        &with_track(&tone(r#"{ "pan": { "value": 2 } }"#)),
        &with_track(&tone(r#"{ "pan": { "value": -1 } }"#)),
    );
    check(
        "SNR-SPEC-024",
        "/tracks/0/channels/taps/value",
        &with_track(&tone(r#"{ "taps": { "value": 1.5 } }"#)),
        &with_track(&tone(r#"{ "taps": { "value": 8 } }"#)),
    );
    check(
        "SNR-SPEC-024",
        "/tracks/0/channels/pitch/value",
        &with_track(&tone(r#"{ "pitch": { "value": { "hz": 10 } } }"#)),
        &with_track(&tone(r#"{ "pitch": { "value": { "hz": 65 } } }"#)),
    );
    check(
        "SNR-SPEC-024",
        "/tracks/0/channels/duration/value",
        &with_track(&tone(r#"{ "duration": { "value": { "s": 0 } } }"#)),
        &with_track(&tone(r#"{ "duration": { "value": { "ms": 40 } } }"#)),
    );
    check(
        "SNR-SPEC-024",
        "/tracks/0/audify/k",
        &with_track(r#"{ "id": "y", "mark": "audify", "audify": { "k": 0 } }"#),
        &with_track(r#"{ "id": "y", "mark": "audify", "audify": { "k": 2200 } }"#),
    );
}

#[test]
fn snr_spec_025_026_mapping_options() {
    check(
        "SNR-SPEC-025",
        "/tracks/0/channels/pan/range",
        &with_track(&tone(r#"{ "pan": { "value": 0, "range": [-1, 1] } }"#)),
        &with_track(&tone(r#"{ "pan": { "field": "v", "range": [-1, 1] } }"#)),
    );
    check(
        "SNR-SPEC-026",
        "/tracks/0/channels/loudness/space",
        &with_track(&tone(r#"{ "loudness": { "field": "v", "space": "mel" } }"#)),
        &with_track(&tone(r#"{ "pitch": { "field": "v", "space": "mel" } }"#)),
    );
}

#[test]
fn snr_spec_030_032_scales() {
    check(
        "SNR-SPEC-030",
        "/tracks/0/channels/pitch/scale",
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "scale": { "type": "pow" } } }"#,
        )),
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "scale": { "type": "pow", "exponent": 2 } } }"#,
        )),
    );
    check(
        "SNR-SPEC-032",
        "/tracks/0/channels/pitch/scale/domain",
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "scale": { "type": "log", "domain": [0, 10] } } }"#,
        )),
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "scale": { "type": "log", "domain": [1, 10] } } }"#,
        )),
    );
}

/// SDR-11: the psychophysical scale needs an exponent m and a reference, on pitch only.
#[test]
fn sdr_11_psychophysical_scale_needs_exponent_and_reference() {
    check(
        "SNR-SPEC-030",
        "/tracks/0/channels/pitch/scale",
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "scale": { "type": "psychophysical", "exponent": 0.93 } } }"#,
        )),
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "scale": { "type": "psychophysical", "exponent": 0.93, "reference": { "value": 1, "hz": 440 } } } }"#,
        )),
    );
}

/// 12 §1's example message.
#[test]
fn snr_spec_031_reversed_ranges_point_to_polarity() {
    let failing = with_track(&tone(
        r#"{ "pitch": { "field": "v", "range": { "hz": [1400, 400] } } }"#,
    ));
    check(
        "SNR-SPEC-031",
        "/tracks/0/channels/pitch/range",
        &failing,
        &with_track(&tone(
            r#"{ "pitch": { "field": "v", "range": { "hz": [400, 1400] } } }"#,
        )),
    );
    let message = diagnose(&failing).into_vec().remove(0).message;
    assert_eq!(
        message,
        r#"Pitch range [1400, 400] Hz is reversed. Use "polarity": "negative" to invert the mapping instead."#
    );
}

/// SDR-10: musical quantization only in Presentation mode.
#[test]
fn sdr_10_music_only_in_presentation_mode() {
    let music = r#", "music": { "tempo": { "bpm": 96 } }"#;
    check(
        "SNR-SPEC-040",
        "/music",
        &with_extra(music),
        &with_extra(&format!(r#", "mode": "presentation"{music}"#)),
    );
}

/// A11Y-02: interpretation (L4) is never automatic.
#[test]
fn a11y_02_no_automatic_l4_descriptions() {
    check(
        "SNR-SPEC-041",
        "/analysis/describe/levels/1",
        &with_extra(r#", "analysis": { "describe": { "levels": ["L1", "L4"] } }"#),
        &with_extra(r#", "analysis": { "describe": { "levels": ["L1", "L3"] } }"#),
    );
}

/// SDR-14: money is a flagged concept, so automatic polarity warns.
#[test]
fn sdr_14_flagged_concept_needs_explicit_polarity() {
    let failing = with_track(&tone(r#"{ "pitch": { "field": "m" } }"#));
    check(
        "SNR-SPEC-042",
        "/tracks/0/channels/pitch",
        &failing,
        &with_track(&tone(
            r#"{ "pitch": { "field": "m", "polarity": "negative" } }"#,
        )),
    );
    assert_eq!(diagnose(&failing).into_vec()[0].severity, Severity::Warning);
}

#[test]
fn snr_spec_050_pace_is_positive() {
    check(
        "SNR-SPEC-050",
        "/substrate/time/pace",
        &with_extra(r#", "substrate": { "time": { "pace": { "pointsPerSecond": 0 } } }"#),
        &with_extra(r#", "substrate": { "time": { "pace": { "pointsPerSecond": 3 } } }"#),
    );
}

#[test]
fn snr_spec_051_source_parameters() {
    let function = |domain: &str| {
        format!(
            r#"{{ "specVersion": "0.1", "data": {{ "source": {{ "kind": "function", "expr": "sin(x)", "domain": {domain} }} }},
                 "tracks": [{{ "id": "f", "mark": "glide", "channels": {{ "pitch": {{ "field": "y" }} }} }}] }}"#
        )
    };
    check(
        "SNR-SPEC-051",
        "/data/source",
        &function("[1, -1]"),
        &function("[-1, 1]"),
    );
    let stream = |address: &str| {
        format!(
            r#"{{ "specVersion": "0.1", "data": {{ "source": {{ "kind": "stream", "transport": "websocket" {address} }} }}, "tracks": [{TONE}] }}"#
        )
    };
    check(
        "SNR-SPEC-051",
        "/data/source",
        &stream(""),
        &stream(r#", "url": "ws://127.0.0.1:9000""#),
    );
}

#[test]
fn snr_spec_052_absolute_paths_warn() {
    let file = |path: &str| {
        format!(
            r#"{{ "specVersion": "0.1", "data": {{ "source": {{ "kind": "file", "path": "{path}" }} }}, "tracks": [{TONE}] }}"#
        )
    };
    check(
        "SNR-SPEC-052",
        "/data/source/path",
        &file("/home/someone/a.csv"),
        &file("data/a.csv"),
    );
    check(
        "SNR-SPEC-052",
        "/data/source/path",
        &file(r"C:\\data\\a.csv"),
        &file("a.csv"),
    );
}

#[test]
fn snr_spec_053_transform_parameters() {
    check(
        "SNR-SPEC-053",
        "/transform/0",
        &with_extra(
            r#", "transform": [{ "op": "smooth", "field": "v", "method": "ema", "alpha": 1.5 }]"#,
        ),
        &with_extra(
            r#", "transform": [{ "op": "smooth", "field": "v", "method": "ema", "alpha": 0.3 }]"#,
        ),
    );
    check(
        "SNR-SPEC-053",
        "/transform/0",
        &with_extra(r#", "transform": [{ "op": "aggregate", "groupby": ["g"], "ops": [] }]"#),
        &with_extra(
            r#", "transform": [{ "op": "aggregate", "groupby": ["g"], "ops": [{ "op": "mean", "field": "v", "as": "v_mean" }] }]"#,
        ),
    );
}

#[test]
fn snr_spec_060_061_output_and_playback() {
    check(
        "SNR-SPEC-060",
        "/output/sampleRate",
        &with_extra(r#", "output": { "sampleRate": 1000 }"#),
        &with_extra(r#", "output": { "sampleRate": 44100 }"#),
    );
    check(
        "SNR-SPEC-060",
        "/output/blockSize",
        &with_extra(r#", "output": { "blockSize": 300 }"#),
        &with_extra(r#", "output": { "blockSize": 512 }"#),
    );
    check(
        "SNR-SPEC-061",
        "/playback/speed",
        &with_extra(r#", "playback": { "speed": 0 }"#),
        &with_extra(r#", "playback": { "speed": 1.5 }"#),
    );
}

#[test]
fn snr_spec_070_fields_must_exist() {
    check(
        "SNR-SPEC-070",
        "/tracks/0/channels/pitch/field",
        &with_track(&tone(r#"{ "pitch": { "field": "nope" } }"#)),
        &with_extra(""),
    );
    let derived = r#", "transform": [{ "op": "derive", "expr": "v * 2", "as": "w" }]"#;
    let track = tone(r#"{ "pitch": { "field": "w" } }"#);
    assert!(
        diagnose(&spec_json(&track, derived)).is_empty(),
        "transform outputs are fields"
    );
}

/// SDR-25: nominal fields map to timbre, quantitative fields to pitch.
#[test]
fn sdr_25_snr_spec_071_field_types_fit_channels() {
    check(
        "SNR-SPEC-071",
        "/tracks/0/channels/timbre/field",
        &with_track(&tone(r#"{ "timbre": { "field": "v" } }"#)),
        &with_track(&tone(r#"{ "timbre": { "field": "g" } }"#)),
    );
    check(
        "SNR-SPEC-071",
        "/tracks/0/channels/pitch/field",
        &with_track(&tone(r#"{ "pitch": { "field": "g" } }"#)),
        &with_track(&tone(r#"{ "pitch": { "field": "t" } }"#)),
    );
}

// --- Spec v0.2 (RFC-0004): a failing and a passing case for every new rule. ---

/// A v0.2 spec with the tone track and extra top-level members.
fn v02(extra: &str) -> String {
    with_extra(extra).replacen(r#""specVersion": "0.1""#, r#""specVersion": "0.2""#, 1)
}

/// A v0.2 spec reading a chart with `source_extra` on the source.
fn chart(source_extra: &str) -> String {
    format!(
        r#"{{ "specVersion": "0.2", "data": {{ "source": {{ "kind": "chart", "path": "fig.png" {source_extra} }} }},
             "tracks": [{TONE}] }}"#
    )
}

/// A v0.2 presentation spec with `quantize`.
fn quantize(body: &str) -> String {
    v02(&format!(
        r#", "mode": "presentation", "music": {{ "quantize": {body} }}"#
    ))
}

const AXES: &str = r#""axes": { "x": { "kind": "linear", "anchors": [ { "at": [112, 940], "value": 0 }, { "at": [1210, 940], "value": 50 } ] },
                                "y": { "kind": "log", "anchors": [ { "at": [112, 940], "value": 1 }, { "at": [112, 120], "value": 1000 } ] } }"#;

#[test]
fn snr_spec_004_v0_2_keys_need_spec_version_0_2() {
    for (key, value) in [
        ("template", r#"{ "id": "bar-discrete" }"#),
        (
            "instruments",
            r#"{ "piano": { "kind": "soundfont", "path": "p.sf2" } }"#,
        ),
        ("lint", r#"{ "experimental": true }"#),
    ] {
        let extra = format!(r#", "{key}": {value}"#);
        check(
            "SNR-SPEC-004",
            &format!("/{key}"),
            &with_extra(&extra),
            &v02(&extra),
        );
    }
    let failing = chart(r#", "corrections": { "edits": [] }"#).replacen("0.2", "0.1", 1);
    check(
        "SNR-SPEC-004",
        "/data/source/corrections",
        &failing,
        &chart(r#", "corrections": { "edits": [] }"#),
    );
}

#[test]
fn snr_spec_010_a_template_stands_in_for_tracks() {
    let template_only = r#"{ "specVersion": "0.2", "data": { "source": { "kind": "file", "path": "s.csv" } },
                             "template": { "id": "bar-discrete", "bind": { "category": "quarter" } } }"#;
    assert!(diagnose(template_only).is_empty());
    let neither =
        r#"{ "specVersion": "0.2", "data": { "source": { "kind": "file", "path": "s.csv" } } }"#;
    assert!(
        diagnose(neither)
            .iter()
            .any(|d| d.code.as_str() == "SNR-SPEC-010")
    );
}

#[test]
fn snr_spec_002_template_ids_are_kebab_case() {
    check(
        "SNR-SPEC-002",
        "/template/id",
        &v02(r#", "template": { "id": "Bar_Discrete" }"#),
        &v02(r#", "template": { "id": "box-five-number" }"#),
    );
}

#[test]
fn snr_spec_024_and_052_instrument_bounds_and_paths() {
    check(
        "SNR-SPEC-024",
        "/instruments/piano/program",
        &v02(
            r#", "instruments": { "piano": { "kind": "soundfont", "path": "p.sf2", "program": 200 } }"#,
        ),
        &v02(
            r#", "instruments": { "piano": { "kind": "soundfont", "path": "p.sf2", "bank": 16383, "program": 127 } }"#,
        ),
    );
    let absolute = diagnose(&v02(
        r#", "instruments": { "piano": { "kind": "soundfont", "path": "/abs/p.sf2" } }"#,
    ));
    assert!(absolute.iter().any(|d| d.code.as_str() == "SNR-SPEC-052"));
}

#[test]
fn snr_spec_045_unknown_lint_rules_warn() {
    let found = diagnose(&v02(r#", "lint": { "rules": { "SDR-99": "off" } }"#));
    let warning = found
        .iter()
        .find(|d| d.code.as_str() == "SNR-SPEC-045")
        .unwrap();
    assert_eq!(warning.severity, Severity::Warning);
    assert_eq!(
        warning.location.as_ref().map(SpecPointer::as_str),
        Some("/lint/rules/SDR-99")
    );
    let known = v02(
        r#", "lint": { "rules": { "SDR-17": "error", "A11Y-14": "warning" }, "experimental": true }"#,
    );
    assert!(diagnose(&known).is_empty());
}

#[test]
fn snr_spec_046_suppressions_need_a_reason() {
    check(
        "SNR-SPEC-046",
        "/lint/suppress/0/reason",
        &v02(r#", "lint": { "suppress": [ { "rule": "SDR-17", "reason": "  " } ] }"#),
        &v02(
            r#", "lint": { "suppress": [ { "rule": "SDR-17", "track": "y", "reason": "The freezing line is the reference." } ] }"#,
        ),
    );
    check(
        "SNR-SPEC-013",
        "/lint/suppress/0/track",
        &v02(
            r#", "lint": { "suppress": [ { "rule": "SDR-17", "track": "nope", "reason": "r" } ] }"#,
        ),
        &v02(r#", "lint": { "suppress": [ { "rule": "SDR-17", "track": "y", "reason": "r" } ] }"#),
    );
}

#[test]
fn snr_spec_054_calibrations_are_checked() {
    let passing = chart(&format!(r#", "calibration": {{ {AXES} }}"#));
    let same_value = format!(
        r#", "calibration": {{ {} }}"#,
        AXES.replace(r#""value": 1 }"#, r#""value": 1000 }"#)
    );
    let same_point = format!(
        r#", "calibration": {{ {} }}"#,
        AXES.replace("[1210, 940]", "[112, 940]")
    );
    let cases: [(&str, &str); 5] = [
        (
            r#", "calibration": { "axes": { "x": { "kind": "linear", "anchors": [ { "at": [1, 1], "value": 0 } ] } } }"#,
            "/data/source/calibration/axes",
        ),
        (&same_value, "/data/source/calibration/axes/y"),
        (&same_point, "/data/source/calibration/axes/x"),
        (
            r#", "calibration": { "axes": { "x": { "kind": "date", "anchors": [ { "at": [0, 0], "value": "March" }, { "at": [9, 0], "value": "2024" } ] }, "y": { "kind": "category", "anchors": [ { "at": [0, 0], "value": "a" }, { "at": [0, 9], "value": "b" } ] } } }"#,
            "/data/source/calibration/axes/x",
        ),
        (
            r#", "calibration": { "units": "px" }"#,
            "/data/source/calibration/units",
        ),
    ];
    for (failing, pointer) in cases {
        check("SNR-SPEC-054", pointer, &chart(failing), &passing);
    }
    // A v0.1 calibration that v0.2 rejects still reads, and fails validation (Q14).
    let old = chart(r#", "calibration": { "x0": 1 }"#).replacen("0.2", "0.1", 1);
    assert!(
        diagnose(&old)
            .iter()
            .any(|d| d.code.as_str() == "SNR-SPEC-054")
    );
}

#[test]
fn snr_spec_055_and_056_corrections_are_checked() {
    let passing = chart(
        r#", "corrections": { "edits": [
        { "op": "add", "series": "S", "to": { "x": 3, "y": 4 } },
        { "op": "move", "series": "S", "point": 12, "was": { "x": 24.1, "y": 210.4 }, "to": { "y": 230 } },
        { "op": "delete", "series": "S", "point": 30, "was": { "x": 48.7, "y": 3.2 } },
        { "op": "retype", "series": "S", "point": 2, "was": { "y": 1 }, "to": { "series": "T" } } ] }"#,
    );
    check(
        "SNR-SPEC-055",
        "/data/source/corrections/edits/0",
        &chart(
            r#", "corrections": { "edits": [ { "op": "move", "series": "S", "point": 1, "was": { "y": 1 }, "to": {} } ] }"#,
        ),
        &passing,
    );
    check(
        "SNR-SPEC-055",
        "/data/source/corrections/edits/0/op",
        &chart(r#", "corrections": { "edits": [ { "op": "nudge", "series": "S" } ] }"#),
        &passing,
    );
    check(
        "SNR-SPEC-056",
        "/data/source/corrections/path",
        &chart(r#", "corrections": { "path": "fig.corrections.json" }"#),
        &passing,
    );
}

/// RFC-0005 §7, Q6: spec 0.3 allows `corrections.path`, a relative path only
/// (`SNR-SPEC-093`); a 0.2 document still gets `SNR-SPEC-056`; a path and edits together
/// are a shape error.
#[test]
fn snr_spec_093_correction_file_paths_are_relative() {
    let v03 = |extra: &str| chart(extra).replacen("\"0.2\"", "\"0.3\"", 1);
    let passing = v03(r#", "corrections": { "path": "charts/fig.png.corrections.json" }"#);
    assert_eq!(
        diagnose(&passing)
            .iter()
            .filter(|d| d.code.as_str().starts_with("SNR-SPEC-05")
                || d.code.as_str().starts_with("SNR-SPEC-09"))
            .count(),
        0
    );
    for bad in [
        "/home/me/fig.json",
        "C:/fig.json",
        "../fig.json",
        "a/../../fig.json",
        "https://example.org/f.json",
        "",
    ] {
        check(
            "SNR-SPEC-093",
            "/data/source/corrections/path",
            &v03(&format!(r#", "corrections": {{ "path": "{bad}" }}"#)),
            &passing,
        );
    }
    check(
        "SNR-SPEC-055",
        "/data/source/corrections",
        &v03(r#", "corrections": { "path": "fig.json", "edits": [] }"#),
        &passing,
    );
}

#[test]
fn snr_spec_043_and_044_quantize_vocabularies() {
    check(
        "SNR-SPEC-043",
        "/music/quantize/scale",
        &quantize(r#"{ "scale": "C ionian" }"#),
        &quantize(r#"{ "scale": "C major pentatonic", "tuning": "12-TET" }"#),
    );
    check(
        "SNR-SPEC-043",
        "/music/quantize/scale",
        &quantize(r#"{ "scale": { "tonic": "D", "cents": [0, 300, 200] } }"#),
        &quantize(r#"{ "scale": { "tonic": "D", "cents": [0, 150, 350, 500, 700, 850, 1050] } }"#),
    );
    check(
        "SNR-SPEC-044",
        "/music/quantize/tuning",
        &quantize(r#"{ "scale": "A minor", "tuning": "19-TET" }"#),
        &quantize(
            r#"{ "scale": "A minor", "tuning": { "cents": [0, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100] } }"#,
        ),
    );
    check(
        "SNR-SPEC-044",
        "/music/quantize/tuning",
        &quantize(r#"{ "scale": { "tonic": "D", "cents": [0, 700] }, "tuning": "12-TET" }"#),
        &quantize(r#"{ "scale": "Eb blues", "tuning": "12-TET" }"#),
    );
    check(
        "SNR-SPEC-044",
        "/music/quantize/tuning",
        &quantize(r#"{ "scale": "Eb blues", "tuning": "just" }"#),
        &quantize(r#"{ "scale": "Eb blues" }"#),
    );
}

/// SDR-10 still holds in v0.2: `music` outside Presentation is an error that no lint
/// setting can lower.
#[test]
fn sdr_10_quantize_rejected_outside_presentation_in_v0_2() {
    let analysis = v02(
        r#", "music": { "quantize": { "scale": "C major" } }, "lint": { "rules": { "SDR-17": "off" } }"#,
    );
    assert!(
        diagnose(&analysis)
            .iter()
            .any(|d| d.code.as_str() == "SNR-SPEC-040")
    );
}

/// SDR-03: a template names no field; only `bind` connects it to the data.
#[test]
fn sdr_03_template_is_data_agnostic() {
    let template: crate::v02::TemplateRef =
        serde_json::from_str(r#"{ "id": "line-overview", "bind": { "value": "price" } }"#).unwrap();
    assert_eq!(
        template.bind.get("value").map(String::as_str),
        Some("price")
    );
    let named_field = serde_json::from_str::<crate::v02::TemplateRef>(
        r#"{ "id": "line-overview", "field": "price" }"#,
    );
    assert!(named_field.is_err(), "a template has no field of its own");
}
