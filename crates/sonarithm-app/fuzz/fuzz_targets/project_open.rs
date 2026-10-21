// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the project reader (RFC-0002 §11, B05-T04): any bytes either open or are refused
//! with an error, never a panic or a read outside the bundle. A bundle that opens and saves
//! again saves to bytes that reopen and save identically.

#![no_main]

use std::path::PathBuf;
use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use sonarithm_app::project;

/// Lets the data layer contain panics of the Parquet decoder as it does in production:
/// libfuzzer-sys installs a hook that aborts on every panic, even a caught one. A panic
/// that escapes `project::open` still reaches libfuzzer-sys's `catch_unwind`, which aborts.
fn contain_decoder_panics() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| std::panic::set_hook(Box::new(|_| {})));
}

/// Two scratch files of this process: the input, and its re-save.
fn scratch() -> &'static (PathBuf, PathBuf) {
    static PATHS: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();
    PATHS.get_or_init(|| {
        let dir = std::env::temp_dir();
        let id = std::process::id();
        (
            dir.join(format!("sonarithm-fuzz-{id}-in.sonarithm")),
            dir.join(format!("sonarithm-fuzz-{id}-again.sonarithm")),
        )
    })
}

fuzz_target!(|data: &[u8]| {
    contain_decoder_panics();
    let (input, again) = scratch();
    if std::fs::write(input, data).is_err() {
        return;
    }
    let Ok(opened) = project::open(input, &mut || 1) else {
        return;
    };
    // A project may open and still refuse to save (an absolute path in a spec); one that
    // saves must reopen and save to the same bytes.
    let Ok(first) = project::write(&opened.project, again, true) else {
        return;
    };
    std::fs::write(again, &first).expect("the scratch file can be written");
    let reopened = project::open(again, &mut || 1).expect("a saved project reopens");
    assert_eq!(reopened.project.specs, opened.project.specs, "specs round-trip");
    let second = project::write(&reopened.project, again, true).expect("it saves again");
    assert_eq!(first, second, "saving is stable");
});
