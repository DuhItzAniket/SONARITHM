// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Storage-type inference for text values (CSV cells, JSON strings, spreadsheet text).
//!
//! Each value is classified on its own ([`classify`]); a column takes the type that at
//! least 95% of its non-missing values share (03 §3.1 states the threshold for ISO-8601
//! dates; numbers use the same one). Values that do not fit become missing, and the
//! [`ColumnReport`] counts them so the source can say so. Parsing is locale-independent:
//! `.` is the decimal separator unless [`TextOptions::decimal_comma`] is set (10 §2,
//! opt-in).

use std::sync::Arc;

use arrow_array::ArrayRef;
use arrow_array::builder::{
    BooleanBuilder, Date32Builder, Float64Builder, Int64Builder, StringBuilder,
    TimestampMicrosecondBuilder,
};
use jiff::fmt::temporal::{Pieces, PiecesOffset};

/// The share of non-missing values that must parse for a typed column (03 §3.1).
pub(crate) const TYPED_SHARE: f64 = 0.95;

/// Tokens that mean "missing" in typed columns. Text columns keep them as text (only empty
/// cells are missing there), so a country code `NA` is never lost.
const NULL_TOKENS: &[&str] = &[
    "", "NA", "N/A", "n/a", "na", "NULL", "null", "Null", "None", "none", "nil", "-", "--", "?",
    ".", "#N/A", "#NA",
];

/// Tokens for "not a number"; normalized to missing with a diagnostic (03 §5).
const NAN_TOKENS: &[&str] = &["NaN", "nan", "NAN", "-NaN", "-nan"];

/// Spreadsheet error values, which become missing with a warning.
const ERROR_TOKENS: &[&str] = &[
    "#DIV/0!", "#VALUE!", "#REF!", "#NAME?", "#NUM!", "#NULL!", "#SPILL!", "#CALC!",
];

/// Currency symbols accepted before or after a number; they become the field's unit.
const CURRENCIES: &[char] = &['$', '€', '£', '¥', '₹', '₩', '₽'];

/// Microseconds per day.
const MICROS_PER_DAY: i64 = 86_400_000_000;

/// Options that change how text is read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextOptions {
    /// Read `,` as the decimal separator and `.` or a space as the thousands separator.
    pub(crate) decimal_comma: bool,
    /// A `strftime`-style format for dates or date-times that are not ISO-8601, such as
    /// `%d.%m.%Y`. It applies to every column.
    pub(crate) date_format: Option<String>,
}

/// The storage type chosen for a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Storage {
    /// `true`/`false`.
    Bool,
    /// 64-bit integers.
    Int,
    /// 64-bit floats.
    Float,
    /// Calendar dates (Arrow `Date32`).
    Date,
    /// Date-times in microseconds (Arrow `Timestamp(µs)`), UTC when the text had offsets.
    Timestamp {
        /// At least one value carried an offset, so all are UTC instants.
        utc: bool,
    },
    /// Text.
    Text,
}

/// The order of a numeric date such as `03/04/2024`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DateOrder {
    /// Day first (`31/12/2024`).
    DayFirst,
    /// Month first (`12/31/2024`).
    MonthFirst,
}

/// One value, classified on its own.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Null,
    NaN,
    Error,
    Bool(bool),
    Int(i64, Decor),
    Float(f64, Decor),
    Date(i32),
    DateTime {
        micros: i64,
        offset: bool,
    },
    /// `a/b/yyyy` with both parts ≤ 31: day-first or month-first.
    NumericDate {
        a: u8,
        b: u8,
        year: i32,
    },
    Text,
}

/// Symbols around a number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Decor {
    currency: Option<char>,
    percent: bool,
    leading_zero: bool,
}

