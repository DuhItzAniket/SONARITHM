// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading a raster chart (C02-T03, C02-T04): the plot area, the series colours, and per
//! colour a curve, bars or point markers, with values through the user's calibration, each a
//! `cv_estimate` with its own confidence (AGENTS rule 7).

use sonarithm_core::{Confidence, Diagnostic, Provenance, ProvenanceClass};
use sonarithm_spec::v02::Calibration as SpecCalibration;

use super::colour::{ColourFilter, ColourMode, Mask, separating_threshold, series_colours};
use super::components::{Component, Labels, label};
use super::lines::{Search, averaging_window, x_step};
use super::marks::{Bar, bars, markers};
use super::{PixelRect, Raster, RasterCalibration, find_axes};
use crate::budget::Work;
use crate::calibrate::ScaleKind;
use crate::codes;
use crate::model::{ChartData, ChartType, Point, Series, XValue};

/// The most series read from one chart.
const MAX_SERIES: usize = 8;

/// The plot area: the calibration's, else the one the axis lines bound, else the box the
/// anchors span (anchors sit on the outermost ticks a person can read), widened by half a
/// category on a category axis and by 2 % of the span (at least 4 pixels) otherwise, so
/// the outer bars and markers stay whole.
fn plot_area(
    raster: &Raster,
    calibration: &SpecCalibration,
    axes: &RasterCalibration,
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
    let pad = |lo: f64, hi: f64| ((hi - lo) * 0.02).max(4.0);
    let x_pad = match &axes.x_labels {
        Some(labels) if labels.len() > 1 => {
            #[expect(clippy::cast_precision_loss, reason = "label counts are small")]
            let step = (x1 - x0) / (labels.len() - 1) as f64;
            step / 2.0
        }
        _ => pad(x0, x1),
    };
    let y_pad = pad(y0, y1);
    Ok(PixelRect {
        min: [x0 - x_pad, y0 - y_pad],
        max: [x1 + x_pad, y1 + y_pad],
    })
}

/// What one series colour draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// A curve.
    Line,
    /// Bars or histogram bins.
    Bars,
    /// Point markers.
    Points,
}

/// Classifies what a colour draws (our design): bars when the bars found in it that stand
/// on (or hang from) the baseline, within 3 pixels, cover at least 80 % of its pixels;
/// points when at least three components are marker-sized (2 pixels to `max_marker`) and
/// hold at least half its pixels; otherwise a line. Specks under 4 pixels are ignored.
fn classify(labels: &Labels, found: &[Bar], baseline: f64, max_marker: u32) -> Kind {
    let shapes: Vec<&Component> = labels.components.iter().filter(|c| c.pixels >= 4).collect();
    let total: u64 = shapes.iter().map(|c| u64::from(c.pixels)).sum();
    if total == 0 {
        return Kind::Line;
    }
    #[expect(clippy::cast_precision_loss, reason = "pixel counts")]
    let total = total as f64;
    let standing: f64 = found
        .iter()
        .filter(|b| (b.base - baseline).abs() <= 3.0)
        .map(Bar::area)
        .sum();
    if standing >= 0.8 * total {
        return Kind::Bars;
    }
    let markers: Vec<&&Component> = shapes
        .iter()
        .filter(|c| (2..=max_marker).contains(&c.width().max(c.height())))
        .collect();
    let marker_pixels: u64 = markers.iter().map(|c| u64::from(c.pixels)).sum();
    #[expect(clippy::cast_precision_loss, reason = "pixel counts")]
    let marker_share = marker_pixels as f64 / total;
    if markers.len() >= 3 && marker_share >= 0.5 {
        return Kind::Points;
    }
    Kind::Line
}

/// A mark found for one value: where it is in pixels, the category it falls in (on a
/// category x axis) and how closely its colour matched.
struct Found {
    label: Option<String>,
    at: [f64; 2],
    closeness: f64,
}

/// The marks one colour draws, as found for its kind: bars, markers, or the curve sampled
/// at the categories (custom independents) or by the averaging window. On a category axis
/// a mark between categories belongs to none and is left out.
fn marks_of(
    kind: Kind,
    found_bars: Vec<Bar>,
    labels: &Labels,
    search: &Search<'_>,
    axes: &RasterCalibration,
    thickness: f64,
    max_marker: u32,
) -> Vec<Found> {
    let label_of = |x: f64| axes.label_at(x).map(str::to_owned);
    let category = axes.x_labels.is_some();
    let found: Vec<Found> = match kind {
        Kind::Bars => found_bars
            .into_iter()
            .map(|b| Found {
                label: label_of(b.centre()),
                at: [b.centre(), b.edge],
                closeness: b.closeness,
            })
            .collect(),
        Kind::Points => markers(labels, search, 2, max_marker)
            .into_iter()
            .map(|m| Found {
                label: label_of(m.at[0]),
                at: m.at,
                // A marker separated from a touching one is less certain.
                closeness: m.closeness / f64::from(m.merged),
            })
            .collect(),
        Kind::Line if category => {
            // Custom independents: the curve at each category's position.
            x_step(search, &axes.label_positions(), thickness, true)
                .into_iter()
                .flatten()
                .map(|p| Found {
                    label: label_of(p.x),
                    at: [p.x, p.y],
                    closeness: p.closeness,
                })
                .collect()
        }
        Kind::Line => {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "small and positive"
            )]
            let window = thickness.ceil() as u32;
            averaging_window(search, window, window)
                .into_iter()
                .map(|p| Found {
                    label: None,
                    at: [p.x, p.y],
                    closeness: p.closeness,
                })
                .collect()
        }
    };
    found
        .into_iter()
        .filter(|f| !category || f.label.is_some())
        .collect()
}

