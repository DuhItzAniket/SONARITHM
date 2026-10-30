// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The raster tier's core (C02-T02, 10 §3 tier 3): decoding PNG and JPEG under a pixel
//! budget, finding the axis lines and the plot area, and mapping pixels to values through a
//! calibration the user gives, with the half-pixel uncertainty of a raster carried through.
//!
//! **Clean room (ADR-0022).** The workflow follows `WebPlotDigitizer`'s user documentation
//! (register CR-01): a 2D XY calibration from two points on each axis, entered by the user
//! with a value and a scale (linear, log or date), placed as far apart as possible. The
//! axis-line search, the darkness threshold and the uncertainty model are SONARITHM's own
//! design; the documentation says nothing about them. No OCR runs: every anchor is entered
//! by hand (C02-T07 waits on OD-23).
//!
//! **Coordinates.** Pixels are counted from the top-left corner, y downwards, and a pixel's
//! centre is at `+0.5` (RFC-0004 §5), so pixel `(i, j)` covers `[i, i + 1) × [j, j + 1)`.
//! A mark drawn into a pixel can lie anywhere in it, so a value read at a pixel centre is
//! only known to within the values half a pixel either side ([`Estimate`]).

pub mod colour;
pub(crate) mod extract;
pub mod lines;

use std::io::Cursor;

use image::{ImageFormat, ImageReader, Limits};
use serde::Serialize;
use sonarithm_core::Diagnostic;
use sonarithm_spec::v02::{AnchorValue, AxisKind, Calibration as SpecCalibration, CalibrationAxis};

use crate::budget::{Budget, Budgets, Work, len64, over};
use crate::calibrate::ticks::parse_date;
use crate::calibrate::{Anchor, Calibration, ScaleKind};
use crate::codes;

/// A decoded image, flattened onto white (transparent pixels count as background).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Raster {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 3]>,
}

impl Raster {
    /// An image from its RGB pixels, row by row.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-327` when `pixels` does not hold `width × height` pixels.
    pub fn from_rgb(
        width: u32,
        height: u32,
        pixels: Vec<[u8; 3]>,
    ) -> Result<Self, Box<Diagnostic>> {
        if len64(pixels.len()) != u64::from(width) * u64::from(height) {
            return Err(unreadable("the pixel count does not match the size"));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// The width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The colour of pixel `(x, y)`, if it is inside the image.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 3]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = usize::try_from(u64::from(y) * u64::from(self.width) + u64::from(x)).ok()?;
        self.pixels.get(index).copied()
    }

    /// Whether pixel `(x, y)` is dark enough to be part of an axis line.
    fn dark(&self, x: u32, y: u32) -> bool {
        self.pixel(x, y).is_some_and(|[r, g, b]| {
            // ITU-R BT.601 luma; axis lines are drawn dark on a light background.
            let luma = 299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b);
            luma < DARK_LUMA * 1000
        })
    }
}

/// Luma (0–255) below which a pixel counts as dark: halfway, our choice.
const DARK_LUMA: u32 = 128;

// An axis line must span at least half the image's width or height (our choice). A grid
// line spans as much, so the axis is the outermost such line (bottom, left).

fn unreadable(why: impl std::fmt::Display) -> Box<Diagnostic> {
    Box::new(Diagnostic::new(
        &codes::RASTER_UNREADABLE,
        format!("The image cannot be read: {why}."),
    ))
}

