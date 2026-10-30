// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "tests"
)]

use proptest::prelude::*;
use serde_json::{Value, json};
use sonarithm_core::ProvenanceClass;

use super::super::colour::{ColourFilter, Mask, Pen, series_colours};
use super::super::lines::{Search, averaging_window, spline, x_step};
use super::super::{PixelRect, Raster};
use crate::budget::Work;
use crate::corpus::folder;
use crate::model::XValue;
use crate::{ExtractOptions, extract};

const RED: [u8; 3] = [200, 40, 40];

/// A white image with a straight line `y = a + b·x` drawn `thick` pixels high in `colour`
/// across `x0..x1`, its pixels shifted by `noise(x, y)`.
fn line_image(
    size: (u32, u32),
    (a, b): (f64, f64),
    (x0, x1): (u32, u32),
    thick: u32,
    colour: [u8; 3],
    noise: &dyn Fn(u32, u32) -> [i16; 3],
) -> Raster {
    let (width, height) = size;
    let mut pixels = vec![[255_u8; 3]; (width * height) as usize];
    for x in x0..x1 {
        let centre = b.mul_add(f64::from(x) + 0.5, a);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "test sizes"
        )]
        let top = (centre - f64::from(thick) / 2.0).round().max(0.0) as u32;
        for y in top..(top + thick).min(height) {
            let n = noise(x, y);
            let shift = |c: u8, d: i16| u8::try_from((i16::from(c) + d).clamp(0, 255)).unwrap();
            pixels[(y * width + x) as usize] = [
                shift(colour[0], n[0]),
                shift(colour[1], n[1]),
                shift(colour[2], n[2]),
            ];
        }
    }
    Raster::from_rgb(width, height, pixels).unwrap()
}

fn whole(raster: &Raster) -> PixelRect {
    PixelRect {
        min: [0.0, 0.0],
        max: [f64::from(raster.width()), f64::from(raster.height())],
    }
}

#[test]
fn the_spline_passes_through_its_points_and_keeps_straight_lines_straight() {
    let points = [(0.0, 1.0), (1.0, 3.0), (2.5, 2.0), (4.0, 5.0)];
    let curve = spline(&points).unwrap();
    for (x, y) in points {
        assert!((curve(x) - y).abs() < 1e-9);
    }
    let line = spline(&[(0.0, 1.0), (2.0, 5.0), (3.0, 7.0), (7.0, 15.0)]).unwrap();
    for x in [0.5, 1.7, 4.2, 6.9] {
        assert!((line(x) - 2.0_f64.mul_add(x, 1.0)).abs() < 1e-9);
    }
    assert!(spline(&[(0.0, 1.0)]).is_none());
    assert!(spline(&[(1.0, 1.0), (1.0, 2.0)]).is_none());
}

#[test]
fn the_averaging_window_follows_a_drawn_line() {
    let raster = line_image((200, 120), (20.0, 0.4), (10, 190), 3, RED, &|_, _| [0; 3]);
    let mask = Mask::default();
    let search = Search {
        raster: &raster,
        filter: ColourFilter::foreground(RED),
        area: whole(&raster),
        mask: &mask,
    };
    let points = averaging_window(&search, 3, 3);
    assert!(points.len() >= 55);
    for p in &points {
        // Each pixel row is placed by rounding, so the centre is within half a pixel.
        assert!((p.y - 0.4_f64.mul_add(p.x, 20.0)).abs() <= 1.0, "{p:?}");
        assert!((p.closeness - 1.0).abs() < 1e-9);
    }
}

#[test]
fn filters_keep_close_colours_or_everything_off_the_background() {
    let red = ColourFilter::foreground(RED);
    assert_eq!(red.closeness(RED), Some(1.0));
    let near = red.closeness([210, 50, 45]).unwrap();
    assert!(near > 0.0 && near < 1.0);
    assert_eq!(red.closeness([40, 40, 200]), None);
    let background = ColourFilter {
        mode: super::super::colour::ColourMode::Background([255; 3]),
        threshold: 70.0,
    };
    assert_eq!(background.closeness([250, 250, 250]), None);
    assert_eq!(background.closeness([40, 40, 200]), Some(1.0));
    assert_eq!(background.closeness(RED), Some(1.0));
}

