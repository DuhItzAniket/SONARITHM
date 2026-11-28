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
//! by the UI. [`check`] can also give each fitted axis's value at every pixel column or row,
//! so the dialog says a crosshair step's value in the same moment as its pixel. [`anchor_area`]
//! and [`anchors_outside`] let the dialog choose the area to read (#516).

use sonarithm_core::Diagnostic;
use sonarithm_spec::v02::{AxisKind, Calibration as SpecCalibration, CalibrationAxis};

use super::{Axes, PixelRect, RasterCalibration, decode, estimate, find_axes};
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
/// `SNR-ENG-300` for bytes no chart format starts with, `SNR-ENG-327` for another chart
/// format (an SVG) or an image that cannot be decoded,
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
    /// Each fitted axis's value at every pixel column (x) or row (y), at the pixel's centre,
    /// when the image's size was given and is at most [`MAX_VALUES`] along the axis.
    pub values: [Option<Vec<ReadValue>>; 2],
}

/// The most pixels along an axis for which [`check`] gives every value: beyond it the reply
/// would be large, and the dialog reads one value at a time instead.
pub const MAX_VALUES: u32 = 10_000;

/// Checks `calibration` as it stands. An axis with fewer than two anchors is not checked,
/// so it adds no diagnostic; `at` is a pixel position (a pixel's centre is `i + 0.5`).
/// With the image's `size` (width, height), each fitted axis also gets its value at every
/// pixel along it, up to [`MAX_VALUES`].
#[must_use]
pub fn check(
    calibration: &SpecCalibration,
    at: Option<[f64; 2]>,
    size: Option<[u32; 2]>,
) -> CalibrationCheck {
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
                if let Some(along) = size
                    .and_then(|size| size.get(index).copied())
                    .filter(|&along| along <= MAX_VALUES)
                {
                    out.values[index] = Some(
                        (0..along)
                            .map(|i| read(axis, &fit, labels.as_deref(), f64::from(i) + 0.5))
                            .collect(),
                    );
                }
            }
            Err(refused) => out.diagnostics.push(*refused),
        }
    }
    out
}

/// The box the anchors span, widened as the extraction widens it when no plot area is
/// given: the area to read by default (#516), rather than a detected plot area an area
/// chart's fill can mislead.
///
/// # Errors
///
/// The calibration's refusal (`SNR-ENG-320` to `322`) when an axis does not fit.
pub fn anchor_area(calibration: &SpecCalibration) -> Result<PixelRect, Box<Diagnostic>> {
    let axes = RasterCalibration::from_spec(calibration)?;
    Ok(super::extract::anchor_box(calibration, &axes))
}

