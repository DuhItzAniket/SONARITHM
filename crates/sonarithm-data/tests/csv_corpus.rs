// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A04-T02 acceptance: a corpus of messy CSVs ingests with the expected storage types.
//!
//! `tests/corpus/csv/expected.json` lists, per file, the source options, the row count, the
//! storage type of every field and the diagnostic codes the file must produce. The files
//! are synthetic and keep their exact bytes (BOM, CRLF, windows-1252) through
//! `.gitattributes`.

use std::collections::BTreeMap;
use std::path::Path;

use arrow_schema::DataType;
use serde::Deserialize;
use sonarithm_data::{CsvOptions, Format, ReadOptions, read_bytes};

#[derive(Deserialize)]
struct Expected {
    rows: u64,
    #[serde(default)]
    options: CsvOptions,
    fields: BTreeMap<String, Field>,
    /// Diagnostic codes the file must produce (others may appear too).
    codes: Vec<String>,
}

#[derive(Deserialize)]
struct Field {
    storage: String,
}

/// The storage names used in the manifest.
fn storage(data_type: &DataType) -> &'static str {
    match data_type {
        DataType::Boolean => "bool",
        DataType::Int64 => "int",
        DataType::Float64 => "float",
        DataType::Date32 => "date",
        DataType::Timestamp(_, None) => "datetime",
        DataType::Timestamp(_, Some(_)) => "instant",
        DataType::Utf8 => "text",
        _ => "other",
    }
}

#[test]
fn the_messy_csv_corpus_ingests_with_the_expected_types() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/csv");
    let manifest = std::fs::read_to_string(dir.join("expected.json")).unwrap();
    let corpus: BTreeMap<String, Expected> = serde_json::from_str(&manifest).unwrap();
    assert!(corpus.len() >= 20, "the corpus has {} files", corpus.len());
    let mut failures = Vec::new();
    for (file, expected) in &corpus {
        let bytes = std::fs::read(dir.join(file)).unwrap();
        let options = ReadOptions::new().with_csv(expected.options.clone());
        let loaded = match read_bytes(&bytes, Format::Csv, file, &options) {
            Ok(loaded) => loaded,
            Err(err) => {
                failures.push(format!("{file}: {err}"));
                continue;
            }
        };
        let codes: Vec<&str> = loaded.diagnostics.iter().map(|d| d.code.as_str()).collect();
        for code in &expected.codes {
            if !codes.contains(&code.as_str()) {
                failures.push(format!("{file}: no {code} in {codes:?}"));
            }
        }
        let table = loaded.bundle.primary().unwrap();
        if table.num_rows() != expected.rows {
            failures.push(format!(
                "{file}: {} rows, expected {}",
                table.num_rows(),
                expected.rows
            ));
        }
        let found: BTreeMap<String, &str> = table
            .schema()
            .fields()
            .iter()
            .map(|field| (field.name().clone(), storage(field.data_type())))
            .collect();
        let wanted: BTreeMap<String, &str> = expected
            .fields
            .iter()
            .map(|(name, field)| (name.clone(), field.storage.as_str()))
            .collect();
        if found != wanted {
            failures.push(format!("{file}: found {found:?}, expected {wanted:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
