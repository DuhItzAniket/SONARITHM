// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Foundational SONARITHM types: units (`Hz`, `Cents`, `Db`, `Seconds`, `Frames`), IDs,
//! the `Score` IR, engine capability traits, provenance and diagnostics.
//!
//! This crate performs no I/O and depends on no other SONARITHM crate, async runtime,
//! audio device, file-system or network crate (docs/architecture/02-repository-and-crate-layout.md
//! §3 rule 1, enforced by `cargo xtask check-deps`). Skeleton created in A01; types arrive in A03.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
