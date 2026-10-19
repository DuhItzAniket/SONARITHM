// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Field semantics (03 §3), stored in Arrow field metadata under the `snr.` prefix (03 §2).
//!
//! Arrow keeps field metadata through Parquet (in the embedded Arrow schema) and through
//! Arrow IPC, so a [`FieldMeta`] written with [`FieldMeta::apply`] survives both round
//! trips. Keys this module does not manage, including unknown `snr.*` keys from newer
//! versions, are left untouched.

use std::fmt;
use std::str::FromStr;

use arrow_schema::{Field, Metadata};
use serde::{Deserialize, Serialize};
use sonarithm_core::Diagnostic;

use crate::codes;

/// The metadata keys of [`FieldMeta`].
pub mod keys {
    /// The prefix shared by every SONARITHM metadata key.
    pub const PREFIX: &str = "snr.";
    /// [`FieldType`](super::FieldType), such as `quantitative`.
    pub const TYPE: &str = "snr.type";
    /// The unit as free text, such as `°C` (03 §3.2).
    pub const UNIT: &str = "snr.unit";
    /// The UCUM code of the unit, such as `Cel` (03 §3.2).
    pub const UNIT_UCUM: &str = "snr.unit.ucum";
    /// The [`Concept`](super::Concept), such as `temperature` (03 §3.3).
    pub const CONCEPT: &str = "snr.concept";
    /// The [`Role`](super::Role), such as `x` (03 §3.4).
    pub const ROLE: &str = "snr.role";
    /// A human-readable label for announcements and legends.
    pub const LABEL: &str = "snr.label";
    /// A longer description of the field.
    pub const DESCRIPTION: &str = "snr.description";

    /// Every key [`FieldMeta`](super::FieldMeta) reads and writes.
    pub const ALL: [&str; 7] = [TYPE, UNIT, UNIT_UCUM, CONCEPT, ROLE, LABEL, DESCRIPTION];
}

/// The longest metadata value quoted in a diagnostic, in characters.
const QUOTE_LIMIT: usize = 64;

/// What kind of quantity a field holds (03 §3.1). It drives the default channel choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum FieldType {
    /// Numbers on a ratio or interval scale.
    Quantitative,
    /// Dates, times and durations; numeric years.
    Temporal,
    /// Ordered categories.
    Ordinal,
    /// Unordered categories.
    Nominal,
}

/// A field role (03 §3.4). Chart templates and default specs use roles to pick encodings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Role {
    /// The horizontal (usually time) axis; the default time substrate (RFC-0001 §5).
    X,
    /// The measured value.
    Y,
    /// Splits rows into separate series.
    Series,
    /// A category label.
    Category,
    /// A weight for each row.
    Weight,
    /// A row identifier.
    Id,
    /// A human-readable row label.
    Label,
    /// A timestamp that is not the x axis.
    Time,
    /// The lower bound of an interval or error bar.
    Lower,
    /// The upper bound of an interval or error bar.
    Upper,
}

/// Generates `as_str`, `Display` and `FromStr` for a unit-only enum with lowercase names.
macro_rules! names {
    ($ty:ty, $error:literal, { $($variant:ident => $name:literal),+ $(,)? }) => {
        impl $ty {
            /// The name used in metadata and specs.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $ty {
            type Err = UnknownName;

            fn from_str(name: &str) -> Result<Self, Self::Err> {
                match name {
                    $($name => Ok(Self::$variant),)+
                    _ => Err(UnknownName { kind: $error }),
                }
            }
        }
    };
}

names!(FieldType, "field type", {
    Quantitative => "quantitative",
    Temporal => "temporal",
    Ordinal => "ordinal",
    Nominal => "nominal",
});

names!(Role, "role", {
    X => "x",
    Y => "y",
    Series => "series",
    Category => "category",
    Weight => "weight",
    Id => "id",
    Label => "label",
    Time => "time",
    Lower => "lower",
    Upper => "upper",
});

/// The error for a name that is not a known field type or role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown {kind}")]
pub struct UnknownName {
    kind: &'static str,
}

/// A unit: free text for announcements plus an optional UCUM code (03 §3.2).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Unit {
    /// The unit as written, such as `°C`.
    pub text: String,
    /// The UCUM code, such as `Cel`.
    pub ucum: Option<String>,
}

impl Unit {
    /// A unit given as free text, or `None` if the text is blank.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Option<Self> {
        let text = text.into();
        (!text.trim().is_empty()).then_some(Self { text, ucum: None })
    }

    /// The same unit with a UCUM code.
    #[must_use]
    pub fn with_ucum(mut self, code: impl Into<String>) -> Self {
        self.ucum = Some(code.into());
        self
    }
}

/// A concept: what a field measures in perceptual terms, such as `temperature` or
/// `graph.degree` (03 §3.3). It indexes the concept table, which is data owned by NUMISONA,
/// so this type only checks the spelling: dot-separated lowercase ASCII words that start
/// with a letter and may contain digits and `_`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Concept(String);

