// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { DiagnosticInfo } from "@sonarithm/ipc";

import { firstSyntaxError, lineColumn, locate, parseSpec } from "./pointer.ts";
import type { Checked } from "./store.ts";
import type { PlacedDiagnostic, Severity } from "./speech.ts";

/** The app core's JSON syntax error code: it has no pointer, so its place is found here. */
const SYNTAX = "SNR-SPEC-001";

export function severityOf(info: DiagnosticInfo): Severity {
  return info.severity === "error" || info.severity === "warning"
    ? info.severity
    : "info";
}

/** Places each diagnostic of a check by its JSON pointer, in the validator's order. */
export function place(checked: Checked): PlacedDiagnostic[] {
  const spec = parseSpec(checked.text);
  return checked.diagnostics.map((info, index) => {
    const span =
      info.pointer !== null
        ? locate(spec, info.pointer)
        : info.code === SYNTAX
          ? firstSyntaxError(spec)
          : null;
    return {
      id: String(index),
      info,
      severity: severityOf(info),
      span,
      at: span === null ? null : lineColumn(checked.text, span.from),
    };
  });
}

export {
  detailOf,
  diagnosticSpeech,
  locationText,
  severityLabel,
} from "./speech.ts";
export type { PlacedDiagnostic, Severity } from "./speech.ts";
