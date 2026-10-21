// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import assert from "node:assert/strict";
import { test } from "node:test";

import { BinaryViewError, isValid, readView } from "./binary.ts";
import { SAMPLE_VIEW, SAMPLE_VIEW_BASE64 } from "./generated/fixtures.ts";
import { BINARY, PROVENANCE_CLASSES } from "./generated/protocol.ts";

function sampleBuffer(): ArrayBuffer {
  const bytes = Uint8Array.from(atob(SAMPLE_VIEW_BASE64), (c) =>
    c.charCodeAt(0),
  );
  return bytes.buffer;
}

void test("a view encoded by Rust reads back with its contents", () => {
  const view = readView(sampleBuffer());
  assert.equal(view.kind, SAMPLE_VIEW.kind);
  assert.equal(view.rows, SAMPLE_VIEW.rows);
  assert.equal(view.generation, SAMPLE_VIEW.generation);
  assert.equal(view.columns.length, SAMPLE_VIEW.columns.length);
  SAMPLE_VIEW.columns.forEach((expected, index) => {
    const column = view.columns[index];
    assert.ok(column !== undefined);
    assert.equal(column.name, expected.name);
    const values: unknown[] = [];
    switch (column.dataType) {
      case "i64":
        values.push(...Array.from(column.values, String));
        break;
      case "bool":
        values.push(...Array.from(column.values, (byte) => byte !== 0));
        break;
      case "f64":
        values.push(...column.values);
        break;
      case "utf8":
        values.push(...column.values);
        break;
      case "provenance":
        values.push(...column.values);
        break;
      case "confidence":
        values.push(...column.values);
        break;
    }
    assert.deepEqual(values, [...expected.values]);
    const validity = Array.from({ length: view.rows }, (_, row) =>
      isValid(column, row),
    );
    assert.deepEqual(validity, expected.validity ?? validity.map(() => true));
  });
});

void test("numeric columns are views, not copies", () => {
  const buffer = sampleBuffer();
  const column = readView(buffer).columns[0];
  assert.ok(column?.dataType === "f64");
  assert.equal(column.values.buffer, buffer);
});

void test("other buffers are refused", () => {
  assert.throws(
    () => readView(new TextEncoder().encode("JSON").buffer),
    BinaryViewError,
  );
  assert.throws(() => readView(sampleBuffer().slice(0, 40)), BinaryViewError);
});

void test("an unknown data type or header version is refused with SNR-APP-050", () => {
  const unknownType = new Uint8Array(sampleBuffer());
  unknownType[BINARY.headerBytes] = 99;
  assert.throws(
    () => readView(unknownType.buffer),
    (error: unknown) =>
      error instanceof BinaryViewError && error.code === "SNR-APP-050",
  );
  const newer = new Uint8Array(sampleBuffer());
  newer[BINARY.offset.version] = 2;
  assert.throws(() => readView(newer.buffer), BinaryViewError);
});

void test("provenance codes name the 03 §4 classes", () => {
  const column = readView(sampleBuffer()).columns.find(
    (c) => c.name === "__snr_prov",
  );
  assert.ok(column?.dataType === "provenance");
  assert.deepEqual(
    Array.from(column.values, (code) => PROVENANCE_CLASSES[code]),
    ["exact", "cv_estimate", "derived"],
  );
});
