// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Locale formatting of numbers through ICU4X, with `Intl.NumberFormat`'s digit rules.

use std::str::FromStr;

use fixed_decimal::{Decimal, FloatPrecision, SignedRoundingMode, UnsignedRoundingMode};
use fluent_bundle::FluentValue;
use icu_decimal::DecimalFormatter;
use icu_decimal::options::{DecimalFormatterOptions, GroupingStrategy};
use icu_locale_core::Locale;
use intl_memoizer::Memoizable;
use intl_memoizer::concurrent::IntlLangMemoizer;
use unic_langid::LanguageIdentifier;

use crate::args::NumberOptions;

/// `Intl.NumberFormat`'s default maximum fraction digits for decimal numbers.
const DEFAULT_MAX_FRACTION: i16 = 3;

/// The decimal for `value` rounded and padded as `options` say, or `None` if `value` is not
/// finite.
///
/// The value starts from its shortest round-trip digits, as V8 and ICU do, so `1.005` with
/// two fraction digits is `1.01` on both sides.
pub(crate) fn decimal(value: f64, options: NumberOptions) -> Option<Decimal> {
    if !value.is_finite() {
        return None;
    }
    let mut decimal = Decimal::try_from_f64(value, FloatPrecision::RoundTrip).ok()?;
    let min_fraction = i16::from(options.min_fraction.unwrap_or(0));
    let max_fraction = options
        .max_fraction
        .map_or(DEFAULT_MAX_FRACTION.max(min_fraction), |max| {
            i16::from(max).max(min_fraction)
        });
    decimal.round_with_mode(
        -max_fraction,
        SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfExpand),
    );
    decimal.absolute.trim_end();
    decimal.absolute.pad_end(-min_fraction);
    if let Some(min_integer) = options.min_integer {
        decimal.absolute.pad_start(i16::from(min_integer));
    }
    Some(decimal)
}

/// `value` rounded to the digits `options` print, or `None` if it is not finite.
pub(crate) fn rounded(value: f64, options: NumberOptions) -> Option<f64> {
    let decimal = decimal(value, options)?;
    f64::from_str(&decimal.to_string()).ok()
}

/// A decimal formatter for one locale and grouping choice, memoized per bundle.
pub(crate) struct IcuDecimal(DecimalFormatter);

impl Memoizable for IcuDecimal {
    type Args = (bool,);
    type Error = ();

    fn construct(lang: LanguageIdentifier, (grouping,): Self::Args) -> Result<Self, ()> {
        new_formatter(&lang.to_string(), grouping).map(Self)
    }
}

/// A decimal formatter for `locale`, falling back to the root locale's data if the locale
/// is unknown to ICU4X.
pub(crate) fn new_formatter(locale: &str, grouping: bool) -> Result<DecimalFormatter, ()> {
    let locale = Locale::try_from_str(locale).map_err(|_| ())?;
    let mut options = DecimalFormatterOptions::default();
    options.grouping_strategy = Some(if grouping {
        GroupingStrategy::Auto
    } else {
        GroupingStrategy::Never
    });
    DecimalFormatter::try_new((&locale).into(), options).map_err(|_| ())
}

/// The Fluent formatter hook: numbers through ICU4X, everything else through Fluent.
pub(crate) fn fluent_formatter(value: &FluentValue, memo: &IntlLangMemoizer) -> Option<String> {
    let FluentValue::Number(number) = value else {
        return None;
    };
    let options = NumberOptions::from_fluent(&number.options);
    let decimal = decimal(number.value, options)?;
    memo.with_try_get::<IcuDecimal, _, _>((options.grouping,), |formatter| {
        formatter.0.format_to_string(&decimal)
    })
    .ok()
}
