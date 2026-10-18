// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Smoke test for the A01 CLI skeleton.

use std::process::Command;

#[test]
fn prints_name_and_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_sonarithm"))
        .output()
        .expect("the sonarithm binary runs");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert_eq!(
        stdout.trim_end(),
        format!("sonarithm {}", env!("CARGO_PKG_VERSION"))
    );
}
