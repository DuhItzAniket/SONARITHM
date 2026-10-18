// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The diagnostic code registry (coding-standards §4.2, 12 §1).
//!
//! Every crate that emits diagnostics declares one [`CodeTable`] for its area, next to the
//! code that emits them, and checks it at compile time:
//!
//! ```
//! use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
//! use sonarithm_core::diagnostic::Severity;
//!
//! pub const UNKNOWN_KEY: CodeInfo = CodeInfo {
//!     code: "SNR-SPEC-004",
//!     severity: Severity::Error,
//!     summary: "Unknown key",
//!     help: "Remove the key or fix its spelling.",
//!     rules: &[],
//!     status: CodeStatus::Active,
//! };
//!
//! pub const CODES: CodeTable = CodeTable {
//!     area: Area::Spec,
//!     owner: "sonarithm-spec",
//!     codes: &[UNKNOWN_KEY],
//! };
//!
//! // A duplicate, malformed or out-of-order code fails the build here.
//! const _: () = assert!(validate(&CODES).is_ok(), "invalid code table");
//! ```
//!
//! `cargo xtask codes` collects every table, checks them together and writes
//! `docs/reference/error-codes.md`. A code, once released, is never reused or renumbered;
//! a retired code stays in its table with [`CodeStatus::Retired`].

use std::fmt;

use crate::diagnostic::Severity;

/// A diagnostic area and the crate that owns it (coding-standards §4.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Area {
    /// `SNR-SPEC`: `sonarithm-spec`.
    Spec,
    /// `SNR-DATA`: `sonarithm-data`.
    Data,
    /// `SNR-COMP`: `sonarithm-compiler`.
    Comp,
    /// `SNR-LINT`: the ECHOMETRIC perceptual lint.
    Lint,
    /// `SNR-ENG`: engine crates.
    Eng,
    /// `SNR-RENDER`: `sonarithm-render`.
    Render,
    /// `SNR-AUDIO`: `sonarithm-audio`.
    Audio,
    /// `SNR-EXPORT`: `sonarithm-export`.
    Export,
    /// `SNR-APP`: `sonarithm-app`.
    App,
    /// `SNR-CLI`: `sonarithm-cli`.
    Cli,
    /// `SNR-PY`: `sonarithm-py`.
    Py,
}

impl Area {
    /// The code prefix, such as `SNR-SPEC-`.
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Spec => "SNR-SPEC-",
            Self::Data => "SNR-DATA-",
            Self::Comp => "SNR-COMP-",
            Self::Lint => "SNR-LINT-",
            Self::Eng => "SNR-ENG-",
            Self::Render => "SNR-RENDER-",
            Self::Audio => "SNR-AUDIO-",
            Self::Export => "SNR-EXPORT-",
            Self::App => "SNR-APP-",
            Self::Cli => "SNR-CLI-",
            Self::Py => "SNR-PY-",
        }
    }
}

/// Whether a code is in use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CodeStatus {
    /// Emitted by current code.
    Active,
    /// No longer emitted; kept so the code is never reused.
    Retired,
}

/// One registered diagnostic code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeInfo {
    /// The code, `SNR-<AREA>-<NNN>`.
    pub code: &'static str,
    /// The severity diagnostics with this code get by default.
    pub severity: Severity,
    /// A one-line meaning, in sentence case.
    pub summary: &'static str,
    /// How to fix it, for the reference page and `--explain`.
    pub help: &'static str,
    /// Design-rule IDs the code enforces, such as `SDR-10`.
    pub rules: &'static [&'static str],
    /// Whether the code is still emitted.
    pub status: CodeStatus,
}

/// The codes of one area, in ascending order.
#[derive(Clone, Copy, Debug)]
pub struct CodeTable {
    /// The area every code in the table belongs to.
    pub area: Area,
    /// The crate that owns the area.
    pub owner: &'static str,
    /// The codes, in strictly ascending order.
    pub codes: &'static [CodeInfo],
}

/// Why a code table is invalid. `index` is the position of the offending entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeTableError {
    /// The code does not start with the table's area prefix followed by three digits.
    Malformed {
        /// Position of the entry.
        index: usize,
    },
    /// The code repeats the previous one.
    Duplicate {
        /// Position of the second occurrence.
        index: usize,
    },
    /// The code is lower than the previous one.
    OutOfOrder {
        /// Position of the entry.
        index: usize,
    },
}

