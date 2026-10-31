// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "tests")]

use proptest::prelude::*;
use serde_json::{Value, json};

use super::*;
use crate::v02::AnchorValue;

const SHA: &str = "9f2c000000000000000000000000000000000000000000000000000000e41a00";

fn value(x: &str, y: f64) -> PointValue {
    serde_json::from_value(json!({ "x": x, "y": y })).unwrap()
}

fn sample() -> CorrectionFile {
    let mut chart = ChartRef::new(SHA, "sonarithm-plotsona/raster", "0.1.0");
    chart.name = Some("rain.png".to_owned());
    let mut file = CorrectionFile::new(chart);
    let before = Before::new(ProvenanceClass::CvEstimate, Some(0.41));
    file.edits = vec![
        FileEdit::Move {
            id: "e1".into(),
            series: "Series 2".into(),
            point: PointRef::Index(4),
            was: value("May", 61.2),
            to: serde_json::from_value(json!({ "y": 64 })).unwrap(),
            before,
            note: None,
        },
        FileEdit::Add {
            id: "e2".into(),
            series: "Series 1".into(),
            to: value("Jun", 12.5),
            note: Some("hidden behind the legend".into()),
        },
        FileEdit::Move {
            id: "e3".into(),
            series: "Series 1".into(),
            point: PointRef::Added("+e2".into()),
            was: value("Jun", 12.5),
            to: serde_json::from_value(json!({ "y": 13 })).unwrap(),
            before: Before::new(ProvenanceClass::UserEdited, None),
            note: None,
        },
        FileEdit::Delete {
            id: "e4".into(),
            series: "Series 1".into(),
            point: PointRef::Index(11),
            was: value("Dec", 3.0),
            before: Before::new(ProvenanceClass::CvEstimate, Some(0.22)),
            note: Some("legend swatch, not data".into()),
        },
        FileEdit::Retype {
            id: "e5".into(),
            series: "Series 3".into(),
            point: PointRef::Index(0),
            was: value("Jan", 40.0),
            to: serde_json::from_value(json!({ "series": "Series 2" })).unwrap(),
            before,
            note: None,
        },
    ];
    file
}

fn codes_of(result: Result<(CorrectionFile, Diagnostics), Diagnostics>) -> Vec<String> {
    match result {
        Ok((_, warnings)) => warnings
            .iter()
            .map(|d| d.code.as_str().to_owned())
            .collect(),
        Err(errors) => errors.iter().map(|d| d.code.as_str().to_owned()).collect(),
    }
}

fn read_json(v: &Value) -> Result<(CorrectionFile, Diagnostics), Diagnostics> {
    read(serde_json::to_string(v).unwrap().as_bytes())
}

fn sample_json() -> Value {
    serde_json::from_str(&write(&sample())).unwrap()
}

/// RFC-0005 §1, §12: writing is canonical and reading what was written gives the file back.
#[test]
fn written_files_read_back_and_write_the_same_bytes() {
    let text = write(&sample());
    assert!(text.ends_with("}\n") && !text.contains('\r'));
    assert!(text.starts_with(
        "{\n  \"format\": \"sonarithm-corrections\",\n  \"version\": \"1.0\",\n  \"chart\": {"
    ));
    let (back, warnings) = read(text.as_bytes()).unwrap();
    assert!(warnings.is_empty());
    assert_eq!(back, sample());
    assert_eq!(write(&back), text);
}

#[test]
fn the_rfc_example_reads() {
    let example = json!({
        "format": "sonarithm-corrections",
        "version": "1.0",
        "chart": { "sha256": SHA, "name": "rain.png", "extractor": "sonarithm-plotsona/raster", "extractorVersion": "0.1.0" },
        "calibration": { "axes": {
            "x": { "kind": "category", "anchors": [{ "at": [77.5, 0], "value": "Jan" }, { "at": [811.5, 0], "value": "Dec" }] },
            "y": { "kind": "linear", "anchors": [{ "at": [0, 507.5], "value": 0 }, { "at": [0, 27.5], "value": 110 }] }
        }},
        "edits": [
            { "op": "move", "id": "e1", "series": "Series 2", "point": 4, "was": { "x": "May", "y": 61.2 }, "to": { "y": 64 },
              "before": { "class": "cv_estimate", "confidence": 0.41 } },
            { "op": "delete", "id": "e2", "series": "Series 1", "point": 11, "was": { "x": "Dec", "y": 3.0 },
              "before": { "class": "cv_estimate", "confidence": 0.22 }, "note": "legend swatch, not data" }
        ]
    });
    let (file, _) = read_json(&example).unwrap();
    assert_eq!(file.edits.len(), 2);
    assert!(file.calibration.is_some());
    assert_eq!(file.edits[0].point(), Some(&PointRef::Index(4)));
}

