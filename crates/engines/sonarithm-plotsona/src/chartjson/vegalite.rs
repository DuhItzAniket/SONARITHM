// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vega-Lite specifications (<https://vega.github.io/vega-lite/docs/>).
//!
//! A view's data is inherited by its layers and facets; its encoding by its layers. Each
//! unit view (a `mark` with an `encoding`) gives series: the category or time channel
//! (`x`, or `y` for horizontal bars), the value channel (the other one, or `theta` for
//! arcs) and the series channel (`color`, then `detail`, then a facet field). A
//! concatenation reads its first view. Transforms other than `fold` are not applied: they
//! need Vega's expression language, which PLOTSONA does not run.

use std::collections::BTreeMap;

use serde_json::{Map, Value};
use sonarithm_core::{Diagnostic, Provenance, ProvenanceClass};

use super::{Cx, XKind, field, label, number, title};
use crate::codes;
use crate::model::{ChartData, ChartType, Point, Series};

/// The data and encoding a view passes on to its children.
#[derive(Clone, Debug, Default)]
struct Inherited<'a> {
    data: Option<&'a Value>,
    encoding: Map<String, Value>,
    facet: Option<String>,
    transforms: Vec<&'a Value>,
}

pub(super) fn extract(spec: &Value, cx: &mut Cx<'_>) -> Result<ChartData, Box<Diagnostic>> {
    let datasets = spec.get("datasets").and_then(Value::as_object);
    let mut units = Vec::new();
    collect(spec, Inherited::default(), &mut units, cx, 0)?;
    let mut chart = ChartData {
        title: title(spec.get("title")),
        ..ChartData::default()
    };
    for unit in units {
        read_unit(&unit, datasets, &mut chart, cx)?;
    }
    Ok(chart)
}

/// A unit view with what it inherited.
struct Unit<'a> {
    mark: String,
    encoding: Map<String, Value>,
    data: Option<&'a Value>,
    facet: Option<String>,
    transforms: Vec<&'a Value>,
}

fn collect<'a>(
    view: &'a Value,
    parent: Inherited<'a>,
    units: &mut Vec<Unit<'a>>,
    cx: &mut Cx<'_>,
    depth: usize,
) -> Result<(), Box<Diagnostic>> {
    cx.work.spend(1)?;
    cx.budgets.check_depth(depth.saturating_add(1))?;
    let mut here = parent;
    if let Some(data) = view.get("data") {
        here.data = Some(data);
    }
    if let Some(encoding) = view.get("encoding").and_then(Value::as_object) {
        for (k, v) in encoding {
            here.encoding.insert(k.clone(), v.clone());
        }
    }
    if let Some(transforms) = view.get("transform").and_then(Value::as_array) {
        here.transforms.extend(transforms);
    }
    if let Some(mark) = view.get("mark") {
        let mark = mark
            .as_str()
            .or_else(|| mark.get("type").and_then(Value::as_str))
            .unwrap_or_default()
            .to_owned();
        units.push(Unit {
            mark,
            encoding: here.encoding,
            data: here.data,
            facet: here.facet,
            transforms: here.transforms,
        });
        return Ok(());
    }
    if let Some(layers) = view.get("layer").and_then(Value::as_array) {
        for layer in layers {
            collect(layer, here.clone(), units, cx, depth + 1)?;
        }
        return Ok(());
    }
    for key in ["concat", "hconcat", "vconcat"] {
        if let Some(views) = view.get(key).and_then(Value::as_array) {
            if views.len() > 1 {
                cx.warn(
                    &codes::OTHER_CHARTS,
                    format!(
                        "The specification concatenates {} views; the first was read.",
                        views.len()
                    ),
                );
            }
            if let Some(first) = views.first() {
                collect(first, here, units, cx, depth + 1)?;
            }
            return Ok(());
        }
    }
    if let (Some(facet), Some(inner)) = (view.get("facet"), view.get("spec")) {
        here.facet = ["field", "row", "column"]
            .iter()
            .find_map(|k| {
                let f = if *k == "field" {
                    Some(facet)
                } else {
                    facet.get(*k)
                };
                f?.get("field").and_then(Value::as_str)
            })
            .map(str::to_owned);
        return collect(inner, here, units, cx, depth + 1);
    }
    if view.get("repeat").is_some() {
        cx.warn(
            &codes::TRANSFORM_NOT_APPLIED,
            "Repeated views (`repeat`) are not read.".to_owned(),
        );
    }
    Ok(())
}

