// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! #508: a dictionary column with no values (every key null, valid Arrow) compiles through
//! the real compiler. Arrow's `normalized_keys` panicked on it; its keys now read as
//! missing values.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

mod support;

use std::sync::Arc;

use arrow_array::{ArrayRef, DictionaryArray, Float64Array, Int8Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use serde_json::json;
use sonarithm_core::ids::TableId;
use sonarithm_data::{
    BundleId, DataBundle, FieldMeta, FieldType, Role, SourceDescriptor, SourceKind, Table,
};
use support::{compile, notes};

/// Four sales with a `fruit` category column whose dictionary is empty: every key is null.
fn all_null_fruit() -> DataBundle {
    let dictionary = DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8));
    let schema = Arc::new(Schema::new(vec![
        Field::new("sold", DataType::Float64, false),
        Field::new("fruit", dictionary, true),
    ]));
    let empty: ArrayRef = Arc::new(StringArray::from(Vec::<&str>::new()));
    let fruit = DictionaryArray::try_new(Int8Array::from(vec![None; 4]), empty).unwrap();
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(Float64Array::from(vec![42.0, 57.0, 18.0, 33.0])),
            Arc::new(fruit),
        ],
    )
    .unwrap();
    let table = Table::new(TableId(1), "fruit", schema, vec![batch])
        .unwrap()
        .with_field_meta(
            "fruit",
            &FieldMeta::new()
                .with_type(FieldType::Nominal)
                .with_role(Role::Category),
        )
        .unwrap()
        .with_field_meta(
            "sold",
            &FieldMeta::new()
                .with_type(FieldType::Quantitative)
                .with_role(Role::Y),
        )
        .unwrap();
    DataBundle::single(
        BundleId(1),
        SourceDescriptor::new(SourceKind::File, "fruit.json", "test"),
        table,
    )
}

#[test]
fn an_all_null_dictionary_column_compiles_as_missing_categories() {
    let spec = json!({
        "specVersion": "0.1",
        "data": { "source": { "kind": "file", "path": "fruit.json" } },
        "tracks": [{ "id": "data", "mark": "tone",
                     "channels": { "pitch": { "field": "sold" }, "timbre": { "field": "fruit" } } }],
    });
    let compiled = compile(&spec.to_string(), Some(all_null_fruit()));
    assert_eq!(notes(&compiled.score).len(), 4, "every sale still sounds; its category is missing");
}
