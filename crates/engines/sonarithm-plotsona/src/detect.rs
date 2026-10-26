// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Container sniffing (07 PLOTSONA `detect`, 10 §3).
//!
//! The file's bytes decide its container, never its extension: a renamed file still goes to
//! the right tier, and a file that only looks like a chart by name is reported as unknown.
//! Chart JSON is told apart by its keys (`$schema`, `mark`, `marks`, traces, `series`).

use serde::Serialize;
use serde_json::Value;
use sonarithm_core::{Diagnostic, ProvenanceClass};
use sonarithm_spec::data::Extractor;

use crate::budget::{Budgets, Work};
use crate::codes;
use crate::container::Package;

/// A chart file's container, as sniffed from its bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Container {
    /// A workbook (Office Open XML spreadsheet).
    Xlsx,
    /// A presentation (Office Open XML slides).
    Pptx,
    /// A word-processing document (Office Open XML).
    Docx,
    /// A Vega-Lite specification.
    VegaLite,
    /// A Vega specification.
    Vega,
    /// A Plotly figure (`data` traces and `layout`).
    Plotly,
    /// Highcharts options (`series` and chart options).
    Highcharts,
    /// A local HTML page; chart JSON may sit in its `<script>` blocks.
    Html,
    /// An SVG image.
    Svg,
    /// A PDF document.
    Pdf,
    /// A PNG image.
    Png,
    /// A JPEG image.
    Jpeg,
}

impl Container {
    /// The extraction tier that reads this container (10 §3).
    #[must_use]
    pub const fn tier(self) -> Tier {
        match self {
            Self::Xlsx
            | Self::Pptx
            | Self::Docx
            | Self::VegaLite
            | Self::Vega
            | Self::Plotly
            | Self::Highcharts
            | Self::Html => Tier::Exact,
            Self::Svg | Self::Pdf => Tier::Vector,
            Self::Png | Self::Jpeg => Tier::Raster,
        }
    }

    /// Whether the spec's `extractor` may read this container. `auto` reads them all;
    /// Highcharts has no extractor name of its own in spec v0.1, so only `auto` reads it.
    #[must_use]
    pub fn fits(self, extractor: Extractor) -> bool {
        match extractor {
            Extractor::Auto => true,
            Extractor::Office => matches!(self, Self::Xlsx | Self::Pptx | Self::Docx),
            Extractor::Vega => matches!(self, Self::Vega | Self::VegaLite | Self::Html),
            Extractor::Plotly => matches!(self, Self::Plotly | Self::Html),
            Extractor::Svg => self == Self::Svg,
            Extractor::PdfVector => self == Self::Pdf,
            Extractor::Raster => matches!(self, Self::Png | Self::Jpeg | Self::Pdf),
            Extractor::Ml => matches!(self, Self::Png | Self::Jpeg | Self::Pdf | Self::Svg),
            _ => false,
        }
    }

    /// The container's name in messages and reports.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xlsx => "XLSX workbook",
            Self::Pptx => "PPTX presentation",
            Self::Docx => "DOCX document",
            Self::VegaLite => "Vega-Lite specification",
            Self::Vega => "Vega specification",
            Self::Plotly => "Plotly figure",
            Self::Highcharts => "Highcharts options",
            Self::Html => "HTML page",
            Self::Svg => "SVG image",
            Self::Pdf => "PDF document",
            Self::Png => "PNG image",
            Self::Jpeg => "JPEG image",
        }
    }
}

/// An extraction tier (10 §3) and the provenance class its values get.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Tier {
    /// Tier 1: the data the chart was built from (Office caches, chart JSON): `exact`.
    Exact,
    /// Tier 2: vector geometry calibrated against the axes (SVG, PDF): `vector`.
    Vector,
    /// Tier 3: classical computer vision on pixels (C02): `cv_estimate`.
    Raster,
}

