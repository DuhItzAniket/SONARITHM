// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The shape of a stream's messages: which fields a row has and their types.
//!
//! A spec's stream `schema` (RFC-0001: an object) is read as a JSON Schema subset:
//! `{"type": "object", "properties": {"t": {"type": "number"}, …}, "required": [...]}`,
//! with property types `number`, `integer`, `string` and `boolean`. Fields are kept in
//! name order, so a schema always gives the same columns. Without a schema, the first
//! valid message decides the fields.

use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema, SchemaRef};
use serde_json::Value;

use crate::StreamError;

/// The most fields a message may have.
pub const MAX_FIELDS: usize = 256;

/// A field's value type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum FieldKind {
    /// Numbers (integers included), stored as 64-bit floats.
    Number,
    /// Text.
    Text,
    /// True or false.
    Boolean,
}

impl FieldKind {
    fn data_type(self) -> DataType {
        match self {
            Self::Number => DataType::Float64,
            Self::Text => DataType::Utf8,
            Self::Boolean => DataType::Boolean,
        }
    }
}

/// The fields of a stream's rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSchema {
    fields: Vec<(String, FieldKind, bool)>,
}

impl MessageSchema {
    /// A schema from `(name, kind, required)` fields, sorted by name.
    ///
    /// # Errors
    ///
    /// [`StreamError::Schema`] for no fields, more than [`MAX_FIELDS`], an empty name or a
    /// duplicate.
    pub fn new(fields: Vec<(String, FieldKind, bool)>) -> Result<Self, StreamError> {
        let mut fields = fields;
        fields.sort_by(|a, b| a.0.cmp(&b.0));
        if fields.is_empty() || fields.len() > MAX_FIELDS {
            return Err(StreamError::Schema(format!(
                "a message needs 1 to {MAX_FIELDS} fields"
            )));
        }
        if fields.iter().any(|f| f.0.is_empty()) || fields.windows(2).any(|w| w[0].0 == w[1].0) {
            return Err(StreamError::Schema(
                "field names must be unique and not empty".to_owned(),
            ));
        }
        Ok(Self { fields })
    }

    /// A schema from a spec's `schema` object (a JSON Schema subset).
    ///
    /// # Errors
    ///
    /// [`StreamError::Schema`] when the object has no `properties`, or a property has
    /// another type.
    pub fn from_json_schema(schema: &Value) -> Result<Self, StreamError> {
        let bad = |why: &str| StreamError::Schema(why.to_owned());
        let properties = schema
            .get("properties")
            .and_then(Value::as_object)
            .ok_or_else(|| bad("it has no `properties` object"))?;
        let required: Vec<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|names| names.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let mut fields = Vec::new();
        for (name, property) in properties {
            let kind = match property.get("type").and_then(Value::as_str) {
                Some("number" | "integer") => FieldKind::Number,
                Some("string") => FieldKind::Text,
                Some("boolean") => FieldKind::Boolean,
                _ => {
                    return Err(StreamError::Schema(format!(
                        "property `{name}` needs a type of number, integer, string or boolean"
                    )));
                }
            };
            fields.push((name.clone(), kind, required.contains(&name.as_str())));
        }
        Self::new(fields)
    }

    /// A schema inferred from one JSON object: numbers, strings and booleans become fields;
    /// nothing is required.
    ///
    /// # Errors
    ///
    /// [`StreamError::Schema`] when the object has no such value.
    pub fn infer(message: &serde_json::Map<String, Value>) -> Result<Self, StreamError> {
        let fields = message
            .iter()
            .filter_map(|(name, value)| {
                let kind = match value {
                    Value::Number(_) => FieldKind::Number,
                    Value::String(_) => FieldKind::Text,
                    Value::Bool(_) => FieldKind::Boolean,
                    _ => return None,
                };
                Some((name.clone(), kind, false))
            })
            .collect();
        Self::new(fields)
    }

    /// The fields, in name order: `(name, kind, required)`.
    #[must_use]
    pub fn fields(&self) -> &[(String, FieldKind, bool)] {
        &self.fields
    }

    /// The Arrow schema of the stream's rows; every field nullable.
    #[must_use]
    pub fn arrow(&self) -> SchemaRef {
        Arc::new(Schema::new(
            self.fields
                .iter()
                .map(|(name, kind, _)| Field::new(name, kind.data_type(), true))
                .collect::<Vec<_>>(),
        ))
    }
}
