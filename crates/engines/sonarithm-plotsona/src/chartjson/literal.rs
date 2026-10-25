// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading JavaScript object and array literals as data, without running anything.
//!
//! Chart options in HTML pages are often JavaScript literals rather than JSON: unquoted
//! keys, single-quoted strings, trailing commas, comments, `undefined`. This reader accepts
//! exactly those and one pure constant expression, `Date.UTC(…)` with number literals,
//! which Highcharts examples use for dates. **Anything else** (a variable, a call, an
//! operator, a function, a template with `${…}`) is refused as "needs script": the value
//! is reported, never evaluated (OD-25).

use serde_json::{Map, Number, Value};

use crate::calibrate::ticks::civil_ms;

/// Why a literal could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LiteralError {
    /// The text needs a script to produce its value, at this byte offset.
    NeedsScript(usize),
    /// The text is not a literal at all, at this byte offset.
    Syntax(usize),
    /// The literal nests deeper than the budget.
    TooDeep,
}

/// Reads one literal at the start of `text` (after whitespace and comments) and returns it
/// with the number of bytes consumed.
pub(crate) fn parse(text: &str, max_depth: usize) -> Result<(Value, usize), LiteralError> {
    let mut reader = Reader {
        bytes: text.as_bytes(),
        text,
        at: 0,
        max_depth,
    };
    let value = reader.value(0)?;
    Ok((value, reader.at))
}

struct Reader<'a> {
    bytes: &'a [u8],
    text: &'a str,
    at: usize,
    max_depth: usize,
}

