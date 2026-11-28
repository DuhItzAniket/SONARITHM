// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Test-only reference checks for ECHOMETRIC's detectors (ADR-0031). The detectors run on
//! `libm` so they give the same results on every platform; this crate's tests compare them
//! with established implementations (statrs, and the augurs crates from C05-T03), which
//! never run in a compile. The library itself is empty.

#![forbid(unsafe_code)]
