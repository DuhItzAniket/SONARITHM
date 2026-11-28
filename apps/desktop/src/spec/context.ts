// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { CodeEditorHandle } from "@sonarithm/ui/code-editor";
import { createContext, use, useSyncExternalStore } from "react";

import type { DiagnosticStep, PlacedDiagnostic } from "./speech.ts";
import type { SpecSnapshot, SpecStore } from "./store.ts";

/**
 * The editor as the app's commands see it. The editor loads on demand (PB-07), so a
 * command run before it is shown waits here until it mounts.
 */
export class EditorBridge {
  private handle: CodeEditorHandle | null = null;
  private pending: ((handle: CodeEditorHandle) => void)[] = [];
  private placed: readonly PlacedDiagnostic[] = [];
  private text: string | null = null;
  private readonly listeners = new Set<() => void>();
  /** Where F8 last went, so diagnostics at one offset are each reached (B05-T03). */
  lastStep: DiagnosticStep | null = null;

  /** For `useSyncExternalStore`: the editor's draft. */
  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  /**
   * The text in the editor, applied or not; `null` while the editor has not loaded. The
   * Diagnostics tab shows only diagnostics found in this text.
   */
  draft = (): string | null => this.text;

  /** Called by the editor on every edit. */
  setDraft(text: string | null): void {
    if (text === this.text) return;
    this.text = text;
    for (const listener of this.listeners) listener();
  }

  /** The diagnostics placed in the editor's current text, in text order. */
  get diagnostics(): readonly PlacedDiagnostic[] {
    return this.placed;
  }

  /** Called by the editor whenever its diagnostics change. */
  setDiagnostics(diagnostics: readonly PlacedDiagnostic[]): void {
    this.placed = [...diagnostics].sort(
      (a, b) => (a.span?.from ?? 0) - (b.span?.from ?? 0),
    );
    this.lastStep = null;
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
      this.lastStep = null;
      this.setDraft(null);
    }
  }

  private failed = false;

  /** Called when the editor's code could not load: nothing queued will ever run. */
  markUnavailable(): void {
    this.failed = true;
    this.pending = [];
  }

  /**
   * Runs `action` with the editor now, or once it mounts. If the editor could not load,
   * `unavailable` runs instead, so a command says why it did nothing.
   */
  withEditor(
    action: (handle: CodeEditorHandle) => void,
    unavailable?: () => void,
  ): void {
    if (this.handle !== null) action(this.handle);
    else if (this.failed) unavailable?.();
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
  failed: null,
};
const noop = () => () => undefined;

/** The session's spec as the store last saw it, re-rendering on every change. */
export function useSpecSnapshot(store: SpecStore | null): SpecSnapshot {
  return useSyncExternalStore(
    store?.subscribe ?? noop,
    store?.snapshot ?? (() => NO_SPEC),
  );
}
