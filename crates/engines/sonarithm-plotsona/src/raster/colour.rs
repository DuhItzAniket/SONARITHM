// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Colour filters and masks (C02-T03).
//!
//! **Clean room (ADR-0022, register CR-01).** WPD's user documentation describes picking a
//! colour and saying whether it is the foreground (the marks) or the background (everything
//! else is data), with a colour-distance setting, and limiting the search to a region the
//! user paints with box and pen masks. It gives no distance measure, threshold or default.
//! SONARITHM's choices: Euclidean distance in 8-bit RGB, a default threshold of 70 (of
//! 441), and an automatic choice of series colours when the user names none
//! ([`series_colours`]), all our own design.

use std::collections::BTreeMap;

use serde::Serialize;

use super::{PixelRect, Raster};

/// Euclidean distance between two 8-bit RGB colours (0 to about 441).
#[must_use]
pub fn distance(a: [u8; 3], b: [u8; 3]) -> f64 {
    let d = |i: usize| f64::from(a[i]) - f64::from(b[i]);
    (d(0) * d(0) + d(1) * d(1) + d(2) * d(2)).sqrt()
}

/// The default colour-distance threshold: our choice, loose enough for anti-aliased lines'
/// cores, tight enough to leave their pale edges and other series alone.
pub const DEFAULT_THRESHOLD: f64 = 70.0;

/// Which pixels a filter keeps.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum ColourMode {
    /// Pixels close to this colour are the data.
    Foreground([u8; 3]),
    /// Pixels far from this colour (the background) are the data.
    Background([u8; 3]),
}

/// A colour filter: a mode and a distance threshold.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ColourFilter {
    /// The colour and what it means.
    pub mode: ColourMode,
    /// The colour distance that separates data from the rest.
    pub threshold: f64,
}

impl ColourFilter {
    /// A foreground filter for `colour` with the default threshold.
    #[must_use]
    pub const fn foreground(colour: [u8; 3]) -> Self {
        Self {
            mode: ColourMode::Foreground(colour),
            threshold: DEFAULT_THRESHOLD,
        }
    }

    /// How closely `colour` matches, in (0, 1], or `None` when it is not data. In foreground
    /// mode a pixel of exactly the colour is 1 and one at the threshold is close to 0; in
    /// background mode every kept pixel is 1, since the mode says nothing about how sure a
    /// pixel is.
    #[must_use]
    pub fn closeness(&self, colour: [u8; 3]) -> Option<f64> {
        let threshold = self.threshold.max(f64::MIN_POSITIVE);
        match self.mode {
            ColourMode::Foreground(target) => {
                let d = distance(colour, target);
                (d <= threshold).then(|| (1.0 - d / threshold).max(f64::EPSILON))
            }
            ColourMode::Background(background) => {
                (distance(colour, background) > threshold).then_some(1.0)
            }
        }
    }
}

/// A pen stroke: a polyline painted with a width, in pixel coordinates.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pen {
    /// The stroke's points.
    pub points: Vec<[f64; 2]>,
    /// The stroke's width in pixels.
    pub width: f64,
}

/// The region a search keeps: boxes and pen strokes (CR-01's box and pen masks). An empty
/// mask keeps everything inside the plot area.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Mask {
    /// Rectangles.
    pub boxes: Vec<PixelRect>,
    /// Pen strokes.
    pub pens: Vec<Pen>,
}

impl Mask {
    /// Whether the mask is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty() && self.pens.is_empty()
    }

    /// Whether the centre of pixel `(x, y)` lies in a box or within half a pen's width of
    /// its stroke.
    #[must_use]
    pub fn contains(&self, x: u32, y: u32) -> bool {
        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
        self.boxes
            .iter()
            .any(|b| px >= b.min[0] && px <= b.max[0] && py >= b.min[1] && py <= b.max[1])
            || self.pens.iter().any(|pen| {
                pen.points
                    .windows(2)
                    .any(|pair| segment_distance([px, py], pair[0], pair[1]) <= pen.width / 2.0)
                    || (pen.points.len() == 1
                        && segment_distance([px, py], pen.points[0], pen.points[0])
                            <= pen.width / 2.0)
            })
    }
}

fn segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (cx, cy) = (a[0] + t * dx, a[1] + t * dy);
    ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt()
}

/// The pixel range of a rectangle, clipped to the image: `(x0, x1, y0, y1)`, end exclusive.
pub(crate) fn pixel_range(raster: &Raster, area: &PixelRect) -> (u32, u32, u32, u32) {
    let clip = |v: f64, max: u32| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to the image size first"
        )]
        let v = v.clamp(0.0, f64::from(max)) as u32;
        v
    };
    (
        clip(area.min[0].floor(), raster.width()),
        clip(area.max[0].ceil(), raster.width()),
        clip(area.min[1].floor(), raster.height()),
        clip(area.max[1].ceil(), raster.height()),
    )
}

