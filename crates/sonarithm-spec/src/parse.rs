// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Validation layers 1 and 2 (04 §8): JSON syntax, then the shape of the spec, with
//! `SNR-SPEC-001` … `005` diagnostics located by JSON pointer.

use serde_json::Value;
use serde_path_to_error::Segment;
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};
use sonarithm_core::score::SpecVersion;

use crate::codes;
use crate::spec::{SPEC_VERSION, Spec};

/// Parses spec text: JSON syntax (layer 1) and the typed shape (layer 2). Semantic checks
/// (layer 3) are [`crate::validate()`].
///
/// # Errors
///
/// One diagnostic: `SNR-SPEC-001` invalid JSON, `002` wrong type or value, `003` missing
/// key, `004` unknown key, or `005` unsupported `specVersion`.
pub fn parse(text: &str) -> Result<Spec, Diagnostics> {
    let value: Value = serde_json::from_str(text).map_err(|err| {
        one(Diagnostic::new(
            &codes::SYNTAX,
            format!(
                "Invalid JSON at line {}, column {}: {}.",
                err.line(),
                err.column(),
                strip_position(&err.to_string())
            ),
        ))
    })?;
    parse_value(value)
}

/// Parses an already-decoded JSON value (layer 2 only).
///
/// # Errors
///
/// As for [`parse`], without `SNR-SPEC-001`.
pub fn parse_value(value: Value) -> Result<Spec, Diagnostics> {
    check_version(&value).map_err(one)?;
    serde_path_to_error::deserialize::<_, Spec>(value).map_err(|err| {
        let pointer = pointer(err.path().iter());
        one(shape_diagnostic(&err.inner().to_string(), pointer))
    })
}

fn one(diagnostic: Diagnostic) -> Diagnostics {
    let mut all = Diagnostics::new();
    all.push(diagnostic);
    all
}

/// Checks `specVersion` before the typed parse, so an unsupported version gets `005`
/// rather than a confusing shape error from a newer document.
#[expect(
    clippy::result_large_err,
    reason = "private helper; the diagnostic moves into a list right away"
)]
fn check_version(value: &Value) -> Result<(), Diagnostic> {
    let at = SpecPointer::root().key("specVersion");
    let Some(object) = value.as_object() else {
        return Err(
            Diagnostic::new(&codes::INVALID_VALUE, "A spec is a JSON object.")
                .at(SpecPointer::root()),
        );
    };
    let Some(version) = object.get("specVersion") else {
        return Err(Diagnostic::new(
            &codes::MISSING_KEY,
            format!("The spec has no `specVersion`. Add \"specVersion\": \"{SPEC_VERSION}\"."),
        )
        .at(SpecPointer::root()));
    };
    let parsed = version.as_str().map(str::parse::<SpecVersion>);
    match parsed {
        Some(Ok(found)) if found == SPEC_VERSION => Ok(()),
        Some(Ok(found)) => Err(Diagnostic::new(
            &codes::UNSUPPORTED_VERSION,
            format!(
                "specVersion \"{found}\" is not supported; this SONARITHM reads \"{SPEC_VERSION}\"."
            ),
        )
        .at(at)),
        _ => Err(Diagnostic::new(
            &codes::INVALID_VALUE,
            format!("specVersion is written \"MAJOR.MINOR\", for example \"{SPEC_VERSION}\"."),
        )
        .at(at)),
    }
}

/// Maps a serde message to a code; serde's messages are stable in their first words.
fn shape_diagnostic(message: &str, at: SpecPointer) -> Diagnostic {
    let message = strip_position(message);
    let (code, text) = if let Some(rest) = message.strip_prefix("missing field ") {
        (&codes::MISSING_KEY, format!("Missing key {rest}."))
    } else if let Some(rest) = message.strip_prefix("unknown field ") {
        (&codes::UNKNOWN_KEY, format!("Unknown key {rest}."))
    } else {
        (&codes::INVALID_VALUE, format!("{}.", capitalize(message)))
    };
    Diagnostic::new(code, text).at(at)
}

/// Converts a `serde_path_to_error` path into an RFC 6901 pointer.
fn pointer<'a>(segments: impl Iterator<Item = &'a Segment>) -> SpecPointer {
    segments.fold(SpecPointer::root(), |pointer, segment| match segment {
        Segment::Seq { index } => pointer.index(*index),
        Segment::Map { key } => pointer.key(key),
        Segment::Enum { .. } | Segment::Unknown => pointer,
    })
}

/// Removes `serde_json`'s " at line L column C" suffix; positions come from the pointer.
fn strip_position(message: &str) -> &str {
    message
        .rfind(" at line ")
        .map_or(message, |index| &message[..index])
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(text: &str) -> Diagnostic {
        parse(text).unwrap_err().into_vec().remove(0)
    }

    const VALID: &str = r#"{ "specVersion": "0.1", "data": { "source": { "kind": "file", "path": "a.csv" } },
        "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "v" } } }] }"#;

    #[test]
    fn a_valid_spec_parses() {
        assert_eq!(parse(VALID).unwrap().tracks.len(), 1);
    }

    #[test]
    fn snr_spec_001_invalid_json_reports_line_and_column() {
        let diagnostic = error("{ \"specVersion\": \"0.1\",\n  \"data\": }");
        assert_eq!(diagnostic.code.as_str(), "SNR-SPEC-001");
        assert!(
            diagnostic
                .message
                .starts_with("Invalid JSON at line 2, column 11"),
            "{}",
            diagnostic.message
        );
    }

    #[test]
    fn snr_spec_002_wrong_types_are_located() {
        let diagnostic = error(&VALID.replace(r#""mark": "tone""#, r#""mark": "drum""#));
        assert_eq!(diagnostic.code.as_str(), "SNR-SPEC-002");
        assert_eq!(diagnostic.location.unwrap().as_str(), "/tracks/0/mark");
        assert_eq!(error("[]").code.as_str(), "SNR-SPEC-002");
        assert_eq!(
            error(&VALID.replace("\"0.1\"", "0.1")).code.as_str(),
            "SNR-SPEC-002"
        );
    }

    #[test]
    fn snr_spec_003_missing_keys_are_located() {
        let diagnostic = error(&VALID.replace(r#""mark": "tone", "#, ""));
        assert_eq!(diagnostic.code.as_str(), "SNR-SPEC-003");
        assert_eq!(diagnostic.location.unwrap().as_str(), "/tracks/0");
        assert_eq!(diagnostic.message, "Missing key `mark`.");
        assert_eq!(error(r#"{ "data": {} }"#).code.as_str(), "SNR-SPEC-003");
    }

    #[test]
    fn snr_spec_004_unknown_nested_keys_are_located() {
        let diagnostic = error(&VALID.replace("\"pitch\"", "\"pich\""));
        assert_eq!(diagnostic.code.as_str(), "SNR-SPEC-004");
        assert_eq!(
            diagnostic.location.unwrap().as_str(),
            "/tracks/0/channels/pich"
        );
        assert!(
            diagnostic.message.starts_with("Unknown key `pich`"),
            "{}",
            diagnostic.message
        );
    }

    #[test]
    fn snr_spec_005_other_versions_are_unsupported() {
        let diagnostic = error(&VALID.replace("\"0.1\"", "\"0.2\""));
        assert_eq!(diagnostic.code.as_str(), "SNR-SPEC-005");
        assert_eq!(diagnostic.location.unwrap().as_str(), "/specVersion");
        assert!(parse(VALID).is_ok());
    }
}