impl Reader<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_space(&mut self) {
        loop {
            match (self.peek(), self.bytes.get(self.at + 1).copied()) {
                (Some(b), _) if b.is_ascii_whitespace() => self.at += 1,
                (Some(b'/'), Some(b'/')) => {
                    while self.peek().is_some_and(|b| b != b'\n') {
                        self.at += 1;
                    }
                }
                (Some(b'/'), Some(b'*')) => {
                    self.at += 2;
                    while self.at < self.bytes.len()
                        && !(self.peek() == Some(b'*')
                            && self.bytes.get(self.at + 1) == Some(&b'/'))
                    {
                        self.at += 1;
                    }
                    self.at = (self.at + 2).min(self.bytes.len());
                }
                _ => return,
            }
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, LiteralError> {
        if depth > self.max_depth {
            return Err(LiteralError::TooDeep);
        }
        self.skip_space();
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"' | b'\'') => self.string().map(Value::String),
            Some(b'`') => self.template().map(Value::String),
            Some(b'-' | b'+' | b'.' | b'0'..=b'9') => self.number(),
            Some(_) => self.word(),
            None => Err(LiteralError::Syntax(self.at)),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, LiteralError> {
        self.at += 1;
        let mut map = Map::new();
        loop {
            self.skip_space();
            match self.peek() {
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Value::Object(map));
                }
                Some(b'"' | b'\'') => {
                    let key = self.string()?;
                    self.entry(key, &mut map, depth)?;
                }
                Some(b) if b.is_ascii_alphanumeric() || b == b'_' || b == b'$' => {
                    let key = self.identifier();
                    self.skip_space();
                    // A method (`key() {…}`) or a computed value is code, not data.
                    if self.peek() == Some(b'(') {
                        return Err(LiteralError::NeedsScript(self.at));
                    }
                    self.entry(key, &mut map, depth)?;
                }
                Some(b'[') => return Err(LiteralError::NeedsScript(self.at)),
                _ => return Err(LiteralError::Syntax(self.at)),
            }
            self.skip_space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {}
                _ => return Err(LiteralError::Syntax(self.at)),
            }
        }
    }

    fn entry(
        &mut self,
        key: String,
        map: &mut Map<String, Value>,
        depth: usize,
    ) -> Result<(), LiteralError> {
        self.skip_space();
        if self.peek() != Some(b':') {
            return Err(LiteralError::Syntax(self.at));
        }
        self.at += 1;
        let value = self.value(depth + 1)?;
        self.after_value()?;
        map.insert(key, value);
        Ok(())
    }

    fn array(&mut self, depth: usize) -> Result<Value, LiteralError> {
        self.at += 1;
        let mut items = Vec::new();
        loop {
            self.skip_space();
            match self.peek() {
                Some(b']') => {
                    self.at += 1;
                    return Ok(Value::Array(items));
                }
                Some(b',') => {
                    // A hole, as in `[1, , 3]`.
                    self.at += 1;
                    items.push(Value::Null);
                    continue;
                }
                _ => {}
            }
            items.push(self.value(depth + 1)?);
            self.after_value()?;
            self.skip_space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {}
                _ => return Err(LiteralError::Syntax(self.at)),
            }
        }
    }

    /// After a value only a separator or a closing bracket may follow; an operator means
    /// an expression.
    fn after_value(&mut self) -> Result<(), LiteralError> {
        self.skip_space();
        match self.peek() {
            Some(b',' | b'}' | b']') | None => Ok(()),
            Some(_) => Err(LiteralError::NeedsScript(self.at)),
        }
    }

    fn identifier(&mut self) -> String {
        let start = self.at;
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
        {
            self.at += 1;
        }
        self.text.get(start..self.at).unwrap_or_default().to_owned()
    }

    fn word(&mut self) -> Result<Value, LiteralError> {
        let start = self.at;
        let word = self.identifier();
        match word.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            // Not-a-number and infinity have no JSON form; they read as gaps.
            "null" | "undefined" | "NaN" | "Infinity" => Ok(Value::Null),
            "Date"
                if self
                    .text
                    .get(self.at..)
                    .is_some_and(|r| r.starts_with(".UTC(")) =>
            {
                self.at += ".UTC(".len();
                self.date_utc()
            }
            "" => Err(LiteralError::Syntax(start)),
            _ => Err(LiteralError::NeedsScript(start)),
        }
    }

    /// `Date.UTC(year, monthIndex, day, hours, minutes, seconds, ms)` with number literals:
    /// milliseconds since the epoch, months counted from 0 and carried as JavaScript does.
    fn date_utc(&mut self) -> Result<Value, LiteralError> {
        let mut args = Vec::new();
        loop {
            self.skip_space();
            if self.peek() == Some(b')') {
                self.at += 1;
                break;
            }
            if !matches!(self.peek(), Some(b'-' | b'+' | b'.' | b'0'..=b'9')) {
                return Err(LiteralError::NeedsScript(self.at));
            }
            match self.number()? {
                Value::Number(n) => args.push(n.as_f64().unwrap_or(f64::NAN)),
                _ => return Err(LiteralError::NeedsScript(self.at)),
            }
            self.skip_space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b')') => {}
                _ => return Err(LiteralError::NeedsScript(self.at)),
            }
        }
        let arg = |i: usize, default: f64| args.get(i).copied().unwrap_or(default);
        let (year, month) = (arg(0, f64::NAN), arg(1, 0.0));
        if !year.is_finite() || args.iter().any(|a| !a.is_finite() || a.fract() != 0.0) {
            return Err(LiteralError::NeedsScript(self.at));
        }
        #[expect(clippy::cast_possible_truncation, reason = "integral checked above")]
        let (year, month) = (year as i64, month as i64);
        let carried_year = year + month.div_euclid(12);
        let month = month.rem_euclid(12) + 1;
        #[expect(clippy::cast_possible_truncation, reason = "integral checked above")]
        let ms = civil_ms(carried_year, month, 1)
            + ((arg(2, 1.0) as i64 - 1) * 86_400_000)
            + (arg(3, 0.0) as i64) * 3_600_000
            + (arg(4, 0.0) as i64) * 60_000
            + (arg(5, 0.0) as i64) * 1000
            + arg(6, 0.0) as i64;
        Ok(Value::Number(Number::from(ms)))
    }

    fn number(&mut self) -> Result<Value, LiteralError> {
        let start = self.at;
        if matches!(self.peek(), Some(b'-' | b'+')) {
            self.at += 1;
        }
        if self.peek() == Some(b'0') && matches!(self.bytes.get(self.at + 1), Some(b'x' | b'X')) {
            self.at += 2;
            let digits = self.at;
            while self.peek().is_some_and(|b| b.is_ascii_hexdigit()) {
                self.at += 1;
            }
            let value = i64::from_str_radix(self.text.get(digits..self.at).unwrap_or(""), 16)
                .map_err(|_| LiteralError::Syntax(start))?;
            let negative = self.bytes.get(start) == Some(&b'-');
            return Ok(Value::Number(Number::from(if negative {
                -value
            } else {
                value
            })));
        }
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'_'))
            || (matches!(self.peek(), Some(b'-' | b'+'))
                && matches!(self.bytes.get(self.at.saturating_sub(1)), Some(b'e' | b'E')))
        {
            self.at += 1;
        }
        let text: String = self
            .text
            .get(start..self.at)
            .unwrap_or_default()
            .chars()
            .filter(|c| *c != '_' && *c != '+')
            .collect();
        if let Ok(i) = text.parse::<i64>() {
            return Ok(Value::Number(Number::from(i)));
        }
        text.parse::<f64>()
            .ok()
            .and_then(Number::from_f64)
            .map(Value::Number)
            .ok_or(LiteralError::Syntax(start))
    }

    fn string(&mut self) -> Result<String, LiteralError> {
        let quote = self.peek().unwrap_or(b'"');
        self.at += 1;
        let mut out = String::new();
        loop {
            let rest = self
                .text
                .get(self.at..)
                .ok_or(LiteralError::Syntax(self.at))?;
            let mut chars = rest.chars();
            let c = chars.next().ok_or(LiteralError::Syntax(self.at))?;
            self.at += c.len_utf8();
            match c {
                c if c as u32 == u32::from(quote) => return Ok(out),
                '\n' => return Err(LiteralError::Syntax(self.at)),
                '\\' => {
                    let e = chars.next().ok_or(LiteralError::Syntax(self.at))?;
                    self.at += e.len_utf8();
                    match e {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        '0' => out.push('\0'),
                        'u' => {
                            let hex = self
                                .text
                                .get(self.at..self.at + 4)
                                .ok_or(LiteralError::Syntax(self.at))?;
                            let code = u32::from_str_radix(hex, 16)
                                .map_err(|_| LiteralError::Syntax(self.at))?;
                            self.at += 4;
                            out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                        '\n' => {}
                        other => out.push(other),
                    }
                }
                other => out.push(other),
            }
        }
    }

    /// A template literal without substitutions is a string; one with `${…}` is code.
    fn template(&mut self) -> Result<String, LiteralError> {
        self.at += 1;
        let start = self.at;
        while let Some(b) = self.peek() {
            match b {
                b'`' => {
                    let text = self.text.get(start..self.at).unwrap_or_default().to_owned();
                    self.at += 1;
                    return Ok(text);
                }
                b'$' if self.bytes.get(self.at + 1) == Some(&b'{') => {
                    return Err(LiteralError::NeedsScript(self.at));
                }
                b'\\' => self.at += 2,
                _ => self.at += 1,
            }
        }
        Err(LiteralError::Syntax(self.at))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn read(text: &str) -> Result<Value, LiteralError> {
        parse(text, 64).map(|(v, _)| v)
    }

    #[test]
    fn javascript_literals_read_as_data() {
        let text = r#"{
            // Highcharts options as pages write them
            chart: { type: 'column' },
            title: { text: "Fruit \"sold\"" },
            xAxis: { categories: ['apple', 'banana',], },
            series: [{ name: 'Sold', data: [42, 57, .5, -1e3, 0x10, undefined, ] }],
            /* a comment */ 'quoted-key': `plain template`,
            when: Date.UTC(2020, 13, 1),
        }"#;
        let value = read(text).unwrap();
        assert_eq!(value["chart"], json!({"type": "column"}));
        assert_eq!(value["title"]["text"], "Fruit \"sold\"");
        assert_eq!(value["xAxis"]["categories"], json!(["apple", "banana"]));
        assert_eq!(
            value["series"][0]["data"],
            json!([42, 57, 0.5, -1000.0, 16, null])
        );
        assert_eq!(value["quoted-key"], "plain template");
        // Month 13 carries into February of the next year, as in JavaScript.
        assert_eq!(value["when"], json!(civil_ms(2021, 2, 1)));
    }

    #[test]
    fn anything_that_needs_a_script_is_refused() {
        for text in [
            "{ data: getData() }",
            "{ data: [1, 2].map(f) }",
            "{ formatter: function () { return 1; } }",
            "{ y: 1 + 2 }",
            "{ y: x }",
            "{ label: `${name}` }",
            "{ [key]: 1 }",
            "{ data() { return []; } }",
            "{ when: Date.UTC(year, 0, 1) }",
            "{ when: new Date(2020, 0, 1) }",
        ] {
            assert!(
                matches!(read(text), Err(LiteralError::NeedsScript(_))),
                "{text}: {:?}",
                read(text)
            );
        }
        assert!(matches!(read("{ a: "), Err(LiteralError::Syntax(_))));
        let deep = format!("{}{}", "[".repeat(100), "]".repeat(100));
        assert_eq!(read(&deep), Err(LiteralError::TooDeep));
    }

    #[test]
    fn the_reader_stops_after_one_literal() {
        let (value, used) = parse(" [1, 2], {layout: 1})", 8).unwrap();
        assert_eq!(value, json!([1, 2]));
        assert_eq!(used, 7);
    }
}
