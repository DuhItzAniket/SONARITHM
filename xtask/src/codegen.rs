// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask codegen`: regenerates every generated artifact.
//!
//! Nothing is generated yet. The JSON Schemas in `schemas/` arrive in A03 and the
//! TypeScript IPC types in `packages/ipc/src/generated/` in B01
//! (docs/engineering/dev-environment.md#common-commands). The `docs / schema-diff` CI job
//! already runs this task and then `git diff --exit-code`, so the check is in place before
//! the first generator lands.

use anyhow::Result;

#[expect(
    clippy::unnecessary_wraps,
    reason = "generators are fallible; the stub keeps the signature they need"
)]
pub(crate) fn run() -> Result<()> {
    println!("codegen: nothing to generate yet (JSON Schemas arrive in A03, IPC types in B01)");
    Ok(())
}
