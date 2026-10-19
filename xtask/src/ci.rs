// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask ci` and `cargo xtask wasm-check`: the local equivalent of the required
//! `ci.yml` checks (docs/engineering/ci-cd.md#what-ciyml-runs-by-phase).
//!
//! Every step runs even if an earlier one fails, and a summary lists the failures at the
//! end. The MSRV check is CI-only: it needs extra toolchains.

use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context as _, Result, bail};

use crate::check_deps;

/// Crates that must build for `wasm32-unknown-unknown`
/// (docs/architecture/02-repository-and-crate-layout.md §2 and §3 rule 4).
/// Engine crates join this list as they are created. `.github/workflows/ci.yml` runs
/// `cargo xtask wasm-check`, so this list is the single source.
pub(crate) const WASM_CRATES: &[&str] = &[
    "sonarithm",
    "sonarithm-core",
    "sonarithm-spec",
    "sonarithm-data",
    "sonarithm-compiler",
    "sonarithm-dsp",
    "sonarithm-render",
    "sonarithm-export",
    "sonarithm-numisona",
];

const WASM_TARGET: &str = "wasm32-unknown-unknown";

/// One cargo invocation run by `cargo xtask ci`.
struct Step {
    name: &'static str,
    args: Vec<String>,
    env: Vec<(&'static str, &'static str)>,
    /// A cargo subcommand that must be installed first, with its install hint.
    requires: Option<(&'static str, &'static str)>,
}

impl Step {
    fn cargo(name: &'static str, args: &[&str]) -> Self {
        Self {
            name,
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            env: Vec::new(),
            requires: None,
        }
    }

    fn env(mut self, key: &'static str, value: &'static str) -> Self {
        self.env.push((key, value));
        self
    }

    fn requires(mut self, subcommand: &'static str, install: &'static str) -> Self {
        self.requires = Some((subcommand, install));
        self
    }
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let cargo = cargo();
    let tests = if has_subcommand(&cargo, root, "nextest") {
        Step::cargo("tests", &["nextest", "run", "--workspace", "--locked"])
    } else {
        println!(
            "note: cargo-nextest not found, using `cargo test` \
             (install: cargo install --locked cargo-nextest)"
        );
        Step::cargo("tests", &["test", "--workspace", "--locked"])
    };
    let steps = [
        Step::cargo("format", &["fmt", "--all", "--check"]),
        Step::cargo(
            "clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        ),
        tests,
        // nextest does not run doctests.
        Step::cargo("doctests", &["test", "--workspace", "--doc", "--locked"]),
        Step::cargo("rustdoc", &["doc", "--workspace", "--no-deps", "--locked"])
            .env("RUSTDOCFLAGS", "-D warnings"),
        Step {
            name: "wasm32 build",
            args: wasm_args(),
            env: Vec::new(),
            requires: None,
        },
        Step::cargo("cargo-deny", &["deny", "check"])
            .requires("deny", "cargo install --locked cargo-deny"),
    ];

    let mut failed: Vec<String> = Vec::new();
    for step in &steps {
        println!("\n==> {}", step.name);
        if let Some((subcommand, install)) = step.requires
            && !has_subcommand(&cargo, root, subcommand)
        {
            failed.push(format!(
                "{} (cargo-{subcommand} is not installed: {install})",
                step.name
            ));
            continue;
        }
        let mut command = Command::new(&cargo);
        command.args(&step.args).current_dir(root);
        for (key, value) in &step.env {
            command.env(key, value);
        }
        let status = command
            .status()
            .with_context(|| format!("starting `cargo {}`", step.args.join(" ")))?;
        if !status.success() {
            failed.push(step.name.to_owned());
        }
    }

    println!("\n==> dependency rules");
    if let Err(err) = check_deps::run(&root.join("Cargo.toml")) {
        eprintln!("error: {err:#}");
        failed.push("dependency rules".to_owned());
    }

    if failed.is_empty() {
        println!("\nAll local CI checks passed.");
        Ok(())
    } else {
        bail!("{} check(s) failed: {}", failed.len(), failed.join(", "))
    }
}

/// Builds every crate in [`WASM_CRATES`] for `wasm32-unknown-unknown`.
pub(crate) fn wasm_check(root: &Path) -> Result<()> {
    let status = Command::new(cargo())
        .args(wasm_args())
        .current_dir(root)
        .status()
        .context("starting the wasm32 build")?;
    if status.success() {
        Ok(())
    } else {
        bail!("the wasm32 build failed (is the {WASM_TARGET} target installed?)")
    }
}

fn wasm_args() -> Vec<String> {
    let mut args: Vec<String> = ["build", "--locked", "--target", WASM_TARGET]
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();
    for krate in WASM_CRATES {
        args.push("-p".to_owned());
        args.push((*krate).to_owned());
    }
    args
}

/// The cargo that runs xtask, so child commands use the same toolchain.
fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

/// Whether `cargo <subcommand>` is installed.
fn has_subcommand(cargo: &OsString, root: &Path, subcommand: &str) -> bool {
    Command::new(cargo)
        .args([subcommand, "--version"])
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