#[test]
fn masks_keep_only_what_they_cover() {
    let raster = line_image((200, 120), (60.0, 0.0), (0, 200), 3, RED, &|_, _| [0; 3]);
    let boxes = Mask {
        boxes: vec![PixelRect {
            min: [50.0, 0.0],
            max: [100.0, 120.0],
        }],
        pens: Vec::new(),
    };
    let search = Search {
        raster: &raster,
        filter: ColourFilter::foreground(RED),
        area: whole(&raster),
        mask: &boxes,
    };
    let points = averaging_window(&search, 5, 3);
    assert!(points.len() > 5);
    assert!(points.iter().all(|p| p.x >= 50.0 && p.x <= 100.0));
    let pen = Mask {
        boxes: Vec::new(),
        pens: vec![Pen {
            points: vec![[150.0, 0.0], [150.0, 120.0]],
            width: 4.0,
        }],
    };
    let search = Search {
        mask: &pen,
        ..search
    };
    let points = averaging_window(&search, 1, 3);
    assert!(points.len() >= 3);
    assert!(points.iter().all(|p| (p.x - 150.0).abs() <= 2.5));
}

#[test]
fn series_colours_skip_greys_and_anti_aliased_tints() {
    let mut raster = line_image((200, 120), (40.0, 0.0), (0, 200), 3, RED, &|_, _| [0; 3]);
    // A pale tint above the line (anti-aliasing) and a grey grid line below it.
    let mut pixels: Vec<[u8; 3]> = (0..120)
        .flat_map(|y| (0..200).map(move |x| (x, y)))
        .map(|(x, y)| raster.pixel(x, y).unwrap())
        .collect();
    for x in 0..200_usize {
        pixels[37 * 200 + x] = [235, 190, 190];
        pixels[90 * 200 + x] = [200, 200, 200];
    }
    raster = Raster::from_rgb(200, 120, pixels).unwrap();
    let colours = series_colours(&raster, &whole(&raster), 8);
    assert_eq!(colours.len(), 1, "{colours:?}");
    assert!(super::super::colour::distance(colours[0], RED) < 8.0);
}

#[test]
fn x_step_reads_categories_and_interpolates_gaps() {
    // The line is missing between x = 80 and x = 120.
    let raster = line_image((200, 120), (20.0, 0.4), (0, 200), 3, RED, &|_, _| [0; 3]);
    let mut pixels: Vec<[u8; 3]> = (0..120)
        .flat_map(|y| (0..200).map(move |x| (x, y)))
        .map(|(x, y)| raster.pixel(x, y).unwrap())
        .collect();
    for y in 0..120_usize {
        for x in 80..120 {
            pixels[y * 200 + x] = [255; 3];
        }
    }
    let raster = Raster::from_rgb(200, 120, pixels).unwrap();
    let mask = Mask::default();
    let search = Search {
        raster: &raster,
        filter: ColourFilter::foreground(RED),
        area: whole(&raster),
        mask: &mask,
    };
    let positions = [20.0, 60.0, 100.0, 140.0, 180.0];
    let points = x_step(&search, &positions, 3.0, false);
    assert!(points[2].is_none());
    let points = x_step(&search, &positions, 3.0, true);
    let filled = points[2].unwrap();
    assert!(filled.interpolated);
    assert!((filled.y - 60.0).abs() <= 1.0, "{filled:?}");
    assert_eq!(
        points.iter().flatten().filter(|p| !p.interpolated).count(),
        4
    );
}

proptest! {
    /// C02-T03 acceptance: the confidence of a point never rises as noise is added to the
    /// line's colour.
    #[test]
    fn confidence_never_rises_with_noise(
        low in 0_i16..15,
        extra in 0_i16..15,
        seed in any::<u64>(),
    ) {
        let high = (low + extra).min(15);
        // A fixed noise pattern in [-1, 1] per channel, scaled by the level.
        let pattern = move |x: u32, y: u32| {
            let h = seed ^ u64::from(x).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ u64::from(y).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
            let unit = |shift: u32| i16::try_from((h >> shift) % 3).unwrap() - 1;
            [unit(0), unit(8), unit(16)]
        };
        let read = |level: i16| {
            let raster = line_image((120, 80), (30.0, 0.2), (5, 115), 3, RED, &|x, y| {
                pattern(x, y).map(|n| n * level)
            });
            let mask = Mask::default();
            let search = Search {
                raster: &raster,
                filter: ColourFilter::foreground(RED),
                area: whole(&raster),
                mask: &mask,
            };
            averaging_window(&search, 3, 3)
        };
        let (calm, noisy) = (read(low), read(high));
        prop_assert_eq!(calm.len(), noisy.len());
        for (a, b) in calm.iter().zip(&noisy) {
            prop_assert!(b.closeness <= a.closeness + 1e-12, "{} > {}", b.closeness, a.closeness);
        }
    }
}