/// A warning when an anchor lies outside `area` along its own axis (an x anchor's x, a y
/// anchor's y), more than a pixel out: the detected plot area is then likely wrong (#516).
#[must_use]
pub fn anchors_outside(calibration: &SpecCalibration, area: PixelRect) -> Option<Diagnostic> {
    let out = |value: f64, min: f64, max: f64| value < min - 1.0 || value > max + 1.0;
    let count = calibration
        .axes
        .x
        .anchors
        .iter()
        .filter(|a| out(a.at[0], area.min[0], area.max[0]))
        .count()
        + calibration
            .axes
            .y
            .anchors
            .iter()
            .filter(|a| out(a.at[1], area.min[1], area.max[1]))
            .count();
    (count > 0).then(|| {
        Diagnostic::new(
            &codes::ANCHORS_OUTSIDE_PLOT,
            format!(
                "{count} anchor(s) lie outside the plot area found from the axis lines, so \
                 that area is likely wrong. Read the area the anchors span instead."
            ),
        )
    })
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
        let checked = check(&partial, Some([60.0, 60.0]), None);
        assert_eq!(checked.x, None, "one anchor is not checked");
        assert!(checked.diagnostics.is_empty(), "{:?}", checked.diagnostics);
        let y = checked.y.unwrap();
        assert_eq!(y.anchors, 2);
        assert_eq!(checked.value_at[0], None);
        assert!(
            matches!(&checked.value_at[1], Some(ReadValue::Number(value)) if (value - 50.0).abs() < 1e-9),
            "{:?}",
            checked.value_at
        );
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
        let checked = check(&unordered, None, None);
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
        let checked = check(&mixed, Some([50.0, 0.0]), None);
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
        assert_eq!(
            check(&bad, None, None).diagnostics[0].code.as_str(),
            "SNR-ENG-320"
        );
        let labels = calibration(
            r#"{"axes": {
                "x": {"kind": "category", "anchors": [
                    {"at": [20, 0], "value": "apples"}, {"at": [80, 0], "value": "pears"}]},
                "y": {"kind": "linear", "anchors": []}}}"#,
        );
        assert_eq!(
            check(&labels, Some([75.0, 0.0]), None).value_at[0],
            Some(ReadValue::Text("pears".to_owned()))
        );
    }

    #[test]
    fn c02_t10_an_image_is_inspected_and_anything_else_refused() {
        let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/raster");
        let read = |name: &str| std::fs::read(corpus.join(name)).unwrap();
        let info = inspect(&read("d3-bar@1x.png"), &Budgets::DEFAULT).unwrap();
        assert_eq!(info.mime, "image/png");
        assert!(info.width > 0 && info.height > 0);
        assert!(
            info.plot.is_some(),
            "the d3 bar chart's axis lines bound a plot area"
        );
        // The line chart's axis lines are not found (C02-T02), so the dialog shows no plot
        // area and the read falls back to the box the anchors span.
        assert_eq!(
            inspect(&read("d3-line@1x.png"), &Budgets::DEFAULT)
                .unwrap()
                .plot,
            None
        );
        let refused = inspect(b"<svg/>", &Budgets::DEFAULT).unwrap_err();
        assert_eq!(refused.code.as_str(), "SNR-ENG-327");
        let refused = inspect(b"not an image", &Budgets::DEFAULT).unwrap_err();
        assert_eq!(refused.code.as_str(), "SNR-ENG-300");
    }

    /// Item 2 of the #517 review: every pixel's value comes with the check, so the dialog's
    /// slider says a step's value at once and never changes what it says afterwards.
    #[test]
    fn c02_t10_a_fitted_axis_gives_its_value_at_every_pixel() {
        let fitted = calibration(
            r#"{"axes": {
                "x": {"kind": "linear", "anchors": [
                    {"at": [10.5, 0], "value": 0}, {"at": [110.5, 0], "value": 100}]},
                "y": {"kind": "linear", "anchors": [{"at": [0, 50], "value": 0}]}}}"#,
        );
        let checked = check(&fitted, None, Some([200, 80]));
        let xs = checked.values[0].as_ref().unwrap();
        assert_eq!(xs.len(), 200);
        assert!(
            matches!(xs[60], ReadValue::Number(v) if (v - 50.0).abs() < 1e-9),
            "{:?}",
            xs[60]
        );
        assert_eq!(checked.values[1], None, "one anchor is not fitted");
        let wide = check(&fitted, None, Some([MAX_VALUES + 1, 80]));
        assert_eq!(wide.values[0], None, "too wide to list");
    }

    /// Item 3 of the #517 review (#516): the area the anchors span is what the dialog reads by
    /// default, and anchors outside a detected plot area are warned about.
    #[test]
    fn c02_t10_the_anchors_area_and_a_warning_for_anchors_outside_the_detected_one() {
        let both = calibration(
            r#"{"axes": {
                "x": {"kind": "linear", "anchors": [
                    {"at": [100, 300], "value": 0}, {"at": [500, 300], "value": 10}]},
                "y": {"kind": "linear", "anchors": [
                    {"at": [90, 280], "value": 0}, {"at": [90, 80], "value": 100}]}}}"#,
        );
        let area = anchor_area(&both).unwrap();
        assert_eq!(area.min, [100.0 - 8.0, 80.0 - 4.0]);
        assert_eq!(area.max, [500.0 + 8.0, 280.0 + 4.0]);
        // vl-area@1x's misread plot area: x anchors left of it.
        let misread = PixelRect {
            min: [419.0, 33.0],
            max: [799.0, 283.0],
        };
        let warned = anchors_outside(&both, misread).unwrap();
        assert_eq!(warned.code.as_str(), "SNR-ENG-330");
        assert_eq!(anchors_outside(&both, area), None);
    }
}
