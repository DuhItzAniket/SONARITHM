// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chart JSON inside a local HTML page, read from its `<script>` blocks as text (OD-25:
//! static parsing only, no script runs, nothing is fetched).
//!
//! A block of a JSON type (`application/json`, `application/vnd.vegalite+json`, …) is read
//! as JSON. In a JavaScript block PLOTSONA looks for the calls chart libraries document,
//! `Plotly.newPlot(el, data, layout)` (also `react` and `plot`), `Highcharts.chart(el,
//! options)` (also `stockChart` and `new Highcharts.Chart(options)`) and `vegaEmbed(el,
//! spec)`, and then for any object literal assigned with `=`. Their arguments are read
//! as literals ([`literal`](super::literal)); a variable name is followed to the literal
//! assigned to it. Options that need a script (a function, a call, a variable that is
//! not a literal) are reported (`SNR-ENG-347`); a `<script src>` is never loaded.

use serde_json::{Map, Value};
use sonarithm_core::Diagnostic;

use super::Cx;
use super::literal::{self, LiteralError};
use crate::budget::len64;
use crate::codes;
use crate::detect::{Container, chart_json};

/// The calls whose arguments hold a chart, and which argument does.
const CALLS: [(&str, Call); 7] = [
    ("Plotly.newPlot(", Call::Plotly),
    ("Plotly.react(", Call::Plotly),
    ("Plotly.plot(", Call::Plotly),
    ("Highcharts.chart(", Call::Highcharts),
    ("Highcharts.stockChart(", Call::Highcharts),
    ("new Highcharts.Chart(", Call::Highcharts),
    ("vegaEmbed(", Call::Second),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Call {
    /// `(el, data, layout)`.
    Plotly,
    /// `(el, options)` or `(options)`.
    Highcharts,
    /// `(el, spec)`.
    Second,
}

/// The first chart in the page's scripts, as a sniffed container and its JSON.
pub(crate) fn find_chart(
    bytes: &[u8],
    cx: &mut Cx<'_>,
) -> Result<(Container, Value), Box<Diagnostic>> {
    cx.work.spend(len64(bytes.len()))?;
    let text = String::from_utf8_lossy(bytes);
    let scripts = scripts(&text);
    let depth = usize::try_from(cx.budgets.max_depth).unwrap_or(usize::MAX);
    let mut needs_script = false;
    // JSON blocks first: they are data by declaration.
    for script in scripts.iter().filter(|s| s.json) {
        if let Ok(value) = serde_json::from_str::<Value>(script.body)
            && let Some(container) = chart_json(&value)
        {
            cx.budgets.check_depth(json_depth(&value))?;
            return Ok((container, value));
        }
    }
    let code: Vec<&str> = scripts.iter().filter(|s| !s.json).map(|s| s.body).collect();
    for body in &code {
        for (marker, call) in CALLS {
            for (at, _) in body.match_indices(marker) {
                cx.work.spend(1)?;
                match call_chart(body, at + marker.len(), call, &code, depth) {
                    Ok(Some(found)) => return Ok(found),
                    Ok(None) | Err(LiteralError::Syntax(_)) => {}
                    Err(LiteralError::NeedsScript(_)) => needs_script = true,
                    Err(LiteralError::TooDeep) => {
                        return Err(Box::new(Diagnostic::new(
                            &codes::NESTING_TOO_DEEP,
                            "A chart literal in the page nests deeper than the limit.",
                        )));
                    }
                }
            }
        }
    }
    // Any object literal assigned in a script (`var spec = {…}`).
    for body in &code {
        for (at, _) in body.match_indices('=') {
            cx.work.spend(1)?;
            let rest = body.get(at + 1..).unwrap_or_default();
            if !rest.trim_start().starts_with('{')
                || body.get(at.saturating_sub(1)..at) == Some("=")
            {
                continue;
            }
            if let Ok((value, _)) = literal::parse(rest, depth)
                && let Some(container) = chart_json(&value)
            {
                return Ok((container, value));
            }
        }
    }
    let diagnostic = if needs_script {
        Diagnostic::new(
            &codes::SCRIPT_DATA,
            "The page builds its chart's data with a script, which PLOTSONA does not run; \
             export the chart's data or its JSON instead.",
        )
    } else {
        Diagnostic::new(
            &codes::NO_CHART,
            "The page's scripts hold no Vega, Vega-Lite, Plotly or Highcharts chart data.",
        )
    };
    Err(Box::new(diagnostic))
}

/// Reads the chart a library call at `start` (just after its `(`) is given.
fn call_chart(
    body: &str,
    start: usize,
    call: Call,
    scripts: &[&str],
    depth: usize,
) -> Result<Option<(Container, Value)>, LiteralError> {
    let mut args = Vec::new();
    let mut at = start;
    for _ in 0..3 {
        let rest = body.get(at..).unwrap_or_default();
        let trimmed = rest.trim_start();
        at += rest.len() - trimmed.len();
        if trimmed.starts_with(')') {
            break;
        }
        let (value, used) = argument(trimmed, scripts, depth)?;
        args.push(value);
        at += used;
        let rest = body.get(at..).unwrap_or_default();
        let trimmed = rest.trim_start();
        at += rest.len() - trimmed.len();
        if trimmed.starts_with(',') {
            at += 1;
        } else {
            break;
        }
    }
    let value = match call {
        Call::Plotly => {
            let mut figure = Map::new();
            figure.insert(
                "data".to_owned(),
                args.get(1).cloned().unwrap_or(Value::Null),
            );
            figure.insert(
                "layout".to_owned(),
                args.get(2).cloned().unwrap_or(Value::Null),
            );
            Value::Object(figure)
        }
        Call::Highcharts => args
            .iter()
            .find(|a| a.is_object())
            .cloned()
            .unwrap_or(Value::Null),
        Call::Second => args.get(1).cloned().unwrap_or(Value::Null),
    };
    Ok(chart_json(&value).map(|container| (container, value)))
}

/// One call argument: a literal, or a name whose assigned literal is looked up. Element
/// selectors and other names become null.
fn argument(text: &str, scripts: &[&str], depth: usize) -> Result<(Value, usize), LiteralError> {
    let first = text.bytes().next().unwrap_or(b')');
    if matches!(first, b'{' | b'[' | b'"' | b'\'' | b'`') || first.is_ascii_digit() {
        return literal::parse(text, depth);
    }
    let name_len = text
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'.'))
        .count();
    let name = text.get(..name_len).unwrap_or_default();
    let after = text.get(name_len..).unwrap_or_default().trim_start();
    if name.is_empty() || !(after.starts_with(',') || after.starts_with(')')) {
        return Err(LiteralError::NeedsScript(0));
    }
    // `document.getElementById("x")`-style element arguments are calls: skip them as null
    // only when they are plain names; a call is not data.
    Ok((
        assigned(name, scripts, depth).unwrap_or(Value::Null),
        name_len,
    ))
}

