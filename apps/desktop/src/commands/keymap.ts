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
 * here are checked by the keymap lint either way. The explorer, table, tree, inspector
 * and editor keymaps join with their widgets (B03, B04, B05).
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
    // B04-T07 (A11Y-26): the alternative to dragging the plot sideways; palette and toolbar.
    id: "view.panLeft",
    name: "command-view-pan-left",
    scope: "global",
    keys: [],
  },
  {
    id: "view.panRight",
    name: "command-view-pan-right",
    scope: "global",
    keys: [],
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
