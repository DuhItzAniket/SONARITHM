// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The catalogues are complete and consistent: every file is embedded, every catalogue
//! loads, every locale has the source locale's messages, every message renders with the
//! sample arguments, and the unit table and the unit messages match (NFR-09).

#![allow(
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use fluent_syntax::ast::Entry;
use sonarithm_i18n::{
    Args, CATALOGUE_LOCALES, I18nError, Localizer, Number, NumberOptions, SOURCE_LOCALE,
    UNIT_MESSAGES, UnitStyle, catalogue_files,
};

fn locale_dir(locale: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("locales")
        .join(locale)
}

/// The message IDs (not terms) of a locale's catalogue, sorted.
fn message_ids(locale: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for (file, source) in catalogue_files(locale).unwrap_or_default() {
        let resource = fluent_syntax::parser::parse(*source)
            .unwrap_or_else(|(_, errors)| panic!("{locale}/{file}: {errors:?}"));
        for entry in resource.body {
            if let Entry::Message(message) = entry {
                ids.insert(message.id.name.to_owned());
            }
        }
    }
    ids
}

/// Arguments that cover every variable the catalogues use. A new variable needs a sample
/// here, so every message is rendered by these tests.
fn sample_args(style: UnitStyle) -> Args {
    Args::new()
        .number_with("value", Number::with_options(28.7, NumberOptions::fixed(1)))
        .text("style", style.selector())
        .number("count", 3.0)
        .number("index", 2.0)
        .number("total", 5.0)
        .text("unit", "furlongs")
        // Data arguments have no letters, so the pseudo-locale test sees only catalogue
        // text; selectors fall back to their default variant.
        .text("text", "12.5")
        .text("first", "1")
        .text("rest", "2")
        .text("from", "3")
        .text("to", "4")
        .text("qualifier", "5")
        .number("confidence", 0.82)
        .text("key", "↓")
        .text("kind", "point")
        .text("stream", "6")
        .text("detail", "7")
        .text("class", "cv_estimate")
        .text("axis", "x")
        .text("included", "yes")
        .text("edge", "last")
        .text("reason", "one-series")
        .number("hours", 14.0)
        .number("minutes", 2.0)
        .number("seconds", 11.0)
        .text("date", "6")
        .text("time", "7")
        .text("field", "8")
        .text("series", "9")
        .text("range", "10")
        .number("low", 400.0)
        .number("high", 1400.0)
        .text("polarity", "positive")
        .text("mode", "analysis")
        .text("ticks", "yes")
        .text("reference", "yes")
}

#[test]
fn every_catalogue_file_is_embedded() {
    for locale in CATALOGUE_LOCALES {
        let on_disk: BTreeSet<String> = std::fs::read_dir(locale_dir(locale))
            .unwrap_or_else(|e| panic!("{locale}: {e}"))
            .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect::<Result<_, _>>()
            .unwrap_or_else(|e| panic!("{locale}: {e}"));
        let embedded: BTreeSet<String> = catalogue_files(locale)
            .unwrap_or_default()
            .iter()
            .map(|(file, _)| (*file).to_owned())
            .collect();
        assert_eq!(on_disk, embedded, "{locale}: embed every .ftl file");
    }
}

#[test]
fn nfr_09_every_catalogue_loads() {
    for locale in CATALOGUE_LOCALES {
        let localizer = Localizer::new(locale).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(localizer.catalogue(), *locale);
    }
}

#[test]
fn nfr_09_every_locale_has_the_source_messages() {
    let source = message_ids(SOURCE_LOCALE);
    assert!(!source.is_empty());
    for locale in CATALOGUE_LOCALES {
        assert_eq!(
            message_ids(locale),
            source,
            "{locale} differs from {SOURCE_LOCALE}"
        );
    }
}

#[test]
fn nfr_09_every_message_renders_with_the_sample_arguments() {
    for locale in CATALOGUE_LOCALES {
        let localizer = Localizer::new(locale).unwrap_or_else(|e| panic!("{e}"));
        for id in message_ids(locale) {
            for style in [UnitStyle::Symbol, UnitStyle::Terse, UnitStyle::Verbose] {
                let text = localizer
                    .format(&id, &sample_args(style))
                    .unwrap_or_else(|e| panic!("{locale} {id}: {e}"));
                assert!(!text.trim().is_empty(), "{locale} {id} is empty");
            }
        }
    }
}

#[test]
fn unit_table_and_unit_messages_match() {
    let table: BTreeSet<&str> = UNIT_MESSAGES.iter().map(|(_, _, id)| *id).collect();
    assert_eq!(
        table.len(),
        UNIT_MESSAGES.len(),
        "one table row per message"
    );
    let ids = UNIT_MESSAGES
        .iter()
        .map(|(_, _, id)| *id)
        .collect::<Vec<_>>();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "UNIT_MESSAGES is sorted by message ID");

    let catalogue: BTreeSet<String> = message_ids(SOURCE_LOCALE)
        .into_iter()
        .filter(|id| id.starts_with("unit-") && id != "unit-other")
        .collect();
    let table: BTreeSet<String> = table.into_iter().map(str::to_owned).collect();
    assert_eq!(
        table, catalogue,
        "every unit message has a table row, and back"
    );
}

#[test]
fn a11y_12_unit_styles_differ_where_the_symbol_is_not_a_word() {
    // Speech never reads a symbol (keyboard-and-navigation §8.1): the verbose form is
    // always a name, so it differs from the symbol form for every known unit.
    let localizer = Localizer::new(SOURCE_LOCALE).unwrap_or_else(|e| panic!("{e}"));
    for (_, _, id) in UNIT_MESSAGES {
        let render = |style: UnitStyle| {
            localizer
                .format(id, &sample_args(style))
                .unwrap_or_else(|e| panic!("{id}: {e}"))
        };
        assert_ne!(
            render(UnitStyle::Symbol),
            render(UnitStyle::Verbose),
            "{id}"
        );
    }
}

#[test]
fn errors_are_returned_not_rendered() {
    let localizer = Localizer::new(SOURCE_LOCALE).unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(
        localizer.format("no-such-message", &Args::new()),
        Err(I18nError::MissingMessage { .. })
    ));
    assert!(matches!(
        localizer.format("common-position", &Args::new().number("index", 1.0)),
        Err(I18nError::Format { .. })
    ));
    assert!(matches!(
        localizer.format("common-number", &Args::new().number("value", f64::NAN)),
        Err(I18nError::NonFinite { .. })
    ));
    assert!(matches!(
        Localizer::new("not a locale"),
        Err(I18nError::InvalidLocale(_))
    ));
}
