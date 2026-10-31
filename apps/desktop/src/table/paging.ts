// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  BinaryView,
  DatasetSummary,
  FieldSummary,
  ViewColumn,
} from "@sonarithm/ipc";
import { BINARY, PROVENANCE_CLASSES, isValid } from "@sonarithm/ipc";
import type { Localizer } from "@sonarithm/i18n";
import type { DataColumn, DataRow } from "@sonarithm/ui";

/**
 * The rows the table holds at a time (B04-T05; 09 §3 as built). React Aria's Table builds a
 * node per row and numbers rows from that collection, so a million rows cannot be one
 * collection; the table holds one window, fetched with `table_page` (at most 10 000 rows),
 * and says where it sits in the whole table in words.
 */
export const WINDOW_ROWS = 1000;

/** The provenance and confidence columns of a table page (ADR-0017 §6). */
const PROVENANCE_COLUMN = "__snr_prov";
const CONFIDENCE_COLUMN = "__snr_conf";
const INTERNAL = new Set<string>([
  BINARY.rowColumn,
  PROVENANCE_COLUMN,
  CONFIDENCE_COLUMN,
]);

/** The column that names each row: its row number in the table's current order. */
export const ROW_NUMBER_COLUMN = "row";
/** The provenance column's ID. */
export const PROVENANCE_ID = "provenance";

/** The first row of the window that holds `row` (0-based). */
export function windowStart(row: number): number {
  return Math.floor(Math.max(0, row) / WINDOW_ROWS) * WINDOW_ROWS;
}

/** The first row of the last window of a table of `total` rows. */
export function lastWindowStart(total: number): number {
  return total === 0 ? 0 : windowStart(total - 1);
}

/** The column ID of field `name`; prefixed so no field can take a fixed column's ID. */
export function fieldColumnId(name: string): string {
  return `field:${name}`;
}

/** The "Rows 1 to 1,000 of 1,000,000" range of a window of `count` rows from `start`. */
export function rangeText(
  t: Localizer,
  start: number,
  count: number,
  total: number,
): string {
  return t.format("ui-table-rows", {
    from: { number: count === 0 ? 0 : start + 1 },
    to: { number: start + count },
    total: { number: total },
  });
}

/** The header of a field: its name, with its unit where it has one. */
export function headingOf(t: Localizer, field: FieldSummary): string {
  return field.unit === null || field.unit === ""
    ? field.name
    : t.format("ui-table-heading-unit", { name: field.name, unit: field.unit });
}

/**
 * The provenance text of one row, in the words speech uses (C02-T09; 03 §4 rule 3): plain
 * classes are named ("exact", "calibrated", "edited"), and estimates say what estimated them
 * and, where known, their confidence with two decimals.
 */
export function provenanceText(
  t: Localizer,
  code: number,
  confidence: number | null,
): string {
  const name = PROVENANCE_CLASSES[code] ?? "exact";
  if (name === "exact" || name === "vector" || name === "user_edited") {
    return t.format("ui-table-provenance-plain", { class: name });
  }
  const qualifier = t.format("announce-provenance", { class: name });
  return confidence === null
    ? qualifier
    : t.format("announce-provenance-confidence", {
        qualifier,
        confidence: {
          number: confidence,
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        },
      });
}

/** One cell's text: numbers in the locale's format; a missing value said in words. */
function cellText(
  t: Localizer,
  numbers: Intl.NumberFormat,
  column: ViewColumn,
  row: number,
): string {
  if (!isValid(column, row)) {
    return t.format("ui-table-no-value");
  }
  switch (column.dataType) {
    case "f64":
      return numbers.format(column.values[row] ?? Number.NaN);
    case "i64":
      return numbers.format(column.values[row] ?? 0n);
    case "utf8":
      return column.values[row] ?? "";
    case "bool":
      return t.format(
        column.values[row] === 1 ? "ui-table-yes" : "ui-table-no",
      );
    case "provenance":
    case "confidence":
      return "";
  }
}

/** A table window ready for `DataTable`. */
export interface TableWindow {
  /** The first row, 0-based, in the table's current order. */
  readonly start: number;
  readonly columns: readonly DataColumn[];
  readonly rows: readonly DataRow[];
  /**
   * Each row's index in the dataset's own order (`__snr_row`), what the cursor names
   * (B04-T06); the same as its position while the table is not sorted.
   */
  readonly datasetRows: readonly number[];
}

/**
 * Turns a `table_page` view into the table's columns and rows. The first column is the row
 * number, a row header, so a screen reader names every cell by its true position in the
 * whole table (A11Y-03): React Aria's own row index only counts the window. A provenance
 * column follows the fields when the page carries one (AGENTS rule 7).
 */
export function windowOf(
  t: Localizer,
  dataset: DatasetSummary,
  view: BinaryView,
  start: number,
): TableWindow {
  const numbers = new Intl.NumberFormat(t.locale, { maximumFractionDigits: 6 });
  const fields = new Map(dataset.fields.map((field) => [field.name, field]));
  const dataColumns = view.columns.filter(
    (column) => !INTERNAL.has(column.name),
  );
  const provenance = view.columns.find((c) => c.name === PROVENANCE_COLUMN);
  const confidence = view.columns.find((c) => c.name === CONFIDENCE_COLUMN);
  const columns: DataColumn[] = [
    {
      id: ROW_NUMBER_COLUMN,
      name: t.format("ui-table-row-number"),
      isRowHeader: true,
      numeric: true,
    },
    ...dataColumns.map((column) => {
      const field = fields.get(column.name);
      return {
        id: fieldColumnId(column.name),
        name: field === undefined ? column.name : headingOf(t, field),
        numeric: column.dataType === "f64" || column.dataType === "i64",
        allowsSorting: true,
      };
    }),
    ...(provenance === undefined
      ? []
      : [{ id: PROVENANCE_ID, name: t.format("ui-table-provenance") }]),
  ];
  const rows: DataRow[] = [];
  const indices = view.columns.find((c) => c.name === BINARY.rowColumn);
  const datasetRows: number[] = [];
  for (let row = 0; row < view.rows; row += 1) {
    datasetRows.push(
      indices?.dataType === "i64"
        ? Number(indices.values[row] ?? BigInt(start + row))
        : start + row,
    );
    const cells: Record<string, string> = {
      [ROW_NUMBER_COLUMN]: numbers.format(start + row + 1),
    };
    for (const column of dataColumns) {
      cells[fieldColumnId(column.name)] = cellText(t, numbers, column, row);
    }
    if (provenance?.dataType === "provenance") {
      const value =
        confidence?.dataType === "confidence" && isValid(confidence, row)
          ? (confidence.values[row] ?? null)
          : null;
      cells[PROVENANCE_ID] = provenanceText(
        t,
        provenance.values[row] ?? 0,
        value,
      );
    }
    rows.push({ id: start + row, cells });
  }
  return { start, columns, rows, datasetRows };
}
