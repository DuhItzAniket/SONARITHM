// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Vega-Lite importer (C01-T10; <https://vega.github.io/vega-lite/docs/>), written from
//! the published documentation.
//!
//! | Vega-Lite | SONARITHM v0.2 |
//! |---|---|
//! | `title`, `description` | `name`, `description` |
//! | `data.values`, `data.name` + `datasets`, `data.url` (a relative path) | `data.source` `inline` or `file`; a URL on the network is reported and not read (ADR-0012) |
//! | `transform`: `filter` with field predicates, `aggregate` (`count`, `sum`, `mean`, `median`, `min`, `max`), `bin` | `filter`, `aggregate`, `bin` (equal-width bins: approximated) |
//! | `mark` `line`, `trail`, `area` | template `chart-line` |
//! | `mark` `bar` | `chart-bar`; with a colour and no `xOffset`, bars stack, so `chart-stacked-bar` |
//! | `mark` `point`, `circle`, `square`, `tick` | `scatter-xy` on two numeric axes, else `chart-line` |
//! | `mark` `arc` | `chart-pie` (`theta` the value, `color` the slice) |
//! | `mark` `rect` | `heatmap-rowscan` (`y` the row, `x` the column, `color` the value) |
//! | `mark` `boxplot` | `box-five-number` |
//! | `encoding` `x`, `y` | time and pitch, through the template's slots; horizontal bars swap them |
//! | `encoding` `color`, `fill`, `stroke` (then `detail`) | the series slot: timbre (SDR-25) |
//! | a channel's `aggregate` (the six above) and `bin` | `aggregate` and `bin` transforms |
//! | a channel's `type`, `title` (or `axis.title`) | `data.fields` type and label |
//!
//! A `layer` imports its first layer and a concatenation its first view; facets, repeats,
//! the other layers and views, selections and parameters are reported. The keys `$schema`,
//! `width`, `height`, `autosize`, `padding`, `background`, `config`, `view`, `usermeta`,
//! `projection`, `resolve`, `spacing`, `bounds`, `align`, `center`, `columns` and `name`
//! only style or name the picture and are skipped without a report, as are a channel's
//! `axis`, `legend`, `scale` colours and `sort` of the picture.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use sonarithm_core::Diagnostic;

use super::{Draft, Import, ImportReport, text_of, token, unreadable};
use crate::budget::{Budgets, Work};
use crate::codes;
use crate::detect;

const IMPORTER: &str = "Vega-Lite";

/// Top-level keys that only style or name the picture.
const VISUAL: &[&str] = &[
    "$schema",
    "width",
    "height",
    "autosize",
    "padding",
    "background",
    "config",
    "view",
    "usermeta",
    "projection",
    "resolve",
    "spacing",
    "bounds",
    "align",
    "center",
    "columns",
    "name",
];

/// Channels that place or name a series; every other encoding channel is reported.
const MAPPED_CHANNELS: &[&str] = &[
    "x", "y", "color", "fill", "stroke", "theta", "detail", "xOffset",
];

/// Imports a Vega-Lite specification.
///
/// # Errors
///
/// `SNR-ENG-301` for input over the size budget, `SNR-ENG-307` or `SNR-ENG-304` for JSON
/// that is invalid or too deep, `SNR-ENG-383` for a document that is not a Vega-Lite unit
/// or layered view with a supported mark, and `SNR-ENG-384` when it has no data SONARITHM
/// can read.
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
pub fn import_value(spec: &Value, budgets: &Budgets) -> Result<Import, Box<Diagnostic>> {
    let root = spec
        .as_object()
        .ok_or_else(|| unreadable(IMPORTER, "the document is not a JSON object"))?;
    let mut report = ImportReport::default();
    let mut view = View::default();
    find_unit(root, "", &mut view, &mut report, 0, budgets)?;
    let mark = view
        .mark
        .clone()
        .ok_or_else(|| unreadable(IMPORTER, "the document has no `mark` to sonify"))?;
    let mut draft = Draft {
        name: text_of(root.get("title")),
        description: text_of(root.get("description")),
        ..Draft::default()
    };
    draft.source = source(&view, root.get("datasets"), &mut report, budgets)?;
    for (pointer, transform) in &view.transforms {
        if let Some(mapped) = transform_of(transform, pointer, &mut report) {
            draft.transforms.push(mapped);
        }
    }
    design(&mark, &view, &mut draft, &mut report)?;
    Ok(Import {
        spec: draft.finish(),
        report,
    })
}

