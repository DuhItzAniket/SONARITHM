// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Polarity: whether higher values sound higher (SDR-12..15; 03 §3.3).
//!
//! A pitch channel's `polarity` is `positive`, `negative` or `auto`. For `auto`, the
//! field's **concept** indexes the concept table (`data/concepts.toml`, data rather than
//! code, with the evidence for every row). Resolution, in order:
//!
//! 1. An explicit `positive` or `negative` in the spec wins over every default.
//! 2. For `auto`, a polarity the listener's profile sets for the concept (SDR-15, SDR-38).
//! 3. For `auto`, the concept table's default. Concepts with no consistent direction
//!    (`avoid`) or no study yet (`unknown`, SDR-41) fall back to the generic default and
//!    are flagged.
//! 4. For `auto` without a concept, or with one the table does not know, the generic
//!    default: positive (SDR-12).
//!
//! The profile's one-key flip (SDR-15) then inverts the result for the fields it lists,
//! whatever its source. Every automatic choice is reported as an `Info` diagnostic that
//! names its rule and evidence (FR-026); a flagged concept read from the data gets a
//! warning (concepts declared in the spec get `SNR-SPEC-042` from validation instead).
//!
//! Polarity acts on the scale position: negative maps `u` to `1 − u` before the pitch
//! space, so a flip mirrors pitches within the range in the space's own coordinate. In the
//! default log-frequency space that is the mirror in cents of 05 §4.2 invariant 4,
//! `low + high − pitch` (F-080).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::OnceLock;

use serde::Deserialize;
use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};
use sonarithm_core::ids::RuleId;

use crate::codes;

/// The concept table shipped with NUMISONA.
const BUILTIN_TABLE: &str = include_str!("../data/concepts.toml");

/// The concept table's file format version.
const SCHEMA: u32 = 1;

/// The concept every field without one gets (SDR-12).
pub const GENERIC: &str = "generic";

/// The family row for `graph.*` concepts that have no row of their own (SDR-41).
const GRAPH_FAMILY: &str = "graph";

/// A resolved mapping direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Polarity {
    /// Higher values sound higher.
    Positive,
    /// Higher values sound lower.
    Negative,
}

impl Polarity {
    /// The opposite direction (the one-key flip, SDR-15).
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }

    /// The scale position `u` in this direction: `u` for positive, `1 − u` for negative.
    #[must_use]
    pub fn apply(self, u: f64) -> f64 {
        match self {
            Self::Positive => u,
            Self::Negative => 1.0 - u,
        }
    }

    /// The spec name, `positive` or `negative`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }
}

impl fmt::Display for Polarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The spec's `polarity` (RFC-0001 §6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PolarityChoice {
    /// From the field's concept (the default).
    #[default]
    Auto,
    /// Explicitly positive.
    Positive,
    /// Explicitly negative.
    Negative,
}

/// A concept's direction in the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TablePolarity {
    /// Higher values sound higher.
    Positive,
    /// Higher values sound lower.
    Negative,
    /// Listeners show no consistent direction; pitch is best avoided for the concept.
    Avoid,
    /// No magnitude-estimation study exists yet (SDR-41).
    Unknown,
}

/// How many listeners gave a positive, no, or negative polarity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Votes {
    /// Positive slopes.
    pub positive: u32,
    /// No polarity.
    pub none: u32,
    /// Negative slopes.
    pub negative: u32,
}

/// One row of the concept table.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ConceptRow {
    /// The concept name.
    pub name: String,
    /// The default direction.
    pub polarity: TablePolarity,
    /// Whether listeners disagree or no study exists, so a warning names the evidence.
    pub flagged: bool,
    /// The magnitude-estimation exponent m, offered as guidance for the psychophysical
    /// scale (SDR-11).
    #[serde(default)]
    pub exponent: Option<f64>,
    /// Listener counts, where a study reported them.
    #[serde(default)]
    pub votes: Option<Votes>,
    /// The design rules the row implements.
    pub rules: Vec<String>,
    /// The evidence, with its source.
    pub evidence: String,
}

