// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Importers from chart grammars into SONARITHM spec v0.2 (C01-T10, 04 §9): Vega-Lite
//! ([`vegalite`]) and MAIDR ([`maidr`]).
//!
//! An importer turns a chart's data and encodings into a spec with the default design
//! (x → time, y → pitch, colour → timbre; SDR-05, A11Y-13): the data becomes an `inline`
//! or `file` source, and the chart type picks a chart-type template ([`crate::templates`])
//! bound to the chart's fields. Like the Erie importer, it **accounts for every construct**:
//! one that maps only approximately is imported and reported as
//! [`Outcome::Approximated`]; one with no equivalent is left out and reported as
//! [`Outcome::Unmapped`], with its JSON pointer in the source document. Keys that only
//! style the picture (sizes, padding, colours of marks, themes) have no sound equivalent;
//! each importer lists the ones it skips without a report.
//!
//! Importers read the document statically (OD-25): no script or expression runs, and a
//! data URL on the network is reported, never fetched (ADR-0012). Input passes the same
//! size, depth and point budgets as extraction.

pub mod maidr;
pub mod vegalite;

use std::fmt;

use serde_json::{Map, Value, json};
use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};

use crate::codes;

/// What became of one construct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    /// Imported with a change of meaning the note describes.
    Approximated,
    /// Not imported: SONARITHM has no equivalent.
    Unmapped,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Approximated => "approximated",
            Self::Unmapped => "unmapped",
        })
    }
}

/// One report line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportEntry {
    /// JSON pointer to the construct in the source document.
    pub pointer: String,
    /// The construct, such as `encoding.size` or `transform.calculate`.
    pub construct: String,
    /// What became of it.
    pub outcome: Outcome,
    /// Why, and what the user can do.
    pub note: String,
}

impl fmt::Display for ReportEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} at {}: {}",
            self.outcome, self.construct, self.pointer, self.note
        )
    }
}

/// Everything an importer could not map exactly, in the order it met it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    entries: Vec<ReportEntry>,
}

impl ImportReport {
    /// The entries, in document order.
    #[must_use]
    pub fn entries(&self) -> &[ReportEntry] {
        &self.entries
    }

    /// Whether everything mapped exactly.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The report as diagnostics: `SNR-ENG-381` warnings for unmapped constructs and
    /// `SNR-ENG-382` notes for approximated ones, located by their pointer in the source
    /// document (not in the SONARITHM spec), in report order.
    #[must_use]
    pub fn diagnostics(&self, importer: &str) -> Vec<Diagnostic> {
        self.entries
            .iter()
            .map(|entry| {
                let code = match entry.outcome {
                    Outcome::Unmapped => &codes::IMPORT_UNMAPPED,
                    Outcome::Approximated => &codes::IMPORT_APPROXIMATED,
                };
                let pointer = entry
                    .pointer
                    .split('/')
                    .skip(1)
                    .map(|part| part.replace("~1", "/").replace("~0", "~"))
                    .fold(SpecPointer::root(), |at, part| at.key(&part));
                Diagnostic::new(
                    code,
                    format!(
                        "PLOTSONA {importer} import: `{}` {}: {}",
                        entry.construct, entry.outcome, entry.note
                    ),
                )
                .at(pointer)
            })
            .collect()
    }

    pub(crate) fn unmapped(&mut self, pointer: &str, construct: &str, note: impl Into<String>) {
        self.push(pointer, construct, Outcome::Unmapped, note);
    }

    pub(crate) fn approximated(&mut self, pointer: &str, construct: &str, note: impl Into<String>) {
        self.push(pointer, construct, Outcome::Approximated, note);
    }

    fn push(&mut self, pointer: &str, construct: &str, outcome: Outcome, note: impl Into<String>) {
        self.entries.push(ReportEntry {
            pointer: pointer.to_owned(),
            construct: construct.to_owned(),
            outcome,
            note: note.into(),
        });
    }
}

/// An imported chart: one SONARITHM spec and the report.
#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    /// The spec (v0.2 JSON).
    pub spec: Value,
    /// What could not be mapped exactly.
    pub report: ImportReport,
}

/// A JSON pointer token: `~` and `/` escaped (RFC 6901).
pub(crate) fn token(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// The spec an importer builds.
#[derive(Debug, Default)]
pub(crate) struct Draft {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) source: Value,
    pub(crate) fields: Map<String, Value>,
    pub(crate) transforms: Vec<Value>,
    /// The template ID and its bindings.
    pub(crate) template: Option<(&'static str, Map<String, Value>)>,
    /// Sections written out when no template fits.
    pub(crate) sections: Map<String, Value>,
}

impl Draft {
    /// Declares `field`'s type and label (`data.fields`).
    pub(crate) fn field(&mut self, field: &str, ftype: &str, label: Option<&str>) {
        let entry = self
            .fields
            .entry(field.to_owned())
            .or_insert_with(|| json!({}));
        entry["type"] = json!(ftype);
        if let Some(label) = label.filter(|l| !l.is_empty()) {
            entry["label"] = json!(label);
        }
    }

    /// Uses the template `id` with `bind` (slot → field).
    pub(crate) fn template(&mut self, id: &'static str, bind: &[(&str, &str)]) {
        let bind = bind
            .iter()
            .map(|(slot, field)| ((*slot).to_owned(), json!(field)))
            .collect();
        self.template = Some((id, bind));
    }

    /// The spec.
    pub(crate) fn finish(self) -> Value {
        let mut spec = Map::new();
        spec.insert("specVersion".to_owned(), json!("0.2"));
        if let Some(name) = self.name {
            spec.insert("name".to_owned(), json!(name));
        }
        if let Some(description) = self.description {
            spec.insert("description".to_owned(), json!(description));
        }
        let mut data = Map::new();
        data.insert("source".to_owned(), self.source);
        if !self.fields.is_empty() {
            data.insert("fields".to_owned(), Value::Object(self.fields));
        }
        spec.insert("data".to_owned(), Value::Object(data));
        if !self.transforms.is_empty() {
            spec.insert("transform".to_owned(), Value::Array(self.transforms));
        }
        if let Some((id, bind)) = self.template {
            let mut template = json!({ "id": id });
            if !bind.is_empty() {
                template["bind"] = Value::Object(bind);
            }
            spec.insert("template".to_owned(), template);
        }
        spec.extend(self.sections);
        Value::Object(spec)
    }
}

/// A title as text: a string, or the `text` of a title object (a list of lines joined).
pub(crate) fn text_of(value: Option<&Value>) -> Option<String> {
    let value = value?;
    let text = match value {
        Value::String(text) => text.clone(),
        Value::Array(lines) => lines
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" "),
        Value::Object(object) => return text_of(object.get("text")),
        _ => return None,
    };
    let text = text.trim().to_owned();
    (!text.is_empty()).then_some(text)
}

/// The import error for a document that is not the expected shape.
pub(crate) fn unreadable(importer: &str, why: impl fmt::Display) -> Box<Diagnostic> {
    Box::new(Diagnostic::new(
        &codes::IMPORT_UNREADABLE,
        format!("PLOTSONA {importer} import: {why}."),
    ))
}
