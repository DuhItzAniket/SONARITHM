// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the Parquet and Arrow IPC sources, including provenance normalization
//! (A04-T10). The first byte picks the format.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use sonarithm_data::{Format, read_bytes};

fuzz_target!(|data: &[u8]| {
    common::contain_decoder_panics();
    let Some((&switches, file)) = data.split_first() else {
        return;
    };
    let format = if switches & 1 == 0 {
        Format::Parquet
    } else {
        Format::Arrow
    };
    if let Ok(loaded) = read_bytes(file, format, "fuzz.bin", &common::options()) {
        common::check(&loaded);
    }
});
