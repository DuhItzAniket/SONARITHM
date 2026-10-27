// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A small PDF object reader (ISO 32000-1 §7.2–§7.5), written for PLOTSONA because no
//! maintained pure-Rust parser fits an engine crate (F-160).
//!
//! It reads only what chart extraction needs, and treats the file as hostile:
//!
//! - **Objects** are found by scanning for `N G obj` rather than trusting the
//!   cross-reference table, so a damaged or lying table cannot misdirect reads; a later
//!   definition of the same number wins, as incremental updates intend. Objects inside
//!   object streams (`/Type /ObjStm`) are read too.
//! - **Filters**: `FlateDecode`, `ASCIIHexDecode` and `ASCII85Decode`, each through the
//!   part and total decompression budgets. Other filters (images, LZW) are not decoded.
//! - **Budgets**: the object count (the node budget), nesting depth, decompressed sizes and
//!   work steps. Reference chains are followed a bounded number of times, so cycles end.
//! - **Encryption** is refused: a file with `/Encrypt` in its trailer is not read.

use std::collections::BTreeMap;
use std::io::Read;

use sonarithm_core::Diagnostic;

use crate::budget::{Budget, Budgets, Work, len64, over};
use crate::codes;

/// A PDF object.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Obj {
    Null,
    Bool(bool),
    Int(i64),
    Real(f64),
    Name(String),
    Str(Vec<u8>),
    Array(Vec<Obj>),
    Dict(BTreeMap<String, Obj>),
    /// A stream: its dictionary and its raw (still encoded) bytes.
    Stream(BTreeMap<String, Obj>, Vec<u8>),
    Ref(u32),
    /// An operator keyword, in content streams.
    Op(String),
}

impl Obj {
    pub(crate) fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(i) => {
                #[expect(clippy::cast_precision_loss, reason = "PDF numbers are small")]
                let v = *i as f64;
                Some(v)
            }
            Self::Real(r) => Some(*r),
            _ => None,
        }
    }

    pub(crate) fn as_name(&self) -> Option<&str> {
        match self {
            Self::Name(n) => Some(n),
            _ => None,
        }
    }

    pub(crate) fn dict(&self) -> Option<&BTreeMap<String, Obj>> {
        match self {
            Self::Dict(d) | Self::Stream(d, _) => Some(d),
            _ => None,
        }
    }
}

/// How deep reference chains are followed before giving up (cycles end here).
const MAX_REF_CHAIN: usize = 32;

/// A parsed PDF file: its objects by number, and its trailer.
pub(crate) struct File {
    objects: BTreeMap<u32, Obj>,
    pub(crate) trailer: BTreeMap<String, Obj>,
    budgets: Budgets,
    inflated: u64,
}

impl File {
    /// Reads every object of `bytes`.
    ///
    /// # Errors
    ///
    /// `SNR-ENG-311` for an encrypted file, the budget codes past a budget, and
    /// `SNR-ENG-307` when no object can be read.
    pub(crate) fn parse(
        bytes: &[u8],
        budgets: &Budgets,
        work: &mut Work,
    ) -> Result<Self, Box<Diagnostic>> {
        let mut file = Self {
            objects: BTreeMap::new(),
            trailer: BTreeMap::new(),
            budgets: *budgets,
            inflated: 0,
        };
        let depth = usize::try_from(budgets.max_depth).unwrap_or(usize::MAX);
        let mut at = 0;
        while let Some(found) = find(bytes, at, b"obj") {
            work.spend(1)?;
            at = found + 3;
            let Some(number) = object_header(bytes, found) else {
                continue;
            };
            let mut lexer = Lexer::new(bytes, found + 3, depth);
            let Ok(value) = lexer.object(0) else { continue };
            let value = match value {
                Obj::Dict(dict) => match lexer.stream_body(&dict) {
                    Some(raw) => Obj::Stream(dict, raw),
                    None => Obj::Dict(dict),
                },
                other => other,
            };
            at = lexer.at.max(at);
            file.insert(number, value)?;
        }
        // Trailers: the last `trailer` dictionary, or the last cross-reference stream's.
        let mut search = 0;
        while let Some(found) = find(bytes, search, b"trailer") {
            search = found + 7;
            let mut lexer = Lexer::new(bytes, search, depth);
            if let Ok(Obj::Dict(dict)) = lexer.object(0) {
                file.trailer.extend(dict);
            }
        }
        let xref_streams: Vec<BTreeMap<String, Obj>> = file
            .objects
            .values()
            .filter_map(|o| match o {
                Obj::Stream(d, _) if d.get("Type").and_then(Obj::as_name) == Some("XRef") => {
                    Some(d.clone())
                }
                _ => None,
            })
            .collect();
        for dict in xref_streams {
            for key in ["Root", "Encrypt", "Info"] {
                if let Some(v) = dict.get(key) {
                    file.trailer.insert(key.to_owned(), v.clone());
                }
            }
        }
        if file.trailer.contains_key("Encrypt") {
            return Err(Box::new(Diagnostic::new(
                &codes::ENCRYPTED,
                "The PDF is encrypted.",
            )));
        }
        file.expand_object_streams(work)?;
        if file.objects.is_empty() {
            return Err(Box::new(Diagnostic::new(
                &codes::MALFORMED,
                "The PDF has no readable objects.",
            )));
        }
        Ok(file)
    }

