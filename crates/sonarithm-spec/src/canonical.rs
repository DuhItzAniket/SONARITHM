// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RFC 8785 JSON Canonicalization Scheme (JCS) and the spec hash (04 §7, ADR-0013).
//!
//! The canonical form has no whitespace, object members sorted by the UTF-16 code units of
//! their keys, strings escaped as ECMAScript `JSON.stringify` does, and numbers written as
//! ECMAScript `Number.prototype.toString` does. Number formatting takes the shortest
//! round-trip digits from `ryu`, which breaks exact ties to even as ECMAScript requires.
//! Rust's own float formatting breaks such ties upward and must not be used here (F-020).

use std::fmt::Write as _;

use serde::Serialize;
use serde_json::{Number, Value};
use sonarithm_core::digest::Sha256Digest;

/// Why a value has no canonical form.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalError {
    /// A number is not an IEEE 754 double exactly (an integer beyond 2^53, NaN, infinity).
    #[error("number {0} cannot be represented exactly as an IEEE 754 double")]
    Number(String),
    /// The value could not be converted to JSON.
    #[error("value could not be converted to JSON: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The input text is not JSON.
    #[error("input is not JSON: {0}")]
    Parse(String),
}

/// The RFC 8785 canonical form of JSON text.
///
/// # Errors
///
/// [`CanonicalError::Parse`] for invalid JSON; [`CanonicalError::Number`] for a number
/// that is not exactly a double.
pub fn canonicalize(text: &str) -> Result<String, CanonicalError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| CanonicalError::Parse(err.to_string()))?;
    canonical_value(&value)
}

/// The RFC 8785 canonical form of any serializable value.
///
/// # Errors
///
/// As for [`canonicalize`], plus [`CanonicalError::Serialize`].
pub fn to_canonical<T: Serialize + ?Sized>(value: &T) -> Result<String, CanonicalError> {
    canonical_value(&serde_json::to_value(value)?)
}

/// SHA-256 of the UTF-8 canonical form.
///
/// # Errors
///
/// As for [`to_canonical`].
pub fn canonical_hash<T: Serialize + ?Sized>(value: &T) -> Result<Sha256Digest, CanonicalError> {
    Ok(Sha256Digest::of(to_canonical(value)?.as_bytes()))
}

fn canonical_value(value: &Value) -> Result<String, CanonicalError> {
    let mut out = String::new();
    write_value(&mut out, value)?;
    Ok(out)
}

fn write_value(out: &mut String, value: &Value) -> Result<(), CanonicalError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&number(n)?),
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut members: Vec<(&String, &Value)> = map.iter().collect();
            members.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (i, (key, item)) in members.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_value(out, item)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

/// Escapes as ECMAScript `JSON.stringify`: the two-character escapes, `\u00xx` for other
/// control characters, everything else literal.
fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// The ECMAScript `Number.prototype.toString` form of a JSON number.
fn number(n: &Number) -> Result<String, CanonicalError> {
    let x = n
        .as_f64()
        .ok_or_else(|| CanonicalError::Number(n.to_string()))?;
    let exact = match (n.as_u64(), n.as_i64()) {
        (Some(u), _) => to_u64_exact(x) == Some(u),
        (None, Some(i)) => to_i64_exact(x) == Some(i),
        _ => true,
    };
    if !exact || !x.is_finite() {
        return Err(CanonicalError::Number(n.to_string()));
    }
    Ok(es_number(x))
}

fn to_u64_exact(x: f64) -> Option<u64> {
    ((0.0..18_446_744_073_709_551_616.0).contains(&x) && x.fract() == 0.0).then(|| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "range and integrality checked above"
        )]
        let u = x as u64;
        u
    })
}

fn to_i64_exact(x: f64) -> Option<i64> {
    ((-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&x) && x.fract() == 0.0)
        .then(|| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "range and integrality checked above"
            )]
            let i = x as i64;
            i
        })
}