/// What inference found in a column, for diagnostics.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ColumnReport {
    /// Non-missing values.
    pub(crate) values: u64,
    /// `NaN` values that became missing.
    pub(crate) nan: u64,
    /// Spreadsheet error values that became missing.
    pub(crate) errors: u64,
    /// Infinite values that were kept.
    pub(crate) infinite: u64,
    /// Values that did not fit the column type and became missing.
    pub(crate) unparsed: u64,
    /// The first row of an unparsed value.
    pub(crate) first_unparsed: Option<u64>,
    /// The unit the values carried (`%` or a currency symbol).
    pub(crate) unit: Option<String>,
    /// Numeric dates whose order could not be decided, so the column stayed text.
    pub(crate) ambiguous_dates: bool,
    /// The order chosen for numeric dates.
    pub(crate) date_order: Option<DateOrder>,
}

/// Infers a column's type from its values and builds the typed array.
pub(crate) fn infer_column<'a, I>(
    values: I,
    options: &TextOptions,
) -> (ArrayRef, Storage, ColumnReport)
where
    I: Iterator<Item = &'a str> + Clone,
{
    let mut stats = Stats::default();
    for value in values.clone() {
        stats.add(classify(value, options));
    }
    let (storage, order) = stats.choose();
    let mut report = ColumnReport {
        values: stats.values,
        nan: stats.nan,
        errors: stats.errors,
        ambiguous_dates: order.is_none() && stats.numeric_dates > 0 && storage == Storage::Text,
        date_order: order,
        unit: stats.unit(),
        ..ColumnReport::default()
    };
    if storage == Storage::Text {
        report.nan = 0;
        report.errors = 0;
    }
    let array = build(values, storage, order, options, &mut report);
    (array, storage, report)
}

/// The storage type a sample of values suggests, without building an array.
pub(crate) fn storage_of<'a>(
    values: impl Iterator<Item = &'a str>,
    options: &TextOptions,
) -> Storage {
    let mut stats = Stats::default();
    for value in values {
        stats.add(classify(value, options));
    }
    stats.choose().0
}

/// Classifies a header candidate: whether the value would parse as `storage`.
pub(crate) fn fits(value: &str, storage: Storage, options: &TextOptions) -> bool {
    let cell = classify(value, options);
    match storage {
        Storage::Bool => matches!(cell, Cell::Bool(_) | Cell::Null),
        Storage::Int => matches!(cell, Cell::Int(..) | Cell::Null | Cell::NaN),
        Storage::Float => matches!(
            cell,
            Cell::Int(..) | Cell::Float(..) | Cell::Null | Cell::NaN
        ),
        Storage::Date | Storage::Timestamp { .. } => matches!(
            cell,
            Cell::Date(_) | Cell::DateTime { .. } | Cell::NumericDate { .. } | Cell::Null
        ),
        Storage::Text => true,
    }
}

#[derive(Default)]
struct Stats {
    values: u64,
    nan: u64,
    errors: u64,
    bools: u64,
    ints: u64,
    floats: u64,
    dates: u64,
    datetimes: u64,
    offsets: u64,
    numeric_dates: u64,
    /// Numeric dates that are invalid when read day-first.
    day_first_invalid: u64,
    /// Numeric dates that are invalid when read month-first.
    month_first_invalid: u64,
    leading_zeros: u64,
    percent: u64,
    currency: Option<char>,
    currency_count: u64,
    mixed_currency: bool,
}

impl Stats {
    fn add(&mut self, cell: Cell) {
        if !matches!(cell, Cell::Null | Cell::NaN | Cell::Error) {
            self.values += 1;
        }
        match cell {
            Cell::Null | Cell::Text => {}
            Cell::NaN => self.nan += 1,
            Cell::Error => self.errors += 1,
            Cell::Bool(_) => self.bools += 1,
            Cell::Int(_, decor) => {
                self.ints += 1;
                self.decor(decor);
            }
            Cell::Float(_, decor) => {
                self.floats += 1;
                self.decor(decor);
            }
            Cell::Date(_) => self.dates += 1,
            Cell::DateTime { offset, .. } => {
                self.datetimes += 1;
                self.offsets += u64::from(offset);
            }
            Cell::NumericDate { a, b, year } => {
                self.numeric_dates += 1;
                self.day_first_invalid += u64::from(civil_days(year, b, a).is_none());
                self.month_first_invalid += u64::from(civil_days(year, a, b).is_none());
            }
        }
    }