/// The relative error of each read value against the nearest truth series, relative to
/// the largest absolute truth value (as the corpus measures digitizing, see `corpus`).
fn errors(found: &crate::model::Series, truth: &[Value], top: f64) -> Vec<f64> {
    truth
        .iter()
        .map(|series| {
            let xs = series["x"].as_array().unwrap();
            let ys = series["y"].as_array().unwrap();
            found
                .points
                .iter()
                .filter_map(|p| {
                    let XValue::Category(label) = &p.x else {
                        return None;
                    };
                    let i = xs.iter().position(|x| x.as_str() == Some(label.as_str()))?;
                    Some((p.y? - ys[i].as_f64()?).abs() / top)
                })
                .collect::<Vec<f64>>()
        })
        .min_by(|a, b| median(a).total_cmp(&median(b)))
        .unwrap()
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted
        .get(sorted.len() / 2)
        .copied()
        .unwrap_or(f64::INFINITY)
}

/// C02-T03 acceptance (FR-008): the raster line charts of DS-007, rendered from the corpus
/// SVGs at 1x and 2x, read with anchors entered on their outermost ticks and category
/// labels, have a median error of at most 2 % of the axis and the `cv_estimate` class
/// with a confidence on every value.
#[test]
fn raster_line_charts_read_within_two_percent() {
    let svg = folder("svg");
    let raster = folder("raster");
    for name in ["vl-line", "plotly-line", "d3-line"] {
        let drawing = std::fs::read(svg.join(format!("{name}.svg"))).unwrap();
        let truth: Value = serde_json::from_str(
            &std::fs::read_to_string(svg.join(format!("{name}.truth.json"))).unwrap(),
        )
        .unwrap();
        let series = truth["series"].as_array().unwrap();
        let top = series
            .iter()
            .flat_map(|s| s["y"].as_array().unwrap().iter().filter_map(Value::as_f64))
            .fold(0.0_f64, |m, y| m.max(y.abs()));
        for scale in [1_u32, 2] {
            let label = format!("{name}@{scale}x");
            let bytes = std::fs::read(raster.join(format!("{label}.png"))).unwrap();
            let hand = crate::svg::hand_calibration(&drawing, f64::from(scale)).unwrap();
            let options =
                ExtractOptions::new().with_calibration(serde_json::from_value(hand).unwrap());
            let read = extract(&bytes, &label, &options).unwrap();
            assert_eq!(read.report.extractor, "sonarithm-plotsona/raster");
            assert_eq!(
                read.chart.chart_type,
                Some(crate::ChartType::Line),
                "{label}"
            );
            assert_eq!(read.chart.series.len(), series.len(), "{label}: series");
            for found in &read.chart.series {
                assert_eq!(found.provenance.class, ProvenanceClass::CvEstimate);
                assert!(found.provenance.confidence.is_some());
                assert_eq!(found.confidences.len(), found.points.len());
                let errors = errors(found, series, top);
                assert!(
                    errors.len() >= series[0]["x"].as_array().unwrap().len() - 1,
                    "{label}: {} points",
                    errors.len()
                );
                let median = median(&errors);
                println!(
                    "{label} {}: median error {:.4}, {} points",
                    found.name,
                    median,
                    errors.len()
                );
                assert!(
                    median <= 0.02,
                    "{label} {}: median error {median}",
                    found.name
                );
            }
        }
    }
}

