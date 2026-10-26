// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The MAIDR importer (C01-T10), written from MAIDR's published data schema
//! (`docs/SCHEMA.md` in the `xability/maidr` repository, read 2026-10-26).
//!
//! **Format version: TODO(verify).** The schema page states no version; at the time it was
//! read the maidr package was the TypeScript rewrite (4.x; `maidr.js` 3.75.1 on cdnjs), and
//! the shape of stacked bars was not documented. A document with one layer's keys (`type`,
//! `axes`, `data`) at the top level, as maidr-legacy wrote a single plot, is read as that
//! layer; whether legacy data shapes match the current ones is TODO(verify).
//!
//! A MAIDR document's data is the plotted data itself, so it becomes an `inline` source:
//!
//! | MAIDR layer `type` | Rows | Design |
//! |---|---|---|
//! | `bar` | `x`, `y` | template `chart-bar` |
//! | `dodged_bar` | `x`, `fill` (series), `y` | `chart-bar` |
//! | `stacked_bar`, `stacked_normalized_bar` | `x`, `fill`, `y` | `chart-stacked-bar` (shape assumed as `dodged_bar`) |
//! | `line`, `step`, `smooth` | `series`, `x`, `y`, one series per inner list | `chart-line` (steps and smooth curves play their points: approximated) |
//! | `point` | `x` (or `xLabel`), `y` | `scatter-xy`, or `chart-line` on a category axis |
//! | `hist` | the bin centre `x`, its count `y` | `chart-bar` (bins play at their centres: approximated) |
//! | `pie` | `x` (slice), `y` | `chart-pie` |
//! | `heat` | `row`, `column`, `value` from `points` | `heatmap-rowscan` |
//! | `box` | five rows per box (`group`, `statistic`, `rank`, `value`) | tracks written out: five tones per box, as `box-five-number` (outliers reported) |
//! | `candlestick` | `date`, `close` | `chart-line` (open, high, low and volume reported) |
//!
//! The document's `title` becomes `name` and the axis labels the fields' labels. A figure
//! imports its first subplot's first layer; the others are reported. Selectors, hover
//! modes, `svg_x`/`svg_y` drawing positions and the layout keys only concern the picture.

use serde_json::{Map, Value, json};
use sonarithm_core::Diagnostic;

use super::{Draft, Import, ImportReport, text_of, unreadable};
use crate::budget::{Budgets, Work};
use crate::codes;
use crate::detect;

const IMPORTER: &str = "MAIDR";

/// Imports a MAIDR document.
///
/// # Errors
///
/// `SNR-ENG-301` for input over the size budget, `SNR-ENG-307` or `SNR-ENG-304` for JSON
/// that is invalid or too deep, `SNR-ENG-383` for a document that is not MAIDR or has a
/// layer type with no design, `SNR-ENG-384` for a layer without data, and `SNR-ENG-306`
/// for more rows than the point budget.
pub fn import(text: &[u8], budgets: &Budgets) -> Result<Import, Box<Diagnostic>> {
    budgets.check_input(crate::budget::len64(text.len()))?;
    let mut work = Work::new(budgets.max_work);
    let value = detect::parse_json(text, budgets, &mut work)?;
    import_value(&value, budgets)
}

/// [`import`] for an already parsed document.
///
/// # Errors
///
/// As [`import`].
pub fn import_value(doc: &Value, budgets: &Budgets) -> Result<Import, Box<Diagnostic>> {
    let root = doc
        .as_object()
        .ok_or_else(|| unreadable(IMPORTER, "the document is not a JSON object"))?;
    let mut report = ImportReport::default();
    let (layer, at) = first_layer(root, &mut report)?;
    let kind = layer
        .get("type")
        .or_else(|| root.get("type"))
        .and_then(Value::as_str)
        .ok_or_else(|| unreadable(IMPORTER, "the layer has no `type`"))?;
    let data = layer
        .get("data")
        .ok_or_else(|| no_data(&format!("the `{kind}` layer has no `data`")))?;
    let axes = layer.get("axes").or_else(|| root.get("axes"));
    let label = |axis: &str| {
        axes.and_then(|a| a.get(axis)).and_then(|a| {
            a.as_str()
                .map(str::to_owned)
                .or_else(|| text_of(a.get("label")))
        })
    };
    let mut draft = Draft {
        name: text_of(root.get("title")).or_else(|| text_of(layer.get("title"))),
        description: text_of(root.get("subtitle")).or_else(|| text_of(root.get("caption"))),
        ..Draft::default()
    };
    let rows = rows(
        kind,
        data,
        &at,
        &mut draft,
        &mut report,
        (label("x"), label("y")),
    )?;
    if crate::budget::len64(rows.len()) > budgets.max_points {
        return Err(Box::new(Diagnostic::new(
            &codes::CONTENT_BUDGET,
            format!(
                "PLOTSONA {IMPORTER} import: {} rows is more than the budget of {}.",
                rows.len(),
                budgets.max_points
            ),
        )));
    }
    if rows.is_empty() {
        return Err(no_data(&format!("the `{kind}` layer's data has no values")));
    }
    draft.source = json!({ "kind": "inline", "values": rows });
    Ok(Import {
        spec: draft.finish(),
        report,
    })
}

