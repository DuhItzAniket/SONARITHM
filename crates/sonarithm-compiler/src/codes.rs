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
