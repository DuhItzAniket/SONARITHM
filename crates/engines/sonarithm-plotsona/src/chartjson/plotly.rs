// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Plotly figures (<https://plotly.com/javascript/reference/>).
//!
//! A figure is `{"data": [traces], "layout": {…}}`. Only the top-level `data` holds the
//! chart's traces; `layout.template.data` holds style prototypes and is never read. A
//! trace's coordinates are arrays, a start and step (`x0`, `dx`) when an array is absent,
//! or typed arrays (`{"dtype": "f8", "bdata": "<base64>"}`), which plotly.py 6 writes for
//! `NumPy` arrays (checked against plotly 6.7.0's output, 2026-10-26). Pie traces use
//! `labels` and `values`; heat maps a `z` grid, one series per row.

use serde_json::{Map, Value};
use sonarithm_core::Diagnostic;

use super::{Cx, XKind, label, number, title};
use crate::codes;
use crate::model::{ChartData, ChartType, Point, Series, XValue};

pub(super) fn extract(figure: &Value, cx: &mut Cx<'_>) -> Result<ChartData, Box<Diagnostic>> {
    let (traces, layout) = match figure {
        Value::Array(traces) => (traces.as_slice(), None),
        other => (
            other
                .get("data")
                .and_then(Value::as_array)
                .map_or(&[][..], Vec::as_slice),
            other.get("layout").and_then(Value::as_object),
        ),
    };
    let axis = |name: &str| layout.and_then(|l| l.get(name));
    let mut chart = ChartData {
        title: title(layout.and_then(|l| l.get("title"))),
        x_title: title(axis("xaxis").and_then(|a| a.get("title"))),
        y_title: title(axis("yaxis").and_then(|a| a.get("title"))),
        ..ChartData::default()
    };
    let barmode = layout
        .and_then(|l| l.get("barmode"))
        .and_then(Value::as_str)
        .unwrap_or("group");
    for (index, trace) in traces.iter().enumerate() {
        cx.work.spend(1)?;
        let Some(trace) = trace.as_object() else {
            continue;
        };
        let kind = trace
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("scatter");
        let name = trace
            .get("name")
            .map(label)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| format!("trace {index}"));
        let chart_type = chart_type(kind, trace, barmode);
        let read = match kind {
            "pie" => pie(trace, cx)?,
            "heatmap" => {
                heatmap(trace, &name, chart_type, &mut chart, cx)?;
                continue;
            }
            "histogram" | "box" | "violin" => samples(trace, cx)?,
            _ => xy(trace, layout, cx)?,
        };
        cx.count(read.len())?;
        let mut series = Series::exact(name, read);
        series.chart_type = Some(chart_type);
        chart.series.push(series);
        chart.chart_type.get_or_insert(chart_type);
    }
    Ok(chart)
}

fn chart_type(kind: &str, trace: &Map<String, Value>, barmode: &str) -> ChartType {
    let fill = trace
        .get("fill")
        .and_then(Value::as_str)
        .is_some_and(|f| f != "none")
        || trace.get("stackgroup").is_some();
    let mode = trace.get("mode").and_then(Value::as_str).unwrap_or("lines");
    match kind {
        "scatter" | "scattergl" if fill => ChartType::Area,
        "scatter" | "scattergl" if mode.contains("lines") => ChartType::Line,
        "scatter" | "scattergl" => ChartType::Scatter,
        "bar" if matches!(barmode, "stack" | "relative") => ChartType::StackedBar,
        "bar" => ChartType::Bar,
        "pie" => ChartType::Pie,
        "histogram" => ChartType::Histogram,
        "box" | "violin" => ChartType::BoxPlot,
        "heatmap" => ChartType::Heatmap,
        _ => ChartType::Other,
    }
}

