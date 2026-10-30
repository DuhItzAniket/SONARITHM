// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The F6 order and the layout breakpoints (B02-T06; design-language §4, keyboard §3).
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  isSwitchable,
  layoutFor,
  nextRegion,
  REGIONS,
} from "../src/app/frame/regions.ts";

test("kb_05_f6_visits_every_region_in_order_and_wraps", () => {
  const seen: string[] = [];
  let at = nextRegion(null, false);
  while (!seen.includes(at)) {
    seen.push(at);
    at = nextRegion(at, false);
  }
  assert.deepEqual(seen, [...REGIONS]);
  assert.equal(at, "top");
});

test("kb_05_shift_f6_reverses_the_order", () => {
  assert.equal(nextRegion(null, true), "status");
  assert.equal(nextRegion("top", true), "status");
  assert.equal(nextRegion("workspace", true), "project");
});

test("the breakpoints are 960 and 1280 pixels", () => {
  assert.equal(layoutFor(1440), "wide");
  assert.equal(layoutFor(1280), "wide");
  assert.equal(layoutFor(1279), "medium");
  assert.equal(layoutFor(960), "medium");
  assert.equal(layoutFor(959), "narrow");
});

test("only the side and centre regions switch in the narrow layout", () => {
  assert.deepEqual(
    REGIONS.filter((region) => isSwitchable(region)),
    ["project", "workspace", "inspector"],
  );
});
