// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The function UI's readout rules (C03-T08b): the critical-point list shows Rust's one
// cursor (A11Y-10), and an all-inaudible audification readout is "will not compile"
// (SDR-31).
import assert from "node:assert/strict";
import { test } from "node:test";

import type { CriticalPointInfo, CursorState } from "@sonarithm/ipc";

import { bandState, currentPoint } from "../src/function/readout.ts";

const point = (row: number | null): CriticalPointInfo => ({
  kind: "zero",
  x: 0,
  end: null,
  y: 0,
  row,
  text: "",
});

const cursor = (dataset: string | null, row: number | null): CursorState => ({
  seq: 1,
  dataset,
  row,
  x: null,
  node: null,
  path: [],
  excluded: [],
  origin: "nav",
  braille: null,
});

test("a11y_10_the_current_point_is_the_cursors_row_in_this_dataset", () => {
  const points = [point(10), point(null), point(30), point(30)];
  assert.equal(currentPoint(points, "f1", cursor("f1", 30)), 2);
  assert.equal(currentPoint(points, "f1", cursor("f1", 10)), 0);
  assert.equal(currentPoint(points, "f1", cursor("f1", 11)), null);
  assert.equal(currentPoint(points, "f1", cursor("d1", 10)), null);
  assert.equal(currentPoint(points, "f1", cursor("f1", null)), null);
  assert.equal(currentPoint(points, "f1", null), null);
});

test("sdr_31_an_all_inaudible_band_will_not_compile", () => {
  assert.equal(bandState([]), "audible");
  assert.equal(bandState(["SNR-ENG-461"]), "partly");
  assert.equal(bandState(["SNR-ENG-462"]), "blocked");
  assert.equal(bandState(["SNR-ENG-461", "SNR-ENG-462"]), "blocked");
});
