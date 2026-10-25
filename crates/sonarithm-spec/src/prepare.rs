// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The full spec pipeline the compiler runs: syntax, migration, shape, semantics, defaults,
//! then the canonical form and the spec hash (04 §7–8, ADR-0013).

use serde_json::Value;
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics};
use sonarithm_core::digest::Sha256Digest;

use crate::canonical::to_canonical;
use crate::codes;
use crate::defaults::apply_defaults;
use crate::migrate::migrate_for_reading;
use crate::parse::{one, parse_value, syntax};
use crate::spec::Spec;
use crate::validate::{FieldCatalog, validate};

/// A validated spec with its defaults, canonical form and hash.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Prepared {
    /// The spec as written (after any in-memory migration).
    pub spec: Spec,
    /// The spec with data-independent defaults applied.
    pub resolved: Spec,
    /// The RFC 8785 canonical form of `resolved`, recorded in render manifests.
    pub canonical: String,
    /// SHA-256 of `canonical`: the spec hash.
    pub hash: Sha256Digest,
    /// Warnings and the `Info` diagnostics of applied defaults, in order.
    pub diagnostics: Diagnostics,
}

/// Runs the whole pipeline on spec text. Without `fields`, the field list comes from the
/// spec itself where it can ([`FieldCatalog::from_spec`]).
///
/// # Errors
///
/// Every diagnostic so far (warnings included) when any of them is an error.
pub fn prepare(text: &str, fields: Option<&FieldCatalog>) -> Result<Prepared, Diagnostics> {
    let value: Value = serde_json::from_str(text).map_err(|err| one(syntax(&err)))?;
    prepare_value(value, fields)
}

/// [`prepare`] for an already-decoded JSON value.
///
/// # Errors
///
/// As for [`prepare`].
pub fn prepare_value(value: Value, fields: Option<&FieldCatalog>) -> Result<Prepared, Diagnostics> {
    let mut diagnostics = Diagnostics::new();
    let migrated = migrate_for_reading(value).map_err(|diagnostic| one(*diagnostic))?;
    if let [first, .., last] = migrated.path.as_slice() {
        diagnostics.push(Diagnostic::new(
            &codes::MIGRATED_VERSION,
            format!(
                "This spec is version \"{first}\" and was migrated to \"{last}\" in memory. Run `sonarithm spec migrate` to update the file."
            ),
        ));
    }
    let spec = parse_value(migrated.value)?;
    let own = if fields.is_none() {
        FieldCatalog::from_spec(&spec)
    } else {
        None
    };
    let fields = fields.or(own.as_ref());
    diagnostics.extend(validate(&spec, fields));
    if diagnostics.has_errors() {
        return Err(diagnostics);
    }
    let (resolved, applied) = apply_defaults(&spec);
    diagnostics.extend(applied);
    let canonical = to_canonical(&resolved).map_err(|err| {
        one(Diagnostic::new(
            &codes::INVALID_VALUE,
            format!("The spec has no canonical form: {err}."),
        ))
    })?;
    let hash = Sha256Digest::of(canonical.as_bytes());
    Ok(Prepared {
        spec,
        resolved,
        canonical,
        hash,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"{ "specVersion": "0.1", "data": { "source": { "kind": "file", "path": "temps.csv" } },
  "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "anomaly" } } }] }"#;

    /// The same spec with keys in another order, other whitespace and an explicit default.
    const PERMUTED: &str = r#"{"tracks":[{"channels":{"pitch":{"field":"anomaly"}},"mark":"tone","id":"y"}],
      "data":{"source":{"path":"temps.csv","kind":"file"}},"mode":"analysis","specVersion":"0.1"}"#;

    /// A03-T08 acceptance: the spec hash is stable across OSes (this test runs on all three
    /// CI OSes) and key-order permutations hash equal.
    #[test]
    fn spec_hash_is_pinned_and_ignores_key_order() {
        let prepared = prepare(MINIMAL, None).unwrap();
        assert_eq!(
            prepared.hash.to_string(),
            "8bc07b9c93786089301abf3af26f583ca6606add0e6fe4e028bb1738d45a9002"
        );
        assert_eq!(prepare(PERMUTED, None).unwrap().hash, prepared.hash);
        assert!(
            prepared
                .canonical
                .starts_with(r#"{"analysis":{"cue":"earcon""#)
        );
    }

    #[test]
    fn errors_stop_before_defaults() {
        let reversed = MINIMAL.replace(
            r#""field": "anomaly""#,
            r#""field": "anomaly", "range": { "hz": [1400, 400] }"#,
        );
        let diagnostics = prepare(&reversed, None).unwrap_err();
        assert!(diagnostics.has_errors());
        assert_eq!(
            prepare("{", None).unwrap_err().into_vec()[0].code.as_str(),
            "SNR-SPEC-001"
        );
    }

    #[test]
    fn applied_defaults_are_reported_as_info() {
        let prepared = prepare(MINIMAL, None).unwrap();
        assert!(!prepared.diagnostics.is_empty());
        assert!(
            prepared
                .diagnostics
                .iter()
                .all(|d| d.code.as_str() == "SNR-SPEC-080")
        );
    }
}