    fn insert(&mut self, number: u32, value: Obj) -> Result<(), Box<Diagnostic>> {
        self.objects.insert(number, value);
        let count = len64(self.objects.len());
        if count > u64::from(self.budgets.max_nodes) {
            return Err(over(
                Budget::Nodes,
                count,
                u64::from(self.budgets.max_nodes),
            ));
        }
        Ok(())
    }

    /// Reads the objects stored inside object streams; an object defined outside one keeps
    /// that definition.
    fn expand_object_streams(&mut self, work: &mut Work) -> Result<(), Box<Diagnostic>> {
        let streams: Vec<u32> = self
            .objects
            .iter()
            .filter(|(_, o)| {
                matches!(o, Obj::Stream(d, _) if d.get("Type").and_then(Obj::as_name) == Some("ObjStm"))
            })
            .map(|(n, _)| *n)
            .collect();
        let depth = usize::try_from(self.budgets.max_depth).unwrap_or(usize::MAX);
        for number in streams {
            let Some(stream) = self.objects.get(&number).cloned() else {
                continue;
            };
            let Some(data) = self.decode(&stream, work)? else {
                continue;
            };
            let dict = stream.dict().cloned().unwrap_or_default();
            let count = dict.get("N").and_then(Obj::as_f64).unwrap_or(0.0);
            let first = dict.get("First").and_then(Obj::as_f64).unwrap_or(0.0);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "checked below"
            )]
            let (count, first) = (count.max(0.0) as usize, first.max(0.0) as usize);
            let mut header = Lexer::new(&data, 0, depth);
            let mut entries = Vec::new();
            for _ in 0..count.min(data.len()) {
                work.spend(1)?;
                let (Ok(Obj::Int(n)), Ok(Obj::Int(offset))) = (header.object(0), header.object(0))
                else {
                    break;
                };
                entries.push((n, offset));
            }
            for (n, offset) in entries {
                let (Ok(n), Ok(offset)) = (u32::try_from(n), usize::try_from(offset)) else {
                    continue;
                };
                if self.objects.contains_key(&n) {
                    continue;
                }
                let mut lexer = Lexer::new(&data, first.saturating_add(offset), depth);
                if let Ok(value) = lexer.object(0) {
                    self.insert(n, value)?;
                }
            }
        }
        Ok(())
    }

    /// The object `obj` refers to, following references a bounded number of times.
    pub(crate) fn resolve<'a>(&'a self, obj: &'a Obj) -> &'a Obj {
        let mut current = obj;
        for _ in 0..MAX_REF_CHAIN {
            match current {
                Obj::Ref(n) => match self.objects.get(n) {
                    Some(next) => current = next,
                    None => return &Obj::Null,
                },
                other => return other,
            }
        }
        &Obj::Null
    }

    /// A dictionary entry, resolved.
    pub(crate) fn get<'a>(&'a self, dict: &'a BTreeMap<String, Obj>, key: &str) -> Option<&'a Obj> {
        dict.get(key)
            .map(|o| self.resolve(o))
            .filter(|o| **o != Obj::Null)
    }

    /// Objects of a given `/Type`, in object-number order.
    pub(crate) fn of_type(&self, kind: &str) -> Vec<&Obj> {
        self.objects
            .values()
            .filter(|o| o.dict().and_then(|d| d.get("Type")).and_then(Obj::as_name) == Some(kind))
            .collect()
    }

    /// A stream's decoded bytes, or `None` when a filter is not one PLOTSONA decodes.
    ///
    /// # Errors
    ///
    /// The decompression budget codes.
    pub(crate) fn decode(
        &mut self,
        stream: &Obj,
        work: &mut Work,
    ) -> Result<Option<Vec<u8>>, Box<Diagnostic>> {
        let Obj::Stream(dict, raw) = stream else {
            return Ok(None);
        };
        let filters: Vec<String> = match dict.get("Filter").map(|f| self.resolve(f)) {
            Some(Obj::Name(n)) => vec![n.clone()],
            Some(Obj::Array(a)) => a
                .iter()
                .filter_map(|f| self.resolve(f).as_name().map(str::to_owned))
                .collect(),
            _ => Vec::new(),
        };
        if dict.contains_key("DecodeParms")
            && dict
                .get("DecodeParms")
                .and_then(|p| self.resolve(p).dict().and_then(|d| d.get("Predictor")))
                .and_then(Obj::as_f64)
                .is_some_and(|p| p > 1.0)
        {
            return Ok(None);
        }
        let mut data = raw.clone();
        for filter in filters {
            work.spend(len64(data.len()))?;
            data = match filter.as_str() {
                "FlateDecode" | "Fl" => match self.inflate(&data)? {
                    Some(d) => d,
                    None => return Ok(None),
                },
                "ASCIIHexDecode" | "AHx" => hex(&data),
                "ASCII85Decode" | "A85" => match ascii85(&data) {
                    Some(d) => d,
                    None => return Ok(None),
                },
                _ => return Ok(None),
            };
        }
        Ok(Some(data))
    }

    fn inflate(&mut self, data: &[u8]) -> Result<Option<Vec<u8>>, Box<Diagnostic>> {
        let limit = self.budgets.max_part_bytes;
        let mut out = Vec::new();
        let mut reader = flate2::read::ZlibDecoder::new(data).take(limit.saturating_add(1));
        if reader.read_to_end(&mut out).is_err() && out.is_empty() {
            return Ok(None);
        }
        let size = len64(out.len());
        if size > limit {
            return Err(over(Budget::PartBytes, size, limit));
        }
        if size > Budgets::RATIO_FLOOR && size / len64(data.len()).max(1) > self.budgets.max_ratio {
            return Err(over(
                Budget::Ratio,
                size / len64(data.len()).max(1),
                self.budgets.max_ratio,
            ));
        }
        self.inflated = self.inflated.saturating_add(size);
        if self.inflated > self.budgets.max_inflated_bytes {
            return Err(over(
                Budget::InflatedBytes,
                self.inflated,
                self.budgets.max_inflated_bytes,
            ));
        }
        Ok(Some(out))
    }
}

