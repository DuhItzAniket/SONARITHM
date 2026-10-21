// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Message catalogues and locale formatting for SONARITHM
//! (docs/architecture/12-cross-cutting-concerns.md §5, 08 §9).
//!
//! - **Catalogues.** Fluent messages live in `locales/<locale>/*.ftl` in this crate. The
//!   desktop UI (`@sonarithm/i18n`) loads the same files, so a message ID means the same text
//!   in Rust announcement templates and in the UI (NFR-09). The files are embedded at build
//!   time, so formatting does no I/O and the crate builds for `wasm32-unknown-unknown`.
//! - **Numbers** go through ICU4X with the rounding and digit options of
//!   `Intl.NumberFormat` (half-expand rounding, at most three fraction digits by default),
//!   so Rust and the UI print the same digits for the same value (the contract test in
//!   `tests/contract.rs` and in `packages/i18n`).
//! - **Dates** go through ICU4X in the proleptic Gregorian calendar ([`Localizer::date`]).
//! - **Units** have three forms: the symbol for visual text and braille, and terse and
//!   verbose spoken names (keyboard-and-navigation §8.1). [`unit_message`] maps a UCUM code
//!   or a unit symbol to its catalogue message.
//! - **Pseudo-locale.** [`PSEUDO_LOCALE`] (`en-XA`) renders the English catalogue with
//!   accented letters, about 40 % extra length and a marker pair around every message.
//!   [`check_pseudo`] rejects text that is not exactly one marked message, which catches
//!   hard-coded strings, concatenated messages and truncation.
//!
//! Message IDs are kebab-case with an area prefix: `unit-` for units, `common-` for shared
//! phrases, `ui-` for UI text and `announce-` for AUDAXIS announcement templates (B03).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod args;
mod catalogue;
mod date;
mod error;
mod localizer;
mod number;
mod pseudo;
mod units;

pub use args::{ArgValue, Args, Number, NumberOptions};
pub use catalogue::{CATALOGUE_LOCALES, SOURCE_LOCALE, catalogue_files};
pub use date::{DateLength, DatePattern};
pub use error::I18nError;
pub use localizer::Localizer;
pub use pseudo::{PSEUDO_CLOSE, PSEUDO_LOCALE, PSEUDO_OPEN, PseudoIssue, check_pseudo};
pub use units::{UNIT_MESSAGES, UnitRef, UnitStyle, unit_message};
