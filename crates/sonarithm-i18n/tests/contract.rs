// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Rust half of the Rust–TypeScript contract (B02-T08, NFR-09): every case in
//! `tests/fixtures/contract.json` renders to its expected text here. `packages/i18n` runs
//! the same cases through `@fluent/bundle` and `Intl`, so a message ID renders the same on
//! both sides.

#![allow(
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::path::PathBuf;

use serde_json::Value;
use sonarithm_i18n::{Args, Localizer, Number, NumberOptions};

fn digits(object: &serde_json::Map<String, Value>, key: &str) -> Option<u8> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .map(|d| u8::try_from(d).unwrap_or_else(|_| panic!("{key} out of range")))
}

fn args(value: &Value) -> Args {
    let mut args = Args::new();
    let Some(object) = value.as_object() else {
        panic!("args must be an object: {value}");
    };
    for (name, arg) in object {
        args = match arg {
            Value::String(text) => args.text(name, text),
            Value::Object(number) => {
                let value = number
                    .get("number")
                    .and_then(Value::as_f64)
                    .unwrap_or_else(|| panic!("{name}: a number argument needs `number`"));
                let options = NumberOptions {
                    min_fraction: digits(number, "minimumFractionDigits"),
                    max_fraction: digits(number, "maximumFractionDigits"),
                    min_integer: digits(number, "minimumIntegerDigits"),
                    grouping: number
                        .get("useGrouping")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                };
                args.number_with(name, Number::with_options(value, options))
            }
            other => panic!("{name}: unsupported argument {other}"),
        };
    }
    args
}

#[test]
fn nfr_09_contract_cases_render_as_expected_in_rust() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
    let fixture: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}"));
    let cases = fixture["cases"]
        .as_array()
        .unwrap_or_else(|| panic!("`cases` must be an array"));
    assert!(
        cases.len() >= 20,
        "the contract covers the catalogue's features"
    );
    let mut failures = Vec::new();
    for case in cases {
        let locale = case["locale"].as_str().unwrap_or_default();
        let id = case["id"].as_str().unwrap_or_default();
        let expected = case["expected"].as_str().unwrap_or_default();
        let localizer = Localizer::new(locale).unwrap_or_else(|e| panic!("{e}"));
        match localizer.format(id, &args(&case["args"])) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => failures.push(format!("{locale} {id}: {actual:?} != {expected:?}")),
            Err(e) => failures.push(format!("{locale} {id}: {e}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
