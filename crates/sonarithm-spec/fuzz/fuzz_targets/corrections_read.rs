// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the chart correction file reader (RFC-0005 §8, §12). Beyond "no panic": every
//! refusal is a registered `SNR-SPEC-09x` code, and an accepted file written again reads
//! back equal and writes the same bytes (the canonical form is a fixed point).

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_spec::correction_file::{read, write};

fuzz_target!(|data: &[u8]| {
    match read(data) {
        Ok((file, _)) => {
            let text = write(&file);
            let Ok((again, _)) = read(text.as_bytes()) else {
                panic!("a written correction file does not read back");
            };
            assert_eq!(again, file, "a correction file changed on a round trip");
            assert_eq!(write(&again), text, "writing is not canonical");
        }
        Err(diagnostics) => {
            for d in &diagnostics {
                assert!(d.code.as_str().starts_with("SNR-SPEC-09"), "{}", d.code.as_str());
            }
        }
    }
});
