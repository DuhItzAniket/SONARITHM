// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The spec migration framework (04 §7).
//!
//! A migration is a pure function on the JSON document that moves it one version step
//! forward and rewrites `specVersion`. Migrations run in order, in memory; the compiler
//! reports a `Warning` (`SNR-SPEC-006`) and never rewrites files. `sonarithm spec migrate`
//! (A07) writes the result only when asked.
//!
//! The 0.1 → 0.2 step is the identity (RFC-0004 §9), and so are 0.2 → 0.3 and 0.3 → 0.4:
//! only `specVersion` changes. Because of that, the compiler reads older documents as they
//! are ([`READABLE_VERSIONS`]), with no migration, no warning and an unchanged spec hash;
//! the steps run only when a file is migrated on purpose.

use serde_json::Value;
use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};
use sonarithm_core::score::SpecVersion;

use crate::codes;
use crate::spec::{READABLE_VERSIONS, SPEC_VERSION};

/// One step from `from` to `to`.
#[derive(Clone, Copy, Debug)]
pub struct Migration {
    /// The version this step reads.
    pub from: SpecVersion,
    /// The version this step writes.
    pub to: SpecVersion,
    /// Rewrites the document in place. It may assume `specVersion` is `from`; the
    /// framework sets it to `to` afterwards.
    pub apply: fn(&mut Value) -> Result<(), String>,
}

/// The migrations shipped with this crate, oldest first.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        from: SpecVersion::V0_1,
        to: SpecVersion::V0_2,
        apply: v0_1_to_v0_2,
    },
    Migration {
        from: SpecVersion::V0_2,
        to: SpecVersion::V0_3,
        apply: v0_2_to_v0_3,
    },
    Migration {
        from: SpecVersion::V0_3,
        to: SpecVersion::V0_4,
        apply: v0_3_to_v0_4,
    },
];

/// 0.1 → 0.2 (RFC-0004 §9): nothing but `specVersion` changes, which the framework sets.
#[expect(
    clippy::unnecessary_wraps,
    reason = "every migration step has this signature"
)]
fn v0_1_to_v0_2(_value: &mut Value) -> Result<(), String> {
    Ok(())
}

/// 0.2 → 0.3 (RFC-0005 Q6): 0.3 only allows `corrections.path`, so nothing but
/// `specVersion` changes.
#[expect(
    clippy::unnecessary_wraps,
    reason = "every migration step has this signature"
)]
fn v0_2_to_v0_3(_value: &mut Value) -> Result<(), String> {
    Ok(())
}

/// 0.3 → 0.4 (RFC-0006 §2): 0.4 only adds `experimental`, so nothing but `specVersion`
/// changes.
#[expect(
    clippy::unnecessary_wraps,
    reason = "every migration step has this signature"
)]
fn v0_3_to_v0_4(_value: &mut Value) -> Result<(), String> {
    Ok(())
}

/// Migrates `value` only when this crate cannot read its version as it is: documents of
/// [`READABLE_VERSIONS`] come back unchanged, with a one-element path.
///
/// # Errors
///
/// As for [`migrate`].
pub fn migrate_for_reading(value: Value) -> Result<Migrated, Box<Diagnostic>> {
    let readable = value
        .get("specVersion")
        .and_then(Value::as_str)
        .and_then(|text| text.parse::<SpecVersion>().ok())
        .filter(|version| READABLE_VERSIONS.contains(version));
    match readable {
        Some(version) => Ok(Migrated {
            value,
            path: vec![version],
        }),
        None => migrate(value),
    }
}

/// The result of migrating a document.
#[derive(Clone, Debug, PartialEq)]
pub struct Migrated {
    /// The migrated document.
    pub value: Value,
    /// The versions the document passed through, starting with its own; one element when
    /// no migration ran.
    pub path: Vec<SpecVersion>,
}

/// Migrates `value` to [`SPEC_VERSION`] with the shipped migrations.
///
/// # Errors
///
/// As for [`migrate_with`].
pub fn migrate(value: Value) -> Result<Migrated, Box<Diagnostic>> {
    migrate_with(value, MIGRATIONS, SPEC_VERSION)
}

