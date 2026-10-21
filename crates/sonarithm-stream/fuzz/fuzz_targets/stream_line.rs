// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the line decoder (serial ports and followed files): JSON lines and separated
//! numbers (B06-T02).

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_stream::{MessageSchema, decode};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(message) = decode::line(text) else {
        return;
    };
    for value in message.values() {
        if let Some(number) = value.as_f64() {
            assert!(number.is_finite());
        }
    }
    if let Ok(schema) = MessageSchema::infer(&message) {
        let _ = decode::row(&message, &schema);
    }
});
