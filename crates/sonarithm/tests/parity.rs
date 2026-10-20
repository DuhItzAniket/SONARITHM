// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Score parity across front ends: the Score the facade compiles for
//! `tests/golden/parity/temps.spec.json` is checked in next to it, with engine versions
//! written as `*`. The Python package's tests compare their Score with the same file, so
//! the facade, the CLI (which calls the facade) and Python agree value for value
//! (ADR-0013). The SHA-256 of its default WAV render is checked in too: audio carries no
//! version, so the hash is the bytes every front end must write on every platform.
//! Regenerate with `UPDATE_PARITY=1`.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::PathBuf;

use serde_json::Value;
use sonarithm::{Sonarithm, Spec};

fn parity_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/parity")
}

/// The Score as JSON with every engine version replaced by `*`.
fn normalized(score: &sonarithm::Score) -> String {
    let mut value = serde_json::to_value(score).unwrap();
    if let Some(engines) = value
        .pointer_mut("/header/engines")
        .and_then(Value::as_object_mut)
    {
        for version in engines.values_mut() {
            *version = Value::from("*");
        }
    }
    let mut text = serde_json::to_string_pretty(&value).unwrap();
    text.push('\n');
    text
}

#[test]
fn the_parity_score_is_current() {
    let dir = parity_dir();
    let spec = Spec::from_file(dir.join("temps.spec.json")).unwrap();
    let compiled = Sonarithm::new().compile(&spec).unwrap();
    let text = normalized(compiled.score());
    let expected = dir.join("temps.score.json");
    if std::env::var_os("UPDATE_PARITY").is_some() {
        std::fs::write(&expected, &text).unwrap();
    }
    let checked_in = std::fs::read_to_string(&expected)
        .unwrap()
        .replace("\r\n", "\n");
    assert!(
        checked_in == text,
        "tests/golden/parity/temps.score.json is stale; rerun with UPDATE_PARITY=1 and state the reason in the PR"
    );
}

#[test]
fn the_parity_render_is_current() {
    use sha2::{Digest, Sha256};

    let dir = parity_dir();
    let spec = Spec::from_file(dir.join("temps.spec.json")).unwrap();
    let snr = Sonarithm::new();
    let compiled = snr.compile(&spec).unwrap();
    let (wav, rate) = snr
        .render_wav(&compiled, &sonarithm::RenderOptions::default())
        .unwrap();
    assert_eq!(rate, 48_000);
    let digest = Sha256::digest(&wav)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            use std::fmt::Write as _;
            let _ = write!(hex, "{byte:02x}");
            hex
        });
    let expected = dir.join("temps.wav.sha256");
    if std::env::var_os("UPDATE_PARITY").is_some() {
        std::fs::write(
            &expected,
            format!(
                "{digest}
"
            ),
        )
        .unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(&expected).unwrap().trim(),
        digest,
        "tests/golden/parity/temps.wav.sha256 is stale; rerun with UPDATE_PARITY=1 and state the reason in the PR"
    );
}