/// The object number of `N G obj` ending just before `at` (the `obj` keyword).
fn object_header(bytes: &[u8], at: usize) -> Option<u32> {
    // `obj` must be a whole keyword.
    if bytes.get(at + 3).is_some_and(u8::is_ascii_alphanumeric) {
        return None;
    }
    let mut i = at;
    let skip_space = |i: &mut usize| {
        while *i > 0 && bytes.get(*i - 1).is_some_and(u8::is_ascii_whitespace) {
            *i -= 1;
        }
    };
    let digits = |i: &mut usize| {
        let end = *i;
        while *i > 0 && bytes.get(*i - 1).is_some_and(u8::is_ascii_digit) {
            *i -= 1;
        }
        (*i < end)
            .then(|| {
                std::str::from_utf8(bytes.get(*i..end)?)
                    .ok()?
                    .parse::<u32>()
                    .ok()
            })
            .flatten()
    };
    skip_space(&mut i);
    digits(&mut i)?;
    skip_space(&mut i);
    let number = digits(&mut i)?;
    Some(number)
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes
        .get(from..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

fn hex(data: &[u8]) -> Vec<u8> {
    let digits: Vec<u8> = data
        .iter()
        .take_while(|b| **b != b'>')
        .filter_map(|b| (*b as char).to_digit(16))
        .filter_map(|d| u8::try_from(d).ok())
        .collect();
    digits
        .chunks(2)
        .map(|c| c.first().copied().unwrap_or(0) * 16 + c.get(1).copied().unwrap_or(0))
        .collect()
}

fn ascii85(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut group: Vec<u32> = Vec::with_capacity(5);
    for &b in data {
        match b {
            b'~' => break,
            b'z' if group.is_empty() => out.extend([0; 4]),
            b'!'..=b'u' => {
                group.push(u32::from(b - b'!'));
                if group.len() == 5 {
                    let v = group
                        .iter()
                        .try_fold(0_u32, |acc, d| acc.checked_mul(85)?.checked_add(*d))?;
                    out.extend(v.to_be_bytes());
                    group.clear();
                }
            }
            b if b.is_ascii_whitespace() => {}
            _ => return None,
        }
    }
    if !group.is_empty() {
        let n = group.len();
        while group.len() < 5 {
            group.push(84);
        }
        let v = group
            .iter()
            .try_fold(0_u32, |acc, d| acc.checked_mul(85)?.checked_add(*d))?;
        out.extend(v.to_be_bytes().get(..n - 1)?);
    }
    Some(out)
}

/// A tokenizer and object parser over PDF syntax (also used for content streams).
pub(crate) struct Lexer<'a> {
    bytes: &'a [u8],
    pub(crate) at: usize,
    max_depth: usize,
}

fn is_delimiter(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

impl<'a> Lexer<'a> {
    pub(crate) const fn new(bytes: &'a [u8], at: usize, max_depth: usize) -> Self {
        Self {
            bytes,
            at,
            max_depth,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_space(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() || b == 0 {
                self.at += 1;
            } else if b == b'%' {
                while self.peek().is_some_and(|b| b != b'\n' && b != b'\r') {
                    self.at += 1;
                }
            } else {
                break;
            }
        }
    }

    fn regular(&mut self) -> &'a [u8] {
        let start = self.at;
        while self
            .peek()
            .is_some_and(|b| !b.is_ascii_whitespace() && !is_delimiter(b) && b != 0)
        {
            self.at += 1;
        }
        self.bytes.get(start..self.at).unwrap_or_default()
    }

    /// The next object, or `Err(())` at the end of input or on bad syntax. Numbers
    /// followed by `G R` become references.
    pub(crate) fn object(&mut self, depth: usize) -> Result<Obj, ()> {
        if depth > self.max_depth {
            return Err(());
        }
        self.skip_space();
        let b = self.peek().ok_or(())?;
        match b {
            b'/' => {
                self.at += 1;
                let raw = self.regular();
                Ok(Obj::Name(name(raw)))
            }
            b'(' => self.literal().map(Obj::Str),
            b'<' if self.bytes.get(self.at + 1) == Some(&b'<') => {
                self.at += 2;
                let mut dict = BTreeMap::new();
                loop {
                    self.skip_space();
                    if self.bytes.get(self.at..self.at + 2) == Some(b">>") {
                        self.at += 2;
                        return Ok(Obj::Dict(dict));
                    }
                    let Obj::Name(key) = self.object(depth + 1)? else {
                        return Err(());
                    };
                    let value = self.object(depth + 1)?;
                    dict.insert(key, value);
                }
            }
            b'<' => {
                self.at += 1;
                let start = self.at;
                while self.peek().is_some_and(|b| b != b'>') {
                    self.at += 1;
                }
                let data = hex(self.bytes.get(start..self.at).unwrap_or_default());
                self.at += 1;
                Ok(Obj::Str(data))
            }
            b'[' => {
                self.at += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_space();
                    match self.peek() {
                        Some(b']') => {
                            self.at += 1;
                            return Ok(Obj::Array(items));
                        }
                        None => return Err(()),
                        _ => items.push(self.object(depth + 1)?),
                    }
                }
            }
            b')' | b'>' | b']' | b'}' | b'{' => {
                self.at += 1;
                Err(())
            }
            _ => {
                let word = self.regular();
                if word.is_empty() {
                    self.at += 1;
                    return Err(());
                }
                Ok(self.word(word))
            }
        }
    }

    fn word(&mut self, word: &[u8]) -> Obj {
        let text = std::str::from_utf8(word).unwrap_or("");
        match text {
            "true" => return Obj::Bool(true),
            "false" => return Obj::Bool(false),
            "null" => return Obj::Null,
            _ => {}
        }
        if let Ok(i) = text.parse::<i64>() {
            // `N G R` is a reference.
            let save = self.at;
            self.skip_space();
            let generation = self.regular();
            self.skip_space();
            if !generation.is_empty()
                && generation.iter().all(u8::is_ascii_digit)
                && self.peek() == Some(b'R')
                && self
                    .bytes
                    .get(self.at + 1)
                    .is_none_or(|b| b.is_ascii_whitespace() || is_delimiter(*b))
                && let Ok(n) = u32::try_from(i)
            {
                self.at += 1;
                return Obj::Ref(n);
            }
            self.at = save;
            return Obj::Int(i);
        }
        if let Ok(r) = text.parse::<f64>() {
            return Obj::Real(r);
        }
        Obj::Op(text.to_owned())
    }

    fn literal(&mut self) -> Result<Vec<u8>, ()> {
        self.at += 1;
        let mut out = Vec::new();
        let mut depth = 1_usize;
        loop {
            let b = self.peek().ok_or(())?;
            self.at += 1;
            match b {
                b'(' => {
                    depth += 1;
                    out.push(b);
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(out);
                    }
                    out.push(b);
                }
                b'\\' => {
                    let e = self.peek().ok_or(())?;
                    self.at += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'0'..=b'7' => {
                            let mut v = u32::from(e - b'0');
                            for _ in 0..2 {
                                match self.peek() {
                                    Some(d @ b'0'..=b'7') => {
                                        v = v * 8 + u32::from(d - b'0');
                                        self.at += 1;
                                    }
                                    _ => break,
                                }
                            }
                            out.push(u8::try_from(v & 0xFF).unwrap_or(0));
                        }
                        b'\r' | b'\n' => {}
                        other => out.push(other),
                    }
                }
                other => out.push(other),
            }
        }
    }

    /// After a stream's dictionary: its raw bytes, by `/Length` when it is a direct number
    /// and ends at `endstream`, else up to the next `endstream`.
    fn stream_body(&mut self, dict: &BTreeMap<String, Obj>) -> Option<Vec<u8>> {
        self.skip_space();
        if self.bytes.get(self.at..self.at + 6) != Some(b"stream") {
            return None;
        }
        self.at += 6;
        if self.peek() == Some(b'\r') {
            self.at += 1;
        }
        if self.peek() == Some(b'\n') {
            self.at += 1;
        }
        let start = self.at;
        let by_length = dict
            .get("Length")
            .and_then(Obj::as_f64)
            .filter(|l| *l >= 0.0)
            .and_then(|l| {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "non-negative"
                )]
                let end = start.checked_add(l as usize)?;
                let after = self.bytes.get(end..)?;
                let trimmed = after
                    .iter()
                    .position(|b| !b.is_ascii_whitespace())
                    .unwrap_or(0);
                after
                    .get(trimmed..)?
                    .starts_with(b"endstream")
                    .then_some(end)
            });
        let end = by_length.or_else(|| find(self.bytes, start, b"endstream"))?;
        self.at = end;
        Some(self.bytes.get(start..end)?.to_vec())
    }
}

