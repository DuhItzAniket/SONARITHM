// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A spec document as the facade handles it: the JSON text, a name for messages, and the
//! directory its `file` sources are relative to.

use std::path::{Path, PathBuf};

use serde_json::Value;
use sonarithm_core::Diagnostics;
use sonarithm_core::score::SpecVersion;
use sonarithm_spec::migrate::{MIGRATIONS, migrate_with};

use crate::error::Error;

/// A spec v0.1 document (RFC-0001).
///
/// A `Spec` always holds well-formed JSON; its shape and meaning are checked by
/// [`Spec::validate`] and by every compile. `file` sources resolve against
/// [`Spec::base_dir`]; a spec read with [`Spec::from_file`] uses its own directory (P8:
/// paths in specs are relative).
///
/// ```
/// let spec = sonarithm::Spec::from_json_str(r#"{ "specVersion": "0.1",
///     "data": { "source": { "kind": "inline", "values": [{ "x": 1, "y": 2 }] } },
///     "tracks": [{ "id": "line", "mark": "tone", "channels": { "pitch": { "field": "y" } } }] }"#)?;
/// assert!(!spec.validate().has_errors());
/// # Ok::<(), sonarithm::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    text: String,
    name: String,
    base_dir: Option<PathBuf>,
}

impl Spec {
    /// A spec from JSON text, named `spec.json` in messages.
    ///
    /// # Errors
    ///
    /// [`Error::Spec`] with an `SNR-SPEC-001` diagnostic when the text is not JSON.
    pub fn from_json_str(text: impl Into<String>) -> Result<Self, Error> {
        let text = text.into();
        if serde_json::from_str::<Value>(&text).is_err() {
            // The spec crate words the syntax error (line, column, cause).
            let diagnostics = sonarithm_spec::parse(&text).err().unwrap_or_default();
            return Err(Error::Spec(diagnostics));
        }
        Ok(Self {
            text,
            name: "spec.json".to_owned(),
            base_dir: None,
        })
    }

    /// A spec from a JSON value, such as a document built in code.
    #[must_use]
    pub fn from_value(value: &Value) -> Self {
        Self {
            text: value.to_string(),
            name: "spec.json".to_owned(),
            base_dir: None,
        }
    }

    /// Reads a spec file. Its name in messages is the file name, and its `file` sources
    /// are relative to its directory.
    ///
    /// # Errors
    ///
    /// [`Error::Read`] when the file cannot be read, and as [`Spec::from_json_str`].
    #[cfg(feature = "fs")]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let name = crate::error::file_name(path);
        let text = std::fs::read_to_string(path).map_err(|source| Error::Read {
            name: name.clone(),
            source,
        })?;
        let base = path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        Ok(Self::from_json_str(text)?
            .with_name(name)
            .with_base_dir(base))
    }

    /// The same spec under another name in messages. Only the last path component is
    /// kept (P8).
    #[must_use]
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = crate::error::file_name(Path::new(name.as_ref()));
        self
    }

    /// The same spec with `file` sources relative to `dir`.
    #[must_use]
    pub fn with_base_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(dir.into());
        self
    }

    /// The JSON text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The name used in messages and reports, such as `temps.spec.json`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The directory `file` sources are relative to, if any.
    #[must_use]
    pub fn base_dir(&self) -> Option<&Path> {
        self.base_dir.as_deref()
    }

    /// Validates the spec without data (04 §8 layers 1–3): syntax, shape, semantics, then
    /// the defaults it would get, as `Info` diagnostics. Field checks use the fields the
    /// spec declares; a compile also checks the data.
    #[must_use]
    pub fn validate(&self) -> Diagnostics {
        match sonarithm_spec::prepare(&self.text, None) {
            Ok(prepared) => prepared.diagnostics,
            Err(diagnostics) => diagnostics,
        }
    }

    /// Migrates the spec to the current spec version ([`sonarithm_spec::SPEC_VERSION`])
    /// in memory (04 §7). Writing the result back is the caller's choice.
    ///
    /// # Errors
    ///
    /// As [`Spec::migrate_to`].
    pub fn migrate(&self) -> Result<Migration, Error> {
        self.migrate_to(sonarithm_spec::SPEC_VERSION)
    }

    /// Migrates the spec to `target` with the shipped migrations.
    ///
    /// # Errors
    ///
    /// [`Error::Spec`] with `SNR-SPEC-003` when there is no `specVersion`, `002` when it is
    /// malformed, and `005` when no chain of migrations reaches `target`.
    pub fn migrate_to(&self, target: SpecVersion) -> Result<Migration, Error> {
        let value: Value = serde_json::from_str(&self.text).map_err(|_| {
            Error::Spec(sonarithm_spec::parse(&self.text).err().unwrap_or_default())
        })?;
        let migrated = migrate_with(value, MIGRATIONS, target).map_err(|diagnostic| {
            let mut one = Diagnostics::new();
            one.push(*diagnostic);
            Error::Spec(one)
        })?;
        let from = migrated.path.first().copied().unwrap_or(target);
        let changed = migrated.path.len() > 1;
        let text = if changed {
            let mut text = serde_json::to_string_pretty(&migrated.value)
                .map_err(|err| Error::Canonical(err.to_string()))?;
            text.push('\n');
            text
        } else {
            self.text.clone()
        };
        Ok(Migration {
            from,
            to: target,
            changed,
            text,
        })
    }
}

/// The result of [`Spec::migrate`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Migration {
    /// The version the spec had.
    pub from: SpecVersion,
    /// The version it has now.
    pub to: SpecVersion,
    /// Whether any migration ran; when not, `text` is the spec unchanged.
    pub changed: bool,
    /// The migrated spec, pretty-printed JSON.
    pub text: String,
}
