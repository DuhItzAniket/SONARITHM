// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! NUMISONA, SONARITHM's parameter-mapping engine: data values become auditory channels
//! (pitch, duration, taps, loudness, pan, timbre) through scales, pitch spaces and
//! polarity (docs/architecture/07-engines.md, NUMISONA; ADR-0007).
//!
//! The mapping of one channel runs in three steps, each in its own module:
//!
//! 1. A [`scale`] maps a datum to a position `u` in [0, 1] (04 §5.3).
//! 2. Polarity flips the position, `u → 1 − u`, when higher values should sound lower
//!    (SDR-12..15).
//! 3. The channel maps the position into its range; for pitch, through a pitch space.
//!
//! Every value that reaches the Score is computed with [`sonarithm_core::math`], so the
//! Score is byte-identical on every platform (ADR-0013); `clippy.toml` denies the `std`
//! transcendental functions here. The engine never depends on another engine (02 §3
//! rule 2).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod codes;
pub mod density;
pub mod encode;
pub mod examples;
pub mod import;
pub mod pitch;
pub mod polarity;
pub mod scale;
pub mod templates;

pub use encode::{Numisona, encode};
