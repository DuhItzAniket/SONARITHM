// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Templates: reusable, data-agnostic designs ("sonecules", SDR-03; Reinsch & Hermann
//! 2023).
//!
//! A template is a sonification design with named **slots** (an x field, a value field, a
//! category field, …). Binding the slots to a dataset's fields and applying the template
//! gives a spec v0.1 document, so any dataset with a compatible schema gets the same design.
//! Spec v0.1 reserves a `template` key (RFC-0001 Q12), so templates live here as spec
//! builders until a spec version adds it.
//!
//! | Template | Slots | Design |
//! |---|---|---|
//! | `line-overview` | x (number), y (number) | x → time, y → pitch, 3 points/s (SDR-19, A11Y-13) |
//! | `bar-discrete` | category, value (number) | one tone per bar in data order, 2 bars/s, pitch from zero, redundant duration (A11Y-13) |
//! | `scatter-xy` | x, y (numbers), series (category, optional) | points ordered by x at 4 points/s, y → pitch, series → timbre (SDR-25) |
//! | `histogram` | value (number) | 10 equal-width bins counted by the data layer; one tone per bin, count → pitch from zero, redundant duration |
//! | `box-five-number` | group (category), value (number) | per group, five tones: minimum, lower quartile, median, upper quartile, maximum |
//! | `heatmap-rowscan` | row, column, value (number) | cells scanned row by row, value → pitch, 4 cells/s |
//!
//! Every design uses the evidence-backed defaults (log-frequency pitch, 400–1400 Hz,
//! polarity from the concept); a template changes only what its chart type needs. Spec v0.1
//! aggregates have no quartiles, so `box-five-number` computes the five numbers itself and
//! writes them as inline values marked `derived` (03 §4); its notes point at those summary
//! rows, not at the original data.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use sonarithm_data::{DataBundle, FieldType, Role, Table};

use crate::encode::columns::{numbers_of, texts_of};

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

const fn slot(name: &'static str, kind: FieldKind, role: Option<Role>) -> Slot {
    Slot {
        name,
        kind,
        role,
        optional: false,
    }
}

/// A template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Template {
    /// The template ID, such as `line-overview`.
    pub id: &'static str,
    /// A one-line description, also spoken by the legend.
    pub description: &'static str,
    /// The slots, in binding order.
    pub slots: &'static [Slot],
}

/// Slot name → field name.
pub type Bindings = BTreeMap<String, String>;

/// A field a slot may bind.
struct Candidate {
    name: String,
    kind: Option<FieldKind>,
    role: Option<Role>,
}

/// Why a template cannot be applied.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateError {
    /// The dataset has no table.
    #[error("the dataset has no table")]
    NoTable,
    /// A required slot has no compatible field.
    #[error("template `{template}` needs a {kind} field for slot `{slot}`, and none is left")]
    Unbound {
        /// The template.
        template: &'static str,
        /// The slot.
        slot: &'static str,
        /// What it needs.
        kind: &'static str,
    },
    /// A binding names a field the data does not have, or of the wrong kind.
    #[error("slot `{slot}` of template `{template}` cannot take field `{field}`: {reason}")]
    BadBinding {
        /// The template.
        template: &'static str,
        /// The slot.
        slot: String,
        /// The field.
        field: String,
        /// Why.
        reason: &'static str,
    },
    /// No template has this ID.
    #[error("no template is called `{0}`")]
    Unknown(String),
}

