// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { DiagnosticList } from "@sonarithm/ui/diagnostic-list";
import { useMemo, useSyncExternalStore } from "react";

import type { RoutedAnnouncer } from "../announcer/announcer.ts";
import { useLocalizer } from "../i18n.ts";
import { useSpecContext, useSpecSnapshot } from "./context.ts";
import {
  diagnosticSpeech,
  locationText,
  place,
  severityLabel,
} from "./place.ts";

const noop = () => () => undefined;
const none = () => null;

/**
 * The Status region's Diagnostics tab (B05-T03; keyboard-and-navigation §5.10): the latest
 * check's diagnostics, from live validation, an apply or a compile. Enter on one goes to its
 * place in the spec editor and says it, as F8 does there. Loaded with the tab, since it
 * parses the spec to place each diagnostic.
 */
export default function DiagnosticsPanel({
  announcer,
  showSpec,
}: {
  readonly announcer: RoutedAnnouncer;
  readonly showSpec: () => void;
}) {
  const t = useLocalizer();
  const context = useSpecContext();
  const snapshot = useSpecSnapshot(context?.store ?? null);
  const draft = useSyncExternalStore(
    context?.bridge.subscribe ?? noop,
    context?.bridge.draft ?? none,
  );
  // Only diagnostics of the text in the editor (or, before it loads, the session's): an
  // older check's offsets would point into text that is gone.
  const current = draft ?? snapshot.text ?? "";
  const checked =
    snapshot.checked !== null && snapshot.checked.text === current
      ? snapshot.checked
      : null;
  const placed = useMemo(
    () => (checked === null ? [] : place(checked)),
    [checked],
  );
  const rows = placed.map((d) => ({
    id: d.id,
    severity: d.severity,
    severityLabel: severityLabel(t, d.severity),
    code: d.info.code,
    message: d.info.message,
    location: locationText(t, d.at),
  }));
  return (
    <DiagnosticList
      label={t.format("ui-tab-diagnostics")}
      rows={rows}
      empty={
        checked === null && current.trim() !== ""
          ? t.format("ui-spec-checking")
          : t.format("ui-no-diagnostics")
      }
      onActivate={(id) => {
        const target = placed.find((d) => d.id === id);
        if (target === undefined || context === null) return;
        showSpec();
        context.bridge.withEditor((editor) => {
          editor.select(target.span?.from ?? 0);
          announcer.announce({
            text: diagnosticSpeech(t, target),
            assertive: false,
          });
        });
      }}
    />
  );
}
