// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The Plot tab's axes and click target (B04-T06).
import assert from "node:assert/strict";
import { test } from "node:test";

import type { DatasetSummary, FieldSummary } from "@sonarithm/ipc";

import { axesOf, nearest } from "../src/plot/axes.ts";

const field = (
  name: string,
  dataType: string,
  fieldType: string | null = null,
): FieldSummary => ({
  name,
  dataType,
  fieldType,
  role: null,
  unit: null,
  concept: null,
});

const dataset = (fields: FieldSummary[]): DatasetSummary => ({
  id: "d1",
  name: "data.csv",
  rows: 3,
  fields,
  diagnostics: [],
});

void test("b04_t06 a time field is x and the next number is y", () => {
  const axes = axesOf(
    dataset([
      field("station", "Utf8"),
      field("temp", "Float64"),
      field("when", "Timestamp(Millisecond, None)", "temporal"),
    ]),
  );
  assert.deepEqual([axes?.x.name, axes?.y.name], ["when", "temp"]);
});

void test("b04_t06 without a time field the first two numbers are used", () => {
  const axes = axesOf(
    dataset([
      field("day", "Int64"),
      field("label", "Utf8"),
      field("v", "Float32"),
    ]),
  );
  assert.deepEqual([axes?.x.name, axes?.y.name], ["day", "v"]);
  assert.equal(axesOf(dataset([field("only", "Float64")])), null);
  assert.equal(axesOf(dataset([field("a", "Utf8")])), null);
});

void test("b04_t06 a click picks the nearest x", () => {
  const xs = [0, 1, 2, 4, 8];
  assert.equal(nearest(xs, -3), 0);
  assert.equal(nearest(xs, 2.9), 2);
  assert.equal(nearest(xs, 3.1), 3);
  assert.equal(nearest(xs, 100), 4);
  assert.equal(nearest([], 1), -1);
});
