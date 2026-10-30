// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/** The platform families with different modifiers (KB-07). */
export type Platform = "windows" | "linux" | "mac";

/**
 * A key combination as text: modifiers in the order Ctrl, Alt, Shift, Meta, then one key,
 * joined by "+": "Ctrl+Shift+P", "F6", "Shift+F6", "Meta+Period". On macOS, Meta is Cmd
 * and Alt is Option.
 */
export type Chord = string;

const MODIFIERS = ["Ctrl", "Alt", "Shift", "Meta"] as const;

/** Punctuation and digits bind by physical key (KB-08): names for `KeyboardEvent.code`. */
const CODE_NAMES: Readonly<Record<string, string>> = {
  Comma: "Comma",
  Period: "Period",
  Slash: "Slash",
  Backslash: "Backslash",
  Semicolon: "Semicolon",
  Quote: "Quote",
  BracketLeft: "BracketLeft",
  BracketRight: "BracketRight",
  Minus: "Minus",
  Equal: "Plus",
  Backquote: "Backquote",
  NumpadAdd: "Plus",
  NumpadSubtract: "Minus",
};

/** The parts of a keyboard event a chord is made from. */
export interface KeyInput {
  readonly key: string;
  readonly code: string;
  readonly ctrlKey: boolean;
  readonly altKey: boolean;
  readonly shiftKey: boolean;
  readonly metaKey: boolean;
}

/**
 * The chord of a key press, or `null` for a bare modifier. Letters bind by character, so
 * an X is X on any layout; punctuation and digits by physical key (KB-08).
 */
export function chordOf(input: KeyInput): Chord | null {
  const { key, code } = input;
  if (["Control", "Alt", "Shift", "Meta", "AltGraph"].includes(key)) {
    return null;
  }
  let name: string;
  if (/^[a-z]$/i.test(key)) {
    name = key.toUpperCase();
  } else if (code.startsWith("Digit")) {
    name = code.slice(5);
  } else if (code in CODE_NAMES) {
    name = CODE_NAMES[code] ?? code;
  } else if (key === " ") {
    name = "Space";
  } else {
    name = key;
  }
  const mods = [
    input.ctrlKey && "Ctrl",
    input.altKey && "Alt",
    input.shiftKey && "Shift",
    input.metaKey && "Meta",
  ].filter((m): m is string => typeof m === "string");
  return [...mods, name].join("+");
}

/** A chord split into its modifiers and key. */
export function parseChord(chord: Chord): {
  readonly mods: ReadonlySet<string>;
  readonly key: string;
} {
  const parts = chord.split("+");
  const key = parts.pop() ?? "";
  return { mods: new Set(parts), key };
}

/** A chord in its canonical modifier order, so equal combinations compare equal. */
export function normalize(chord: Chord): Chord {
  const { mods, key } = parseChord(chord);
  return [...MODIFIERS.filter((m) => mods.has(m)), key].join("+");
}

/** How a chord is shown to the user on `platform` (KB-09): Cmd and Option on macOS. */
export function displayChord(chord: Chord, platform: Platform): string {
  const { mods, key } = parseChord(chord);
  const names: Record<string, string> =
    platform === "mac"
      ? { Ctrl: "Ctrl", Alt: "Option", Shift: "Shift", Meta: "Cmd" }
      : { Ctrl: "Ctrl", Alt: "Alt", Shift: "Shift", Meta: "Win" };
  return [
    ...MODIFIERS.filter((m) => mods.has(m)).map((m) => names[m]),
    key,
  ].join("+");
}

/** The platform the UI runs on, from the user agent. */
export function currentPlatform(userAgent: string): Platform {
  if (userAgent.includes("Mac")) {
    return "mac";
  }
  return userAgent.includes("Windows") ? "windows" : "linux";
}