/// Every template, in a fixed order.
pub const TEMPLATES: [Template; 6] = [
    Template {
        id: "line-overview",
        description: "A line chart overview: time along x, pitch for y.",
        slots: &[
            slot("x", FieldKind::Number, Some(Role::X)),
            slot("y", FieldKind::Number, Some(Role::Y)),
        ],
    },
    Template {
        id: "bar-discrete",
        description: "A bar chart: one tone per bar, higher bars sound higher and longer.",
        slots: &[
            slot("category", FieldKind::Category, Some(Role::Category)),
            slot("value", FieldKind::Number, Some(Role::Y)),
        ],
    },
    Template {
        id: "scatter-xy",
        description: "A scatter plot: points from left to right, pitch for y, timbre for the series.",
        slots: &[
            slot("x", FieldKind::Number, Some(Role::X)),
            slot("y", FieldKind::Number, Some(Role::Y)),
            Slot {
                name: "series",
                kind: FieldKind::Category,
                role: Some(Role::Series),
                optional: true,
            },
        ],
    },
    Template {
        id: "histogram",
        description: "A histogram: one tone per bin, higher counts sound higher and longer.",
        slots: &[slot("value", FieldKind::Number, Some(Role::Y))],
    },
    Template {
        id: "box-five-number",
        description: "Box plots: for each group, the minimum, quartiles, median and maximum, low to high.",
        slots: &[
            slot("group", FieldKind::Category, Some(Role::Category)),
            slot("value", FieldKind::Number, Some(Role::Y)),
        ],
    },
    Template {
        id: "heatmap-rowscan",
        description: "A heat map scanned row by row: pitch for the value of each cell.",
        slots: &[
            slot("row", FieldKind::Any, None),
            slot("column", FieldKind::Any, Some(Role::X)),
            slot("value", FieldKind::Number, None),
        ],
    },
];

/// The template with `id`.
///
/// # Errors
///
/// [`TemplateError::Unknown`].
pub fn template(id: &str) -> Result<&'static Template, TemplateError> {
    TEMPLATES
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| TemplateError::Unknown(id.to_owned()))
}

/// The kind of a table's field: its declared type, else its Arrow type.
fn field_kind(table: &Table, field: &str) -> Option<FieldKind> {
    let meta = table.field_meta(field).ok()?;
    match meta.ftype {
        Some(FieldType::Quantitative | FieldType::Temporal) => Some(FieldKind::Number),
        Some(FieldType::Nominal | FieldType::Ordinal) => Some(FieldKind::Category),
        _ => {
            let column = table.column(field).ok()?;
            if numbers_of(&column).is_some() {
                Some(FieldKind::Number)
            } else {
                texts_of(&column).map(|_| FieldKind::Category)
            }
        }
    }
}

fn accepts(slot: FieldKind, field: FieldKind) -> bool {
    slot == FieldKind::Any || slot == field
}

fn kind_name(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::Number => "numeric",
        FieldKind::Category => "categorical",
        FieldKind::Any => "",
    }
}