/// A unit view with what it inherited, and where each part is in the document.
#[derive(Debug, Default)]
struct View {
    mark: Option<String>,
    mark_at: String,
    data: Option<(String, Value)>,
    encoding: Map<String, Value>,
    encoding_at: Map<String, Value>,
    transforms: Vec<(String, Value)>,
}

/// Walks from `node` to the first unit view, collecting what it inherits.
fn find_unit(
    node: &Map<String, Value>,
    at: &str,
    view: &mut View,
    report: &mut ImportReport,
    depth: usize,
    budgets: &Budgets,
) -> Result<(), Box<Diagnostic>> {
    budgets.check_depth(depth.saturating_add(1))?;
    report_unknown_keys(node, at, report);
    if let Some(data) = node.get("data") {
        view.data = Some((format!("{at}/data"), data.clone()));
    }
    if let Some(transforms) = node.get("transform").and_then(Value::as_array) {
        for (i, t) in transforms.iter().enumerate() {
            view.transforms
                .push((format!("{at}/transform/{i}"), t.clone()));
        }
    }
    if let Some(encoding) = node.get("encoding").and_then(Value::as_object) {
        for (channel, value) in encoding {
            view.encoding.insert(channel.clone(), value.clone());
            view.encoding_at.insert(
                channel.clone(),
                json!(format!("{at}/encoding/{}", token(channel))),
            );
        }
    }
    for key in ["params", "selection"] {
        if node.contains_key(key) {
            report.unmapped(
                &format!("{at}/{key}"),
                key,
                "selections and parameters are interactive; sound navigation replaces them",
            );
        }
    }
    for key in ["facet", "repeat"] {
        if node.contains_key(key) {
            report.unmapped(
                &format!("{at}/{key}"),
                key,
                "the facets play together as one dataset; write a `filter` per facet to hear them apart",
            );
        }
    }
    if let Some(mark) = node.get("mark") {
        view.mark = mark
            .as_str()
            .or_else(|| mark.get("type").and_then(Value::as_str))
            .map(str::to_owned);
        view.mark_at = format!("{at}/mark");
        return Ok(());
    }
    for key in ["layer", "concat", "hconcat", "vconcat"] {
        if let Some(children) = node.get(key).and_then(Value::as_array) {
            let Some(first) = children.first().and_then(Value::as_object) else {
                return Err(unreadable(IMPORTER, format!("`{key}` holds no view")));
            };
            for i in 1..children.len() {
                report.unmapped(
                    &format!("{at}/{key}/{i}"),
                    key,
                    format!("only the first view is imported; add a track for view {i} by hand"),
                );
            }
            return find_unit(
                first,
                &format!("{at}/{key}/0"),
                view,
                report,
                depth + 1,
                budgets,
            );
        }
    }
    if let Some(inner) = node.get("spec").and_then(Value::as_object) {
        return find_unit(
            inner,
            &format!("{at}/spec"),
            view,
            report,
            depth + 1,
            budgets,
        );
    }
    Ok(())
}

/// Reports the keys of a view that SONARITHM neither reads nor skips as visual.
fn report_unknown_keys(node: &Map<String, Value>, at: &str, report: &mut ImportReport) {
    const READ: &[&str] = &[
        "title",
        "description",
        "data",
        "datasets",
        "transform",
        "encoding",
        "mark",
        "layer",
        "concat",
        "hconcat",
        "vconcat",
        "spec",
        "params",
        "selection",
        "facet",
        "repeat",
    ];
    for key in node.keys() {
        if !VISUAL.contains(&key.as_str()) && !READ.contains(&key.as_str()) {
            report.unmapped(
                &format!("{at}/{}", token(key)),
                key,
                "not a Vega-Lite key SONARITHM reads",
            );
        }
    }
}

