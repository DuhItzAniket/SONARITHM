// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Raster chart calibration through the IPC protocol (C02-T10; ADR-0017 "Raster
//! calibration"): open a PNG chart, check anchors as they are entered, apply the calibration
//! to open the chart as a dataset, and the session rules. The calibration in
//! `fixtures/raster` is the one PLOTSONA's accuracy harness clicks on the image's ticks
//! (DS-007, C02-T08).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests; clippy treats their helpers as non-test code"
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};
use sonarithm_app::ipc::IpcError;
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, CollectingSink};

struct Harness {
    app: App,
    dir: TempDir,
}

impl Harness {
    fn new(label: &str) -> Self {
        let dir = TempDir::new(label);
        let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
        let app = App::new(config, Arc::new(CollectingSink::new())).unwrap();
        Self { app, dir }
    }

    fn send(&mut self, request: &Value) -> Result<Value, IpcError> {
        self.app.dispatch_json(&request.to_string())
    }

    fn ok(&mut self, request: &Value) -> Value {
        self.send(request)
            .unwrap_or_else(|err| panic!("{request} failed: {err}"))
    }

    /// The DS-007 line chart at 1x, copied into the test folder as `name`.
    fn chart(&self, name: &str) -> PathBuf {
        self.corpus("d3-line@1x.png", name)
    }

    /// The DS-007 raster `image`, copied into the test folder as `name`.
    fn corpus(&self, image: &str, name: &str) -> PathBuf {
        let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../engines/sonarithm-plotsona/tests/corpus/raster")
            .join(image);
        let path = self.dir.path().join(name);
        std::fs::copy(corpus, &path).unwrap();
        path
    }

    fn open(&mut self, path: &Path) -> Value {
        self.ok(&json!({ "command": "chart_raster_open", "args": { "path": path } }))
    }
}

/// The calibration a person clicks on the chart's ticks (fixtures/raster).
fn calibration() -> Value {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/raster/d3-line@1x.calibration.json"),
    )
    .unwrap();
    serde_json::from_str(&text).unwrap()
}

/// The hand calibration of a DS-007 raster chart (`fixtures/raster/<stem>.calibration.json`).
fn calibration_of(stem: &str) -> Value {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/raster/{stem}.calibration.json")),
    )
    .unwrap();
    serde_json::from_str(&text).unwrap()
}

/// The dialog's form of a spec calibration: `{ x, y, plotArea? }` without `axes`.
fn entered(calibration: &Value) -> Value {
    json!({ "x": calibration["axes"]["x"], "y": calibration["axes"]["y"] })
}

