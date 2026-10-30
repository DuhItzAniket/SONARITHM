// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading a raster chart's lines (C02-T03): the plot area, the series colours, a curve per
//! colour and its values through the user's calibration, each a `cv_estimate` with its own
//! confidence (AGENTS rule 7).

use sonarithm_core::{Confidence, Diagnostic, Provenance, ProvenanceClass};
use sonarithm_spec::v02::Calibration as SpecCalibration;

use super::colour::{ColourFilter, ColourMode, Mask, separating_threshold, series_colours};
use super::lines::{PixelPoint, Search, averaging_window, x_step};
use super::{PixelRect, Raster, RasterCalibration, find_axes};
use crate::budget::Work;
use crate::calibrate::ScaleKind;
use crate::codes;
use crate::model::{ChartData, ChartType, Point, Series, XValue};

/// The most series read from one chart.
const MAX_SERIES: usize = 8;

/// The plot area: the calibration's, else the one the axis lines bound, else the box the
/// anchors span (anchors sit on the outermost ticks a person can read).
fn plot_area(
    raster: &Raster,
    calibration: &SpecCalibration,
    work: &mut Work,
) -> Result<PixelRect, Box<Diagnostic>> {
    if let Some(area) = &calibration.plot_area {
        return Ok(PixelRect {
            min: area.min,
            max: area.max,
        });
    }
    if let Some(plot) = find_axes(raster, work)?.plot {
        return Ok(plot);
    }
    let xs = calibration.axes.x.anchors.iter().map(|a| a.at[0]);
    let ys = calibration.axes.y.anchors.iter().map(|a| a.at[1]);
    let span = |v: Vec<f64>| {
        (
            v.iter().copied().fold(f64::INFINITY, f64::min),
            v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        )
    };
    let (x0, x1) = span(xs.collect());
    let (y0, y1) = span(ys.collect());
    // Half a category step beyond the outer labels, and a few pixels around the value ticks.
    let pad = 4.0;
    Ok(PixelRect {
        min: [x0 - pad, y0 - pad],
        max: [x1 + pad, y1 + pad],
    })
}

/// Reads the lines of a raster chart with `calibration`.
///
/// # Errors
///
/// The calibration's error (`SNR-ENG-320`, `321`), `SNR-ENG-309` when no series colour is
/// found, `SNR-ENG-305` over the work budget.
pub(crate) fn read_lines(
    raster: &Raster,
    calibration: &SpecCalibration,
    work: &mut Work,
) -> Result<ChartData, Box<Diagnostic>> {
    let axes = RasterCalibration::from_spec(calibration)?;
    let area = plot_area(raster, calibration, work)?;
    work.spend(u64::from(raster.width()) * u64::from(raster.height()))?;
    let colours = series_colours(raster, &area, MAX_SERIES);
    if colours.is_empty() {
        return Err(Box::new(Diagnostic::new(
            &codes::NO_CHART,
            "No coloured marks were found in the plot area; the raster tier reads coloured lines.",
        )));
    }
    let mask = Mask::default();
    let width = (area.max[0] - area.min[0]).abs();
    // The curve's width in pixels: about 0.4 % of the plot (2 to 3 pixels at screen size).
    let thickness = (width * 0.004).max(3.0);
    let y_span = (axes.y.value_at(area.min[1]) - axes.y.value_at(area.max[1])).abs();
    let mut series = Vec::new();
    for (index, &colour) in colours.iter().enumerate() {
        // Each series scans the plot area once more.
        work.spend(u64::from(raster.width()) * u64::from(raster.height()))?;
        let others: Vec<[u8; 3]> = colours.iter().copied().filter(|c| *c != colour).collect();
        let search = Search {
            raster,
            filter: ColourFilter {
                mode: ColourMode::Foreground(colour),
                threshold: separating_threshold(colour, &others),
            },
            area,
            mask: &mask,
        };
        let found: Vec<(Option<String>, PixelPoint)> = if axes.x_labels.is_some() {
            // Custom independents: the curve at each category's position.
            x_step(&search, &axes.label_positions(), thickness, true)
                .into_iter()
                .flatten()
                .map(|p| (axes.label_at(p.x).map(str::to_owned), p))
                .collect()
        } else {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "small and positive"
            )]
            let window = thickness.ceil() as u32;
            averaging_window(&search, window, window)
                .into_iter()
                .map(|p| (None, p))
                .collect()
        };
        if found.is_empty() {
            continue;
        }
        let mut points = Vec::with_capacity(found.len());
        let mut confidences = Vec::with_capacity(found.len());
        for (label, p) in found {
            let (x, y) = axes.value_at(p.x, p.y);
            let x = match (label, axes.x.kind()) {
                (Some(label), _) => XValue::Category(label),
                #[expect(clippy::cast_possible_truncation, reason = "milliseconds fit i64")]
                (None, ScaleKind::Date) => XValue::Time(x.value.round() as i64),
                (None, _) => XValue::Number(x.value),
            };
            points.push(Point {
                x,
                y: Some(y.value),
            });
            // Colour closeness, times how much of the plot's value span half a pixel covers.
            let quantization = if y_span > 0.0 {
                1.0 - (y.uncertainty() / y_span).min(1.0)
            } else {
                0.0
            };
            #[expect(clippy::cast_possible_truncation, reason = "a confidence in [0, 1]")]
            let c = (p.closeness * quantization).clamp(0.0, 1.0) as f32;
            confidences.extend(Confidence::new(c).or(Confidence::new(0.0)));
        }
        let mut sorted: Vec<f32> = confidences.iter().map(|c| f32::from(*c)).collect();
        sorted.sort_by(f32::total_cmp);
        let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);
        let mut line = Series::exact(format!("Series {}", index + 1), points);
        line.provenance = Provenance {
            class: ProvenanceClass::CvEstimate,
            confidence: Confidence::new(median),
        };
        line.confidences = confidences;
        series.push(line);
    }
    Ok(ChartData {
        chart_type: Some(ChartType::Line),
        series,
        ..ChartData::default()
    })
}

#[cfg(test)]
mod tests;
