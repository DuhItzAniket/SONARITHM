// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Connected components of the pixels a search keeps (C02-T04): the shapes that bars and
//! markers are read from.
//!
//! **Clean room (ADR-0022).** The two-pass labelling with a union-find of provisional labels
//! is the textbook algorithm (register CR-08); WPD's documentation is not involved here.

use sonarithm_core::Diagnostic;

use super::colour::pixel_range;
use super::lines::Search;
use crate::budget::{Work, len64};

/// One 8-connected group of kept pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Component {
    /// The number of pixels.
    pub pixels: u32,
    /// The top-left pixel of the bounding box.
    pub min: [u32; 2],
    /// The bottom-right pixel of the bounding box (inclusive).
    pub max: [u32; 2],
    /// The sum of the pixels' closeness.
    weight: f64,
    /// The closeness-weighted sums of the pixel centres.
    sum: [f64; 2],
}

impl Component {
    /// The bounding box's width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.max[0] - self.min[0] + 1
    }

    /// The bounding box's height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.max[1] - self.min[1] + 1
    }

    /// The share of the bounding box the pixels fill, in (0, 1].
    #[must_use]
    pub fn fill(&self) -> f64 {
        f64::from(self.pixels) / (f64::from(self.width()) * f64::from(self.height()))
    }

    /// The closeness-weighted centre, in pixel coordinates (centres at +0.5).
    #[must_use]
    pub fn centroid(&self) -> [f64; 2] {
        [self.sum[0] / self.weight, self.sum[1] / self.weight]
    }

    /// The mean closeness of the pixels.
    #[must_use]
    pub fn closeness(&self) -> f64 {
        self.weight / f64::from(self.pixels)
    }
}

/// The components of a search's plot area, and which component each pixel belongs to.
#[derive(Clone, Debug)]
pub struct Labels {
    /// The area's first column and row.
    origin: [u32; 2],
    /// The area's size.
    size: [u32; 2],
    /// Per pixel, row by row: 0 for none, else the component's index + 1.
    cells: Vec<u32>,
    /// The components, in the order their first pixel is met (row by row).
    pub components: Vec<Component>,
}

impl Labels {
    /// The component index of pixel `(x, y)`, if it belongs to one.
    #[must_use]
    pub fn at(&self, x: u32, y: u32) -> Option<usize> {
        let (dx, dy) = (
            x.checked_sub(self.origin[0])?,
            y.checked_sub(self.origin[1])?,
        );
        if dx >= self.size[0] || dy >= self.size[1] {
            return None;
        }
        let index =
            usize::try_from(u64::from(dy) * u64::from(self.size[0]) + u64::from(dx)).ok()?;
        let label = *self.cells.get(index)?;
        usize::try_from(label).ok()?.checked_sub(1)
    }

    /// The pixels of component `index`, as pixel centres with their closeness.
    #[must_use]
    pub fn pixels_of(&self, index: usize, search: &Search<'_>) -> Vec<([f64; 2], f64)> {
        let Some(c) = self.components.get(index) else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(usize::try_from(c.pixels).unwrap_or(0));
        for y in c.min[1]..=c.max[1] {
            for x in c.min[0]..=c.max[0] {
                if self.at(x, y) == Some(index)
                    && let Some(closeness) = search.closeness(x, y)
                {
                    out.push(([f64::from(x) + 0.5, f64::from(y) + 0.5], closeness));
                }
            }
        }
        out
    }
}

fn find(parent: &mut [u32], mut a: u32) -> u32 {
    // Path halving keeps the trees shallow without recursion.
    loop {
        let Some(&p) = parent.get(a as usize) else {
            return a;
        };
        if p == a {
            return a;
        }
        let grand = parent.get(p as usize).copied().unwrap_or(p);
        if let Some(slot) = parent.get_mut(a as usize) {
            *slot = grand;
        }
        a = grand;
    }
}

