// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Validation layer 3 (04 §8): semantic checks with `SNR-SPEC-0xx` codes and JSON
//! pointers. The data-aware checks (`070`, `071`) run when the field list is known
//! ([`FieldCatalog`]); the perceptual lint (layer 4) is ECHOMETRIC's.

mod channels;
mod sections;
mod v02;

use std::collections::{BTreeMap, BTreeSet};

use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, SpecPointer};

use crate::codes;
use crate::data::{FieldType, Source};
use crate::json::JsonValue;
use crate::spec::{Mode, Spec};

/// The largest seed every JSON implementation round-trips exactly: 2^53 − 1 (RFC-0001 Q7).
pub const MAX_SEED: u64 = (1 << 53) - 1;

/// Concepts whose polarity listeners disagree on, so `polarity: "auto"` warns
/// (SDR-13 size, SDR-14 money and attractiveness, SDR-41 graph concepts). NUMISONA's
/// concept table (A06) becomes the source of truth.
pub const FLAGGED_CONCEPTS: &[&str] = &["size", "money", "attractiveness"];

/// The fields a spec's data provides, with their types where known.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FieldCatalog {
    fields: BTreeMap<String, Option<FieldType>>,
}

impl FieldCatalog {
    /// An empty catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a field.
    pub fn insert(&mut self, name: impl Into<String>, field_type: Option<FieldType>) {
        self.fields.insert(name.into(), field_type);
    }

    /// The fields known from the spec alone: inline record keys (numbers are quantitative,
    /// strings nominal) or a function source's `x`, `y` and `dy`, plus `data.fields`
    /// declarations. `None` for sources whose fields are only known after loading.
    #[must_use]
    pub fn from_spec(spec: &Spec) -> Option<Self> {
        let mut catalog = Self::new();
        match &spec.data.source {
            Source::Inline { values } => {
                for (name, value) in values.iter().flatten() {
                    let inferred = match value {
                        JsonValue::Number(_) => Some(FieldType::Quantitative),
                        JsonValue::String(_) => Some(FieldType::Nominal),
                        _ => None,
                    };
                    let entry = catalog.fields.entry(name.clone()).or_insert(inferred);
                    if *entry != inferred && value != &JsonValue::Null {
                        *entry = None;
                    }
                }
            }
            Source::Function { .. } => {
                for name in ["x", "y", "dy"] {
                    catalog.insert(name, Some(FieldType::Quantitative));
                }
            }
            _ => return None,
        }
        for (name, field) in &spec.data.fields {
            if let Some(field_type) = field.field_type {
                catalog.insert(name.clone(), Some(field_type));
            }
        }
        Some(catalog)
    }

    /// The type of `name`: `None` if unknown, `Some(None)` if present but untyped.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Option<FieldType>> {
        self.fields.get(name).copied()
    }
}

/// Runs every semantic check and returns the diagnostics in document order. When `fields`
/// is given, field names and types are checked too.
#[must_use]
pub fn validate(spec: &Spec, fields: Option<&FieldCatalog>) -> Diagnostics {
    let mut out = Diagnostics::new();
    let root = SpecPointer::root();
    for key in spec.extensions.keys().filter(|key| !key.starts_with("x-")) {
        out.push(
            Diagnostic::new(
                &codes::UNKNOWN_KEY,
                format!("Unknown top-level key `{key}`. Vendor extensions start with `x-`."),
            )
            .at(root.key(key)),
        );
    }
    if spec.seed.is_some_and(|seed| seed > MAX_SEED) {
        out.push(
            Diagnostic::new(
                &codes::INVALID_VALUE,
                "The seed must be at most 2^53 − 1 (9007199254740991) so every JSON reader keeps it exact.",
            )
            .at(root.key("seed")),
        );
    }
    sections::data(spec, &mut out);
    sections::substrate(spec, &mut out);
    let ids = track_ids(spec, &mut out);
    for (index, track) in spec.tracks.iter().enumerate() {
        channels::track(spec, track, &root.key("tracks").index(index), &mut out);
    }
    sections::context(spec, &ids, &mut out);
    v02::check(spec, &ids, &mut out);
    sections::analysis(spec, &mut out);
    if spec.music.is_some() && spec.mode.unwrap_or(Mode::Analysis) != Mode::Presentation {
        out.push(
            Diagnostic::new(
                &codes::MUSIC_OUTSIDE_PRESENTATION,
                "`music` is allowed only in Presentation mode; Analysis keeps pitch unquantized (SDR-10). Set \"mode\": \"presentation\" or remove `music`.",
            )
            .at(root.key("music")),
        );
    }
    sections::playback_and_output(spec, &mut out);
    if let Some(fields) = fields {
        channels::fields(spec, fields, &mut out);
    }
    out
}

/// Checks track IDs (010–012) and returns the valid, unique ones.
fn track_ids(spec: &Spec, out: &mut Diagnostics) -> BTreeSet<String> {
    let tracks = SpecPointer::root().key("tracks");
    if spec.tracks.is_empty() && spec.template.is_none() {
        out.push(
            Diagnostic::new(
                &codes::NO_TRACKS,
                "The spec has no tracks. Add at least one track, or a `template` (spec v0.2).",
            )
            .at(tracks.clone()),
        );
    }
    let mut seen = BTreeSet::new();
    for (index, track) in spec.tracks.iter().enumerate() {
        let at = tracks.index(index).key("id");
        if !valid_id(&track.id) {
            out.push(
                Diagnostic::new(
                    &codes::INVALID_TRACK_ID,
                    format!(
                        "Track ID \"{}\" is invalid. Start with a letter and use only letters, digits, `_` and `-`, up to 64 characters.",
                        track.id
                    ),
                )
                .at(at),
            );
        } else if !seen.insert(track.id.clone()) {
            out.push(
                Diagnostic::new(
                    &codes::DUPLICATE_TRACK_ID,
                    format!(
                        "Track ID \"{}\" is used twice. Give every track a unique ID.",
                        track.id
                    ),
                )
                .at(at),
            );
        }
    }
    seen
}

/// `^[A-Za-z][A-Za-z0-9_-]{0,63}$` (RFC-0001 Q18).
fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && id.len() <= 64
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Whether a path is absolute or home-relative on any platform (P8).
fn is_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with('/')
        || path.starts_with('\\')
        || path.starts_with('~')
        || (bytes.len() > 2
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'/' || bytes[2] == b'\\'))
}

/// Warns about an absolute path (052).
fn path_check(path: &str, at: SpecPointer, out: &mut Diagnostics) {
    if is_absolute(path) {
        out.push(
            Diagnostic::new(
                &codes::ABSOLUTE_PATH,
                "This path is absolute. Use a path relative to the spec, so the spec carries no user paths and stays portable.",
            )
            .at(at),
        );
    }
}

/// Whether `x` is a finite number greater than zero.
fn positive(x: f64) -> bool {
    x.is_finite() && x > 0.0
}

#[cfg(test)]
mod tests;
