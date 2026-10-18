// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask codegen`: regenerates every generated artifact.
//!
//! | Output | Generator |
//! |---|---|
//! | `docs/reference/error-codes.md` | [`crate::codes`] (A03-T02) |
//!
//! The JSON Schemas in `schemas/` join in A03-T06 and the TypeScript IPC types in
//! `packages/ipc/src/generated/` in B01 (docs/engineering/dev-environment.md#common-commands).
//! The `docs / schema-diff` CI job runs this task and then fails on any diff, so a stale
//! generated file cannot be merged.

use std::path::Path;

use anyhow::Result;

use crate::codes;

pub(crate) fn run(root: &Path) -> Result<()> {
    codes::run(root)
}
