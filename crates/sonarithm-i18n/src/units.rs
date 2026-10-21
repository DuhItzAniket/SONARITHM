// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The unit table: from a field's unit (03 §3.2) to its catalogue message.
//!
//! Every unit message takes a number `$value` and a `$style` and prints the value with the
//! unit, so translators control the order, spacing and plural forms. Units the table does
//! not know are printed through `unit-other` with the unit text as written.

/// Which form of a unit to use (keyboard-and-navigation §8.1, DL-P12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitStyle {
    /// The symbol, for visual text, braille and the log: "28.7 °C".
    Symbol,
    /// The short spoken form, for terse announcements: "28.7 degrees C".
    Terse,
    /// The full spoken name, for verbose announcements: "28.7 degrees Celsius".
    Verbose,
}

impl UnitStyle {
    /// The `$style` selector value in the catalogue.
    #[must_use]
    pub fn selector(self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::Terse => "terse",
            Self::Verbose => "verbose",
        }
    }
}

/// A unit as a field declares it: free text plus an optional UCUM code (03 §3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnitRef<'a> {
    /// The unit as written, such as `°C`.
    pub text: &'a str,
    /// The UCUM code, such as `Cel`.
    pub ucum: Option<&'a str>,
}

/// The message for units the table does not know: `{ $value } { $unit }`.
pub(crate) const UNIT_OTHER: &str = "unit-other";

/// Known units: (UCUM code, symbols and spellings that also select it, message ID).
/// Sorted by message ID. A unit joins the table with its message in every catalogue; the
/// tests check both directions.
pub const UNIT_MESSAGES: &[(&str, &[&str], &str)] = &[
    ("A", &["A"], "unit-ampere"),
    ("Cel", &["°C", "℃", "degC"], "unit-celsius"),
    ("cm", &["cm"], "unit-centimeter"),
    ("d", &["d", "day", "days"], "unit-day"),
    ("dB", &["dB"], "unit-decibel"),
    ("[degF]", &["°F", "℉", "degF"], "unit-fahrenheit"),
    ("g", &["g"], "unit-gram"),
    ("hPa", &["hPa"], "unit-hectopascal"),
    ("Hz", &["Hz"], "unit-hertz"),
    ("h", &["h", "hr", "hour", "hours"], "unit-hour"),
    ("K", &["K"], "unit-kelvin"),
    ("kg", &["kg"], "unit-kilogram"),
    ("kHz", &["kHz"], "unit-kilohertz"),
    ("km", &["km"], "unit-kilometer"),
    ("km/h", &["km/h", "kph"], "unit-kilometer-per-hour"),
    ("kW", &["kW"], "unit-kilowatt"),
    ("m", &["m"], "unit-meter"),
    ("m/s", &["m/s"], "unit-meter-per-second"),
    ("mm", &["mm"], "unit-millimeter"),
    ("min", &["min"], "unit-minute"),
    ("[ppm]", &["ppm"], "unit-part-per-million"),
    ("Pa", &["Pa"], "unit-pascal"),
    ("%", &["%"], "unit-percent"),
    ("s", &["s", "sec"], "unit-second"),
    ("V", &["V"], "unit-volt"),
    ("W", &["W"], "unit-watt"),
];

/// The catalogue message for `unit`: by UCUM code first, then by its text. `None` means the
/// unit is not in the table and is printed as written.
#[must_use]
pub fn unit_message(unit: UnitRef<'_>) -> Option<&'static str> {
    if let Some(code) = unit.ucum
        && let Some((_, _, id)) = UNIT_MESSAGES.iter().find(|(ucum, _, _)| *ucum == code)
    {
        return Some(id);
    }
    let text = unit.text.trim();
    UNIT_MESSAGES
        .iter()
        .find(|(_, spellings, _)| spellings.contains(&text))
        .map(|(_, _, id)| *id)
}