/// A series from the marks found for one colour: values through the calibration, the
/// `cv_estimate` class, and per value its colour closeness times how much of the plot's
/// value span (`y_span`) half a pixel leaves certain. `None` when nothing was found.
fn series_of(
    found: Vec<Found>,
    axes: &RasterCalibration,
    y_span: f64,
    kind: Kind,
    index: usize,
) -> Option<Series> {
    if found.is_empty() {
        return None;
    }
    let category = axes.x_labels.is_some();
    let mut points = Vec::with_capacity(found.len());
    let mut confidences = Vec::with_capacity(found.len());
    for f in found {
        let (x, y) = axes.value_at(f.at[0], f.at[1]);
        let x = match (f.label, axes.x.kind()) {
            (Some(label), _) => XValue::Category(label),
            #[expect(clippy::cast_possible_truncation, reason = "milliseconds fit i64")]
            (None, ScaleKind::Date) => XValue::Time(x.value.round() as i64),
            (None, _) => XValue::Number(x.value),
        };
        points.push(Point {
            x,
            y: Some(y.value),
        });
        let quantization = if y_span > 0.0 {
            1.0 - (y.uncertainty() / y_span).min(1.0)
        } else {
            0.0
        };
        #[expect(clippy::cast_possible_truncation, reason = "a confidence in [0, 1]")]
        let c = (f.closeness * quantization).clamp(0.0, 1.0) as f32;
        confidences.extend(Confidence::new(c).or(Confidence::new(0.0)));
    }
    let mut sorted: Vec<f32> = confidences.iter().map(|c| f32::from(*c)).collect();
    sorted.sort_by(f32::total_cmp);
    let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);
    let mut one = Series::exact(format!("Series {}", index + 1), points);
    one.chart_type = Some(match kind {
        Kind::Line => ChartType::Line,
        Kind::Bars if category => ChartType::Bar,
        Kind::Bars => ChartType::Histogram,
        Kind::Points => ChartType::Scatter,
    });
    one.provenance = Provenance {
        class: ProvenanceClass::CvEstimate,
        confidence: Confidence::new(median),
    };
    one.confidences = confidences;
    Some(one)
}

/// Reads a raster chart with `calibration`: each series colour as a line, bars or points
/// (C02-T03, C02-T04).
///
/// # Errors
///
/// The calibration's error (`SNR-ENG-320`, `321`), `SNR-ENG-309` when no series colour is
/// found, `SNR-ENG-305` over the work budget.
pub(crate) fn read_raster(
    raster: &Raster,
    calibration: &SpecCalibration,
    work: &mut Work,
) -> Result<ChartData, Box<Diagnostic>> {
    let axes = RasterCalibration::from_spec(calibration)?;
    let area = plot_area(raster, calibration, &axes, work)?;
    work.spend(u64::from(raster.width()) * u64::from(raster.height()))?;
    let colours = series_colours(raster, &area, MAX_SERIES);
    if colours.is_empty() {
        return Err(Box::new(Diagnostic::new(
            &codes::NO_CHART,
            "No coloured marks were found in the plot area; the raster tier reads coloured lines, bars and points.",
        )));
    }
    let mask = Mask::default();
    let width = (area.max[0] - area.min[0]).abs();
    // The curve's width in pixels: about 0.4 % of the plot (2 to 3 pixels at screen size).
    let thickness = (width * 0.004).max(3.0);
    // The largest single marker: 4 % of the plot's width, at least 6 pixels (our choice).
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "small and positive"
    )]
    let max_marker = (width * 0.04).max(6.0) as u32;
    // The row of value 0 (where bars stand), clamped into the plot area.
    let baseline = axes
        .y
        .position_of(0.0)
        .map_or(area.max[1], |p| p.clamp(area.min[1], area.max[1]));
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
        let labels = label(&search, work)?;
        let found_bars = bars(&labels, baseline, 1.5);
        let kind = classify(&labels, &found_bars, baseline, max_marker);
        let found = marks_of(
            kind, found_bars, &labels, &search, &axes, thickness, max_marker,
        );
        if let Some(one) = series_of(found, &axes, y_span, kind, index) {
            series.push(one);
        }
    }
    // One chart type when every series agrees; otherwise each series keeps its own.
    let first = series.first().and_then(|s| s.chart_type);
    let chart_type = if series.iter().all(|s| s.chart_type == first) {
        for s in &mut series {
            s.chart_type = None;
        }
        first
    } else {
        None
    };
    Ok(ChartData {
        chart_type,
        series,
        ..ChartData::default()
    })
}

#[cfg(test)]
mod tests;
