// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vega specifications (<https://vega.github.io/vega/docs/>).
//!
//! Vega names its data sets and draws marks from them. A mark's `from.data` names a data
//! set, or a facet that a group mark made with `from.facet` (`data`, `name`, `groupby`),
//! whose groups become series. The `x` and `y` fields of the mark's `encode` (its `enter`
//! and `update` sets) give the points; a scale of type `band` or `point` marks the category
//! axis, `time` or `utc` a date axis. An arc reads the field of its data's `pie`
//! transform. A data set derived with `source` reads its source's values; transforms that
//! need expressions (`filter`, `formula`, …) are not applied and are reported.

use std::collections::BTreeMap;

use serde_json::{Map, Value};
use sonarithm_core::Diagnostic;

use super::{Cx, XKind, field, label, number, title};
use crate::codes;
use crate::model::{ChartData, ChartType, Point, Series};

/// Transforms that only lay marks out and leave the values alone.
const LAYOUT_TRANSFORMS: [&str; 5] = ["pie", "stack", "collect", "identifier", "extent"];

struct Spec<'a> {
    data: BTreeMap<&'a str, &'a Value>,
    scales: BTreeMap<&'a str, &'a str>,
}

/// A facet: a group mark's split of a data set.
#[derive(Clone)]
struct Facet {
    data: String,
    groupby: Vec<String>,
}

pub(super) fn extract(spec: &Value, cx: &mut Cx<'_>) -> Result<ChartData, Box<Diagnostic>> {
    let mut s = Spec {
        data: BTreeMap::new(),
        scales: BTreeMap::new(),
    };
    for d in spec
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(name) = d.get("name").and_then(Value::as_str) {
            s.data.insert(name, d);
        }
    }
    collect_scales(spec, &mut s, cx, 0)?;
    let mut chart = ChartData {
        title: title(spec.get("title")),
        ..ChartData::default()
    };
    let mut facets = BTreeMap::new();
    walk(spec, &s, &mut facets, &mut chart, cx, 0)?;
    // Axis titles, when the specification gives them, win over field names.
    for axis in spec
        .get("axes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(text) = title(axis.get("title")) else {
            continue;
        };
        match axis.get("orient").and_then(Value::as_str) {
            Some("bottom" | "top") => chart.x_title = Some(text),
            Some("left" | "right") => chart.y_title = Some(text),
            _ => {}
        }
    }
    Ok(chart)
}

fn collect_scales<'a>(
    node: &'a Value,
    s: &mut Spec<'a>,
    cx: &mut Cx<'_>,
    depth: usize,
) -> Result<(), Box<Diagnostic>> {
    cx.budgets.check_depth(depth)?;
    for scale in node
        .get("scales")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let (Some(name), kind) = (
            scale.get("name").and_then(Value::as_str),
            scale
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("linear"),
        ) {
            s.scales.insert(name, kind);
        }
    }
    for mark in node
        .get("marks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        collect_scales(mark, s, cx, depth + 1)?;
    }
    Ok(())
}

fn walk(
    node: &Value,
    s: &Spec<'_>,
    facets: &mut BTreeMap<String, Facet>,
    chart: &mut ChartData,
    cx: &mut Cx<'_>,
    depth: usize,
) -> Result<(), Box<Diagnostic>> {
    cx.budgets.check_depth(depth)?;
    for mark in node
        .get("marks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        cx.work.spend(1)?;
        let kind = mark.get("type").and_then(Value::as_str).unwrap_or_default();
        let from = mark.get("from");
        if kind == "group" {
            if let Some(facet) = from.and_then(|f| f.get("facet")) {
                let name = facet.get("name").and_then(Value::as_str);
                let data = facet.get("data").and_then(Value::as_str);
                if let (Some(name), Some(data)) = (name, data) {
                    let groupby = match facet.get("groupby") {
                        Some(Value::String(g)) => vec![g.clone()],
                        Some(Value::Array(g)) => g
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect(),
                        _ => Vec::new(),
                    };
                    facets.insert(
                        name.to_owned(),
                        Facet {
                            data: data.to_owned(),
                            groupby,
                        },
                    );
                }
            }
            walk(mark, s, facets, chart, cx, depth + 1)?;
            continue;
        }
        let Some(source) = from.and_then(|f| f.get("data")).and_then(Value::as_str) else {
            continue;
        };
        if !matches!(kind, "line" | "area" | "rect" | "symbol" | "arc") {
            continue;
        }
        let (data_name, groupby) = match facets.get(source) {
            Some(f) => (f.data.clone(), f.groupby.clone()),
            None => (source.to_owned(), Vec::new()),
        };
        let Some(rows) = rows(&data_name, s, cx, 0) else {
            continue;
        };
        read_mark(mark, kind, &data_name, &rows, &groupby, s, chart, cx)?;
    }
    Ok(())
}