/// ECMAScript `Number::toString(x)` for finite `x` (ECMA-262 §6.1.6.1.20), from the
/// shortest round-trip digits `s` (k digits) and exponent `n` with `x = s × 10^(n−k)`.
pub(crate) fn es_number(x: f64) -> String {
    if x == 0.0 {
        return "0".to_owned();
    }
    let mut buffer = ryu::Buffer::new();
    let text = buffer.format_finite(x.abs());
    let (mantissa, exponent) = match text.split_once('e') {
        Some((m, e)) => (m, e.parse::<i32>().unwrap_or(0)),
        None => (text, 0),
    };
    let point = mantissa.find('.').unwrap_or(mantissa.len());
    let mut digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let mut n = i32::try_from(point).unwrap_or(0) + exponent;
    while digits.starts_with('0') && digits.len() > 1 {
        digits.remove(0);
        n -= 1;
    }
    while digits.ends_with('0') && digits.len() > 1 {
        digits.pop();
    }
    let k = i32::try_from(digits.len()).unwrap_or(1);
    let sign = if x < 0.0 { "-" } else { "" };
    let body = if k <= n && n <= 21 {
        format!(
            "{digits}{}",
            "0".repeat(usize::try_from(n - k).unwrap_or(0))
        )
    } else if 0 < n && n <= 21 {
        let (int, frac) = digits.split_at(usize::try_from(n).unwrap_or(0));
        format!("{int}.{frac}")
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat(usize::try_from(-n).unwrap_or(0)))
    } else {
        let e = n - 1;
        let e_sign = if e < 0 { '-' } else { '+' };
        let (first, rest) = digits.split_at(1);
        if rest.is_empty() {
            format!("{first}e{e_sign}{}", e.abs())
        } else {
            format!("{first}.{rest}e{e_sign}{}", e.abs())
        }
    };
    format!("{sign}{body}")
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    /// The number test vectors of RFC 8785 Appendix B.
    #[test]
    fn rfc_8785_appendix_b_numbers() {
        let cases: &[(u64, &str)] = &[
            (0x0000_0000_0000_0000, "0"),
            (0x8000_0000_0000_0000, "0"),
            (0x0000_0000_0000_0001, "5e-324"),
            (0x8000_0000_0000_0001, "-5e-324"),
            (0x7fef_ffff_ffff_ffff, "1.7976931348623157e+308"),
            (0xffef_ffff_ffff_ffff, "-1.7976931348623157e+308"),
            (0x4340_0000_0000_0000, "9007199254740992"),
            (0xc340_0000_0000_0000, "-9007199254740992"),
            (0x4430_0000_0000_0000, "295147905179352830000"),
            (0x44b5_2d02_c7e1_4af5, "9.999999999999997e+22"),
            (0x44b5_2d02_c7e1_4af6, "1e+23"),
            (0x44b5_2d02_c7e1_4af7, "1.0000000000000001e+23"),
            (0x444b_1ae4_d6e2_ef4e, "999999999999999700000"),
            (0x444b_1ae4_d6e2_ef4f, "999999999999999900000"),
            (0x444b_1ae4_d6e2_ef50, "1e+21"),
            (0x3eb0_c6f7_a0b5_ed8c, "9.999999999999997e-7"),
            (0x3eb0_c6f7_a0b5_ed8d, "0.000001"),
            (0x41b3_de43_5555_5553, "333333333.3333332"),
            (0x41b3_de43_5555_5554, "333333333.33333325"),
            (0x41b3_de43_5555_5555, "333333333.3333333"),
            (0x41b3_de43_5555_5556, "333333333.3333334"),
            (0x41b3_de43_5555_5557, "333333333.33333343"),
            (0xbecb_f647_612f_3696, "-0.0000033333333333333333"),
            // An exact tie: ECMAScript picks the even digit; Rust's `{}` prints …206.3 (F-020).
            (0x4314_3ff3_c1cb_0959, "1424953923781206.2"),
        ];
        for (bits, expected) in cases {
            assert_eq!(es_number(f64::from_bits(*bits)), *expected, "{bits:#018x}");
        }
    }

    #[test]
    fn rfc_8785_section_3_2_example() {
        // The string member is written with JSON escapes; `b` is a backslash, so the escapes
        // stay escapes in this source file.
        let b = '\x5c';
        let string = format!("{b}u20ac${b}u000F{b}u000aA'{b}u0042{b}u0022{b}u005c{b}{b}{b}\"{b}/");
        let input = r#"{
          "numbers": [333333333.33333329, 1E30, 4.50, 2e-3, 0.000000000000000000000000001],
          "string": STRING,
          "literals": [null, true, false]
        }"#
        .replace("STRING", &format!("\"{string}\""));
        assert_eq!(
            canonicalize(&input).unwrap(),
            "{\"literals\":[null,true,false],\"numbers\":[333333333.3333333,1e+30,4.5,0.002,1e-27],\"string\":\"€$\\u000f\\nA'B\\\"\\\\\\\\\\\"/\"}"
        );
    }

    #[test]
    fn keys_sort_by_utf16_code_units() {
        // U+1F600 (a surrogate pair, 0xD83D…) sorts before U+FB01 in UTF-16 but after it in
        // UTF-8; RFC 8785 §3.2.3 requires the UTF-16 order.
        let input = "{\"\u{FB01}\": 1, \"\u{1F600}\": 2, \"a\": 3}";
        assert_eq!(
            canonicalize(input).unwrap(),
            "{\"a\":3,\"\u{1F600}\":2,\"\u{FB01}\":1}"
        );
    }

    #[test]
    fn integers_beyond_2_pow_53_are_rejected() {
        assert!(canonicalize("[9007199254740993]").is_err());
        assert_eq!(
            canonicalize("[9007199254740992]").unwrap(),
            "[9007199254740992]"
        );
    }

    fn json() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            (-1e12_f64..1e12).prop_map(|x| serde_json::json!(x)),
            "[a-z\u{e9}\u{1F600}]{0,4}".prop_map(Value::String),
        ];
        leaf.prop_recursive(3, 24, 4, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                prop::collection::btree_map("[a-z\u{FB01}]{1,3}", inner, 0..4)
                    .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        })
    }

    /// Writes `value` with object members in reverse key order and with whitespace.
    fn reordered(value: &Value) -> String {
        match value {
            Value::Object(map) => {
                let members: Vec<String> = map
                    .iter()
                    .rev()
                    .map(|(k, v)| format!("{} : {}", Value::String(k.clone()), reordered(v)))
                    .collect();
                format!("{{ {} }}", members.join(" , "))
            }
            Value::Array(items) => {
                let items: Vec<String> = items.iter().map(reordered).collect();
                format!("[ {} ]", items.join(" , "))
            }
            other => other.to_string(),
        }
    }

    proptest! {
        /// A03-T08 acceptance: key-order permutations hash equal.
        #[test]
        fn key_order_and_whitespace_do_not_change_the_hash(value in json()) {
            let canonical = canonical_value(&value).unwrap();
            prop_assert_eq!(canonicalize(&reordered(&value)).unwrap(), canonical.clone());
            prop_assert_eq!(canonicalize(&canonical).unwrap(), canonical);
        }
    }
}
