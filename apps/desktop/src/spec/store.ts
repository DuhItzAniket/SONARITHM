// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Client,
  CompileReport,
  Counts,
  DiagnosticInfo,
  HistoryState,
  IpcError,
  PatchOp,
  SpecPatchResult,
} from "@sonarithm/ipc";
import { IpcFailure } from "@sonarithm/ipc";

/** Diagnostics and the text they were found in, so they can be placed in that text. */
export interface Checked {
  readonly text: string;
  readonly diagnostics: readonly DiagnosticInfo[];
  readonly counts: Counts;
}

/** What the UI shows of the session's spec. Rust owns the spec; this is its projection. */
export interface SpecSnapshot {
  /** The session's spec text, `null` before one is set. */
  readonly text: string | null;
  /** Counts every change of the session's spec, whatever made it (editor, inspector, undo). */
  readonly revision: number;
  /** The latest diagnostics: live validation, an apply or a compile. */
  readonly checked: Checked | null;
  /** Whether a newer text is being validated. */
  readonly checking: boolean;
  /**
   * The text whose check could not run (the transport failed), so views say so instead of
   * "Checking"; the next change checks again.
   */
  readonly failed: string | null;
}

/**
 * Whether the editor's `draft` needs checking now (B05-T03): it has text, nothing is being
 * checked, the latest check is of another text (typing during an Apply, an inspector patch,
 * a compile), and its own check has not just failed.
 */
export function needsCheck(snapshot: SpecSnapshot, draft: string): boolean {
  return (
    draft.trim() !== "" &&
    !snapshot.checking &&
    snapshot.checked?.text !== draft &&
    snapshot.failed !== draft
  );
}

/** What Apply did (B05-T03). */
export type ApplyOutcome =
  | { readonly kind: "applied"; readonly report: CompileReport }
  /** The spec was set (and is undoable), but the compile could not run. */
  | { readonly kind: "appliedNotCompiled"; readonly error: IpcError }
  | { readonly kind: "notApplied"; readonly error: IpcError };

/** What an undo or redo did, and which history it stepped. */
export type HistoryOutcome =
  | { readonly kind: "stepped"; readonly label: string }
  | { readonly kind: "nothing" }
  | { readonly kind: "failed"; readonly error: IpcError };

const EMPTY: SpecSnapshot = {
  text: null,
  revision: 0,
  checked: null,
  checking: false,
  failed: null,
};

function ipcError(error: unknown): IpcError {
  if (error instanceof IpcFailure) {
    const { code, message, hint, pointer } = error;
    return { code, message, hint, pointer };
  }
  return {
    code: "SNR-APP-009",
    message: error instanceof Error ? error.message : String(error),
    hint: null,
    pointer: null,
  };
}

/**
 * The one spec store of the UI (B05-T03): every view that shows or edits the spec (the
 * editor, the inspector of B05-T02, undo and redo) reads it and changes the spec only
 * through it, because the app core sends no event when the spec changes. Each reply that
 * carries the spec (`spec`, `spec_patch`, `history`) updates it, so all views follow the
 * same text. Rust stays authoritative (09 §3).
 */
export class SpecStore {
  private state: SpecSnapshot = EMPTY;
  private readonly listeners = new Set<() => void>();
  private validation = 0;
  private readonly client: Client;

  constructor(client: Client) {
    this.client = client;
  }

  /** For `useSyncExternalStore`. */
  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  /** For `useSyncExternalStore`: stable between changes. */
  snapshot = (): SpecSnapshot => this.state;

  /** Reads the session's spec (`spec_get`), as after opening a project. */
  async refresh(): Promise<void> {
    const reply = await this.client.call({ command: "spec_get" });
    this.setText(reply.text);
  }

  /**
   * Apply: sets the spec to `text` (`spec_set`, one undoable edit), then compiles it. A text
   * that is not a spec is refused and nothing changes; a spec that does not compile is kept,
   * and the compile's diagnostics are the latest. `onSet` runs as soon as the spec is set,
   * before the compile, so the editor's own undo history starts again from the applied text
   * and keeps what is typed while the compile runs.
   */
  async apply(text: string, onSet?: () => void): Promise<ApplyOutcome> {
    try {
      const reply = await this.client.call({
        command: "spec_set",
        args: { text },
      });
      this.setText(reply.text);
      onSet?.();
    } catch (error) {
      return { kind: "notApplied", error: ipcError(error) };
    }
    try {
      const report = await this.client.call({ command: "compile" });
      this.validation += 1;
      this.update({
        checked: {
          text,
          diagnostics: report.diagnostics,
          counts: report.counts,
        },
        checking: false,
      });
      return { kind: "applied", report };
    } catch (error) {
      return { kind: "appliedNotCompiled", error: ipcError(error) };
    }
  }

  /**
   * Clears the latest check when there is nothing to check (an empty editor), so no view
   * shows diagnostics of a text that is gone.
   */
  clearChecked(text: string): void {
    this.validation += 1;
    this.update({
      checked: {
        text,
        diagnostics: [],
        counts: { errors: 0, warnings: 0, info: 0 },
      },
      checking: false,
    });
  }

  /** A JSON Patch (`spec_apply`), as the inspector sends: one undoable edit. */
  async patch(
    patch: readonly PatchOp[],
    label: string | null,
  ): Promise<SpecPatchResult> {
    const reply = await this.client.call({
      command: "spec_apply",
      args: { patch: [...patch], label },
    });
    this.validation += 1;
    this.setText(reply.text, {
      text: reply.text,
      diagnostics: reply.diagnostics,
      counts: reply.counts,
    });
    return reply;
  }

  /** Steps the spec history back (`history_undo`). */
  undo(): Promise<HistoryOutcome> {
    return this.step("history_undo");
  }

  /** Steps the spec history forward (`history_redo`). */
  redo(): Promise<HistoryOutcome> {
    return this.step("history_redo");
  }

  /**
   * Live validation of a text being edited (`spec_validate`; the session is unchanged). A
   * reply for an older text never replaces a newer one.
   */
  async validate(text: string): Promise<void> {
    const ticket = ++this.validation;
    this.update({ checking: true, failed: null });
    try {
      const reply = await this.client.call({
        command: "spec_validate",
        args: { text },
      });
      if (ticket === this.validation) {
        this.update({
          checked: {
            text,
            diagnostics: reply.diagnostics,
            counts: reply.counts,
          },
          checking: false,
        });
      }
    } catch {
      if (ticket === this.validation) {
        this.update({ checking: false, failed: text });
      }
    }
  }

  private async step(
    command: "history_undo" | "history_redo",
  ): Promise<HistoryOutcome> {
    let reply: HistoryState;
    try {
      reply = await this.client.call({ command });
    } catch (error) {
      return { kind: "failed", error: ipcError(error) };
    }
    if (reply.token === null) {
      return { kind: "nothing" };
    }
    this.setText(reply.spec.text);
    return { kind: "stepped", label: reply.label ?? "" };
  }

  private setText(text: string | null, checked?: Checked): void {
    const changed = text !== this.state.text;
    this.update({
      text,
      revision: changed ? this.state.revision + 1 : this.state.revision,
      ...(checked === undefined ? {} : { checked, checking: false }),
    });
  }

  private update(next: Partial<SpecSnapshot>): void {
    this.state = { ...this.state, ...next };
    for (const listener of this.listeners) listener();
  }
}
