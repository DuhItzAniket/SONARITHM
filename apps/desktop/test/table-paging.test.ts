// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The data table's windows and text (B04-T05): window arithmetic, the true row number as
// the row header, provenance in the words speech uses, and missing values said in words.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";

import { SOURCE_LOCALE, createLocalizer } from "@sonarithm/i18n";
import type { DatasetSummary } from "@sonarithm/ipc";
import { encodeView, readView } from "@sonarithm/ipc";

import {
  PROVENANCE_ID,
  ROW_NUMBER_COLUMN,
  WINDOW_ROWS,
  fieldColumnId,
  lastWindowStart,
  provenanceText,
  rangeText,
  windowOf,
  windowStart,
} from "../src/table/paging.ts";

const locales = new URL(
  "../../../crates/sonarithm-i18n/locales/en-US/",
  import.meta.url,
);
const t = createLocalizer(
  SOURCE_LOCALE,
  readdirSync(locales)
    .filter((name) => name.endsWith(".ftl"))
    .sort()
    .map((name) => readFileSync(new URL(name, locales), "utf8")),
);

const DATASET: DatasetSummary = {
  id: "d1",
  name: "Readings",
  rows: 1_000_000,
  fields: [
    {
      name: "t",
      dataType: "f64",
      fieldType: null,
      role: null,
      unit: "s",
      concept: null,
    },
    {
      name: "station",
      dataType: "utf8",
      fieldType: null,
      role: null,
      unit: null,
      concept: null,
    },
  ],
  diagnostics: [],
};

void test("b04_t05 windows are aligned and the last one holds the last row", () => {
  assert.equal(windowStart(0), 0);
  assert.equal(windowStart(WINDOW_ROWS - 1), 0);
  assert.equal(windowStart(WINDOW_ROWS), WINDOW_ROWS);
  assert.equal(windowStart(500_000), 500_000);
  assert.equal(lastWindowStart(1_000_000), 999_000);
  assert.equal(lastWindowStart(1_001), 1_000);
  assert.equal(lastWindowStart(0), 0);
});

void test("b04_t05 the range names the loaded rows in the whole table", () => {
  assert.equal(
    rangeText(t, 10_000, 1_000, 1_000_000),
    "Rows 10,001 to 11,000 of 1,000,000",
  );
});

void test("a11y_03 the row header is the true row number and estimates say so", () => {
  const view = readView(
    encodeView({
      kind: 3,
      generation: 1,
      columns: [
        { name: "__snr_row", dataType: "i64", values: [500_000n, 500_001n] },
        {
          name: "t",
          dataType: "f64",
          values: [1.5, 2.25],
          validity: [true, false],
        },
        { name: "station", dataType: "utf8", values: ["North", "South"] },
        { name: "__snr_prov", dataType: "provenance", values: [0, 2] },
        {
          name: "__snr_conf",
          dataType: "confidence",
          values: [0, 0.8125],
          validity: [false, true],
        },
      ],
    }),
  );
  const window = windowOf(t, DATASET, view, 500_000);
  assert.deepEqual(
    window.columns.map((column) => column.id),
    [
      ROW_NUMBER_COLUMN,
      fieldColumnId("t"),
      fieldColumnId("station"),
      PROVENANCE_ID,
    ],
  );
  assert.equal(window.columns[0]?.isRowHeader, true);
  assert.equal(window.columns[1]?.name, "t (s)");
  const [first, second] = window.rows;
  assert.ok(first !== undefined && second !== undefined);
  assert.equal(first.cells[ROW_NUMBER_COLUMN], "500,001");
  assert.equal(second.cells[ROW_NUMBER_COLUMN], "500,002");
  assert.equal(first.cells[fieldColumnId("t")], "1.5");
  // A missing value is said in words, never a blank cell.
  assert.equal(second.cells[fieldColumnId("t")], "no value");
  assert.equal(first.cells[PROVENANCE_ID], "exact");
  assert.equal(
    second.cells[PROVENANCE_ID],
    "estimated from a chart image, confidence 0.81",
  );
});

void test("a11y_24 provenance uses speech's words for every class", () => {
  assert.equal(provenanceText(t, 1, null), "calibrated");
  assert.equal(provenanceText(t, 5, null), "edited");
  assert.equal(
    provenanceText(t, 3, null),
    "estimated by a machine-learning model",
  );
  assert.equal(provenanceText(t, 6, null), "derived from other values");
});