impl fmt::Display for CodeTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { index } => write!(f, "entry {index}: code is not SNR-<AREA>-<NNN>"),
            Self::Duplicate { index } => write!(f, "entry {index}: duplicate code"),
            Self::OutOfOrder { index } => write!(f, "entry {index}: codes are not ascending"),
        }
    }
}

impl std::error::Error for CodeTableError {}

/// Checks a table: every code has the area prefix and three digits, and codes are strictly
/// ascending, which rules out duplicates. Usable in a `const` item so a bad table fails the
/// build.
///
/// # Errors
///
/// The first problem found, as a [`CodeTableError`].
pub const fn validate(table: &CodeTable) -> Result<(), CodeTableError> {
    let prefix = table.area.prefix().as_bytes();
    let mut index = 0;
    while index < table.codes.len() {
        let code = table.codes[index].code.as_bytes();
        if !well_formed(code, prefix) {
            return Err(CodeTableError::Malformed { index });
        }
        if index > 0 {
            match compare(table.codes[index - 1].code.as_bytes(), code) {
                Comparison::Equal => return Err(CodeTableError::Duplicate { index }),
                Comparison::Greater => return Err(CodeTableError::OutOfOrder { index }),
                Comparison::Less => {}
            }
        }
        index += 1;
    }
    Ok(())
}

const fn well_formed(code: &[u8], prefix: &[u8]) -> bool {
    if code.len() != prefix.len() + 3 {
        return false;
    }
    let mut i = 0;
    while i < code.len() {
        let ok = if i < prefix.len() {
            code[i] == prefix[i]
        } else {
            code[i].is_ascii_digit()
        };
        if !ok {
            return false;
        }
        i += 1;
    }
    true
}

enum Comparison {
    Less,
    Equal,
    Greater,
}

/// Byte-wise comparison of two codes of the same length.
const fn compare(a: &[u8], b: &[u8]) -> Comparison {
    let mut i = 0;
    while i < a.len() && i < b.len() {
        if a[i] < b[i] {
            return Comparison::Less;
        }
        if a[i] > b[i] {
            return Comparison::Greater;
        }
        i += 1;
    }
    if a.len() < b.len() {
        Comparison::Less
    } else if a.len() > b.len() {
        Comparison::Greater
    } else {
        Comparison::Equal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn info(code: &'static str) -> CodeInfo {
        CodeInfo {
            code,
            severity: Severity::Error,
            summary: "Test code",
            help: "None.",
            rules: &[],
            status: CodeStatus::Active,
        }
    }

    fn table(codes: &'static [CodeInfo]) -> CodeTable {
        CodeTable {
            area: Area::Comp,
            owner: "sonarithm-compiler",
            codes,
        }
    }

    #[test]
    fn ascending_well_formed_codes_are_valid() {
        const CODES: &[CodeInfo] = &[info("SNR-COMP-001"), info("SNR-COMP-010")];
        assert_eq!(validate(&table(CODES)), Ok(()));
        assert_eq!(validate(&table(&[])), Ok(()));
    }

    #[test]
    fn duplicate_codes_are_rejected() {
        const CODES: &[CodeInfo] = &[info("SNR-COMP-010"), info("SNR-COMP-010")];
        assert_eq!(
            validate(&table(CODES)),
            Err(CodeTableError::Duplicate { index: 1 })
        );
    }

    #[test]
    fn out_of_order_codes_are_rejected() {
        const CODES: &[CodeInfo] = &[info("SNR-COMP-011"), info("SNR-COMP-010")];
        assert_eq!(
            validate(&table(CODES)),
            Err(CodeTableError::OutOfOrder { index: 1 })
        );
    }

    #[test]
    fn codes_from_another_area_or_malformed_are_rejected() {
        for bad in [
            "SNR-SPEC-010",
            "SNR-COMP-10",
            "SNR-COMP-0100",
            "SNR-COMP-01a",
            "snr-comp-010",
        ] {
            let codes: &'static [CodeInfo] = Box::leak(Box::new([info(bad)]));
            assert_eq!(
                validate(&table(codes)),
                Err(CodeTableError::Malformed { index: 0 }),
                "{bad}"
            );
        }
    }
}