impl ConceptRow {
    /// The polarity `auto` resolves to: the table's direction, or the generic positive
    /// default (SDR-12) for concepts marked `avoid` or `unknown`.
    #[must_use]
    pub const fn default_polarity(&self) -> Polarity {
        match self.polarity {
            TablePolarity::Negative => Polarity::Negative,
            TablePolarity::Positive | TablePolarity::Avoid | TablePolarity::Unknown => {
                Polarity::Positive
            }
        }
    }

    fn rule_ids(&self) -> Vec<RuleId> {
        self.rules
            .iter()
            .map(|rule| RuleId::new(rule.clone()))
            .collect()
    }

    /// "Walker 2007 counted 9 positive, 2 none, 8 negative" style vote text, if any.
    fn votes_text(&self) -> String {
        self.votes.map_or_else(String::new, |v| {
            format!(
                " Listener counts: {} positive, {} none, {} negative.",
                v.positive, v.none, v.negative
            )
        })
    }
}

/// Why the concept table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ConceptTableError {
    /// The TOML does not have the table's shape.
    #[error("the concept table cannot be read: {0}")]
    Parse(String),
    /// A row breaks a rule of the table.
    #[error("concept table row `{concept}`: {problem}")]
    Invalid {
        /// The row.
        concept: String,
        /// What is wrong.
        problem: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TableFile {
    schema: u32,
    concept: Vec<ConceptRow>,
}

/// The concept table: concept name → row, sorted by name.
#[derive(Clone, Debug, PartialEq)]
pub struct ConceptTable {
    rows: BTreeMap<String, ConceptRow>,
}

impl ConceptTable {
    /// The table shipped in `data/concepts.toml`, parsed once.
    ///
    /// # Errors
    ///
    /// The [`ConceptTableError`] if the shipped table is invalid (a build defect, caught by
    /// this crate's tests).
    pub fn builtin() -> Result<&'static Self, ConceptTableError> {
        static TABLE: OnceLock<Result<ConceptTable, ConceptTableError>> = OnceLock::new();
        TABLE
            .get_or_init(|| Self::parse(BUILTIN_TABLE))
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Parses and checks a concept table.
    ///
    /// # Errors
    ///
    /// [`ConceptTableError::Parse`] for malformed TOML; [`ConceptTableError::Invalid`] for a
    /// row without evidence or rules, a bad name, a repeated name, a non-positive exponent,
    /// an unflagged `avoid` or `unknown` row, or a missing or flagged `generic` row.
    pub fn parse(text: &str) -> Result<Self, ConceptTableError> {
        let file: TableFile =
            toml::from_str(text).map_err(|err| ConceptTableError::Parse(err.to_string()))?;
        if file.schema != SCHEMA {
            return Err(ConceptTableError::Parse(format!(
                "schema {} is not the supported schema {SCHEMA}",
                file.schema
            )));
        }
        let mut rows = BTreeMap::new();
        for row in file.concept {
            check_row(&row)?;
            if rows.contains_key(&row.name) {
                return Err(invalid(&row, "the concept appears twice"));
            }
            rows.insert(row.name.clone(), row);
        }
        match rows.get(GENERIC) {
            Some(row) if row.polarity == TablePolarity::Positive && !row.flagged => {}
            _ => {
                return Err(ConceptTableError::Invalid {
                    concept: GENERIC.to_owned(),
                    problem: "the table needs an unflagged, positive `generic` row (SDR-12)",
                });
            }
        }
        Ok(Self { rows })
    }

    /// The row for `concept`. A `graph.*` concept without a row of its own gets the
    /// `graph` family row (SDR-41).
    #[must_use]
    pub fn get(&self, concept: &str) -> Option<&ConceptRow> {
        self.rows.get(concept).or_else(|| {
            concept
                .strip_prefix("graph.")
                .and_then(|_| self.rows.get(GRAPH_FAMILY))
        })
    }

    /// The rows, by name.
    pub fn rows(&self) -> impl Iterator<Item = &ConceptRow> {
        self.rows.values()
    }
}

fn invalid(row: &ConceptRow, problem: &'static str) -> ConceptTableError {
    ConceptTableError::Invalid {
        concept: row.name.clone(),
        problem,
    }
}

/// Dot-separated lowercase words starting with a letter (the `Concept` rule of 03 §3.3).
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|word| {
            let mut chars = word.chars();
            chars.next().is_some_and(|c| c.is_ascii_lowercase())
                && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
}

/// `SDR-nn` or `A11Y-nn`.
fn valid_rule(rule: &str) -> bool {
    let digits = rule
        .strip_prefix("SDR-")
        .or_else(|| rule.strip_prefix("A11Y-"));
    digits.is_some_and(|d| d.len() == 2 && d.bytes().all(|b| b.is_ascii_digit()))
}

fn check_row(row: &ConceptRow) -> Result<(), ConceptTableError> {
    if !valid_name(&row.name) {
        return Err(invalid(
            row,
            "the name is not dot-separated lowercase words",
        ));
    }
    if row.evidence.trim().is_empty() {
        return Err(invalid(row, "every row cites its evidence"));
    }
    if row.rules.is_empty() || !row.rules.iter().all(|rule| valid_rule(rule)) {
        return Err(invalid(
            row,
            "every row names its design rules (SDR-nn or A11Y-nn)",
        ));
    }
    if row.exponent.is_some_and(|m| !(m.is_finite() && m > 0.0)) {
        return Err(invalid(row, "an exponent is a positive number"));
    }
    if matches!(row.polarity, TablePolarity::Avoid | TablePolarity::Unknown) && !row.flagged {
        return Err(invalid(
            row,
            "`avoid` and `unknown` concepts must be flagged",
        ));
    }
    Ok(())
}

/// The listener's polarity preferences, persisted per profile (SDR-15, SDR-38).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolarityProfile {
    /// The polarity `auto` resolves to for a concept, replacing the table's default (for
    /// example, a listener who hears money as negative).
    pub concepts: BTreeMap<String, Polarity>,
    /// Fields whose resolved polarity is flipped (the one-key flip).
    pub flipped_fields: BTreeSet<String>,
}

