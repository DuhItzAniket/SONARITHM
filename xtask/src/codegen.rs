// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask codegen`: regenerates every generated artifact.
//!
//! | Output | Generator |
//! |---|---|
//! | `docs/reference/error-codes.md` | [`crate::codes`] (A03-T02) |
//! | `schemas/sonarithm-spec-0.3.schema.json` | `sonarithm_spec::json_schema` (A03-T06, RFC-0004, RFC-0005); the 0.1 and 0.2 files stay as written for older editors |
//! | `schemas/sonarithm-render-manifest-1.schema.json` | `sonarithm_export::manifest::json_schema` (A05-T08) |
//! | `schemas/sonarithm-profile-1.schema.json` | `sonarithm_app::profile::json_schema` (B03-T08, ADR-0020) |
//! | `packages/ipc/src/generated/protocol.ts` | `sonarithm_app::ipc::typescript` with ts-rs (B01-T02, ADR-0011, ADR-0017) |
//! | `packages/ipc/src/generated/fixtures.ts` | `sonarithm_app::ipc::typescript::fixtures` (a Rust-encoded binary view for the TypeScript reader's test) |
//!
//! The `docs / schema-diff` CI job runs this task and then fails on any diff, so a stale
//! generated file cannot be merged.

use std::path::Path;

use anyhow::{Context as _, Result};

use crate::codes;

pub(crate) fn run(root: &Path) -> Result<()> {
    codes::run(root)?;
    write(
        root,
        sonarithm_spec::schema::SCHEMA_PATH,
        &sonarithm_spec::json_schema(),
    )?;
    write(
        root,
        sonarithm_export::manifest::MANIFEST_SCHEMA_PATH,
        &sonarithm_export::manifest::json_schema(),
    )?;
    write(
        root,
        sonarithm_app::profile::SCHEMA_PATH,
        &sonarithm_app::profile::json_schema(),
    )?;
    write(
        root,
        sonarithm_app::ipc::typescript::PATH,
        &sonarithm_app::ipc::typescript::typescript(),
    )?;
    write(
        root,
        sonarithm_app::ipc::typescript::FIXTURES_PATH,
        &sonarithm_app::ipc::typescript::fixtures(),
    )
}

fn write(root: &Path, relative: &str, contents: &str) -> Result<()> {
    let path = root.join(relative);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    std::fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))?;
    println!("codegen: {relative}");
    Ok(())
}
