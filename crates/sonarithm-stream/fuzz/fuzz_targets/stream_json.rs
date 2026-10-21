// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the JSON message decoder (WebSocket and MQTT payloads): decoding, schema
//! inference and row building must never panic (B06-T02).

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_stream::{MessageSchema, RowBuffer, decode};

fuzz_target!(|data: &[u8]| {
    let Ok(message) = decode::json_object(data) else {
        return;
    };
    let Ok(schema) = MessageSchema::infer(&message) else {
        return;
    };
    let mut buffer = RowBuffer::new(schema.clone());
    if let Ok(row) = decode::row(&message, &schema) {
        let _ = buffer.push(row);
        let _ = buffer.take();
    }
});
