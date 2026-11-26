// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Localizer } from "@sonarithm/i18n";
import type { DiagnosticInfo } from "@sonarithm/ipc";

/**
 * Diagnostics in words, and the step from one to the next (B05-T03). No parser here: the
 * app's commands use this at start-up, while placing diagnostics in the text (`place.ts`)
 * loads with the editor.
 */

/** A span of the spec text: offsets from and to. */
export interface Span {
  readonly from: number;
  readonly to: number;
}

/** A line and column, both counted from 1. */
export interface LineColumn {
  readonly line: number;
  readonly column: number;
}

/** The severities the app core reports (DiagnosticInfo.severity). */
export type Severity = "error" | "warning" | "info";

/** A diagnostic placed in the text it was found in. */
export interface PlacedDiagnostic {
  /** Its index in the validator's order, as text: stable while the check is current. */
  readonly id: string;
  readonly info: DiagnosticInfo;
  readonly severity: Severity;
  /** Where it is, or `null` for the whole spec. */
  readonly span: Span | null;
  readonly at: LineColumn | null;
}

/** The next (or previous) diagnostic after `caret`, wrapping at the ends; `null` if none. */
export function nextDiagnosticFrom<D extends { readonly from: number }>(
  diagnostics: readonly D[],
  caret: number,
  direction: "next" | "previous",
): D | null {
  const sorted = [...diagnostics].sort((a, b) => a.from - b.from);
  if (direction === "next") {
    return sorted.find((d) => d.from > caret) ?? sorted[0] ?? null;
  }
  return sorted.findLast((d) => d.from < caret) ?? sorted.at(-1) ?? null;
}

/** The severity in words. */
export function severityLabel(t: Localizer, severity: Severity): string {
  return t.format(`ui-severity-${severity}`);
}

/** "Line 4, column 12", or "Whole spec". */
export function locationText(t: Localizer, at: LineColumn | null): string {
  return at === null
    ? t.format("ui-diagnostic-location-none")
    : t.format("ui-diagnostic-location", {
        line: { number: at.line },
        column: { number: at.column },
      });
}

/** The code and the app core's message, as one detail. */
export function detailOf(info: DiagnosticInfo): string {
  return `${info.code}: ${info.message}`;
}

/** What F8 and Enter on a diagnostic say (keyboard §5.9, §5.10). */
export function diagnosticSpeech(
  t: Localizer,
  placed: PlacedDiagnostic,
): string {
  const at = placed.at ?? { line: 1, column: 1 };
  return t.format("ui-spec-diagnostic", {
    severity: severityLabel(t, placed.severity),
    line: { number: at.line },
    column: { number: at.column },
    detail: detailOf(placed.info),
  });
}
