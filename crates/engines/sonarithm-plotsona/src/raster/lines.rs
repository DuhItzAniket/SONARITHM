// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Line extraction (C02-T03).
//!
//! **Clean room (ADR-0022).** WPD's user documentation (register CR-01) names the
//! algorithms and their parameters without describing how they work: the averaging window
//! ("for continuous curves and evenly spaced points", a moving window of ∆X by ∆Y sized to
//! the curve's thickness), X step ("regular X intervals", a line width), X step with
//! interpolation (the same with cubic spline interpolation) and custom independents (X values
//! the user lists). How each one groups pixels, picks among several runs in a column and
//! weights them is SONARITHM's design, described on each function. The natural cubic spline
//! is the textbook construction (register CR-07).

use serde::Serialize;

use super::colour::{ColourFilter, Mask, pixel_range};
use super::{PixelRect, Raster};

/// A point found on a curve, in pixel coordinates (centres at +0.5).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PixelPoint {
    /// The horizontal position.
    pub x: f64,
    /// The vertical position.
    pub y: f64,
    /// How closely its pixels match the colour, in (0, 1]: the mean of their closeness.
    pub closeness: f64,
    /// Whether the point was interpolated rather than seen.
    pub interpolated: bool,
}

/// Where a search looks: an image, a colour filter, the plot area and a mask.
#[derive(Clone, Copy, Debug)]
pub struct Search<'a> {
    /// The image.
    pub raster: &'a Raster,
    /// Which pixels are data.
    pub filter: ColourFilter,
    /// The plot area: pixels outside it are never data.
    pub area: PixelRect,
    /// The user's masks; empty keeps the whole plot area.
    pub mask: &'a Mask,
}

impl Search<'_> {
    /// The closeness of pixel `(x, y)`, if it is data.
    fn keep(&self, x: u32, y: u32) -> Option<f64> {
        if !self.mask.is_empty() && !self.mask.contains(x, y) {
            return None;
        }
        self.filter.closeness(self.raster.pixel(x, y)?)
    }

    /// The data pixels of columns `x0..x1` in the plot area, as `(x, y, closeness)` with
    /// pixel-centre coordinates, sorted by `y`.
    fn columns(&self, x0: u32, x1: u32) -> Vec<(f64, f64, f64)> {
        let (ax0, ax1, y0, y1) = pixel_range(self.raster, &self.area);
        let mut found = Vec::new();
        for x in x0.max(ax0)..x1.min(ax1) {
            for y in y0..y1 {
                if let Some(c) = self.keep(x, y) {
                    found.push((f64::from(x) + 0.5, f64::from(y) + 0.5, c));
                }
            }
        }
        found.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.total_cmp(&b.0)));
        found
    }
}

/// Splits pixels sorted by `y` where consecutive `y` values are more than `gap` apart.
fn runs(pixels: &[(f64, f64, f64)], gap: f64) -> Vec<&[(f64, f64, f64)]> {
    let mut out = Vec::new();
    let mut start = 0;
    for i in 1..=pixels.len() {
        if i == pixels.len() || pixels[i].1 - pixels[i - 1].1 > gap {
            if i > start {
                out.push(&pixels[start..i]);
            }
            start = i;
        }
    }
    out
}

/// The closeness-weighted centre of some pixels.
fn centre(pixels: &[(f64, f64, f64)]) -> PixelPoint {
    let weight: f64 = pixels.iter().map(|p| p.2).sum();
    let mean =
        |f: fn(&(f64, f64, f64)) -> f64| pixels.iter().map(|p| f(p) * p.2).sum::<f64>() / weight;
    #[expect(clippy::cast_precision_loss, reason = "pixel counts are small")]
    let closeness = weight / pixels.len() as f64;
    PixelPoint {
        x: mean(|p| p.0),
        y: mean(|p| p.1),
        closeness,
        interpolated: false,
    }
}

/// The averaging window (CR-01): the plot area is cut into column strips `dx` pixels wide;
/// in each strip the data pixels, sorted by height, are split wherever two are more than
/// `dy` apart, and each part gives one point at its closeness-weighted centre. A curve gives
/// one point per strip; points of several curves of one colour, or a curve's steep part,
/// give several.
#[must_use]
pub fn averaging_window(search: &Search<'_>, dx: u32, dy: u32) -> Vec<PixelPoint> {
    let (x0, x1, _, _) = pixel_range(search.raster, &search.area);
    let step = dx.max(1);
    let mut points = Vec::new();
    let mut x = x0;
    while x < x1 {
        let pixels = search.columns(x, x.saturating_add(step));
        points.extend(runs(&pixels, f64::from(dy.max(1))).into_iter().map(centre));
        x = x.saturating_add(step);
    }
    points
}

