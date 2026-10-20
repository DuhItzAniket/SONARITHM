// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask codegen`: regenerates every generated artifact.
//!
//! | Output | Generator |
//! |---|---|
//! | `docs/reference/error-codes.md` | [`crate::codes`] (A03-T02) |
//! | `schemas/sonarithm-spec-0.1.schema.json` | `sonarithm_spec::json_schema` (A03-T06) |
//! | `schemas/sonarithm-render-manifest-1.schema.json` | `sonarithm_export::manifest::json_schema` (A05-T08) |
//!
//! The TypeScript IPC types in `packages/ipc/src/generated/` join in B01
//! (docs/engineering/dev-environment.md#common-commands). The `docs / schema-diff` CI job
//! runs this task and then fails on any diff, so a stale generated file cannot be merged.

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
