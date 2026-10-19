// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the spreadsheet source (calamine: XLSX, XLSB, XLS, ODS), sheet and range
//! selection and inference (A04-T10). The first byte picks the format, a sheet position
//! and whether a range is used.

#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use sonarithm_data::{Format, SheetOptions, SheetRef, read_bytes};

fuzz_target!(|data: &[u8]| {
    common::contain_decoder_panics();
    let Some((&switches, file)) = data.split_first() else {
        return;
    };
    let format = if switches & 1 == 0 {
        Format::Xlsx
    } else {
        Format::Ods
    };
    let mut sheet = SheetOptions::new();
    if switches & 2 != 0 {
        sheet = sheet.with_sheet(SheetRef::Index(usize::from(switches >> 4)));
    }
    if switches & 4 != 0 {
        sheet = sheet.with_range("A1:F40");
    }
    let options = common::options().with_sheet(sheet);
    if let Ok(loaded) = read_bytes(file, format, "fuzz.xlsx", &options) {
        common::check(&loaded);
    }
});
