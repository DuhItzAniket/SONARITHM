// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { CompileReport } from "@sonarithm/ipc";
import type { Localizer } from "@sonarithm/i18n";

import type { RoutedAnnouncer } from "../announcer/announcer.ts";
import { useCommand } from "../commands/context.tsx";
import { useLocalizer } from "../i18n.ts";
import { useSpecContext } from "./context.ts";
import type { PlacedDiagnostic } from "./speech.ts";
import { diagnosticSpeech, nextDiagnosticFrom } from "./speech.ts";
import type { ApplyOutcome, HistoryOutcome } from "./store.ts";

/** What Apply says: applied and compiled, applied with errors, or not applied. */
export function applySpeech(t: Localizer, outcome: ApplyOutcome): string {
  if (outcome.kind === "notApplied") {
    return t.format("ui-spec-not-applied", { cause: outcome.error.message });
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
    context.bridge.withEditor(action);
  };
  const goTo = (direction: "next" | "previous") => {
    withEditor((editor) => {
      const placed: readonly PlacedDiagnostic[] =
        context?.bridge.diagnostics ?? [];
      const target = nextDiagnosticFrom(
        placed.map((d) => ({ d, from: d.span?.from ?? 0 })),
        editor.caret(),
        direction,
      );
      if (target === null) {
        editor.focus();
        say(t.format("ui-spec-no-diagnostics"));
        return;
      }
      editor.select(target.from);
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
      void store.apply(editor.text()).then((outcome) => {
        if (outcome.kind === "applied") {
          // The applied text is now a spec change: Undo outside steps it back, and the
          // editor's own undo starts again from here.
          editor.resetHistory();
        }
        say(applySpeech(t, outcome), outcome.kind === "notApplied");
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
      let value: unknown;
      try {
        value = JSON.parse(editor.text());
      } catch {
        say(t.format("ui-spec-format-failed"));
        return;
      }
      editor.replaceAll(`${JSON.stringify(value, null, 2)}\n`);
      editor.focus();
      say(t.format("ui-spec-formatted"));
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
