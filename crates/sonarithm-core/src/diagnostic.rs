// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! User-facing diagnostics (05 §7, 12 §1).
//!
//! A [`Diagnostic`] reports a problem the user can fix, or a default the compiler applied.
//! It carries a stable code from a [`crate::codes`] table, the design rules involved, a
//! message that states the cause and the fix, and where the problem is: a JSON pointer
//! into the spec and, for data problems, a row range. Rendering (miette in the CLI, the
//! diagnostics panel in the desktop app) happens at the edges, not here.

use std::borrow::Cow;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::codes::CodeInfo;
use crate::ids::{RowRange, RuleId};

/// How serious a diagnostic is. Ordered from least to most serious.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Information, such as an applied default and the rule it implements.
    Info,
    /// A likely problem; compilation continues (unless warnings are denied).
    Warning,
    /// A problem that stops compilation.
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        })
    }
}

/// A diagnostic code such as `SNR-SPEC-031` (coding-standards §4.2).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Code(Cow<'static, str>);

impl Code {
    /// A code from a string constant.
    #[must_use]
    pub const fn from_static(code: &'static str) -> Self {
        Self(Cow::Borrowed(code))
    }

    /// The code text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A JSON pointer (RFC 6901) into a spec document, such as `/tracks/0/channels/pitch`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpecPointer(String);

impl SpecPointer {
    /// The pointer to the whole document (the empty string).
    #[must_use]
    pub const fn root() -> Self {
        Self(String::new())
    }

    /// This pointer extended by an object key, escaped per RFC 6901 (`~` → `~0`,
    /// `/` → `~1`).
    #[must_use]
    pub fn key(&self, key: &str) -> Self {
        let mut pointer = self.0.clone();
        pointer.push('/');
        for c in key.chars() {
            match c {
                '~' => pointer.push_str("~0"),
                '/' => pointer.push_str("~1"),
                c => pointer.push(c),
            }
        }
        Self(pointer)
    }

    /// This pointer extended by an array index.
    #[must_use]
    pub fn index(&self, index: usize) -> Self {
        Self(format!("{}/{index}", self.0))
    }

    /// The pointer text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpecPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            f.write_str("/")
        } else {
            f.write_str(&self.0)
        }
    }
}

/// A problem or applied default, reported to the user (05 §7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Diagnostic {
    /// How serious it is.
    pub severity: Severity,
    /// The stable code, such as `SNR-SPEC-031`.
    pub code: Code,
    /// The design rules involved, such as `SDR-19`.
    pub rules: Vec<RuleId>,
    /// One or two sentences: the cause and the fix, with units, and no exclamation marks
    /// (12 §1).
    pub message: String,
    /// Where in the spec, as a JSON pointer.
    pub location: Option<SpecPointer>,
    /// Which data rows, for data problems.
    pub data: Option<RowRange>,
}

impl Diagnostic {
    /// A diagnostic with the code, default severity and rules of a registered code.
    #[must_use]
    pub fn new(code: &CodeInfo, message: impl Into<String>) -> Self {
        Self {
            severity: code.severity,
            code: Code::from_static(code.code),
            rules: code
                .rules
                .iter()
                .map(|rule| RuleId::from_static(rule))
                .collect(),
            message: message.into(),
            location: None,
            data: None,
        }
    }

    /// The same diagnostic located at `pointer` in the spec.
    #[must_use]
    pub fn at(mut self, pointer: SpecPointer) -> Self {
        self.location = Some(pointer);
        self
    }

    /// The same diagnostic attached to a range of data rows.
    #[must_use]
    pub fn rows(mut self, rows: RowRange) -> Self {
        self.data = Some(rows);
        self
    }

    /// The same diagnostic with another severity (for example a warning denied by the
    /// user's settings).
    #[must_use]
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Whether the diagnostic stops compilation.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Display for Diagnostic {
    /// `error SNR-SPEC-031 at /tracks/0/channels/pitch/range: <message>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.severity, self.code)?;
        if let Some(location) = &self.location {
            write!(f, " at {location}")?;
        }
        write!(f, ": {}", self.message)
    }
}

/// An ordered collection of diagnostics. Order is insertion order, which is deterministic
/// because every producer runs in a fixed order (coding-standards §10).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    /// An empty collection.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds a diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.0.push(diagnostic);
    }

    /// Whether any diagnostic is an error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.0.iter().any(Diagnostic::is_error)
    }

    /// The diagnostics in insertion order.
    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.0.iter()
    }

    /// The number of diagnostics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no diagnostics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The diagnostics as a vector.
    #[must_use]
    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.0
    }
}

impl Extend<Diagnostic> for Diagnostics {
    fn extend<I: IntoIterator<Item = Diagnostic>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::CodeStatus;

    const REVERSED_RANGE: CodeInfo = CodeInfo {
        code: "SNR-SPEC-031",
        severity: Severity::Error,
        summary: "A range is written high to low",
        help: "Write ranges low to high and use polarity to invert a mapping.",
        rules: &["SDR-15"],
        status: CodeStatus::Active,
    };

    #[test]
    fn pointers_escape_tilde_and_slash() {
        let pointer = SpecPointer::root().key("tracks").index(0).key("a/b~c");
        assert_eq!(pointer.as_str(), "/tracks/0/a~1b~0c");
        assert_eq!(SpecPointer::root().to_string(), "/");
    }

    #[test]
    fn diagnostics_take_code_severity_and_rules_from_the_registry() {
        let diagnostic =
            Diagnostic::new(&REVERSED_RANGE, "pitch range [1400, 400] Hz is reversed.")
                .at(SpecPointer::root().key("tracks").index(0));
        assert_eq!(diagnostic.code.as_str(), "SNR-SPEC-031");
        assert_eq!(diagnostic.rules, [RuleId::from_static("SDR-15")]);
        assert_eq!(
            diagnostic.to_string(),
            "error SNR-SPEC-031 at /tracks/0: pitch range [1400, 400] Hz is reversed."
        );
        let mut all = Diagnostics::new();
        all.push(diagnostic.clone().with_severity(Severity::Warning));
        assert!(!all.has_errors());
        all.push(diagnostic);
        assert!(all.has_errors());
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn diagnostics_serialize_for_ipc_and_python() {
        let diagnostic = Diagnostic::new(&REVERSED_RANGE, "reversed").at(SpecPointer::root());
        let json = serde_json::to_string(&diagnostic).unwrap();
        assert_eq!(
            json,
            r#"{"severity":"error","code":"SNR-SPEC-031","rules":["SDR-15"],"message":"reversed","location":"","data":null}"#
        );
        assert_eq!(
            serde_json::from_str::<Diagnostic>(&json).unwrap(),
            diagnostic
        );
    }
}
