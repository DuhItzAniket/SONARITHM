// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes spec migrations (testing-strategy §7): no panic on any JSON document, and
//! migrating twice gives the same document as migrating once.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) else {
        return;
    };
    if let Ok(migrated) = sonarithm_spec::migrate::migrate(value) {
        let again = sonarithm_spec::migrate::migrate(migrated.value.clone());
        let Ok(again) = again else {
            panic!("a migrated document does not migrate again: {again:?}");
        };
        assert_eq!(again.value, migrated.value, "migration is not idempotent");
    }
});
