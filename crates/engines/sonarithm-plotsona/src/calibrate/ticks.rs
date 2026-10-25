// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading tick labels as numbers and dates, to turn tick marks into calibration anchors.
//!
//! Chart libraries format ticks in many ways. This reads the common ones and nothing more:
//!
//! - numbers with a sign (`-` or the Unicode minus `−`), thousands separators (`,`, thin
//!   and no-break spaces), a decimal point, an exponent (`1e3`), superscript powers of ten
//!   (`10³`, `10⁻²`), currency signs and a trailing `%` (read as written: `50%` is 50);
//! - SI suffixes as d3-format writes them: `k`, `M`, `G`, `T`, `B` (billions), `m`
//!   (milli), `µ`/`u`, `n`, `p`;
//! - ISO dates (`2020-03`, `2020-03-15`, `2020-03-15T10:30`), month names with a year
//!   (`Mar 2020`, `March 15, 2020`, `15 Mar 2020`), and the multi-scale ticks of d3 and
//!   Vega, where a year tick (`2020`) is followed by month ticks (`April`, `July`).
//!
//! A label it cannot read is skipped and reported (`SNR-ENG-326`), never guessed.

use sonarithm_core::Diagnostic;

use super::Anchor;
use crate::codes;

/// Milliseconds in a day.
const DAY_MS: i64 = 86_400_000;

/// A tick: its position along the axis and its label text.
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    /// The position along the axis, in drawing units.
    pub position: f64,
    /// The label as drawn.
    pub text: String,
}

impl Tick {
    /// A tick.
    #[must_use]
    pub fn new(position: f64, text: impl Into<String>) -> Self {
        Self {
            position,
            text: text.into(),
        }
    }
}

/// Numeric anchors from ticks; unreadable labels are skipped with one `SNR-ENG-326`
/// warning naming them.
#[must_use]
pub fn number_anchors(ticks: &[Tick], axis: &str) -> (Vec<Anchor>, Option<Diagnostic>) {
    let mut anchors = Vec::new();
    let mut unread = Vec::new();
    for tick in ticks {
        match parse_number(&tick.text) {
            Some(value) => anchors.push(Anchor::new(tick.position, value)),
            None => unread.push(tick.text.as_str()),
        }
    }
    (anchors, unread_labels(&unread, axis))
}

/// Date anchors (milliseconds since the epoch) from ticks, reading year-then-month
/// sequences in order; unreadable labels are skipped with one `SNR-ENG-326` warning.
#[must_use]
pub fn date_anchors(ticks: &[Tick], axis: &str) -> (Vec<Anchor>, Option<Diagnostic>) {
    let mut sorted: Vec<&Tick> = ticks.iter().collect();
    sorted.sort_by(|a, b| a.position.total_cmp(&b.position));
    let mut anchors = Vec::new();
    let mut unread = Vec::new();
    let mut year: Option<i64> = None;
    for tick in sorted {
        let text = tick.text.trim();
        let date = parse_date(text).or_else(|| {
            // A bare month continues the last year seen (d3 multi-scale ticks).
            let month = month_number(text)?;
            Some(civil_ms(year?, month, 1))
        });
        match date {
            Some(ms) => {
                year = Some(year_of(ms));
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "dates within ±285,000 years are exact in f64"
                )]
                anchors.push(Anchor::new(tick.position, ms as f64));
            }
            None => unread.push(tick.text.as_str()),
        }
    }
    (anchors, unread_labels(&unread, axis))
}

