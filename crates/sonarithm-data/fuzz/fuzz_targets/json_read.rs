// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the JSON and NDJSON source: shape detection, flattening and inference
//! (A04-T10). The first byte picks the format and whether a pointer is used.

#![no_main]
#![allow(dead_code, reason = "not every target uses every helper in `common`")]

mod common;

use libfuzzer_sys::fuzz_target;
use sonarithm_data::{Format, JsonOptions, read_bytes};

fuzz_target!(|data: &[u8]| {
    let Some((&switches, file)) = data.split_first() else {
        return;
    };
    let format = if switches & 1 == 0 {
        Format::Json
    } else {
        Format::Ndjson
    };
    let mut json = JsonOptions::new();
    if switches & 2 != 0 {
        json = json.with_pointer("/data");
    }
    let options = common::options().with_json(json);
    if let Ok(loaded) = read_bytes(file, format, "fuzz.json", &options) {
        common::check(&loaded);
    }
});