#[test]
fn c02_t10_a_chart_image_opens_for_calibration_as_its_bytes_say() {
    let mut h = Harness::new("raster-open");
    let png = h.chart("rain.png");
    let opened = h.open(&png);
    assert_eq!(opened["reply"], "raster_chart");
    assert_eq!(opened["name"], "rain.png");
    assert!(opened["widthPx"].as_u64().unwrap() > 0);
    assert!(opened["heightPx"].as_u64().unwrap() > 0);
    // This chart's axis lines are not found, so it has no plot area; a bar chart's are.
    assert_eq!(opened["plotArea"], Value::Null);
    let bars = h.corpus("d3-bar@1x.png", "bars.png");
    let area = &h.open(&bars)["plotArea"];
    assert!(area["min"].is_array() && area["max"].is_array(), "{area}");
    assert!(
        opened["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,iVBOR")
    );
    // The media type comes from the bytes, never the extension.
    let misnamed = h.chart("rain.jpg");
    let opened = h.open(&misnamed);
    assert!(
        opened["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
}

#[test]
fn c02_t10_files_that_are_not_chart_images_are_refused_by_name_alone() {
    let mut h = Harness::new("raster-refuse");
    let folder = h.dir.path().to_string_lossy().into_owned();
    let not_image = h.dir.path().join("notes.png");
    std::fs::write(&not_image, "not an image").unwrap();
    let err = h
        .send(&json!({ "command": "chart_raster_open", "args": { "path": not_image } }))
        .unwrap_err();
    // Bytes no chart format starts with; an SVG is a chart format, but not an image.
    assert_eq!(err.code, "SNR-ENG-300", "{err}");
    let big = h.dir.path().join("huge.png");
    std::fs::write(&big, vec![0_u8; 21 * 1024 * 1024]).unwrap();
    let err = h
        .send(&json!({ "command": "chart_raster_open", "args": { "path": big } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-060", "{err}");
    assert!(err.message.contains("`huge.png`"), "{err}");
    let svg = h.dir.path().join("chart.svg");
    std::fs::write(&svg, "<svg/>").unwrap();
    let err = h
        .send(&json!({ "command": "chart_raster_open", "args": { "path": svg } }))
        .unwrap_err();
    // Refused by its name before it is read.
    assert_eq!(err.code, "SNR-APP-060", "{err}");
    assert!(!err.message.contains(&folder), "no path in {}", err.message);
}

#[test]
fn c02_t10_anchors_are_checked_as_they_are_entered() {
    let mut h = Harness::new("raster-check");
    let png = h.chart("rain.png");
    let id = h.open(&png)["id"].clone();
    let full = calibration();
    let mut partial = entered(&full);
    partial["x"]["anchors"] = json!([full["axes"]["x"]["anchors"][0]]);
    let first = &full["axes"]["y"]["anchors"][0];
    let checked = h.ok(&json!({ "command": "chart_calibration_check", "args": {
        "id": id, "calibration": partial, "at": first["at"] } }));
    assert_eq!(checked["reply"], "calibration_check");
    assert_eq!(checked["x"], Value::Null, "one anchor is not checked");
    assert_eq!(checked["y"]["anchors"].as_u64(), Some(2));
    assert_eq!(checked["diagnostics"], json!([]), "{checked}");
    assert_eq!(checked["valueAt"]["x"], Value::Null);
    let read = checked["valueAt"]["y"].as_f64().unwrap();
    let expected = first["value"].as_f64().unwrap();
    assert!(
        (read - expected).abs() < 1e-6 * expected.abs().max(1.0),
        "{read} {expected}"
    );

    // Ticks out of order: the axis is refused with its code, and nothing else changes.
    let mut unordered = entered(&full);
    let anchors = unordered["y"]["anchors"].as_array_mut().unwrap();
    let (a, b) = (anchors[0]["value"].clone(), anchors[1]["value"].clone());
    let half = a.as_f64().unwrap() * 0.5;
    anchors[0]["value"] = b;
    anchors[1]["value"] = a;
    anchors.push(json!({ "at": [0, 0], "value": half }));
    let checked = h.ok(&json!({ "command": "chart_calibration_check", "args": {
        "id": id, "calibration": unordered } }));
    assert_eq!(checked["y"], Value::Null);
    assert_eq!(checked["valueAt"], Value::Null);
    assert!(
        checked["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "SNR-ENG-322"),
        "{checked}"
    );
}

#[test]
fn c02_t10_applying_the_calibration_opens_the_chart_as_a_dataset() {
    let mut h = Harness::new("raster-apply");
    let png = h.chart("rain.png");
    let id = h.open(&png)["id"].clone();
    let applied = h.ok(&json!({ "command": "chart_calibration_apply", "args": {
        "id": id, "calibration": entered(&calibration()) } }));
    assert_eq!(applied["reply"], "dataset");
    assert_eq!(applied["id"], "d1");
    assert!(applied["rows"].as_u64().unwrap() > 0, "{applied}");
    assert!(
        applied["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "SNR-ENG-385"),
        "the extraction summary: {applied}"
    );
    // The correction document keeps the calibration, and an export carries it.
    let state = h.ok(&json!({ "command": "corrections_get", "args": { "dataset": "d1" } }));
    let file: Value = serde_json::from_str(state["text"].as_str().unwrap()).unwrap();
    assert!(
        file["calibration"]["axes"]["x"]["anchors"].is_array(),
        "{file}"
    );
    // The session ended with the apply.
    let err = h
        .send(&json!({ "command": "chart_raster_close", "args": { "id": id } }))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-096");
}

/// The behaviour change in `corrections.rs` (`reextract`): a correction reads a raster chart
/// again through the calibration its document keeps. Before C02-T10 the re-read passed no
/// calibration, so any correction to a raster chart failed with `SNR-ENG-328`; now an edit
/// that matches no point is reported as not applied and the chart keeps its rows.
#[test]
fn c02_t10_a_correction_reads_a_raster_chart_through_its_stored_calibration() {
    let mut h = Harness::new("raster-reextract");
    let png = h.chart("rain.png");
    let id = h.open(&png)["id"].clone();
    let applied = h.ok(&json!({ "command": "chart_calibration_apply", "args": {
        "id": id, "calibration": entered(&calibration()) } }));
    let corrected = h
        .send(&json!({ "command": "corrections_apply", "args": {
            "dataset": "d1", "label": "Delete", "patch": [{ "op": "add", "path": "/edits/-", "value": {
                "op": "delete", "id": "e1", "series": "Series 1", "point": 0,
                "was": { "y": -1e9 }, "before": { "class": "cv_estimate" } } }] } }))
        .unwrap_or_else(|err| panic!("the chart is read again through its calibration: {err}"));
    assert_eq!(corrected["notApplied"], json!(["e1"]), "{corrected}");
    assert_eq!(corrected["summary"]["rows"], applied["rows"]);
}

#[test]
fn c02_t10_a_fifth_image_ends_the_oldest_session() {
    let mut h = Harness::new("raster-sessions");
    let png = h.chart("rain.png");
    let ids: Vec<Value> = (0..5).map(|_| h.open(&png)["id"].clone()).collect();
    let check = |h: &mut Harness, id: &Value| {
        h.send(&json!({ "command": "chart_calibration_check", "args": {
            "id": id, "calibration": entered(&calibration()) } }))
    };
    assert_eq!(check(&mut h, &ids[0]).unwrap_err().code, "SNR-APP-096");
    for id in &ids[1..] {
        assert_eq!(check(&mut h, id).unwrap()["reply"], "calibration_check");
    }
    let closed = h.ok(&json!({ "command": "chart_raster_close", "args": { "id": ids[4] } }));
    assert_eq!(closed, json!({ "reply": "raster_closed", "id": ids[4] }));
    assert_eq!(check(&mut h, &ids[4]).unwrap_err().code, "SNR-APP-096");
}

/// Item 2 of the #517 review: the check gives each fitted axis's value at every pixel, so
/// the crosshair's slider says a step's value at once, from the same fit as the extraction.
#[test]
fn c02_t10_a_check_gives_every_pixels_value_on_a_fitted_axis() {
    let mut h = Harness::new("raster-values");
    let png = h.chart("rain.png");
    let opened = h.open(&png);
    let checked = h.ok(&json!({ "command": "chart_calibration_check", "args": {
        "id": opened["id"], "calibration": entered(&calibration()) } }));
    let xs = checked["values"]["x"].as_array().unwrap();
    let ys = checked["values"]["y"].as_array().unwrap();
    assert_eq!(xs.len() as u64, opened["widthPx"].as_u64().unwrap());
    assert_eq!(ys.len() as u64, opened["heightPx"].as_u64().unwrap());
    // A category axis gives labels; the y axis numbers.
    assert!(xs.iter().all(Value::is_string), "{}", xs[0]);
    assert!(ys.iter().all(Value::is_number), "{}", ys[0]);
    assert_eq!(checked["valueAt"], Value::Null, "no `at`, no single value");
}

/// Item 3 of the #517 review (#516): the area read is the one chosen, the anchors' box by
/// default, never the detected plot area unless asked for. On vl-area@1x the detected area is
/// the fill's, so asking for it warns (`SNR-ENG-330`), and the default reads the anchors' box.
#[test]
fn c02_t10_the_area_read_is_the_one_chosen_and_the_anchors_box_by_default() {
    let mut h = Harness::new("raster-area");
    let png = h.corpus("vl-area@1x.png", "area.png");
    let opened = h.open(&png);
    let mut chosen = entered(&calibration_of("vl-area@1x"));
    chosen["area"] = json!("detected");
    let checked = h.ok(&json!({ "command": "chart_calibration_check", "args": {
        "id": opened["id"], "calibration": chosen } }));
    let codes: Vec<&str> = checked["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["code"].as_str())
        .collect();
    if opened["plotArea"].is_object() {
        assert!(codes.contains(&"SNR-ENG-330"), "{checked}");
    }
    let applied = h.ok(&json!({ "command": "chart_calibration_apply", "args": {
        "id": opened["id"], "calibration": entered(&calibration_of("vl-area@1x")) } }));
    assert!(applied["rows"].as_u64().unwrap() > 0, "{applied}");
    // The box read is kept with the calibration: the anchors', not the detected one.
    let state = h.ok(&json!({ "command": "corrections_get", "args": { "dataset": applied["id"] } }));
    let file: Value = serde_json::from_str(state["text"].as_str().unwrap()).unwrap();
    let area = &file["calibration"]["plotArea"];
    assert!(area["min"].is_array(), "{file}");
    if let Some(detected) = opened["plotArea"].as_object() {
        assert_ne!(area["min"], detected["min"], "not the detected area");
    }
    let anchors = &calibration_of("vl-area@1x")["axes"]["x"]["anchors"];
    let leftmost = anchors
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["at"][0].as_f64())
        .fold(f64::INFINITY, f64::min);
    assert!(area["min"][0].as_f64().unwrap() < leftmost, "{area}");
}
