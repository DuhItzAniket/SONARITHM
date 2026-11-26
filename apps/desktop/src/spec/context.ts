// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { CodeEditorHandle } from "@sonarithm/ui/code-editor";
import { createContext, use, useSyncExternalStore } from "react";

import type { PlacedDiagnostic } from "./speech.ts";
import type { SpecSnapshot, SpecStore } from "./store.ts";

/**
 * The editor as the app's commands see it. The editor loads on demand (PB-07), so a
 * command run before it is shown waits here until it mounts.
 */
export class EditorBridge {
  private handle: CodeEditorHandle | null = null;
  private pending: ((handle: CodeEditorHandle) => void)[] = [];
  private placed: readonly PlacedDiagnostic[] = [];

  /** The diagnostics placed in the editor's current text, in text order. */
  get diagnostics(): readonly PlacedDiagnostic[] {
    return this.placed;
  }

  /** Called by the editor whenever its diagnostics change. */
  setDiagnostics(diagnostics: readonly PlacedDiagnostic[]): void {
    this.placed = [...diagnostics].sort(
      (a, b) => (a.span?.from ?? 0) - (b.span?.from ?? 0),
    );
  }

  /** Called by the editor when it mounts. */
  attach(handle: CodeEditorHandle): void {
    this.handle = handle;
    const waiting = this.pending;
    this.pending = [];
    for (const action of waiting) action(handle);
  }

  /** Called by the editor when it unmounts. */
  detach(handle: CodeEditorHandle): void {
    if (this.handle === handle) {
      this.handle = null;
      this.placed = [];
    }
  }

  /** Runs `action` with the editor now, or once it mounts. */
  withEditor(action: (handle: CodeEditorHandle) => void): void {
    if (this.handle !== null) action(this.handle);
    else this.pending.push(action);
  }
}

/** The spec store and the editor bridge, for the views below the app root. */
export interface SpecContextValue {
  readonly store: SpecStore;
  readonly bridge: EditorBridge;
}

export const SpecContext = createContext<SpecContextValue | null>(null);

/** The spec context, or `null` before the app core answers. */
export function useSpecContext(): SpecContextValue | null {
  return use(SpecContext);
}

const NO_SPEC: SpecSnapshot = {
  text: null,
  revision: 0,
  checked: null,
  checking: false,
};
const noop = () => () => undefined;

/** The session's spec as the store last saw it, re-rendering on every change. */
export function useSpecSnapshot(store: SpecStore | null): SpecSnapshot {
  return useSyncExternalStore(
    store?.subscribe ?? noop,
    store?.snapshot ?? (() => NO_SPEC),
  );
}