/// §8: not a correction file, or a newer major version, is `SNR-SPEC-090`.
#[test]
fn other_files_and_newer_major_versions_are_refused() {
    assert_eq!(codes_of(read(b"[1, 2]")), ["SNR-SPEC-090"]);
    let mut other = sample_json();
    other["format"] = json!("something-else");
    assert_eq!(codes_of(read_json(&other)), ["SNR-SPEC-090"]);
    let mut newer = sample_json();
    newer["version"] = json!("2.0");
    let err = read_json(&newer).unwrap_err();
    assert!(
        err.iter()
            .next()
            .unwrap()
            .message
            .contains("newer SONARITHM")
    );
    let mut bad = sample_json();
    bad["version"] = json!("one");
    assert_eq!(codes_of(read_json(&bad)), ["SNR-SPEC-090"]);
}

/// §8: a newer minor version is read, its unknown keys dropped, with `SNR-SPEC-094`; in
/// 1.0 an unknown key is `SNR-SPEC-091` at its pointer.
#[test]
fn unknown_keys_are_errors_in_known_versions_and_dropped_in_newer_minor_ones() {
    let mut extra = sample_json();
    extra["edits"][0]["confidenceBand"] = json!([0.3, 0.5]);
    extra["chart"]["dpi"] = json!(144);
    let err = read_json(&extra).unwrap_err();
    let pointers: Vec<&str> = err
        .iter()
        .map(|d| d.location.as_ref().map_or("", |p| p.as_str()))
        .collect();
    assert!(
        pointers.contains(&"/edits/0/confidenceBand") && pointers.contains(&"/chart/dpi"),
        "{pointers:?}"
    );
    extra["version"] = json!("1.3");
    let (file, warnings) = read_json(&extra).unwrap();
    assert_eq!(codes_of(Ok((file.clone(), warnings))), ["SNR-SPEC-094"]);
    assert_eq!(file.edits, sample().edits);
}

/// §4, §8: edit IDs are `e<n>` and unique; `+e<n>` names an earlier `add`.
#[test]
fn edit_ids_and_references_must_be_consistent() {
    let shape = |mutate: &dyn Fn(&mut Value)| {
        let mut v = sample_json();
        mutate(&mut v);
        codes_of(read_json(&v))
    };
    assert_eq!(
        shape(&|v| v["edits"][3]["id"] = json!("e1")),
        ["SNR-SPEC-091"]
    );
    assert_eq!(
        shape(&|v| v["edits"][0]["id"] = json!("E1")),
        ["SNR-SPEC-091"]
    );
    assert_eq!(
        shape(&|v| v["edits"][0]["id"] = json!("e01")),
        ["SNR-SPEC-091"]
    );
    // A reference to an add the file does not define, and one to a later add.
    assert_eq!(
        shape(&|v| v["edits"][2]["point"] = json!("+e9")),
        ["SNR-SPEC-091"]
    );
    assert_eq!(
        shape(&|v| v["edits"][0]["point"] = json!("+e2")),
        ["SNR-SPEC-091"]
    );
    assert_eq!(
        shape(&|v| v["edits"][2]["point"] = json!("e2")),
        ["SNR-SPEC-091"]
    );
    // `before` is required where a point existed.
    assert_eq!(
        shape(&|v| {
            v["edits"][0].as_object_mut().unwrap().remove("before");
        }),
        ["SNR-SPEC-091"]
    );
}

/// §3, §5: the binding is a SHA-256, and the chart's name is a file name only.
#[test]
fn the_chart_binding_holds_no_paths() {
    let shape = |mutate: &dyn Fn(&mut Value)| {
        let mut v = sample_json();
        mutate(&mut v);
        codes_of(read_json(&v))
    };
    assert_eq!(
        shape(&|v| v["chart"]["sha256"] = json!("ABC")),
        ["SNR-SPEC-091"]
    );
    assert_eq!(
        shape(&|v| v["chart"]["sha256"] = json!(SHA.to_uppercase())),
        ["SNR-SPEC-091"]
    );
    for name in [
        "C:\\Users\\me\\rain.png",
        "/home/me/rain.png",
        "charts/rain.png",
        "",
    ] {
        assert_eq!(
            shape(&|v| v["chart"]["name"] = json!(name)),
            ["SNR-SPEC-091"],
            "{name}"
        );
    }
    assert_eq!(shape(&|v| v["chart"]["page"] = json!(0)), ["SNR-SPEC-091"]);
}