/// The rows of a view's data, or `None` with a diagnostic when they are not in the file.
fn rows<'a>(
    data: Option<&'a Value>,
    datasets: Option<&'a Map<String, Value>>,
    cx: &mut Cx<'_>,
) -> Option<Vec<Value>> {
    let data = data?;
    if let Some(url) = data.get("url") {
        cx.warn(
            &codes::REMOTE_DATA,
            format!("The data at {url} was not fetched; PLOTSONA reads only data inside the file."),
        );
        return None;
    }
    let values = match (data.get("values"), data.get("name").and_then(Value::as_str)) {
        (Some(values), _) => values,
        (None, Some(name)) => datasets?.get(name)?,
        _ => {
            cx.warn(
                &codes::SCRIPT_DATA,
                "The data is generated (`sequence`, `sphere` or `graticule`) rather than \
                 stored, and generators are not run."
                    .to_owned(),
            );
            return None;
        }
    };
    match values {
        Value::Array(items) => Some(
            items
                .iter()
                .map(|v| {
                    if v.is_object() {
                        v.clone()
                    } else {
                        // Vega-Lite wraps primitive values as `{"data": v}`.
                        let mut m = Map::new();
                        m.insert("data".to_owned(), v.clone());
                        Value::Object(m)
                    }
                })
                .collect(),
        ),
        Value::Object(o) if o.contains_key("signal") => {
            cx.warn(
                &codes::SCRIPT_DATA,
                "The data comes from a signal, which needs Vega's runtime.".to_owned(),
            );
            None
        }
        _ => {
            cx.warn(
                &codes::TRANSFORM_NOT_APPLIED,
                "Inline data written as text (CSV or TSV in `values`) is not read.".to_owned(),
            );
            None
        }
    }
}

/// Applies `fold` transforms; reports the others.
fn transform(rows: Vec<Value>, transforms: &[&Value], cx: &mut Cx<'_>) -> Vec<Value> {
    let mut rows = rows;
    for t in transforms {
        if let Some(fields) = t.get("fold").and_then(Value::as_array) {
            let names = t.get("as").and_then(Value::as_array);
            let key = names
                .and_then(|a| a.first())
                .and_then(Value::as_str)
                .unwrap_or("key")
                .to_owned();
            let value = names
                .and_then(|a| a.get(1))
                .and_then(Value::as_str)
                .unwrap_or("value")
                .to_owned();
            let mut folded = Vec::new();
            for row in &rows {
                for f in fields.iter().filter_map(Value::as_str) {
                    let mut out = row.as_object().cloned().unwrap_or_default();
                    out.insert(key.clone(), Value::String(f.to_owned()));
                    out.insert(value.clone(), field(row, f).cloned().unwrap_or(Value::Null));
                    folded.push(Value::Object(out));
                }
            }
            rows = folded;
        } else {
            let kind = t
                .as_object()
                .and_then(|o| o.keys().next())
                .cloned()
                .unwrap_or_default();
            cx.warn(
                &codes::TRANSFORM_NOT_APPLIED,
                format!(
                    "The `{kind}` transform was not applied; values are the stored data \
                     before it."
                ),
            );
        }
    }
    rows
}