/// Decodes a PNG or JPEG image. The size in the header is checked against the pixel budget
/// before any pixel is decoded, and the decoder's own allocation limit is set from it, so a
/// decompression bomb stops without allocating its pixels.
///
/// # Errors
///
/// `SNR-ENG-306` over the pixel budget, `SNR-ENG-327` for an image that cannot be decoded,
/// `SNR-ENG-305` over the work budget.
pub fn decode(
    bytes: &[u8],
    format: ImageFormat,
    budgets: &Budgets,
    work: &mut Work,
) -> Result<Raster, Box<Diagnostic>> {
    work.spend(len64(bytes.len()))?;
    let reader = |bytes| ImageReader::with_format(Cursor::new(bytes), format);
    let (width, height) = reader(bytes).into_dimensions().map_err(unreadable)?;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > budgets.max_pixels {
        return Err(over(Budget::Pixels, pixels, budgets.max_pixels));
    }
    work.spend(pixels)?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(width);
    limits.max_image_height = Some(height);
    // RGBA at 16 bits per channel, the widest buffer a decoder fills, plus a little room.
    limits.max_alloc = Some(pixels.saturating_mul(8).saturating_add(1 << 20));
    let mut decoder = reader(bytes);
    decoder.limits(limits);
    let rgba = decoder.decode().map_err(unreadable)?.to_rgba8();
    let pixels = rgba
        .pixels()
        .map(|p| {
            let [red, green, blue, alpha] = p.0;
            // Composite onto white.
            let over_white = |channel: u8| {
                let blended =
                    u32::from(channel) * u32::from(alpha) + 255 * (255 - u32::from(alpha));
                u8::try_from((blended + 127) / 255).unwrap_or(u8::MAX)
            };
            [over_white(red), over_white(green), over_white(blue)]
        })
        .collect();
    Raster::from_rgb(width, height, pixels)
}

/// A straight axis line: where it lies across its direction (a pixel-centre coordinate,
/// the middle of a line several pixels thick) and where it starts and ends along it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AxisLine {
    /// The line's position across its direction (y for the x axis, x for the y axis).
    pub position: f64,
    /// The first pixel edge along the line.
    pub start: f64,
    /// The last pixel edge along the line.
    pub end: f64,
}

/// A rectangle in pixel coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PixelRect {
    /// The corner with the smaller coordinates.
    pub min: [f64; 2],
    /// The corner with the larger coordinates.
    pub max: [f64; 2],
}

/// The axis lines and plot area found in an image.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Axes {
    /// The x axis: the lowest dark horizontal line spanning half the width.
    pub x: Option<AxisLine>,
    /// The y axis: the leftmost dark vertical line spanning half the height.
    pub y: Option<AxisLine>,
    /// The plot area between them, when both are found.
    pub plot: Option<PixelRect>,
}

/// The longest run of dark pixels along a line, as `(start, length)`.
fn longest_run(len: u32, dark: impl Fn(u32) -> bool) -> (u32, u32) {
    let (mut best, mut start, mut run) = ((0, 0), 0, 0);
    for i in 0..len {
        if dark(i) {
            if run == 0 {
                start = i;
            }
            run += 1;
            if run > best.1 {
                best = (start, run);
            }
        } else {
            run = 0;
        }
    }
    best
}

/// The outermost group of adjacent lines whose dark run spans half of `along`:
/// `lines` is the number of lines, `last` picks the group with the highest index (the
/// bottom row) instead of the lowest (the left column).
fn outer_line(
    lines: u32,
    along: u32,
    last: bool,
    run: impl Fn(u32) -> (u32, u32),
) -> Option<AxisLine> {
    let needed = u64::from(along.div_ceil(2));
    let long = |i: u32| u64::from(run(i).1) >= needed.max(2);
    let order: Box<dyn Iterator<Item = u32>> = if last {
        Box::new((0..lines).rev())
    } else {
        Box::new(0..lines)
    };
    let mut group: Vec<u32> = Vec::new();
    for i in order {
        if long(i) {
            group.push(i);
        } else if !group.is_empty() {
            break;
        }
    }
    let first = *group.iter().min()?;
    let lastline = *group.iter().max()?;
    // The run along the line, widest over the group.
    let (start, end) = group
        .iter()
        .map(|&i| run(i))
        .fold((u32::MAX, 0), |(s, e), (rs, rl)| {
            (s.min(rs), e.max(rs + rl))
        });
    Some(AxisLine {
        position: (f64::from(first) + f64::from(lastline) + 1.0) / 2.0,
        start: f64::from(start),
        end: f64::from(end),
    })
}