/// §8: duplicate keys, deep nesting and invalid JSON are shape errors; nothing is read.
#[test]
fn duplicate_keys_and_deep_nesting_are_refused() {
    let text = write(&sample()).replacen(
        "\"version\": \"1.0\",",
        "\"version\": \"1.0\",\n  \"version\": \"1.0\",",
        1,
    );
    assert_eq!(codes_of(read(text.as_bytes())), ["SNR-SPEC-091"]);
    let deep = format!("{}{}", "[".repeat(40), "]".repeat(40));
    assert_eq!(codes_of(read(deep.as_bytes())), ["SNR-SPEC-091"]);
    assert_eq!(codes_of(read(b"{\"format\": ")), ["SNR-SPEC-091"]);
}

/// §6, §8: the limits are `SNR-SPEC-092`, checked before anything is read.
#[test]
fn limits_are_refused() {
    let huge = vec![b' '; MAX_BYTES + 1];
    assert_eq!(codes_of(read(&huge)), ["SNR-SPEC-092"]);
    let mut long_note = sample_json();
    long_note["edits"][1]["note"] = json!("n".repeat(MAX_NOTE + 1));
    assert_eq!(codes_of(read_json(&long_note)), ["SNR-SPEC-092"]);
    let mut image = sample_json();
    image["image"] = json!({ "media": "image/png", "data": "A".repeat(MAX_IMAGE + 4), "region": { "min": [0, 0], "max": [10, 10] } });
    assert_eq!(codes_of(read_json(&image)), ["SNR-SPEC-092"]);
    let mut many = sample_json();
    let edit = many["edits"][0].clone();
    many["edits"] = Value::Array(
        (1..=MAX_EDITS + 1)
            .map(|n| {
                let mut e = edit.clone();
                e["id"] = json!(format!("e{n}"));
                e
            })
            .collect(),
    );
    assert_eq!(codes_of(read_json(&many)), ["SNR-SPEC-092"]);
}

/// §6: only a PNG, as base64.
#[test]
fn images_are_png_as_base64() {
    let with = |media: &str, data: &str| {
        let mut v = sample_json();
        v["image"] =
            json!({ "media": media, "data": data, "region": { "min": [0, 0], "max": [10, 10] } });
        codes_of(read_json(&v))
    };
    assert_eq!(with("image/png", "iVBORw0KGgo="), Vec::<String>::new());
    assert_eq!(with("image/jpeg", "iVBORw0KGgo="), ["SNR-SPEC-091"]);
    assert_eq!(with("image/png", "not base64!"), ["SNR-SPEC-091"]);
}

fn any_edits() -> impl Strategy<Value = Vec<FileEdit>> {
    prop::collection::vec((0_u8..4, 0_u32..50, -1e6_f64..1e6, "[a-z ]{0,12}"), 0..20).prop_map(
        |specs| {
            let mut adds = Vec::new();
            specs
                .into_iter()
                .enumerate()
                .map(|(i, (op, point, y, note))| {
                    let id = format!("e{}", i + 1);
                    let was: PointValue =
                        serde_json::from_value(json!({ "x": point, "y": y })).unwrap();
                    let point = adds.first().map_or(PointRef::Index(point), |a: &String| {
                        if point % 2 == 0 {
                            PointRef::Added(format!("+{a}"))
                        } else {
                            PointRef::Index(point)
                        }
                    });
                    let note = (!note.is_empty()).then_some(note);
                    let before = Before::new(ProvenanceClass::CvEstimate, Some(0.5));
                    match op {
                        0 => {
                            adds.push(id.clone());
                            FileEdit::Add {
                                id,
                                series: "s".into(),
                                to: was,
                                note,
                            }
                        }
                        1 => FileEdit::Move {
                            id,
                            series: "s".into(),
                            point,
                            was: was.clone(),
                            to: was,
                            before,
                            note,
                        },
                        2 => FileEdit::Delete {
                            id,
                            series: "s".into(),
                            point,
                            was,
                            before,
                            note,
                        },
                        _ => FileEdit::Retype {
                            id,
                            series: "s".into(),
                            point,
                            was,
                            to: serde_json::from_value(json!({ "series": "t" })).unwrap(),
                            before,
                            note,
                        },
                    }
                })
                .collect()
        },
    )
}

proptest! {
    /// RFC-0005 §12: any valid file reads back as written, and writes the same bytes again.
    #[test]
    fn any_file_round_trips(edits in any_edits()) {
        let mut file = CorrectionFile::new(ChartRef::new(SHA, "x", "1"));
        file.edits = edits;
        let text = write(&file);
        let (back, _) = read(text.as_bytes()).map_err(|e| TestCaseError::fail(format!("{e:?}")))?;
        prop_assert_eq!(&back, &file);
        prop_assert_eq!(write(&back), text);
        let _ = AnchorValue::Number(0.0);
    }
}
