// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { CompileReport } from "@sonarithm/ipc";
import type { Localizer } from "@sonarithm/i18n";

import type { RoutedAnnouncer } from "../announcer/announcer.ts";
import { useCommand } from "../commands/context.tsx";
import { useLocalizer } from "../i18n.ts";
import { useSpecContext } from "./context.ts";
import type { PlacedDiagnostic } from "./speech.ts";
import { diagnosticSpeech, stepDiagnostic } from "./speech.ts";
import type { ApplyOutcome, HistoryOutcome } from "./store.ts";

/** What Apply says: applied and compiled, applied with errors, or not applied. */
export function applySpeech(t: Localizer, outcome: ApplyOutcome): string {
  if (outcome.kind === "notApplied") {
    return t.format("ui-spec-not-applied", { cause: outcome.error.message });
  }
  if (outcome.kind === "appliedNotCompiled") {
    return t.format("ui-spec-applied-compile-failed", {
      cause: outcome.error.message,
    });
  }
  const report: CompileReport = outcome.report;
  if (report.ok && report.score !== null) {
    return t.format("ui-spec-applied", {
      count: { number: report.score.tracks.length },
      duration: t.format("unit-second", {
        style: "verbose",
        value: { number: report.score.durationS, maximumFractionDigits: 1 },
      }),
      warnings: { number: report.counts.warnings },
    });
  }
  const first = report.diagnostics.find((d) => d.severity === "error");
  return t.format("ui-spec-applied-not-compiled", {
    errors: { number: report.counts.errors },
    detail: first === undefined ? "" : `${first.code}: ${first.message}`,
  });
}

/** What Undo and Redo say about the spec history (B05-T03): which history, and the edit. */
export function historySpeech(
  t: Localizer,
  step: "undo" | "redo",
  outcome: HistoryOutcome,
): string {
  if (outcome.kind === "failed") {
    return outcome.error.message;
  }
  if (outcome.kind === "nothing") {
    return t.format(
      step === "undo" ? "ui-nothing-to-undo" : "ui-nothing-to-redo",
    );
  }
  return t.format(step === "undo" ? "ui-undo-spec" : "ui-redo-spec", {
    label: outcome.label,
  });
}

/**
 * The spec editor's commands (B05-T03; keyboard-and-navigation §5.9), registered for the
 * whole app like the explorer's, so the palette lists and runs them from anywhere (KB-01).
 * Their keys work only in the editor (scope `editor`). A command run while the Spec tab is
 * hidden shows it first and waits for the editor to load.
 *
 * Undo and Redo outside the editor step the spec history (`history_undo`, `history_redo`)
 * and say so. Inside it, Primary+Z undoes typing until there is none left to undo; the
 * editor says which happened, so the two histories are never confused.
 */
export function SpecCommands({
  announcer,
  showSpec,
}: {
  readonly announcer: RoutedAnnouncer;
  /** Selects the Spec tab, so the editor renders. */
  readonly showSpec: () => void;
}) {
  const t = useLocalizer();
  const context = useSpecContext();
  const say = (text: string, assertive = false) => {
    announcer.announce({ text, assertive });
  };
  const withEditor: NonNullable<typeof context>["bridge"]["withEditor"] = (
    action,
  ) => {
    if (context === null) return;
    showSpec();
    context.bridge.withEditor(action, () => {
      say(t.format("ui-spec-editor-unavailable"), true);
    });
  };
  const goTo = (direction: "next" | "previous") => {
    withEditor((editor) => {
      const bridge = context?.bridge;
      const placed: readonly PlacedDiagnostic[] = bridge?.diagnostics ?? [];
      const sorted = placed.map((d) => ({ d, from: d.span?.from ?? 0 }));
      const caret = editor.caret();
      const index = stepDiagnostic(
        sorted,
        caret,
        bridge?.lastStep ?? null,
        direction,
      );
      const target = index === null ? undefined : sorted[index];
      if (index === null || target === undefined) {
        editor.focus();
        say(t.format("ui-spec-no-diagnostics"));
        return;
      }
      editor.select(target.from);
      if (bridge !== undefined) bridge.lastStep = { index, caret: target.from };
      say(diagnosticSpeech(t, target.d));
    });
  };

  useCommand("spec.edit", () => {
    withEditor((editor) => {
      editor.focus();
    });
  });
  useCommand("spec.apply", () => {
    withEditor((editor) => {
      const store = context?.store;
      if (store === undefined) return;
      void store
        .apply(editor.text(), () => {
          // The applied text is now a spec change: Undo outside steps it back, and the
          // editor's own undo starts again from here, as soon as the spec is set, so what
          // is typed while the compile runs keeps its own undo.
          editor.resetHistory();
        })
        .then((outcome) => {
          say(applySpeech(t, outcome), outcome.kind !== "applied");
        });
    });
  });
  useCommand("spec.nextDiagnostic", () => {
    goTo("next");
  });
  useCommand("spec.previousDiagnostic", () => {
    goTo("previous");
  });
  useCommand("spec.format", () => {
    withEditor((editor) => {
      // The formatter loads with the editor's parser, not at start-up (PB-07).
      void import("./format.ts").then(({ formatJson }) => {
        const formatted = formatJson(editor.text());
        if (!formatted.ok) {
          editor.select(formatted.offset);
          say(t.format("ui-spec-format-failed"));
          return;
        }
        editor.replaceAll(formatted.text);
        editor.focus();
        say(t.format("ui-spec-formatted"));
      });
    });
  });
  useCommand("spec.complete", () => {
    withEditor((editor) => {
      editor.focus();
      if (!editor.complete()) say(t.format("ui-spec-no-completions"));
    });
  });
  useCommand("spec.find", () => {
    withEditor((editor) => {
      editor.openSearch();
    });
  });
  const history = (step: "undo" | "redo") => {
    const store = context?.store;
    if (store === undefined) return;
    void (step === "undo" ? store.undo() : store.redo()).then((outcome) => {
      say(historySpeech(t, step, outcome));
    });
  };
  useCommand("edit.undo", () => {
    history("undo");
  });
  useCommand("edit.redo", () => {
    history("redo");
  });
  return null;
}