/// Whether most labels read as dates and few as plain numbers other than years, so the
/// axis is a date axis.
#[must_use]
pub fn looks_like_dates(ticks: &[Tick]) -> bool {
    let dated = ticks
        .iter()
        .filter(|t| {
            let text = t.text.trim();
            parse_date(text).is_some() || month_number(text).is_some()
        })
        .count();
    let non_year_numbers = ticks
        .iter()
        .filter(|t| parse_number(&t.text).is_some() && parse_year(t.text.trim()).is_none())
        .count();
    let has_non_year = ticks.iter().any(|t| parse_year(t.text.trim()).is_none());
    // Month names alone are categories unless some tick gives a year.
    let has_full_date = ticks.iter().any(|t| parse_date(t.text.trim()).is_some());
    dated * 2 > ticks.len() && non_year_numbers == 0 && has_non_year && has_full_date
}

fn unread_labels(unread: &[&str], axis: &str) -> Option<Diagnostic> {
    (!unread.is_empty()).then(|| {
        let shown: Vec<String> = unread.iter().take(5).map(|t| format!("`{t}`")).collect();
        Diagnostic::new(
            &codes::TICK_LABEL_UNREAD,
            format!(
                "{} {axis}-axis tick label(s) could not be read as values and were skipped: {}.",
                unread.len(),
                shown.join(", ")
            ),
        )
    })
}

/// Reads a tick label as a number (see the module documentation).
#[must_use]
pub fn parse_number(text: &str) -> Option<f64> {
    let mut s: String = text
        .trim()
        .chars()
        .filter(|c| !matches!(c, ',' | '\u{2009}' | '\u{202F}' | '\u{00A0}' | ' ' | '\''))
        .map(|c| if c == '\u{2212}' { '-' } else { c })
        .collect();
    for sign in ['$', '€', '£', '¥', '₹'] {
        s = s.replace(sign, "");
    }
    let s = s.strip_suffix('%').unwrap_or(&s);
    if s.is_empty() {
        return None;
    }
    if let Some(value) = superscript_power(s) {
        return Some(value);
    }
    if let Ok(value) = s.parse::<f64>() {
        return value.is_finite().then_some(value);
    }
    let (body, scale) = si_suffix(s)?;
    let value: f64 = body.parse().ok()?;
    let value = value * scale;
    value.is_finite().then_some(value)
}

/// `10³`, `10⁻²`, `2×10⁴`, `1e3` written with superscripts.
fn superscript_power(s: &str) -> Option<f64> {
    let (mantissa, rest) = if let Some(index) = s.find(['×', 'x', '·']) {
        let (m, r) = s.split_at(index);
        let r = r.chars().skip(1).collect::<String>();
        (m.parse::<f64>().ok()?, r)
    } else {
        (1.0, s.to_owned())
    };
    let exponent = rest.strip_prefix("10")?;
    if exponent.is_empty() {
        return None;
    }
    let mut digits = String::new();
    for c in exponent.chars() {
        digits.push(match c {
            '⁻' => '-',
            '⁰' => '0',
            '¹' => '1',
            '²' => '2',
            '³' => '3',
            '⁴' => '4',
            '⁵' => '5',
            '⁶' => '6',
            '⁷' => '7',
            '⁸' => '8',
            '⁹' => '9',
            _ => return None,
        });
    }
    let power: i32 = digits.parse().ok()?;
    let value = mantissa * libm::pow(10.0, f64::from(power));
    value.is_finite().then_some(value)
}

fn si_suffix(s: &str) -> Option<(&str, f64)> {
    let last = s.chars().last()?;
    let scale = match last {
        'k' | 'K' => 1e3,
        'M' => 1e6,
        'G' | 'B' => 1e9,
        'T' => 1e12,
        'm' => 1e-3,
        'µ' | 'μ' | 'u' => 1e-6,
        'n' => 1e-9,
        'p' => 1e-12,
        _ => return None,
    };
    Some((s.get(..s.len() - last.len_utf8())?, scale))
}

/// Reads a tick label as a date, in milliseconds since the epoch (UTC).
#[must_use]
pub fn parse_date(text: &str) -> Option<i64> {
    let text = text.trim();
    iso_date(text)
        .or_else(|| month_year(text))
        .or_else(|| parse_year(text).map(|y| civil_ms(y, 1, 1)))
}

