// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! View tiles and table pages through the IPC protocol (B04-T03, 03 §7).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    reason = "integration tests on small data; tiles carry the data's exact values"
)]

use std::fmt::Write as _;
use std::sync::Arc;

use proptest::prelude::*;
use serde_json::json;
use sonarithm_app::ipc::binary::{ROW_COLUMN, Values, View, ViewKind};
use sonarithm_app::ipc::{Request, ViewMethod, ViewRange};
use sonarithm_app::testing::TempDir;
use sonarithm_app::{App, AppConfig, AudioOutput, NullSink};
use sonarithm_data::XRange;
use sonarithm_data::downsample::testing::raster;

fn app(dir: &TempDir) -> App {
    let config = AppConfig::new(dir.path().join("config")).with_audio(AudioOutput::null());
    App::new(config, Arc::new(NullSink)).unwrap()
}

/// Opens `csv` as a dataset and returns its ID.
fn open(app: &mut App, dir: &TempDir, name: &str, csv: &str) -> String {
    let path = dir.path().join(name);
    std::fs::write(&path, csv).unwrap();
    let reply = app
        .dispatch_json(&json!({ "command": "dataset_open", "args": { "path": path } }).to_string())
        .unwrap();
    reply["id"].as_str().unwrap().to_owned()
}

fn f64s(view: &View, name: &str) -> Vec<f64> {
    match &view.columns.iter().find(|c| c.name == name).unwrap().values {
        Values::F64(v) => v.clone(),
        other => panic!("{name} is {other:?}"),
    }
}

fn i64s(view: &View, name: &str) -> Vec<i64> {
    match &view.columns.iter().find(|c| c.name == name).unwrap().values {
        Values::I64(v) => v.clone(),
        other => panic!("{name} is {other:?}"),
    }
}

fn view_get(dataset: &str, range: Option<ViewRange>, width: u32, method: ViewMethod) -> Request {
    serde_json::from_value(json!({
        "command": "view_get",
        "args": { "dataset": dataset, "x": "t", "y": "v", "range": range,
                  "pxWidth": width, "method": method }
    }))
    .unwrap()
}

fn walk(steps: &[(f64, f64)]) -> (Vec<f64>, Vec<f64>, String) {
    let (mut t, mut v) = (0.0, 0.0);
    let mut csv = String::from("t,v\n");
    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    for (dt, dv) in steps {
        t += dt;
        v += dv;
        xs.push(t);
        ys.push(v);
        let _ = writeln!(csv, "{t:?},{v:?}");
    }
    (xs, ys, csv)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// A04's property through the protocol: drawn in row order, a tile's points give exactly
    /// the pixels of the full polyline (03 §7, F-043).
    #[test]
    fn tiles_are_pixel_exact_against_the_full_polyline(
        steps in proptest::collection::vec((0.01_f64..3.0, -5.0_f64..5.0), 2..300),
        width in 1_u32..100,
        height in 1_u32..60,
    ) {
        let dir = TempDir::new("tiles");
        let mut app = app(&dir);
        let (xs, ys, csv) = walk(&steps);
        let id = open(&mut app, &dir, "walk.csv", &csv);
        let range = ViewRange { start: xs[0], end: xs[xs.len() - 1] + 1e-9 };
        let bytes = app.dispatch_binary(view_get(&id, Some(range), width, ViewMethod::M4)).unwrap();
        let view = View::decode(&bytes).unwrap();
        prop_assert_eq!(view.kind, ViewKind::M4Tile);
        let rows: Vec<u64> = i64s(&view, "row").into_iter().map(|r| r as u64).collect();
        // The tile carries the data's own values at those rows.
        for (i, row) in rows.iter().enumerate() {
            prop_assert_eq!(f64s(&view, "x")[i], xs[*row as usize]);
            prop_assert_eq!(f64s(&view, "y")[i], ys[*row as usize]);
        }
        let all: Vec<u64> = (0..xs.len() as u64).collect();
        let xrange = XRange::new(range.start, range.end).unwrap();
        prop_assert_eq!(
            raster(&xs, &ys, &rows, xrange, width, height),
            raster(&xs, &ys, &all, xrange, width, height)
        );
    }
}

