// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bars, histogram bins and point markers (C02-T04), read from connected components.
//!
//! **Clean room (ADR-0022).** WPD's user documentation (register CR-01) describes a bar
//! extraction with a step and a value tolerance, and a blob detector with a minimum and a
//! maximum diameter; it does not say how they work. How bars are profiled and split, which
//! end of a bar is its value, how a blob is accepted and how touching markers are separated
//! is SONARITHM's design, described on each function. The k-means step that separates
//! touching markers is Lloyd's textbook algorithm (register CR-08).

use serde::Serialize;

use super::components::{Component, Labels};
use super::lines::Search;

/// A bar or a histogram bin, in pixel coordinates (boundaries, not centres).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Bar {
    /// The left edge.
    pub left: f64,
    /// The right edge.
    pub right: f64,
    /// The edge that gives the value: the end away from the baseline.
    pub edge: f64,
    /// The other end, which a bar stands on (or hangs from).
    pub base: f64,
    /// The mean colour closeness of its pixels.
    pub closeness: f64,
}

impl Bar {
    /// The horizontal centre.
    #[must_use]
    pub fn centre(&self) -> f64 {
        self.left.midpoint(self.right)
    }

    /// The area in pixels.
    #[must_use]
    pub fn area(&self) -> f64 {
        (self.right - self.left) * (self.edge - self.base).abs()
    }
}

/// One column's extent within a component: its top and bottom pixels.
#[derive(Clone, Copy)]
struct Column {
    x: u32,
    top: u32,
    bottom: u32,
}

/// The vertical bars in the components (CR-01's bar extraction, our design). Each
/// component is profiled column by column (its top and bottom pixel in each column); one
/// with holes in its columns (under 90 % of the spans filled, as in hollow markers) is not
/// a bar. The columns are split into bars wherever a column is missing (the step
/// tolerance: none) or the top or bottom moves by more than `tolerance` pixels (the value
/// tolerance), which separates touching histogram bins of different heights; bins of equal
/// height that touch stay one bar. The value is read at the bar's end farther from
/// `baseline` (the pixel row of value 0, clamped into the plot area): its top for a bar
/// standing on the baseline, its bottom for one hanging from it. Bars narrower than 2
/// columns are anti-aliasing and dropped.
#[must_use]
pub fn bars(labels: &Labels, baseline: f64, tolerance: f64) -> Vec<Bar> {
    let mut out = Vec::new();
    for (index, c) in labels.components.iter().enumerate() {
        if c.width() < 2 {
            continue;
        }
        let mut columns: Vec<Column> = Vec::new();
        let mut spans = 0_u64;
        for x in c.min[0]..=c.max[0] {
            let rows = (c.min[1]..=c.max[1]).filter(|&y| labels.at(x, y) == Some(index));
            let (mut top, mut bottom) = (None, None);
            for y in rows {
                top.get_or_insert(y);
                bottom = Some(y);
            }
            if let (Some(top), Some(bottom)) = (top, bottom) {
                spans += u64::from(bottom - top + 1);
                columns.push(Column { x, top, bottom });
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "pixel counts")]
        let solid = f64::from(c.pixels) / spans.max(1) as f64;
        if solid < 0.9 {
            continue;
        }
        let mut start = 0;
        for i in 1..=columns.len() {
            let split = i == columns.len() || {
                let (a, b) = (columns[i - 1], columns[i]);
                b.x != a.x + 1
                    || f64::from(b.top.abs_diff(a.top)) > tolerance
                    || f64::from(b.bottom.abs_diff(a.bottom)) > tolerance
            };
            if !split {
                continue;
            }
            let run = &columns[start..i];
            start = i;
            if run.len() < 2 {
                continue;
            }
            let median = |f: fn(&Column) -> u32| {
                let mut v: Vec<u32> = run.iter().map(f).collect();
                v.sort_unstable();
                v[v.len() / 2]
            };
            let top = f64::from(median(|c| c.top));
            let bottom = f64::from(median(|c| c.bottom)) + 1.0;
            let (edge, base) = if (top - baseline).abs() >= (bottom - baseline).abs() {
                (top, bottom)
            } else {
                (bottom, top)
            };
            out.push(Bar {
                left: f64::from(run[0].x),
                right: f64::from(run[run.len() - 1].x) + 1.0,
                edge,
                base,
                closeness: c.closeness(),
            });
        }
    }
    out.sort_by(|a, b| a.left.total_cmp(&b.left));
    out
}

/// A point marker found in the image.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Marker {
    /// Its centre, in pixel coordinates.
    pub at: [f64; 2],
    /// The mean colour closeness of its pixels.
    pub closeness: f64,
    /// How many markers its blob held: 1, or more where markers touch and were separated.
    pub merged: u32,
}

/// The diameter of a component: the longer side of its bounding box.
fn diameter(c: &Component) -> u32 {
    c.width().max(c.height())
}