/// Migrates `value` to `target` with `migrations`. A document already at `target` is
/// returned unchanged, so migrating is idempotent.
///
/// # Errors
///
/// `SNR-SPEC-003` without `specVersion`, `002` for a malformed one, and `005` when no chain
/// of migrations reaches `target` or a step fails.
pub fn migrate_with(
    mut value: Value,
    migrations: &[Migration],
    target: SpecVersion,
) -> Result<Migrated, Box<Diagnostic>> {
    let at = SpecPointer::root().key("specVersion");
    let mut version = match value.get("specVersion") {
        None => {
            return Err(Box::new(
                Diagnostic::new(&codes::MISSING_KEY, "The spec has no `specVersion`.")
                    .at(SpecPointer::root()),
            ));
        }
        Some(found) => found
            .as_str()
            .and_then(|text| text.parse::<SpecVersion>().ok())
            .ok_or_else(|| {
                Box::new(
                    Diagnostic::new(
                        &codes::INVALID_VALUE,
                        "specVersion is written \"MAJOR.MINOR\", for example \"0.1\".",
                    )
                    .at(at.clone()),
                )
            })?,
    };
    let mut path = vec![version];
    while version != target {
        let unsupported = || {
            Box::new(
                Diagnostic::new(
                    &codes::UNSUPPORTED_VERSION,
                    format!("specVersion \"{version}\" is not supported; this SONARITHM reads \"{target}\"."),
                )
                .at(at.clone()),
            )
        };
        let step = migrations
            .iter()
            .find(|step| step.from == version)
            .ok_or_else(unsupported)?;
        (step.apply)(&mut value).map_err(|_| unsupported())?;
        if let Some(object) = value.as_object_mut() {
            object.insert("specVersion".to_owned(), Value::String(step.to.to_string()));
        }
        version = step.to;
        path.push(version);
    }
    Ok(Migrated { value, path })
}

#[cfg(test)]
mod tests {
    /// RFC-0004 §9, RFC-0005 Q6, RFC-0006 §2: the 0.1 → 0.2 → 0.3 → 0.4 steps change
    /// `specVersion` and nothing else, and reading a 0.1 document does not migrate it.
    #[test]
    fn migrating_v0_1_changes_only_the_version_and_reading_does_not_migrate() {
        let old =
            serde_json::json!({ "specVersion": "0.1", "tracks": [{ "id": "a" }], "x-note": 1 });
        let migrated = super::migrate(old.clone()).unwrap();
        let mut expected = old.clone();
        expected["specVersion"] = serde_json::json!("0.4");
        assert_eq!(migrated.value, expected);
        assert_eq!(
            migrated.path,
            [
                super::SpecVersion::V0_1,
                super::SpecVersion::V0_2,
                super::SpecVersion::V0_3,
                super::SpecVersion::V0_4
            ]
        );
        let read = super::migrate_for_reading(old.clone()).unwrap();
        assert_eq!(
            (read.value, read.path),
            (old, vec![super::SpecVersion::V0_1])
        );
    }

    use serde_json::json;

    use super::*;

    const V0_0: SpecVersion = SpecVersion { major: 0, minor: 0 };

    /// A test step: 0.0 called the tracks `voices`.
    fn rename_voices(value: &mut Value) -> Result<(), String> {
        let object = value.as_object_mut().ok_or("not an object")?;
        if let Some(voices) = object.remove("voices") {
            object.insert("tracks".to_owned(), voices);
        }
        Ok(())
    }

    const TEST: &[Migration] = &[Migration {
        from: V0_0,
        to: SpecVersion::V0_1,
        apply: rename_voices,
    }];

    #[test]
    fn a_chain_migrates_in_memory_and_records_its_path() {
        let old = json!({ "specVersion": "0.0", "voices": [] });
        let migrated = migrate_with(old, TEST, SpecVersion::V0_1).unwrap();
        assert_eq!(
            migrated.value,
            json!({ "specVersion": "0.1", "tracks": [] })
        );
        assert_eq!(migrated.path, [V0_0, SpecVersion::V0_1]);
    }

    #[test]
    fn migrating_is_idempotent() {
        let old = json!({ "specVersion": "0.0", "voices": [1] });
        let once = migrate_with(old, TEST, SpecVersion::V0_1).unwrap().value;
        let twice = migrate_with(once.clone(), TEST, SpecVersion::V0_1).unwrap();
        assert_eq!(twice.value, once);
        assert_eq!(twice.path, [SpecVersion::V0_1]);
    }

    #[test]
    fn versions_without_a_chain_are_unsupported() {
        let newer = json!({ "specVersion": "0.5" });
        let err = migrate(newer).unwrap_err();
        assert_eq!(err.code.as_str(), "SNR-SPEC-005");
        let current = json!({ "specVersion": "0.4" });
        assert_eq!(migrate(current).unwrap().path, [SpecVersion::V0_4]);
        assert_eq!(
            migrate(json!({})).unwrap_err().code.as_str(),
            "SNR-SPEC-003"
        );
    }
}
