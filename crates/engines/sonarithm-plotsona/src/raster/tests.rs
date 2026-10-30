// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "tests"
)]

use std::io::Cursor;

use proptest::prelude::*;
use serde_json::json;

use super::*;

fn calibration(value: serde_json::Value) -> SpecCalibration {
    serde_json::from_value(value).unwrap()
}

/// A white image with dark axis lines: the x axis `thick` rows from `row`, the y axis
/// `thick` columns from `column`, spanning the image from the axes outwards, plus a light
/// grid line that must not count.
fn axes_image(width: u32, height: u32, row: u32, column: u32, thick: u32) -> Raster {
    let mut pixels = vec![[255_u8; 3]; (width * height) as usize];
    let mut set = |x: u32, y: u32, c: [u8; 3]| {
        if x < width && y < height {
            pixels[(y * width + x) as usize] = c;
        }
    };
    for t in 0..thick {
        for x in column..width {
            set(x, row + t, [20, 20, 20]);
        }
        for y in 0..row + thick {
            set(column + t, y, [20, 20, 20]);
        }
    }
    // A light grey grid line across the plot, right of the axis (drawn under it).
    for x in column + thick..width {
        set(x, row / 2, [200, 200, 200]);
    }
    Raster::from_rgb(width, height, pixels).unwrap()
}

fn png(image: &image::RgbaImage) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, ImageFormat::Png).unwrap();
    out.into_inner()
}

