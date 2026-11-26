// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A raster chart before it is read (C02-T10; ADR-0017 "Raster calibration"): what the
//! calibration dialog needs to show the image and check anchors as the user enters them.
//!
//! [`inspect`] decodes the image under the budgets, as an extraction would, so a hostile
//! file fails here first, and finds the axis lines and plot area (C02-T02). [`check`] fits
//! each axis that has at least two anchors with C01-T04's core and runs the 10 §3.1 checks
//! on it, without reading any data. The values at a pixel come from the same fit, so the
//! dialog's readout and the extraction agree, and every date and label is read here, never
//! by the UI.

use sonarithm_core::Diagnostic;
use sonarithm_spec::v02::{AxisKind, Calibration as SpecCalibration, CalibrationAxis};

use super::{Axes, PixelRect, decode, estimate, find_axes};
use crate::budget::{Budgets, Work};
use crate::calibrate::ticks::iso_from_ms;
use crate::codes;
use crate::detect::{Container, sniff};

/// An image a chart can be read from: its format, size and the plot area its axis lines
/// bound.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct RasterInfo {
    /// The media type of the bytes, from their signature: `image/png` or `image/jpeg`.
    pub mime: &'static str,
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// The plot area between the axis lines, when both are found.
    pub plot: Option<PixelRect>,
}

/// Decodes `bytes` under `budgets` and finds its axes.
///
/// # Errors
///
/// `SNR-ENG-327` for bytes that are not a PNG or JPEG image or cannot be decoded,
/// `SNR-ENG-306` over the pixel budget, `SNR-ENG-305` over the work budget, and the input
/// size limit's code over `budgets.max_bytes`.
pub fn inspect(bytes: &[u8], budgets: &Budgets) -> Result<RasterInfo, Box<Diagnostic>> {
    let mut work = Work::new(budgets.max_work);
    let container = sniff(bytes, budgets, &mut work)?.container;
    let (format, mime) = match container {
        Container::Png => (image::ImageFormat::Png, "image/png"),
        Container::Jpeg => (image::ImageFormat::Jpeg, "image/jpeg"),
        _ => {
            return Err(Box::new(Diagnostic::new(
                &codes::RASTER_UNREADABLE,
                "The file is not a PNG or JPEG image.",
            )));
        }
    };
    let raster = decode(bytes, format, budgets, &mut work)?;
    let Axes { plot, .. } = find_axes(&raster, &mut work)?;
    Ok(RasterInfo {
        mime,
        width: raster.width(),
        height: raster.height(),
        plot,
    })
}

/// How well one axis's anchors fit.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct AxisFit {
    /// The anchors fitted.
    pub anchors: usize,
    /// The root-mean-square distance of the anchors from the fit, as a share of their span.
    pub residual: f64,
    /// The confidence values read through the axis get, from the residual.
    pub confidence: f64,
}

/// A value read at a pixel: a number, or the text of a date or category.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ReadValue {
    /// A linear or logarithmic axis's value.
    Number(f64),
    /// A date axis's value (ISO 8601) or a category axis's label.
    Text(String),
}

/// A calibration being entered, checked.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct CalibrationCheck {
    /// The x axis's fit, once it has two anchors that fit.
    pub x: Option<AxisFit>,
    /// The y axis's fit.
    pub y: Option<AxisFit>,
    /// Why an axis with two anchors or more does not fit (`SNR-ENG-320` to `322`), and the
    /// warnings of one that does (`SNR-ENG-323`, `325`), x first.
    pub diagnostics: Vec<Diagnostic>,
    /// The values at the pixel asked about, x then y, each once its axis fits.
    pub value_at: [Option<ReadValue>; 2],
}

/// Checks `calibration` as it stands. An axis with fewer than two anchors is not checked,
/// so it adds no diagnostic; `at` is a pixel position (a pixel's centre is `i + 0.5`).
#[must_use]
pub fn check(calibration: &SpecCalibration, at: Option<[f64; 2]>) -> CalibrationCheck {
    let mut out = CalibrationCheck::default();
    for (index, axis) in [&calibration.axes.x, &calibration.axes.y]
        .into_iter()
        .enumerate()
    {
        if axis.anchors.len() < 2 {
            continue;
        }
        let name = if index == 0 { "x" } else { "y" };
        let mut warnings = Vec::new();
        match super::axis(axis, name, index, &mut warnings) {
            Ok((fit, labels)) => {
                let summary = AxisFit {
                    anchors: fit.anchors(),
                    residual: fit.residual(),
                    confidence: f64::from(fit.confidence().get()),
                };
                if index == 0 {
                    out.x = Some(summary);
                } else {
                    out.y = Some(summary);
                }
                out.diagnostics.extend(warnings);
                if let Some(position) = at.and_then(|at| at.get(index).copied()) {
                    out.value_at[index] = Some(read(axis, &fit, labels.as_deref(), position));
                }
            }
            Err(refused) => out.diagnostics.push(*refused),
        }
    }
    out
}

