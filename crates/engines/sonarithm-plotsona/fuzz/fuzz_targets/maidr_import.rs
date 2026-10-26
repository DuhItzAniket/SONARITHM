// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the MAIDR importer (C01-T10): every document either imports to a v0.2 spec or
//! fails with a registered PLOTSONA code, and every report entry has a JSON pointer.

#![no_main]

use libfuzzer_sys::fuzz_target;
use sonarithm_plotsona::Budgets;
use sonarithm_plotsona::import::maidr;

fuzz_target!(|data: &[u8]| {
    match maidr::import(data, &Budgets::DEFAULT) {
        Ok(import) => {
            assert_eq!(import.spec["specVersion"], "0.2");
            for entry in import.report.entries() {
                assert!(entry.pointer.is_empty() || entry.pointer.starts_with('/'));
            }
        }
        Err(diagnostic) => assert!(diagnostic.code.as_str().starts_with("SNR-ENG-3")),
    }
});
