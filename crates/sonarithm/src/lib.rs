// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM facade: the stable public API for compiling a sonification spec into a
//! Score and rendering it (`Sonarithm::compile`, `render_to_file`, builders).
//!
//! Skeleton created in phase A01; the API arrives in A07. Library features are re-exported
//! here behind feature flags (docs/architecture/02-repository-and-crate-layout.md §2, §4).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
