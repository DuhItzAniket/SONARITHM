// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Template slots and how they bind to a table's fields (RFC-0004 §2, 03 §3.4).
//!
//! A template names **slots** (an x field, a value field, a category field, …) instead of
//! fields, so one design fits any dataset with a compatible schema (SDR-03). Binding gives
//! each slot a field: first the slots the spec binds by name, then every slot with a role
//! takes an unused field with that role, then each remaining slot, in order, takes the
//! first unused field of its kind, preferring fields without a role, in schema order.
//! Reserved `__snr_` columns are never bound. NUMISONA's and PLOTSONA's templates share
//! these rules.

use std::collections::BTreeMap;

use arrow_schema::DataType;

use crate::meta::{FieldType, Role};
use crate::table::Table;

/// What a slot accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldKind {
    /// Quantitative or temporal fields: numbers.
    Number,
    /// Nominal or ordinal fields: categories.
    Category,
    /// Either.
    Any,
}

impl FieldKind {
    /// The kind as an adjective for messages: `numeric`, `categorical` or `any`.
    #[must_use]
    pub const fn adjective(self) -> &'static str {
        match self {
            Self::Number => "numeric",
            Self::Category => "categorical",
            Self::Any => "any",
        }
    }

    /// Whether a slot of this kind accepts a field of kind `field`.
    #[must_use]
    pub fn accepts(self, field: Self) -> bool {
        self == Self::Any || self == field
    }
}

/// One slot of a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// The slot name, such as `x`.
    pub name: &'static str,
    /// What the slot accepts.
    pub kind: FieldKind,
    /// The field role that binds to the slot first (03 §3.4).
    pub role: Option<Role>,
    /// Whether the template works without it.
    pub optional: bool,
}

impl Slot {
    /// A required slot.
    #[must_use]
    pub const fn new(name: &'static str, kind: FieldKind, role: Option<Role>) -> Self {
        Self {
            name,
            kind,
            role,
            optional: false,
        }
    }

    /// The same slot, optional.
    #[must_use]
    pub const fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// Slot name → field name.
pub type Bindings = BTreeMap<String, String>;

/// Why slots cannot be bound.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SlotError {
    /// A binding names an unknown slot, a field the data does not have, or a field of the
    /// wrong kind.
    #[error("slot `{slot}` cannot take field `{field}`: {reason}")]
    BadBinding {
        /// The slot.
        slot: String,
        /// The field.
        field: String,
        /// Why.
        reason: &'static str,
    },
    /// A required slot has no compatible field.
    #[error("slot `{slot}` needs a {} field, and none is left", kind.adjective())]
    Unbound {
        /// The slot.
        slot: &'static str,
        /// What it needs.
        kind: FieldKind,
    },
}

impl From<SlotError> for sonarithm_core::engine::BindingError {
    /// The error as the compiler reports it (`SNR-COMP-031`, `032`).
    fn from(err: SlotError) -> Self {
        match err {
            SlotError::Unbound { slot, kind } => Self::Unbound {
                slot: slot.to_owned(),
                kind: kind.adjective().to_owned(),
            },
            SlotError::BadBinding {
                slot,
                field,
                reason,
            } => Self::Invalid {
                slot,
                field,
                reason: reason.to_owned(),
            },
        }
    }
}

/// The kind of `table`'s field: its declared type, else its Arrow type. `None` for a field
/// the table does not have or whose type is neither numbers nor labels.
#[must_use]
pub fn field_kind(table: &Table, field: &str) -> Option<FieldKind> {
    let meta = table.field_meta(field).ok()?;
    match meta.ftype {
        Some(FieldType::Quantitative | FieldType::Temporal) => Some(FieldKind::Number),
        Some(FieldType::Nominal | FieldType::Ordinal) => Some(FieldKind::Category),
        _ => {
            let schema = table.schema();
            let data_type = schema.field_with_name(field).ok()?.data_type();
            kind_of_type(data_type)
        }
    }
}

fn kind_of_type(data_type: &DataType) -> Option<FieldKind> {
    match data_type {
        t if t.is_numeric() => Some(FieldKind::Number),
        DataType::Date32 | DataType::Date64 | DataType::Timestamp(..) => Some(FieldKind::Number),
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View | DataType::Boolean => {
            Some(FieldKind::Category)
        }
        DataType::Dictionary(_, values) => match values.as_ref() {
            DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => Some(FieldKind::Category),
            _ => None,
        },
        _ => None,
    }
}

/// A field a slot may bind.
struct Candidate {
    name: String,
    kind: Option<FieldKind>,
    role: Option<Role>,
}

