// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Binding a trial's stimulus into its condition's spec, and the protocol digest.
//!
//! A condition spec is data-agnostic (SDR-03): its data is the dataset [`STIMULUS`], which
//! the front end supplies from the trial's stimulus. [`bind_spec`] makes that so whatever
//! source the spec names, and keeps the spec's declared fields, which override the
//! semantics the stimulus carries.

use serde_json::{Value, json};
use sonarithm_core::digest::Sha256Digest;

/// The dataset ID a bound spec reads its data from.
pub const STIMULUS: &str = "stimulus";

/// Why a condition spec cannot take a stimulus.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BindError {
    /// The spec is not a JSON object.
    #[error("the condition spec is not a JSON object: {0}")]
    NotAnObject(String),
}

/// Returns the condition spec `spec_json` with its data source set to the dataset
/// [`STIMULUS`].
///
/// # Errors
///
/// [`BindError::NotAnObject`] if the text is not a JSON object.
pub fn bind_spec(spec_json: &str) -> Result<String, BindError> {
    let mut spec: Value =
        serde_json::from_str(spec_json).map_err(|err| BindError::NotAnObject(err.to_string()))?;
    let root = spec
        .as_object_mut()
        .ok_or_else(|| BindError::NotAnObject("the top level is not an object".to_owned()))?;
    let data = root.entry("data").or_insert_with(|| json!({}));
    if !data.is_object() {
        *data = json!({});
    }
    if let Some(data) = data.as_object_mut() {
        data.insert(
            "source".to_owned(),
            json!({ "kind": "dataset", "id": STIMULUS }),
        );
    }
    Ok(spec.to_string())
}

/// SHA-256 of a protocol file's bytes, as 64 hexadecimal digits: the `protocol_sha256` of
/// a session, which resumption checks.
#[must_use]
pub fn digest(protocol_text: &str) -> String {
    Sha256Digest::of(protocol_text.as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_replaces_the_source_and_keeps_the_fields() {
        let spec = r#"{"specVersion":"0.1","data":{"source":{"kind":"file","path":"x.csv"},"fields":{"price":{"unit":"$"}}},"tracks":[]}"#;
        let bound: Value = serde_json::from_str(&bind_spec(spec).unwrap()).unwrap();
        assert_eq!(
            bound["data"]["source"],
            json!({ "kind": "dataset", "id": "stimulus" })
        );
        assert_eq!(bound["data"]["fields"]["price"]["unit"], "$");
        let bare: Value =
            serde_json::from_str(&bind_spec(r#"{"specVersion":"0.1"}"#).unwrap()).unwrap();
        assert_eq!(bare["data"]["source"]["id"], "stimulus");
        assert!(bind_spec("[1]").is_err());
    }

    #[test]
    fn the_digest_is_sha256_hex() {
        assert_eq!(
            digest(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