/// The spec's `data.source`.
fn source(
    view: &View,
    datasets: Option<&Value>,
    report: &mut ImportReport,
    budgets: &Budgets,
) -> Result<Value, Box<Diagnostic>> {
    let no_data = |why: &str| {
        Box::new(Diagnostic::new(
            &codes::IMPORT_NO_DATA,
            format!("PLOTSONA {IMPORTER} import: {why}."),
        ))
    };
    let Some((at, data)) = &view.data else {
        return Err(no_data("the chart has no `data`"));
    };
    let inline = |values: &Value| -> Result<Value, Box<Diagnostic>> {
        let rows = values
            .as_array()
            .ok_or_else(|| no_data("inline `values` are not a list of records"))?;
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
        Ok(json!({ "kind": "inline", "values": rows }))
    };
    if let Some(values) = data.get("values") {
        if data.get("format").is_some() {
            report.approximated(
                &format!("{at}/format"),
                "data.format",
                "values are read as records",
            );
        }
        return inline(values);
    }
    if let Some(name) = data.get("name").and_then(Value::as_str) {
        let values = datasets
            .and_then(|d| d.get(name))
            .ok_or_else(|| no_data(&format!("the dataset `{name}` is not in `datasets`")))?;
        return inline(values);
    }
    if let Some(url) = data.get("url").and_then(Value::as_str) {
        let remote = url.contains("://") || url.starts_with("//") || url.starts_with("data:");
        if remote {
            report.unmapped(
                &format!("{at}/url"),
                "data.url",
                "SONARITHM does not fetch data from the network (ADR-0012); download the file and point `path` at it",
            );
            return Err(no_data(
                "its data is on the network, which SONARITHM never reads",
            ));
        }
        if data.get("format").is_some() {
            report.approximated(
                &format!("{at}/format"),
                "data.format",
                "the file's format is taken from its extension",
            );
        }
        return Ok(json!({ "kind": "file", "path": url }));
    }
    report.unmapped(
        at,
        "data",
        "generated data (sequence, sphere, graticule) has no equivalent",
    );
    Err(no_data(
        "its data is generated, which SONARITHM cannot import",
    ))
}

/// One literal for a filter expression.
fn literal(value: &Value) -> Option<String> {
    match value {
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::String(s) => Some(format!("{s:?}")),
        _ => None,
    }
}

/// A field predicate (`{ "field": …, "lt": … }`) as an expression, or `None`.
fn predicate(p: &Map<String, Value>) -> Option<String> {
    if let Some(and) = p.get("and").and_then(Value::as_array) {
        let parts: Option<Vec<String>> = and
            .iter()
            .map(|q| q.as_object().and_then(predicate))
            .collect();
        return parts.map(|parts| format!("({})", parts.join(" && ")));
    }
    if let Some(or) = p.get("or").and_then(Value::as_array) {
        let parts: Option<Vec<String>> = or
            .iter()
            .map(|q| q.as_object().and_then(predicate))
            .collect();
        return parts.map(|parts| format!("({})", parts.join(" || ")));
    }
    if let Some(not) = p.get("not").and_then(Value::as_object) {
        return predicate(not).map(|inner| format!("!{inner}"));
    }
    let field = p.get("field")?.as_str()?;
    if p.contains_key("timeUnit") {
        return None;
    }
    let col = format!("col({field:?})");
    let mut parts = Vec::new();
    for (key, op) in [
        ("equal", "=="),
        ("lt", "<"),
        ("lte", "<="),
        ("gt", ">"),
        ("gte", ">="),
    ] {
        if let Some(v) = p.get(key) {
            parts.push(format!("{col} {op} {}", literal(v)?));
        }
    }
    if let Some(range) = p.get("range").and_then(Value::as_array) {
        let (Some(lo), Some(hi)) = (range.first(), range.get(1)) else {
            return None;
        };
        parts.push(format!(
            "{col} >= {} && {col} <= {}",
            literal(lo)?,
            literal(hi)?
        ));
    }
    if let Some(one_of) = p.get("oneOf").and_then(Value::as_array) {
        let options: Option<Vec<String>> = one_of
            .iter()
            .map(|v| literal(v).map(|v| format!("{col} == {v}")))
            .collect();
        parts.push(format!("({})", options?.join(" || ")));
    }
    if p.get("valid") == Some(&Value::Bool(true)) {
        parts.push(format!("!is_null({col})"));
    }
    (!parts.is_empty()).then(|| format!("({})", parts.join(" && ")))
}

const AGGREGATES: &[&str] = &["count", "sum", "mean", "median", "min", "max"];

/// Vega-Lite's transform operators that SONARITHM does not import.
const TRANSFORMS: &[&str] = &[
    "calculate",
    "fold",
    "pivot",
    "window",
    "joinaggregate",
    "timeUnit",
    "lookup",
    "flatten",
    "density",
    "loess",
    "regression",
    "quantile",
    "impute",
    "stack",
    "sample",
    "extent",
];

