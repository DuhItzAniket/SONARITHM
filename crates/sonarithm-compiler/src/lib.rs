// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM compiler: pipeline orchestration, the engine registry and diagnostics
//! aggregation. It compiles a spec plus data into a Score.
//!
//! The pipeline skeleton arrives in A03 and the first engine in A06
//! (docs/architecture/05-compiler-and-score-ir.md).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codes;
pub mod registry;

pub use registry::build_registry;