#[test]
fn zooming_uses_m4_on_the_visible_range_only() {
    let dir = TempDir::new("zoom");
    let mut app = app(&dir);
    let steps: Vec<(f64, f64)> = (0..1000).map(|i| (1.0, f64::from(i % 7) - 3.0)).collect();
    let (xs, _, csv) = walk(&steps);
    let id = open(&mut app, &dir, "long.csv", &csv);
    let zoom = ViewRange {
        start: 100.0,
        end: 200.0,
    };
    for _ in 0..2 {
        // The second request is served from the tile cache, with the same bytes.
        let view = View::decode(
            &app.dispatch_binary(view_get(&id, Some(zoom), 50, ViewMethod::M4))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(view.kind, ViewKind::M4Tile);
        assert!(view.rows <= 4 * 50);
        assert!(f64s(&view, "x").iter().all(|x| (100.0..=200.0).contains(x)));
    }
    let full = View::decode(
        &app.dispatch_binary(view_get(&id, None, 50, ViewMethod::M4))
            .unwrap(),
    )
    .unwrap();
    let x = f64s(&full, "x");
    assert_eq!(x.first().copied(), xs.first().copied());
    assert_eq!(x.last().copied(), xs.last().copied());
}

#[test]
fn an_lttb_overview_keeps_the_ends() {
    let dir = TempDir::new("lttb");
    let mut app = app(&dir);
    let steps: Vec<(f64, f64)> = (0..500).map(|i| (0.5, f64::from(i % 11) - 5.0)).collect();
    let (_, _, csv) = walk(&steps);
    let id = open(&mut app, &dir, "s.csv", &csv);
    let view = View::decode(
        &app.dispatch_binary(view_get(&id, None, 40, ViewMethod::Lttb))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(view.kind, ViewKind::LttbTile);
    let rows = i64s(&view, "row");
    assert_eq!(rows.len(), 40);
    assert_eq!((rows[0], rows[39]), (0, 499));
}

#[test]
fn table_pages_carry_row_indices_types_and_missing_values() {
    let dir = TempDir::new("pages");
    let mut app = app(&dir);
    let id = open(
        &mut app,
        &dir,
        "people.csv",
        "name,age,score\nAda,36,9.5\nGrace,,7.25\nLin,29,8.0\nMo,41,\n",
    );
    let request: Request = serde_json::from_value(json!({
        "command": "table_page",
        "args": { "dataset": id, "offset": 1, "limit": 2,
                  "sort": { "field": "age", "descending": true } }
    }))
    .unwrap();
    let view = View::decode(&app.dispatch_binary(request).unwrap()).unwrap();
    assert_eq!(view.kind, ViewKind::TablePage);
    assert_eq!(view.columns[0].name, ROW_COLUMN);
    // Ages descending: Mo 41, Ada 36, Lin 29, then Grace (missing) last; page 1..3.
    assert_eq!(i64s(&view, ROW_COLUMN), [0, 2]);
    let names = view.columns.iter().find(|c| c.name == "name").unwrap();
    assert_eq!(names.values, Values::Utf8(vec!["Ada".into(), "Lin".into()]));
    let page_all: Request = serde_json::from_value(json!({
        "command": "table_page", "args": { "dataset": id, "offset": 0, "limit": 10, "sort": null }
    }))
    .unwrap();
    let all = View::decode(&app.dispatch_binary(page_all).unwrap()).unwrap();
    let age = all.columns.iter().find(|c| c.name == "age").unwrap();
    assert_eq!(age.validity, Some(vec![true, false, true, true]));
}

#[test]
fn binary_and_json_commands_use_their_own_entry_points() {
    let dir = TempDir::new("entry");
    let mut app = app(&dir);
    let id = open(&mut app, &dir, "a.csv", "t,v\n1,2\n2,3\n");
    let err = app
        .dispatch(view_get(&id, None, 10, ViewMethod::M4))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-009");
    let err = app.dispatch_binary(Request::TransportState).unwrap_err();
    assert_eq!(err.code, "SNR-APP-009");
    let err = app
        .dispatch_binary(view_get("d9", None, 10, ViewMethod::M4))
        .unwrap_err();
    assert_eq!(err.code, "SNR-APP-002");
    let err = app
        .dispatch_binary(view_get(&id, None, 0, ViewMethod::M4))
        .unwrap_err();
    assert_eq!(err.code, "SNR-DATA-060");
}

#[test]
fn two_datasets_never_share_cached_tiles() {
    let dir = TempDir::new("cache");
    let mut app = app(&dir);
    let a = open(&mut app, &dir, "a.csv", "t,v\n0,1\n1,2\n2,3\n");
    let b = open(&mut app, &dir, "b.csv", "t,v\n0,-1\n1,-2\n2,-3\n");
    let range = Some(ViewRange {
        start: 0.0,
        end: 3.0,
    });
    let ya = f64s(
        &View::decode(
            &app.dispatch_binary(view_get(&a, range, 3, ViewMethod::M4))
                .unwrap(),
        )
        .unwrap(),
        "y",
    );
    let yb = f64s(
        &View::decode(
            &app.dispatch_binary(view_get(&b, range, 3, ViewMethod::M4))
                .unwrap(),
        )
        .unwrap(),
        "y",
    );
    assert_eq!(ya, [1.0, 2.0, 3.0]);
    assert_eq!(yb, [-1.0, -2.0, -3.0]);
}

/// AGENTS.md rule 7, ADR-0017 §6: a page of a table with provenance names every estimate.
#[test]
fn table_pages_carry_provenance_as_class_codes_and_confidences() {
    let dir = TempDir::new("provenance");
    let mut app = app(&dir);
    let id = open(
        &mut app,
        &dir,
        "est.csv",
        "t,v,__snr_prov,__snr_conf\n1,2,exact,\n2,3,cv_estimate,0.75\n3,4,derived,\n",
    );
    let request: Request = serde_json::from_value(json!({
        "command": "table_page", "args": { "dataset": id, "offset": 0, "limit": 10, "sort": null }
    }))
    .unwrap();
    let view = View::decode(&app.dispatch_binary(request).unwrap()).unwrap();
    let prov = view
        .columns
        .iter()
        .find(|c| c.name == "__snr_prov")
        .unwrap();
    assert_eq!(prov.values, Values::Provenance(vec![0, 2, 6]));
    let conf = view
        .columns
        .iter()
        .find(|c| c.name == "__snr_conf")
        .unwrap();
    assert_eq!(conf.values, Values::Confidence(vec![0.0, 0.75, 0.0]));
    assert_eq!(conf.validity, Some(vec![false, true, false]));
    // A table without provenance columns is all exact, and its pages say nothing more.
    let plain = open(&mut app, &dir, "plain.csv", "t,v\n1,2\n");
    let request: Request = serde_json::from_value(json!({
        "command": "table_page", "args": { "dataset": plain, "offset": 0, "limit": 10, "sort": null }
    }))
    .unwrap();
    let view = View::decode(&app.dispatch_binary(request).unwrap()).unwrap();
    assert!(
        view.columns
            .iter()
            .all(|c| !c.name.starts_with("__snr_prov"))
    );
}
