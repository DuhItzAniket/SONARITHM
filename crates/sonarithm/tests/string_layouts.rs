// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RFC-0008 §3 and §10: the same data in each of Arrow's string layouts (plain, large and
//! view) infers the same semantics and compiles to the same Score, data fingerprint
//! included, since fingerprint scheme 2 hashes values, not Arrow's bytes.

#![allow(
    clippy::unwrap_used,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::sync::Arc;

use arrow_array::{
    ArrayRef, Float64Array, Int64Array, LargeStringArray, RecordBatch, StringArray, StringViewArray,
};
use sonarithm::{Datasets, Format, ReadOptions, Sonarithm, Spec, Table, read_bytes};
use sonarithm_core::TableId;

const SPEC: &str = r#"{
    "specVersion": "0.1",
    "data": { "source": { "kind": "dataset", "id": "main" } },
    "substrate": { "time": { "field": "day" } },
    "tracks": [
        { "id": "all", "mark": "tone",
          "channels": { "pitch": { "field": "temp" }, "timbre": { "field": "region" } } },
        { "id": "north", "mark": "tone", "filter": { "expr": "region == \"north\"" },
          "channels": { "pitch": { "field": "temp" } } },
        { "id": "high", "mark": "tone", "filter": { "expr": "level == \"High\"" },
          "channels": { "pitch": { "field": "temp" } } }
    ]
}"#;

const REGION: [&str; 6] = ["north", "south", "north", "east", "south", "north"];
const LEVEL: [&str; 6] = ["High", "low", "Medium", "low", "High", "Medium"];

/// The table with its text columns in one layout, written as Arrow IPC and read back the
/// way a `.arrow` file is (with inference), so the layout survives into the data.
fn dataset(layout: fn(&[&str]) -> ArrayRef) -> sonarithm::DataBundle {
    let batch = RecordBatch::try_from_iter([
        (
            "day",
            Arc::new(Int64Array::from((1..=6).collect::<Vec<i64>>())) as ArrayRef,
        ),
        (
            "temp",
            Arc::new(Float64Array::from(vec![8.5, 9.25, 7.0, 6.5, 10.0, 9.0])) as ArrayRef,
        ),
        ("region", layout(&REGION)),
        ("level", layout(&LEVEL)),
    ])
    .unwrap();
    let table = Table::from_batch(TableId(0), "t", batch);
    let bytes = sonarithm_data::source::write_ipc(&table).unwrap();
    read_bytes(&bytes, Format::Arrow, "t.arrow", &ReadOptions::new())
        .unwrap()
        .bundle
}

#[test]
fn every_string_layout_compiles_to_the_same_score() {
    let layouts: [fn(&[&str]) -> ArrayRef; 3] = [
        |v| Arc::new(StringArray::from(v.to_vec())),
        |v| Arc::new(LargeStringArray::from(v.to_vec())),
        |v| Arc::new(StringViewArray::from(v.to_vec())),
    ];
    let spec = Spec::from_json_str(SPEC)
        .unwrap()
        .with_name("layouts.spec.json");
    let scores: Vec<String> = layouts
        .iter()
        .map(|layout| {
            let bundle = dataset(*layout);
            let region = bundle
                .primary()
                .unwrap()
                .schema()
                .field(2)
                .data_type()
                .clone();
            let datasets = Datasets::new().with("main", bundle);
            let compiled = Sonarithm::new().compile_with(&spec, &datasets).unwrap();
            let score = compiled.score();
            assert!(
                score.tracks.iter().all(|t| !t.events.is_empty()),
                "every track plays ({region})"
            );
            assert_eq!(score.header.data_fingerprint_scheme, 2);
            serde_json::to_string(score).unwrap()
        })
        .collect();
    for (other, name) in scores[1..].iter().zip(["LargeUtf8", "Utf8View"]) {
        assert!(*other == scores[0], "{name} compiles differently from Utf8");
    }
}
