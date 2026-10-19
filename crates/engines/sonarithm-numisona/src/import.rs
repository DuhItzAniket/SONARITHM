// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Importers from other sonification grammars into SONARITHM spec v0.1 (04 §9).
//!
//! Importers emit a report of every construct they could not map exactly; they never drop
//! one silently.

pub mod erie;

use sonarithm_core::diagnostic::{Diagnostic, SpecPointer};

use crate::codes;

impl erie::ImportReport {
    /// The report as diagnostics: `SNR-ENG-080` warnings for unmapped constructs and
    /// `SNR-ENG-081` notes for approximated ones, located by their pointer in the Erie
    /// spec (not in the SONARITHM spec), in report order.
    #[must_use]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.entries()
            .iter()
            .map(|entry| {
                let code = match entry.outcome {
                    erie::Outcome::Unmapped => &codes::ERIE_UNMAPPED,
                    erie::Outcome::Approximated => &codes::ERIE_APPROXIMATED,
                };
                let pointer = entry
                    .pointer
                    .split('/')
                    .skip(1)
                    .map(|part| part.replace("~1", "/").replace("~0", "~"))
                    .fold(SpecPointer::root(), |at, part| at.key(&part));
                Diagnostic::new(
                    code,
                    format!(
                        "NUMISONA Erie import: `{}` {}: {}",
                        entry.construct, entry.outcome, entry.note
                    ),
                )
                .at(pointer)
            })
            .collect()
    }
}
