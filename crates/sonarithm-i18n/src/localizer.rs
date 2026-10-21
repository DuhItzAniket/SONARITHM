// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The localizer: one locale's catalogue plus its formatters.

use std::fmt::Write as _;

use fluent_bundle::FluentResource;
use fluent_bundle::concurrent::FluentBundle;
use icu_locale_core::Locale;
use unic_langid::LanguageIdentifier;

use crate::args::{Args, Number};
use crate::catalogue::{CATALOGUE_LOCALES, SOURCE_LOCALE, catalogue_files};
use crate::date::{self, DateLength, DatePattern};
use crate::error::I18nError;
use crate::number;
use crate::pseudo::{self, PSEUDO_LOCALE};
use crate::units::{UNIT_OTHER, UnitRef, UnitStyle, unit_message};

/// Formats catalogue messages, numbers, dates and quantities for one locale.
///
/// The localizer is `Send` and `Sync`, so the app core can share one per session.
pub struct Localizer {
    bundle: FluentBundle<FluentResource>,
    /// The locale used for numbers, dates and plural rules.
    locale: Locale,
    /// The catalogue the messages come from.
    catalogue: &'static str,
    pseudo: bool,
}

impl std::fmt::Debug for Localizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Localizer")
            .field("locale", &self.locale.to_string())
            .field("catalogue", &self.catalogue)
            .field("pseudo", &self.pseudo)
            .finish_non_exhaustive()
    }
}

impl Localizer {
    /// A localizer for `requested`, a BCP 47 language identifier such as `en-GB`.
    ///
    /// Messages come from the catalogue of the same language (`en-GB` reads `en-US`), or
    /// from the English catalogue if the language has none. Numbers, dates and plural rules
    /// follow `requested` when its language has a catalogue, and English otherwise, so text
    /// and formats never mix languages. [`PSEUDO_LOCALE`] gives the pseudo-locale.
    ///
    /// # Errors
    ///
    /// [`I18nError::InvalidLocale`] for an identifier that does not parse, and
    /// [`I18nError::Catalogue`] if an embedded catalogue is broken (the tests rule this out).
    pub fn new(requested: &str) -> Result<Self, I18nError> {
        let invalid = || I18nError::InvalidLocale(requested.to_owned());
        let langid: LanguageIdentifier = requested.parse().map_err(|_| invalid())?;
        let pseudo_locale: LanguageIdentifier = PSEUDO_LOCALE.parse().map_err(|_| invalid())?;
        let pseudo = langid == pseudo_locale;
        let catalogue = if pseudo {
            SOURCE_LOCALE
        } else {
            negotiate(&langid)
        };
        let format_locale = if pseudo || same_language(&langid, catalogue) {
            langid
        } else {
            catalogue.parse().map_err(|_| invalid())?
        };
        let locale = Locale::try_from_str(&format_locale.to_string()).map_err(|_| invalid())?;

        let mut bundle = FluentBundle::new_concurrent(vec![format_locale]);
        // Bidi isolation marks around arguments are off: full right-to-left support is
        // post-1.0 (12 §5), and the marks reach speech synthesizers as characters.
        bundle.set_use_isolating(false);
        bundle.set_formatter(Some(number::fluent_formatter));
        if pseudo {
            bundle.set_transform(Some(pseudo::accent));
        }
        for (file, source) in catalogue_files(catalogue).unwrap_or_default() {
            let broken = |details: String| I18nError::Catalogue {
                locale: catalogue.to_owned(),
                file: (*file).to_owned(),
                details,
            };
            let resource = FluentResource::try_new((*source).to_owned())
                .map_err(|(_, errors)| broken(format!("{errors:?}")))?;
            bundle
                .add_resource(resource)
                .map_err(|errors| broken(format!("{errors:?}")))?;
        }
        Ok(Self {
            bundle,
            locale,
            catalogue,
            pseudo,
        })
    }

    /// The locale used for numbers, dates and plural rules.
    #[must_use]
    pub fn locale(&self) -> String {
        self.locale.to_string()
    }

