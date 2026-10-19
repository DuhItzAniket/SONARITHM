// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the whole spec pipeline: syntax, shape, semantics, defaults, canonical form and
//! the miette span locator (A03-T10; exit criterion: ≥ 1 h without a panic).
//!
//! Properties checked beyond "no panic": the canonical form of a valid spec is itself a
//! valid spec with the same hash (a fixed point), and every diagnostic can be located.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if let Err(diagnostics) = sonarithm_spec::parse(text) {
        let _ = sonarithm_spec::reports("fuzz.json", text, &diagnostics);
    }
    match sonarithm_spec::prepare(text, None) {
        Ok(prepared) => {
            let again = sonarithm_spec::prepare(&prepared.canonical, None);
            let Ok(again) = again else {
                panic!("the canonical form of a valid spec does not prepare: {again:?}");
            };
            assert_eq!(
                again.hash, prepared.hash,
                "the canonical form is not a fixed point"
            );
        }
        Err(diagnostics) => {
            let _ = sonarithm_spec::reports("fuzz.json", text, &diagnostics);
        }
    }
});
