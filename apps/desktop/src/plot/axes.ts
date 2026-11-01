// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { DatasetSummary, FieldSummary } from "@sonarithm/ipc";

/** Arrow's names for numbers and times (`Float64`, `Int32`, `Timestamp(…)`, `Date32`). */
const NUMERIC = /^(Float|Int|UInt|Decimal)/;
const TEMPORAL = /^(Timestamp|Date|Time|Duration)/;

function isNumeric(field: FieldSummary): boolean {
  return NUMERIC.test(field.dataType);
}

function isTemporal(field: FieldSummary): boolean {
  return field.fieldType === "temporal" || TEMPORAL.test(field.dataType);
}

/**
 * The plot's axes until the spec chooses them (B04-T06): x is the first time field, or else
 * the first numeric one; y is the next numeric field. `null` when there are not two.
 */
export function axesOf(
  dataset: DatasetSummary,
): { readonly x: FieldSummary; readonly y: FieldSummary } | null {
  const x =
    dataset.fields.find(isTemporal) ?? dataset.fields.find(isNumeric) ?? null;
  const y =
    x === null
      ? null
      : (dataset.fields.find((field) => field !== x && isNumeric(field)) ??
        null);
  return x === null || y === null ? null : { x, y };
}

/** The index of the value in ascending `xs` nearest to `value`; -1 when `xs` is empty. */
export function nearest(xs: ArrayLike<number>, value: number): number {
  if (xs.length === 0) return -1;
  let low = 0;
  let high = xs.length - 1;
  while (low < high) {
    const middle = (low + high) >> 1;
    if ((xs[middle] ?? 0) < value) low = middle + 1;
    else high = middle;
  }
  const before = low - 1;
  if (
    before >= 0 &&
    Math.abs((xs[before] ?? 0) - value) <= Math.abs((xs[low] ?? 0) - value)
  ) {
    return before;
  }
  return low;
}