impl Tier {
    /// The provenance class of the tier's values (03 §4, 10 §3).
    #[must_use]
    pub const fn class(self) -> ProvenanceClass {
        match self {
            Self::Exact => ProvenanceClass::Exact,
            Self::Vector => ProvenanceClass::Vector,
            Self::Raster => ProvenanceClass::CvEstimate,
        }
    }
}

/// What sniffing found: the container, and for chart JSON the parsed document, so it is
/// parsed once.
#[derive(Debug)]
pub(crate) struct Sniffed {
    pub(crate) container: Container,
    pub(crate) json: Option<Value>,
}

/// The container of a chart file.
///
/// # Errors
///
/// `SNR-ENG-300` for input that is no chart container PLOTSONA knows, and the budget and
/// malformed-input codes when a zip package or JSON document cannot be opened within
/// `budgets`.
pub fn detect(bytes: &[u8], budgets: &Budgets) -> Result<Container, Box<Diagnostic>> {
    let mut work = Work::new(budgets.max_work);
    sniff(bytes, budgets, &mut work).map(|sniffed| sniffed.container)
}

pub(crate) fn sniff(
    bytes: &[u8],
    budgets: &Budgets,
    work: &mut Work,
) -> Result<Sniffed, Box<Diagnostic>> {
    budgets.check_input(crate::budget::len64(bytes.len()))?;
    let found = |container| {
        Ok(Sniffed {
            container,
            json: None,
        })
    };
    if bytes.starts_with(b"PK\x03\x04") {
        return found(office(bytes, budgets)?);
    }
    if bytes.starts_with(b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1") {
        return Err(unknown(
            "a legacy binary Office file (XLS, PPT or DOC) or a password-protected Office \
             file; save it as XLSX, PPTX or DOCX without a password",
        ));
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1A\n") {
        return found(Container::Png);
    }
    if bytes.starts_with(b"\xFF\xD8\xFF") {
        return found(Container::Jpeg);
    }
    // PDF readers accept the header anywhere in the first kilobyte.
    if find(bytes.get(..1024).unwrap_or(bytes), b"%PDF-").is_some() {
        return found(Container::Pdf);
    }
    let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let start = text
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(text.len());
    let text = text.get(start..).unwrap_or_default();
    match text.first() {
        Some(b'<') => markup(text),
        Some(b'{' | b'[') => {
            let json = parse_json(text, budgets, work)?;
            let container = chart_json(&json).ok_or_else(|| {
                unknown(
                    "JSON that is not a Vega, Vega-Lite, Plotly or Highcharts chart; load \
                     plain data as a file source instead",
                )
            })?;
            Ok(Sniffed {
                container,
                json: Some(json),
            })
        }
        _ => Err(unknown("not a chart format PLOTSONA reads")),
    }
}

/// An Office package by its main part.
fn office(bytes: &[u8], budgets: &Budgets) -> Result<Container, Box<Diagnostic>> {
    let package = Package::open(bytes, budgets)?;
    if package.contains("xl/workbook.xml") || package.contains("xl/workbook.bin") {
        Ok(Container::Xlsx)
    } else if package.contains("ppt/presentation.xml") {
        Ok(Container::Pptx)
    } else if package.contains("word/document.xml") {
        Ok(Container::Docx)
    } else {
        Err(unknown(
            "a zip archive that is not an XLSX, PPTX or DOCX package",
        ))
    }
}

/// SVG or HTML from the first 4 KiB of markup.
fn markup(text: &[u8]) -> Result<Sniffed, Box<Diagnostic>> {
    let head: Vec<u8> = text.iter().take(4096).map(u8::to_ascii_lowercase).collect();
    let html = [
        &b"<!doctype html"[..],
        b"<html",
        b"<head",
        b"<body",
        b"<script",
    ]
    .iter()
    .filter_map(|tag| find(&head, tag))
    .min();
    let svg = find(&head, b"<svg");
    let container = match (svg, html) {
        (Some(svg), Some(html)) if svg < html => Container::Svg,
        (Some(_), None) => Container::Svg,
        (_, Some(_)) => Container::Html,
        (None, None) => return Err(unknown("XML that is neither SVG nor HTML")),
    };
    Ok(Sniffed {
        container,
        json: None,
    })
}

