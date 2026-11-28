// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The calibration dialog's model (C02-T10, ADR-0017 "Raster calibration"): anchors at pixel
// centres, their short names, the checks in words, and the crosshair's checks sent latest
// only, at most once per frame.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";

import { SOURCE_LOCALE, createLocalizer } from "@sonarithm/i18n";

import type { DialogState } from "../src/calibrate/model.ts";
import {
  Latest,
  START,
  checkLines,
  labelOf,
  takesNumbers,
  toCalibration,
  valueText,
} from "../src/calibrate/model.ts";

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

const STATE: DialogState = {
  kinds: { x: "date", y: "linear" },
  anchors: [
    { id: 1, axis: "x", at: [20, 100], value: "2024-01-01" },
    { id: 2, axis: "y", at: [20, 100], value: 0 },
    { id: 3, axis: "x", at: [180, 100], value: "2024-12-31" },
  ],
  area: "anchors",
};

test("c02_t10 anchors are sent at their pixel centres, with dates as text", () => {
  const calibration = toCalibration(STATE);
  assert.deepEqual(calibration.x, {
    kind: "date",
    anchors: [
      { at: [20.5, 100.5], value: "2024-01-01" },
      { at: [180.5, 100.5], value: "2024-12-31" },
    ],
  });
  assert.deepEqual(calibration.y.anchors, [{ at: [20.5, 100.5], value: 0 }]);
  assert.equal(calibration.plotArea, null);
  assert.equal(calibration.area, "anchors", "the anchors' area unless chosen");
  assert.equal(takesNumbers("log"), true);
  assert.equal(takesNumbers("category"), false);
});

test("c02_t10 anchors are named by axis and order", () => {
  const names = STATE.anchors.map((anchor) => labelOf(STATE, anchor));
  assert.deepEqual(names, ["x1", "y1", "x2"]);
});

test("c02_t10 the checks say how each axis stands, then the app core's messages", () => {
  const pending = checkLines(t, START, { x: null, y: null }, []);
  assert.deepEqual(pending, [
    "x axis: 0 anchors, at least 2 needed",
    "y axis: 0 anchors, at least 2 needed",
  ]);
  const fitted = checkLines(
    t,
    STATE,
    { x: { anchors: 2, residual: 0.0012, confidence: 0.99 }, y: null },
    ["The y axis cannot be calibrated: it has fewer than two anchors."],
  );
  assert.deepEqual(fitted, [
    "x axis: 2 anchors fit within 0.12%",
    "y axis: 1 anchor, at least 2 needed",
    "The y axis cannot be calibrated: it has fewer than two anchors.",
  ]);
  assert.equal(valueText(t, 1234.56789), "1,234.5679");
  assert.equal(valueText(t, "2024-06-01"), "2024-06-01");
  assert.equal(valueText(t, null), null);
});

test("c02_t10 only the latest crosshair position is sent, once per frame, never while one is in flight", async () => {
  const frames: (() => void)[] = [];
  const sent: number[] = [];
  let finish: (() => void) | null = null;
  const latest = new Latest<number>(
    (value) => {
      sent.push(value);
      return new Promise((resolve) => {
        finish = resolve;
      });
    },
    (run) => {
      frames.push(run);
    },
    () => {
      assert.fail("no send fails here");
    },
  );
  latest.push(1);
  latest.push(2);
  latest.push(3);
  assert.equal(frames.length, 1, "one frame asked for");
  frames.shift()?.();
  assert.deepEqual(sent, [3], "only the newest of a frame");
  latest.push(4);
  latest.push(5);
  assert.equal(frames.length, 0, "nothing scheduled while a send is in flight");
  (finish as (() => void) | null)?.();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(frames.length, 1);
  frames.shift()?.();
  assert.deepEqual(
    sent,
    [3, 5],
    "the newest value waiting is sent next; 4 never is",
  );
});

test("c02_t10 a failed check is handed on, never swallowed, and the next value still goes", async () => {
  const frames: (() => void)[] = [];
  const failures: unknown[] = [];
  const sent: number[] = [];
  const latest = new Latest<number>(
    (value) => {
      sent.push(value);
      return value === 1
        ? Promise.reject(new Error("SNR-APP-096"))
        : Promise.resolve();
    },
    (run) => {
      frames.push(run);
    },
    (error) => {
      failures.push(error);
    },
  );
  latest.push(1);
  frames.shift()?.();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(failures.length, 1);
  latest.push(2);
  frames.shift()?.();
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(sent, [1, 2]);
  assert.equal(failures.length, 1);
});