/// Whether a colour is greyish (white, black, grey: axes, grid, text, background): its
/// channels differ by less than 40, our choice.
fn greyish(c: [u8; 3]) -> bool {
    let (lo, hi) = (c.iter().min().copied(), c.iter().max().copied());
    hi.unwrap_or(0).saturating_sub(lo.unwrap_or(0)) < 40
}

/// Whether `c` is a paler tint of `base`: close to the segment from `base` to white, as
/// anti-aliased edges are.
fn tint_of(c: [u8; 3], base: [u8; 3]) -> bool {
    let point = [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])];
    let a = [f64::from(base[0]), f64::from(base[1]), f64::from(base[2])];
    let b = [255.0; 3];
    let ab: Vec<f64> = (0..3).map(|i| b[i] - a[i]).collect();
    let ap: Vec<f64> = (0..3).map(|i| point[i] - a[i]).collect();
    let length: f64 = ab.iter().map(|v| v * v).sum();
    let t = if length > 0.0 {
        (ab.iter().zip(&ap).map(|(u, v)| u * v).sum::<f64>() / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let off: f64 = (0..3)
        .map(|i| (ap[i] - t * ab[i]).powi(2))
        .sum::<f64>()
        .sqrt();
    off < 30.0
}

/// The distance from a colour to the nearest grey (the line where red, green and blue are
/// equal).
fn grey_distance(c: [u8; 3]) -> f64 {
    let v = c.map(f64::from);
    let mean = (v[0] + v[1] + v[2]) / 3.0;
    v.iter().map(|x| (x - mean).powi(2)).sum::<f64>().sqrt()
}

/// The threshold for a series colour found automatically: the default, but at most half
/// the distance to the nearest grey and to the nearest other series colour, so that axis
/// lines, grid lines and the other series stay out (our design; for example Vega's blue
/// lies 70 from its grey axis line).
#[must_use]
pub fn separating_threshold(colour: [u8; 3], others: &[[u8; 3]]) -> f64 {
    others
        .iter()
        .map(|other| distance(colour, *other) / 2.0)
        .fold(DEFAULT_THRESHOLD.min(grey_distance(colour) / 2.0), f64::min)
}

/// The series colours in `area`, most frequent first, at most `max` (our design: the
/// documentation has the user pick each colour). Greyish colours (axes, grid lines, text,
/// background) are skipped; colours are counted in buckets of 8 levels per channel, and a
/// bucket with fewer than 1 in 2,000 of the area's pixels (at least 12) is noise. A bucket
/// whose colour is a paler tint of one already chosen, or within 40 of it, is that series'
/// anti-aliasing and is skipped (distinct palette colours, such as Vega's orange and red,
/// lie about 80 apart).
#[must_use]
pub fn series_colours(raster: &Raster, area: &PixelRect, max: usize) -> Vec<[u8; 3]> {
    let (x0, x1, y0, y1) = pixel_range(raster, area);
    let mut buckets: BTreeMap<[u8; 3], (u64, [u64; 3])> = BTreeMap::new();
    let mut total = 0_u64;
    for y in y0..y1 {
        for x in x0..x1 {
            total += 1;
            let Some(c) = raster.pixel(x, y) else {
                continue;
            };
            if greyish(c) {
                continue;
            }
            let key = [c[0] >> 3, c[1] >> 3, c[2] >> 3];
            let entry = buckets.entry(key).or_insert((0, [0; 3]));
            entry.0 += 1;
            for (sum, channel) in entry.1.iter_mut().zip(c) {
                *sum += u64::from(channel);
            }
        }
    }
    let floor = (total / 2000).max(12);
    let mut ranked: Vec<(u64, [u8; 3])> = buckets
        .into_values()
        .filter(|(count, _)| *count >= floor)
        .map(|(count, sum)| {
            let mean = |i: usize| u8::try_from(sum[i] / count).unwrap_or(u8::MAX);
            (count, [mean(0), mean(1), mean(2)])
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut chosen: Vec<[u8; 3]> = Vec::new();
    for (_, colour) in ranked {
        if chosen.len() >= max {
            break;
        }
        let taken = chosen
            .iter()
            .any(|c| tint_of(colour, *c) || distance(colour, *c) < 40.0);
        if !taken {
            chosen.push(colour);
        }
    }
    chosen
}
