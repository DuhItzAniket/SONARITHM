// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The embedded catalogues.
//!
//! `tests/catalogue.rs` checks that every `.ftl` file in a locale directory is listed here,
//! so a new file cannot be forgotten.

/// The locale every message is written in first; other catalogues translate it.
pub const SOURCE_LOCALE: &str = "en-US";

/// The locales with a catalogue. The second locale arrives in B07 (OD-12).
pub const CATALOGUE_LOCALES: &[&str] = &[SOURCE_LOCALE];

/// The English catalogue: (file name, contents).
const EN_US: &[(&str, &str)] = &[
    (
        "announce.ftl",
        include_str!("../locales/en-US/announce.ftl"),
    ),
    ("chart.ftl", include_str!("../locales/en-US/chart.ftl")),
    ("common.ftl", include_str!("../locales/en-US/common.ftl")),
    ("earcons.ftl", include_str!("../locales/en-US/earcons.ftl")),
    (
        "function.ftl",
        include_str!("../locales/en-US/function.ftl"),
    ),
    ("stream.ftl", include_str!("../locales/en-US/stream.ftl")),
    ("ui.ftl", include_str!("../locales/en-US/ui.ftl")),
    ("units.ftl", include_str!("../locales/en-US/units.ftl")),
];

/// The catalogue files of `locale` as (file name, contents), or `None` if `locale` has no
/// catalogue. `locale` must be one of [`CATALOGUE_LOCALES`] exactly.
#[must_use]
pub fn catalogue_files(locale: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match locale {
        SOURCE_LOCALE => Some(EN_US),
        _ => None,
    }
}
