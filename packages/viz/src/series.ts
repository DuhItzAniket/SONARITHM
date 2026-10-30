// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * Redundant series encoding (design-language §6): with its colour, every series has its own
 * dash pattern, so two series never differ by colour alone (DL-X08). Series 1 is solid; the
 * others alternate dash lengths and gaps so neighbours look different at a glance. Lengths
 * are CSS pixels.
 */
export const DASHES: readonly (readonly number[])[] = [
  [],
  [8, 4],
  [2, 3],
  [12, 3, 2, 3],
  [5, 5],
  [1, 6],
  [10, 2, 2, 2, 2, 2],
  [16, 6],
];

/** The colour token of the series at `index`; past eight the palette wraps (§6). */
export function seriesToken(index: number): string {
  return `--series-${String((index % 8) + 1)}`;
}

/** The dash pattern of the series at `index`; past eight it wraps with the palette. */
export function dashFor(index: number): readonly number[] {
  return DASHES[index % DASHES.length] ?? [];
}
