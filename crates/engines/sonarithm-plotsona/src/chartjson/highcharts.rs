// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Highcharts options (<https://api.highcharts.com/highcharts/>). The library itself is not
//! open source, so it is never used or bundled; this reads its documented options format
//! (checked against `series.line.data` in the API reference, 2026-10-26).
//!
//! `series[].data` takes three forms: plain values, whose x comes from `xAxis.categories`
//! or from `pointStart` and `pointInterval` (0 and 1 by default); `[x, y]` pairs, where a
//! string first item is the point's name; and point objects with `x`, `y` and `name`. A
//! `datetime` x axis reads x as milliseconds since the epoch. Stacking comes from
//! `plotOptions` or the series.

use serde_json::{Map, Value};
use sonarithm_core::Diagnostic;

use super::{Cx, label, number, title};
use crate::model::{ChartData, ChartType, Point, Series, XValue};

#[expect(
    clippy::too_many_lines,
    reason = "one pass over the documented options"
)]
pub(super) fn extract(options: &Value, cx: &mut Cx<'_>) -> Result<ChartData, Box<Diagnostic>> {
    let first = |v: Option<&Value>| match v {
        Some(Value::Array(a)) => a.first().cloned(),
        other => other.cloned(),
    };
    let x_axis = first(options.get("xAxis")).unwrap_or(Value::Null);
    let y_axis = first(options.get("yAxis")).unwrap_or(Value::Null);
    let categories: Vec<String> = x_axis
        .get("categories")
        .and_then(Value::as_array)
        .map(|c| c.iter().map(label).collect())
        .unwrap_or_default();
    let datetime = x_axis.get("type").and_then(Value::as_str) == Some("datetime");
    let default_type = options
        .get("chart")
        .and_then(|c| c.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("line");
    let plot = options.get("plotOptions");
    let mut chart = ChartData {
        title: title(options.get("title")),
        x_title: title(x_axis.get("title")),
        y_title: title(y_axis.get("title")),
        ..ChartData::default()
    };
    for (index, series) in options
        .get("series")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        cx.work.spend(1)?;
        let Some(series) = series.as_object() else {
            continue;
        };
        let kind = series
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or(default_type);
        // Series options override the type's plot options, which override `series`.
        let option = |key: &str| {
            series
                .get(key)
                .or_else(|| plot.and_then(|p| p.get(kind)).and_then(|o| o.get(key)))
                .or_else(|| plot.and_then(|p| p.get("series")).and_then(|o| o.get(key)))
        };
        let start = option("pointStart").and_then(number).unwrap_or(0.0);
        let interval = option("pointInterval").and_then(number).unwrap_or(1.0);
        let stacked = option("stacking")
            .and_then(Value::as_str)
            .is_some_and(|s| matches!(s, "normal" | "percent" | "stream" | "overlap"));
        let chart_type = match kind {
            "line" | "spline" | "step" => ChartType::Line,
            "area" | "areaspline" | "arearange" => ChartType::Area,
            "column" | "bar" if stacked => ChartType::StackedBar,
            "column" | "bar" | "columnrange" => ChartType::Bar,
            "scatter" | "bubble" => ChartType::Scatter,
            "pie" | "variablepie" => ChartType::Pie,
            "boxplot" => ChartType::BoxPlot,
            "heatmap" => ChartType::Heatmap,
            "histogram" => ChartType::Histogram,
            _ => ChartType::Other,
        };
        let pie = chart_type == ChartType::Pie;
        let data = series
            .get("data")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        let x_of = |i: usize, given: Option<&Value>| -> XValue {
            if let Some(x) = given {
                return if datetime {
                    super::time(x).map_or_else(|| XValue::Category(label(x)), XValue::Time)
                } else {
                    number(x).map_or_else(|| XValue::Category(label(x)), XValue::Number)
                };
            }
            if let Some(c) = categories.get(i) {
                return XValue::Category(c.clone());
            }
            #[expect(clippy::cast_precision_loss, reason = "point indices far below 2^52")]
            let x = start + interval * i as f64;
            if datetime {
                #[expect(clippy::cast_possible_truncation, reason = "times in ms")]
                let ms = x.round() as i64;
                XValue::Time(ms)
            } else {
                XValue::Number(x)
            }
        };
        let points: Vec<Point> = data
            .iter()
            .enumerate()
            .map(|(i, item)| point(i, item, pie, &x_of))
            .collect();
        cx.count(points.len())?;
        let name = series
            .get("name")
            .map(label)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| format!("Series {}", index + 1));
        let mut s = Series::exact(name, points);
        s.chart_type = Some(chart_type);
        chart.series.push(s);
        chart.chart_type.get_or_insert(chart_type);
    }
    Ok(chart)
}

fn point(
    i: usize,
    item: &Value,
    pie: bool,
    x_of: &impl Fn(usize, Option<&Value>) -> XValue,
) -> Point {
    match item {
        Value::Array(pair) => match pair.as_slice() {
            // `["name", y]`: the point is named; x is inferred.
            [Value::String(name), y, ..] => Point {
                x: if pie {
                    XValue::Category(name.clone())
                } else {
                    named(i, name, x_of)
                },
                y: number(y),
            },
            [x, y, ..] => Point {
                x: x_of(i, Some(x)),
                y: number(y),
            },
            [y] => Point {
                x: x_of(i, None),
                y: number(y),
            },
            [] => Point {
                x: x_of(i, None),
                y: None,
            },
        },
        Value::Object(o) => object_point(i, o, pie, x_of),
        other => Point {
            x: x_of(i, None),
            y: number(other),
        },
    }
}

fn object_point(
    i: usize,
    o: &Map<String, Value>,
    pie: bool,
    x_of: &impl Fn(usize, Option<&Value>) -> XValue,
) -> Point {
    let name = o.get("name").map(label);
    let x = match (o.get("x"), &name) {
        (Some(x), _) => x_of(i, Some(x)),
        (None, Some(n)) if pie => XValue::Category(n.clone()),
        (None, Some(n)) => named(i, n, x_of),
        (None, None) => x_of(i, None),
    };
    Point {
        x,
        y: o.get("y").and_then(number),
    }
}

/// A named point without x: its category or position, else its name.
fn named(i: usize, name: &str, x_of: &impl Fn(usize, Option<&Value>) -> XValue) -> XValue {
    match x_of(i, None) {
        XValue::Category(c) if !c.is_empty() => XValue::Category(c),
        XValue::Number(_) | XValue::Time(_) => XValue::Category(name.to_owned()),
        other @ XValue::Category(_) => other,
    }
}