/// A Vega-Lite transform as a SONARITHM transform, or `None` after reporting it.
fn transform_of(t: &Value, at: &str, report: &mut ImportReport) -> Option<Value> {
    if let Some(filter) = t.get("filter") {
        if let Some(expr) = filter.as_object().and_then(predicate) {
            return Some(json!({ "op": "filter", "expr": expr }));
        }
        report.unmapped(
            &format!("{at}/filter"),
            "transform.filter",
            "only field predicates are imported; an expression is JavaScript, which SONARITHM does not run; write the filter in SONARITHM's expression language",
        );
        return None;
    }
    if let Some(ops) = t.get("aggregate").and_then(Value::as_array) {
        let groupby: Vec<Value> = t
            .get("groupby")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mapped: Option<Vec<Value>> = ops
            .iter()
            .map(|op| {
                let name = op.get("op")?.as_str()?;
                AGGREGATES.contains(&name).then_some(())?;
                let output = op.get("as")?.as_str()?;
                let field = op
                    .get("field")
                    .and_then(Value::as_str)
                    .or_else(|| groupby.first().and_then(Value::as_str))?;
                Some(json!({ "op": name, "field": field, "as": output }))
            })
            .collect();
        if let Some(ops) = mapped {
            return Some(json!({ "op": "aggregate", "groupby": groupby, "ops": ops }));
        }
        report.unmapped(
            &format!("{at}/aggregate"),
            "transform.aggregate",
            "only count, sum, mean, median, min and max are imported",
        );
        return None;
    }
    if let (Some(bin), Some(field)) = (t.get("bin"), t.get("field").and_then(Value::as_str)) {
        let output = match t.get("as") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(a)) => a
                .first()
                .and_then(Value::as_str)
                .unwrap_or(field)
                .to_owned(),
            _ => format!("bin_{field}"),
        };
        let count = bin.get("maxbins").and_then(Value::as_u64).unwrap_or(10);
        report.approximated(
            &format!("{at}/bin"),
            "transform.bin",
            format!("Vega-Lite picks nice bin edges; SONARITHM uses {count} equal-width bins"),
        );
        return Some(
            json!({ "op": "bin", "field": field, "method": "equal-width", "count": count, "as": output }),
        );
    }
    // A transform is named by its operator key (`calculate`, `fold`, …), not its options.
    let name = TRANSFORMS
        .iter()
        .find(|key| t.get(**key).is_some())
        .copied()
        .unwrap_or("unknown");
    report.unmapped(
        at,
        &format!("transform.{name}"),
        "this transform has no SONARITHM equivalent; later fields it creates will be missing",
    );
    None
}

/// One encoding channel, as read.
#[derive(Clone, Debug)]
struct Channel {
    name: String,
    field: String,
    vtype: Option<String>,
    title: Option<String>,
    aggregate: Option<String>,
    bin: Option<u64>,
    at: String,
}

impl Channel {
    fn numeric(&self) -> bool {
        matches!(self.vtype.as_deref(), Some("quantitative" | "temporal")) || self.bin.is_some()
    }
}

/// Reads the mapped channels; reports the rest.
fn channels(view: &View, report: &mut ImportReport) -> BTreeMap<String, Channel> {
    let mut out = BTreeMap::new();
    for (name, def) in &view.encoding {
        let at = view.encoding_at[name]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if !MAPPED_CHANNELS.contains(&name.as_str()) {
            report.unmapped(
                &at,
                &format!("encoding.{name}"),
                "SONARITHM's default design maps x, y and colour; map this field to another channel by hand",
            );
            continue;
        }
        if name == "xOffset" {
            continue;
        }
        let field = def.get("field").and_then(Value::as_str);
        let aggregate = def
            .get("aggregate")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let Some(field) = field.or(aggregate.as_deref().filter(|a| *a == "count").map(|_| ""))
        else {
            report.unmapped(
                &at,
                &format!("encoding.{name}"),
                "a constant `value` or `datum` has no data to map",
            );
            continue;
        };
        if field.contains('.') || field.contains('[') {
            report.unmapped(
                &at,
                &format!("encoding.{name}"),
                "nested field paths are not read",
            );
            continue;
        }
        if def.get("timeUnit").is_some() {
            report.approximated(
                &format!("{at}/timeUnit"),
                &format!("encoding.{name}.timeUnit"),
                "the times play as they are, not grouped by the time unit",
            );
        }
        if def.get("scale").and_then(|s| s.get("type")).is_some() {
            report.unmapped(
                &format!("{at}/scale/type"),
                &format!("encoding.{name}.scale.type"),
                "SONARITHM's pitch scale is log-frequency over the data range (SDR-09); set `scale` on the channel to change it",
            );
        }
        let bin = match def.get("bin") {
            Some(Value::Bool(true)) => Some(10),
            Some(Value::Object(b)) => Some(b.get("maxbins").and_then(Value::as_u64).unwrap_or(10)),
            _ => None,
        };
        let channel = Channel {
            name: name.clone(),
            field: field.to_owned(),
            vtype: def.get("type").and_then(Value::as_str).map(str::to_owned),
            title: text_of(def.get("title"))
                .or_else(|| text_of(def.get("axis").and_then(|a| a.get("title")))),
            aggregate,
            bin,
            at,
        };
        out.insert(name.clone(), channel);
    }
    out
}

