// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The function UI's readout rules (C03-T08b): the critical-point list shows Rust's one
// cursor (A11Y-10), and an all-inaudible audification readout is "will not compile"
// (SDR-31).
import assert from "node:assert/strict";
import { test } from "node:test";

import type { CriticalPointInfo, CursorState } from "@sonarithm/ipc";

import {
  audifySummary,
  bandState,
  currentPoint,
  expressionReason,
  significant,
} from "../src/function/readout.ts";

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

test("a11y_10_when_two_points_share_a_row_the_entered_one_is_current", () => {
  // A zero and a local minimum at the same row (x = 0).
  const points = [point(5), point(20), point(20)];
  assert.equal(
    currentPoint(points, "f1", cursor("f1", 20)),
    1,
    "the first, by default",
  );
  assert.equal(
    currentPoint(points, "f1", cursor("f1", 20), 2),
    2,
    "the one entered",
  );
  assert.equal(
    currentPoint(points, "f1", cursor("f1", 5), 2),
    0,
    "the cursor moved away: the row decides again",
  );
  assert.equal(
    currentPoint(points, "f1", cursor("f1", 20), 9),
    1,
    "a stale index",
  );
});

test("c03_t08b_an_expression_error_says_where_once", () => {
  // The host's message already carries the place, counted from 1, and its span is that
  // same place: the reason is the message as it is (#498 review: `sin(x))` read
  // "(at character 7). (at character 8)").
  assert.deepEqual(
    expressionReason("Unexpected `)` (at character 7).", [7, 1]),
    { text: "Unexpected `)` (at character 7)." },
  );
  assert.deepEqual(expressionReason("Too many variables.", null), {
    id: "ui-function-expression-refused",
    args: { detail: "Too many variables." },
  });
  assert.deepEqual(expressionReason(undefined, [0, 1]), {
    id: "ui-function-expression-unreadable",
    args: {},
  });
  assert.deepEqual(expressionReason("", null), {
    id: "ui-function-expression-unreadable",
    args: {},
  });
});

test("sdr_31_band_edges_keep_three_significant_digits", () => {
  const shown = (value: number) => {
    const arg = significant(value);
    return new Intl.NumberFormat("en-US", {
      maximumFractionDigits: arg.maximumFractionDigits,
    }).format(arg.number);
  };
  assert.equal(shown(0.04), "0.04");
  assert.equal(shown(0.0001234), "0.000123");
  assert.equal(shown(40), "40");
  assert.equal(shown(12.345), "12.3");
  assert.equal(shown(20_000), "20,000");
  assert.equal(shown(0), "0");
});

test("c03_t08b_a_committed_k_is_summarised_with_the_readouts_digits", () => {
  assert.deepEqual(
    audifySummary({
      k: 2200,
      durationS: 0.0045,
      bandHz: [40, 1_100_000],
      audibleHz: [40, 20_000],
    }),
    {
      id: "ui-audify-summary",
      args: {
        k: { number: 2200, maximumFractionDigits: 3 },
        seconds: { number: 0.0045, maximumFractionDigits: 3 },
        low: significant(40),
        high: significant(1_100_000),
        audibleLow: significant(40),
        audibleHigh: significant(20_000),
      },
    },
  );
  assert.equal(
    audifySummary({
      k: 0.001,
      durationS: 10,
      bandHz: [0.0001, 0.05],
      audibleHz: null,
    }).id,
    "ui-audify-summary-inaudible",
  );
});