/// A four-digit year from 1000 to 2999.
fn parse_year(text: &str) -> Option<i64> {
    (text.len() == 4 && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
        .filter(|y| (1000..3000).contains(y))
}

/// `YYYY-MM`, `YYYY-MM-DD`, optionally with `THH:MM[:SS]` or ` HH:MM[:SS]`.
fn iso_date(text: &str) -> Option<i64> {
    let (date, time) = match text.find(['T', ' ']) {
        Some(i) => (text.get(..i)?, Some(text.get(i + 1..)?)),
        None => (text, None),
    };
    let mut parts = date.split('-');
    let year = parse_year(parts.next()?)?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = match parts.next() {
        Some(d) => d.parse().ok()?,
        None => 1,
    };
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let mut ms = civil_ms(year, month, day);
    if let Some(time) = time {
        let time = time.trim_end_matches('Z');
        let mut hms = time.split(':');
        let h: i64 = hms.next()?.parse().ok()?;
        let m: i64 = hms.next()?.parse().ok()?;
        let s: f64 = hms.next().map_or(Some(0.0), |s| s.parse().ok())?;
        if !(0..24).contains(&h) || !(0..60).contains(&m) || !(0.0..61.0).contains(&s) {
            return None;
        }
        #[expect(clippy::cast_possible_truncation, reason = "seconds below 61")]
        let millis = (s * 1000.0).round() as i64;
        ms += (h * 60 + m) * 60_000 + millis;
    }
    Some(ms)
}

/// `Mar 2020`, `March 2020`, `Mar 15, 2020`, `15 Mar 2020`.
fn month_year(text: &str) -> Option<i64> {
    let words: Vec<&str> = text.split([' ', ',']).filter(|w| !w.is_empty()).collect();
    match words.as_slice() {
        [month, year] => Some(civil_ms(parse_year(year)?, month_number(month)?, 1)),
        [first, second, year] => {
            let year = parse_year(year)?;
            let (month, day) = match (month_number(first), month_number(second)) {
                (Some(m), None) => (m, second.parse().ok()?),
                (None, Some(m)) => (m, first.parse().ok()?),
                _ => return None,
            };
            (1..=31).contains(&day).then(|| civil_ms(year, month, day))
        }
        _ => None,
    }
}

/// The month number of an English month name or its three-letter abbreviation.
fn month_number(word: &str) -> Option<i64> {
    const MONTHS: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    let word = word.trim().trim_end_matches('.').to_ascii_lowercase();
    if word.len() < 3 {
        return None;
    }
    MONTHS
        .iter()
        .position(|name| *name == word || (word.len() == 3 && name.starts_with(&word)))
        .and_then(|i| i64::try_from(i + 1).ok())
}

/// Milliseconds since the epoch at the start of a civil (proleptic Gregorian) day, UTC.
/// Howard Hinnant's `days_from_civil`.
#[must_use]
pub fn civil_ms(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) * DAY_MS
}