/// Declares a channel's field in `data.fields`.
fn declare(draft: &mut Draft, c: &Channel) {
    if let Some(vtype) = c
        .vtype
        .as_deref()
        .filter(|t| ["quantitative", "temporal", "nominal", "ordinal"].contains(t))
        && !c.field.is_empty()
        && c.aggregate.is_none()
        && c.bin.is_none()
    {
        draft.field(&c.field, vtype, c.title.as_deref());
    }
}

/// Applies a channel's `bin` and `aggregate`, returning the field that holds its values.
fn summarize(
    value: &Channel,
    groupby: &[String],
    draft: &mut Draft,
    report: &mut ImportReport,
) -> String {
    let Some(op) = value.aggregate.as_deref() else {
        return value.field.clone();
    };
    if !AGGREGATES.contains(&op) {
        report.unmapped(
            &format!("{}/aggregate", value.at),
            &format!("encoding.{}.aggregate", value.name),
            "only count, sum, mean, median, min and max are imported; the values play unaggregated",
        );
        return value.field.clone();
    }
    let input = if value.field.is_empty() {
        groupby.first().cloned().unwrap_or_default()
    } else {
        value.field.clone()
    };
    let output = if op == "count" {
        "count".to_owned()
    } else {
        format!("{op}_{}", value.field)
    };
    draft.transforms.push(json!({
        "op": "aggregate", "groupby": groupby,
        "ops": [{ "op": op, "field": input, "as": output }],
    }));
    if let Some(title) = &value.title {
        draft.field(&output, "quantitative", Some(title));
    }
    output
}

/// Bins a channel with `bin`, returning the bin field.
fn bin(c: &Channel, draft: &mut Draft, report: &mut ImportReport) -> String {
    let Some(count) = c.bin else {
        return c.field.clone();
    };
    let output = format!("bin_{}", c.field);
    report.approximated(
        &format!("{}/bin", c.at),
        &format!("encoding.{}.bin", c.name),
        format!("Vega-Lite picks nice bin edges; SONARITHM uses {count} equal-width bins"),
    );
    draft.transforms.push(json!({
        "op": "bin", "field": c.field, "method": "equal-width", "count": count, "as": output,
    }));
    output
}