/// The value at `position` on a fitted axis, in the axis's own terms.
fn read(
    axis: &CalibrationAxis,
    fit: &crate::calibrate::Calibration,
    labels: Option<&[String]>,
    position: f64,
) -> ReadValue {
    let value = estimate(fit, position).value;
    match (axis.kind, labels) {
        (AxisKind::Category, Some(labels)) => {
            let index = value.round();
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "checked to lie within the labels first"
            )]
            let label = (index >= 0.0)
                .then(|| labels.get(index as usize))
                .flatten()
                .cloned()
                .unwrap_or_default();
            ReadValue::Text(label)
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a date's milliseconds since 1970 fit i64"
        )]
        (AxisKind::Date, _) => ReadValue::Text(iso_from_ms(value.round() as i64)),
        _ => ReadValue::Number(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calibration(text: &str) -> SpecCalibration {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn c02_t10_an_axis_is_checked_once_it_has_two_anchors() {
        let partial = calibration(
            r#"{"axes": {
                "x": {"kind": "linear", "anchors": [{"at": [10, 0], "value": 0}]},
                "y": {"kind": "linear", "anchors": [
                    {"at": [0, 110], "value": 0}, {"at": [0, 10], "value": 100}]}}}"#,
        );
        let checked = check(&partial, Some([60.0, 60.0]));
        assert_eq!(checked.x, None, "one anchor is not checked");
        assert!(checked.diagnostics.is_empty(), "{:?}", checked.diagnostics);
        let y = checked.y.unwrap();
        assert_eq!(y.anchors, 2);
        assert_eq!(checked.value_at[0], None);
        let Some(ReadValue::Number(value)) = &checked.value_at[1] else {
            panic!("{:?}", checked.value_at);
        };
        assert!((value - 50.0).abs() < 1e-9, "{value}");
    }

    #[test]
    fn c02_t10_unordered_ticks_are_refused_with_their_code() {
        let unordered = calibration(
            r#"{"axes": {
                "x": {"kind": "linear", "anchors": [
                    {"at": [10, 0], "value": 0}, {"at": [60, 0], "value": 10},
                    {"at": [110, 0], "value": 5}]},
                "y": {"kind": "linear", "anchors": []}}}"#,
        );
        let checked = check(&unordered, None);
        assert_eq!(checked.x, None);
        assert_eq!(checked.diagnostics[0].code.as_str(), "SNR-ENG-322");
    }

    #[test]
    fn c02_t10_dates_and_categories_are_read_in_rust() {
        let mixed = calibration(
            r#"{"axes": {
                "x": {"kind": "date", "anchors": [
                    {"at": [0, 0], "value": "2024-01-01"}, {"at": [100, 0], "value": "2024-01-11"}]},
                "y": {"kind": "linear", "anchors": [
                    {"at": [0, 100], "value": 0}, {"at": [0, 0], "value": 1}]}}}"#,
        );
        let checked = check(&mixed, Some([50.0, 0.0]));
        assert_eq!(
            checked.value_at[0],
            Some(ReadValue::Text("2024-01-06".to_owned()))
        );
        let bad = calibration(
            r#"{"axes": {
                "x": {"kind": "date", "anchors": [
                    {"at": [0, 0], "value": "soon"}, {"at": [100, 0], "value": "2024-01-11"}]},
                "y": {"kind": "linear", "anchors": []}}}"#,
        );
        assert_eq!(check(&bad, None).diagnostics[0].code.as_str(), "SNR-ENG-320");
        let labels = calibration(
            r#"{"axes": {
                "x": {"kind": "category", "anchors": [
                    {"at": [20, 0], "value": "apples"}, {"at": [80, 0], "value": "pears"}]},
                "y": {"kind": "linear", "anchors": []}}}"#,
        );
        assert_eq!(
            check(&labels, Some([75.0, 0.0])).value_at[0],
            Some(ReadValue::Text("pears".to_owned()))
        );
    }

    #[test]
    fn c02_t10_an_image_is_inspected_and_anything_else_refused() {
        let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/corpus/raster/d3-line@1x.png");
        let info = inspect(&std::fs::read(corpus).unwrap(), &Budgets::DEFAULT).unwrap();
        assert_eq!(info.mime, "image/png");
        assert!(info.width > 0 && info.height > 0);
        assert!(info.plot.is_some(), "the d3 axes bound a plot area");
        let refused = inspect(b"<svg/>", &Budgets::DEFAULT).unwrap_err();
        assert_eq!(refused.code.as_str(), "SNR-ENG-327");
    }
}