/// A raster corpus chart read with its hand calibration, its truth series and the largest
/// absolute truth value.
fn read_corpus(name: &str, scale: u32) -> (crate::Extraction, Vec<Value>, f64) {
    let svg = folder("svg");
    let drawing = std::fs::read(svg.join(format!("{name}.svg"))).unwrap();
    let truth: Value = serde_json::from_str(
        &std::fs::read_to_string(svg.join(format!("{name}.truth.json"))).unwrap(),
    )
    .unwrap();
    let series = truth["series"].as_array().unwrap().clone();
    let top = series
        .iter()
        .flat_map(|s| s["y"].as_array().unwrap().iter().filter_map(Value::as_f64))
        .fold(0.0_f64, |m, y| m.max(y.abs()));
    let label = format!("{name}@{scale}x");
    let bytes = std::fs::read(folder("raster").join(format!("{label}.png"))).unwrap();
    let hand = crate::svg::hand_calibration(&drawing, f64::from(scale)).unwrap();
    let options = ExtractOptions::new().with_calibration(serde_json::from_value(hand).unwrap());
    let read = extract(&bytes, &label, &options).unwrap();
    // The 10 §3.1 checks pass on clean renders with good anchors (C02-T05).
    let codes: Vec<&str> = read
        .report
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert!(codes.is_empty(), "{label}: {codes:?}");
    for found in &read.chart.series {
        assert_eq!(
            found.provenance.class,
            ProvenanceClass::CvEstimate,
            "{label}"
        );
        assert!(found.provenance.confidence.is_some(), "{label}");
        assert_eq!(found.confidences.len(), found.points.len(), "{label}");
    }
    (read, series, top)
}

/// C02-T04 acceptance (FR-008): the raster bar charts of DS-007 (Vega-Lite, Plotly, D3 at
/// 1x and 2x) are read as bar charts with every category, a median error of at most 2 % of
/// the axis, and the `cv_estimate` class with a confidence on every value.
#[test]
fn raster_bar_charts_read_within_two_percent() {
    for name in ["vl-bar", "plotly-bar", "d3-bar"] {
        for scale in [1_u32, 2] {
            let (read, truth, top) = read_corpus(name, scale);
            let label = format!("{name}@{scale}x");
            assert_eq!(
                read.chart.chart_type,
                Some(crate::ChartType::Bar),
                "{label}"
            );
            assert_eq!(read.chart.series.len(), 1, "{label}");
            let errors = errors(&read.chart.series[0], &truth, top);
            assert_eq!(
                errors.len(),
                truth[0]["x"].as_array().unwrap().len(),
                "{label}"
            );
            let median = median(&errors);
            let worst = errors.iter().copied().fold(0.0, f64::max);
            println!("{label}: median error {median:.4}, worst {worst:.4}");
            assert!(median <= 0.02, "{label}: median error {median}");
        }
    }
}

/// C02-T04 acceptance: the raster scatter charts of DS-007 report their markers' count and
/// positions; each truth point's nearest marker is within 2 % of each axis at the median.
#[test]
fn raster_scatter_charts_report_their_markers() {
    for name in ["vl-scatter", "plotly-scatter", "d3-scatter"] {
        for scale in [1_u32, 2] {
            let (read, truth, top) = read_corpus(name, scale);
            let label = format!("{name}@{scale}x");
            assert_eq!(
                read.chart.chart_type,
                Some(crate::ChartType::Scatter),
                "{label}"
            );
            let found: Vec<(f64, f64)> = read.chart.series[0]
                .points
                .iter()
                .filter_map(|p| match p.x {
                    XValue::Number(x) => Some((x, p.y?)),
                    _ => None,
                })
                .collect();
            let xs: Vec<f64> = truth[0]["x"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_f64)
                .collect();
            let ys: Vec<f64> = truth[0]["y"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_f64)
                .collect();
            let x_span = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                - xs.iter().copied().fold(f64::INFINITY, f64::min);
            let (mut dx, mut dy) = (Vec::new(), Vec::new());
            for (x, y) in xs.iter().zip(&ys) {
                let nearest = found
                    .iter()
                    .min_by(|a, b| {
                        let d = |p: &&(f64, f64)| ((p.0 - x) / x_span).hypot((p.1 - y) / top);
                        d(a).total_cmp(&d(b))
                    })
                    .unwrap();
                dx.push((nearest.0 - x).abs() / x_span);
                dy.push((nearest.1 - y).abs() / top);
            }
            let (mx, my) = (median(&dx), median(&dy));
            println!(
                "{label}: {} markers for {} points, median error x {mx:.4}, y {my:.4}",
                found.len(),
                xs.len()
            );
            assert!(found.len() == xs.len(), "{label}: {} markers", found.len());
            assert!(mx <= 0.02 && my <= 0.02, "{label}: {mx} {my}");
        }
    }
}