/// An `x`/`y` trace; a horizontal bar (`orientation: "h"`) has its categories in `y`.
fn xy(
    trace: &Map<String, Value>,
    layout: Option<&Map<String, Value>>,
    cx: &mut Cx<'_>,
) -> Result<Vec<Point>, Box<Diagnostic>> {
    let axis_type = |name: &str| {
        layout
            .and_then(|l| l.get(name))
            .and_then(|a| a.get("type"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let horizontal = trace.get("orientation").and_then(Value::as_str) == Some("h");
    let (cat, val, cat_axis) = if horizontal {
        ("y", "x", "yaxis")
    } else {
        ("x", "y", "xaxis")
    };
    let values = array(trace, val, cx)?.unwrap_or_default();
    let cats = if let Some(c) = array(trace, cat, cx)? {
        c
    } else {
        // No array: a linear space from `x0` by `dx` (0 and 1 by default).
        let start = trace
            .get(&format!("{cat}0"))
            .and_then(number)
            .unwrap_or(0.0);
        let step = trace
            .get(&format!("d{cat}"))
            .and_then(number)
            .unwrap_or(1.0);
        #[expect(clippy::cast_precision_loss, reason = "point indices far below 2^52")]
        let made = (0..values.len())
            .map(|i| Value::from(start + step * i as f64))
            .collect();
        made
    };
    let kind = match axis_type(cat_axis).as_deref() {
        Some("date") => XKind::Time,
        Some("category" | "multicategory") => XKind::Category,
        Some("linear" | "log") => XKind::Number,
        _ => XKind::infer(cats.iter()),
    };
    Ok(values
        .iter()
        .enumerate()
        .map(|(i, v)| Point {
            x: cats
                .get(i)
                .map_or(XValue::Category(String::new()), |c| kind.x(c)),
            y: number(v),
        })
        .collect())
}

fn pie(trace: &Map<String, Value>, cx: &mut Cx<'_>) -> Result<Vec<Point>, Box<Diagnostic>> {
    let values = array(trace, "values", cx)?.unwrap_or_default();
    let labels = array(trace, "labels", cx)?.unwrap_or_default();
    Ok(values
        .iter()
        .enumerate()
        .map(|(i, v)| Point {
            x: XValue::Category(labels.get(i).map_or_else(|| i.to_string(), label)),
            y: number(v),
        })
        .collect())
}

/// Raw samples (histograms and box plots draw summaries of them): one point per sample,
/// at its 1-based position.
fn samples(trace: &Map<String, Value>, cx: &mut Cx<'_>) -> Result<Vec<Point>, Box<Diagnostic>> {
    let values = match array(trace, "x", cx)? {
        Some(x) if trace.get("y").is_none() => x,
        _ => array(trace, "y", cx)?.unwrap_or_default(),
    };
    #[expect(clippy::cast_precision_loss, reason = "sample indices far below 2^52")]
    let points = values
        .iter()
        .enumerate()
        .map(|(i, v)| Point::xy((i + 1) as f64, number(v).unwrap_or(f64::NAN)))
        .map(|mut p| {
            p.y = p.y.filter(|y| y.is_finite());
            p
        })
        .collect();
    Ok(points)
}

/// A heat map: one series per row of `z`, named by `y`, at the columns of `x`.
fn heatmap(
    trace: &Map<String, Value>,
    name: &str,
    chart_type: ChartType,
    chart: &mut ChartData,
    cx: &mut Cx<'_>,
) -> Result<(), Box<Diagnostic>> {
    let rows: Vec<Value> = match trace.get("z") {
        Some(Value::Array(rows)) => rows.clone(),
        Some(typed @ Value::Object(_)) => {
            let shape: Vec<usize> = typed
                .get("shape")
                .and_then(|s| match s {
                    Value::String(s) => {
                        Some(s.split(',').filter_map(|n| n.trim().parse().ok()).collect())
                    }
                    Value::Array(a) => Some(
                        a.iter()
                            .filter_map(|n| n.as_u64().and_then(|n| usize::try_from(n).ok()))
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_default();
            let flat = decode_typed(typed, cx)?.unwrap_or_default();
            match shape.as_slice() {
                [_, cols] if *cols > 0 => flat
                    .chunks(*cols)
                    .map(|c| Value::Array(c.to_vec()))
                    .collect(),
                _ => vec![Value::Array(flat)],
            }
        }
        _ => Vec::new(),
    };
    let xs = array(trace, "x", cx)?;
    let ys = array(trace, "y", cx)?;
    for (r, row) in rows.iter().enumerate() {
        let cells = row.as_array().map_or(&[][..], Vec::as_slice);
        cx.count(cells.len())?;
        let points = cells
            .iter()
            .enumerate()
            .map(|(c, v)| Point {
                x: xs
                    .as_ref()
                    .and_then(|x| x.get(c))
                    .map_or_else(|| XValue::Category(c.to_string()), |x| XKind::Category.x(x)),
                y: number(v),
            })
            .collect();
        let row_name = ys
            .as_ref()
            .and_then(|y| y.get(r))
            .map_or_else(|| format!("{name} row {r}"), label);
        let mut series = Series::exact(row_name, points);
        series.chart_type = Some(chart_type);
        chart.series.push(series);
    }
    chart.chart_type.get_or_insert(chart_type);
    Ok(())
}

/// A data array: a JSON array or a typed array.
fn array(
    trace: &Map<String, Value>,
    key: &str,
    cx: &mut Cx<'_>,
) -> Result<Option<Vec<Value>>, Box<Diagnostic>> {
    match trace.get(key) {
        Some(Value::Array(items)) => Ok(Some(items.clone())),
        Some(typed @ Value::Object(_)) if typed.get("bdata").is_some() => decode_typed(typed, cx),
        Some(Value::String(s)) if !s.is_empty() => {
            cx.warn(
                &codes::SCRIPT_DATA,
                format!(
                    "The trace's `{key}` is a reference (`{s}`), not data; it was not resolved."
                ),
            );
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// A typed array: little-endian values of `dtype` (`f8`, `f4`, `i1` to `i4`, `u1` to
/// `u4`), base64-encoded in `bdata`.
fn decode_typed(typed: &Value, cx: &mut Cx<'_>) -> Result<Option<Vec<Value>>, Box<Diagnostic>> {
    let dtype = typed.get("dtype").and_then(Value::as_str).unwrap_or("f8");
    let Some(bytes) = typed.get("bdata").and_then(Value::as_str).and_then(base64) else {
        cx.warn(
            &codes::VALUE_UNREADABLE,
            "A typed array is not valid base64.".to_owned(),
        );
        return Ok(None);
    };
    cx.work.spend(crate::budget::len64(bytes.len()))?;
    let width = match dtype {
        "f8" => 8,
        "f4" | "i4" | "u4" => 4,
        "i2" | "u2" => 2,
        "i1" | "u1" => 1,
        other => {
            cx.warn(
                &codes::VALUE_UNREADABLE,
                format!("The typed-array type `{other}` is not read."),
            );
            return Ok(None);
        }
    };
    let values = bytes
        .chunks_exact(width)
        .map(|c| {
            let v = match dtype {
                "f8" => f64::from_le_bytes(le(c)),
                "f4" => f64::from(f32::from_le_bytes(le(c))),
                "i4" => f64::from(i32::from_le_bytes(le(c))),
                "u4" => f64::from(u32::from_le_bytes(le(c))),
                "i2" => f64::from(i16::from_le_bytes(le(c))),
                "u2" => f64::from(u16::from_le_bytes(le(c))),
                "i1" => f64::from(i8::from_le_bytes(le(c))),
                _ => f64::from(u8::from_le_bytes(le(c))),
            };
            serde_json::Number::from_f64(v).map_or(Value::Null, Value::Number)
        })
        .collect();
    Ok(Some(values))
}

/// A chunk as a fixed-size array (chunks come from `chunks_exact` of the same width).
fn le<const N: usize>(chunk: &[u8]) -> [u8; N] {
    chunk.try_into().unwrap_or([0; N])
}

/// Standard base64 with optional padding, or `None` if it is not valid.
fn base64(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut buffer = 0_u32;
    let mut bits = 0;
    for c in text.bytes().filter(|b| !b.is_ascii_whitespace()) {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            _ => return None,
        };
        buffer = (buffer << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((buffer >> bits) & 0xFF).ok()?);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decodes_typed_arrays() {
        assert_eq!(base64("AAAAAAAA+D8="), Some(1.5_f64.to_le_bytes().to_vec()));
        assert_eq!(base64("aGk"), Some(b"hi".to_vec()));
        assert_eq!(base64("a*b"), None);
    }
}