/// A name with `#xx` escapes decoded.
fn name(raw: &[u8]) -> String {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while let Some(&b) = raw.get(i) {
        if b == b'#'
            && let Some(hexd) = raw.get(i + 1..i + 3)
            && let Ok(v) = u8::from_str_radix(std::str::from_utf8(hexd).unwrap_or("zz"), 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Obj {
        Lexer::new(text.as_bytes(), 0, 64).object(0).unwrap()
    }

    #[test]
    fn objects_read_as_the_syntax_says() {
        assert_eq!(read("42"), Obj::Int(42));
        assert_eq!(read("-.5"), Obj::Real(-0.5));
        assert_eq!(read("/A#20B"), Obj::Name("A B".into()));
        assert_eq!(read("(a\\(b\\) \\101)"), Obj::Str(b"a(b) A".to_vec()));
        assert_eq!(read("<48 65 6>"), Obj::Str(b"He`".to_vec()));
        assert_eq!(read("12 0 R"), Obj::Ref(12));
        assert_eq!(
            read("[1 2 0 R /x]"),
            Obj::Array(vec![Obj::Int(1), Obj::Ref(2), Obj::Name("x".into())])
        );
        let Obj::Dict(d) = read("<< /Type /Page /Kids [3 0 R] >>") else {
            unreachable!()
        };
        assert_eq!(d.get("Type"), Some(&Obj::Name("Page".into())));
        assert_eq!(read("re"), Obj::Op("re".into()));
    }

    #[test]
    fn deep_nesting_stops_at_the_depth_limit() {
        let deep = "[".repeat(100);
        assert!(Lexer::new(deep.as_bytes(), 0, 32).object(0).is_err());
    }

    #[test]
    fn filters_decode() {
        assert_eq!(hex(b"48656c6c6f>"), b"Hello");
        assert_eq!(
            ascii85(b"87cURD]i,\"Ebo80~>").as_deref(),
            Some(&b"Hello World!"[..])
        );
    }

    /// A one-page PDF with one content stream, written by hand.
    fn tiny(content: &str, extra: &str) -> Vec<u8> {
        format!(
            "%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
             2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n\
             3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 300] /Contents 4 0 R \
             /Resources << /Font << /F1 5 0 R >> >> >> endobj\n\
             4 0 obj << /Length {} >>\nstream\n{content}\nendstream\nendobj\n\
             5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n\
             trailer << /Root 1 0 R {extra} >>\n%%EOF\n",
            content.len()
        )
        .into_bytes()
    }

    #[test]
    fn files_index_objects_and_refuse_encryption() {
        let bytes = tiny("0 0 m 1 1 l S", "");
        let file = File::parse(&bytes, &Budgets::DEFAULT, &mut Work::new(u64::MAX)).unwrap();
        assert_eq!(file.of_type("Page").len(), 1);
        let root = file.get(&file.trailer, "Root").and_then(Obj::dict).unwrap();
        assert!(root.contains_key("Pages"));
        let encrypted = tiny("", "/Encrypt 9 0 R");
        let err = File::parse(&encrypted, &Budgets::DEFAULT, &mut Work::new(u64::MAX))
            .err()
            .unwrap();
        assert_eq!(err.code.as_str(), "SNR-ENG-311");
    }

    #[test]
    fn reference_cycles_end() {
        let bytes = b"%PDF-1.7\n1 0 obj 2 0 R endobj\n2 0 obj 1 0 R endobj\ntrailer << >>\n";
        let file = File::parse(bytes, &Budgets::DEFAULT, &mut Work::new(u64::MAX)).unwrap();
        assert_eq!(file.resolve(&Obj::Ref(1)), &Obj::Null);
    }

    #[test]
    fn the_object_budget_is_enforced() {
        use std::fmt::Write as _;

        let mut text = String::from("%PDF-1.7\n");
        for i in 1..=50 {
            let _ = writeln!(text, "{i} 0 obj {i} endobj");
        }
        let mut budgets = Budgets::DEFAULT;
        budgets.max_nodes = 10;
        let err = File::parse(text.as_bytes(), &budgets, &mut Work::new(u64::MAX))
            .err()
            .unwrap();
        assert_eq!(err.code.as_str(), "SNR-ENG-306");
    }
}