/// The literal assigned to `name` in any script (`var name = …`, `name = …`).
fn assigned(name: &str, scripts: &[&str], depth: usize) -> Option<Value> {
    for body in scripts {
        for (at, _) in body.match_indices(name) {
            let before = body.get(..at).and_then(|b| b.chars().last());
            if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '.')
            {
                continue;
            }
            let rest = body.get(at + name.len()..)?.trim_start();
            let Some(value_text) = rest.strip_prefix('=') else {
                continue;
            };
            if value_text.starts_with('=') {
                continue;
            }
            if let Ok((value, _)) = literal::parse(value_text, depth) {
                return Some(value);
            }
        }
    }
    None
}

struct Script<'a> {
    json: bool,
    body: &'a str,
}

/// The inline `<script>` blocks of a page, in order. Blocks with a `src` are skipped:
/// they are never loaded.
fn scripts(text: &str) -> Vec<Script<'_>> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(open) = lower.get(at..).and_then(|r| r.find("<script")) {
        let open = at + open;
        let Some(tag_end) = lower
            .get(open..)
            .and_then(|r| r.find('>'))
            .map(|e| open + e)
        else {
            break;
        };
        let tag = lower.get(open..tag_end).unwrap_or_default();
        let Some(close) = lower
            .get(tag_end..)
            .and_then(|r| r.find("</script"))
            .map(|c| tag_end + c)
        else {
            break;
        };
        let body = text.get(tag_end + 1..close).unwrap_or_default();
        at = close + "</script".len();
        if tag.contains(" src=") || tag.contains(" src =") {
            continue;
        }
        let json = tag.contains("json");
        out.push(Script { json, body });
    }
    out
}

fn json_depth(value: &Value) -> usize {
    let mut deepest = 0;
    let mut stack = vec![(value, 0_usize)];
    while let Some((v, d)) = stack.pop() {
        deepest = deepest.max(d);
        match v {
            Value::Array(items) => stack.extend(items.iter().map(|i| (i, d + 1))),
            Value::Object(m) => stack.extend(m.values().map(|i| (i, d + 1))),
            _ => {}
        }
    }
    deepest
}
