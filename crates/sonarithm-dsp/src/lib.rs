// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SONARITHM DSP: oscillators, envelopes, filters, panners and granular, wavetable and modal
//! nodes; ISO 226 loudness compensation and EBU R128 metering
//! (docs/architecture/06-audio-engine.md §3).
//!
//! Every node is real-time safe once constructed: processing allocates nothing, takes no
//! lock and cannot panic (coding-standards §11). Per-sample code uses IEEE 754 arithmetic
//! only, never the platform maths library, so renders are bit-identical across platforms
//! (F-039, ADR-0013); `clippy.toml` denies the `std` transcendental functions here.
//!
//! This crate is on the unsafe register (coding-standards §5), so it does not
//! `forbid(unsafe_code)`; the workspace-wide `deny` applies until registered unsafe code lands.

#![deny(missing_docs)]
#![allow(
    clippy::cast_precision_loss,
    reason = "buffer indices and frame counts become f32 or f64 sample positions (coding-standards §2.2)"
)]

pub mod env;
pub mod filter;
pub mod limiter;
pub mod loudness;
pub mod meter;
pub mod osc;
pub mod pan;
pub mod smooth;

#[cfg(test)]
mod conformance;
#[cfg(test)]
mod testing;

pub use env::{Adsr, AdsrParams, Stage, note_ramp};
pub use filter::{Biquad, BiquadKind, OnePole};
pub use limiter::TruePeakLimiter;
pub use loudness::Compensation;
pub use meter::{Loudness, LoudnessMeter};
pub use osc::{Oscillator, Waveform, sine_turns, sine_turns_unit};
pub use pan::PanGains;
pub use smooth::LinearSmoother;
