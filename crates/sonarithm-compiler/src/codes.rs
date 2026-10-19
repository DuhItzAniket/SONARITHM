// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `SNR-COMP` diagnostic codes. 010–019: engines and the registry (05 §3); 020–029:
//! the compile pipeline and its budget. A released code is never reused
//! (coding-standards §4.2).

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

macro_rules! code {
    ($(#[$doc:meta])* $name:ident, $code:literal, $summary:literal, $help:literal) => {
        $(#[$doc])*
        pub const $name: CodeInfo = CodeInfo {
            code: $code,
            severity: Severity::Error,
            summary: $summary,
            help: $help,
            rules: &[],
            status: CodeStatus::Active,
        };
    };
}

code!(
    /// The spec needs an engine this build does not include (05 §3 rule 2).
    ENGINE_NOT_AVAILABLE,
    "SNR-COMP-010",
    "Engine not available in this build",
    "Use a SONARITHM build that includes the engine, or remove the spec keys it owns."
);
code!(
    /// An engine was registered twice.
    DUPLICATE_ENGINE,
    "SNR-COMP-011",
    "Engine registered twice",
    "Register each engine once when building the compiler."
);
code!(
    /// Two engines register the same source kind.
    DUPLICATE_SOURCE,
    "SNR-COMP-012",
    "Two engines register the same source kind",
    "Each source kind belongs to one engine; remove one of the registrations."
);
code!(
    /// Two engines register the same analyzer.
    DUPLICATE_ANALYZER,
    "SNR-COMP-013",
    "Two engines register the same analyzer",
    "Each analyzer ID belongs to one engine; remove one of the registrations."
);
code!(
    /// Conflicting encoder registration: a duplicate encoder or two default claims.
    CONFLICTING_ENCODER,
    "SNR-COMP-014",
    "Conflicting encoder registration",
    "Register one encoder per engine and mark, and let only one engine claim each mark's default."
);
code!(
    /// Two engines register the same shaper stage.
    DUPLICATE_SHAPER,
    "SNR-COMP-015",
    "Two engines register the same shaper stage",
    "Each shaper stage (quantize, loudness, spatial, lint) belongs to one engine."
);

code!(
    /// The compiled Score breaks an invariant of 05 §4.2: an engine bug.
    INVARIANT_BROKEN,
    "SNR-COMP-020",
    "The compiled Score breaks an invariant",
    "This is a bug in an engine; please report it with the spec that triggered it."
);
code!(
    /// The compile exceeded its event or duration budget.
    BUDGET_EXCEEDED,
    "SNR-COMP-021",
    "Compile budget exceeded",
    "Use less data, a slower pace or a shorter duration, or raise the budget."
);

/// `transform` is not applied because the compiler's data type cannot apply it.
pub const TRANSFORMS_NOT_APPLIED: CodeInfo = CodeInfo {
    severity: Severity::Warning,
    ..code_info(
        "SNR-COMP-022",
        "Transforms are not applied",
        "Compile with the data layer's `DataBundle`, which applies transforms (03 §6); otherwise the data is used as loaded.",
    )
};

/// No context provider (AUDAXIS) is registered.
pub const CONTEXT_UNAVAILABLE: CodeInfo = CodeInfo {
    severity: Severity::Warning,
    ..code_info(
        "SNR-COMP-023",
        "Context cues are not available in this build",
        "Ticks, the reference tone, the spoken legend and earcons come from AUDAXIS (B03).",
    )
};

code!(
    /// A data file cannot be read by the platform data engine.
    DATA_UNREADABLE,
    "SNR-COMP-024",
    "Data file cannot be read",
    "Check the path (relative to the spec) and the format."
);
code!(
    /// A spec transform cannot be handed to the data layer.
    TRANSFORM_UNREADABLE,
    "SNR-COMP-025",
    "Transform cannot run",
    "Check the transform's parameters against RFC-0001 §4."
);

const fn code_info(code: &'static str, summary: &'static str, help: &'static str) -> CodeInfo {
    CodeInfo {
        code,
        severity: Severity::Error,
        summary,
        help,
        rules: &[],
        status: CodeStatus::Active,
    }
}

/// Every `SNR-COMP` code, in ascending order.
pub const CODES: CodeTable = CodeTable {
    area: Area::Comp,
    owner: "sonarithm-compiler",
    codes: &[
        ENGINE_NOT_AVAILABLE,
        DUPLICATE_ENGINE,
        DUPLICATE_SOURCE,
        DUPLICATE_ANALYZER,
        CONFLICTING_ENCODER,
        DUPLICATE_SHAPER,
        INVARIANT_BROKEN,
        BUDGET_EXCEEDED,
        TRANSFORMS_NOT_APPLIED,
        CONTEXT_UNAVAILABLE,
        DATA_UNREADABLE,
        TRANSFORM_UNREADABLE,
    ],
};

const _: () = assert!(
    validate(&CODES).is_ok(),
    "the SNR-COMP code table is invalid; run `cargo test -p sonarithm-compiler codes` for details"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_table_is_valid() {
        assert_eq!(validate(&CODES), Ok(()));
    }
}
