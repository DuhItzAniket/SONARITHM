// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Chord, Platform } from "./chords.ts";
import { parseChord } from "./chords.ts";
import type { Scope } from "./scopes.ts";

/** Why a binding is refused, with the keymap rule it breaks. */
export interface BindingProblem {
  readonly rule: "KB-02" | "KB-03" | "KB-04" | "reserved";
  readonly reason: string;
}

const ARROWS = new Set(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"]);

/** A key that types a character: letters, digits and punctuation. */
function isCharacter(key: string): boolean {
  return (
    /^[A-Z0-9]$/.test(key) ||
    [
      "Comma",
      "Period",
      "Slash",
      "Backslash",
      "Semicolon",
      "Quote",
      "BracketLeft",
      "BracketRight",
      "Minus",
      "Plus",
      "Backquote",
    ].includes(key)
  );
}

/**
 * The keymap rules a binding breaks on `platform` (keyboard-and-navigation §1, §12, §13
 * rule 6). Empty when the binding is allowed. F6 and Shift+F6 are reserved for the region
 * commands, which `regionCommand` marks.
 */
export function bindingProblems(
  chord: Chord,
  scope: Scope,
  platform: Platform,
  regionCommand = false,
): BindingProblem[] {
  const { mods, key } = parseChord(chord);
  const ctrl = mods.has("Ctrl");
  const alt = mods.has("Alt");
  const meta = mods.has("Meta");
  const problems: BindingProblem[] = [];

  if (key === "Insert" || key === "CapsLock") {
    problems.push({ rule: "KB-03", reason: "screen-reader modifier key" });
  }
  if (platform === "mac" && ctrl && alt) {
    problems.push({ rule: "KB-03", reason: "VoiceOver keys (Ctrl+Option)" });
  }
  if (ctrl && alt && /^[A-Z]$/.test(key)) {
    problems.push({
      rule: "KB-04",
      reason: "Ctrl+Alt+letter types AltGr characters",
    });
  }
  // Alt+arrows are history in WebView2 and WebKitGTK; on macOS, Option+arrows are Jump
  // (KB-07) and Ctrl+Option is caught above as VoiceOver keys.
  if (alt && ARROWS.has(key) && (platform !== "mac" || ctrl)) {
    problems.push({
      rule: "KB-04",
      reason: ctrl
        ? "Ctrl+Alt+arrows are screen-reader table navigation"
        : "Alt+arrows are webview history",
    });
  }
  if (ctrl && key === "Space" && !mods.has("Shift") && !alt && !meta) {
    problems.push({
      rule: "KB-04",
      reason: "Ctrl+Space switches input methods",
    });
  }
  const onlyShift = [...mods].every((m) => m === "Shift");
  if (onlyShift && isCharacter(key) && scope !== "explorer") {
    problems.push({
      rule: "KB-02",
      reason: "single-character shortcuts belong to the chart explorer only",
    });
  }
  const reserved =
    (key === "Tab" && [...mods].every((m) => m === "Shift")) ||
    (key === "Escape" && mods.size === 0 && scope !== "explorer") ||
    (key === "F4" && alt) ||
    (key === "Space" && alt && !ctrl && !meta) ||
    (platform !== "mac" && meta) ||
    (platform === "mac" && meta && key === "Q") ||
    (key === "F6" && [...mods].every((m) => m === "Shift") && !regionCommand);
  if (reserved) {
    problems.push({
      rule: "reserved",
      reason: "reserved, never bindable (§13 rule 6)",
    });
  }
  return problems;
}