    /// The catalogue the messages come from.
    #[must_use]
    pub fn catalogue(&self) -> &'static str {
        self.catalogue
    }

    /// Whether this is the pseudo-locale.
    #[must_use]
    pub fn is_pseudo(&self) -> bool {
        self.pseudo
    }

    /// Whether the catalogue has a message with this ID.
    #[must_use]
    pub fn has_message(&self, id: &str) -> bool {
        self.bundle.has_message(id)
    }

    /// Formats message `id` with `args`.
    ///
    /// # Errors
    ///
    /// [`I18nError::MissingMessage`], [`I18nError::NoValue`], [`I18nError::NonFinite`] for
    /// a NaN or infinite number, and [`I18nError::Format`] when Fluent reports an error,
    /// such as a missing argument. Errors are returned, never rendered into the text.
    pub fn format(&self, id: &str, args: &Args) -> Result<String, I18nError> {
        let message = self
            .bundle
            .get_message(id)
            .ok_or_else(|| I18nError::MissingMessage {
                id: id.to_owned(),
                locale: self.catalogue.to_owned(),
            })?;
        let pattern = message
            .value()
            .ok_or_else(|| I18nError::NoValue { id: id.to_owned() })?;
        let fluent_args = args.to_fluent()?;
        let mut errors = Vec::new();
        let text = self
            .bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors);
        if !errors.is_empty() {
            let mut details = String::new();
            for error in &errors {
                let _ = write!(details, "{error}; ");
            }
            return Err(I18nError::Format {
                id: id.to_owned(),
                details: details.trim_end_matches("; ").to_owned(),
            });
        }
        Ok(if self.pseudo {
            pseudo::wrap(&text)
        } else {
            text.into_owned()
        })
    }

    /// Formats a number for the locale, outside any message (a table cell, a readout).
    ///
    /// # Errors
    ///
    /// [`I18nError::NonFinite`] for NaN and infinite values.
    pub fn number(&self, value: Number) -> Result<String, I18nError> {
        let non_finite = || I18nError::NonFinite {
            name: "value".to_owned(),
        };
        let decimal = number::decimal(value.value, value.options).ok_or_else(non_finite)?;
        let formatter = number::new_formatter(&self.locale.to_string(), value.options.grouping)
            .map_err(|()| non_finite())?;
        Ok(formatter.format_to_string(&decimal))
    }

    /// Formats a Gregorian date (`year`, `month` 1–12, `day`) for the locale.
    ///
    /// # Errors
    ///
    /// [`I18nError::InvalidDate`] for a date that does not exist.
    pub fn date(
        &self,
        ymd: (i32, u8, u8),
        pattern: DatePattern,
        length: DateLength,
    ) -> Result<String, I18nError> {
        date::format(&self.locale, ymd, pattern, length)
    }

    /// Formats a value with its unit, in `style`: "28.7 °C", "28.7 degrees C" or
    /// "28.7 degrees Celsius". Without a unit, the number alone. A unit the table does not
    /// know is printed as written, after the value.
    ///
    /// # Errors
    ///
    /// As [`Localizer::format`].
    pub fn quantity(
        &self,
        value: Number,
        unit: Option<UnitRef<'_>>,
        style: UnitStyle,
    ) -> Result<String, I18nError> {
        let Some(unit) = unit else {
            return self.format("common-number", &Args::new().number_with("value", value));
        };
        let args = Args::new()
            .number_with("value", value)
            .text("style", style.selector());
        match unit_message(unit) {
            Some(id) => self.format(id, &args),
            None => self.format(UNIT_OTHER, &args.text("unit", unit.text)),
        }
    }
}

/// The catalogue for `langid`: an exact match, else the first catalogue of the same
/// language, else the source catalogue.
fn negotiate(langid: &LanguageIdentifier) -> &'static str {
    let wanted = langid.to_string();
    CATALOGUE_LOCALES
        .iter()
        .find(|locale| **locale == wanted)
        .or_else(|| {
            CATALOGUE_LOCALES
                .iter()
                .find(|locale| same_language(langid, locale))
        })
        .copied()
        .unwrap_or(SOURCE_LOCALE)
}

fn same_language(langid: &LanguageIdentifier, catalogue: &str) -> bool {
    catalogue
        .parse::<LanguageIdentifier>()
        .is_ok_and(|other| other.language == langid.language)
}
