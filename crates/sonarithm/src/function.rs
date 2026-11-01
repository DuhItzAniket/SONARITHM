// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Functions as data (C03-T07): sample an expression with CURVASONA, list its critical
//! points as text (the text equivalent of the function cues, A11Y-24, A11Y-28), and get a
//! spec that plays it as a glide with those cues.
//!
//! ```
//! use sonarithm::{FunctionSpec, Sonarithm, read_function};
//!
//! let function = read_function(&FunctionSpec::new("x^2 - 1", [-2.0, 2.0]))?;
//! let points = function.critical_points();
//! assert_eq!(points[0].text, "zero at x = -1");
//! assert_eq!(points[1].text, "local minimum at x = 0, y = -1");
//! let compiled = Sonarithm::new().compile(&function.spec())?;
//! # Ok::<(), sonarithm::Error>(())
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Serialize;
use serde_json::{Value, json};
use sonarithm_core::{Diagnostic, Diagnostics, SpecPointer, TableId};
use sonarithm_curvasona::{
    Budgets, Expr, FUNCTION_TABLE, Feature, FeatureKind, FunctionSpec, Limits, Sampled,
    offscreen_features, sample_function,
};
use sonarithm_data::{BundleId, DataBundle};
use sonarithm_i18n::{Args, I18nError, Localizer, Number, NumberOptions};

use crate::error::Error;
use crate::spec::Spec;

/// How long [`Function::spec`] plays the whole domain, in seconds: the length of the
/// function example in 04 §6.3. It is not a research default: no rule covers how long a
/// function's sweep should last (TODO(verify), HP-21).
pub const FUNCTION_DURATION_S: f64 = 8.0;

/// A sampled function: its table, its critical points and how to play it.
#[derive(Clone, Debug)]
pub struct Function {
    source: FunctionSpec,
    sampled: Sampled,
    window: Option<[f64; 2]>,
}

/// One entry of a function's critical-point list: a feature with its text equivalent.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[non_exhaustive]
pub struct CriticalPoint {
    /// The feature's label, as its insight node and the function cues name it (07
    /// CURVASONA): `zero`, `local maximum`, `vertical asymptote`, `undefined` and so on.
    pub kind: &'static str,
    /// Where it is, or where its span starts. A horizontal asymptote is at the domain's end
    /// on its side.
    pub x: f64,
    /// Where its span ends, for spans only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
    /// Its value: `f(x)`, or the limit at a hole or a horizontal asymptote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// The table row its insight covers first: where its cue sounds and where the cursor
    /// goes to reach it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<u64>,
    /// The entry as text, such as `local maximum at x = 1.5708, y = 1`.
    pub text: String,
}

/// Samples `function` with CURVASONA's default budgets and finds its critical points.
///
/// # Errors
///
/// [`Error::Spec`] with CURVASONA's diagnostic when the expression or a variable cannot
/// be used (`SNR-ENG-400` to `403`) or the grid exceeds its budget (`SNR-ENG-404`); its
/// pointer names the key of a function source (`/data/source/expr`).
pub fn read_function(function: &FunctionSpec) -> Result<Function, Error> {
    read_function_with_ids(function, BundleId(0), FUNCTION_TABLE)
}

/// [`read_function`] into a bundle and table with these IDs, for hosts that hold several
/// datasets (the app core's `function_open`, C03-T08).
///
/// # Errors
///
/// As [`read_function`].
pub fn read_function_with_ids(
    function: &FunctionSpec,
    bundle: BundleId,
    table: TableId,
) -> Result<Function, Error> {
    let sampled = sample_function(function, &Budgets::DEFAULT, (bundle, table)).map_err(|err| {
        let at = SpecPointer::root().key("data").key("source").key(err.key);
        let mut one = Diagnostics::new();
        one.push(err.diagnostic.at(at));
        Error::Spec(one)
    })?;
    Ok(Function {
        source: function.clone(),
        sampled,
        window: None,
    })
}