/// A data set's rows, following `source`; transforms that change values are reported.
fn rows(name: &str, s: &Spec<'_>, cx: &mut Cx<'_>, depth: usize) -> Option<Vec<Value>> {
    if depth > 32 {
        return None;
    }
    let data = s.data.get(name)?;
    for t in data
        .get("transform")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let kind = t.get("type").and_then(Value::as_str).unwrap_or_default();
        if !LAYOUT_TRANSFORMS.contains(&kind) {
            cx.warn(
                &codes::TRANSFORM_NOT_APPLIED,
                format!("The `{kind}` transform of data `{name}` was not applied; values are the stored data before it."),
            );
        }
    }
    if let Some(url) = data.get("url") {
        cx.warn(
            &codes::REMOTE_DATA,
            format!("The data at {url} was not fetched; PLOTSONA reads only data inside the file."),
        );
        return None;
    }
    match data.get("values") {
        Some(Value::Array(items)) => Some(items.clone()),
        Some(Value::Object(o)) if o.contains_key("signal") => {
            cx.warn(
                &codes::SCRIPT_DATA,
                format!("The data `{name}` comes from a signal, which needs Vega's runtime."),
            );
            None
        }
        Some(_) => None,
        None => match data.get("source") {
            Some(Value::String(source)) => rows(source, s, cx, depth + 1),
            _ => None,
        },
    }
}

/// The field an encoding channel reads: `{"field": "x"}` (or `{"scale": …, "field": …}`).
fn channel<'a>(encode: &'a Map<String, Value>, name: &str) -> Option<(&'a str, Option<&'a str>)> {
    let c = encode.get(name)?;
    let f = c.get("field").and_then(Value::as_str)?;
    Some((f, c.get("scale").and_then(Value::as_str)))
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one mark's whole context, read in one pass"
)]
fn read_mark(
    mark: &Value,
    kind: &str,
    data_name: &str,
    rows: &[Value],
    groupby: &[String],
    s: &Spec<'_>,
    chart: &mut ChartData,
    cx: &mut Cx<'_>,
) -> Result<(), Box<Diagnostic>> {
    let mut encode = Map::new();
    for set in ["enter", "update"] {
        if let Some(o) = mark
            .get("encode")
            .and_then(|e| e.get(set))
            .and_then(Value::as_object)
        {
            for (k, v) in o {
                encode.insert(k.clone(), v.clone());
            }
        }
    }
    let scale_kind = |scale: Option<&str>| scale.and_then(|n| s.scales.get(n)).copied();
    let discrete =
        |scale: Option<&str>| matches!(scale_kind(scale), Some("band" | "point" | "ordinal"));
    let (cat, val, chart_type) = if kind == "arc" {
        let pie = s
            .data
            .get(data_name)
            .and_then(|d| d.get("transform"))
            .and_then(Value::as_array)
            .and_then(|ts| {
                ts.iter()
                    .find(|t| t.get("type").and_then(Value::as_str) == Some("pie"))
            })
            .and_then(|t| t.get("field"))
            .and_then(Value::as_str);
        let cat = channel(&encode, "fill").or_else(|| channel(&encode, "stroke"));
        (cat, pie.map(|p| (p, None)), ChartType::Pie)
    } else {
        let x = channel(&encode, "x").or_else(|| channel(&encode, "xc"));
        let y = channel(&encode, "y").or_else(|| channel(&encode, "yc"));
        let (cat, val) =
            if y.is_some_and(|(_, sc)| discrete(sc)) && !x.is_some_and(|(_, sc)| discrete(sc)) {
                (y, x)
            } else {
                (x, y)
            };
        let stacked = s
            .data
            .get(data_name)
            .and_then(|d| d.get("transform"))
            .and_then(Value::as_array)
            .is_some_and(|ts| {
                ts.iter()
                    .any(|t| t.get("type").and_then(Value::as_str) == Some("stack"))
            });
        let chart_type = match kind {
            "line" => ChartType::Line,
            "area" => ChartType::Area,
            "rect" if stacked => ChartType::StackedBar,
            "rect" => ChartType::Bar,
            _ => ChartType::Scatter,
        };
        (cat, val, chart_type)
    };
    let Some((val_field, _)) = val else {
        return Ok(());
    };
    let xkind = match cat.and_then(|(_, sc)| scale_kind(sc)) {
        Some("time" | "utc") => XKind::Time,
        Some("band" | "point" | "ordinal") => XKind::Category,
        Some(_) => XKind::Number,
        None if kind == "arc" => XKind::Category,
        None => XKind::infer(rows.iter().filter_map(|r| field(r, cat?.0))),
    };
    let mut order: Vec<String> = Vec::new();
    let mut groups: BTreeMap<String, Vec<Point>> = BTreeMap::new();
    for (i, row) in rows.iter().enumerate() {
        let key: Vec<String> = groupby
            .iter()
            .map(|g| field(row, g).map(label).unwrap_or_default())
            .collect();
        let key = key.join(" / ");
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        let x = match cat {
            Some((f, _)) => {
                field(row, f).map_or_else(|| XKind::Category.x(&Value::Null), |v| xkind.x(v))
            }
            #[expect(clippy::cast_precision_loss, reason = "row indices far below 2^52")]
            None => XKind::Number.x(&Value::from((i + 1) as f64)),
        };
        groups.entry(key).or_default().push(Point {
            x,
            y: field(row, val_field).and_then(number),
        });
    }
    let fallback = mark
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(val_field)
        .to_owned();
    for key in order {
        let points = groups.remove(&key).unwrap_or_default();
        cx.count(points.len())?;
        let mut series = Series::exact(
            if key.is_empty() {
                fallback.clone()
            } else {
                key
            },
            points,
        );
        series.chart_type = Some(chart_type);
        chart.series.push(series);
    }
    chart.chart_type.get_or_insert(chart_type);
    if chart.x_title.is_none() {
        chart.x_title = cat.map(|(f, _)| f.to_owned());
    }
    if chart.y_title.is_none() && kind != "arc" {
        chart.y_title = Some(val_field.to_owned());
    }
    Ok(())
}