impl Concept {
    /// The concept used when nothing more specific applies (03 §3.3, SDR-12).
    #[must_use]
    pub fn generic() -> Self {
        Self("generic".to_owned())
    }

    /// A concept, or `None` if the name is not dot-separated lowercase words.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        let valid = !name.is_empty()
            && name.split('.').all(|word| {
                let mut chars = word.chars();
                chars.next().is_some_and(|c| c.is_ascii_lowercase())
                    && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            });
        valid.then_some(Self(name))
    }

    /// The concept name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Concept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Concept {
    type Error = InvalidConcept;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        Self::new(name).ok_or(InvalidConcept)
    }
}

impl From<Concept> for String {
    fn from(concept: Concept) -> Self {
        concept.0
    }
}

/// The error for a concept name that is not dot-separated lowercase words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("a concept is dot-separated lowercase words, such as `graph.degree`")]
pub struct InvalidConcept;

/// The semantics of one field (03 §3). Every part is optional: `None` means neither
/// declared nor inferred. Sources fill in `ftype` for every field they produce.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FieldMeta {
    /// The field type.
    pub ftype: Option<FieldType>,
    /// The unit.
    pub unit: Option<Unit>,
    /// The concept.
    pub concept: Option<Concept>,
    /// The role.
    pub role: Option<Role>,
    /// A human-readable label.
    pub label: Option<String>,
    /// A longer description.
    pub description: Option<String>,
}

impl FieldMeta {
    /// Metadata with nothing declared.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same metadata with a field type.
    #[must_use]
    pub fn with_type(mut self, ftype: FieldType) -> Self {
        self.ftype = Some(ftype);
        self
    }

    /// The same metadata with a unit.
    #[must_use]
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }

    /// The same metadata with a concept.
    #[must_use]
    pub fn with_concept(mut self, concept: Concept) -> Self {
        self.concept = Some(concept);
        self
    }

    /// The same metadata with a role.
    #[must_use]
    pub fn with_role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// The same metadata with a label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The same metadata with a description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Reads the `snr.*` metadata of a field. Values that cannot be read are skipped and
    /// returned as [`InvalidMetadata`], so a damaged file still loads.
    #[must_use]
    pub fn read(field: &Field) -> (Self, Vec<InvalidMetadata>) {
        Self::from_metadata(field.name(), field.metadata())
    }

    /// Reads `snr.*` keys from a metadata map; `field` names the field in problems.
    #[must_use]
    pub fn from_metadata(field: &str, metadata: &Metadata) -> (Self, Vec<InvalidMetadata>) {
        let mut meta = Self::default();
        let mut invalid = Vec::new();
        let mut reject = |key: &'static str, value: &str| {
            invalid.push(InvalidMetadata {
                field: field.to_owned(),
                key,
                value: quote(value),
            });
        };
        if let Some(value) = metadata.get(keys::TYPE) {
            match value.parse() {
                Ok(ftype) => meta.ftype = Some(ftype),
                Err(_) => reject(keys::TYPE, value),
            }
        }
        if let Some(value) = metadata.get(keys::UNIT) {
            match Unit::new(value.as_str()) {
                Some(unit) => meta.unit = Some(unit),
                None => reject(keys::UNIT, value),
            }
        }
        if let Some(value) = metadata.get(keys::UNIT_UCUM) {
            match (&mut meta.unit, value.trim().is_empty()) {
                (Some(unit), false) => unit.ucum = Some(value.clone()),
                _ => reject(keys::UNIT_UCUM, value),
            }
        }
        if let Some(value) = metadata.get(keys::CONCEPT) {
            match Concept::new(value.as_str()) {
                Some(concept) => meta.concept = Some(concept),
                None => reject(keys::CONCEPT, value),
            }
        }
        if let Some(value) = metadata.get(keys::ROLE) {
            match value.parse() {
                Ok(role) => meta.role = Some(role),
                Err(_) => reject(keys::ROLE, value),
            }
        }
        meta.label = metadata.get(keys::LABEL).cloned();
        meta.description = metadata.get(keys::DESCRIPTION).cloned();
        (meta, invalid)
    }

    /// Writes this metadata into a metadata map: keys for parts that are set are written,
    /// keys for parts that are `None` are removed, and every other key is kept.
    pub fn write_to(&self, metadata: &mut Metadata) {
        let ucum = self.unit.as_ref().and_then(|unit| unit.ucum.clone());
        let values = [
            (keys::TYPE, self.ftype.map(|t| t.as_str().to_owned())),
            (keys::UNIT, self.unit.as_ref().map(|unit| unit.text.clone())),
            (keys::UNIT_UCUM, ucum),
            (keys::CONCEPT, self.concept.as_ref().map(|c| c.0.clone())),
            (keys::ROLE, self.role.map(|role| role.as_str().to_owned())),
            (keys::LABEL, self.label.clone()),
            (keys::DESCRIPTION, self.description.clone()),
        ];
        for (key, value) in values {
            match value {
                Some(value) => {
                    metadata.insert(key, value);
                }
                None => {
                    metadata.remove(key);
                }
            }
        }
    }

    /// The field with this metadata written into its metadata map (see
    /// [`FieldMeta::write_to`]).
    #[must_use]
    pub fn apply(&self, field: Field) -> Field {
        let mut metadata = field.metadata().clone();
        self.write_to(&mut metadata);
        field.with_metadata(metadata)
    }
}