/// Finds the axis lines and the plot area (SONARITHM's own method; the documentation has
/// the user mark them). The x axis is the lowest group of dark rows whose longest dark run
/// spans at least half the width; the y axis is the leftmost such group of columns. The plot
/// area runs from the y axis to the end of the x axis, and from the top of the y axis down
/// to the x axis.
///
/// # Errors
///
/// `SNR-ENG-305` over the work budget (two passes over the pixels).
pub fn find_axes(raster: &Raster, work: &mut Work) -> Result<Axes, Box<Diagnostic>> {
    work.spend(u64::from(raster.width) * u64::from(raster.height) * 2)?;
    let row_run = |y: u32| longest_run(raster.width, |x| raster.dark(x, y));
    let column_run = |x: u32| longest_run(raster.height, |y| raster.dark(x, y));
    let x = outer_line(raster.height, raster.width, true, row_run);
    let y = outer_line(raster.width, raster.height, false, column_run);
    let plot = match (x, y) {
        (Some(x), Some(y)) if x.end > y.position && y.start < x.position => Some(PixelRect {
            min: [y.position, y.start],
            max: [x.end, x.position],
        }),
        _ => None,
    };
    Ok(Axes { x, y, plot })
}

/// A value read from a pixel, with the range of values half a pixel either side of it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Estimate {
    /// The value at the position.
    pub value: f64,
    /// The smaller end of the half-pixel range.
    pub low: f64,
    /// The larger end of the half-pixel range.
    pub high: f64,
}

impl Estimate {
    /// Whether `value` lies in the half-pixel range.
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        let tolerance = 1e-9 * (self.high - self.low).abs().max(value.abs()).max(1.0);
        value >= self.low - tolerance && value <= self.high + tolerance
    }

    /// Half the range's width: the half-pixel uncertainty in value units.
    #[must_use]
    pub fn uncertainty(&self) -> f64 {
        (self.high - self.low) / 2.0
    }
}

/// An x and a y axis calibrated from anchors in pixels.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RasterCalibration {
    /// The horizontal axis: anchor x positions to values. On a category axis the values are
    /// the labels' indices in position order (0, 1, 2, …), fitted linearly.
    pub x: Calibration,
    /// The vertical axis: anchor y positions to values.
    pub y: Calibration,
    /// The x axis's labels in position order, when it is a category axis.
    pub x_labels: Option<Vec<String>>,
}

/// One axis from the spec's anchors: the anchors' x (or y) pixel coordinates and values,
/// and for a category axis its labels in position order.
fn axis(
    axis: &CalibrationAxis,
    name: &str,
    coordinate: usize,
) -> Result<(Calibration, Option<Vec<String>>), Box<Diagnostic>> {
    let bad = |why: String| {
        Box::new(Diagnostic::new(
            &codes::CALIBRATION_ANCHORS,
            format!("The {name} axis cannot be calibrated: {why}."),
        ))
    };
    let kind = match axis.kind {
        AxisKind::Linear => ScaleKind::Linear,
        AxisKind::Log => ScaleKind::Log,
        AxisKind::Date => ScaleKind::Date,
        AxisKind::Category if coordinate == 0 => return category(axis, &bad),
        _ => {
            return Err(bad(
                "only the x axis can be a category axis; the y axis needs values".to_owned(),
            ));
        }
    };
    let mut anchors = Vec::with_capacity(axis.anchors.len());
    for anchor in &axis.anchors {
        let position = anchor.at.get(coordinate).copied().unwrap_or(f64::NAN);
        let value = match (&anchor.value, kind) {
            (AnchorValue::Number(n), ScaleKind::Linear | ScaleKind::Log) => *n,
            (AnchorValue::Text(text), ScaleKind::Date) => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "milliseconds since 1970 fit f64"
                )]
                let ms =
                    parse_date(text).ok_or_else(|| bad(format!("`{text}` is not a date")))? as f64;
                ms
            }
            (other, _) => {
                return Err(bad(format!(
                    "the anchor value {other:?} does not fit a {kind:?} axis"
                )));
            }
        };
        anchors.push(Anchor::new(position, value));
    }
    Ok((Calibration::fit(kind, &anchors)?, None))
}