    fn decor(&mut self, decor: Decor) {
        self.leading_zeros += u64::from(decor.leading_zero);
        self.percent += u64::from(decor.percent);
        if let Some(symbol) = decor.currency {
            self.currency_count += 1;
            match self.currency {
                None => self.currency = Some(symbol),
                Some(seen) if seen != symbol => self.mixed_currency = true,
                Some(_) => {}
            }
        }
    }

    fn share(&self, count: u64) -> bool {
        // Counts are far below 2^53, so the conversion to f64 is exact.
        #[expect(clippy::cast_precision_loss, reason = "row counts are below 2^53")]
        let share = count as f64 / self.values as f64;
        self.values > 0 && share >= TYPED_SHARE
    }

    fn choose(&self) -> (Storage, Option<DateOrder>) {
        if self.values == 0 {
            return (Storage::Text, None);
        }
        if self.bools == self.values {
            return (Storage::Bool, None);
        }
        let numbers = self.ints + self.floats;
        if self.leading_zeros == 0 && self.share(numbers) {
            return (
                if self.floats == 0 {
                    Storage::Int
                } else {
                    Storage::Float
                },
                None,
            );
        }
        let order = match (self.day_first_invalid, self.month_first_invalid) {
            _ if self.numeric_dates == 0 => None,
            (0, invalid) if invalid > 0 => Some(DateOrder::DayFirst),
            (invalid, 0) if invalid > 0 => Some(DateOrder::MonthFirst),
            _ => None,
        };
        let temporal = self.dates
            + self.datetimes
            + if order.is_some() {
                self.numeric_dates
            } else {
                0
            };
        if temporal > 0 && self.share(temporal) {
            let storage = if self.datetimes == 0 {
                Storage::Date
            } else {
                Storage::Timestamp {
                    utc: self.offsets > 0,
                }
            };
            return (storage, order);
        }
        (Storage::Text, None)
    }

    fn unit(&self) -> Option<String> {
        let numbers = self.ints + self.floats;
        if numbers == 0 {
            return None;
        }
        if self.percent == numbers {
            return Some("%".to_owned());
        }
        match self.currency {
            Some(symbol) if !self.mixed_currency && self.currency_count == numbers => {
                Some(symbol.to_string())
            }
            _ => None,
        }
    }
}

/// Classifies one value.
fn classify(raw: &str, options: &TextOptions) -> Cell {
    let value = raw.trim();
    if NULL_TOKENS.contains(&value) {
        return Cell::Null;
    }
    if NAN_TOKENS.contains(&value) {
        return Cell::NaN;
    }
    if value.starts_with('#') && ERROR_TOKENS.contains(&value) {
        return Cell::Error;
    }
    if value.eq_ignore_ascii_case("true") {
        return Cell::Bool(true);
    }
    if value.eq_ignore_ascii_case("false") {
        return Cell::Bool(false);
    }
    if let Some(cell) = number(value, options.decimal_comma) {
        return cell;
    }
    if let Some(format) = &options.date_format {
        return custom_date(value, format).unwrap_or(Cell::Text);
    }
    iso_date(value)
        .or_else(|| numeric_date(value))
        .unwrap_or(Cell::Text)
}

