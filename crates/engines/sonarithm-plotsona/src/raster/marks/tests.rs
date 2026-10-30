// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "tests"
)]

use proptest::prelude::*;

use super::super::colour::{ColourFilter, Mask};
use super::super::components::label;
use super::super::lines::Search;
use super::super::{PixelRect, Raster};
use super::{bars, markers};
use crate::budget::Work;

const BLUE: [u8; 3] = [40, 90, 200];

/// A white `width × height` image where `paint(x, y)` pixels are blue.
fn image(width: u32, height: u32, paint: &dyn Fn(u32, u32) -> bool) -> Raster {
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| if paint(x, y) { BLUE } else { [255; 3] })
        .collect();
    Raster::from_rgb(width, height, pixels).unwrap()
}

fn read<T>(
    raster: &Raster,
    f: impl FnOnce(&super::super::components::Labels, &Search<'_>) -> T,
) -> T {
    let mask = Mask::default();
    let search = Search {
        raster,
        filter: ColourFilter::foreground(BLUE),
        area: PixelRect {
            min: [0.0, 0.0],
            max: [f64::from(raster.width()), f64::from(raster.height())],
        },
        mask: &mask,
    };
    let labels = label(&search, &mut Work::new(u64::MAX)).unwrap();
    f(&labels, &search)
}

#[test]
fn components_join_diagonal_neighbours_and_keep_apart_what_does_not_touch() {
    // A diagonal staircase (one component), a separate square, and a lone pixel.
    let raster = image(40, 20, &|x, y| {
        (x < 8 && x == y) || (20..25).contains(&x) && (5..10).contains(&y) || (x, y) == (35, 15)
    });
    read(&raster, |labels, _| {
        assert_eq!(labels.components.len(), 3);
        let stairs = labels.components[0];
        assert_eq!((stairs.pixels, stairs.min, stairs.max), (8, [0, 0], [7, 7]));
        let square = labels.components[1];
        assert_eq!((square.pixels, square.width(), square.height()), (25, 5, 5));
        assert_eq!(square.fill(), 1.0);
        assert_eq!(square.centroid(), [22.5, 7.5]);
        assert_eq!(labels.at(22, 7), Some(1));
        assert_eq!(labels.at(30, 7), None);
    });
}

#[test]
fn bars_read_the_end_away_from_the_baseline() {
    // Baseline at row 50: two bars standing on it (tops 10 and 30), one hanging to row 70.
    let raster = image(100, 80, &|x, y| {
        ((10..20).contains(&x) && (10..50).contains(&y))
            || ((40..50).contains(&x) && (30..50).contains(&y))
            || ((70..80).contains(&x) && (50..70).contains(&y))
    });
    let found = read(&raster, |labels, _| bars(labels, 50.0, 1.5));
    assert_eq!(found.len(), 3);
    assert_eq!(
        (found[0].left, found[0].right, found[0].edge),
        (10.0, 20.0, 10.0)
    );
    assert_eq!(found[1].edge, 30.0);
    assert_eq!((found[2].centre(), found[2].edge), (75.0, 70.0));
}

#[test]
fn touching_histogram_bins_split_where_their_height_changes() {
    // Three touching bins, 10 columns each, tops at rows 20, 35 and 5.
    let raster = image(60, 60, &|x, y| {
        let top = match x {
            10..20 => 20,
            20..30 => 35,
            30..40 => 5,
            _ => return false,
        };
        (top..50).contains(&y)
    });
    let found = read(&raster, |labels, _| bars(labels, 50.0, 1.5));
    let edges: Vec<(f64, f64)> = found.iter().map(|b| (b.centre(), b.edge)).collect();
    assert_eq!(edges, vec![(15.0, 20.0), (25.0, 35.0), (35.0, 5.0)]);
}

#[test]
fn touching_markers_are_separated_and_counted() {
    let disc = |cx: f64, cy: f64, r: f64| {
        move |x: u32, y: u32| (f64::from(x) + 0.5 - cx).hypot(f64::from(y) + 0.5 - cy) <= r
    };
    let (a, b, c, d) = (
        disc(20.0, 20.0, 4.0),
        disc(60.0, 30.0, 4.0),
        disc(100.0, 20.0, 4.0),
        disc(106.0, 20.0, 4.0),
    );
    let raster = image(140, 50, &|x, y| a(x, y) || b(x, y) || c(x, y) || d(x, y));
    let found = read(&raster, |labels, search| markers(labels, search, 2, 12));
    assert_eq!(found.len(), 4, "{found:?}");
    assert_eq!(found.iter().filter(|m| m.merged == 2).count(), 2);
    for (m, x) in found.iter().zip([20.0, 60.0, 100.0, 106.0]) {
        assert!((m.at[0] - x).abs() <= 1.0, "{m:?}");
    }
}

#[test]
fn fragments_of_one_hollow_marker_count_once() {
    // Four rings, one of them broken into three arcs, plus a lone speck far away.
    let ring = |cx: f64, cy: f64| {
        move |x: u32, y: u32| {
            let d = (f64::from(x) + 0.5 - cx).hypot(f64::from(y) + 0.5 - cy);
            (2.5..=4.0).contains(&d)
        }
    };
    let (a, b, c, d) = (
        ring(15.0, 15.0),
        ring(45.0, 15.0),
        ring(75.0, 15.0),
        ring(105.0, 15.0),
    );
    let raster = image(140, 40, &|x, y| {
        let gap = (104..=106).contains(&x) || (y >= 16 && (100..=104).contains(&x));
        a(x, y) || b(x, y) || c(x, y) || (d(x, y) && !gap) || (x, y) == (130, 35)
    });
    let found = read(&raster, |labels, search| markers(labels, search, 2, 12));
    assert_eq!(found.len(), 4, "{found:?}");
    assert!((found[3].at[0] - 105.0).abs() <= 2.0, "{:?}", found[3]);
}

proptest! {
    /// C02-T04: a disc drawn anywhere is found as one marker within half a pixel of its
    /// centre, and a line is not a marker.
    #[test]
    fn a_drawn_disc_is_one_marker_at_its_centre(
        cx in 10.0_f64..90.0,
        cy in 10.0_f64..50.0,
        r in 2.0_f64..5.0,
    ) {
        let raster = image(100, 60, &|x, y| {
            (f64::from(x) + 0.5 - cx).hypot(f64::from(y) + 0.5 - cy) <= r || (y == 2 && x >= 5)
        });
        let found = read(&raster, |labels, search| markers(labels, search, 2, 12));
        prop_assert_eq!(found.len(), 1);
        prop_assert!((found[0].at[0] - cx).abs() <= 0.5 && (found[0].at[1] - cy).abs() <= 0.5, "{:?}", found[0]);
        prop_assert_eq!(found[0].merged, 1);
    }
}
