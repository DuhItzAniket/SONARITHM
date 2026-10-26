// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A03-T11: every JSON example in docs/architecture/04-sonification-spec.md validates, so
//! example drift breaks CI (A03 exit criterion: 100% of documented spec examples validate).
//!
//! - ```` ```json ```` blocks are complete specs: they must parse and validate without
//!   errors (warnings are allowed).
//! - ```` ```jsonc ```` blocks are fragments (`"key": value`) with `//` comments: each is
//!   placed into a small valid spec under its key and must then parse and validate.
//! - The §2 skeleton (`{ "specVersion": … }` with empty placeholder objects) is checked for
//!   its key set only.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use serde_json::{Map, Value, json};
use sonarithm_core::diagnostic::Diagnostic;
use sonarithm_spec::{FieldCatalog, parse::parse_value, validate};

const DOC: &str = include_str!("../../../docs/architecture/04-sonification-spec.md");

/// Top-level keys of RFC-0001 §2 and RFC-0004 §1.
const TOP_LEVEL: &[&str] = &[
    "specVersion",
    "name",
    "description",
    "mode",
    "seed",
    "data",
    "transform",
    "substrate",
    "tracks",
    "context",
    "analysis",
    "music",
    "playback",
    "output",
    "template",
    "instruments",
    "lint",
];

/// Fenced blocks as (info string, body, first line number).
fn blocks() -> Vec<(String, String, usize)> {
    let mut found = Vec::new();
    let mut current: Option<(String, String, usize)> = None;
    for (index, line) in DOC.lines().enumerate() {
        let trimmed = line.trim_start();
        match &mut current {
            None => {
                if let Some(info) = trimmed.strip_prefix("```")
                    && (info == "json" || info == "jsonc")
                {
                    current = Some((info.to_owned(), String::new(), index + 2));
                }
            }
            Some((_, body, _)) if trimmed.starts_with("```") => {
                let _ = body;
                found.push(current.take().unwrap());
            }
            Some((_, body, _)) => {
                body.push_str(line);
                body.push('\n');
            }
        }
    }
    found
}

/// Removes `//` comments outside strings.
fn strip_comments(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let mut in_string = false;
        let mut escaped = false;
        let mut cut = line.len();
        let bytes = line.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if in_string {
                match b {
                    _ if escaped => escaped = false,
                    b'\\' => escaped = true,
                    b'"' => in_string = false,
                    _ => {}
                }
            } else if b == b'"' {
                in_string = true;
            } else if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
                cut = i;
                break;
            }
        }
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}

/// A small valid spec that fragments are placed into.
fn base() -> Map<String, Value> {
    let value = json!({
        "specVersion": "0.2",
        "data": { "source": { "kind": "file", "path": "data/gistemp.csv" } },
        "tracks": [{ "id": "anomaly", "mark": "tone", "channels": { "pitch": { "field": "anomaly" } } }]
    });
    value.as_object().unwrap().clone()
}

fn errors(spec_value: Value, line: usize) -> Vec<String> {
    let spec = match parse_value(spec_value) {
        Ok(spec) => spec,
        Err(diagnostics) => return messages(diagnostics, line),
    };
    let catalog = FieldCatalog::from_spec(&spec);
    let found = validate(&spec, catalog.as_ref());
    messages(found.into_iter().filter(Diagnostic::is_error), line)
}

fn messages(diagnostics: impl IntoIterator<Item = Diagnostic>, line: usize) -> Vec<String> {
    diagnostics
        .into_iter()
        .map(|d| format!("04 line {line}: {d}"))
        .collect()
}

#[test]
fn every_json_example_in_04_validates() {
    let mut problems = Vec::new();
    let (mut complete, mut fragments, mut skeletons) = (0, 0, 0);
    for (info, body, line) in blocks() {
        let text = if info == "jsonc" {
            strip_comments(&body)
        } else {
            body
        };
        let trimmed = text.trim();
        if trimmed.starts_with('{') {
            let value: Value = serde_json::from_str(trimmed)
                .unwrap_or_else(|err| panic!("04 line {line}: not JSON: {err}"));
            if info == "jsonc" {
                // The §2 skeleton: check its keys.
                skeletons += 1;
                for key in value.as_object().unwrap().keys() {
                    if !TOP_LEVEL.contains(&key.as_str()) {
                        problems.push(format!("04 line {line}: unknown top-level key {key}"));
                    }
                }
            } else {
                complete += 1;
                problems.extend(errors(value, line));
            }
        } else {
            fragments += 1;
            let wrapped: Value = serde_json::from_str(&format!("{{{trimmed}}}"))
                .unwrap_or_else(|err| panic!("04 line {line}: fragment is not JSON: {err}"));
            let mut spec = base();
            for (key, value) in wrapped.as_object().unwrap() {
                spec.insert(key.clone(), value.clone());
                if key == "music" {
                    spec.insert("mode".to_owned(), json!("presentation"));
                }
            }
            problems.extend(errors(Value::Object(spec), line));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    // 04 has five complete examples (§6), thirteen section fragments (§3–§5) and one
    // skeleton.
    assert_eq!((complete, fragments, skeletons), (5, 13, 1));
}