/// Parses JSON with the parser's recursion limit (128 levels) and the depth budget.
pub(crate) fn parse_json(
    text: &[u8],
    budgets: &Budgets,
    work: &mut Work,
) -> Result<Value, Box<Diagnostic>> {
    work.spend(crate::budget::len64(text.len()))?;
    let value: Value = serde_json::from_slice(text).map_err(|err| {
        if err.to_string().contains("recursion limit") {
            Box::new(Diagnostic::new(
                &codes::NESTING_TOO_DEEP,
                "The JSON nests deeper than 128 levels.",
            ))
        } else {
            Box::new(Diagnostic::new(
                &codes::MALFORMED,
                format!("The JSON is not valid: {err}."),
            ))
        }
    })?;
    budgets.check_depth(json_depth(&value))?;
    Ok(value)
}

/// The nesting depth of a JSON value, without recursion.
fn json_depth(value: &Value) -> usize {
    let mut deepest = 0;
    let mut stack = vec![(value, 0_usize)];
    while let Some((value, depth)) = stack.pop() {
        deepest = deepest.max(depth);
        match value {
            Value::Array(items) => stack.extend(items.iter().map(|v| (v, depth + 1))),
            Value::Object(map) => stack.extend(map.values().map(|v| (v, depth + 1))),
            _ => {}
        }
    }
    deepest
}

/// The chart library a JSON document belongs to, by its keys.
pub(crate) fn chart_json(json: &Value) -> Option<Container> {
    if let Some(traces) = json.as_array() {
        return traces.iter().all(is_trace).then_some(Container::Plotly);
    }
    let object = json.as_object()?;
    if let Some(schema) = object.get("$schema").and_then(Value::as_str) {
        if schema.contains("vega-lite") {
            return Some(Container::VegaLite);
        }
        if schema.contains("vega") {
            return Some(Container::Vega);
        }
    }
    let has = |key: &str| object.contains_key(key);
    if has("marks") && (has("data") || has("scales") || has("axes")) {
        return Some(Container::Vega);
    }
    let composite = ["layer", "hconcat", "vconcat", "concat", "facet", "repeat"];
    if has("mark") || (composite.iter().any(|k| has(k)) && (has("data") || has("spec"))) {
        return Some(Container::VegaLite);
    }
    if let Some(traces) = object.get("data").and_then(Value::as_array)
        && !traces.is_empty()
        && traces.iter().all(is_trace)
    {
        return Some(Container::Plotly);
    }
    // Highcharts options: a `series` array of objects, some with `data`.
    let series = object.get("series").and_then(Value::as_array)?;
    (!series.is_empty()
        && series.iter().all(Value::is_object)
        && series.iter().any(|s| s.get("data").is_some()))
    .then_some(Container::Highcharts)
}

