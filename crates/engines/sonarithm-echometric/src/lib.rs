// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ECHOMETRIC, SONARITHM's evaluation engine (docs/architecture/07-engines.md, ECHOMETRIC).
//!
//! Version 0 (roadmap A08) is the research instrument: the [`harness`] with stimulus
//! generators that re-create published study designs from a seed, and the study runner. The analytics half
//! (detectors, descriptions, queries and the perceptual lint) arrives in C05.
//!
//! Generated values reach the Score through gallery and study specs, so they are computed
//! with [`sonarithm_core::math`] and seeded streams and are identical on every platform
//! (ADR-0013); `clippy.toml` denies the `std` transcendental functions here. The engine
//! never depends on another engine (02 §3 rule 2).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(
    clippy::cast_precision_loss,
    reason = "row counts and indices become f64 positions; all stay below 2^53 (coding-standards §2.2)"
)]

pub mod codes;
pub mod harness;
