// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Message arguments.

use fluent_bundle::types::{FluentNumber, FluentNumberOptions};
use fluent_bundle::{FluentArgs, FluentValue};

use crate::error::I18nError;
use crate::number;

/// Digit options for a number, with the meaning and defaults of `Intl.NumberFormat`.
///
/// The defaults print between zero and three fraction digits with locale grouping, as the
/// UI does, so both sides agree on every digit (DL-P12: one value, one formatter).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NumberOptions {
    /// At least this many fraction digits (`minimumFractionDigits`); default 0.
    pub min_fraction: Option<u8>,
    /// At most this many fraction digits (`maximumFractionDigits`); default
    /// `max(min_fraction, 3)`.
    pub max_fraction: Option<u8>,
    /// At least this many integer digits (`minimumIntegerDigits`); default 1.
    pub min_integer: Option<u8>,
    /// Locale grouping separators (`useGrouping`); default on. Turn it off for years and
    /// identifiers ("1980", not "1,980").
    pub grouping: bool,
}

impl Default for NumberOptions {
    fn default() -> Self {
        Self {
            min_fraction: None,
            max_fraction: None,
            min_integer: None,
            grouping: true,
        }
    }
}

impl NumberOptions {
    /// Exactly `digits` fraction digits.
    #[must_use]
    pub fn fixed(digits: u8) -> Self {
        Self {
            min_fraction: Some(digits),
            max_fraction: Some(digits),
            ..Self::default()
        }
    }

    /// The same options without grouping separators.
    #[must_use]
    pub fn without_grouping(mut self) -> Self {
        self.grouping = false;
        self
    }

    pub(crate) fn to_fluent(self) -> FluentNumberOptions {
        FluentNumberOptions {
            use_grouping: self.grouping,
            minimum_integer_digits: self.min_integer.map(usize::from),
            minimum_fraction_digits: self.min_fraction.map(usize::from),
            maximum_fraction_digits: self.max_fraction.map(usize::from),
            ..FluentNumberOptions::default()
        }
    }

    pub(crate) fn from_fluent(options: &FluentNumberOptions) -> Self {
        let digits = |d: Option<usize>| d.map(|d| u8::try_from(d).unwrap_or(u8::MAX));
        Self {
            min_fraction: digits(options.minimum_fraction_digits),
            max_fraction: digits(options.maximum_fraction_digits),
            min_integer: digits(options.minimum_integer_digits),
            grouping: options.use_grouping,
        }
    }
}

/// A number argument: a value and how to print it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Number {
    /// The value.
    pub value: f64,
    /// The digit options.
    pub options: NumberOptions,
}

impl Number {
    /// A number with the default options.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self {
            value,
            options: NumberOptions::default(),
        }
    }

    /// A number with the given options.
    #[must_use]
    pub fn with_options(value: f64, options: NumberOptions) -> Self {
        Self { value, options }
    }

    /// The Fluent value, rounded to the printed digits first. Fluent picks plural forms
    /// from the value and the minimum fraction digits only, so rounding here makes
    /// `1.0004` with two fraction digits select "one" as it does in the UI.
    pub(crate) fn to_fluent(self, name: &str) -> Result<FluentValue<'static>, I18nError> {
        let rounded =
            number::rounded(self.value, self.options).ok_or_else(|| I18nError::NonFinite {
                name: name.to_owned(),
            })?;
        Ok(FluentValue::Number(FluentNumber::new(
            rounded,
            self.options.to_fluent(),
        )))
    }
}

/// One argument value.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    /// Text that is inserted as it is (a series name, a category, a formatted date).
    Text(String),
    /// A number, formatted for the locale.
    Number(Number),
}

/// The named arguments of one message, in insertion order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Args {
    entries: Vec<(String, ArgValue)>,
}

impl Args {
    /// No arguments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a text argument.
    #[must_use]
    pub fn text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries
            .push((name.into(), ArgValue::Text(value.into())));
        self
    }

    /// Adds a number argument with the default options.
    #[must_use]
    pub fn number(self, name: impl Into<String>, value: f64) -> Self {
        self.number_with(name, Number::new(value))
    }

    /// Adds a number argument.
    #[must_use]
    pub fn number_with(mut self, name: impl Into<String>, value: Number) -> Self {
        self.entries.push((name.into(), ArgValue::Number(value)));
        self
    }

    /// The arguments in insertion order.
    #[must_use]
    pub fn entries(&self) -> &[(String, ArgValue)] {
        &self.entries
    }

    pub(crate) fn to_fluent(&self) -> Result<FluentArgs<'static>, I18nError> {
        let mut args = FluentArgs::with_capacity(self.entries.len());
        for (name, value) in &self.entries {
            let value = match value {
                ArgValue::Text(text) => FluentValue::from(text.clone()),
                ArgValue::Number(number) => number.to_fluent(name)?,
            };
            args.set(name.clone(), value);
        }
        Ok(args)
    }
}