/// Parses a number with optional sign, accounting parentheses, currency, percent and
/// thousands separators.
fn number(value: &str, decimal_comma: bool) -> Option<Cell> {
    let first = value.chars().next()?;
    if !(first.is_ascii_digit()
        || matches!(first, '+' | '-' | '.' | ',' | '(')
        || CURRENCIES.contains(&first)
        || first.eq_ignore_ascii_case(&'i'))
    {
        return None;
    }
    let mut decor = Decor::default();
    let mut rest = value;
    let mut negative = false;
    if let Some(inner) = rest.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        negative = true;
        rest = inner.trim();
    }
    if let Some(inner) = rest.strip_prefix('-') {
        negative = !negative;
        rest = inner.trim_start();
    } else if let Some(inner) = rest.strip_prefix('+') {
        rest = inner.trim_start();
    }
    if let Some(inner) = rest.strip_suffix('%') {
        decor.percent = true;
        rest = inner.trim_end();
    }
    if let Some(symbol) = rest.chars().next().filter(|c| CURRENCIES.contains(c)) {
        decor.currency = Some(symbol);
        rest = rest[symbol.len_utf8()..].trim_start();
        // `$-5` and `-$5` both occur.
        if let Some(inner) = rest.strip_prefix('-') {
            negative = !negative;
            rest = inner;
        }
    } else if let Some(symbol) = rest.chars().next_back().filter(|c| CURRENCIES.contains(c)) {
        decor.currency = Some(symbol);
        rest = rest[..rest.len() - symbol.len_utf8()].trim_end();
    }
    if decor.currency.is_some() && decor.percent {
        return None;
    }
    if rest.eq_ignore_ascii_case("inf") || rest.eq_ignore_ascii_case("infinity") {
        let infinity = if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        return Some(Cell::Float(infinity, decor));
    }
    let (canonical, is_float) = canonical_number(rest, decimal_comma)?;
    decor.leading_zero = !is_float && canonical.len() > 1 && canonical.starts_with('0');
    if !is_float && let Ok(magnitude) = canonical.parse::<u64>() {
        let int = if negative {
            0_i64.checked_sub_unsigned(magnitude)
        } else {
            i64::try_from(magnitude).ok()
        };
        if let Some(int) = int {
            return Some(Cell::Int(int, decor));
        }
    }
    let float: f64 = canonical.parse().ok()?;
    Some(Cell::Float(if negative { -float } else { float }, decor))
}

/// The digits of a number without grouping separators, with `.` as the decimal point, and
/// whether it has a fraction or exponent. `None` unless the text is a well-formed number.
fn canonical_number(text: &str, decimal_comma: bool) -> Option<(std::borrow::Cow<'_, str>, bool)> {
    let (decimal, groups): (u8, &[u8]) = if decimal_comma {
        (b',', b". \xa0")
    } else {
        (b'.', b",")
    };
    let bytes = text.as_bytes();
    let mantissa_end = bytes
        .iter()
        .position(|b| matches!(b, b'e' | b'E'))
        .unwrap_or(bytes.len());
    let (mantissa, exponent) = bytes.split_at(mantissa_end);
    let (int_part, frac_part) = match mantissa.iter().position(|&b| b == decimal) {
        Some(i) => (&mantissa[..i], Some(&mantissa[i + 1..])),
        None => (mantissa, None),
    };
    // Integer part: plain digits, or digits grouped in threes by one separator.
    let grouped = int_part.iter().any(|b| groups.contains(b));
    if grouped {
        let separator = *int_part.iter().find(|b| groups.contains(b))?;
        let mut parts = int_part.split(|&b| b == separator);
        let head = parts.next()?;
        if head.is_empty() || head.len() > 3 || !head.iter().all(u8::is_ascii_digit) {
            return None;
        }
        if !parts.all(|part| part.len() == 3 && part.iter().all(u8::is_ascii_digit)) {
            return None;
        }
    } else if !int_part.iter().all(u8::is_ascii_digit) {
        return None;
    }
    if let Some(frac) = frac_part
        && !frac.iter().all(u8::is_ascii_digit)
    {
        return None;
    }
    let has_digits = int_part.iter().any(u8::is_ascii_digit)
        || frac_part.is_some_and(|frac| frac.iter().any(u8::is_ascii_digit));
    if !has_digits {
        return None;
    }
    if !exponent.is_empty() {
        let digits = exponent.get(1..)?;
        let digits = digits
            .strip_prefix(b"-")
            .or_else(|| digits.strip_prefix(b"+"))
            .unwrap_or(digits);
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return None;
        }
    }
    let is_float = frac_part.is_some() || !exponent.is_empty();
    if !grouped && decimal == b'.' {
        return Some((std::borrow::Cow::Borrowed(text), is_float));
    }
    let mut canonical = String::with_capacity(text.len());
    for &b in bytes {
        if b == decimal {
            canonical.push('.');
        } else if !groups.contains(&b) {
            canonical.push(char::from(b));
        }
    }
    Some((std::borrow::Cow::Owned(canonical), is_float))
}

