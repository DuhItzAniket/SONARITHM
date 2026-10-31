// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import assert from "node:assert/strict";
import { test } from "node:test";

import { readView } from "./binary.ts";
import type { EncodeColumn } from "./encode.ts";
import { encodeView } from "./encode.ts";
import { SAMPLE_VIEW, SAMPLE_VIEW_BASE64 } from "./generated/fixtures.ts";

const TYPES = [
  "",
  "f64",
  "i64",
  "utf8",
  "bool",
  "provenance",
  "confidence",
] as const;

void test("the encoder writes the shared sample byte for byte as Rust does", () => {
  const columns = SAMPLE_VIEW.columns.map((column): EncodeColumn => {
    const dataType = TYPES[column.dataType];
    const validity =
      column.validity === null ? {} : { validity: [...column.validity] };
    switch (dataType) {
      case "i64":
        return {
          name: column.name,
          dataType,
          values: column.values.map((v) => BigInt(v)),
          ...validity,
        };
      case "utf8":
        return {
          name: column.name,
          dataType,
          values: column.values.map(String),
          ...validity,
        };
      case "bool":
        return {
          name: column.name,
          dataType,
          values: column.values.map(Boolean),
          ...validity,
        };
      case "f64":
      case "provenance":
      case "confidence":
        return {
          name: column.name,
          dataType,
          values: column.values.map(Number),
          ...validity,
        };
      default:
        throw new Error("unknown type");
    }
  });
  const bytes = new Uint8Array(
    encodeView({
      kind: SAMPLE_VIEW.kind,
      generation: SAMPLE_VIEW.generation,
      columns,
    }),
  );
  const expected = Uint8Array.from(atob(SAMPLE_VIEW_BASE64), (c) =>
    c.charCodeAt(0),
  );
  assert.deepEqual(bytes, expected);
});

void test("an encoded page reads back", () => {
  const view = readView(
    encodeView({
      kind: 3,
      generation: 1,
      columns: [
        { name: "__snr_row", dataType: "i64", values: [10n, 11n] },
        {
          name: "t",
          dataType: "f64",
          values: [0.5, 1.5],
          validity: [true, false],
        },
      ],
    }),
  );
  assert.equal(view.rows, 2);
  assert.deepEqual(
    view.columns.map((c) => c.name),
    ["__snr_row", "t"],
  );
});