impl Function {
    /// The function of a spec whose source is a `function`, with the y-window of its
    /// off-screen cues (the first track's pitch `scale.domain`, RFC-0007 Q5); `None` for
    /// any other spec, or one whose source cannot be read (its compile reports why).
    ///
    /// # Errors
    ///
    /// As [`read_function`], inside the `Some`.
    #[must_use]
    pub fn from_spec(spec: &Spec) -> Option<Result<Self, Error>> {
        let value: Value = serde_json::from_str(spec.text()).ok()?;
        let source = value.pointer("/data/source")?;
        if source.get("kind").and_then(Value::as_str) != Some("function") {
            return None;
        }
        let sonarithm_spec::Source::Function {
            expr,
            domain,
            samples,
            step,
            variables,
        } = serde_json::from_value(source.clone()).ok()?
        else {
            return None;
        };
        let mut function = FunctionSpec::new(expr, domain);
        function.samples = samples;
        function.step = step;
        function.variables = variables.unwrap_or_default();
        let window = sonarithm_curvasona::cues::window(&value);
        Some(read_function(&function).map(|read| read.with_window(window)))
    }

    /// The same function with a y-window: its critical-point list also gives the spans
    /// above or below it, as the off-screen cues do.
    #[must_use]
    pub fn with_window(mut self, window: Option<[f64; 2]>) -> Self {
        self.window = window.filter(|[low, high]| low < high);
        self
    }

    /// The expression in its canonical form (ordinary precedence, fewest parentheses).
    #[must_use]
    pub fn expr(&self) -> &str {
        self.sampled.expr.canonical()
    }

    /// The function source as given.
    #[must_use]
    pub fn source(&self) -> &FunctionSpec {
        &self.source
    }

    /// The table `x`, `y`, `dy` (provenance `synthetic`) in a bundle.
    #[must_use]
    pub fn bundle(&self) -> &DataBundle {
        &self.sampled.bundle
    }

    /// The bundle, taking it out of the function.
    #[must_use]
    pub fn into_bundle(self) -> DataBundle {
        self.sampled.bundle
    }

    /// Sampling's non-error diagnostics: a numerical derivative (`SNR-ENG-405`), undefined
    /// samples (`406`), a scan that stopped early (`407`).
    #[must_use]
    pub fn diagnostics(&self) -> Diagnostics {
        let mut all = Diagnostics::new();
        for diagnostic in &self.sampled.diagnostics {
            all.push(diagnostic.clone());
        }
        all
    }

    /// Every critical point and span in increasing `x`: zeros, extrema, inflections,
    /// asymptotes, holes, jumps, undefined spans and, with a y-window, the off-screen
    /// spans. Each is a feature the function cues mark (horizontal asymptotes, which have
    /// no cue, are listed too), so the list is their text equivalent (A11Y-24, A11Y-28).
    #[must_use]
    pub fn critical_points(&self) -> Vec<CriticalPoint> {
        Localizer::new(sonarithm_i18n::SOURCE_LOCALE)
            .and_then(|l| self.critical_points_with(&l))
            .unwrap_or_else(|_| self.critical_points_english())
    }

    /// [`Function::critical_points`] with their text in `locale` (or the pseudo-locale
    /// `en-XA`), from the catalogue (`function.ftl`).
    ///
    /// # Errors
    ///
    /// [`Error::Render`] with the catalogue's error when the locale has no catalogue.
    pub fn critical_points_in(&self, locale: &str) -> Result<Vec<CriticalPoint>, Error> {
        Localizer::new(locale)
            .and_then(|l| self.critical_points_with(&l))
            .map_err(|err| Error::Render(err.to_string()))
    }

