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

/** Where F8 last went: the index in the sorted list and the caret it left. */
export interface DiagnosticStep {
  readonly index: number;
  readonly caret: number;
}

/**
 * The index of the next (or previous) diagnostic in `sorted` (by offset), wrapping at the
 * ends; `null` if there is none. While the caret is where the last step left it, steps go by
 * index, so several diagnostics at one offset (one pointer, or the whole spec at 0) are each
 * reached; once the caret has moved, the step starts from the caret.
 */
export function stepDiagnostic(
  sorted: readonly { readonly from: number }[],
  caret: number,
  last: DiagnosticStep | null,
  direction: "next" | "previous",
): number | null {
  const count = sorted.length;
  if (count === 0) return null;
  if (last !== null && last.caret === caret && last.index < count) {
    return direction === "next"
      ? (last.index + 1) % count
      : (last.index - 1 + count) % count;
  }
  if (direction === "next") {
    const found = sorted.findIndex((d) => d.from > caret);
    return found === -1 ? 0 : found;
  }
  const found = sorted.findLastIndex((d) => d.from < caret);
  return found === -1 ? count - 1 : found;
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