/// Picks the template and binds it.
fn design(
    mark: &str,
    view: &View,
    draft: &mut Draft,
    report: &mut ImportReport,
) -> Result<(), Box<Diagnostic>> {
    let found = channels(view, report);
    let get = |name: &str| found.get(name).cloned();
    let series = get("color")
        .or_else(|| get("fill"))
        .or_else(|| get("stroke"))
        .or_else(|| get("detail"));
    for c in ["x", "y", "theta"]
        .iter()
        .filter_map(|n| get(n))
        .chain(series.clone())
    {
        declare(draft, &c);
    }
    let need = |c: Option<Channel>, what: &str| {
        c.ok_or_else(|| {
            unreadable(
                IMPORTER,
                format!("a `{mark}` mark needs a `{what}` channel"),
            )
        })
    };
    match mark {
        "arc" => {
            let value = need(get("theta"), "theta")?;
            let slice = need(series, "color")?;
            let field = summarize(&value, std::slice::from_ref(&slice.field), draft, report);
            draft.template("chart-pie", &[("slice", &slice.field), ("value", &field)]);
        }
        "rect" => {
            let (x, y) = (need(get("x"), "x")?, need(get("y"), "y")?);
            let value = need(series, "color")?;
            let field = summarize(&value, &[y.field.clone(), x.field.clone()], draft, report);
            draft.template(
                "heatmap-rowscan",
                &[("row", &y.field), ("column", &x.field), ("value", &field)],
            );
        }
        "boxplot" => {
            let (x, y) = (need(get("x"), "x")?, need(get("y"), "y")?);
            let (group, value) = if x.numeric() && !y.numeric() {
                (y, x)
            } else {
                (x, y)
            };
            draft.template(
                "box-five-number",
                &[("group", &group.field), ("value", &value.field)],
            );
        }
        "bar" | "line" | "trail" | "area" | "point" | "circle" | "square" | "tick" => {
            xy(
                mark,
                view,
                get("x"),
                get("y"),
                series.as_ref(),
                draft,
                report,
            )?;
        }
        other => {
            report.unmapped(
                &view.mark_at,
                "mark",
                format!("the `{other}` mark has no chart-type template"),
            );
            return Err(unreadable(
                IMPORTER,
                format!(
                    "the `{other}` mark has no chart-type template; write the spec's tracks by hand"
                ),
            ));
        }
    }
    Ok(())
}

/// Marks on x and y: bars, lines, areas and points.
fn xy(
    mark: &str,
    view: &View,
    x: Option<Channel>,
    y: Option<Channel>,
    series: Option<&Channel>,
    draft: &mut Draft,
    report: &mut ImportReport,
) -> Result<(), Box<Diagnostic>> {
    let (Some(x), Some(y)) = (x, y) else {
        return Err(unreadable(
            IMPORTER,
            format!("a `{mark}` mark needs `x` and `y` channels"),
        ));
    };
    // Horizontal bars and lines: the numeric value on x, the categories on y.
    let horizontal =
        (x.aggregate.is_some() || x.numeric()) && !y.numeric() && y.aggregate.is_none();
    let (place, value) = if horizontal { (y, x) } else { (x, y) };
    let place_field = bin(&place, draft, report);
    let mut groupby = vec![place_field.clone()];
    if let Some(s) = series {
        groupby.push(s.field.clone());
    }
    let value_field = summarize(&value, &groupby, draft, report);
    let series_field = series.map(|s| s.field.clone());
    let mut bind: Vec<(&str, &str)> = Vec::new();
    let id = match mark {
        "bar" => {
            let grouped =
                view.encoding.contains_key("xOffset") || view.encoding.contains_key("yOffset");
            let unstacked = matches!(
                value_def(view, &value.name).and_then(|d| d.get("stack")),
                Some(Value::Null | Value::Bool(false))
            );
            bind.push(("category", &place_field));
            bind.push(("value", &value_field));
            if series.is_some() && !grouped && !unstacked {
                "chart-stacked-bar"
            } else {
                "chart-bar"
            }
        }
        "point" | "circle" | "square" | "tick" if place.numeric() && value.numeric() => {
            bind.push(("x", &place_field));
            bind.push(("y", &value_field));
            "scatter-xy"
        }
        _ => {
            bind.push(("x", &place_field));
            bind.push(("y", &value_field));
            "chart-line"
        }
    };
    if let Some(field) = &series_field {
        bind.push(("series", field));
    }
    draft.template(id, &bind);
    Ok(())
}

