// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the OSC packet decoder: nested bundles, argument types and the message limit
//! (B06-T02).

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_stream::{MessageSchema, decode};

fuzz_target!(|data: &[u8]| {
    let Ok(messages) = decode::osc(data) else {
        return;
    };
    assert!(messages.len() <= decode::MAX_OSC_MESSAGES);
    for message in &messages {
        if let Ok(schema) = MessageSchema::infer(message) {
            let _ = decode::row(message, &schema);
        }
    }
});
