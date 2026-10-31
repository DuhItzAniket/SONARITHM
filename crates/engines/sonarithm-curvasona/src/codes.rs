// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CURVASONA's diagnostic codes, in its block of the shared `SNR-ENG` area:
//! `SNR-ENG-400` to `SNR-ENG-499` (coding-standards §4.2).
//!
//! 400–419 expressions and sampling; 420–439 critical points; 440–459 function cues;
//! 460–479 audification; 480–499 reserved. A released code is never reused.

use sonarithm_core::codes::{Area, CodeInfo, CodeStatus, CodeTable, validate};
use sonarithm_core::diagnostic::Severity;

/// The expression is not well formed.
pub const EXPR_SYNTAX: CodeInfo = CodeInfo {
    code: "SNR-ENG-400",
    severity: Severity::Error,
    summary: "The expression cannot be read",
    help: "Write the function in `x` with numbers, `+ - * / ^`, parentheses and functions \
           such as `sin(x)`. Write every product with `*` (`2*x`, not `2x`); the message \
           names the characters where reading stopped.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The expression names something that is not `x`, a constant, a declared variable or a
/// known function.
pub const EXPR_UNKNOWN_NAME: CodeInfo = CodeInfo {
    code: "SNR-ENG-401",
    severity: Severity::Error,
    summary: "The expression uses an unknown name",
    help: "An expression may use `x`, the constants pi, e and tau, the source's `variables` \
           and the functions listed in 07 CURVASONA. Declare other names in `variables`. \
           `log` is refused because it means ln in some books and log10 in others: write \
           ln(x) or log10(x).",
    rules: &[],
    status: CodeStatus::Active,
};

/// The expression is longer, deeper or larger than its budget.
pub const EXPR_TOO_LARGE: CodeInfo = CodeInfo {
    code: "SNR-ENG-402",
    severity: Severity::Error,
    summary: "The expression exceeds a size limit",
    help: "Expressions are untrusted input with budgets for length, nesting depth and size \
           (12 §6). Simplify the expression, or split it with `variables`.",
    rules: &[],
    status: CodeStatus::Active,
};

/// A name in `variables` cannot be used.
pub const VARIABLE_INVALID: CodeInfo = CodeInfo {
    code: "SNR-ENG-403",
    severity: Severity::Error,
    summary: "A variable cannot be used",
    help: "Variable names start with a letter, use letters, digits and `_`, are at most 32 \
           characters long and are not `x`, a constant or a function name. A source declares \
           at most 16 variables.",
    rules: &[],
    status: CodeStatus::Active,
};

/// The domain and `samples` or `step` ask for more samples than the budget allows.
pub const TOO_MANY_SAMPLES: CodeInfo = CodeInfo {
    code: "SNR-ENG-404",
    severity: Severity::Error,
    summary: "The function needs more samples than the limit",
    help: "Lower `samples`, raise `step` or narrow the `domain`. The limit keeps a function \
           source from exhausting memory or time.",
    rules: &[],
    status: CodeStatus::Active,
};

/// `dy` is a numerical derivative, because the symbolic one is not available.
pub const NUMERIC_DERIVATIVE: CodeInfo = CodeInfo {
    code: "SNR-ENG-405",
    severity: Severity::Info,
    summary: "The derivative is computed numerically",
    help: "The expression uses a function without a symbolic derivative (such as abs, floor \
           or min), or its derivative is larger than the size limit, so `dy` comes from \
           central differences. It is accurate where the function is smooth.",
    rules: &["A11Y-28"],
    status: CodeStatus::Active,
};

/// The function is undefined at some samples, whose `y` is missing.
pub const UNDEFINED_SAMPLES: CodeInfo = CodeInfo {
    code: "SNR-ENG-406",
    severity: Severity::Warning,
    summary: "The function is undefined at some samples",
    help: "Division by zero, roots and logarithms of negative numbers and overflows give no \
           value; those rows have no `y` and follow `data.missing`. Narrow the domain to \
           leave them out.",
    rules: &["A11Y-28"],
    status: CodeStatus::Active,
};

/// Every code CURVASONA emits.
pub const CODES: CodeTable = CodeTable {
    area: Area::Eng,
    owner: "sonarithm-curvasona",
    codes: &[
        EXPR_SYNTAX,
        EXPR_UNKNOWN_NAME,
        EXPR_TOO_LARGE,
        VARIABLE_INVALID,
        TOO_MANY_SAMPLES,
        NUMERIC_DERIVATIVE,
        UNDEFINED_SAMPLES,
    ],
};

const _: () = assert!(validate(&CODES).is_ok(), "invalid CURVASONA code table");