/// A PNG whose header declares `width × height` but holds almost nothing: a decompression
/// bomb's shape.
fn png_header_only(width: u32, height: u32) -> Vec<u8> {
    fn crc(bytes: &[u8]) -> u32 {
        let mut crc = 0xffff_ffff_u32;
        for &b in bytes {
            crc ^= u32::from(b);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }
    let chunk = |out: &mut Vec<u8>, kind: &[u8], data: &[u8]| {
        out.extend(u32::try_from(data.len()).unwrap().to_be_bytes());
        let mut body = kind.to_vec();
        body.extend(data);
        out.extend(&body);
        out.extend(crc(&body).to_be_bytes());
    };
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = width.to_be_bytes().to_vec();
    ihdr.extend(height.to_be_bytes());
    ihdr.extend([8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(
        &mut out,
        b"IDAT",
        &[0x78, 0x9c, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01],
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

proptest! {
    /// C02-T02 acceptance: on synthetic rasters with a known calibration, a value drawn into
    /// the pixel its exact position falls in is read back within the half-pixel bound, on
    /// linear and log axes.
    #[test]
    fn values_land_within_the_half_pixel_bound(
        origin in 20.0_f64..200.0,
        pixels_per_unit in 0.5_f64..40.0,
        log in any::<bool>(),
        t in 0.0_f64..1.0,
    ) {
        let (kind, lo, hi) = if log { ("log", 1.0, 1000.0) } else { ("linear", -50.0, 50.0) };
        // Anchors at pixel centres, as a user clicks them, far apart.
        let span = if log { 3.0 } else { hi - lo };
        let p_lo = (origin.floor()) + 0.5;
        let p_hi = (origin + span * pixels_per_unit).floor() + 0.5;
        let cal = calibration(json!({ "axes": {
            "x": { "kind": kind, "anchors": [{ "at": [p_lo, 0.0], "value": lo }, { "at": [p_hi, 0.0], "value": hi }] },
            "y": { "kind": "linear", "anchors": [{ "at": [0.0, 10.5], "value": 1 }, { "at": [0.0, 0.5], "value": 0 }] },
        }}));
        let raster = RasterCalibration::from_spec(&cal).unwrap();
        // The true value, its exact position, and the pixel a renderer puts it in.
        let truth = if log { lo * (hi / lo).powf(t) } else { lo + (hi - lo) * t };
        let exact = raster.x.position_of(truth).unwrap();
        let pixel = exact.floor();
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "positive and small")]
        let (estimate, _) = raster.value_at_pixel(pixel as u32, 0);
        prop_assert!(estimate.contains(truth), "{truth} not in {estimate:?}");
        prop_assert!(estimate.uncertainty() > 0.0);
    }

    /// The axes of synthetic rasters are found where they were drawn, a light grid line
    /// does not count, and the plot area lies between them.
    #[test]
    fn axis_lines_are_found_where_they_were_drawn(
        width in 40_u32..240,
        height in 40_u32..240,
        thick in 1_u32..4,
        margin in 2_u32..20,
    ) {
        let (row, column) = (height - margin - thick, margin);
        let image = axes_image(width, height, row, column, thick);
        let axes = find_axes(&image, &mut Work::new(u64::MAX)).unwrap();
        let x = axes.x.unwrap();
        let y = axes.y.unwrap();
        prop_assert_eq!(x.position, f64::from(row) + f64::from(thick) / 2.0);
        prop_assert_eq!(y.position, f64::from(column) + f64::from(thick) / 2.0);
        prop_assert_eq!(x.end, f64::from(width));
        let plot = axes.plot.unwrap();
        prop_assert_eq!(plot.min, [y.position, 0.0]);
        prop_assert_eq!(plot.max, [f64::from(width), x.position]);
    }
}

/// C02-T02 acceptance: every calibration can be entered by hand (no OCR): linear, log and
/// date axes from anchors in pixels, read back exactly at the anchors.
#[test]
fn every_calibration_can_be_entered_by_hand() {
    let cal = calibration(json!({ "axes": {
        "x": { "kind": "date", "anchors": [
            { "at": [100.5, 400.5], "value": "2020-01-01" },
            { "at": [500.5, 400.5], "value": "2024-01-01" },
        ] },
        "y": { "kind": "log", "anchors": [
            { "at": [100.5, 400.5], "value": 10 },
            { "at": [100.5, 100.5], "value": 1000 },
        ] },
    }}));
    let raster = RasterCalibration::from_spec(&cal).unwrap();
    let (x, y) = raster.value_at(500.5, 100.5);
    #[expect(clippy::cast_precision_loss, reason = "test dates")]
    let end = parse_date("2024-01-01").unwrap() as f64;
    assert!((x.value - end).abs() < 1.0, "{x:?}");
    assert!((y.value - 1000.0).abs() < 1e-9, "{y:?}");
    // Halfway up a log axis is the geometric mean; y grows upwards as pixels grow down.
    let (_, mid) = raster.value_at(0.0, 250.5);
    assert!((mid.value - 100.0).abs() < 1e-9);
    let linear = calibration(json!({ "axes": {
        "x": { "kind": "linear", "anchors": [{ "at": [10.5, 0], "value": 0 }, { "at": [110.5, 0], "value": 50 }] },
        "y": { "kind": "linear", "anchors": [{ "at": [0, 90.5], "value": 0 }, { "at": [0, 10.5], "value": 8 }] },
    }}));
    let raster = RasterCalibration::from_spec(&linear).unwrap();
    let (x, y) = raster.value_at_pixel(60, 50);
    assert!((x.value - 25.0).abs() < 1e-9 && (y.value - 4.0).abs() < 1e-9);
    // Half a pixel is a quarter of a unit on x (2 pixels per unit) and 0.05 on y.
    assert!((x.uncertainty() - 0.25).abs() < 1e-9);
    assert!((y.uncertainty() - 0.05).abs() < 1e-9);
}

#[test]
fn category_y_axes_and_mistyped_anchors_are_refused() {
    let category = calibration(json!({ "axes": {
        "x": { "kind": "linear", "anchors": [{ "at": [1, 0], "value": 0 }, { "at": [9, 0], "value": 1 }] },
        "y": { "kind": "category", "anchors": [{ "at": [0, 1], "value": "a" }, { "at": [0, 9], "value": "b" }] },
    }}));
    assert_eq!(
        RasterCalibration::from_spec(&category)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-320"
    );
    let mistyped = calibration(json!({ "axes": {
        "x": { "kind": "date", "anchors": [{ "at": [1, 0], "value": 5 }, { "at": [9, 0], "value": 6 }] },
        "y": { "kind": "linear", "anchors": [{ "at": [0, 1], "value": 0 }, { "at": [0, 9], "value": 1 }] },
    }}));
    assert_eq!(
        RasterCalibration::from_spec(&mistyped)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-320"
    );
    let log_zero = calibration(json!({ "axes": {
        "x": { "kind": "linear", "anchors": [{ "at": [1, 0], "value": 0 }, { "at": [9, 0], "value": 1 }] },
        "y": { "kind": "log", "anchors": [{ "at": [0, 1], "value": 0 }, { "at": [0, 9], "value": 10 }] },
    }}));
    assert_eq!(
        RasterCalibration::from_spec(&log_zero)
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-321"
    );
}

#[test]
fn images_decode_onto_white() {
    let mut image = image::RgbaImage::new(3, 2);
    image.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    image.put_pixel(1, 0, image::Rgba([0, 0, 0, 0]));
    image.put_pixel(2, 0, image::Rgba([0, 0, 0, 128]));
    let raster = decode(
        &png(&image),
        ImageFormat::Png,
        &Budgets::DEFAULT,
        &mut Work::new(u64::MAX),
    )
    .unwrap();
    assert_eq!((raster.width(), raster.height()), (3, 2));
    assert_eq!(raster.pixel(0, 0), Some([255, 0, 0]));
    assert_eq!(
        raster.pixel(1, 0),
        Some([255, 255, 255]),
        "transparent is background"
    );
    assert_eq!(raster.pixel(2, 0), Some([127, 127, 127]));
    assert_eq!(raster.pixel(3, 0), None);
}

/// C02-T02 acceptance: decompression bombs fail within budget, from the header, before
/// their pixels are allocated; a damaged image is reported, not a panic.
#[test]
fn bombs_fail_within_budget() {
    let bomb = png_header_only(100_000, 100_000);
    let err = decode(
        &bomb,
        ImageFormat::Png,
        &Budgets::DEFAULT,
        &mut Work::new(u64::MAX),
    )
    .unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-306", "{err}");
    let mut tight = Budgets::DEFAULT;
    tight.max_pixels = 100;
    let small = png(&image::RgbaImage::new(20, 20));
    assert_eq!(
        decode(&small, ImageFormat::Png, &tight, &mut Work::new(u64::MAX))
            .unwrap_err()
            .code
            .as_str(),
        "SNR-ENG-306"
    );
    let damaged = png_header_only(4, 4);
    assert_eq!(
        decode(
            &damaged,
            ImageFormat::Png,
            &Budgets::DEFAULT,
            &mut Work::new(u64::MAX)
        )
        .unwrap_err()
        .code
        .as_str(),
        "SNR-ENG-327"
    );
    let jpeg = b"\xff\xd8\xff\xe0 not a jpeg";
    assert_eq!(
        decode(
            jpeg,
            ImageFormat::Jpeg,
            &Budgets::DEFAULT,
            &mut Work::new(u64::MAX)
        )
        .unwrap_err()
        .code
        .as_str(),
        "SNR-ENG-327"
    );
}

/// Through `extract`, a raster chart is decoded under the budgets before anything else; it
/// then needs a calibration (C02-T03).
#[test]
fn extract_decodes_rasters_under_budget() {
    let chart = png(&image::RgbaImage::from_pixel(
        50,
        40,
        image::Rgba([255, 255, 255, 255]),
    ));
    let err = crate::extract(&chart, "chart.png", &crate::ExtractOptions::new()).unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-328");
    let bomb = png_header_only(60_000, 60_000);
    let err = crate::extract(&bomb, "bomb.png", &crate::ExtractOptions::new()).unwrap_err();
    assert_eq!(err.code.as_str(), "SNR-ENG-306");
}

fn axis_codes(calibration_json: serde_json::Value) -> Result<Vec<String>, String> {
    RasterCalibration::from_spec(&calibration(calibration_json))
        .map(|c| {
            c.checks
                .iter()
                .map(|d| d.code.as_str().to_owned())
                .collect()
        })
        .map_err(|d| d.code.as_str().to_owned())
}

/// C02-T05 (10 §3.1, monotonic ticks): ticks whose values rise and fall along the axis are
/// refused; ticks in order pass.
#[test]
fn ticks_that_rise_and_fall_are_refused() {
    let y = |values: [f64; 3]| {
        json!({ "axes": {
            "x": { "kind": "linear", "anchors": [{ "at": [0, 0], "value": 0 }, { "at": [100, 0], "value": 1 }] },
            "y": { "kind": "linear", "anchors": [
                { "at": [0, 90], "value": values[0] }, { "at": [0, 50], "value": values[1] }, { "at": [0, 10], "value": values[2] }
            ] },
        }})
    };
    assert_eq!(axis_codes(y([0.0, 50.0, 100.0])), Ok(vec![]));
    assert_eq!(
        axis_codes(y([0.0, 120.0, 100.0])),
        Err("SNR-ENG-322".to_owned())
    );
}

/// C02-T05 (10 §3.1, log against linear): log-spaced ticks entered as a linear axis are
/// reported, and the same ticks entered as log pass.
#[test]
fn a_log_axis_entered_as_linear_is_reported() {
    let y = |kind: &str| {
        json!({ "axes": {
            "x": { "kind": "linear", "anchors": [{ "at": [0, 0], "value": 0 }, { "at": [100, 0], "value": 1 }] },
            "y": { "kind": kind, "anchors": [
                { "at": [0, 90], "value": 1 }, { "at": [0, 60], "value": 10 }, { "at": [0, 30], "value": 100 }, { "at": [0, 0], "value": 1000 }
            ] },
        }})
    };
    assert_eq!(axis_codes(y("log")), Ok(vec![]));
    let found = axis_codes(y("linear")).unwrap();
    assert!(found.contains(&"SNR-ENG-323".to_owned()), "{found:?}");
}
