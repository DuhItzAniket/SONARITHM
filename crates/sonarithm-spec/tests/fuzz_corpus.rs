// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The committed fuzz seeds stay valid specs, and each is a fixed point of the canonical
//! form, so the fuzzer starts from documents that reach every pipeline stage.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::Path;

#[test]
fn every_seed_prepares_and_is_a_canonical_fixed_point() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/spec_parse");
    let mut seeds: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    seeds.sort();
    assert_eq!(
        seeds.len(),
        6,
        "4 examples from 04 plus every-section.json and every-section-0.2.json"
    );
    for seed in seeds {
        let text = std::fs::read_to_string(&seed).unwrap();
        let prepared = sonarithm_spec::prepare(&text, None)
            .unwrap_or_else(|d| panic!("{}: {d:#?}", seed.display()));
        let again = sonarithm_spec::prepare(&prepared.canonical, None).unwrap();
        assert_eq!(again.hash, prepared.hash, "{}", seed.display());
    }
}
