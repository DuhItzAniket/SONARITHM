// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask`: SONARITHM workspace automation.
//!
//! A plain Rust binary, so it behaves the same on every OS
//! (docs/engineering/dev-environment.md#recommended-tools). The `xtask` alias lives in
//! `.cargo/config.toml`. Run `cargo xtask help` for the task list.

#![forbid(unsafe_code)]
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "xtask is a terminal tool: it reports progress and results on stdout and stderr"
)]

mod check_deps;
mod ci;
mod citation;
mod codegen;
mod codes;
mod gallery;
mod golden;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result, bail};

const HELP: &str = "\
Usage: cargo xtask <task> [options]

Tasks:
  ci                 Run the local equivalent of the required CI checks
  check-deps         Check the crate dependency rules (02-repository-and-crate-layout.md §3)
                     Options: --manifest-path <path>  check another workspace
  codegen            Regenerate every generated file (code registry; JSON Schemas from A03-T06)
  citation           Check that CITATION.cff and .zenodo.json name the workspace version
                     Options: --release YYYY-MM-DD  write the version and release date
  codes              Check the diagnostic code tables and write docs/reference/error-codes.md
  gallery            Write the example datasets DS-001 to DS-005 to examples/data
                     Options: --check  compare instead of writing (CI)
  golden             Check the golden-audio cases (tests/golden/); options:
                     --update --reason '<why>' [case...]   rewrite the references
                     --render-set <dir>                     write WAVs and summary.json
                     --compare <reference> <set>...         null residual and ΔLUFS (PB-06)
  wasm-check         Build the wasm32-compatible crates for wasm32-unknown-unknown
  help               Show this message";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let task = args.next();
    let rest: Vec<String> = args.collect();
    let root = workspace_root()?;
    match task.as_deref() {
        Some("ci") => {
            no_options(&rest)?;
            ci::run(&root)
        }
        Some("check-deps") => {
            let manifest = manifest_path_option(&rest)?.unwrap_or_else(|| root.join("Cargo.toml"));
            check_deps::run(&manifest)
        }
        Some("codegen") => {
            no_options(&rest)?;
            codegen::run(&root)
        }
        Some("citation") => citation::run(&root, &rest),
        Some("gallery") => gallery::run(&root, &rest),
        Some("codes") => {
            no_options(&rest)?;
            codes::run(&root)
        }
        Some("golden") => golden::run(&root, &rest),
        Some("wasm-check") => {
            no_options(&rest)?;
            ci::wasm_check(&root)
        }
        Some("help" | "-h" | "--help") => {
            println!("{HELP}");
            Ok(())
        }
        Some(other) => bail!("unknown task `{other}`\n\n{HELP}"),
        None => bail!("no task given\n\n{HELP}"),
    }
}

/// The workspace root: the parent of the `xtask/` directory.
fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask/ has no parent directory")
}

fn no_options(rest: &[String]) -> Result<()> {
    match rest.first() {
        Some(option) => bail!("unexpected option `{option}`\n\n{HELP}"),
        None => Ok(()),
    }
}

fn manifest_path_option(rest: &[String]) -> Result<Option<PathBuf>> {
    match rest {
        [] => Ok(None),
        [flag, path] if flag == "--manifest-path" => Ok(Some(PathBuf::from(path))),
        _ => bail!("expected `--manifest-path <path>` or nothing\n\n{HELP}"),
    }
}