    /// The features in increasing `x` with their first rows.
    fn sorted_features(&self) -> Vec<(Feature, Option<u64>)> {
        let mut features: Vec<Feature> = self.sampled.features.clone();
        if let Some(window) = self.window {
            features.extend(offscreen_features(
                &self.sampled.bundle,
                window,
                &Budgets::DEFAULT,
            ));
        }
        features.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.kind.cmp(&b.kind)));
        let rows =
            sonarithm_curvasona::feature_rows(&self.sampled.bundle, &features, &Budgets::DEFAULT);
        features.into_iter().zip(rows).collect()
    }

    fn critical_points_with(&self, l: &Localizer) -> Result<Vec<CriticalPoint>, I18nError> {
        self.sorted_features()
            .iter()
            .map(|(feature, row)| {
                Ok(CriticalPoint {
                    row: *row,
                    text: entry_text(l, feature)?,
                    ..critical_point(feature)
                })
            })
            .collect()
    }

    fn critical_points_english(&self) -> Vec<CriticalPoint> {
        self.sorted_features()
            .iter()
            .map(|(feature, row)| CriticalPoint {
                row: *row,
                ..critical_point(feature)
            })
            .collect()
    }

    /// One sentence: the expression, the domain and how many of each feature, such as
    /// "f(x) = x^2 - 1 from x = -2 to 2, 1000 samples: 2 zeros, 1 local minimum".
    #[must_use]
    pub fn summary(&self) -> String {
        Localizer::new(sonarithm_i18n::SOURCE_LOCALE)
            .and_then(|l| self.summary_with(&l))
            .unwrap_or_else(|_| self.summary_english())
    }

    /// [`Function::summary`] in `locale` (or the pseudo-locale `en-XA`).
    ///
    /// # Errors
    ///
    /// [`Error::Render`] with the catalogue's error when the locale has no catalogue.
    pub fn summary_in(&self, locale: &str) -> Result<String, Error> {
        Localizer::new(locale)
            .and_then(|l| self.summary_with(&l))
            .map_err(|err| Error::Render(err.to_string()))
    }

    fn summary_with(&self, l: &Localizer) -> Result<String, I18nError> {
        let [low, high] = self.source.domain;
        let grid = match (self.source.step, self.source.samples) {
            (Some(step), _) => l.format(
                "function-grid-step",
                &Args::new().number_with("step", text_number(step)),
            )?,
            (None, samples) => l.format(
                "function-grid-samples",
                &Args::new().number_with(
                    "count",
                    count_number(
                        samples.map_or(1000, |s| usize::try_from(s).unwrap_or(usize::MAX)),
                    ),
                ),
            )?,
        };
        let mut counts: Vec<(&'static str, usize)> = Vec::new();
        for (feature, _) in self.sorted_features() {
            let kind = feature.kind.label();
            match counts.iter_mut().find(|(k, _)| *k == kind) {
                Some((_, count)) => *count += 1,
                None => counts.push((kind, 1)),
            }
        }
        let args = Args::new()
            .text("expr", self.expr())
            .number_with("min", text_number(low))
            .number_with("max", text_number(high))
            .text("grid", grid);
        if counts.is_empty() {
            return l.format("function-summary-none", &args);
        }
        let parts = counts
            .iter()
            .map(|(kind, count)| {
                l.format(
                    &format!("function-count-{}", slug(kind)),
                    &Args::new().number_with("count", count_number(*count)),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let separator = l.format("function-list-separator", &Args::new())?;
        l.format(
            "function-summary",
            &args.text("counts", parts.join(&separator)),
        )
    }

    /// The summary without the catalogue, if it cannot be read.
    fn summary_english(&self) -> String {
        let [low, high] = self.source.domain;
        let grid = match (self.source.step, self.source.samples) {
            (Some(step), _) => format!("a step of {}", number(step)),
            (None, Some(samples)) => format!("{samples} samples"),
            (None, None) => "1000 samples".to_owned(),
        };
        let mut text = format!(
            "f(x) = {} from x = {} to {}, {grid}",
            self.expr(),
            number(low),
            number(high)
        );
        let points = self.critical_points_english();
        let mut counts: Vec<(&'static str, usize)> = Vec::new();
        for point in &points {
            match counts.iter_mut().find(|(kind, _)| *kind == point.kind) {
                Some((_, count)) => *count += 1,
                None => counts.push((point.kind, 1)),
            }
        }
        if counts.is_empty() {
            text.push_str(": no critical points");
        } else {
            text.push(':');
            for (i, (kind, count)) in counts.iter().enumerate() {
                let sep = if i == 0 { " " } else { ", " };
                let _ = write!(text, "{sep}{count} {}", plural(kind, *count));
            }
        }
        text
    }

    /// A spec that plays the function as one glide over [`FUNCTION_DURATION_S`] seconds,
    /// y as pitch with every other choice from the defaults, with its critical points as
    /// earcons (`analysis.detect: ["critical-points"]`, 07 CURVASONA): the shape of the
    /// example in 04 §6.3.
    #[must_use]
    pub fn spec(&self) -> Spec {
        self.spec_lasting(FUNCTION_DURATION_S)
    }

    /// [`Function::spec`] lasting `seconds`.
    #[must_use]
    pub fn spec_lasting(&self, seconds: f64) -> Spec {
        function_spec(&self.source, seconds)
    }
}

/// The spec of [`Function::spec`] for a function source that has not been read, lasting
/// `seconds`: its compile reports any problem with the expression, with a span in the
/// spec text.
#[must_use]
pub fn function_spec(function: &FunctionSpec, seconds: f64) -> Spec {
    let mut source = json!({
        "kind": "function",
        "expr": function.expr,
        "domain": function.domain,
    });
    if let Some(samples) = function.samples {
        source["samples"] = json!(samples);
    }
    if let Some(step) = function.step {
        source["step"] = json!(step);
    }
    if !function.variables.is_empty() {
        source["variables"] = json!(function.variables);
    }
    let value = json!({
        "specVersion": sonarithm_spec::SPEC_VERSION,
        "name": "function",
        "data": { "source": source },
        "substrate": { "time": { "pace": { "duration": { "s": seconds } } } },
        "tracks": [{ "id": "f", "mark": "glide", "channels": { "pitch": { "field": "y" } } }],
        "analysis": { "detect": ["critical-points"], "cue": "earcon" }
    });
    Spec::from_value(&value).with_name("function.spec.json")
}

/// An expression read without sampling it: its canonical form, or why it cannot be read
/// (C03-T08's `function_check`, for an editor's live diagnostics).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ExpressionCheck {
    /// The canonical form, when it reads.
    pub canonical: Option<String>,
    /// Why it does not (`SNR-ENG-400` to `403`).
    pub diagnostic: Option<Diagnostic>,
    /// Where reading stopped, as `(at, len)` in characters, when the error has a place.
    pub span: Option<(usize, usize)>,
}

/// Reads `expr` with the named constants `variables` under the function source's limits
/// (ADR-0028), without sampling it.
#[must_use]
pub fn check_expression(expr: &str, variables: &BTreeMap<String, f64>) -> ExpressionCheck {
    match Expr::parse(expr, variables, &Limits::DEFAULT) {
        Ok(parsed) => ExpressionCheck {
            canonical: Some(parsed.canonical().to_owned()),
            diagnostic: None,
            span: None,
        },
        Err(err) => ExpressionCheck {
            canonical: None,
            span: err.span(),
            diagnostic: Some(err.to_diagnostic()),
        },
    }
}

/// A catalogue message ID's form of a label: `local maximum` → `local-maximum`.
fn slug(label: &str) -> String {
    label.replace(' ', "-")
}

/// A coordinate for text: at most four fraction digits, no grouping, never `-0`, in the
/// locale's digits; in English the same text as [`number`].
fn text_number(value: f64) -> Number {
    let value = if value.abs() < 5e-5 { 0.0 } else { value };
    let mut options = NumberOptions::default().without_grouping();
    options.max_fraction = Some(4);
    Number::with_options(value, options)
}

/// A count for text, without grouping ("1000 samples", as the CLI prints).
fn count_number(count: usize) -> Number {
    let value = u32::try_from(count).map_or(f64::from(u32::MAX), f64::from);
    Number::with_options(value, NumberOptions::default().without_grouping())
}

/// An entry of the list from the catalogue (`function.ftl`).
fn entry_text(l: &Localizer, feature: &Feature) -> Result<String, I18nError> {
    let label = l.format(
        &format!("function-kind-{}", slug(feature.kind.label())),
        &Args::new(),
    )?;
    let args = Args::new().text("label", label.clone());
    let x = text_number(feature.x);
    match feature.kind {
        FeatureKind::HorizontalAsymptote { .. } => match feature.y {
            Some(y) => l.format(
                "function-asymptote-value",
                &args.number_with("y", text_number(y)),
            ),
            None => Ok(label),
        },
        FeatureKind::Zero => l.format("function-point", &args.number_with("x", x)),
        _ if !feature.is_point() => l.format(
            "function-span",
            &args
                .number_with("x", x)
                .number_with("end", text_number(feature.end)),
        ),
        _ => match feature.y {
            Some(y) => l.format(
                "function-point-value",
                &args.number_with("x", x).number_with("y", text_number(y)),
            ),
            None => l.format("function-point", &args.number_with("x", x)),
        },
    }
}

/// A feature as a list entry.
fn critical_point(feature: &Feature) -> CriticalPoint {
    let kind = feature.kind.label();
    let span = !feature.is_point();
    let text = match feature.kind {
        FeatureKind::HorizontalAsymptote { .. } => match feature.y {
            Some(y) => format!("{kind}, y = {}", number(y)),
            None => kind.to_owned(),
        },
        FeatureKind::Zero => format!("{kind} at x = {}", number(feature.x)),
        _ if span => format!(
            "{kind} from x = {} to {}",
            number(feature.x),
            number(feature.end)
        ),
        _ => match feature.y {
            Some(y) => format!("{kind} at x = {}, y = {}", number(feature.x), number(y)),
            None => format!("{kind} at x = {}", number(feature.x)),
        },
    };
    CriticalPoint {
        kind,
        x: feature.x,
        end: span.then_some(feature.end),
        y: feature.y,
        row: None,
        text,
    }
}

/// A label's plural, for counts.
fn plural(kind: &str, count: usize) -> String {
    if count == 1 {
        return match kind {
            "undefined" => "undefined span".to_owned(),
            "above the window" | "below the window" => format!("span {kind}"),
            other => other.to_owned(),
        };
    }
    match kind {
        "local maximum" => "local maxima".to_owned(),
        "local minimum" => "local minima".to_owned(),
        "undefined" => "undefined spans".to_owned(),
        "above the window" => "spans above the window".to_owned(),
        "below the window" => "spans below the window".to_owned(),
        "horizontal asymptote to the left" | "horizontal asymptote to the right" => {
            kind.replacen("asymptote", "asymptotes", 1)
        }
        other => format!("{other}s"),
    }
}

/// A number for text: at most four decimals, without trailing zeros, and never `-0`.
fn number(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let mut text = format!("{value:.4}");
    if text.contains('.') {
        let kept = text.trim_end_matches('0').trim_end_matches('.').len();
        text.truncate(kept);
    }
    if text == "-0" { "0".to_owned() } else { text }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_without_noise() {
        assert_eq!(number(1.234_567_89), "1.2346");
        assert_eq!(number(-0.000_01), "0");
        assert_eq!(number(2.0), "2");
        assert_eq!(number(-1.5), "-1.5");
    }

    #[test]
    fn plurals_read_as_english() {
        assert_eq!(plural("local maximum", 2), "local maxima");
        assert_eq!(plural("zero", 3), "zeros");
        assert_eq!(plural("inflection point", 1), "inflection point");
        assert_eq!(plural("below the window", 1), "span below the window");
        assert_eq!(plural("undefined", 1), "undefined span");
        assert_eq!(
            plural("horizontal asymptote to the left", 2),
            "horizontal asymptotes to the left"
        );
    }
}
