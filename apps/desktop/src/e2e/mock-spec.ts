// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  CompileReport,
  DiagnosticInfo,
  MockHandlers,
  SpecPatchResult,
} from "@sonarithm/ipc";
import { IpcFailure } from "@sonarithm/ipc";

/**
 * The spec commands of the mock host (B05-T03): the session's spec, its undo history, and a
 * small validator with the app core's codes and messages for the cases the E2E tests use
 * (JSON syntax, an unknown field). The app core's own tests cover the real validator.
 *
 * A test sets `window.__SONARITHM_MOCK_SPEC__ = "<text>"` before the page loads to start
 * with a spec.
 */
export interface MockSpec {
  readonly handlers: MockHandlers;
  /** The session's spec text. */
  text(): string | null;
}

/** The fields the mock dataset has (mock-host.ts), besides those a spec declares. */
const DATA_FIELDS = ["time", "temperature"];

function diagnostic(
  code: string,
  message: string,
  pointer: string | null,
  severity = "error",
): DiagnosticInfo {
  return { severity, code, rules: [], message, pointer };
}

function counts(diagnostics: readonly DiagnosticInfo[]) {
  return {
    errors: diagnostics.filter((d) => d.severity === "error").length,
    warnings: diagnostics.filter((d) => d.severity === "warning").length,
    info: diagnostics.filter((d) => d.severity === "info").length,
  };
}

function syntaxError(text: string): DiagnosticInfo | null {
  try {
    JSON.parse(text);
    return null;
  } catch (error) {
    const detail = error instanceof Error ? error.message : "";
    return diagnostic("SNR-SPEC-001", `Invalid JSON: ${detail}.`, null);
  }
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** The mock validator: syntax first, then fields named by channels. */
export function validate(text: string): DiagnosticInfo[] {
  const syntax = syntaxError(text);
  if (syntax !== null) {
    return [syntax];
  }
  const spec: unknown = JSON.parse(text);
  if (!isObject(spec)) {
    return [diagnostic("SNR-SPEC-002", "The spec must be a JSON object.", "")];
  }
  const out: DiagnosticInfo[] = [];
  const data = isObject(spec.data) ? spec.data : {};
  const declared = isObject(data.fields) ? Object.keys(data.fields) : [];
  const known = new Set([...DATA_FIELDS, ...declared]);
  const tracks = Array.isArray(spec.tracks) ? spec.tracks : [];
  if (!Array.isArray(spec.tracks)) {
    out.push(
      diagnostic("SNR-SPEC-003", "A required key is missing: `tracks`.", ""),
    );
  }
  tracks.forEach((track: unknown, index) => {
    const channels =
      isObject(track) && isObject(track.channels) ? track.channels : {};
    for (const [channel, value] of Object.entries(channels)) {
      if (isObject(value) && typeof value.field === "string") {
        if (!known.has(value.field)) {
          out.push(
            diagnostic(
              "SNR-SPEC-070",
              `The data has no field "${value.field}". Name a field of the data, or declare it in \`data.fields\`.`,
              `/tracks/${String(index)}/channels/${channel}/field`,
            ),
          );
        }
      }
    }
  });
  if (spec.mode === undefined) {
    out.push(
      diagnostic(
        "SNR-SPEC-090",
        "The mode defaults to analysis (SDR-36).",
        "",
        "info",
      ),
    );
  }
  return out;
}

interface Edit {
  readonly label: string;
  readonly before: string | null;
  readonly after: string | null;
}

/** The mock spec session. */
export function mockSpec(): MockSpec {
  const start: unknown = Reflect.get(window, "__SONARITHM_MOCK_SPEC__");
  let text: string | null = typeof start === "string" ? start : null;
  const done: Edit[] = [];
  const undone: Edit[] = [];
  let token = 0;

  const record = (label: string, next: string): number => {
    done.push({ label, before: text, after: next });
    undone.length = 0;
    text = next;
    return ++token;
  };

  const handlers: MockHandlers = {
    spec_get: () => ({ reply: "spec" as const, text, undoToken: null }),
    spec_set: (request) => {
      const syntax = syntaxError(request.args.text);
      if (syntax !== null) {
        throw new IpcFailure({
          code: syntax.code,
          message: syntax.message,
          hint: null,
          pointer: null,
        });
      }
      const undoToken = record("Set spec", request.args.text);
      return { reply: "spec" as const, text, undoToken };
    },
    spec_validate: (request) => {
      const diagnostics = validate(request.args.text);
      return {
        reply: "validation" as const,
        diagnostics,
        counts: counts(diagnostics),
      };
    },
    spec_apply: (request): SpecPatchResult & { reply: "spec_patch" } => {
      const document: unknown = JSON.parse(text ?? "{}");
      for (const op of request.args.patch) {
        if (op.op === "replace" || op.op === "add") {
          const parts = op.path.slice(1).split("/");
          const last = parts.pop() ?? "";
          let target: unknown = document;
          for (const part of parts) {
            target =
              isObject(target) || Array.isArray(target)
                ? (target as Record<string, unknown>)[part]
                : undefined;
          }
          if (isObject(target)) target[last] = op.value;
        }
      }
      const next = JSON.stringify(document, null, 2);
      const undoToken = record(request.args.label ?? "Edit spec", next);
      const diagnostics = validate(next);
      return {
        reply: "spec_patch" as const,
        undoToken,
        text: next,
        diagnostics,
        counts: counts(diagnostics),
      };
    },
    compile: (): CompileReport & { reply: "compile" } => {
      const diagnostics = text === null ? [] : validate(text);
      const ok =
        text !== null && !diagnostics.some((d) => d.severity === "error");
      return {
        reply: "compile" as const,
        ok,
        diagnostics,
        counts: counts(diagnostics),
        score: ok
          ? {
              durationS: 12.5,
              mode: "analysis",
              specHash: "mock",
              scoreHash: "mock",
              tracks: [
                {
                  id: "temperature",
                  instrument: "sine",
                  events: 125,
                  estimated: 0,
                },
              ],
              engines: {},
            }
          : null,
      };
    },
    history_undo: () => {
      const edit = done.pop();
      if (edit !== undefined) {
        undone.push(edit);
        text = edit.before;
      }
      return {
        reply: "history" as const,
        token: edit === undefined ? null : ++token,
        label: edit?.label ?? null,
        canUndo: done.length > 0,
        canRedo: undone.length > 0,
        spec: { text, undoToken: null },
        corrections: null,
      };
    },
    history_redo: () => {
      const edit = undone.pop();
      if (edit !== undefined) {
        done.push(edit);
        text = edit.after;
      }
      return {
        reply: "history" as const,
        token: edit === undefined ? null : ++token,
        label: edit?.label ?? null,
        canUndo: done.length > 0,
        canRedo: undone.length > 0,
        spec: { text, undoToken: null },
        corrections: null,
      };
    },
  };
  return { handlers, text: () => text };
}
