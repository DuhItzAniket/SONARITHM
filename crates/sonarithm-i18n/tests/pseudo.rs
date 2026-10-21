// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The pseudo-locale check (B02-T08, NFR-09): every catalogue message renders in `en-XA`
//! as exactly one marked, accented and padded message, and the check catches a seeded
//! concatenated string, a seeded hard-coded string and a seeded truncation.

#![allow(
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use fluent_syntax::ast::Entry;
use sonarithm_i18n::{
    Args, Localizer, Number, NumberOptions, PSEUDO_LOCALE, PseudoIssue, SOURCE_LOCALE, UnitStyle,
    catalogue_files, check_pseudo,
};

fn pseudo() -> Localizer {
    Localizer::new(PSEUDO_LOCALE).unwrap_or_else(|e| panic!("{e}"))
}

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
fn nfr_09_every_message_passes_the_pseudo_locale_check() {
    let localizer = pseudo();
    assert!(localizer.is_pseudo());
    let mut checked = 0;
    for (file, source) in catalogue_files(SOURCE_LOCALE).unwrap_or_default() {
        let resource = fluent_syntax::parser::parse(*source)
            .unwrap_or_else(|(_, errors)| panic!("{file}: {errors:?}"));
        for entry in resource.body {
            let Entry::Message(message) = entry else {
                continue;
            };
            for style in [UnitStyle::Symbol, UnitStyle::Terse, UnitStyle::Verbose] {
                let id = message.id.name;
                let text = localizer
                    .format(id, &sample_args(style))
                    .unwrap_or_else(|e| panic!("{id}: {e}"));
                check_pseudo(&text).unwrap_or_else(|issue| panic!("{id}: {issue}"));
                // The only ASCII letters left come from the `unit` text argument.
                assert!(
                    !text
                        .replace("furlongs", "")
                        .chars()
                        .any(|c| c.is_ascii_alphabetic()),
                    "{id}: untransformed catalogue text in {text:?}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}

#[test]
fn nfr_09_pseudo_locale_keeps_arguments_as_they_are() {
    let text = pseudo()
        .format(
            "unit-other",
            &Args::new().number("value", 3.0).text("unit", "furlongs"),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(text.contains("3 furlongs"), "{text}");
}

#[test]
fn nfr_09_pseudo_check_catches_a_seeded_concatenated_string() {
    let localizer = pseudo();
    let count = localizer
        .format("common-point-count", &Args::new().number("count", 3.0))
        .unwrap_or_else(|e| panic!("{e}"));
    let position = localizer
        .format(
            "common-position",
            &Args::new().number("index", 2.0).number("total", 5.0),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    // Seeded defect: a sentence built by joining two messages with a separator.
    let joined = format!("{count}, {position}");
    assert!(matches!(
        check_pseudo(&joined),
        Err(PseudoIssue::Concatenated(_))
    ));
    // Seeded defect: a hard-coded prefix in front of a message.
    let prefixed = format!("Series: {count}");
    assert!(matches!(
        check_pseudo(&prefixed),
        Err(PseudoIssue::Concatenated(_))
    ));
}

#[test]
fn nfr_09_pseudo_check_catches_hard_coded_and_truncated_strings() {
    assert!(matches!(
        check_pseudo("Stop all audio"),
        Err(PseudoIssue::Unmarked(_))
    ));
    let text = pseudo()
        .format("ui-stop-all-audio", &Args::new())
        .unwrap_or_else(|e| panic!("{e}"));
    // Seeded defect: a label cut to a fixed width.
    let cut: String = text.chars().take(8).collect();
    assert!(matches!(check_pseudo(&cut), Err(PseudoIssue::Truncated(_))));
}

#[test]
fn nfr_09_pseudo_messages_are_at_least_forty_percent_longer() {
    let english = Localizer::new(SOURCE_LOCALE).unwrap_or_else(|e| panic!("{e}"));
    let plain = english
        .format("ui-stop-all-audio", &Args::new())
        .unwrap_or_else(|e| panic!("{e}"));
    let marked = pseudo()
        .format("ui-stop-all-audio", &Args::new())
        .unwrap_or_else(|e| panic!("{e}"));
    let inner = marked.chars().count() - 2;
    assert!(inner * 5 >= plain.chars().count() * 7, "{marked}");
}