fn no_data(why: &str) -> Box<Diagnostic> {
    Box::new(Diagnostic::new(
        &codes::IMPORT_NO_DATA,
        format!("PLOTSONA {IMPORTER} import: {why}."),
    ))
}

/// The first subplot's first layer and its pointer; the legacy format is its own layer.
fn first_layer<'a>(
    root: &'a Map<String, Value>,
    report: &mut ImportReport,
) -> Result<(&'a Map<String, Value>, String), Box<Diagnostic>> {
    let Some(rows) = root.get("subplots").and_then(Value::as_array) else {
        if root.contains_key("type") && root.contains_key("data") {
            return Ok((root, String::new()));
        }
        return Err(unreadable(
            IMPORTER,
            "the document has neither `subplots` nor a top-level `type` and `data`",
        ));
    };
    let mut first = None;
    for (r, row) in rows.iter().enumerate() {
        for (c, subplot) in row.as_array().into_iter().flatten().enumerate() {
            for (l, layer) in subplot
                .get("layers")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                let at = format!("/subplots/{r}/{c}/layers/{l}");
                match (first.is_none(), layer.as_object()) {
                    (true, Some(layer)) => first = Some((layer, at)),
                    _ => report.unmapped(
                        &at,
                        "layer",
                        "only the first layer is imported; import the others one at a time",
                    ),
                }
            }
        }
    }
    first.ok_or_else(|| unreadable(IMPORTER, "no subplot has a layer"))
}

fn number(v: Option<&Value>) -> Option<f64> {
    v.and_then(Value::as_f64)
}

