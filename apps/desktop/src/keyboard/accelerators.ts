// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/** The modifier and key state of a keyboard event. */
export interface KeyChord {
  readonly key: string;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly altKey: boolean;
  readonly shiftKey: boolean;
}

/** Keys that are browser commands on their own (keyboard §12.3). */
const BARE_KEYS = new Set([
  "F5", // reload
  "F7", // caret browsing
  "F12", // developer tools
  "BrowserBack",
  "BrowserForward",
  "BrowserRefresh",
  "BrowserSearch",
]);

/** Letters that are browser commands with the platform's primary modifier. */
const PRIMARY_LETTERS = new Set([
  "r", // reload (and Shift for a hard reload)
  "f", // find
  "g", // find next
  "p", // print
  "u", // view source
  "s", // save page
  "j", // downloads or console
]);

/** Primary+Shift letters that open developer tools. */
const PRIMARY_SHIFT_LETTERS = new Set(["i", "j", "c"]);

/** Primary with these is page zoom; the app re-implements zoom (keyboard §12.3). */
const ZOOM_KEYS = new Set(["+", "=", "-", "_", "0"]);

/**
 * Whether a key press would run a webview browser command: reload, find, print, caret
 * browsing, developer tools, history or zoom (keyboard §12.3, KB-04). WebView2's own
 * setting turns these off on Windows (`src-tauri/src/platform`); this guard covers every
 * webview, and the app's commands still see the event.
 */
export function isBrowserAccelerator(chord: KeyChord): boolean {
  if (BARE_KEYS.has(chord.key)) {
    return true;
  }
  if (
    chord.altKey &&
    (chord.key === "ArrowLeft" || chord.key === "ArrowRight")
  ) {
    return true; // history back and forward
  }
  const primary = chord.ctrlKey || chord.metaKey;
  if (!primary || chord.altKey) {
    return false;
  }
  const key = chord.key.toLowerCase();
  if (ZOOM_KEYS.has(key)) {
    return true;
  }
  return chord.shiftKey
    ? PRIMARY_SHIFT_LETTERS.has(key) || key === "r"
    : PRIMARY_LETTERS.has(key);
}

/** Something that dispatches key events, such as `window`. */
export interface KeyTarget {
  addEventListener(
    type: "keydown",
    listener: (event: KeyboardEvent) => void,
    options: { capture: boolean },
  ): void;
  removeEventListener(
    type: "keydown",
    listener: (event: KeyboardEvent) => void,
    options: { capture: boolean },
  ): void;
}

/**
 * Stops the webview's default action for browser accelerators on `target`; returns a
 * function that removes the guard. The event still propagates, so app commands bound to
 * the same keys (find, zoom) keep working.
 */
export function installAcceleratorGuard(target: KeyTarget): () => void {
  const listener = (event: KeyboardEvent): void => {
    if (isBrowserAccelerator(event)) {
      event.preventDefault();
    }
  };
  target.addEventListener("keydown", listener, { capture: true });
  return () => {
    target.removeEventListener("keydown", listener, { capture: true });
  };
}
