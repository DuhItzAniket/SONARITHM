// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Keymap overrides (keyboard-and-navigation §13; B03-T09).
//!
//! The settings' `keymap` maps a command ID to its keys: chords such as `Ctrl+Shift+C`,
//! separated by spaces when there are several, or the empty string to unbind it. The UI's
//! recorder refuses reserved combinations first; this check refuses them again in
//! `settings_set` (`SNR-APP-091`), so a hand-edited settings file cannot bind Tab either.
//! It mirrors the desktop's keymap lint (`apps/desktop/src/commands/lint.ts`), §13 rule 6:
//! Tab and Shift+Tab, F6 and Shift+F6 (except for the region commands), Escape alone
//! (except as Stop), Insert and CapsLock combinations, Ctrl+Option on macOS, Win and Super
//! combinations, Alt+F4, Alt+Space and Cmd+Q.

use std::collections::BTreeMap;

/// The region commands, which may use F6 and Shift+F6 (§13 rule 6).
const REGION_COMMANDS: [&str; 2] = ["view.nextRegion", "view.prevRegion"];

/// Whether this build runs on macOS, where Meta is Cmd and Ctrl+Option are VoiceOver's keys.
const MAC: bool = cfg!(target_os = "macos");

/// The first chord of `keys` that `command` may not use, if any.
#[must_use]
pub fn reserved_chord<'k>(command: &str, keys: &'k str) -> Option<&'k str> {
    keys.split_whitespace()
        .find(|chord| is_reserved(command, chord))
}

/// The first keymap entry with a reserved chord: its command and that chord.
#[must_use]
pub fn first_reserved(keymap: &BTreeMap<String, String>) -> Option<(&str, &str)> {
    keymap.iter().find_map(|(command, keys)| {
        reserved_chord(command, keys).map(|chord| (command.as_str(), chord))
    })
}

fn is_reserved(command: &str, chord: &str) -> bool {
    let mut parts: Vec<&str> = chord.split('+').collect();
    let Some(key) = parts.pop() else {
        return false;
    };
    let has = |m: &str| parts.contains(&m);
    let (ctrl, alt, shift, meta) = (has("Ctrl"), has("Alt"), has("Shift"), has("Meta"));
    let only_shift = !ctrl && !alt && !meta;
    let none = only_shift && !shift;
    (key == "Tab" && only_shift)
        || (key == "F6" && only_shift && !REGION_COMMANDS.contains(&command))
        || (key == "Escape" && none && command != "transport.stop")
        || key == "Insert"
        || key == "CapsLock"
        || parts.iter().any(|m| *m == "Insert" || *m == "CapsLock")
        || (MAC && ctrl && alt)
        || (!MAC && meta)
        || (MAC && meta && key == "Q")
        || (key == "F4" && alt)
        || (key == "Space" && alt && !ctrl && !meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Keyboard §13 rule 6: the reserved combinations are refused for any command, and
    /// only the region commands may take F6.
    #[test]
    fn keyboard_13_rule_6_reserved_combinations_are_refused() {
        for chord in ["Tab", "Shift+Tab", "F6", "Shift+F6", "Ctrl+Insert", "CapsLock"] {
            assert_eq!(reserved_chord("nav.next", chord), Some(chord), "{chord}");
        }
        assert_eq!(reserved_chord("nav.next", "Alt+F4"), Some("Alt+F4"));
        assert_eq!(reserved_chord("nav.next", "Alt+Space"), Some("Alt+Space"));
        assert_eq!(reserved_chord("nav.next", "Escape"), Some("Escape"));
        assert_eq!(reserved_chord("view.nextRegion", "F6"), None);
        assert_eq!(reserved_chord("transport.stop", "Escape"), None);
        assert_eq!(reserved_chord("nav.next", "ArrowRight L"), None);
        assert_eq!(reserved_chord("nav.next", ""), None, "an empty value unbinds");
        assert_eq!(
            reserved_chord("nav.next", "ArrowRight Shift+Tab"),
            Some("Shift+Tab"),
            "every chord of a value is checked"
        );
        if MAC {
            assert_eq!(reserved_chord("nav.next", "Meta+Q"), Some("Meta+Q"));
            assert_eq!(reserved_chord("nav.next", "Ctrl+Alt+N"), Some("Ctrl+Alt+N"));
        } else {
            assert_eq!(reserved_chord("nav.next", "Meta+N"), Some("Meta+N"));
        }
    }
}