#[expect(clippy::too_many_lines, reason = "one unit view read in one pass")]
fn read_unit(
    unit: &Unit<'_>,
    datasets: Option<&Map<String, Value>>,
    chart: &mut ChartData,
    cx: &mut Cx<'_>,
) -> Result<(), Box<Diagnostic>> {
    let Some(rows) = rows(unit.data, datasets, cx) else {
        return Ok(());
    };
    let rows = transform(rows, &unit.transforms, cx);
    let enc = &unit.encoding;
    let channel = |name: &str| enc.get(name).filter(|c| c.is_object());
    let discrete = |c: Option<&Value>| matches!(kind_of(c), Some("nominal" | "ordinal"));
    let arc = unit.mark == "arc";
    let (cat, val) = if arc {
        (channel("color"), channel("theta"))
    } else if discrete(channel("y")) && !discrete(channel("x")) {
        (channel("y"), channel("x"))
    } else {
        (channel("x"), channel("y"))
    };
    let series_field = if arc {
        None
    } else {
        channel("color")
            .or_else(|| channel("detail"))
            .and_then(|c| c.get("field"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| unit.facet.clone())
    };
    let cat_field = cat.and_then(|c| c.get("field")).and_then(Value::as_str);
    let val_field = val.and_then(|c| c.get("field")).and_then(Value::as_str);
    let aggregate = val
        .and_then(|c| c.get("aggregate"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    if val_field.is_none() && aggregate.as_deref() != Some("count") {
        return Ok(());
    }
    for c in [cat, val].into_iter().flatten() {
        for key in ["timeUnit", "bin"] {
            if c.get(key)
                .is_some_and(|v| !v.is_null() && v != &Value::Bool(false))
            {
                cx.warn(
                    &codes::TRANSFORM_NOT_APPLIED,
                    format!(
                        "A `{key}` on an encoding was not applied; values are the stored data."
                    ),
                );
            }
        }
    }
    let kind = match kind_of(cat) {
        Some("temporal") => XKind::Time,
        Some("quantitative") => XKind::Number,
        Some(_) => XKind::Category,
        None => XKind::infer(rows.iter().filter_map(|r| field(r, cat_field?))),
    };
    // Group rows by series, keeping first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut groups: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for row in &rows {
        let key = series_field
            .as_deref()
            .and_then(|f| field(row, f))
            .map(label)
            .unwrap_or_default();
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(row);
    }
    let fallback_name = val
        .and_then(|c| title(c.get("title")))
        .or_else(|| val_field.map(str::to_owned))
        .unwrap_or_else(|| "count".to_owned());
    for key in order {
        let members = groups.remove(&key).unwrap_or_default();
        let (points, derived) = match &aggregate {
            Some(op) => (
                aggregate_points(&members, cat_field, val_field, op, kind, cx),
                true,
            ),
            None => (
                members
                    .iter()
                    .map(|row| Point {
                        x: cat_field
                            .and_then(|f| field(row, f))
                            .map_or_else(|| XKind::Category.x(&Value::Null), |v| kind.x(v)),
                        y: val_field.and_then(|f| field(row, f)).and_then(number),
                    })
                    .collect(),
                false,
            ),
        };
        cx.count(points.len())?;
        let mut series = Series::exact(
            if key.is_empty() {
                fallback_name.clone()
            } else {
                key
            },
            sort(points, cat, kind),
        );
        if derived {
            series.provenance = Provenance {
                class: ProvenanceClass::Derived,
                confidence: None,
            };
        }
        series.chart_type = Some(chart_type(&unit.mark, enc, series_field.is_some()));
        chart.series.push(series);
    }
    chart.chart_type = chart
        .chart_type
        .or_else(|| Some(chart_type(&unit.mark, enc, series_field.is_some())));
    if chart.x_title.is_none() {
        chart.x_title = cat
            .and_then(|c| title(c.get("title")))
            .or_else(|| cat_field.map(str::to_owned));
    }
    if chart.y_title.is_none() && !arc {
        chart.y_title = Some(fallback_name);
    }
    Ok(())
}

/// One point per category with the aggregate of its values (03 §4: `derived`).
fn aggregate_points(
    rows: &[&Value],
    cat_field: Option<&str>,
    val_field: Option<&str>,
    op: &str,
    kind: XKind,
    cx: &mut Cx<'_>,
) -> Vec<Point> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: BTreeMap<String, (Value, Vec<f64>)> = BTreeMap::new();
    for row in rows {
        let x = cat_field
            .and_then(|f| field(row, f))
            .cloned()
            .unwrap_or(Value::Null);
        let key = label(&x);
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        let entry = groups.entry(key).or_insert_with(|| (x, Vec::new()));
        match val_field.and_then(|f| field(row, f)).and_then(number) {
            Some(v) => entry.1.push(v),
            None if op == "count" => entry.1.push(0.0),
            None => {}
        }
    }
    order
        .into_iter()
        .filter_map(|key| groups.remove(&key))
        .map(|(x, values)| Point {
            x: kind.x(&x),
            y: reduce(op, values, cx),
        })
        .collect()
}

#[expect(clippy::cast_precision_loss, reason = "row counts far below 2^52")]
fn reduce(op: &str, mut values: Vec<f64>, cx: &mut Cx<'_>) -> Option<f64> {
    let n = values.len() as f64;
    match op {
        "count" => Some(n),
        "sum" => Some(values.iter().sum()),
        "mean" | "average" => (n > 0.0).then(|| values.iter().sum::<f64>() / n),
        "min" => values.into_iter().reduce(f64::min),
        "max" => values.into_iter().reduce(f64::max),
        "median" => {
            values.sort_by(f64::total_cmp);
            let mid = values.len() / 2;
            match values.len() {
                0 => None,
                len if len % 2 == 1 => values.get(mid).copied(),
                _ => Some((values.get(mid - 1)? + values.get(mid)?) / 2.0),
            }
        }
        other => {
            cx.warn(
                &codes::TRANSFORM_NOT_APPLIED,
                format!("The `{other}` aggregate is not computed."),
            );
            None
        }
    }
}

/// Orders points as Vega-Lite draws them: continuous axes ascending; discrete axes
/// ascending unless `sort` says otherwise (`null` keeps the data order, an array gives
/// the order, `descending` reverses, `y`/`-y` sort by value).
fn sort(mut points: Vec<Point>, cat: Option<&Value>, kind: XKind) -> Vec<Point> {
    use crate::model::XValue;
    let sort = cat.and_then(|c| c.get("sort"));
    let by_x = |a: &Point, b: &Point| match (&a.x, &b.x) {
        (XValue::Number(x), XValue::Number(y)) => x.total_cmp(y),
        (XValue::Time(x), XValue::Time(y)) => x.cmp(y),
        (x, y) => super::label(&serde_json::to_value(x).unwrap_or_default())
            .cmp(&super::label(&serde_json::to_value(y).unwrap_or_default())),
    };
    let by_label = |a: &Point, b: &Point| match (&a.x, &b.x) {
        (XValue::Category(x), XValue::Category(y)) => x.cmp(y),
        _ => by_x(a, b),
    };
    let by_value = |a: &Point, b: &Point| {
        a.y.unwrap_or(f64::NEG_INFINITY)
            .total_cmp(&b.y.unwrap_or(f64::NEG_INFINITY))
    };
    match sort {
        Some(Value::Null) => {}
        Some(Value::Array(order)) => {
            let rank = |p: &Point| {
                match &p.x {
                    XValue::Category(l) => order.iter().position(|o| label(o) == *l),
                    _ => None,
                }
                .unwrap_or(usize::MAX)
            };
            points.sort_by_key(|p| rank(p));
        }
        Some(Value::String(s)) if s == "descending" => {
            points.sort_by(by_label);
            points.reverse();
        }
        Some(Value::String(s)) if s.trim_start_matches('-') != s && s.len() > 1 => {
            points.sort_by(by_value);
            points.reverse();
        }
        Some(Value::String(s)) if s != "ascending" => points.sort_by(by_value),
        Some(Value::Object(o)) => {
            points.sort_by(by_value);
            if o.get("order").and_then(Value::as_str) == Some("descending") {
                points.reverse();
            }
        }
        _ if kind == XKind::Category => points.sort_by(by_label),
        _ => points.sort_by(by_x),
    }
    points
}

/// The `type` of an encoding channel.
fn kind_of(channel: Option<&Value>) -> Option<&str> {
    channel.and_then(|c| c.get("type")).and_then(Value::as_str)
}

fn chart_type(mark: &str, enc: &Map<String, Value>, has_series: bool) -> ChartType {
    match mark {
        "line" | "trail" => ChartType::Line,
        "area" => ChartType::Area,
        "bar" => {
            let unstacked = ["x", "y"].iter().any(|c| {
                enc.get(*c)
                    .and_then(|c| c.get("stack"))
                    .is_some_and(|s| s.is_null() || s == &Value::Bool(false))
            }) || enc.contains_key("xOffset")
                || enc.contains_key("yOffset");
            if has_series && !unstacked {
                ChartType::StackedBar
            } else {
                ChartType::Bar
            }
        }
        "point" | "circle" | "square" | "tick" => ChartType::Scatter,
        "arc" => ChartType::Pie,
        "rect" => ChartType::Heatmap,
        "boxplot" => ChartType::BoxPlot,
        _ => ChartType::Other,
    }
}
