// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `sonarithm` command-line interface (docs/architecture/11-bindings-cli-plugins.md §2).
//!
//! Commands: `render`, `validate`, `inspect`, `lint` (a stub until C05), `quick`, `schema`,
//! `spec migrate`, and `study check` and `study run` (ECHOMETRIC's study runner, A08). Exit codes: 0 success, 1 errors, 2 usage, 3 warnings denied. `--json`
//! prints one JSON object with stable fields; `--plain` prints linear text for screen
//! readers. Apps hold no business logic: every command calls the `sonarithm` facade
//! (docs/architecture/02-repository-and-crate-layout.md §3 rule 6).

#![forbid(unsafe_code)]

mod cli;
mod commands;
mod output;
mod study;

use std::process::ExitCode;

use clap::Parser as _;

use crate::output::{Exit, Out};

fn main() -> ExitCode {
    let cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Help and the version go to standard output and succeed; usage errors exit
            // with 2 (11 §2).
            let exit = if err.use_stderr() {
                Exit::Usage
            } else {
                Exit::Success
            };
            let _ = err.print();
            return exit.into();
        }
    };
    let out = Out::new(cli.json, cli.plain);
    commands::run(&cli.command, out).into()
}