/// A Plotly trace: an object with a `type`, or with `x`, `y`, `values` or `z` arrays.
fn is_trace(value: &Value) -> bool {
    value.as_object().is_some_and(|trace| {
        trace.get("type").is_some_and(Value::is_string)
            || ["x", "y", "values", "z"]
                .iter()
                .any(|k| trace.get(*k).is_some_and(Value::is_array))
    })
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn unknown(what: &str) -> Box<Diagnostic> {
    Box::new(Diagnostic::new(
        &codes::UNKNOWN_FORMAT,
        format!("The file is {what}."),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::tests::package;

    fn container(bytes: &[u8]) -> Result<Container, String> {
        detect(bytes, &Budgets::DEFAULT).map_err(|d| d.code.as_str().to_owned())
    }

    /// One file of each kind, and the tier each goes to (C01-T03 acceptance).
    #[test]
    fn sniffing_picks_the_container_and_tier_of_each_kind() {
        let xlsx = package(&[("xl/workbook.xml", b"<workbook/>")]);
        let pptx = package(&[("ppt/presentation.xml", b"<p/>")]);
        let docx = package(&[("word/document.xml", b"<d/>")]);
        let cases: Vec<(&[u8], Container, Tier)> = vec![
            (&xlsx, Container::Xlsx, Tier::Exact),
            (&pptx, Container::Pptx, Tier::Exact),
            (&docx, Container::Docx, Tier::Exact),
            (
                br#"{"$schema": "https://vega.github.io/schema/vega-lite/v6.json", "mark": "line"}"#,
                Container::VegaLite,
                Tier::Exact,
            ),
            (
                br#"{"$schema": "https://vega.github.io/schema/vega/v6.json", "marks": []}"#,
                Container::Vega,
                Tier::Exact,
            ),
            (
                br#"{"data": [{"type": "scatter", "x": [1], "y": [2]}], "layout": {}}"#,
                Container::Plotly,
                Tier::Exact,
            ),
            (
                br#"{"chart": {"type": "line"}, "series": [{"data": [1, 2]}]}"#,
                Container::Highcharts,
                Tier::Exact,
            ),
            (
                b"<!DOCTYPE html><html><script type=\"application/json\">{}</script></html>",
                Container::Html,
                Tier::Exact,
            ),
            (
                b"\xEF\xBB\xBF<?xml version=\"1.0\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
                Container::Svg,
                Tier::Vector,
            ),
            (b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n", Container::Pdf, Tier::Vector),
            (b"\x89PNG\r\n\x1A\n\0\0\0\rIHDR", Container::Png, Tier::Raster),
            (b"\xFF\xD8\xFF\xE0\0\x10JFIF", Container::Jpeg, Tier::Raster),
        ];
        for (bytes, expected, tier) in cases {
            let found = container(bytes).expect("known container");
            assert_eq!(found, expected);
            assert_eq!(found.tier(), tier, "{expected:?}");
        }
    }

    #[test]
    fn unknown_input_is_a_diagnostic() {
        for bytes in [
            &b"just some text"[..],
            b"",
            br#"{"rows": [1, 2, 3]}"#,
            b"<note><to>x</to></note>",
            b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1rest",
            &package(&[("readme.txt", b"hello")]),
        ] {
            assert_eq!(container(bytes), Err("SNR-ENG-300".to_owned()));
        }
    }

    #[test]
    fn deep_json_and_oversized_input_fail_within_budget() {
        let deep = format!("{}{}", "[".repeat(200), "]".repeat(200));
        assert_eq!(container(deep.as_bytes()), Err("SNR-ENG-304".to_owned()));
        let mut budgets = Budgets::DEFAULT;
        budgets.max_depth = 5;
        let nested = br#"{"series": [{"data": [[[[[1]]]]]}]}"#;
        assert_eq!(
            detect(nested, &budgets).map_err(|d| d.code.as_str().to_owned()),
            Err("SNR-ENG-304".to_owned())
        );
        budgets = Budgets::DEFAULT;
        budgets.max_bytes = 10;
        assert_eq!(
            detect(b"%PDF-1.7 and more than ten bytes", &budgets)
                .map_err(|d| d.code.as_str().to_owned()),
            Err("SNR-ENG-301".to_owned())
        );
    }

    #[test]
    fn extractors_fit_their_containers() {
        assert!(Container::Pptx.fits(Extractor::Office));
        assert!(Container::Highcharts.fits(Extractor::Auto));
        assert!(!Container::Highcharts.fits(Extractor::Vega));
        assert!(!Container::Svg.fits(Extractor::Office));
        assert!(Container::Pdf.fits(Extractor::PdfVector));
    }
}
