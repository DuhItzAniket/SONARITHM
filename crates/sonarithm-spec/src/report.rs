// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spec diagnostics as `miette` reports, with the JSON pointer resolved to a span of the
//! spec text, so the CLI and editors can show the offending value (12 §1).
//!
//! `sonarithm-core` keeps `Diagnostic` free of rendering; this module adds the source
//! text, the span and the help from the code registry. miette's narratable handler gives a
//! plain-text rendering that reads well with a screen reader.

use std::fmt;

use miette::{LabeledSpan, NamedSource, SourceCode, SourceSpan};
use sonarithm_core::diagnostic::{Diagnostic, Diagnostics, Severity, SpecPointer};

use crate::codes::CODES;

/// One spec diagnostic with its source text, ready for a miette report handler.
#[derive(Debug)]
pub struct SpecReport {
    diagnostic: Diagnostic,
    help: Option<String>,
    source: NamedSource<String>,
    span: Option<SourceSpan>,
}

impl SpecReport {
    /// The underlying diagnostic.
    #[must_use]
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }

    /// The byte span the diagnostic points at, if the pointer resolves.
    #[must_use]
    pub fn span(&self) -> Option<(usize, usize)> {
        self.span.map(|span| (span.offset(), span.len()))
    }
}

/// Reports for `diagnostics` found in the spec text `source` named `name` (a file name,
/// never an absolute user path: P8).
#[must_use]
pub fn reports(name: &str, source: &str, diagnostics: &Diagnostics) -> Vec<SpecReport> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let span = diagnostic
                .location
                .as_ref()
                .and_then(|pointer| locate(source, pointer))
                .map(|(offset, len)| SourceSpan::from((offset, len)));
            let info = CODES
                .codes
                .iter()
                .find(|info| info.code == diagnostic.code.as_str());
            let mut help = info.map(|info| info.help.to_owned());
            if !diagnostic.rules.is_empty() {
                let rules: Vec<&str> = diagnostic
                    .rules
                    .iter()
                    .map(sonarithm_core::ids::RuleId::as_str)
                    .collect();
                let line = format!("Design rules: {}.", rules.join(", "));
                help = Some(help.map_or(line.clone(), |help| format!("{help} {line}")));
            }
            SpecReport {
                diagnostic: diagnostic.clone(),
                help,
                source: NamedSource::new(name, source.to_owned()),
                span,
            }
        })
        .collect()
}

impl fmt::Display for SpecReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.diagnostic.message)
    }
}

impl std::error::Error for SpecReport {}

impl miette::Diagnostic for SpecReport {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(&self.diagnostic.code))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(match self.diagnostic.severity {
            Severity::Info => miette::Severity::Advice,
            Severity::Warning => miette::Severity::Warning,
            Severity::Error => miette::Severity::Error,
        })
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.help
            .as_ref()
            .map(|help| Box::new(help) as Box<dyn fmt::Display>)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let span = self.span?;
        let label = self
            .diagnostic
            .location
            .as_ref()
            .map(SpecPointer::to_string);
        Some(Box::new(std::iter::once(LabeledSpan::new_with_span(
            label, span,
        ))))
    }
}

/// The byte offset and length of the value a JSON pointer names in `source`, or of the
/// deepest existing value on its path (for a missing key, the object that lacks it).
/// `None` if `source` is not valid JSON up to that value.
#[must_use]
pub fn locate(source: &str, pointer: &SpecPointer) -> Option<(usize, usize)> {
    let mut scanner = Scanner {
        text: source.as_bytes(),
        pos: 0,
    };
    scanner.skip_ws();
    let mut span = scanner.value_span()?;
    let path = pointer.as_str();
    if path.is_empty() {
        return Some(span);
    }
    for raw in path.split('/').skip(1) {
        let token = raw.replace("~1", "/").replace("~0", "~");
        scanner.pos = span.0;
        match scanner.child(&token) {
            Some(child) => span = child,
            None => break,
        }
    }
    Some(span)
}

/// A minimal JSON scanner: enough to walk objects and arrays and measure values.
struct Scanner<'a> {
    text: &'a [u8],
    pos: usize,
}