/// Binds `slots` to `table`'s fields, starting from the bindings in `given` (the spec's
/// `template.bind`); see the module docs for the order.
///
/// # Errors
///
/// [`SlotError::BadBinding`] for a given binding that names an unknown slot, a missing
/// field or a field of the wrong kind; [`SlotError::Unbound`] when a required slot finds no
/// field, so the dataset is not compatible with the template.
pub fn bind(slots: &[Slot], table: &Table, given: &Bindings) -> Result<Bindings, SlotError> {
    let mut bindings = Bindings::new();
    for (name, field) in given {
        let bad = |reason| SlotError::BadBinding {
            slot: name.clone(),
            field: field.clone(),
            reason,
        };
        let slot = slots
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| bad("the template has no such slot"))?;
        let kind = field_kind(table, field).ok_or_else(|| bad("the data has no such field"))?;
        if !slot.kind.accepts(kind) {
            return Err(bad("the field is of the wrong kind"));
        }
        bindings.insert(name.clone(), field.clone());
    }
    let fields: Vec<Candidate> = table
        .schema()
        .fields()
        .iter()
        .map(|f| f.name().clone())
        .filter(|name| !name.starts_with("__snr_"))
        .map(|name| {
            let kind = field_kind(table, &name);
            let role = table.field_meta(&name).ok().and_then(|meta| meta.role);
            Candidate { name, kind, role }
        })
        .collect();
    let free = |bindings: &Bindings, slot: &Slot, field: &Candidate| {
        !bindings.values().any(|v| *v == field.name)
            && field.kind.is_some_and(|k| slot.kind.accepts(k))
    };
    for slot in slots.iter().filter(|s| s.role.is_some()) {
        if bindings.contains_key(slot.name) {
            continue;
        }
        if let Some(field) = fields
            .iter()
            .find(|f| free(&bindings, slot, f) && f.role == slot.role)
        {
            bindings.insert(slot.name.to_owned(), field.name.clone());
        }
    }
    for slot in slots {
        if bindings.contains_key(slot.name) {
            continue;
        }
        let candidate = fields
            .iter()
            .filter(|f| free(&bindings, slot, f))
            .min_by_key(|f| f.role.is_some());
        match candidate {
            Some(field) => {
                bindings.insert(slot.name.to_owned(), field.name.clone());
            }
            None if slot.optional => {}
            None => {
                return Err(SlotError::Unbound {
                    slot: slot.name,
                    kind: slot.kind,
                });
            }
        }
    }
    Ok(bindings)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "test helpers")]

    use std::sync::Arc;

    use arrow_array::{Float64Array, RecordBatch, StringArray};
    use arrow_schema::{Field, Schema};
    use sonarithm_core::TableId;

    use super::*;
    use crate::meta::FieldMeta;

    fn table() -> Table {
        let schema = Arc::new(Schema::new(vec![
            Field::new("name", DataType::Utf8, false),
            Field::new("a", DataType::Float64, false),
            FieldMeta::new()
                .with_role(Role::Y)
                .apply(Field::new("b", DataType::Float64, false)),
        ]));
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["p", "q"])),
                Arc::new(Float64Array::from(vec![1.0, 2.0])),
                Arc::new(Float64Array::from(vec![3.0, 4.0])),
            ],
        )
        .unwrap_or_else(|e| panic!("{e}"));
        Table::from_batch(TableId::default(), "t", batch)
    }

    const SLOTS: [Slot; 3] = [
        Slot::new("category", FieldKind::Category, Some(Role::Category)),
        Slot::new("value", FieldKind::Number, Some(Role::Y)),
        Slot::new("other", FieldKind::Number, None).optional(),
    ];

    #[test]
    fn slots_bind_by_name_then_role_then_kind() {
        let auto = bind(&SLOTS, &table(), &Bindings::new()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(auto["value"], "b", "the role binds first");
        assert_eq!(auto["category"], "name");
        assert_eq!(auto["other"], "a");
        let given = Bindings::from([("value".to_owned(), "a".to_owned())]);
        let named = bind(&SLOTS, &table(), &given).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(named["value"], "a");
        assert_eq!(named["other"], "b");
    }

    #[test]
    fn bad_and_missing_bindings_are_errors() {
        let wrong = Bindings::from([("value".to_owned(), "name".to_owned())]);
        assert!(matches!(
            bind(&SLOTS, &table(), &wrong),
            Err(SlotError::BadBinding {
                reason: "the field is of the wrong kind",
                ..
            })
        ));
        let unknown = Bindings::from([("nope".to_owned(), "a".to_owned())]);
        assert!(matches!(
            bind(&SLOTS, &table(), &unknown),
            Err(SlotError::BadBinding { .. })
        ));
        let slots = [
            Slot::new("c", FieldKind::Category, None),
            Slot::new("d", FieldKind::Category, None),
        ];
        assert_eq!(
            bind(&slots, &table(), &Bindings::new()),
            Err(SlotError::Unbound {
                slot: "d",
                kind: FieldKind::Category
            })
        );
    }
}
