// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chart-type templates (C01-T09, RFC-0004 §2): the design a chart's type calls for, as
//! data-agnostic NUMISONA specs a spec names with its `template` key.
//!
//! An extracted chart is a long table (`series`, `x`, `y`, [`crate::model`]); the templates
//! bind it by role like any other data (SDR-03), so they also fit other datasets with
//! a compatible schema. [`template_for`] names the template for a chart type, which may be
//! one of NUMISONA's (`scatter-xy`, `box-five-number`, `heatmap-rowscan`):
//!
//! | Chart type | Template | Design |
//! |---|---|---|
//! | line, area | `chart-line` | x → time at 3 points/s, y → pitch, series → timbre (SDR-07, SDR-19, SDR-25, A11Y-13). With a category axis, time follows the table, so each series plays in turn |
//! | bar, grouped bar, histogram | `chart-bar` | one tone per bar at 2 bars/s in table order (each series in turn), pitch from zero, redundant duration, series → timbre (A11Y-13, SDR-25) |
//! | stacked bar | `chart-stacked-bar` | as `chart-bar`, each segment's own value; **experimental** |
//! | pie | `chart-pie` | one tone per slice at 2 slices/s, pitch from zero, redundant duration; **experimental** |
//! | scatter | `scatter-xy` (NUMISONA) | points by x at 4 points/s, y → pitch, series → timbre |
//! | box plot | `box-five-number` (NUMISONA) | five tones per group |
//! | heat map | `heatmap-rowscan` (NUMISONA) | cells row by row, value → pitch |
//!
//! No design rule covers part-to-whole charts or stacked segments, so `chart-pie` and
//! `chart-stacked-bar` are experimental: applying one reports `SNR-ENG-380`, and their
//! designs are TODO(verify) until a study supports them (design-rules change control).

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value, json};
use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};
use sonarithm_core::engine::{
    CompileContext, CompileError, SpecFragment, TemplateExpansion, TemplateProvider,
};
use sonarithm_data::slots::{self, Bindings, FieldKind, Slot, SlotError};
use sonarithm_data::{DataBundle, Role, Table};

use crate::ENGINE;
use crate::codes;
use crate::model::ChartType;

/// A chart-type template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartTemplate {
    /// The template ID, such as `chart-line`.
    pub id: &'static str,
    /// A one-line description, also spoken by the legend.
    pub description: &'static str,
    /// The slots, in binding order.
    pub slots: &'static [Slot],
    /// Whether no design rule covers the design yet (`SNR-ENG-380`).
    pub experimental: bool,
}

const SERIES: Slot = Slot::new("series", FieldKind::Category, Some(Role::Series)).optional();

const BARS: &[Slot] = &[
    Slot::new("category", FieldKind::Any, Some(Role::X)),
    Slot::new("value", FieldKind::Number, Some(Role::Y)),
    SERIES,
];

/// PLOTSONA's templates, in a fixed order.
pub const TEMPLATES: [ChartTemplate; 4] = [
    ChartTemplate {
        id: "chart-line",
        description: "A line chart: time along x, pitch for y, a timbre for each series.",
        slots: &[
            Slot::new("x", FieldKind::Any, Some(Role::X)),
            Slot::new("y", FieldKind::Number, Some(Role::Y)),
            SERIES,
        ],
        experimental: false,
    },
    ChartTemplate {
        id: "chart-bar",
        description: "A bar chart: one tone per bar, higher bars sound higher and longer, a timbre for each series.",
        slots: BARS,
        experimental: false,
    },
    ChartTemplate {
        id: "chart-stacked-bar",
        description: "A stacked bar chart: one tone per segment, larger segments sound higher and longer, a timbre for each series.",
        slots: BARS,
        experimental: true,
    },
    ChartTemplate {
        id: "chart-pie",
        description: "A pie chart: one tone per slice, larger slices sound higher and longer.",
        slots: &[
            Slot::new("slice", FieldKind::Any, Some(Role::X)),
            Slot::new("value", FieldKind::Number, Some(Role::Y)),
        ],
        experimental: true,
    },
];

/// The template with `id`, if PLOTSONA registers it.
#[must_use]
pub fn template(id: &str) -> Option<&'static ChartTemplate> {
    TEMPLATES.iter().find(|t| t.id == id)
}