fn value_def<'a>(view: &'a View, channel: &str) -> Option<&'a Value> {
    view.encoding.get(channel)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    fn run(spec: &Value) -> Import {
        import_value(spec, &Budgets::DEFAULT).unwrap()
    }

    #[test]
    fn a_bar_chart_imports_to_the_bar_template() {
        let out = run(&json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
            "title": "Sales",
            "data": { "values": [{ "q": "Q1", "v": 3 }, { "q": "Q2", "v": 5 }] },
            "mark": "bar",
            "encoding": {
                "x": { "field": "q", "type": "nominal", "title": "Quarter" },
                "y": { "field": "v", "type": "quantitative" },
                "tooltip": { "field": "v" }
            }
        }));
        assert_eq!(
            out.spec["template"],
            json!({ "id": "chart-bar", "bind": { "category": "q", "value": "v" } })
        );
        assert_eq!(out.spec["name"], "Sales");
        assert_eq!(
            out.spec["data"]["fields"]["q"],
            json!({ "type": "nominal", "label": "Quarter" })
        );
        assert_eq!(out.report.entries().len(), 1, "the tooltip is reported");
        assert_eq!(out.report.entries()[0].pointer, "/encoding/tooltip");
    }

    #[test]
    fn coloured_bars_stack_unless_offset_and_histograms_bin_and_count() {
        let base = json!({
            "data": { "values": [{ "a": "x", "b": "p", "v": 1 }] },
            "mark": "bar",
            "encoding": {
                "x": { "field": "a", "type": "nominal" },
                "y": { "field": "v", "type": "quantitative", "aggregate": "sum" },
                "color": { "field": "b", "type": "nominal" }
            }
        });
        let stacked = run(&base);
        assert_eq!(stacked.spec["template"]["id"], "chart-stacked-bar");
        assert_eq!(stacked.spec["template"]["bind"]["value"], "sum_v");
        assert_eq!(stacked.spec["transform"][0]["groupby"], json!(["a", "b"]));
        let mut grouped = base.clone();
        grouped["encoding"]["xOffset"] = json!({ "field": "b" });
        assert_eq!(run(&grouped).spec["template"]["id"], "chart-bar");
        let histogram = run(&json!({
            "data": { "url": "data/movies.json" },
            "mark": "bar",
            "encoding": {
                "x": { "field": "rating", "bin": true },
                "y": { "aggregate": "count" }
            }
        }));
        assert_eq!(
            histogram.spec["data"]["source"],
            json!({ "kind": "file", "path": "data/movies.json" })
        );
        assert_eq!(histogram.spec["transform"][0]["op"], "bin");
        assert_eq!(
            histogram.spec["transform"][1]["ops"][0],
            json!({ "op": "count", "field": "bin_rating", "as": "count" })
        );
        assert_eq!(
            histogram.spec["template"]["bind"],
            json!({ "category": "bin_rating", "value": "count" })
        );
        assert_eq!(
            histogram.report.entries()[0].outcome,
            super::super::Outcome::Approximated
        );
    }

    #[test]
    fn network_data_is_reported_and_never_read() {
        let err = import_value(
            &json!({ "data": { "url": "https://example.org/d.csv" }, "mark": "line" }),
            &Budgets::DEFAULT,
        )
        .unwrap_err();
        assert_eq!(err.code.as_str(), "SNR-ENG-384");
    }

    #[test]
    fn field_predicates_become_filters_and_expressions_are_reported() {
        let out = run(&json!({
            "data": { "values": [{ "t": 1, "v": 2, "s": "a" }] },
            "transform": [
                { "filter": { "field": "v", "range": [0, 5] } },
                { "filter": { "and": [{ "field": "s", "oneOf": ["a", "b"] }, { "field": "t", "gt": 0 }] } },
                { "filter": "datum.v > 1" },
                { "calculate": "datum.v * 2", "as": "w" }
            ],
            "mark": "line",
            "encoding": { "x": { "field": "t", "type": "quantitative" }, "y": { "field": "v", "type": "quantitative" } }
        }));
        assert_eq!(
            out.spec["transform"][0]["expr"],
            r#"(col("v") >= 0 && col("v") <= 5)"#
        );
        assert_eq!(
            out.spec["transform"][1]["expr"],
            r#"(((col("s") == "a" || col("s") == "b")) && (col("t") > 0))"#
        );
        let unmapped: Vec<&str> = out
            .report
            .entries()
            .iter()
            .map(|e| e.construct.as_str())
            .collect();
        assert_eq!(unmapped, ["transform.filter", "transform.calculate"]);
        assert_eq!(out.spec["template"]["id"], "chart-line");
    }

    #[test]
    fn layers_import_the_first_and_report_the_rest() {
        let out = run(&json!({
            "data": { "values": [{ "x": 1, "y": 2 }] },
            "encoding": { "x": { "field": "x", "type": "quantitative" } },
            "layer": [
                { "mark": "point", "encoding": { "y": { "field": "y", "type": "quantitative" } } },
                { "mark": "rule", "encoding": { "y": { "datum": 1 } } }
            ]
        }));
        assert_eq!(
            out.spec["template"],
            json!({ "id": "scatter-xy", "bind": { "x": "x", "y": "y" } })
        );
        assert_eq!(out.report.entries()[0].pointer, "/layer/1");
    }
}