fn union(parent: &mut [u32], a: u32, b: u32) {
    let (ra, rb) = (find(parent, a), find(parent, b));
    if ra != rb {
        // The smaller label is the root, so roots keep the order labels were made in.
        let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
        if let Some(slot) = parent.get_mut(hi as usize) {
            *slot = lo;
        }
    }
}

/// Labels the 8-connected components of the pixels `search` keeps in its plot area: a first
/// pass gives each pixel a provisional label from its west, north-west, north and
/// north-east neighbours and records which labels touch; a second resolves them (CR-08).
///
/// # Errors
///
/// `SNR-ENG-305` when the two passes exceed the work budget.
pub fn label(search: &Search<'_>, work: &mut Work) -> Result<Labels, Box<Diagnostic>> {
    let (x0, x1, y0, y1) = pixel_range(search.raster, &search.area);
    let size = [x1 - x0, y1 - y0];
    let count = usize::try_from(u64::from(size[0]) * u64::from(size[1])).unwrap_or(usize::MAX);
    work.spend(len64(count).saturating_mul(2))?;
    let width = size[0] as usize;
    let mut labels = vec![0_u32; count];
    // parent[0] is unused: label 0 means "no component".
    let mut parent: Vec<u32> = vec![0];
    for y in 0..size[1] as usize {
        for x in 0..width {
            #[expect(clippy::cast_possible_truncation, reason = "x and y index the image")]
            let (px, py) = (x0 + x as u32, y0 + y as u32);
            if search.closeness(px, py).is_none() {
                continue;
            }
            let mut neighbours = [0_u32; 4];
            if x > 0 {
                neighbours[0] = labels[y * width + x - 1];
            }
            if y > 0 {
                let above = (y - 1) * width;
                if x > 0 {
                    neighbours[1] = labels[above + x - 1];
                }
                neighbours[2] = labels[above + x];
                if x + 1 < width {
                    neighbours[3] = labels[above + x + 1];
                }
            }
            let first = neighbours.iter().copied().find(|&l| l != 0);
            let own = if let Some(first) = first {
                for &other in &neighbours {
                    if other != 0 && other != first {
                        union(&mut parent, first, other);
                    }
                }
                first
            } else {
                let next = u32::try_from(parent.len()).unwrap_or(u32::MAX);
                parent.push(next);
                next
            };
            labels[y * width + x] = own;
        }
    }
    // Second pass: roots to compact indices, statistics per component.
    let mut index_of_root = vec![0_u32; parent.len()];
    let mut components: Vec<Component> = Vec::new();
    for y in 0..size[1] as usize {
        for x in 0..width {
            let slot = &mut labels[y * width + x];
            if *slot == 0 {
                continue;
            }
            let root = find(&mut parent, *slot) as usize;
            if index_of_root[root] == 0 {
                index_of_root[root] = u32::try_from(components.len() + 1).unwrap_or(u32::MAX);
                components.push(Component {
                    pixels: 0,
                    min: [u32::MAX; 2],
                    max: [0; 2],
                    weight: 0.0,
                    sum: [0.0; 2],
                });
            }
            *slot = index_of_root[root];
            #[expect(clippy::cast_possible_truncation, reason = "x and y index the image")]
            let (px, py) = (x0 + x as u32, y0 + y as u32);
            let closeness = search.closeness(px, py).unwrap_or(0.0);
            if let Some(c) = components.get_mut(*slot as usize - 1) {
                c.pixels += 1;
                c.min = [c.min[0].min(px), c.min[1].min(py)];
                c.max = [c.max[0].max(px), c.max[1].max(py)];
                c.weight += closeness;
                c.sum[0] += (f64::from(px) + 0.5) * closeness;
                c.sum[1] += (f64::from(py) + 0.5) * closeness;
            }
        }
    }
    Ok(Labels {
        origin: [x0, y0],
        size,
        cells: labels,
        components,
    })
}