impl Scanner<'_> {
    fn peek(&self) -> Option<u8> {
        self.text.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> Option<()> {
        self.skip_ws();
        (self.peek() == Some(byte)).then(|| self.pos += 1)
    }

    /// The span of the value starting at `pos` (after whitespace); advances past it.
    fn value_span(&mut self) -> Option<(usize, usize)> {
        self.skip_ws();
        let start = self.pos;
        match self.peek()? {
            b'"' => {
                self.string()?;
            }
            open @ (b'{' | b'[') => {
                let close = if open == b'{' { b'}' } else { b']' };
                self.pos += 1;
                if self.eat(close).is_none() {
                    loop {
                        if open == b'{' {
                            self.skip_ws();
                            self.string()?;
                            self.eat(b':')?;
                        }
                        self.value_span()?;
                        if self.eat(b',').is_none() {
                            self.eat(close)?;
                            break;
                        }
                    }
                }
            }
            _ => {
                while self
                    .peek()
                    .is_some_and(|b| !matches!(b, b',' | b'}' | b']') && !b.is_ascii_whitespace())
                {
                    self.pos += 1;
                }
            }
        }
        Some((start, self.pos - start))
    }

    /// Skips a string at `pos` and returns its raw content (escapes left as written).
    fn string(&mut self) -> Option<&str> {
        if self.peek()? != b'"' {
            return None;
        }
        let start = self.pos + 1;
        self.pos += 1;
        loop {
            match self.peek()? {
                b'\\' => self.pos += 2,
                b'"' => break,
                _ => self.pos += 1,
            }
        }
        let raw = std::str::from_utf8(self.text.get(start..self.pos)?).ok();
        self.pos += 1;
        raw
    }

    /// The span of member `token` of the object or array at `pos`.
    fn child(&mut self, token: &str) -> Option<(usize, usize)> {
        self.skip_ws();
        match self.peek()? {
            b'{' => {
                self.pos += 1;
                if self.eat(b'}').is_some() {
                    return None;
                }
                loop {
                    self.skip_ws();
                    let key = self.string()?.replace("\\\"", "\"").replace("\\\\", "\\");
                    self.eat(b':')?;
                    let span = self.value_span()?;
                    if key == token {
                        return Some(span);
                    }
                    self.eat(b',')?;
                }
            }
            b'[' => {
                let index: usize = token.parse().ok()?;
                self.pos += 1;
                for i in 0.. {
                    let span = self.value_span()?;
                    if i == index {
                        return Some(span);
                    }
                    self.eat(b',')?;
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use miette::NarratableReportHandler;

    use super::*;
    use crate::{parse, validate};

    const SPEC: &str = r#"{
  "specVersion": "0.1",
  "data": { "source": { "kind": "file", "path": "a.csv" } },
  "tracks": [
    { "id": "y", "mark": "tone",
      "channels": { "pitch": { "field": "v", "range": { "hz": [1400, 400] } } } }
  ]
}"#;

    fn text_at(pointer: &str) -> &str {
        let mut at = SpecPointer::root();
        for token in pointer.split('/').skip(1) {
            at = match token.parse::<usize>() {
                Ok(index) => at.index(index),
                Err(_) => at.key(token),
            };
        }
        let (offset, len) = locate(SPEC, &at).unwrap();
        &SPEC[offset..offset + len]
    }

    #[test]
    fn pointers_resolve_to_the_value_text() {
        assert_eq!(text_at("/specVersion"), r#""0.1""#);
        assert_eq!(
            text_at("/tracks/0/channels/pitch/range"),
            r#"{ "hz": [1400, 400] }"#
        );
        assert_eq!(text_at("/tracks/0/channels/pitch/range/hz/1"), "400");
        // A missing key resolves to the object that lacks it.
        assert_eq!(
            text_at("/data/source/format"),
            r#"{ "kind": "file", "path": "a.csv" }"#
        );
        assert_eq!(locate("{", &SpecPointer::root()), None);
    }

    #[test]
    fn reports_carry_code_help_and_span_for_screen_reader_output() {
        let spec = parse(SPEC).unwrap();
        let diagnostics = validate(&spec, None);
        let reports = reports("demo.json", SPEC, &diagnostics);
        assert_eq!(reports.len(), 1);
        let mut text = String::new();
        NarratableReportHandler::new()
            .render_report(&mut text, &reports[0])
            .unwrap();
        assert!(text.contains("SNR-SPEC-031"), "{text}");
        assert!(
            text.contains("Pitch range [1400, 400] Hz is reversed."),
            "{text}"
        );
        assert!(text.contains("Design rules: SDR-15."), "{text}");
        assert!(text.contains("demo.json"), "{text}");
        assert_eq!(reports[0].span().map(|(_, len)| len), Some(21));
    }
}
