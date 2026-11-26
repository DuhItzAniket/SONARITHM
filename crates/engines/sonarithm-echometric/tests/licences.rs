// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! C05-T01's licence check (AGENTS rule 8, ADR-0031): `cargo deny` refuses `anomaly_detection`
//! (GPL-3.0-or-later), so a change that adds it to any crate fails CI; the detectors' test
//! references, statrs and augurs, are not banned.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration test"
)]

use std::path::Path;

/// The crates `deny.toml`'s `[bans]` refuses.
fn banned() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../deny.toml");
    let text = std::fs::read_to_string(path).expect("deny.toml");
    let deny: toml::Table = text.parse().expect("deny.toml is TOML");
    deny["bans"]["deny"]
        .as_array()
        .expect("bans.deny")
        .iter()
        .filter_map(|entry| entry.get("crate").and_then(toml::Value::as_str))
        .map(str::to_owned)
        .collect()
}

#[test]
fn c05_t01_cargo_deny_refuses_anomaly_detection() {
    assert!(
        banned().iter().any(|c| c == "anomaly_detection"),
        "deny.toml must ban anomaly_detection (GPL-3.0-or-later)"
    );
}

#[test]
fn c05_t01_the_test_references_are_not_banned() {
    let banned = banned();
    for reference in ["statrs", "augurs"] {
        assert!(
            !banned.iter().any(|c| c == reference),
            "{reference} is a test reference (ADR-0031)"
        );
    }
}
