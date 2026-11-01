// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Chord, Platform } from "./chords.ts";
import type { Scope } from "./scopes.ts";

/** A command of the v0 keymap: a stable ID, a name, a scope and default bindings. */
export interface CommandSpec {
  /** Stable; what users remap (keyboard §13 rule 1). */
  readonly id: string;
  /** The Fluent message that names it, with a verb (design-language §12). */
  readonly name: string;
  readonly scope: Scope;
  /** Default bindings on Windows and Linux. */
  readonly keys: readonly Chord[];
  /** Default bindings on macOS, where they differ (Cmd for Ctrl, KB-07). */
  readonly macKeys?: readonly Chord[];
}

/**
 * The global and region commands of the v0 keymap (keyboard-and-navigation §5.1, §5.2).
 * Commands appear in the palette once their feature registers a handler; the bindings
 * here are checked by the keymap lint either way. The table, tree, inspector and editor
 * keymaps join with their widgets (B04, B05).
 *
 * The chart explorer's keymap v0 (B03-T09; keyboard-and-navigation §4.2, §5.4, §5.5) is
 * scoped to the explorer: its single keys work only there (KB-02, WCAG 2.1.4), Jump is Ctrl
 * on Windows and Linux and Option on macOS, and no binding takes a screen reader's keys
 * (§12: never Insert, CapsLock, the numpad or Ctrl+Option). Compare (C), selection
 * (Shift+arrows), polarity (P), the query menu (Q) and the braille mode (Shift+B) join with
 * their features; A, J, K, M, O and U stay free and E reserved.
 */