/// Where a field's concept came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConceptOrigin {
    /// `data.fields.<field>.concept` in the spec.
    Spec,
    /// The data's own metadata (`snr.concept`) or inference.
    Data,
}

/// What a polarity is resolved for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolarityRequest<'a> {
    /// The field mapped to pitch.
    pub field: &'a str,
    /// The field's concept and where it came from, if it has one.
    pub concept: Option<(&'a str, ConceptOrigin)>,
    /// The spec's `polarity`.
    pub choice: PolarityChoice,
}

/// Why a field got its polarity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolaritySource {
    /// The spec's explicit `positive` or `negative`.
    Explicit,
    /// The listener's profile chose it for the concept.
    Profile,
    /// The concept table's row for this concept.
    Concept(String),
    /// The generic default (SDR-12): no concept, or one the table does not know.
    Generic,
}

/// A resolved polarity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPolarity {
    /// The direction the encoder uses.
    pub polarity: Polarity,
    /// Where the direction came from, before any flip.
    pub source: PolaritySource,
    /// Whether the profile's one-key flip inverted it.
    pub flipped: bool,
}

/// Resolves the polarity of `request`, with the `Info` and warning diagnostics that
/// explain it, located at `at` (the channel's JSON pointer).
#[must_use]
pub fn resolve(
    table: &ConceptTable,
    profile: &PolarityProfile,
    request: &PolarityRequest<'_>,
    at: &SpecPointer,
) -> (ResolvedPolarity, Vec<Diagnostic>) {
    let field = request.field;
    let mut diagnostics = Vec::new();
    let (polarity, source) = match request.choice {
        PolarityChoice::Positive => (Polarity::Positive, PolaritySource::Explicit),
        PolarityChoice::Negative => (Polarity::Negative, PolaritySource::Explicit),
        PolarityChoice::Auto => {
            let (resolved, mut explained) = resolve_auto(table, profile, request, at);
            diagnostics.append(&mut explained);
            resolved
        }
    };
    let flipped = profile.flipped_fields.contains(field);
    let polarity = if flipped {
        diagnostics.push(
            Diagnostic::new(
                &codes::POLARITY_FROM_PROFILE,
                format!(
                    "NUMISONA: the profile flips the polarity of field `{field}` from {polarity} to {} (SDR-15).",
                    polarity.flipped()
                ),
            )
            .at(at.key("polarity")),
        );
        polarity.flipped()
    } else {
        polarity
    };
    (
        ResolvedPolarity {
            polarity,
            source,
            flipped,
        },
        diagnostics,
    )
}