/// An ISO-8601 / RFC 3339 date or date-time.
fn iso_date(value: &str) -> Option<Cell> {
    // A quick shape check keeps plain words away from the full parser.
    let bytes = value.as_bytes();
    if bytes.len() < 10 || !bytes[..4].iter().all(u8::is_ascii_digit) || bytes[4] != b'-' {
        return None;
    }
    let pieces = Pieces::parse(value).ok()?;
    if pieces.time_zone_annotation().is_some() {
        return None;
    }
    let date = pieces.date();
    let days = civil_days(
        i32::from(date.year()),
        u8::try_from(date.month()).ok()?,
        u8::try_from(date.day()).ok()?,
    )?;
    let Some(time) = pieces.time() else {
        return Some(Cell::Date(days));
    };
    let offset = pieces.offset();
    let offset_seconds = match offset {
        None | Some(PiecesOffset::Zulu) => 0,
        Some(other) => i64::from(other.to_numeric_offset().seconds()),
    };
    let seconds =
        i64::from(time.hour()) * 3600 + i64::from(time.minute()) * 60 + i64::from(time.second());
    let micros = i64::from(days) * MICROS_PER_DAY
        + (seconds - offset_seconds) * 1_000_000
        + i64::from(time.subsec_nanosecond()) / 1000;
    Some(Cell::DateTime {
        micros,
        offset: offset.is_some(),
    })
}

/// `a/b/yyyy`, `a.b.yyyy` or `a-b-yyyy` with one- or two-digit parts.
fn numeric_date(value: &str) -> Option<Cell> {
    let separator = value.chars().find(|c| matches!(c, '/' | '.' | '-'))?;
    let mut parts = value.split(separator);
    let (a, b, year) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some()
        || year.len() != 4
        || a.is_empty()
        || b.is_empty()
        || a.len() > 2
        || b.len() > 2
    {
        return None;
    }
    let a: u8 = a.parse().ok()?;
    let b: u8 = b.parse().ok()?;
    let year: i32 = year.parse().ok()?;
    (civil_days(year, a, b).is_some() || civil_days(year, b, a).is_some())
        .then_some(Cell::NumericDate { a, b, year })
}

/// A date or date-time in a user-given `strftime` format.
fn custom_date(value: &str, format: &str) -> Option<Cell> {
    let parsed = jiff::fmt::strtime::parse(format, value).ok()?;
    if parsed.hour().is_some()
        && let Ok(datetime) = parsed.to_datetime()
    {
        let days = civil_days(
            i32::from(datetime.year()),
            u8::try_from(datetime.month()).ok()?,
            u8::try_from(datetime.day()).ok()?,
        )?;
        let seconds = i64::from(datetime.hour()) * 3600
            + i64::from(datetime.minute()) * 60
            + i64::from(datetime.second());
        let offset = parsed.offset();
        let micros = i64::from(days) * MICROS_PER_DAY
            + (seconds - offset.map_or(0, |o| i64::from(o.seconds()))) * 1_000_000
            + i64::from(datetime.subsec_nanosecond()) / 1000;
        return Some(Cell::DateTime {
            micros,
            offset: offset.is_some(),
        });
    }
    let date = parsed.to_date().ok()?;
    civil_days(
        i32::from(date.year()),
        u8::try_from(date.month()).ok()?,
        u8::try_from(date.day()).ok()?,
    )
    .map(Cell::Date)
}