#[test]
fn a_blank_image_with_a_calibration_has_no_chart() {
    let calibration = json!({ "axes": {
        "x": { "kind": "linear", "anchors": [{ "at": [5, 0], "value": 0 }, { "at": [45, 0], "value": 1 }] },
        "y": { "kind": "linear", "anchors": [{ "at": [0, 35], "value": 0 }, { "at": [0, 5], "value": 1 }] },
    }});
    let raster = Raster::from_rgb(50, 40, vec![[255; 3]; 2000]).unwrap();
    let err = super::read_raster(
        &raster,
        &serde_json::from_value(calibration).unwrap(),
        None,
        &mut Work::new(u64::MAX),
        &mut Vec::new(),
    )
    .unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-309");
}

fn codes(diagnostics: &[sonarithm_core::Diagnostic]) -> Vec<&str> {
    diagnostics.iter().map(|d| d.code.as_str()).collect()
}

/// A drawn line read through `calibration` (JSON), with the diagnostics it raised.
fn read_line(
    calibration: &Value,
    legend: Option<&[String]>,
) -> (crate::model::ChartData, Vec<sonarithm_core::Diagnostic>) {
    let raster = line_image((200, 120), (20.0, 0.4), (10, 190), 3, RED, &|_, _| [0; 3]);
    let mut diagnostics = Vec::new();
    let chart = super::read_raster(
        &raster,
        &serde_json::from_value(calibration.clone()).unwrap(),
        legend,
        &mut Work::new(u64::MAX),
        &mut diagnostics,
    )
    .unwrap();
    (chart, diagnostics)
}

/// Anchors on ticks at x = 10 and 190 and y = 10 and 110, so the line lies within them;
/// `y_mid` moves the middle y anchor to make the fit imprecise.
fn ticks(y_mid: f64) -> Value {
    json!({ "plotArea": { "min": [0, 0], "max": [200, 120] }, "axes": {
        "x": { "kind": "linear", "anchors": [{ "at": [10, 0], "value": 0 }, { "at": [190, 0], "value": 180 }] },
        "y": { "kind": "linear", "anchors": [
            { "at": [0, 110], "value": 0 }, { "at": [0, y_mid], "value": 50 }, { "at": [0, 10], "value": 100 }
        ] },
    }})
}

/// C02-T05 (10 §3.1, points within bounds): values beyond the outermost ticks are reported
/// as extrapolated and kept as read; within them, nothing is reported.
#[test]
fn values_beyond_the_ticks_are_reported_not_dropped() {
    let (inside, diagnostics) = read_line(&ticks(60.0), None);
    assert!(codes(&diagnostics).is_empty(), "{:?}", codes(&diagnostics));
    let narrow = json!({ "plotArea": { "min": [0, 0], "max": [200, 120] }, "axes": {
        "x": { "kind": "linear", "anchors": [{ "at": [50, 0], "value": 0 }, { "at": [150, 0], "value": 100 }] },
        "y": { "kind": "linear", "anchors": [{ "at": [0, 110], "value": 0 }, { "at": [0, 10], "value": 100 }] },
    }});
    let (outside, diagnostics) = read_line(&narrow, None);
    assert_eq!(codes(&diagnostics), ["SNR-ENG-324"]);
    assert!(diagnostics[0].message.contains("x axis"));
    assert_eq!(outside.points(), inside.points());
}