fn resolve_auto(
    table: &ConceptTable,
    profile: &PolarityProfile,
    request: &PolarityRequest<'_>,
    at: &SpecPointer,
) -> ((Polarity, PolaritySource), Vec<Diagnostic>) {
    let field = request.field;
    let at = at.key("polarity");
    let Some((concept, origin)) = request.concept else {
        let message = format!(
            "NUMISONA: field `{field}` has no concept, so its polarity is positive, the generic default: higher values sound higher (SDR-12)."
        );
        let diagnostic = Diagnostic::new(&codes::POLARITY_FROM_CONCEPT, message).at(at);
        return (
            (Polarity::Positive, PolaritySource::Generic),
            vec![with_rules(diagnostic, &["SDR-12"])],
        );
    };
    if let Some(&polarity) = profile.concepts.get(concept) {
        let message = format!(
            "NUMISONA: the profile sets the polarity of concept `{concept}` to {polarity} for field `{field}` (SDR-15)."
        );
        let diagnostic = Diagnostic::new(&codes::POLARITY_FROM_PROFILE, message).at(at);
        return ((polarity, PolaritySource::Profile), vec![diagnostic]);
    }
    let Some(row) = table.get(concept) else {
        let message = format!(
            "NUMISONA: concept `{concept}` of field `{field}` is not in the concept table, so its polarity is positive, the generic default (SDR-12)."
        );
        let diagnostic = Diagnostic::new(&codes::UNKNOWN_CONCEPT, message).at(at);
        return (
            (Polarity::Positive, PolaritySource::Generic),
            vec![diagnostic],
        );
    };
    let polarity = row.default_polarity();
    let fallback = match row.polarity {
        TablePolarity::Avoid => {
            " Listeners show no consistent direction, so pitch is best avoided for this concept; the generic default applies."
        }
        TablePolarity::Unknown => {
            " No magnitude-estimation study exists yet (SDR-41); the generic default applies."
        }
        TablePolarity::Positive | TablePolarity::Negative => "",
    };
    let mut diagnostic = Diagnostic::new(
        &codes::POLARITY_FROM_CONCEPT,
        format!(
            "NUMISONA: field `{field}` is {polarity} from concept `{}` ({}).{fallback}{} Evidence: {}",
            row.name,
            row.rules.join(", "),
            row.votes_text(),
            row.evidence
        ),
    )
    .at(at.clone());
    diagnostic.rules = row.rule_ids();
    let mut diagnostics = vec![diagnostic];
    if row.flagged && origin == ConceptOrigin::Data {
        diagnostics.push(
            Diagnostic::new(
                &codes::FLAGGED_CONCEPT,
                format!(
                    "NUMISONA: listeners disagree on the pitch direction for concept `{concept}` of field `{field}`.{} Set \"polarity\" to \"positive\" or \"negative\", or flip it in your profile.",
                    row.votes_text()
                ),
            )
            .at(at),
        );
    }
    let source = PolaritySource::Concept(row.name.clone());
    ((polarity, source), diagnostics)
}

fn with_rules(mut diagnostic: Diagnostic, rules: &[&'static str]) -> Diagnostic {
    diagnostic.rules = rules.iter().map(|rule| RuleId::from_static(rule)).collect();
    diagnostic
}

#[cfg(test)]
mod tests;