/// The civil year of a time in milliseconds since the epoch (UTC).
fn year_of(ms: i64) -> i64 {
    let days = ms.div_euclid(DAY_MS) + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    yoe + era * 400 + i64::from(month <= 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_in_the_formats_charts_draw() {
        let cases: [(&str, f64); 16] = [
            ("0", 0.0),
            ("1,000", 1000.0),
            ("−5", -5.0),
            ("-2.5", -2.5),
            ("1.5k", 1500.0),
            ("20M", 2e7),
            ("3B", 3e9),
            ("500m", 0.5),
            ("2µ", 2e-6),
            ("1e3", 1000.0),
            ("10³", 1000.0),
            ("10⁻²", 0.01),
            ("2×10⁴", 20000.0),
            ("$1,250", 1250.0),
            ("50%", 50.0),
            ("1\u{2009}000\u{2009}000", 1e6),
        ];
        for (text, expected) in cases {
            let value = parse_number(text).unwrap_or(f64::NAN);
            assert!(
                (value - expected).abs() <= expected.abs() * 1e-12,
                "{text}: {value} != {expected}"
            );
        }
        for text in ["", "Q1", "abc", "1.2.3", "Jan", "10^"] {
            assert_eq!(parse_number(text), None, "{text}");
        }
    }

    #[test]
    fn dates_in_the_formats_charts_draw() {
        let day = |y, m, d| civil_ms(y, m, d);
        assert_eq!(civil_ms(1970, 1, 1), 0);
        assert_eq!(civil_ms(2000, 3, 1), 951_868_800_000);
        assert_eq!(parse_date("2020-03-15"), Some(day(2020, 3, 15)));
        assert_eq!(parse_date("2020-03"), Some(day(2020, 3, 1)));
        assert_eq!(
            parse_date("2020-03-15T10:30"),
            Some(day(2020, 3, 15) + 37_800_000)
        );
        assert_eq!(parse_date("Mar 2020"), Some(day(2020, 3, 1)));
        assert_eq!(parse_date("March 15, 2020"), Some(day(2020, 3, 15)));
        assert_eq!(parse_date("15 Mar 2020"), Some(day(2020, 3, 15)));
        assert_eq!(parse_date("2021"), Some(day(2021, 1, 1)));
        assert_eq!(parse_date("2020-13-01"), None);
        assert_eq!(parse_date("Q1 2020"), None);
        for ms in [0, day(1999, 12, 31), day(2024, 2, 29), day(1900, 1, 1)] {
            let back = year_of(ms);
            assert!(civil_ms(back, 1, 1) <= ms);
            assert!(civil_ms(back + 1, 1, 1) > ms);
        }
    }

    #[test]
    fn d3_multi_scale_ticks_continue_the_year() {
        let ticks = [
            Tick::new(0.0, "2020"),
            Tick::new(25.0, "April"),
            Tick::new(50.0, "July"),
            Tick::new(75.0, "October"),
            Tick::new(100.0, "2021"),
            Tick::new(125.0, "April"),
        ];
        assert!(looks_like_dates(&ticks));
        let (anchors, unread) = date_anchors(&ticks, "x");
        assert!(unread.is_none());
        #[expect(clippy::cast_precision_loss, reason = "test dates")]
        let expected: Vec<f64> = [
            civil_ms(2020, 1, 1),
            civil_ms(2020, 4, 1),
            civil_ms(2020, 7, 1),
            civil_ms(2020, 10, 1),
            civil_ms(2021, 1, 1),
            civil_ms(2021, 4, 1),
        ]
        .iter()
        .map(|ms| *ms as f64)
        .collect();
        let values: Vec<f64> = anchors.iter().map(|a| a.value).collect();
        assert_eq!(values, expected);
    }

    #[test]
    fn year_numbers_alone_are_numbers_not_dates() {
        let years = [Tick::new(0.0, "2019"), Tick::new(10.0, "2020")];
        assert!(!looks_like_dates(&years));
        let mixed = [Tick::new(0.0, "0"), Tick::new(10.0, "Jan 2020")];
        assert!(!looks_like_dates(&mixed));
        let months = [
            Tick::new(0.0, "Jan"),
            Tick::new(10.0, "Feb"),
            Tick::new(20.0, "Mar"),
        ];
        assert!(
            !looks_like_dates(&months),
            "month names alone are categories"
        );
    }

    #[test]
    fn unreadable_labels_are_skipped_and_reported() {
        let ticks = [
            Tick::new(0.0, "0"),
            Tick::new(10.0, "ten"),
            Tick::new(20.0, "20"),
        ];
        let (anchors, unread) = number_anchors(&ticks, "y");
        assert_eq!(anchors.len(), 2);
        let unread = unread.expect("reported");
        assert_eq!(unread.code.as_str(), "SNR-ENG-326");
        assert!(unread.message.contains("`ten`"));
    }
}
