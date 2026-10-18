// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The canonical, full-precision serialization of a Score and its SHA-256 (05 §4.3).
//!
//! "Byte-identical Score" (05 §4.2 invariant 6, ADR-0013) refers to these bytes. The form
//! is compact JSON (no whitespace) as written by `serde_json`:
//!
//! - struct fields in declaration order, `BTreeMap` keys in sorted order, events with their
//!   `type` tag first, absent options as `null`;
//! - integers exactly; floats as the shortest decimal that round-trips (Ryū), so no
//!   precision is lost;
//! - only finite numbers: a Score that breaks an invariant has no canonical form.
//!
//! Any change to these bytes for the same Score is a format change and bumps
//! [`super::IR_VERSION`]. Golden test cases and render manifests record
//! [`Score::canonical_hash`]; the CI test matrix recomputes it on Linux, macOS and Windows.

use super::{InvariantViolation, Score};
use crate::digest::Sha256Digest;

/// Why a Score has no canonical form.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalError {
    /// The Score breaks an invariant.
    #[error("the Score breaks an invariant: {0}")]
    Invariant(#[from] InvariantViolation),
    /// Serialization failed.
    #[error("the Score could not be serialized: {0}")]
    Serialize(#[from] serde_json::Error),
}

impl Score {
    /// The canonical full-precision serialization.
    ///
    /// # Errors
    ///
    /// [`CanonicalError::Invariant`] when [`Score::check`] fails;
    /// [`CanonicalError::Serialize`] if serialization fails.
    pub fn canonical_json(&self) -> Result<Vec<u8>, CanonicalError> {
        self.check()?;
        Ok(serde_json::to_vec(self)?)
    }

    /// The SHA-256 of [`Score::canonical_json`].
    ///
    /// # Errors
    ///
    /// As for [`Score::canonical_json`].
    pub fn canonical_hash(&self) -> Result<Sha256Digest, CanonicalError> {
        Ok(Sha256Digest::of(&self.canonical_json()?))
    }
}