/// A `snr.*` metadata value that could not be read and was ignored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidMetadata {
    /// The field name.
    pub field: String,
    /// The metadata key.
    pub key: &'static str,
    /// The value, shortened to 64 characters.
    pub value: String,
}

impl InvalidMetadata {
    /// The warning that reports it (`SNR-DATA-040`).
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::new(
            &codes::INVALID_METADATA,
            format!(
                "Field `{}` has an unreadable `{}` value \"{}\"; it was ignored. Fix the value \
                 in the file or declare the field in the spec's `data.fields`.",
                self.field, self.key, self.value
            ),
        )
    }
}

/// A value shortened for quoting in a diagnostic.
fn quote(value: &str) -> String {
    match value.char_indices().nth(QUOTE_LIMIT) {
        Some((end, _)) => format!("{}…", &value[..end]),
        None => value.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use arrow_schema::DataType;
    use proptest::prelude::*;

    use super::*;

    fn full() -> FieldMeta {
        FieldMeta::new()
            .with_type(FieldType::Quantitative)
            .with_unit(Unit::new("°C").unwrap().with_ucum("Cel"))
            .with_concept(Concept::new("temperature").unwrap())
            .with_role(Role::Y)
            .with_label("Air temperature")
            .with_description("Hourly mean at 2 m")
    }

    #[test]
    fn metadata_round_trips_through_a_field() {
        let field = full().apply(Field::new("temp", DataType::Float64, true));
        assert_eq!(field.metadata()["snr.type"], "quantitative");
        assert_eq!(field.metadata()["snr.unit.ucum"], "Cel");
        let (read, invalid) = FieldMeta::read(&field);
        assert_eq!(read, full());
        assert_eq!(invalid, Vec::new());
    }

    #[test]
    fn writing_keeps_foreign_keys_and_removes_cleared_parts() {
        let mut metadata = Metadata::from([("pandas", "x"), ("snr.future", "kept")]);
        full().write_to(&mut metadata);
        FieldMeta::new()
            .with_type(FieldType::Nominal)
            .write_to(&mut metadata);
        let keys: Vec<_> = metadata.keys().map(String::as_str).collect();
        assert_eq!(keys, ["pandas", "snr.future", "snr.type"]);
    }

    #[test]
    fn unreadable_values_are_reported_and_skipped() {
        let metadata = Metadata::from([
            ("snr.type", "numeric"),
            ("snr.role", "Y"),
            ("snr.concept", "Temperature"),
            ("snr.unit.ucum", "Cel"),
            ("snr.label", "kept"),
        ]);
        let (meta, invalid) = FieldMeta::from_metadata("t", &metadata);
        assert_eq!(meta, FieldMeta::new().with_label("kept"));
        let mut keys: Vec<_> = invalid.iter().map(|i| i.key).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            ["snr.concept", "snr.role", "snr.type", "snr.unit.ucum"]
        );
        let diagnostic = invalid[0].to_diagnostic();
        assert_eq!(diagnostic.code.as_str(), "SNR-DATA-040");
        assert!(diagnostic.message.contains("Field `t`"));
    }

    #[test]
    fn long_values_are_shortened_in_diagnostics() {
        let metadata = Metadata::from([("snr.role", "é".repeat(100))]);
        let (_, invalid) = FieldMeta::from_metadata("t", &metadata);
        assert_eq!(invalid[0].value.chars().count(), QUOTE_LIMIT + 1);
    }

    #[test]
    fn concepts_are_dot_separated_lowercase_words() {
        for good in ["money", "graph.degree", "graph.edge_weight", "f2"] {
            assert!(Concept::new(good).is_some(), "{good}");
        }
        for bad in [
            "",
            "Money",
            "graph.",
            ".degree",
            "2d",
            "edge weight",
            "a..b",
        ] {
            assert!(Concept::new(bad).is_none(), "{bad}");
        }
        assert!(serde_json::from_str::<Concept>(r#""Bad""#).is_err());
        assert_eq!(Concept::generic().as_str(), "generic");
    }

    #[test]
    fn names_match_the_spec_vocabulary() {
        assert_eq!(serde_json::to_string(&Role::Lower).unwrap(), r#""lower""#);
        assert_eq!("temporal".parse::<FieldType>(), Ok(FieldType::Temporal));
        assert!("Temporal".parse::<FieldType>().is_err());
        assert_eq!(Role::Upper.to_string(), "upper");
        assert!(Unit::new("  ").is_none());
    }

    proptest! {
        #[test]
        fn any_label_and_description_round_trip(label in ".*", description in ".*") {
            let meta = FieldMeta::new().with_label(label).with_description(description);
            let field = meta.apply(Field::new("f", DataType::Utf8, true));
            prop_assert_eq!(FieldMeta::read(&field), (meta, Vec::new()));
        }
    }
}