/// The point markers among the components (CR-01's blob detector, our design): a component
/// whose diameter lies within `min..=max` pixels is one marker at its closeness-weighted
/// centre. A larger component (up to four times `max`) whose area is 1.5 to 6 times the
/// median single marker's is that many touching markers, round(area / median), separated
/// by k-means on its pixels (CR-08) and marked as merged. A marker-sized component under
/// half the median area is a fragment (anti-aliasing breaks thin hollow markers into
/// pieces): fragments whose centres lie within the median diameter of each other are one
/// marker, at their pixel-weighted centre with half their closeness, and a group under 40 %
/// of the median area is a speck and dropped. Everything else (lines, text, legends) is
/// not a marker.
#[must_use]
pub fn markers(labels: &Labels, search: &Search<'_>, min: u32, max: u32) -> Vec<Marker> {
    let single = |c: &Component| (min..=max).contains(&diameter(c));
    let mut areas: Vec<u32> = labels
        .components
        .iter()
        .filter(|c| single(c))
        .map(|c| c.pixels)
        .collect();
    areas.sort_unstable();
    let median = areas.get(areas.len() / 2).copied().unwrap_or(0);
    let mut out = Vec::new();
    let fragment = |c: &Component| 2 * c.pixels < median;
    let mut diameters: Vec<u32> = labels
        .components
        .iter()
        .filter(|c| single(c) && !fragment(c))
        .map(diameter)
        .collect();
    diameters.sort_unstable();
    let reach = f64::from(diameters.get(diameters.len() / 2).copied().unwrap_or(max));
    let fragments: Vec<&Component> = labels
        .components
        .iter()
        .filter(|c| single(c) && fragment(c))
        .collect();
    out.extend(group_fragments(&fragments, reach, median));
    for (index, c) in labels.components.iter().enumerate() {
        if single(c) && fragment(c) {
            continue;
        }
        if single(c) {
            out.push(Marker {
                at: c.centroid(),
                closeness: c.closeness(),
                merged: 1,
            });
            continue;
        }
        if median == 0 || diameter(c) > max.saturating_mul(4) {
            continue;
        }
        let ratio = f64::from(c.pixels) / f64::from(median);
        if !(1.5..=6.0).contains(&ratio) {
            continue;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "1.5 to 6"
        )]
        let k = ratio.round() as usize;
        let pixels = labels.pixels_of(index, search);
        for at in k_means(&pixels, k, c) {
            #[expect(clippy::cast_possible_truncation, reason = "at most 6")]
            let merged = k as u32;
            out.push(Marker {
                at,
                closeness: c.closeness(),
                merged,
            });
        }
    }
    out.sort_by(|a, b| {
        a.at[0]
            .total_cmp(&b.at[0])
            .then(a.at[1].total_cmp(&b.at[1]))
    });
    out
}

/// Groups fragments whose centres are within `reach` of each other (transitively) into
/// one marker each; groups under 40 % of the `median` marker area are dropped.
fn group_fragments(fragments: &[&Component], reach: f64, median: u32) -> Vec<Marker> {
    let mut group: Vec<usize> = (0..fragments.len()).collect();
    for i in 0..fragments.len() {
        for j in 0..i {
            let (a, b) = (fragments[i].centroid(), fragments[j].centroid());
            if (a[0] - b[0]).hypot(a[1] - b[1]) <= reach {
                let (gi, gj) = (group[i], group[j]);
                for g in &mut group {
                    if *g == gi {
                        *g = gj;
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    let mut ids: Vec<usize> = group.clone();
    ids.sort_unstable();
    ids.dedup();
    for id in ids {
        let members: Vec<&Component> = fragments
            .iter()
            .zip(&group)
            .filter(|(_, g)| **g == id)
            .map(|(c, _)| *c)
            .collect();
        let pixels: u32 = members.iter().map(|c| c.pixels).sum();
        if 5 * pixels < 2 * median {
            continue;
        }
        let weight = f64::from(pixels);
        let at = members.iter().fold([0.0, 0.0], |acc, c| {
            let p = c.centroid();
            let w = f64::from(c.pixels) / weight;
            [acc[0] + p[0] * w, acc[1] + p[1] * w]
        });
        let closeness = members
            .iter()
            .map(|c| c.closeness() * f64::from(c.pixels))
            .sum::<f64>()
            / weight;
        out.push(Marker {
            at,
            closeness: closeness / 2.0,
            merged: 1,
        });
    }
    out
}

/// Lloyd's k-means on weighted pixels (CR-08), started from `k` centres spread evenly along
/// the longer side of the component's box through its centroid, ten rounds: deterministic.
fn k_means(pixels: &[([f64; 2], f64)], k: usize, c: &Component) -> Vec<[f64; 2]> {
    let centre = c.centroid();
    let along = usize::from(c.width() < c.height());
    let (lo, hi) = (f64::from(c.min[along]), f64::from(c.max[along]) + 1.0);
    #[expect(clippy::cast_precision_loss, reason = "k is at most 6")]
    let mut centres: Vec<[f64; 2]> = (0..k)
        .map(|i| {
            let mut p = centre;
            p[along] = lo + (hi - lo) * (i as f64 + 0.5) / k as f64;
            p
        })
        .collect();
    for _ in 0..10 {
        let mut sums = vec![([0.0_f64; 2], 0.0_f64); k];
        for (p, w) in pixels {
            let nearest = (0..k)
                .min_by(|&a, &b| {
                    let d = |q: [f64; 2]| (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2);
                    d(centres[a]).total_cmp(&d(centres[b]))
                })
                .unwrap_or(0);
            sums[nearest].0[0] += p[0] * w;
            sums[nearest].0[1] += p[1] * w;
            sums[nearest].1 += w;
        }
        for (centre, (sum, weight)) in centres.iter_mut().zip(&sums) {
            if *weight > 0.0 {
                *centre = [sum[0] / weight, sum[1] / weight];
            }
        }
    }
    centres
}

#[cfg(test)]
mod tests;
