// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RFC-0008 across crates: the data layer writes the same canonical JSON as
//! `sonarithm_spec::canonical` (it cannot depend on it), equal fingerprints compile alike,
//! and a dictionary column and a plain column with the same strings differ in fingerprint
//! (§1, §3).

#![allow(
    clippy::unwrap_used,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::collections::HashMap;
use std::sync::Arc;

use arrow_array::types::Int32Type;
use arrow_array::{ArrayRef, DictionaryArray, Float64Array, Int32Array, Int64Array, RecordBatch};
use arrow_array::{StringArray, StringViewArray};
use arrow_schema::{DataType, Field, Schema};
use sonarithm::{Datasets, Format, ReadOptions, Sonarithm, Spec, Table, read_bytes};
use sonarithm_core::{Sha256Digest, TableId};
use sonarithm_data::table_fingerprint;

/// The schema JSON hashed by scheme 2 is RFC 8785's form of the same value, as
/// `sonarithm-spec` writes it, for names and metadata that need escaping and UTF-16 key
/// order.
#[test]
fn rfc_0008_the_schema_is_hashed_in_the_spec_crates_canonical_form() {
    let name = "t\u{e4}\"\n\u{1}\u{1F600}";
    let field_meta = HashMap::from([
        ("snr.\u{1F600}".to_owned(), "x".to_owned()),
        ("snr.\u{FF5E}".to_owned(), "\\".to_owned()),
        ("snr.type".to_owned(), "quantitative".to_owned()),
        ("ARROW:extension:name".to_owned(), "left out".to_owned()),
    ]);
    let schema_meta = HashMap::from([("snr.function.expr".to_owned(), "x^2 / 2".to_owned())]);
    let schema = Schema::new(vec![
        Field::new(name, DataType::Int64, false).with_metadata(field_meta),
    ])
    .with_metadata(schema_meta);
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(Int64Array::from(vec![5])) as ArrayRef],
    )
    .unwrap();
    let got = table_fingerprint(&Table::from_batch(TableId(0), "t", batch)).unwrap();

    let header = sonarithm_spec::canonical::to_canonical(&serde_json::json!({
        "fields": [{
            "name": name,
            "type": "int64",
            "meta": {
                "snr.type": "quantitative",
                "snr.\u{FF5E}": "\\",
                "snr.\u{1F600}": "x",
            },
        }],
        "meta": { "snr.function.expr": "x^2 / 2" },
    }))
    .unwrap();
    let mut input = b"sonarithm-data table fingerprint v2\0".to_vec();
    input.extend_from_slice(&(header.len() as u64).to_le_bytes());
    input.extend_from_slice(header.as_bytes());
    input.extend_from_slice(&1_u64.to_le_bytes());
    input.extend_from_slice(&9_u64.to_le_bytes());
    input.push(1);
    input.extend_from_slice(&5_i64.to_le_bytes());
    assert_eq!(got, Sha256Digest::of(&input));
}

const SPEC: &str = r#"{
    "specVersion": "0.1",
    "data": { "source": { "kind": "dataset", "id": "main" } },
    "substrate": { "time": { "field": "day" } },
    "tracks": [
        { "id": "all", "mark": "tone",
          "channels": { "pitch": { "field": "temp" }, "timbre": { "field": "group" } } }
    ]
}"#;

/// The table with `group` as `column`, written as Arrow IPC and read back as a file is.
fn dataset(column: ArrayRef) -> sonarithm::DataBundle {
    let batch = RecordBatch::try_from_iter([
        (
            "day",
            Arc::new(Int64Array::from((1..=4).collect::<Vec<i64>>())) as ArrayRef,
        ),
        (
            "temp",
            Arc::new(Float64Array::from(vec![8.5, 9.25, 7.0, 6.5])) as ArrayRef,
        ),
        ("group", column),
    ])
    .unwrap();
    let table = Table::from_batch(TableId(0), "t", batch);
    let bytes = sonarithm_data::source::write_ipc(&table).unwrap();
    read_bytes(&bytes, Format::Arrow, "t.arrow", &ReadOptions::new())
        .unwrap()
        .bundle
}

fn compile(bundle: sonarithm::DataBundle) -> sonarithm_core::Score {
    let spec = Spec::from_json_str(SPEC)
        .unwrap()
        .with_name("group.spec.json");
    let datasets = Datasets::new().with("main", bundle);
    Sonarithm::new()
        .compile_with(&spec, &datasets)
        .unwrap()
        .score()
        .clone()
}

/// RFC-0008 §1 and §7: equal fingerprints compile alike (two string layouts), and a
/// dictionary column differs in fingerprint from a plain column with the same strings.
#[test]
fn rfc_0008_equal_fingerprints_compile_alike_and_a_dictionary_differs() {
    let rows = ["alpha", "zeta", "alpha", "zeta"];
    let plain = compile(dataset(Arc::new(StringArray::from(rows.to_vec()))));
    let view = compile(dataset(Arc::new(StringViewArray::from(rows.to_vec()))));
    assert_eq!(view.header.data_fingerprint, plain.header.data_fingerprint);
    assert_eq!(view, plain);

    // The same strings over a dictionary that lists "zeta" first: NUMISONA takes that as
    // the category order.
    let dictionary = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![1, 0, 1, 0]),
        Arc::new(StringArray::from(vec!["zeta", "alpha"])),
    )
    .unwrap();
    // Whether this spec's Score differs depends on how the encoder uses the order (here it
    // does not); the fingerprint must differ in any case, since a compile path may.
    let ordered = compile(dataset(Arc::new(dictionary)));
    assert_ne!(
        ordered.header.data_fingerprint,
        plain.header.data_fingerprint
    );
}
