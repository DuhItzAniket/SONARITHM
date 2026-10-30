// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the raster tier (C02-T02, C02-T03): PNG or JPEG decoding under the pixel budget,
//! the axis-line search, then colour detection and line reading through a calibration that
//! spans the image. The first byte picks the format. Every error is a registered code.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::fuzzing::raster_image;

fuzz_target!(|data: &[u8]| {
    let Some((&switches, image)) = data.split_first() else {
        return;
    };
    let mut budgets = Budgets::DEFAULT;
    // Small enough that no input can exhaust the fuzzer's memory.
    budgets.max_pixels = 4_000_000;
    if let Err(diagnostic) = raster_image(switches & 1 == 0, image, &budgets) {
        assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3"));
    }
});
