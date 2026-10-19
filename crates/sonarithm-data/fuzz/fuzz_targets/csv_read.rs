// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the CSV and TSV source: encoding detection, dialect and header sniffing, type
//! and semantic inference (A04-T10). The first byte picks options, the rest is the file.

#![no_main]
#![allow(dead_code, reason = "not every target uses every helper in `common`")]

mod common;

use libfuzzer_sys::fuzz_target;
use sonarithm_data::{CsvOptions, Format, read_bytes};

fuzz_target!(|data: &[u8]| {
    let Some((&switches, file)) = data.split_first() else {
        return;
    };
    let format = if switches & 1 == 0 {
        Format::Csv
    } else {
        Format::Tsv
    };
    let mut csv = CsvOptions::new();
    if switches & 2 != 0 {
        csv = csv.with_decimal_comma();
    }
    if switches & 4 != 0 {
        csv = csv.with_header(switches & 8 != 0);
    }
    if switches & 16 != 0 {
        csv = csv.with_date_format("%d.%m.%Y");
    }
    let options = common::options().with_csv(csv);
    if let Ok(loaded) = read_bytes(file, format, "fuzz.csv", &options) {
        common::check(&loaded);
    }
});
