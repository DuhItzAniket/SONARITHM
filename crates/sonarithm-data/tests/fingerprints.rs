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
const CORPUS_DIGEST: &str = "48ba3b4c2f4c3385f344a091e0727ef1f4f2bcf8663d672a753fa460ddc9d525";

/// The fingerprint of one file with dates, units and a header, with inference.
const PLAIN_WITH_INFERENCE: &str =
    "ac86b4ae456f6382c6bf158639b0f05d898df4899248c6c30154bf840c9c090f";

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
