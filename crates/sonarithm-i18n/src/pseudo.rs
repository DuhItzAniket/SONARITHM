// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The pseudo-locale and its check (12 §5, NFR-09).
//!
//! `en-XA` renders the English catalogue with accented letters, so untranslated text stands
//! out; pads each message by about 40 %, so layouts that clip longer translations show it;
//! and wraps each message in [`PSEUDO_OPEN`] … [`PSEUDO_CLOSE`]. Arguments (numbers, data
//! names) are left as they are. `packages/i18n` implements the same transform, and the
//! contract test checks that both produce the same text.

use std::borrow::Cow;

/// The pseudo-locale: English text, accented, padded and marked.
pub const PSEUDO_LOCALE: &str = "en-XA";
/// Opens every message in the pseudo-locale.
pub const PSEUDO_OPEN: char = '⟦';
/// Closes every message in the pseudo-locale.
pub const PSEUDO_CLOSE: char = '⟧';
/// The padding character, added before the closing marker.
const PAD: char = '~';

const PLAIN: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ACCENTED: &str = "àƀçðéƒĝĥîĵķļɱñöƥǫŕšţûṽŵẋýžÀƁÇÐÉƑĜĤÎĴĶĻṀÑÖƤǪŔŠŢÛṼŴẊÝŽ";

/// Replaces ASCII letters with accented look-alikes. Used as the Fluent transform, which
/// applies to the catalogue's text and never to arguments.
pub(crate) fn accent(text: &str) -> Cow<'_, str> {
    if !text.bytes().any(|b| b.is_ascii_alphabetic()) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(
        text.chars()
            .map(|c| {
                PLAIN
                    .chars()
                    .position(|p| p == c)
                    .and_then(|i| ACCENTED.chars().nth(i))
                    .unwrap_or(c)
            })
            .collect(),
    )
}

/// Wraps one formatted message: markers, and about 40 % padding before the closing one.
pub(crate) fn wrap(text: &str) -> String {
    let length = text.chars().count();
    let padding = (2 * length).div_ceil(5);
    let mut out = String::with_capacity(text.len() + padding + 6);
    out.push(PSEUDO_OPEN);
    out.push_str(text);
    out.extend(std::iter::repeat_n(PAD, padding));
    out.push(PSEUDO_CLOSE);
    out
}

/// What [`check_pseudo`] found wrong with a pseudo-locale string.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PseudoIssue {
    /// No marker at all: the text did not come from the catalogue (a hard-coded string).
    #[error("text outside the catalogue (hard-coded string): {0:?}")]
    Unmarked(String),
    /// Text before or after the message, or two messages side by side: a string built by
    /// concatenation.
    #[error("concatenated string: {0:?}")]
    Concatenated(String),
    /// The closing marker is missing: the message was cut short.
    #[error("truncated string: {0:?}")]
    Truncated(String),
}

/// Checks that `text` is exactly one pseudo-locale message: it starts with the opening
/// marker, its markers balance, and the outermost message ends at the last character.
/// Messages nested inside it (a unit message passed as an argument) are allowed.
///
/// # Errors
///
/// The [`PseudoIssue`] that describes the first problem found.
pub fn check_pseudo(text: &str) -> Result<(), PseudoIssue> {
    if !text.starts_with(PSEUDO_OPEN) {
        return Err(if text.contains(PSEUDO_OPEN) {
            PseudoIssue::Concatenated(text.to_owned())
        } else {
            PseudoIssue::Unmarked(text.to_owned())
        });
    }
    let mut depth = 0_usize;
    for (index, c) in text.char_indices() {
        if c == PSEUDO_OPEN {
            depth += 1;
        } else if c == PSEUDO_CLOSE {
            depth = depth
                .checked_sub(1)
                .ok_or_else(|| PseudoIssue::Concatenated(text.to_owned()))?;
            if depth == 0 {
                return if index + c.len_utf8() == text.len() {
                    Ok(())
                } else {
                    Err(PseudoIssue::Concatenated(text.to_owned()))
                };
            }
        }
    }
    Err(PseudoIssue::Truncated(text.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_maps_every_ascii_letter_to_one_character() {
        assert_eq!(PLAIN.chars().count(), ACCENTED.chars().count());
        let accented = accent(PLAIN);
        assert_eq!(accented, ACCENTED);
        assert!(accented.chars().all(|c| !c.is_ascii()));
    }

    #[test]
    fn accent_leaves_digits_and_symbols_alone() {
        assert_eq!(accent("28.7 °, 1/2 – ⟦⟧"), "28.7 °, 1/2 – ⟦⟧");
    }

    #[test]
    fn wrap_pads_by_forty_percent_rounded_up() {
        assert_eq!(wrap("abcde"), "⟦abcde~~⟧");
        assert_eq!(wrap("a"), "⟦a~⟧");
        assert_eq!(wrap(""), "⟦⟧");
    }

    #[test]
    fn check_accepts_one_message_with_nested_messages() {
        assert_eq!(check_pseudo("⟦a ⟦b~⟧~~⟧"), Ok(()));
        assert_eq!(check_pseudo("⟦⟧"), Ok(()));
    }

    #[test]
    fn check_rejects_hard_coded_concatenated_and_truncated_text() {
        assert!(matches!(
            check_pseudo("Play"),
            Err(PseudoIssue::Unmarked(_))
        ));
        assert!(matches!(
            check_pseudo("Value: ⟦a~⟧"),
            Err(PseudoIssue::Concatenated(_))
        ));
        assert!(matches!(
            check_pseudo("⟦a~⟧ ⟦b~⟧"),
            Err(PseudoIssue::Concatenated(_))
        ));
        assert!(matches!(
            check_pseudo("⟦a~⟧."),
            Err(PseudoIssue::Concatenated(_))
        ));
        assert!(matches!(
            check_pseudo("⟦abc"),
            Err(PseudoIssue::Truncated(_))
        ));
        assert!(matches!(
            check_pseudo("⟦a ⟦b~⟧"),
            Err(PseudoIssue::Truncated(_))
        ));
    }
}
