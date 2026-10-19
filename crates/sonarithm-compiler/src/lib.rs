// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The SONARITHM compiler: pipeline orchestration, the engine registry and diagnostics
//! aggregation. It compiles a spec plus data into a Score.
//!
//! The pipeline skeleton arrived in A03; A06 added the data layer as the compiler's data
//! (`DataBundle`, with transforms) and the platform data engine
//! (docs/architecture/05-compiler-and-score-ir.md).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codes;
pub mod data;
pub mod pipeline;
pub mod registry;

pub use data::Platform;
pub use pipeline::{Compiled, Compiler, CompilerData, Failure};
pub use registry::build_registry;
