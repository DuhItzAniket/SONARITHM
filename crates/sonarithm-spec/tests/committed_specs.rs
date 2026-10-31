// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RFC-0004 §11 (amendment 1): every spec committed to the repository validates under the
//! v0.2 reader, whichever version it declares, and every v0.1 spec migrates to a document
//! equal to it except `specVersion`, while reading it keeps its spec hash.
//!
//! The specs are the JSON documents with top-level `specVersion` and `data` under the
//! folders below, except fixtures named `invalid…`, which are wrong on purpose: CLI
//! fixtures, the fuzz seeds, the examples gallery and the pilot study,
//! the replication gallery and the golden parity spec. Doc 04's examples are covered by
//! `doc_examples.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::{Path, PathBuf};

use serde_json::Value;

/// The folders, relative to the workspace root, that hold committed specs.
const ROOTS: &[&str] = &[
    "apps/cli/tests/fixtures",
    "crates/sonarithm-spec/fuzz/corpus",
    "examples",
    "replication",
    "tests/golden",
];

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            json_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            out.push(path);
        }
    }
}

/// Every committed spec, as (path, text, JSON).
fn committed_specs() -> Vec<(PathBuf, String, Value)> {
    let mut files = Vec::new();
    for root in ROOTS {
        json_files(&workspace().join(root), &mut files);
    }
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            // Fixtures named `invalid…` are wrong on purpose (the CLI's error tests).
            let name = path.file_name()?.to_string_lossy();
            if name.starts_with("invalid") {
                return None;
            }
            let text = std::fs::read_to_string(&path).ok()?;
            let value: Value = serde_json::from_str(&text).ok()?;
            let object = value.as_object()?;
            (object.contains_key("specVersion") && object.contains_key("data"))
                .then_some((path, text, value))
        })
        .collect()
}

#[test]
fn every_committed_spec_validates_under_the_v0_2_reader() {
    let specs = committed_specs();
    assert!(
        specs.len() >= 40,
        "found only {} committed specs",
        specs.len()
    );
    for (path, text, _) in &specs {
        let spec = sonarithm_spec::parse(text)
            .unwrap_or_else(|d| panic!("{} does not parse: {d:#?}", path.display()));
        let catalog = sonarithm_spec::FieldCatalog::from_spec(&spec);
        let errors: Vec<_> = sonarithm_spec::validate(&spec, catalog.as_ref())
            .into_iter()
            .filter(sonarithm_core::Diagnostic::is_error)
            .collect();
        assert!(errors.is_empty(), "{}: {errors:#?}", path.display());
    }
}

#[test]
fn every_committed_v0_1_spec_migrates_by_its_version_alone_and_keeps_its_hash() {
    let mut v0_1 = 0;
    for (path, text, value) in committed_specs() {
        if value["specVersion"] != "0.1" {
            continue;
        }
        v0_1 += 1;
        let migrated = sonarithm_spec::migrate::migrate(value.clone()).unwrap();
        let mut expected = value.clone();
        expected["specVersion"] = Value::from("0.3");
        assert_eq!(migrated.value, expected, "{}", path.display());
        // Reading the file does not migrate it: same version, same canonical form.
        let read = sonarithm_spec::migrate::migrate_for_reading(value.clone()).unwrap();
        assert_eq!(read.value, value, "{}", path.display());
        let parsed = sonarithm_spec::parse(&text).unwrap();
        assert_eq!(
            parsed.spec_version,
            sonarithm_core::score::SpecVersion::V0_1
        );
    }
    assert!(v0_1 >= 40, "found only {v0_1} committed v0.1 specs");
}

/// The replication gallery's manifests recorded each spec's hash before v0.2: the v0.2
/// reader gives the same hashes (RFC-0004 §9).
#[test]
fn recorded_spec_hashes_are_unchanged() {
    let mut checked = 0;
    let root = workspace().join("replication");
    for group in std::fs::read_dir(&root).unwrap() {
        let dir = group.unwrap().path();
        let Ok(text) = std::fs::read_to_string(dir.join("manifests.json")) else {
            continue;
        };
        let manifests: Value = serde_json::from_str(&text).unwrap();
        for (name, manifest) in manifests.as_object().unwrap() {
            let Some(recorded) = manifest["spec"]["hash"].as_str() else {
                continue;
            };
            let Ok(spec) = std::fs::read_to_string(dir.join(format!("{name}.json"))) else {
                continue;
            };
            let prepared = sonarithm_spec::prepare(&spec, None).unwrap();
            assert_eq!(
                prepared.hash.to_string(),
                recorded.trim_start_matches("sha256:"),
                "{}/{name}",
                dir.display()
            );
            checked += 1;
        }
    }
    assert!(checked >= 5, "checked only {checked} recorded hashes");
}