/// The template a chart of `chart_type` calls for, PLOTSONA's or NUMISONA's. A chart
/// whose type the file does not state is played as a line; `Other` (radar, bubble, stock,
/// …) has no template yet.
#[must_use]
pub fn template_for(chart_type: Option<ChartType>) -> Option<&'static str> {
    Some(match chart_type {
        None | Some(ChartType::Line | ChartType::Area) => "chart-line",
        Some(ChartType::Bar | ChartType::Histogram) => "chart-bar",
        Some(ChartType::StackedBar) => "chart-stacked-bar",
        Some(ChartType::Pie) => "chart-pie",
        Some(ChartType::Scatter) => "scatter-xy",
        Some(ChartType::BoxPlot) => "box-five-number",
        Some(ChartType::Heatmap) => "heatmap-rowscan",
        Some(_) => return None,
    })
}

impl ChartTemplate {
    /// Binds the slots to `table`'s fields, starting from `given` (the spec's
    /// `template.bind`), with the shared rules of [`sonarithm_data::slots::bind`].
    ///
    /// # Errors
    ///
    /// The [`SlotError`] when a binding is invalid or a required slot finds no field.
    pub fn bind(&self, table: &Table, given: &Bindings) -> Result<Bindings, SlotError> {
        slots::bind(self.slots, table, given)
    }

    /// The spec sections the template designs for `bindings`: `substrate` and `tracks`.
    #[must_use]
    pub fn sections(&self, table: &Table, bindings: &Bindings) -> Value {
        let field = |slot: &str| bindings.get(slot).cloned().unwrap_or_default();
        let timbre = |channels: &mut Value| {
            if let Some(series) = bindings.get("series") {
                channels["timbre"] = json!({ "field": series });
            }
        };
        let (substrate, track) = if self.id == "chart-line" {
            let x = field("x");
            let numeric_x = slots::field_kind(table, &x) == Some(FieldKind::Number);
            // A numeric or time axis places each point at its x; a category axis
            // follows the table, one series after another.
            let time = if numeric_x {
                json!({ "field": x, "pace": { "pointsPerSecond": 3 } })
            } else {
                json!({ "pace": { "pointsPerSecond": 3 } })
            };
            let mut channels = json!({ "pitch": { "field": field("y") } });
            timbre(&mut channels);
            (
                json!({ "time": time }),
                json!({ "id": "line", "mark": "tone", "channels": channels }),
            )
        } else {
            let id = match self.id {
                "chart-pie" => "slices",
                "chart-stacked-bar" => "segments",
                _ => "bars",
            };
            let mut channels = json!({ "pitch": { "field": field("value"), "scale": { "domain": "zero-based" } } });
            timbre(&mut channels);
            (
                json!({ "time": { "pace": { "pointsPerSecond": 2 } } }),
                json!({ "id": id, "mark": "tone", "channels": channels, "redundant": ["duration"] }),
            )
        };
        let mut sections = Map::new();
        sections.insert("substrate".to_owned(), substrate);
        sections.insert("tracks".to_owned(), json!([track]));
        Value::Object(sections)
    }
}

/// The spec's `template` object.
#[derive(Deserialize)]
struct Request {
    id: String,
    #[serde(default)]
    bind: BTreeMap<String, String>,
}

/// PLOTSONA's chart-type templates as the compiler's template provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChartTemplates;

impl TemplateProvider<DataBundle> for ChartTemplates {
    fn expand(
        &self,
        request: &SpecFragment,
        data: &DataBundle,
        cx: &mut CompileContext,
    ) -> Result<TemplateExpansion, CompileError> {
        let parsed: Request = request.parse()?;
        let engine_error = |message: String| CompileError::Engine {
            engine: ENGINE,
            message,
        };
        let template = template(&parsed.id)
            .ok_or_else(|| engine_error(format!("PLOTSONA has no template `{}`", parsed.id)))?;
        let table = data.primary().ok_or_else(|| {
            engine_error("the data has no table to bind the template to".to_owned())
        })?;
        let bindings = template
            .bind(table, &parsed.bind)
            .map_err(|err| CompileError::Binding {
                template: template.id.to_owned(),
                error: err.into(),
            })?;
        if template.experimental {
            cx.report(
                Diagnostic::new(
                    &codes::EXPERIMENTAL_TEMPLATE,
                    format!(
                        "PLOTSONA: template `{}` is experimental; no design rule covers it yet.",
                        template.id
                    ),
                )
                .at(SpecPointer::root().key("template").key("id")),
            );
        }
        let sections = template.sections(table, &bindings);
        Ok(TemplateExpansion::new(
            bindings,
            SpecFragment::new(request.pointer().clone(), &sections)?,
        ))
    }
}
