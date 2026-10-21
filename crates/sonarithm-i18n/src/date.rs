// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Locale formatting of calendar dates through ICU4X.
//!
//! Data dates are civil ISO dates (03 §3), so they are formatted in the proleptic Gregorian
//! calendar for every locale; only the field order, month names and separators follow the
//! locale. Times of day arrive with SONOTRACE streams (B06).

use icu_calendar::Date;
use icu_calendar::cal::Gregorian;
use icu_datetime::FixedCalendarDateTimeFormatter;
use icu_datetime::fieldsets::{MD, YM, YMD};
use icu_locale_core::Locale;

use crate::error::I18nError;

/// How long the month and year fields are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DateLength {
    /// "July 14, 2026" in English.
    Long,
    /// "Jul 14, 2026".
    Medium,
    /// "7/14/26".
    Short,
}

/// Which fields a date shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DatePattern {
    /// Year, month and day.
    YearMonthDay,
    /// Year and month ("July 2026"), for monthly segments.
    YearMonth,
    /// Month and day ("July 14"), for terse announcements inside a known year.
    MonthDay,
}

/// Formats `year-month-day` for `locale`.
pub(crate) fn format(
    locale: &Locale,
    (year, month, day): (i32, u8, u8),
    pattern: DatePattern,
    length: DateLength,
) -> Result<String, I18nError> {
    let invalid = || I18nError::InvalidDate { year, month, day };
    let date = Date::try_new_gregorian(year, month, day).map_err(|_| invalid())?;
    let prefs = locale.into();
    let formatted = match pattern {
        DatePattern::YearMonthDay => {
            let fields = match length {
                DateLength::Long => YMD::long(),
                DateLength::Medium => YMD::medium(),
                DateLength::Short => YMD::short(),
            };
            FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(prefs, fields)
                .map_err(|_| invalid())?
                .format(&date)
                .to_string()
        }
        DatePattern::YearMonth => {
            let fields = match length {
                DateLength::Long => YM::long(),
                DateLength::Medium => YM::medium(),
                DateLength::Short => YM::short(),
            };
            FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(prefs, fields)
                .map_err(|_| invalid())?
                .format(&date)
                .to_string()
        }
        DatePattern::MonthDay => {
            let fields = match length {
                DateLength::Long => MD::long(),
                DateLength::Medium => MD::medium(),
                DateLength::Short => MD::short(),
            };
            FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(prefs, fields)
                .map_err(|_| invalid())?
                .format(&date)
                .to_string()
        }
    };
    Ok(formatted)
}