/// C02-T05 (10 §3.1, the fit): an imprecise calibration is reported and lowers every
/// value's confidence; the values themselves are not adjusted.
#[test]
fn an_imprecise_calibration_is_reported_and_lowers_confidence() {
    let (exact, diagnostics) = read_line(&ticks(60.0), None);
    assert_eq!(codes(&diagnostics), Vec::<&str>::new());
    let (rough, diagnostics) = read_line(&ticks(66.0), None);
    assert_eq!(codes(&diagnostics), ["SNR-ENG-325"]);
    let best = |c: &crate::model::ChartData| {
        c.series[0]
            .confidences
            .iter()
            .map(|c| c.get())
            .fold(0.0_f32, f32::max)
    };
    assert!(
        best(&rough) < best(&exact) - 0.1,
        "{} {}",
        best(&rough),
        best(&exact)
    );
    assert_eq!(rough.points(), exact.points());
}

/// C02-T05 (10 §3.1, series count against the legend): a legend with the series count
/// found passes; another count is reported and the series are left as found.
#[test]
fn the_series_count_is_checked_against_a_given_legend() {
    let one = vec!["rain".to_owned()];
    let (_, diagnostics) = read_line(&ticks(60.0), Some(&one));
    assert_eq!(codes(&diagnostics), Vec::<&str>::new());
    let two = vec!["rain".to_owned(), "snow".to_owned()];
    let (chart, diagnostics) = read_line(&ticks(60.0), Some(&two));
    assert_eq!(codes(&diagnostics), ["SNR-ENG-329"]);
    assert_eq!(chart.series.len(), 1);
    // Through `extract`, the legend comes from the options: vl-line has three series.
    let drawing = std::fs::read(folder("svg").join("vl-line.svg")).unwrap();
    let bytes = std::fs::read(folder("raster").join("vl-line@1x.png")).unwrap();
    let hand = crate::svg::hand_calibration(&drawing, 1.0).unwrap();
    let options = ExtractOptions::new()
        .with_calibration(serde_json::from_value(hand).unwrap())
        .with_legend(two);
    let read = extract(&bytes, "vl-line", &options).unwrap();
    assert_eq!(codes(&read.report.diagnostics), ["SNR-ENG-329"]);
    assert_eq!(read.chart.series.len(), 3);
}

/// C02-T05 (03 §4): raster values keep the `cv_estimate` class and each row's confidence
/// through a filter and a sort, and an aggregate takes the lowest confidence of its rows
/// with the estimate class (A04's propagation rules, extended to raster tables).
#[test]
fn raster_provenance_survives_transforms() {
    use sonarithm_data::RowProvenance;
    use sonarithm_data::transform::Transform;
    use sonarithm_data::transform::apply;

    let step = |v: Value| -> Transform { serde_json::from_value(v).unwrap() };

    let (read, _, _) = read_corpus("vl-line", 1);
    let table = read.bundle.primary().unwrap();
    let before = RowProvenance::read(table).unwrap();
    let expected: Vec<f32> = read
        .chart
        .series
        .iter()
        .flat_map(|s| s.confidences.iter().map(|c| c.get()))
        .collect();
    let stored: Vec<f32> = before
        .rows()
        .iter()
        .map(|p| p.confidence.unwrap().get())
        .collect();
    assert_eq!(stored, expected, "__snr_conf holds each value's confidence");
    let mut notes = sonarithm_core::Diagnostics::default();
    let filtered = apply(
        table,
        &step(json!({ "op": "filter", "expr": "y > 50" })),
        &mut notes,
    )
    .unwrap();
    let sorted = apply(
        &filtered,
        &step(json!({ "op": "sort", "field": "y" })),
        &mut notes,
    )
    .unwrap();
    let after = RowProvenance::read(&sorted).unwrap();
    assert!(!after.is_empty() && after.len() < before.len());
    for p in after.rows() {
        assert_eq!(p.class, ProvenanceClass::CvEstimate);
        assert!(stored.contains(&p.confidence.unwrap().get()));
    }
    let mean = apply(
        table,
        &step(json!({ "op": "aggregate", "groupby": ["series"], "ops": [{ "op": "mean", "field": "y", "as": "mean" }] })),
        &mut notes,
    )
    .unwrap();
    let grouped = RowProvenance::read(&mean).unwrap();
    assert_eq!(grouped.len(), 3);
    let lowest = stored.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        grouped
            .rows()
            .iter()
            .all(|p| p.class == ProvenanceClass::CvEstimate)
    );
    assert!(
        grouped
            .rows()
            .iter()
            .any(|p| (p.confidence.unwrap().get() - lowest).abs() < 1e-6)
    );
}
