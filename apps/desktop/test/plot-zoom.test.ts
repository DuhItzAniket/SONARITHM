// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Zoom and pan without dragging (B04-T07; A11Y-26): the arithmetic behind the buttons and
// commands.
import assert from "node:assert/strict";
import { test } from "node:test";

import { clampRange, isFull, pan, zoom } from "../src/plot/zoom.ts";

const FULL = { min: 0, max: 100 };

void test("a11y_26 zoom in halves the view around the cursor", () => {
  assert.deepEqual(zoom(FULL, FULL, 0.5, 20), { min: 10, max: 60 });
  // Without a cursor in view, around the middle.
  assert.deepEqual(zoom(FULL, FULL, 0.5, null), { min: 25, max: 75 });
  assert.deepEqual(zoom({ min: 0, max: 10 }, FULL, 0.5, 50), {
    min: 2.5,
    max: 7.5,
  });
});

void test("a11y_26 zoom out doubles the view and stays inside the data", () => {
  assert.deepEqual(zoom({ min: 40, max: 60 }, FULL, 2, null), {
    min: 30,
    max: 70,
  });
  assert.deepEqual(zoom({ min: 0, max: 20 }, FULL, 2, 0), { min: 0, max: 40 });
  assert.deepEqual(zoom({ min: 20, max: 90 }, FULL, 2, null), FULL);
  assert.equal(isFull(zoom({ min: 20, max: 90 }, FULL, 2, null), FULL), true);
});

void test("a11y_26 pan moves a quarter of the view and stops at the ends", () => {
  assert.deepEqual(pan({ min: 40, max: 60 }, FULL, 1), { min: 45, max: 65 });
  assert.deepEqual(pan({ min: 0, max: 20 }, FULL, -1), { min: 0, max: 20 });
  assert.deepEqual(pan({ min: 85, max: 95 }, FULL, 1), {
    min: 87.5,
    max: 97.5,
  });
  assert.deepEqual(pan({ min: 90, max: 100 }, FULL, 1), { min: 90, max: 100 });
});

void test("a11y_26 a view never shrinks to nothing or leaves the data", () => {
  const tiny = clampRange({ min: 50, max: 50 }, FULL);
  assert.ok(tiny.max > tiny.min);
  assert.deepEqual(clampRange({ min: -10, max: 5 }, FULL), { min: 0, max: 15 });
});
