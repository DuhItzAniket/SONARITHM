// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The evaluation harness (07, ECHOMETRIC `harness`; evaluation-plan "Study
//! infrastructure").
//!
//! [`generators`] re-create the stimuli of published studies from documented parameters
//! and a seed (docs/research-paper/datasets.md, GEN-001 to GEN-004). Each returns a
//! [`Stimulus`]: a numeric table, the ground truth that scores a response, and notes on
//! where it departs from the published design. The same parameters and seed give the same
//! bytes on every platform.
//!
//! [`study`] runs listening sessions from a protocol file: a seeded trial order, keyboard
//! responses scored against each stimulus's ground truth, a JSONL log, and pause and
//! resume.

pub mod generators;
pub mod stimulus;
pub mod study;

pub use generators::{GeneratorError, Params, generate};
pub use stimulus::{
    Column, ColumnKind, GeneratorId, Note, Query, Stimulus, StimulusManifest, Truth,
};
