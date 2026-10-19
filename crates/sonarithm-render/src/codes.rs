// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `SNR-RENDER` diagnostic codes. 001–009: plan building (05 §5). A released code is
//! never reused (coding-standards §4.2).

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

/// A track or note names an instrument preset the renderer does not have.
pub const UNKNOWN_INSTRUMENT: CodeInfo = CodeInfo {
    code: "SNR-RENDER-001",
    severity: Severity::Warning,
    summary: "Unknown instrument preset",
    help: "Use one of the presets mallet, sine, square, triangle or saw. Until then the mallet \
           preset, the default timbre, plays instead.",
    rules: &["SDR-29"],
    status: CodeStatus::Active,
};

/// Every `SNR-RENDER` code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Render,
    owner: "sonarithm-render",
    codes: &[UNKNOWN_INSTRUMENT],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid SNR-RENDER code table");
