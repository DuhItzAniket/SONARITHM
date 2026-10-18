// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `sonarithm` command-line interface.
//!
//! Skeleton created in phase A01: it prints its name and version. The subcommands
//! (`render`, `play`, `validate`, `schema`, `describe`, `lint`, `inspect`) arrive in A07
//! (docs/architecture/11-bindings-cli-plugins.md). Apps hold no business logic
//! (docs/architecture/02-repository-and-crate-layout.md §3 rule 6).

#![forbid(unsafe_code)]

use std::io::Write as _;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut stdout = std::io::stdout().lock();
    match writeln!(stdout, "sonarithm {}", env!("CARGO_PKG_VERSION")) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
