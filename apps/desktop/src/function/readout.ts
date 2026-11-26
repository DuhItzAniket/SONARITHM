// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

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
): number | null {
  if (cursor?.dataset !== dataset || cursor.row === null) {
    return null;
  }
  const at = points.findIndex((point) => point.row === cursor.row);
  return at < 0 ? null : at;
}
