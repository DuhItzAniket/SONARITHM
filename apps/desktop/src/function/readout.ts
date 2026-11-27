// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { MessageArgs, NumberArg } from "@sonarithm/i18n";
import type { CriticalPointInfo, CursorState } from "@sonarithm/ipc";

/** Part of the band is inaudible (`SNR-ENG-461`): the track compiles, with a warning. */
export const PART_INAUDIBLE = "SNR-ENG-461";

/** The whole band is inaudible (`SNR-ENG-462`): a compile of the track refuses it. */
export const ALL_INAUDIBLE = "SNR-ENG-462";

/**
 * What an audification readout says about its band (SDR-31): all audible, partly outside
 * hearing, or "will not compile". `audify_calculate` replies `Ok` even when the whole band
 * is inaudible, but a compile of the same track refuses it, so the UI must never show that
 * readout as a normal one.
 */
export function bandState(
  codes: readonly string[],
): "audible" | "partly" | "blocked" {
  if (codes.includes(ALL_INAUDIBLE)) return "blocked";
  if (codes.includes(PART_INAUDIBLE)) return "partly";
  return "audible";
}

/**
 * The index of the critical point the cursor is on: the first whose row is the cursor's
 * row in this dataset, or `null` (A11Y-10: the list shows Rust's one cursor, it holds none
 * of its own).
 */
export function currentPoint(
  points: readonly CriticalPointInfo[],
  dataset: string,
  cursor: CursorState | null,
  entered: number | null = null,
): number | null {
  if (cursor?.dataset !== dataset || cursor.row === null) {
    return null;
  }
  // Two points can share a row (a zero and a minimum at x = 0): the one the listener went
  // to stays current while the cursor is on its row.
  if (entered !== null && points[entered]?.row === cursor.row) {
    return entered;
  }
  const at = points.findIndex((point) => point.row === cursor.row);
  return at < 0 ? null : at;
}

/** A catalogue message and its arguments. */
export interface Message {
  readonly id: string;
  readonly args: MessageArgs;
}

/**
 * Why an expression does not read: a catalogue message, or the app core's own message as
 * it is.
 */
export type Reason = Message | { readonly text: string };

/**
 * Why an expression does not read, from `function_check`'s first diagnostic and span. An
 * error with a place is the app core's message alone: it already says where reading
 * stopped, counted from 1 ("(at character 7)"), and the span is that same place, so the
 * place is not added again (#498 review). An error with no place (a budget or a variable
 * count) says it cannot be used, and a plain sentence stands in when the app core gave no
 * reason.
 */
export function expressionReason(
  detail: string | undefined,
  span: readonly [number, number] | null,
): Reason {
  if (detail === undefined || detail === "") {
    return { id: "ui-function-expression-unreadable", args: {} };
  }
  if (span === null) {
    return { id: "ui-function-expression-refused", args: { detail } };
  }
  return { text: detail };
}

/** The readout fields the spoken summary uses. */
export interface AudifyFigures {
  readonly k: number;
  readonly durationS: number;
  readonly bandHz: readonly [number, number];
  readonly audibleHz: readonly [number, number] | null;
}

/**
 * The one polite summary said when k is committed (#498 review): k, how long the series
 * plays, its band and the audible part, with the same digits the readout shows.
 */
export function audifySummary(readout: AudifyFigures): Message {
  const common = {
    k: { number: readout.k, maximumFractionDigits: 3 },
    seconds: { number: readout.durationS, maximumFractionDigits: 3 },
    low: significant(readout.bandHz[0]),
    high: significant(readout.bandHz[1]),
  };
  if (readout.audibleHz === null) {
    return { id: "ui-audify-summary-inaudible", args: common };
  }
  return {
    id: "ui-audify-summary",
    args: {
      ...common,
      audibleLow: significant(readout.audibleHz[0]),
      audibleHigh: significant(readout.audibleHz[1]),
    },
  };
}

/**
 * A number argument that shows `value` to at least `digits` significant digits, so a small
 * band edge such as 0.04 Hz does not read as 0. Whole parts are never rounded.
 */
export function significant(value: number, digits = 3): NumberArg {
  const magnitude =
    value === 0 || !Number.isFinite(value)
      ? 0
      : Math.floor(Math.log10(Math.abs(value)));
  const fraction = Math.min(20, Math.max(0, digits - 1 - magnitude));
  return { number: value, maximumFractionDigits: fraction };
}