impl Template {
    /// Binds the slots to `table`'s fields. First every slot with a role takes an unused
    /// field with that role (03 §3.4); then each remaining slot, in order, takes the first
    /// unused field of its kind, preferring fields without a role, in schema order.
    /// Reserved `__snr_` columns are never bound.
    ///
    /// # Errors
    ///
    /// [`TemplateError::Unbound`] when a required slot finds no field; the dataset is then
    /// not compatible with the template.
    pub fn bind(&self, table: &Table) -> Result<Bindings, TemplateError> {
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
        let mut bindings = Bindings::new();
        let free = |bindings: &Bindings, slot: &Slot, field: &Candidate| {
            !bindings.values().any(|v| *v == field.name)
                && field.kind.is_some_and(|k| accepts(slot.kind, k))
        };
        for slot in self.slots.iter().filter(|s| s.role.is_some()) {
            if let Some(field) = fields
                .iter()
                .find(|f| free(&bindings, slot, f) && f.role == slot.role)
            {
                bindings.insert(slot.name.to_owned(), field.name.clone());
            }
        }
        for slot in self.slots {
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
                    return Err(TemplateError::Unbound {
                        template: self.id,
                        slot: slot.name,
                        kind: kind_name(slot.kind),
                    });
                }
            }
        }
        Ok(bindings)
    }

    /// Checks explicit bindings against `table`.
    ///
    /// # Errors
    ///
    /// [`TemplateError::BadBinding`] for an unknown slot, a missing field or a field of the
    /// wrong kind; [`TemplateError::Unbound`] for a required slot left out.
    pub fn check(&self, table: &Table, bindings: &Bindings) -> Result<(), TemplateError> {
        for (slot_name, field) in bindings {
            let bad = |reason| TemplateError::BadBinding {
                template: self.id,
                slot: slot_name.clone(),
                field: field.clone(),
                reason,
            };
            let slot = self
                .slots
                .iter()
                .find(|s| s.name == slot_name)
                .ok_or_else(|| bad("the template has no such slot"))?;
            let kind = field_kind(table, field).ok_or_else(|| bad("the data has no such field"))?;
            if !accepts(slot.kind, kind) {
                return Err(bad("the field is of the wrong kind"));
            }
        }
        for slot in self.slots {
            if !slot.optional && !bindings.contains_key(slot.name) {
                return Err(TemplateError::Unbound {
                    template: self.id,
                    slot: slot.name,
                    kind: kind_name(slot.kind),
                });
            }
        }
        Ok(())
    }

    /// The spec for `bindings`, reading the data from `source` (a spec `data.source`
    /// object). `bundle` supplies the data only where spec v0.1 cannot compute a summary
    /// itself (`box-five-number`).
    ///
    /// # Errors
    ///
    /// As [`Template::check`], and [`TemplateError::NoTable`].
    pub fn apply(
        &self,
        bundle: &DataBundle,
        bindings: &Bindings,
        source: &Value,
    ) -> Result<Value, TemplateError> {
        let table = bundle.primary().ok_or(TemplateError::NoTable)?;
        self.check(table, bindings)?;
        let field = |slot: &str| bindings.get(slot).cloned().unwrap_or_default();
        let mut spec = Map::new();
        spec.insert("specVersion".to_owned(), json!("0.1"));
        spec.insert("name".to_owned(), json!(self.id));
        spec.insert("description".to_owned(), json!(self.description));
        let (data, transform, substrate, track) = match self.id {
            "line-overview" => (
                json!({ "source": source }),
                json!([]),
                json!({ "time": { "field": field("x"), "pace": { "pointsPerSecond": 3 } } }),
                json!({ "id": "line", "mark": "tone", "channels": { "pitch": { "field": field("y") } } }),
            ),
            "bar-discrete" => (
                json!({ "source": source }),
                json!([]),
                json!({ "time": { "pace": { "pointsPerSecond": 2 } } }),
                json!({
                    "id": "bars", "mark": "tone",
                    "channels": { "pitch": { "field": field("value"), "scale": { "domain": "zero-based" } } },
                    "redundant": ["duration"],
                }),
            ),
            "scatter-xy" => {
                let mut channels = json!({ "pitch": { "field": field("y") } });
                if let Some(series) = bindings.get("series") {
                    channels["timbre"] = json!({ "field": series });
                }
                (
                    json!({ "source": source }),
                    json!([]),
                    json!({ "time": { "field": field("x"), "pace": { "pointsPerSecond": 4 } } }),
                    json!({ "id": "points", "mark": "tone", "channels": channels }),
                )
            }
            "histogram" => {
                let value = field("value");
                (
                    json!({ "source": source }),
                    json!([
                        { "op": "bin", "field": value, "method": "equal-width", "count": HISTOGRAM_BINS, "as": "bin" },
                        { "op": "aggregate", "groupby": ["bin"], "ops": [{ "op": "count", "field": value, "as": "count" }] },
                        { "op": "sort", "field": "bin" },
                    ]),
                    json!({ "time": { "field": "bin", "pace": { "pointsPerSecond": 2 } } }),
                    json!({
                        "id": "bins", "mark": "tone",
                        "channels": { "pitch": { "field": "count", "scale": { "domain": "zero-based" } } },
                        "redundant": ["duration"],
                    }),
                )
            }
            "box-five-number" => (
                json!({ "source": { "kind": "inline", "values": five_numbers(table, &field("group"), &field("value")) } }),
                json!([]),
                json!({ "time": { "pace": { "pointsPerSecond": 3 } } }),
                json!({ "id": "boxes", "mark": "tone", "channels": { "pitch": { "field": "value" } } }),
            ),
            _ => (
                json!({ "source": source }),
                json!([
                    { "op": "sort", "field": field("column") },
                    { "op": "sort", "field": field("row") },
                ]),
                json!({ "time": { "pace": { "pointsPerSecond": 4 } } }),
                json!({ "id": "cells", "mark": "tone", "channels": { "pitch": { "field": field("value") } } }),
            ),
        };
        spec.insert("data".to_owned(), data);
        if transform.as_array().is_some_and(|t| !t.is_empty()) {
            spec.insert("transform".to_owned(), transform);
        }
        spec.insert("substrate".to_owned(), substrate);
        spec.insert("tracks".to_owned(), json!([track]));
        Ok(Value::Object(spec))
    }
}

