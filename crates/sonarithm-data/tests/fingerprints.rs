// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A04-T09 acceptance: fingerprints are identical across operating systems for the corpus.
//!
//! The digests below were computed on Windows; CI checks them on Linux and macOS too. A
//! change here means the canonical form changed (for example an Arrow upgrade that encodes
//! IPC differently): record it in the changelog, because render manifests carry these
//! digests (ADR-0013). The corpus is read without semantic inference, so inference rule
//! changes do not move these digests; the full pipeline is covered by `loaded_tables_*`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;
use sonarithm_core::Sha256Digest;
use sonarithm_data::{CsvOptions, Format, ReadOptions, read_bytes, table_fingerprint};

/// The fingerprint of every corpus file (read without inference), hashed in file order.
const CORPUS_DIGEST: &str = "d6416b53d768aedfc5644aada58bd3863011bb8bbb6d4cc3a3362a35ed6221d0";

/// The fingerprint of one file with dates, units and a header, with inference.
const PLAIN_WITH_INFERENCE: &str =
    "405c630f942256e6e49caf93d6912f79b14d30a8163531a496c11eb0e35a4009";

#[derive(Deserialize)]
struct Expected {
    #[serde(default)]
    options: CsvOptions,
}

fn corpus() -> (std::path::PathBuf, BTreeMap<String, Expected>) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/csv");
    let manifest = std::fs::read_to_string(dir.join("expected.json")).unwrap_or_default();
    (dir, serde_json::from_str(&manifest).unwrap_or_default())
}

#[test]
fn corpus_fingerprints_are_pinned() {
    let (dir, corpus) = corpus();
    assert!(corpus.len() >= 20);
    let mut lines = String::new();
    for (file, expected) in &corpus {
        let bytes = std::fs::read(dir.join(file)).unwrap();
        let options = ReadOptions::new()
            .with_csv(expected.options.clone())
            .without_inference();
        let loaded = read_bytes(&bytes, Format::Csv, file, &options).unwrap();
        let digest = table_fingerprint(loaded.bundle.primary().unwrap()).unwrap();
        writeln!(lines, "{file} {digest}").unwrap();
    }
    println!("{lines}");
    let combined = Sha256Digest::of(lines.as_bytes()).to_string();
    assert_eq!(combined, CORPUS_DIGEST, "per-file digests:\n{lines}");
}

#[test]
fn loaded_tables_with_inference_are_pinned() {
    let (dir, _) = corpus();
    let bytes = std::fs::read(dir.join("01_plain.csv")).unwrap();
    let loaded = read_bytes(&bytes, Format::Csv, "01_plain.csv", &ReadOptions::new()).unwrap();
    let digest = table_fingerprint(loaded.bundle.primary().unwrap())
        .unwrap()
        .to_string();
    assert_eq!(digest, PLAIN_WITH_INFERENCE);
}
