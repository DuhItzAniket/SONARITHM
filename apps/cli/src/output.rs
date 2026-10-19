// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Output: results on standard output, diagnostics on standard error, in one of three
//! forms (11 §2):
//!
//! - text on a terminal: sentences, with diagnostics drawn by miette's graphical handler;
//! - plain text (`--plain`, or when standard error is not a terminal): the same sentences,
//!   with diagnostics from miette's narratable handler and outlines indented, never drawn;
//! - JSON (`--json`): one pretty-printed object on standard output with stable field names,
//!   diagnostics included; nothing on standard error.
//!
//! No output uses colour alone to carry meaning: severities are words.

use std::io::{IsTerminal as _, Write as _};
use std::process::ExitCode;

use miette::{GraphicalReportHandler, GraphicalTheme, NarratableReportHandler};
use serde::Serialize;
use sonarithm::{Diagnostics, Severity};

/// The documented exit codes (11 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Exit {
    /// Success.
    Success,
    /// Errors: an invalid spec, a failed compile, a file that cannot be read or written.
    Errors,
    /// A usage error on the command line.
    Usage,
    /// Warnings, with `--deny-warnings`.
    WarningsDenied,
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> Self {
        Self::from(match exit {
            Exit::Success => 0,
            Exit::Errors => 1,
            Exit::Usage => 2,
            Exit::WarningsDenied => 3,
        })
    }
}

impl Exit {
    /// The exit for a finished command: errors win, then denied warnings.
    pub(crate) fn of(diagnostics: &Diagnostics, deny_warnings: bool) -> Self {
        if diagnostics.has_errors() {
            Self::Errors
        } else if deny_warnings && diagnostics.iter().any(|d| d.severity == Severity::Warning) {
            Self::WarningsDenied
        } else {
            Self::Success
        }
    }
}

/// How to print.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Out {
    /// One JSON object on standard output.
    Json,
    /// Linear text; narratable diagnostics.
    Plain,
    /// Text on a terminal; graphical diagnostics, coloured unless `NO_COLOR` is set.
    Drawn {
        /// Whether colour is allowed (it never carries meaning alone).
        colour: bool,
    },
}

impl Out {
    pub(crate) fn new(json: bool, plain: bool) -> Self {
        if json {
            Self::Json
        } else if plain || !std::io::stderr().is_terminal() {
            Self::Plain
        } else {
            // https://no-color.org: any non-empty value turns colour off.
            let colour = std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty());
            Self::Drawn { colour }
        }
    }

    pub(crate) fn json(self) -> bool {
        self == Self::Json
    }

    /// Whether outlines may use box-drawing characters.
    pub(crate) fn drawn(self) -> bool {
        matches!(self, Self::Drawn { .. })
    }

    /// A one-line note on standard error; none with `--json`, whose output says it all.
    pub(crate) fn note(self, message: &str) {
        if self.json() {
            return;
        }
        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(stderr, "note: {message}");
    }

    /// Diagnostics found in the spec text `source` named `name`, on standard error. `Info`
    /// diagnostics only with `verbose`.
    pub(crate) fn diagnostics(
        self,
        name: &str,
        source: &str,
        diagnostics: &Diagnostics,
        verbose: bool,
    ) {
        let shown = shown(diagnostics, verbose);
        let mut stderr = std::io::stderr().lock();
        for report in sonarithm::reports(name, source, &shown) {
            let mut text = String::new();
            let rendered = match self {
                Self::Drawn { colour } => {
                    let theme = if colour {
                        GraphicalTheme::unicode()
                    } else {
                        GraphicalTheme::unicode_nocolor()
                    };
                    GraphicalReportHandler::new_themed(theme).render_report(&mut text, &report)
                }
                Self::Plain | Self::Json => {
                    NarratableReportHandler::new().render_report(&mut text, &report)
                }
            };
            if rendered.is_err() {
                text = report.diagnostic().to_string();
            }
            let _ = writeln!(stderr, "{}", text.trim_end());
            let _ = writeln!(stderr);
        }
    }
}

/// One line on standard output. A closed pipe is not an error worth reporting.
pub(crate) fn line(text: impl AsRef<str>) {
    let text = text.as_ref();
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{text}");
}

/// Raw text on standard output (a schema, a migrated spec), ending with a newline.
pub(crate) fn raw(text: &str) {
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    if !text.ends_with('\n') {
        let _ = writeln!(stdout);
    }
}

/// `value` as pretty JSON on standard output.
pub(crate) fn json_value(value: &impl Serialize) {
    match serde_json::to_string_pretty(value) {
        Ok(text) => raw(&text),
        Err(err) => error(&format!("the JSON output could not be written: {err}")),
    }
}

/// A one-line error on standard error, for problems that are not spec diagnostics.
pub(crate) fn error(message: &str) {
    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "error: {message}");
}

/// The diagnostics to show: `Info` only with `verbose`.
pub(crate) fn shown(diagnostics: &Diagnostics, verbose: bool) -> Diagnostics {
    let mut shown = Diagnostics::new();
    for diagnostic in diagnostics {
        if verbose || diagnostic.severity != Severity::Info {
            shown.push(diagnostic.clone());
        }
    }
    shown
}

/// Counts as a sentence: "0 errors, 1 warning, 38 applied defaults".
pub(crate) fn counts(diagnostics: &Diagnostics) -> String {
    let count = |severity| {
        diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .count()
    };
    let (errors, warnings, info) = (
        count(Severity::Error),
        count(Severity::Warning),
        count(Severity::Info),
    );
    format!(
        "{errors} {}, {warnings} {}, {info} {}",
        if errors == 1 { "error" } else { "errors" },
        if warnings == 1 { "warning" } else { "warnings" },
        if info == 1 {
            "applied default or choice"
        } else {
            "applied defaults and choices"
        },
    )
}

/// The counts for JSON output.
#[derive(Debug, Serialize)]
pub(crate) struct Counts {
    errors: usize,
    warnings: usize,
    info: usize,
}

impl Counts {
    pub(crate) fn of(diagnostics: &Diagnostics) -> Self {
        let count = |severity| {
            diagnostics
                .iter()
                .filter(|d| d.severity == severity)
                .count()
        };
        Self {
            errors: count(Severity::Error),
            warnings: count(Severity::Warning),
            info: count(Severity::Info),
        }
    }
}