/// The number of equal-width bins of `histogram`.
pub const HISTOGRAM_BINS: u32 = 10;

/// The five-number summary of each group, in order of first appearance, as inline records
/// marked `derived` (03 §4). Quartiles interpolate linearly between order statistics
/// (Hyndman & Fan type 7, the default of R and `NumPy`).
fn five_numbers(table: &Table, group: &str, value: &str) -> Vec<Value> {
    let groups = table.column(group).ok().and_then(|c| texts_of(&c));
    let values = table.column(value).ok().and_then(|c| numbers_of(&c));
    let (Some(groups), Some(values)) = (groups, values) else {
        return Vec::new();
    };
    let mut order: Vec<String> = Vec::new();
    let mut members: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (g, v) in groups.into_iter().zip(values) {
        let (Some(g), false) = (g, v.is_nan()) else {
            continue;
        };
        if !members.contains_key(&g) {
            order.push(g.clone());
        }
        members.entry(g).or_default().push(v);
    }
    let mut rows = Vec::new();
    for g in order {
        let mut sorted = members.remove(&g).unwrap_or_default();
        sorted.sort_by(f64::total_cmp);
        for (rank, (stat, p)) in [
            ("minimum", 0.0),
            ("lower quartile", 0.25),
            ("median", 0.5),
            ("upper quartile", 0.75),
            ("maximum", 1.0),
        ]
        .into_iter()
        .enumerate()
        {
            rows.push(json!({
                "group": g,
                "statistic": stat,
                "rank": rank,
                "value": quantile(&sorted, p),
                "__snr_prov": "derived",
            }));
        }
    }
    rows
}

/// The `p` quantile of sorted values, type 7.
fn quantile(sorted: &[f64], p: f64) -> f64 {
    let Some(&first) = sorted.first() else {
        return f64::NAN;
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "group sizes stay far below 2^52"
    )]
    let h = (sorted.len() - 1) as f64 * p;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "h is in [0, n − 1]"
    )]
    let below = h.floor() as usize;
    let above = (below + 1).min(sorted.len() - 1);
    let (lo, hi) = (sorted.get(below).copied().unwrap_or(first), sorted[above]);
    lo + (h - h.floor()) * (hi - lo)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, reason = "quartiles of small integers are exact")]

    use super::*;

    #[test]
    fn quartiles_interpolate_like_r_and_numpy() {
        let values = [1.0, 2.0, 3.0, 4.0, 10.0];
        assert_eq!(quantile(&values, 0.0), 1.0);
        assert_eq!(quantile(&values, 0.25), 2.0);
        assert_eq!(quantile(&values, 0.5), 3.0);
        assert_eq!(quantile(&values, 0.75), 4.0);
        assert_eq!(quantile(&values, 1.0), 10.0);
        assert_eq!(quantile(&[1.0, 2.0], 0.25), 1.25);
        assert!(quantile(&[], 0.5).is_nan());
    }

    #[test]
    fn templates_are_listed_and_found() {
        let ids: Vec<&str> = TEMPLATES.iter().map(|t| t.id).collect();
        assert_eq!(
            ids,
            [
                "line-overview",
                "bar-discrete",
                "scatter-xy",
                "histogram",
                "box-five-number",
                "heatmap-rowscan"
            ]
        );
        assert!(template("histogram").is_ok());
        assert_eq!(
            template("pie"),
            Err(TemplateError::Unknown("pie".to_owned()))
        );
    }
}