/// The rows for one layer, setting the draft's template and fields.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per MAIDR layer type keeps each mapping next to its documentation row"
)]
fn rows(
    kind: &str,
    data: &Value,
    at: &str,
    draft: &mut Draft,
    report: &mut ImportReport,
    (x_label, y_label): (Option<String>, Option<String>),
) -> Result<Vec<Value>, Box<Diagnostic>> {
    let list = || data.as_array().cloned().unwrap_or_default();
    let x_type = |rows: &[Value]| {
        if rows.iter().all(|r| r["x"].is_number()) {
            "quantitative"
        } else {
            "nominal"
        }
    };
    let mut out = Vec::new();
    match kind {
        "bar" | "pie" => {
            for p in list() {
                out.push(json!({ "x": p.get("x").cloned().unwrap_or(Value::Null), "y": number(p.get("y")) }));
            }
            let (id, slot) = if kind == "pie" {
                ("chart-pie", "slice")
            } else {
                ("chart-bar", "category")
            };
            draft.field("x", "nominal", x_label.as_deref());
            draft.field("y", "quantitative", y_label.as_deref());
            draft.template(id, &[(slot, "x"), ("value", "y")]);
        }
        "dodged_bar" | "stacked_bar" | "stacked_normalized_bar" => {
            for series in list() {
                for p in series.as_array().cloned().unwrap_or_default() {
                    out.push(json!({
                        "x": p.get("x").cloned().unwrap_or(Value::Null),
                        "series": p.get("fill").cloned().unwrap_or(Value::Null),
                        "y": number(p.get("y")),
                    }));
                }
            }
            if kind == "stacked_normalized_bar" {
                report.approximated(
                    &format!("{at}/type"),
                    "type",
                    "the segments play their shares as given",
                );
            }
            let id = if kind == "dodged_bar" {
                "chart-bar"
            } else {
                "chart-stacked-bar"
            };
            draft.field("x", "nominal", x_label.as_deref());
            draft.field("y", "quantitative", y_label.as_deref());
            draft.template(
                id,
                &[("category", "x"), ("value", "y"), ("series", "series")],
            );
        }
        "line" | "step" | "smooth" => {
            for (i, series) in list().into_iter().enumerate() {
                for p in series.as_array().cloned().unwrap_or_default() {
                    out.push(json!({
                        "series": format!("Line {}", i + 1),
                        "x": p.get("x").cloned().unwrap_or(Value::Null),
                        "y": number(p.get("y")),
                    }));
                }
            }
            if kind != "line" {
                report.approximated(
                    &format!("{at}/type"),
                    "type",
                    format!("a `{kind}` layer plays its points; the curve between them is not drawn in sound"),
                );
            }
            draft.field("x", x_type(&out), x_label.as_deref());
            draft.field("y", "quantitative", y_label.as_deref());
            draft.template(
                "chart-line",
                &[("x", "x"), ("y", "y"), ("series", "series")],
            );
        }
        "point" => {
            let mut labelled = false;
            for p in list() {
                let x = match p.get("xLabel") {
                    Some(label) => {
                        labelled = true;
                        label.clone()
                    }
                    None => p.get("x").cloned().unwrap_or(Value::Null),
                };
                out.push(json!({ "x": x, "y": number(p.get("y")) }));
            }
            let numeric = !labelled && x_type(&out) == "quantitative";
            draft.field(
                "x",
                if numeric { "quantitative" } else { "nominal" },
                x_label.as_deref(),
            );
            draft.field("y", "quantitative", y_label.as_deref());
            let id = if numeric { "scatter-xy" } else { "chart-line" };
            draft.template(id, &[("x", "x"), ("y", "y")]);
        }
        "hist" => {
            for p in list() {
                out.push(json!({ "bin": number(p.get("x")), "count": number(p.get("y")) }));
            }
            report.approximated(
                &format!("{at}/data"),
                "data.xMin",
                "each bin plays at its centre; its edges are not heard",
            );
            draft.field("bin", "quantitative", x_label.as_deref());
            draft.field("count", "quantitative", y_label.as_deref());
            draft.template("chart-bar", &[("category", "bin"), ("value", "count")]);
        }
        "heat" => {
            let points = data
                .get("points")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let columns = data
                .get("x")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let row_names = data
                .get("y")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for (i, row) in points.iter().enumerate() {
                for (j, value) in row.as_array().into_iter().flatten().enumerate() {
                    out.push(json!({
                        "row": row_names.get(i).cloned().unwrap_or_else(|| json!(i)),
                        "column": columns.get(j).cloned().unwrap_or_else(|| json!(j)),
                        "value": value.as_f64(),
                    }));
                }
            }
            draft.field("column", "nominal", x_label.as_deref());
            draft.field("row", "nominal", y_label.as_deref());
            draft.field("value", "quantitative", None);
            draft.template(
                "heatmap-rowscan",
                &[("row", "row"), ("column", "column"), ("value", "value")],
            );
        }
        "box" => {
            for (i, b) in list().into_iter().enumerate() {
                let group = b
                    .get("fill")
                    .cloned()
                    .unwrap_or_else(|| json!(format!("Box {}", i + 1)));
                for (rank, (key, statistic)) in [
                    ("min", "minimum"),
                    ("q1", "lower quartile"),
                    ("q2", "median"),
                    ("q3", "upper quartile"),
                    ("max", "maximum"),
                ]
                .into_iter()
                .enumerate()
                {
                    out.push(json!({ "group": group, "statistic": statistic, "rank": rank, "value": number(b.get(key)) }));
                }
                for key in ["lowerOutliers", "upperOutliers"] {
                    if b.get(key)
                        .and_then(Value::as_array)
                        .is_some_and(|o| !o.is_empty())
                    {
                        report.unmapped(
                            &format!("{at}/data/{i}/{key}"),
                            key,
                            "outliers are not played; the box plays its five numbers",
                        );
                    }
                }
            }
            draft.field("group", "nominal", x_label.as_deref());
            draft.field("value", "quantitative", y_label.as_deref());
            draft.sections.insert(
                "substrate".to_owned(),
                json!({ "time": { "pace": { "pointsPerSecond": 3 } } }),
            );
            draft.sections.insert(
                "tracks".to_owned(),
                json!([{ "id": "boxes", "mark": "tone", "channels": { "pitch": { "field": "value" } } }]),
            );
        }
        "candlestick" => {
            for p in list() {
                out.push(json!({ "date": p.get("value").cloned().unwrap_or(Value::Null), "close": number(p.get("close")) }));
            }
            report.unmapped(
                &format!("{at}/data"),
                "data.open",
                "a candlestick plays its closing price; open, high, low and volume are not played",
            );
            draft.field("date", "ordinal", x_label.as_deref());
            draft.field("close", "quantitative", y_label.as_deref());
            draft.template("chart-line", &[("x", "date"), ("y", "close")]);
        }
        other => {
            report.unmapped(
                &format!("{at}/type"),
                "type",
                format!("the `{other}` layer type has no design"),
            );
            return Err(unreadable(
                IMPORTER,
                format!("the `{other}` layer type has no design yet"),
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    fn run(doc: &Value) -> Import {
        import_value(doc, &Budgets::DEFAULT).unwrap()
    }

    #[test]
    fn the_readme_bar_chart_imports() {
        let out = run(&json!({
            "id": "my-chart",
            "title": "Sales by Quarter",
            "subplots": [[{ "layers": [{
                "id": "0", "type": "bar",
                "axes": { "x": "Quarter", "y": "Revenue" },
                "data": [{ "x": "Q1", "y": 120 }, { "x": "Q2", "y": 200 }]
            }] }]]
        }));
        assert_eq!(out.spec["name"], "Sales by Quarter");
        assert_eq!(
            out.spec["template"],
            json!({ "id": "chart-bar", "bind": { "category": "x", "value": "y" } })
        );
        assert_eq!(out.spec["data"]["fields"]["y"]["label"], "Revenue");
        assert!(out.report.is_empty());
    }

    #[test]
    fn boxes_play_five_numbers_and_report_outliers() {
        let out = run(&json!({
            "id": "b", "subplots": [[{ "layers": [{ "id": "0", "type": "box", "data": [
                { "lowerOutliers": [40.0], "min": 71.35, "q1": 92.62, "q2": 99.64, "q3": 107.66, "max": 118.19, "upperOutliers": [], "fill": "Group 1" }
            ] }] }]]
        }));
        assert_eq!(
            out.spec["data"]["source"]["values"]
                .as_array()
                .unwrap()
                .len(),
            5
        );
        assert_eq!(out.report.entries().len(), 1);
        assert_eq!(
            out.report.entries()[0].pointer,
            "/subplots/0/0/layers/0/data/0/lowerOutliers"
        );
        assert!(out.spec.get("template").is_none());
    }

    #[test]
    fn heat_maps_unroll_and_other_layers_are_reported() {
        let out = run(&json!({
            "id": "h", "subplots": [[{ "layers": [
                { "id": "0", "type": "heat", "data": { "points": [[1, 2], [3, 4]], "x": ["a", "b"], "y": ["r", "s"] } },
                { "id": "1", "type": "line", "data": [[{ "x": 1, "y": 2 }]] }
            ] }]]
        }));
        let values = out.spec["data"]["source"]["values"].as_array().unwrap();
        assert_eq!(
            values[3],
            json!({ "row": "s", "column": "b", "value": 4.0 })
        );
        assert_eq!(out.report.entries()[0].pointer, "/subplots/0/0/layers/1");
    }

    #[test]
    fn unknown_types_and_empty_data_are_errors() {
        let unknown = import_value(
            &json!({ "id": "x", "subplots": [[{ "layers": [{ "type": "sankey", "data": [] }] }]] }),
            &Budgets::DEFAULT,
        )
        .unwrap_err();
        assert_eq!(unknown.code.as_str(), "SNR-ENG-383");
        let empty =
            import_value(&json!({ "type": "bar", "data": [] }), &Budgets::DEFAULT).unwrap_err();
        assert_eq!(empty.code.as_str(), "SNR-ENG-384");
    }
}
