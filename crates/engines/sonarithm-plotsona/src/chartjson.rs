// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chart JSON: tier 1, exact (10 §3, notes/06 §6).
//!
//! Vega-Lite, Vega, Plotly and Highcharts charts carry the data they draw. This module reads
//! it **statically**: from a JSON file, or from the `<script>` blocks of a local HTML page
//! ([`html`]). No script runs and nothing is fetched (OD-25, ADR-0012). Data that lives
//! elsewhere (a Vega `url`) or that only a script would produce (a function call, a signal)
//! is reported, never resolved: `SNR-ENG-346` and `SNR-ENG-347`.
//!
//! | Library | What is read |
//! |---|---|
//! | Vega-Lite | Inline `data.values` and named `datasets`, in unit, layered, faceted and concatenated views; `x`/`y` (or `theta` for arcs) and `color`/`detail` series; `fold`; encoding aggregates (`derived`); the default sort of discrete axes |
//! | Vega | Inline `data` (following `source`), marks with `from.data` or a facet's `groupby`, `x`/`y` fields of `encode`, `pie` fields of arcs, scale types for dates and categories |
//! | Plotly | Figure `data` traces (never `layout.template`): `x`/`y`, `x0`/`dx`, typed arrays (`bdata`), `labels`/`values`, `z` grids; `layout` titles, axis types and `barmode` |
//! | Highcharts | `series[].data` in its three documented forms (values, `[x, y]` pairs, point objects), `xAxis.categories`, `pointStart`/`pointInterval`, `datetime` axes, `stacking` |
//!
//! Exact values keep the class `exact`; values PLOTSONA aggregates (a Vega-Lite `sum`) are
//! `derived` (03 §4).

mod highcharts;
pub(crate) mod html;
pub(crate) mod literal;
mod plotly;
mod vega;
mod vegalite;

use serde_json::Value;
use sonarithm_core::Diagnostic;
use sonarithm_core::diagnostic::Severity;

use crate::budget::{Budget, Budgets, Work, len64, over};
use crate::calibrate::ticks::parse_date;
use crate::detect::Container;
use crate::model::{ChartData, XValue};

/// Shared state of one chart-JSON extraction.
pub(crate) struct Cx<'a> {
    pub(crate) budgets: &'a Budgets,
    pub(crate) work: &'a mut Work,
    pub(crate) diagnostics: &'a mut Vec<Diagnostic>,
    points: u64,
}

impl<'a> Cx<'a> {
    pub(crate) fn new(
        budgets: &'a Budgets,
        work: &'a mut Work,
        diagnostics: &'a mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            budgets,
            work,
            diagnostics,
            points: 0,
        }
    }

    /// Counts `n` more points against the point budget, and their work.
    pub(crate) fn count(&mut self, n: usize) -> Result<(), Box<Diagnostic>> {
        self.points = self.points.saturating_add(len64(n));
        self.work.spend(len64(n))?;
        if self.points > self.budgets.max_points {
            return Err(over(Budget::Points, self.points, self.budgets.max_points));
        }
        Ok(())
    }

    /// Reports a warning, once per message.
    pub(crate) fn warn(&mut self, code: &sonarithm_core::codes::CodeInfo, message: String) {
        let diagnostic = Diagnostic::new(code, message).with_severity(Severity::Warning);
        if !self.diagnostics.contains(&diagnostic) {
            self.diagnostics.push(diagnostic);
        }
    }
}

/// Extracts the chart of a chart-JSON document of the given library.
pub(crate) fn extract(
    container: Container,
    json: &Value,
    cx: &mut Cx<'_>,
) -> Result<ChartData, Box<Diagnostic>> {
    let chart = match container {
        Container::VegaLite => vegalite::extract(json, cx)?,
        Container::Vega => vega::extract(json, cx)?,
        Container::Plotly => plotly::extract(json, cx)?,
        Container::Highcharts => highcharts::extract(json, cx)?,
        _ => ChartData::default(),
    };
    // With no data left, remote or script-built data is the reason: report it as the
    // error rather than as a warning beside "no chart".
    if chart.points() == 0
        && let Some(i) = cx
            .diagnostics
            .iter()
            .position(|d| matches!(d.code.as_str(), "SNR-ENG-346" | "SNR-ENG-347"))
    {
        let mut reason = cx.diagnostics.remove(i);
        reason.severity = Severity::Error;
        return Err(Box::new(reason));
    }
    Ok(chart)
}

/// How the x values of a series read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XKind {
    Number,
    Time,
    Category,
}

impl XKind {
    /// The kind every present value fits: numbers, date strings, or else labels.
    pub(crate) fn infer<'v>(values: impl Iterator<Item = &'v Value>) -> Self {
        let present: Vec<&Value> = values.filter(|v| !v.is_null()).collect();
        if present.iter().all(|v| v.is_number()) {
            Self::Number
        } else if present
            .iter()
            .all(|v| v.as_str().is_some_and(|s| date_text(s).is_some()))
        {
            Self::Time
        } else {
            Self::Category
        }
    }

    /// The x position of `value` read as this kind; a value that does not read becomes a
    /// label.
    pub(crate) fn x(self, value: &Value) -> XValue {
        match self {
            Self::Number => {
                number(value).map_or_else(|| XValue::Category(label(value)), XValue::Number)
            }
            Self::Time => time(value).map_or_else(|| XValue::Category(label(value)), XValue::Time),
            Self::Category => XValue::Category(label(value)),
        }
    }
}

/// A number, or a string that reads as one.
pub(crate) fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        Value::Bool(b) => Some(f64::from(u8::from(*b))),
        _ => None,
    }
    .filter(|v| v.is_finite())
}

/// A time in milliseconds: a number of milliseconds, or a date string.
pub(crate) fn time(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => {
            let ms = n.as_f64()?;
            #[expect(clippy::cast_possible_truncation, reason = "finite times in ms")]
            ms.is_finite().then(|| ms.round() as i64)
        }
        Value::String(s) => date_text(s),
        _ => None,
    }
}

/// A date string: ISO dates and times (also with a space, as Plotly writes them) and
/// month names.
fn date_text(text: &str) -> Option<i64> {
    let text = text.trim();
    let trimmed = text
        .strip_suffix('Z')
        .unwrap_or(text)
        .split('.')
        .next()
        .unwrap_or(text);
    // A number written as text (a year such as "2020" included) stays a label.
    parse_date(trimmed).filter(|_| text.parse::<f64>().is_err())
}

/// A label: strings as they are, numbers and booleans written out, null as empty.
pub(crate) fn label(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// A title: a string, or an object's `text`, or several lines joined.
pub(crate) fn title(value: Option<&Value>) -> Option<String> {
    let text = match value? {
        Value::String(s) => s.clone(),
        Value::Array(lines) => lines.iter().map(label).collect::<Vec<_>>().join(" "),
        Value::Object(o) => return title(o.get("text")),
        _ => return None,
    };
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// A field of a data row, following a dotted path (`a.b`), as Vega's field accessors do.
pub(crate) fn field<'v>(row: &'v Value, path: &str) -> Option<&'v Value> {
    if let Some(v) = row.get(path) {
        return Some(v);
    }
    path.split('.').try_fold(row, |v, key| v.get(key))
}

#[cfg(test)]
mod tests;