export const DEFAULT_KEYMAP: readonly CommandSpec[] = [
  {
    id: "app.palette",
    name: "command-app-palette",
    scope: "global",
    keys: ["Ctrl+Shift+P", "Ctrl+K"],
    macKeys: ["Shift+Meta+P", "Meta+K"],
  },
  {
    id: "file.new",
    name: "command-file-new",
    scope: "global",
    keys: ["Ctrl+N"],
    macKeys: ["Meta+N"],
  },
  {
    id: "file.open",
    name: "command-file-open",
    scope: "global",
    keys: ["Ctrl+O"],
    macKeys: ["Meta+O"],
  },
  {
    id: "file.save",
    name: "command-file-save",
    scope: "global",
    keys: ["Ctrl+S"],
    macKeys: ["Meta+S"],
  },
  {
    id: "file.saveAs",
    name: "command-file-save-as",
    scope: "global",
    keys: ["Ctrl+Shift+S"],
    macKeys: ["Shift+Meta+S"],
  },
  {
    id: "file.export",
    name: "command-file-export",
    scope: "global",
    keys: ["Ctrl+E"],
    macKeys: ["Meta+E"],
  },
  {
    id: "edit.undo",
    name: "command-edit-undo",
    scope: "global",
    keys: ["Ctrl+Z"],
    macKeys: ["Meta+Z"],
  },
  {
    id: "edit.redo",
    name: "command-edit-redo",
    scope: "global",
    keys: ["Ctrl+Y", "Ctrl+Shift+Z"],
    macKeys: ["Shift+Meta+Z"],
  },
  {
    id: "edit.find",
    name: "command-edit-find",
    scope: "global",
    keys: ["Ctrl+F"],
    macKeys: ["Meta+F"],
  },
  {
    id: "app.preferences",
    name: "command-app-preferences",
    scope: "global",
    keys: ["Ctrl+Comma"],
    macKeys: ["Meta+Comma"],
  },
  {
    id: "help.context",
    name: "command-help-context",
    scope: "global",
    keys: ["F1"],
  },
  {
    id: "help.keyboard",
    name: "command-help-keyboard",
    scope: "global",
    keys: ["Shift+F1"],
  },
  {
    id: "view.zoomIn",
    name: "command-view-zoom-in",
    scope: "global",
    keys: ["Ctrl+Plus"],
    macKeys: ["Meta+Plus"],
  },
  {
    id: "view.zoomOut",
    name: "command-view-zoom-out",
    scope: "global",
    keys: ["Ctrl+Minus"],
    macKeys: ["Meta+Minus"],
  },
  {
    id: "view.zoomReset",
    name: "command-view-zoom-reset",
    scope: "global",
    keys: ["Ctrl+0"],
    macKeys: ["Meta+0"],
  },
  {
    id: "view.focusExplorer",
    name: "command-view-focus-explorer",
    scope: "global",
    keys: ["Ctrl+Shift+C"],
    macKeys: ["Shift+Meta+C"],
  },
  {
    id: "view.announcements",
    name: "command-view-announcements",
    scope: "global",
    keys: ["Ctrl+Shift+H"],
    macKeys: ["Shift+Meta+H"],
  },
  {
    id: "transport.playPause",
    name: "command-transport-play-pause",
    scope: "global",
    keys: ["Ctrl+Shift+Space"],
    macKeys: ["Shift+Meta+Space"],
  },
  {
    // Keyboard §5.3: Escape in the explorer, after closing an overlay; the toolbar's Stop
    // button and stop-all audio (Ctrl+Period) always work too.
    id: "transport.stop",
    name: "command-transport-stop",
    scope: "explorer",
    keys: ["Escape"],
  },
  {
    // Keyboard §5.3: unbound by default; reached from the toolbar and the palette.
    id: "transport.loop",
    name: "command-transport-loop",
    scope: "global",
    keys: [],
  },
  {
    id: "transport.stopAll",
    name: "command-transport-stop-all",
    scope: "global",
    keys: ["Ctrl+Period"],
    macKeys: ["Meta+Period"],
  },
  {
    id: "view.nextRegion",
    name: "command-view-next-region",
    scope: "global",
    keys: ["F6"],
  },
  {
    id: "view.prevRegion",
    name: "command-view-prev-region",
    scope: "global",
    keys: ["Shift+F6"],
  },
  {
    id: "view.menuBar",
    name: "command-view-menu-bar",
    scope: "global",
    keys: ["F10"],
    macKeys: [],
  },
  {
    id: "view.contextMenu",
    name: "command-view-context-menu",
    scope: "global",
    keys: ["Shift+F10"],
  },
  {
    id: "view.toggleRegion",
    name: "command-view-toggle-region",
    scope: "region",
    keys: ["Ctrl+Shift+Backslash"],
    macKeys: ["Shift+Meta+Backslash"],
  },
  {
    id: "nav.next",
    name: "command-nav-next",
    scope: "explorer",
    keys: ["ArrowRight"],
  },
  {
    id: "nav.previous",
    name: "command-nav-previous",
    scope: "explorer",
    keys: ["ArrowLeft"],
  },
  {
    id: "nav.jumpForward",
    name: "command-nav-jump-forward",
    scope: "explorer",
    keys: ["Ctrl+ArrowRight"],
    macKeys: ["Alt+ArrowRight"],
  },
  {
    id: "nav.jumpBack",
    name: "command-nav-jump-back",
    scope: "explorer",
    keys: ["Ctrl+ArrowLeft"],
    macKeys: ["Alt+ArrowLeft"],
  },
  {
    id: "nav.first",
    name: "command-nav-first",
    scope: "explorer",
    keys: ["Home"],
  },
  {
    id: "nav.last",
    name: "command-nav-last",
    scope: "explorer",
    keys: ["End"],
  },
  {
    id: "nav.into",
    name: "command-nav-into",
    scope: "explorer",
    keys: ["ArrowDown"],
  },
  {
    id: "nav.activate",
    name: "command-nav-activate",
    scope: "explorer",
    keys: ["Enter"],
  },
  {
    id: "nav.out",
    name: "command-nav-out",
    scope: "explorer",
    keys: ["ArrowUp", "Backspace"],
  },
  {
    id: "nav.nextSeries",
    name: "command-nav-next-series",
    scope: "explorer",
    keys: ["PageDown"],
  },
  {
    id: "nav.previousSeries",
    name: "command-nav-previous-series",
    scope: "explorer",
    keys: ["PageUp"],
  },
  {
    id: "nav.maximum",
    name: "command-nav-maximum",
    scope: "explorer",
    keys: ["Ctrl+ArrowUp"],
    macKeys: ["Alt+ArrowUp"],
  },
  {
    id: "nav.minimum",
    name: "command-nav-minimum",
    scope: "explorer",
    keys: ["Ctrl+ArrowDown"],
    macKeys: ["Alt+ArrowDown"],
  },
  {
    id: "nav.nextInsight",
    name: "command-nav-next-insight",
    scope: "explorer",
    keys: ["N"],
  },
  {
    id: "nav.previousInsight",
    name: "command-nav-previous-insight",
    scope: "explorer",
    keys: ["Shift+N"],
  },
  {
    id: "nav.xAxis",
    name: "command-nav-x-axis",
    scope: "explorer",
    keys: ["X"],
  },
  {
    id: "nav.yAxis",
    name: "command-nav-y-axis",
    scope: "explorer",
    keys: ["Y"],
  },
  {
    id: "nav.insights",
    name: "command-nav-insights",
    scope: "explorer",
    keys: ["I"],
  },
  {
    id: "nav.filters",
    name: "command-nav-filters",
    scope: "explorer",
    keys: ["F"],
  },
  {
    id: "nav.dataset",
    name: "command-nav-dataset",
    scope: "explorer",
    keys: ["D"],
  },
  {
    id: "nav.goTo",
    name: "command-nav-go-to",
    scope: "explorer",
    keys: ["G"],
  },
  {
    id: "nav.findValue",
    name: "command-nav-find-value",
    scope: "explorer",
    keys: ["Ctrl+F"],
    macKeys: ["Meta+F"],
  },
  {
    id: "nav.whereAmI",
    name: "command-nav-where-am-i",
    scope: "explorer",
    keys: ["W"],
  },
  {
    id: "nav.repeat",
    name: "command-nav-repeat",
    scope: "explorer",
    keys: ["R"],
  },
  {
    id: "nav.help",
    name: "command-nav-help",
    scope: "explorer",
    keys: ["H"],
  },
  {
    id: "modality.speech",
    name: "command-modality-speech",
    scope: "explorer",
    keys: ["T"],
  },
  {
    id: "verbosity.toggle",
    name: "command-verbosity-toggle",
    scope: "explorer",
    keys: ["Shift+T"],
  },
  {
    id: "modality.sonification",
    name: "command-modality-sonification",
    scope: "explorer",
    keys: ["S"],
  },
  {
    id: "modality.braille",
    name: "command-modality-braille",
    scope: "explorer",
    keys: ["B"],
  },
  {
    id: "modality.visual",
    name: "command-modality-visual",
    scope: "explorer",
    keys: ["V"],
  },
  {
    // Keyboard §5.6 (B04-T05): from the palette while a data table is open.
    id: "table.goToRow",
    name: "command-table-go-to-row",
    scope: "table",
    keys: [],
  },
];

/** A command's default bindings on `platform`. */
export function bindingsFor(
  spec: CommandSpec,
  platform: Platform,
): readonly Chord[] {
  return platform === "mac" ? (spec.macKeys ?? spec.keys) : spec.keys;
}

/** The two region commands may use F6 and Shift+F6, which are otherwise reserved. */
export const REGION_COMMANDS: ReadonlySet<string> = new Set([
  "view.nextRegion",
  "view.prevRegion",
]);
