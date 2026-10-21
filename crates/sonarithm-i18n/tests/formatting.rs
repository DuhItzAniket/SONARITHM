// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Locale formatting of numbers, dates and quantities, and locale negotiation (NFR-09).

#![allow(
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use proptest::prelude::*;
use sonarithm_i18n::{
    DateLength, DatePattern, Localizer, Number, NumberOptions, UnitRef, UnitStyle, unit_message,
};

fn localizer(locale: &str) -> Localizer {
    Localizer::new(locale).unwrap_or_else(|e| panic!("{e}"))
}

fn number(localizer: &Localizer, value: f64, options: NumberOptions) -> String {
    localizer
        .number(Number::with_options(value, options))
        .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn nfr_09_numbers_follow_intl_number_format_digit_rules() {
    let en = localizer("en-US");
    let default = NumberOptions::default();
    assert_eq!(number(&en, 1234.5, default), "1,234.5");
    // At most three fraction digits by default, rounded half away from zero from the
    // shortest round-trip digits, as V8 does.
    assert_eq!(number(&en, 0.123_45, default), "0.123");
    assert_eq!(number(&en, 2.0005, default), "2.001");
    assert_eq!(number(&en, -2.0005, default), "-2.001");
    let two = NumberOptions {
        max_fraction: Some(2),
        ..default
    };
    assert_eq!(number(&en, 1.005, two), "1.01");
    assert_eq!(number(&en, 1.999, two), "2");
    assert_eq!(number(&en, 28.7, NumberOptions::fixed(2)), "28.70");
    assert_eq!(number(&en, 1980.0, default.without_grouping()), "1980");
    let padded = NumberOptions {
        min_integer: Some(3),
        ..default
    };
    assert_eq!(number(&en, 7.5, padded), "007.5");
}

#[test]
fn nfr_09_numbers_use_the_locale_separators() {
    assert_eq!(
        number(&localizer("en-IN"), 1_234_567.0, NumberOptions::default()),
        "12,34,567"
    );
    // A language without a catalogue falls back to English text and English formats, so
    // a sentence never mixes languages.
    let de = localizer("de-DE");
    assert_eq!(de.catalogue(), "en-US");
    assert_eq!(de.locale(), "en-US");
    assert_eq!(number(&de, 1234.5, NumberOptions::default()), "1,234.5");
}

#[test]
fn nfr_09_dates_follow_the_locale() {
    let date = |locale: &str, pattern, length| {
        localizer(locale)
            .date((2026, 7, 14), pattern, length)
            .unwrap_or_else(|e| panic!("{e}"))
    };
    assert_eq!(
        date("en-US", DatePattern::YearMonthDay, DateLength::Long),
        "July 14, 2026"
    );
    assert_eq!(
        date("en-GB", DatePattern::YearMonthDay, DateLength::Long),
        "14 July 2026"
    );
    assert_eq!(
        date("en-US", DatePattern::YearMonth, DateLength::Long),
        "July 2026"
    );
    assert_eq!(
        date("en-GB", DatePattern::MonthDay, DateLength::Medium),
        "14 Jul"
    );
    assert!(
        localizer("en-US")
            .date((2026, 2, 30), DatePattern::YearMonthDay, DateLength::Long)
            .is_err()
    );
}

#[test]
fn a11y_12_quantities_speak_unit_names_and_show_symbols() {
    let en = localizer("en-US");
    let celsius = UnitRef {
        text: "°C",
        ucum: Some("Cel"),
    };
    let value = Number::with_options(28.7, NumberOptions::fixed(1));
    let quantity = |unit, style| {
        en.quantity(value, unit, style)
            .unwrap_or_else(|e| panic!("{e}"))
    };
    assert_eq!(quantity(Some(celsius), UnitStyle::Symbol), "28.7 °C");
    assert_eq!(quantity(Some(celsius), UnitStyle::Terse), "28.7 degrees C");
    assert_eq!(
        quantity(Some(celsius), UnitStyle::Verbose),
        "28.7 degrees Celsius"
    );
    let unknown = UnitRef {
        text: "furlongs",
        ucum: None,
    };
    assert_eq!(quantity(Some(unknown), UnitStyle::Verbose), "28.7 furlongs");
    assert_eq!(quantity(None, UnitStyle::Verbose), "28.7");
    let one = en
        .quantity(Number::new(1.0), Some(celsius), UnitStyle::Verbose)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(one, "1 degree Celsius");
}

#[test]
fn unit_lookup_prefers_the_ucum_code_then_the_symbol() {
    let unit = |text, ucum| unit_message(UnitRef { text, ucum });
    assert_eq!(unit("°C", None), Some("unit-celsius"));
    assert_eq!(unit(" hPa ", None), Some("unit-hectopascal"));
    assert_eq!(unit("degrees", Some("[degF]")), Some("unit-fahrenheit"));
    assert_eq!(unit("furlongs", None), None);
}

#[test]
fn localizer_is_shareable_between_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Localizer>();
}

proptest! {
    /// Formatting never fails for a finite value, and the printed digits respect the
    /// fraction-digit bounds.
    #[test]
    fn nfr_09_finite_numbers_always_format(
        value in -1.0e12_f64..1.0e12,
        min in 0_u8..4,
        extra in 0_u8..4,
    ) {
        let options = NumberOptions {
            min_fraction: Some(min),
            max_fraction: Some(min + extra),
            ..NumberOptions::default()
        };
        let text = number(&localizer("en-US"), value, options);
        let fraction = text.split('.').nth(1).map_or(0, str::len);
        prop_assert!(fraction >= usize::from(min));
        prop_assert!(fraction <= usize::from(min + extra));
    }
}