/// Samples a curve at horizontal pixel positions (X step and custom independents, CR-01):
/// at each position the data pixels within `width / 2` of it (at least its own column) are
/// split into runs as in the averaging window, with `width` as the gap, and one run is the
/// curve: the one nearest the previous point's height, or the largest when there is none.
/// A position with no data pixels gives `None`. With `interpolate`, the gaps between found
/// points are filled by a natural cubic spline through them ([`spline`]) and marked as
/// interpolated; gaps before the first or after the last found point stay empty.
#[must_use]
pub fn x_step(
    search: &Search<'_>,
    positions: &[f64],
    width: f64,
    interpolate: bool,
) -> Vec<Option<PixelPoint>> {
    let half = (width / 2.0).max(0.5);
    let mut previous: Option<f64> = None;
    let mut points: Vec<Option<PixelPoint>> = positions
        .iter()
        .map(|&p| {
            let column = |v: f64| {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "clamped at 0"
                )]
                let c = v.floor().max(0.0) as u32;
                c
            };
            let pixels = search.columns(column(p - half), column(p + half).saturating_add(1));
            let chosen = runs(&pixels, width.max(1.0))
                .into_iter()
                .min_by(|a, b| match previous {
                    Some(y) => (centre(a).y - y).abs().total_cmp(&(centre(b).y - y).abs()),
                    None => b.len().cmp(&a.len()),
                });
            let point = chosen.map(|run| PixelPoint {
                x: p,
                ..centre(run)
            });
            if let Some(point) = point {
                previous = Some(point.y);
            }
            point
        })
        .collect();
    if interpolate {
        let known: Vec<(f64, f64)> = points.iter().flatten().map(|p| (p.x, p.y)).collect();
        if let Some(curve) = spline(&known) {
            let (first, last) = (known[0].0, known[known.len() - 1].0);
            for (slot, &x) in points.iter_mut().zip(positions) {
                if slot.is_none() && x > first && x < last {
                    *slot = Some(PixelPoint {
                        x,
                        y: curve(x),
                        closeness: 0.5,
                        interpolated: true,
                    });
                }
            }
        }
    }
    points
}

/// A natural cubic spline through points with increasing `x` (register CR-07): the second
/// derivative is 0 at both ends and continuous inside, found by solving the tridiagonal
/// system for the second derivatives. `None` with fewer than two points or `x` not
/// strictly increasing.
#[must_use]
#[expect(
    clippy::many_single_char_names,
    reason = "the textbook's names: n points, steps h, second derivatives m, weights a and b"
)]
pub fn spline(points: &[(f64, f64)]) -> Option<impl Fn(f64) -> f64 + use<>> {
    let n = points.len();
    if n < 2 || points.windows(2).any(|w| w[1].0 <= w[0].0) {
        return None;
    }
    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let h: Vec<f64> = xs.windows(2).map(|w| w[1] - w[0]).collect();
    // Second derivatives m[0..n]; m[0] = m[n-1] = 0. Thomas algorithm on the inner rows.
    let mut m = vec![0.0; n];
    if n > 2 {
        let inner = n - 2;
        let mut diag = vec![0.0; inner];
        let mut rhs = vec![0.0; inner];
        for i in 0..inner {
            diag[i] = 2.0 * (h[i] + h[i + 1]);
            rhs[i] = 6.0 * ((ys[i + 2] - ys[i + 1]) / h[i + 1] - (ys[i + 1] - ys[i]) / h[i]);
        }
        for i in 1..inner {
            let factor = h[i] / diag[i - 1];
            diag[i] -= factor * h[i];
            rhs[i] -= factor * rhs[i - 1];
        }
        for i in (0..inner).rev() {
            let next = if i + 1 < inner { m[i + 2] } else { 0.0 };
            m[i + 1] = (rhs[i] - h[i + 1] * next) / diag[i];
        }
    }
    Some(move |x: f64| {
        let i = xs.partition_point(|&v| v <= x).clamp(1, n - 1) - 1;
        let (x0, x1, hi) = (xs[i], xs[i + 1], h[i]);
        let (a, b) = ((x1 - x) / hi, (x - x0) / hi);
        a * ys[i]
            + b * ys[i + 1]
            + ((a * a * a - a) * m[i] + (b * b * b - b) * m[i + 1]) * hi * hi / 6.0
    })
}