/// A category x axis: every label is an anchor (the user names the categories; no OCR), and
/// the labels' indices in position order are fitted linearly to their positions, so a
/// position reads as a fractional index (our design; the documentation calibrates bar
/// charts on the value axis only).
fn category(
    axis: &CalibrationAxis,
    bad: &dyn Fn(String) -> Box<Diagnostic>,
) -> Result<(Calibration, Option<Vec<String>>), Box<Diagnostic>> {
    let mut labels = Vec::with_capacity(axis.anchors.len());
    for anchor in &axis.anchors {
        let AnchorValue::Text(label) = &anchor.value else {
            return Err(bad("a category anchor needs a label".to_owned()));
        };
        labels.push((anchor.at[0], label.clone()));
    }
    labels.sort_by(|a, b| a.0.total_cmp(&b.0));
    #[expect(clippy::cast_precision_loss, reason = "label counts are small")]
    let anchors: Vec<Anchor> = labels
        .iter()
        .enumerate()
        .map(|(i, (position, _))| Anchor::new(*position, i as f64))
        .collect();
    let calibration = Calibration::fit(ScaleKind::Linear, &anchors)?;
    Ok((
        calibration,
        Some(labels.into_iter().map(|(_, l)| l).collect()),
    ))
}

impl RasterCalibration {
    /// The calibration a spec gives (`data.source.calibration`, RFC-0004 §5): every anchor
    /// entered by hand, in pixels. The x axis reads the anchors' x coordinates and the y
    /// axis their y coordinates.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-320` for a category y axis, an anchor value of the wrong type or too few
    /// distinct anchors; `SNR-ENG-321` for a log axis with a value that is not positive.
    pub fn from_spec(calibration: &SpecCalibration) -> Result<Self, Box<Diagnostic>> {
        let (x, x_labels) = axis(&calibration.axes.x, "x", 0)?;
        let (y, _) = axis(&calibration.axes.y, "y", 1)?;
        Ok(Self { x, y, x_labels })
    }

    /// The category at horizontal position `px`: the label whose index is nearest, when it
    /// is within half a step. `None` on a continuous axis.
    #[must_use]
    pub fn label_at(&self, px: f64) -> Option<&str> {
        let labels = self.x_labels.as_ref()?;
        let index = self.x.value_at(px).round();
        if index < 0.0 {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "not negative; an index past the labels finds none"
        )]
        let at = index as usize;
        labels.get(at).map(String::as_str)
    }

    /// The labels' positions in pixels, on a category axis.
    #[must_use]
    pub fn label_positions(&self) -> Vec<f64> {
        let count = self.x_labels.as_ref().map_or(0, Vec::len);
        #[expect(clippy::cast_precision_loss, reason = "label counts are small")]
        let positions = (0..count)
            .filter_map(|i| self.x.position_of(i as f64))
            .collect();
        positions
    }

    /// The values at the pixel position `(px, py)` (for a pixel's centre, `i + 0.5`), each
    /// with the range half a pixel either side.
    #[must_use]
    pub fn value_at(&self, px: f64, py: f64) -> (Estimate, Estimate) {
        (estimate(&self.x, px), estimate(&self.y, py))
    }

    /// The values at the centre of pixel `(x, y)`.
    #[must_use]
    pub fn value_at_pixel(&self, x: u32, y: u32) -> (Estimate, Estimate) {
        self.value_at(f64::from(x) + 0.5, f64::from(y) + 0.5)
    }
}

fn estimate(axis: &Calibration, position: f64) -> Estimate {
    let (a, b) = (axis.value_at(position - 0.5), axis.value_at(position + 0.5));
    Estimate {
        value: axis.value_at(position),
        low: a.min(b),
        high: a.max(b),
    }
}

#[cfg(test)]
mod tests;
