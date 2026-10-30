// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * Where a binding applies (keyboard-and-navigation §2). `widget` scopes are named by the
 * focused widget: explorer, table, tree, inspector, editor.
 */
export type Scope =
  | "global"
  | "region"
  | "explorer"
  | "table"
  | "tree"
  | "inspector"
  | "editor"
  | "overlay";

/**
 * The scopes active for the focused element, highest precedence first: overlay, then the
 * focused widget, then the region, then global (§2). A text field keeps its own editing
 * keys: `typing` tells the dispatcher to let character keys through.
 */
export interface ActiveScopes {
  readonly scopes: readonly Scope[];
  readonly typing: boolean;
}

const WIDGET_SCOPES = new Set<Scope>([
  "explorer",
  "table",
  "tree",
  "inspector",
  "editor",
]);

/** The active scopes for `element` (the focused element, or `null` for the page). */
export function activeScopes(element: Element | null): ActiveScopes {
  const scopes: Scope[] = [];
  if (
    element?.closest('[role="dialog"], [role="alertdialog"], [role="menu"]')
  ) {
    scopes.push("overlay");
  }
  const widget = element?.closest<HTMLElement>("[data-scope]")?.dataset
    .scope as Scope | undefined;
  if (widget !== undefined && WIDGET_SCOPES.has(widget)) {
    scopes.push(widget);
  }
  if (element?.closest("[data-region]")) {
    scopes.push("region");
  }
  scopes.push("global");
  const typing =
    element instanceof HTMLInputElement ||
    element instanceof HTMLTextAreaElement ||
    (element instanceof HTMLElement && element.isContentEditable);
  return { scopes, typing };
}