/// Days since 1970-01-01 of a valid proleptic Gregorian date, or `None`
/// (H. Hinnant's `days_from_civil`, integer-only).
pub(crate) fn civil_days(year: i32, month: u8, day: u8) -> Option<i32> {
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i32::try_from(era * 146_097 + doe - 719_468).ok()
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 31,
    }
}

/// Builds the typed array, counting values that do not fit.
fn build<'a>(
    values: impl Iterator<Item = &'a str>,
    storage: Storage,
    order: Option<DateOrder>,
    options: &TextOptions,
    report: &mut ColumnReport,
) -> ArrayRef {
    let unparsed = |row: u64, report: &mut ColumnReport| {
        report.unparsed += 1;
        report.first_unparsed.get_or_insert(row);
    };
    match storage {
        Storage::Text => {
            let mut builder = StringBuilder::new();
            for value in values {
                if value.trim().is_empty() {
                    builder.append_null();
                } else {
                    builder.append_value(value);
                }
            }
            Arc::new(builder.finish())
        }
        Storage::Bool => {
            let mut builder = BooleanBuilder::new();
            for value in values {
                match classify(value, options) {
                    Cell::Bool(b) => builder.append_value(b),
                    _ => builder.append_null(),
                }
            }
            Arc::new(builder.finish())
        }
        Storage::Int => {
            let mut builder = Int64Builder::new();
            for (row, value) in (0_u64..).zip(values) {
                match classify(value, options) {
                    Cell::Int(int, _) => builder.append_value(int),
                    Cell::Null | Cell::NaN | Cell::Error => builder.append_null(),
                    _ => {
                        unparsed(row, report);
                        builder.append_null();
                    }
                }
            }
            Arc::new(builder.finish())
        }
        Storage::Float => {
            let mut builder = Float64Builder::new();
            for (row, value) in (0_u64..).zip(values) {
                match classify(value, options) {
                    // i64 → f64 rounds to nearest above 2^53, as every reader does.
                    #[expect(clippy::cast_precision_loss, reason = "documented rounding")]
                    Cell::Int(int, _) => builder.append_value(int as f64),
                    Cell::Float(float, _) => {
                        report.infinite += u64::from(float.is_infinite());
                        builder.append_value(float);
                    }
                    Cell::Null | Cell::NaN | Cell::Error => builder.append_null(),
                    _ => {
                        unparsed(row, report);
                        builder.append_null();
                    }
                }
            }
            Arc::new(builder.finish())
        }
        Storage::Date => {
            let mut builder = Date32Builder::new();
            for (row, value) in (0_u64..).zip(values) {
                match temporal(classify(value, options), order) {
                    Some(Temporal::Date(days)) => builder.append_value(days),
                    Some(Temporal::Missing) => builder.append_null(),
                    _ => {
                        unparsed(row, report);
                        builder.append_null();
                    }
                }
            }
            Arc::new(builder.finish())
        }
        Storage::Timestamp { utc } => {
            let mut builder = TimestampMicrosecondBuilder::new();
            for (row, value) in (0_u64..).zip(values) {
                match temporal(classify(value, options), order) {
                    Some(Temporal::Date(days)) => {
                        builder.append_value(i64::from(days) * MICROS_PER_DAY);
                    }
                    Some(Temporal::Micros(micros)) => builder.append_value(micros),
                    Some(Temporal::Missing) => builder.append_null(),
                    None => {
                        unparsed(row, report);
                        builder.append_null();
                    }
                }
            }
            let array = builder.finish();
            Arc::new(if utc {
                array.with_timezone("UTC")
            } else {
                array
            })
        }
    }
}

enum Temporal {
    Date(i32),
    Micros(i64),
    Missing,
}

fn temporal(cell: Cell, order: Option<DateOrder>) -> Option<Temporal> {
    match cell {
        Cell::Null | Cell::NaN | Cell::Error => Some(Temporal::Missing),
        Cell::Date(days) => Some(Temporal::Date(days)),
        Cell::DateTime { micros, .. } => Some(Temporal::Micros(micros)),
        Cell::NumericDate { a, b, year } => {
            let (month, day) = match order? {
                DateOrder::DayFirst => (b, a),
                DateOrder::MonthFirst => (a, b),
            };
            civil_days(year, month, day).map(Temporal::Date)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::Array;
    use arrow_array::cast::AsArray;
    use arrow_array::types::{Date32Type, Float64Type, Int64Type, TimestampMicrosecondType};
    use proptest::prelude::*;

    use super::*;

    fn infer(values: &[&str]) -> (ArrayRef, Storage, ColumnReport) {
        infer_column(values.iter().copied(), &TextOptions::default())
    }

    #[test]
    fn integers_floats_and_missing_tokens() {
        let (array, storage, report) = infer(&["1", " 2 ", "NA", "", "-3"]);
        assert_eq!(storage, Storage::Int);
        let ints = array.as_primitive::<Int64Type>();
        assert_eq!(
            ints.iter().collect::<Vec<_>>(),
            [Some(1), Some(2), None, None, Some(-3)]
        );
        assert_eq!(report.values, 3);

        let (array, storage, _) = infer(&["1", "2.5", "1e3", "-.5"]);
        assert_eq!(storage, Storage::Float);
        let floats = array.as_primitive::<Float64Type>();
        assert_eq!(floats.values().to_vec(), [1.0, 2.5, 1000.0, -0.5]);
    }

    #[test]
    fn nan_becomes_missing_and_infinity_is_kept() {
        let (array, storage, report) = infer(&["1.5", "NaN", "inf", "-Infinity"]);
        assert_eq!(storage, Storage::Float);
        assert_eq!(report.nan, 1);
        assert_eq!(report.infinite, 2);
        let floats = array.as_primitive::<Float64Type>();
        assert!(floats.is_null(1));
        assert_eq!(floats.value(3), f64::NEG_INFINITY);
    }

    #[test]
    fn thousands_currency_percent_and_parentheses() {
        let (array, storage, report) = infer(&["$1,234.50", "$-5", "($20)", "$0.75"]);
        assert_eq!(storage, Storage::Float);
        assert_eq!(report.unit.as_deref(), Some("$"));
        let floats = array.as_primitive::<Float64Type>();
        assert_eq!(floats.values().to_vec(), [1234.5, -5.0, -20.0, 0.75]);

        let (_, storage, report) = infer(&["12%", "7.5 %", "100%"]);
        assert_eq!(storage, Storage::Float);
        assert_eq!(report.unit.as_deref(), Some("%"));

        let (_, storage, _) = infer(&["1,23", "4,56"]);
        assert_eq!(
            storage,
            Storage::Text,
            "a comma is not a decimal point by default"
        );
        let options = TextOptions {
            decimal_comma: true,
            ..TextOptions::default()
        };
        let (array, storage, _) = infer_column(["1.234,5", "2,25", "3"].into_iter(), &options);
        assert_eq!(storage, Storage::Float);
        assert_eq!(
            array.as_primitive::<Float64Type>().values().to_vec(),
            [1234.5, 2.25, 3.0]
        );
    }

    #[test]
    fn leading_zeros_keep_identifiers_as_text() {
        let (array, storage, _) = infer(&["007", "012", "100"]);
        assert_eq!(storage, Storage::Text);
        assert_eq!(array.as_string::<i32>().value(0), "007");
        assert_eq!(infer(&["0", "10"]).1, Storage::Int);
    }

    #[test]
    fn dates_and_date_times() {
        let (array, storage, _) = infer(&["2024-02-29", "1970-01-01", ""]);
        assert_eq!(storage, Storage::Date);
        let dates = array.as_primitive::<Date32Type>();
        assert_eq!(dates.value(0), 19_782);
        assert_eq!(dates.value(1), 0);
        assert!(dates.is_null(2));

        let (array, storage, _) = infer(&["2024-01-01T00:00:01Z", "2024-01-01 01:00:00+01:00"]);
        assert_eq!(storage, Storage::Timestamp { utc: true });
        let stamps = array.as_primitive::<TimestampMicrosecondType>();
        assert_eq!(stamps.value(0) - stamps.value(1), 1_000_000);
        assert_eq!(stamps.timezone(), Some("UTC"));

        let (_, storage, _) = infer(&["2024-01-01T12:00", "2024-01-02"]);
        assert_eq!(storage, Storage::Timestamp { utc: false });
    }

    #[test]
    fn numeric_dates_need_a_decidable_order() {
        let (array, storage, report) = infer(&["31/12/2024", "01/02/2024"]);
        assert_eq!(
            (storage, report.date_order),
            (Storage::Date, Some(DateOrder::DayFirst))
        );
        assert_eq!(
            array.as_primitive::<Date32Type>().value(1),
            civil_days(2024, 2, 1).unwrap()
        );

        let (_, storage, report) = infer(&["12/31/2024", "1/2/2024"]);
        assert_eq!(report.date_order, Some(DateOrder::MonthFirst));
        assert_eq!(storage, Storage::Date);

        let (_, storage, report) = infer(&["01/02/2024", "03/04/2024"]);
        assert_eq!(storage, Storage::Text);
        assert!(report.ambiguous_dates);

        let options = TextOptions {
            date_format: Some("%d.%m.%Y".to_owned()),
            ..TextOptions::default()
        };
        let (_, storage, _) = infer_column(["01.02.2024", "03.04.2024"].into_iter(), &options);
        assert_eq!(storage, Storage::Date);
    }

    #[test]
    fn up_to_five_percent_of_values_may_be_junk() {
        let mut values = vec!["1.5"; 19];
        values.push("n.d.");
        let (array, storage, report) = infer(&values);
        assert_eq!(storage, Storage::Float);
        assert_eq!((report.unparsed, report.first_unparsed), (1, Some(19)));
        assert!(array.is_null(19));
        values.push("junk");
        assert_eq!(infer(&values).1, Storage::Text);
    }

    #[test]
    fn text_columns_keep_missing_tokens_as_text() {
        let (array, storage, _) = infer(&["NA", "EU", "", "AS"]);
        assert_eq!(storage, Storage::Text);
        let text = array.as_string::<i32>();
        assert_eq!(text.value(0), "NA");
        assert!(text.is_null(2));
    }

    #[test]
    fn booleans_and_spreadsheet_errors() {
        assert_eq!(infer(&["true", "FALSE", "True"]).1, Storage::Bool);
        let (array, storage, report) = infer(&["1", "#DIV/0!", "3"]);
        assert_eq!(storage, Storage::Int);
        assert_eq!(report.errors, 1);
        assert!(array.is_null(1));
    }

    #[test]
    fn civil_days_matches_known_dates() {
        assert_eq!(civil_days(1970, 1, 1), Some(0));
        assert_eq!(civil_days(2000, 3, 1), Some(11_017));
        assert_eq!(civil_days(1969, 12, 31), Some(-1));
        assert_eq!(civil_days(2023, 2, 29), None);
        assert_eq!(civil_days(2024, 13, 1), None);
    }

    proptest! {
        #[test]
        fn integers_round_trip_through_text(n in any::<i64>()) {
            let text = n.to_string();
            prop_assert_eq!(classify(&text, &TextOptions::default()), Cell::Int(n, Decor::default()));
        }

        #[test]
        fn floats_round_trip_through_text(x in any::<f64>().prop_filter("finite", |x| x.is_finite())) {
            let text = format!("{x:e}");
            match classify(&text, &TextOptions::default()) {
                Cell::Float(y, _) => prop_assert_eq!(x.to_bits(), y.to_bits()),
                other => prop_assert!(false, "{other:?}"),
            }
        }

        #[test]
        fn classification_never_panics(text in "\\PC{0,24}") {
            let _ = classify(&text, &TextOptions::default());
            let _ = classify(&text, &TextOptions { decimal_comma: true, date_format: Some("%d/%m/%Y".into()) });
        }
    }
}
