// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The perceptual colormaps (B04-T01; design-language §6) and the series dash patterns.
import assert from "node:assert/strict";
import { test } from "node:test";

import { colormapColor, colormapLut } from "./colormaps.ts";
import type { ColormapName } from "./colormaps.ts";
import { DASHES, dashFor, seriesToken } from "./series.ts";

const MAPS: readonly ColormapName[] = ["viridis", "magma"];

/** WCAG relative luminance of an 8-bit RGB triple. */
function luminance(r: number, g: number, b: number): number {
  const linear = (c: number) => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
}

void test("the colormaps are 256-entry rgb tables with their published end points", () => {
  for (const name of MAPS) {
    assert.equal(colormapLut(name).length, 256 * 3);
  }
  assert.equal(colormapColor("viridis", 0), "#440154");
  assert.equal(colormapColor("viridis", 1), "#fde725");
  assert.equal(colormapColor("magma", 0), "#000004");
  assert.equal(colormapColor("magma", 1), "#fcfdbf");
  // Out-of-range positions clamp.
  assert.equal(colormapColor("viridis", -1), "#440154");
  assert.equal(colormapColor("viridis", 2), "#fde725");
});

void test("dl_x08 the colormaps rise in luminance end to end", () => {
  // A sequential map must order values without hue: luminance rises, so it still reads for
  // CVD viewers and in greyscale. 8-bit rounding leaves dips of at most 3e-4 (viridis).
  for (const name of MAPS) {
    const lut = colormapLut(name);
    let previous = -1;
    for (let i = 0; i < 256; i += 1) {
      const y = luminance(
        lut[i * 3] ?? 0,
        lut[i * 3 + 1] ?? 0,
        lut[i * 3 + 2] ?? 0,
      );
      assert.ok(y >= previous - 5e-4, `${name} entry ${String(i)}`);
      previous = y;
    }
  }
});

void test("dl_x08 every series of the palette has its own dash pattern", () => {
  const patterns = DASHES.map((dash) => dash.join(","));
  assert.equal(new Set(patterns).size, 8);
  assert.deepEqual(dashFor(9), dashFor(1));
  assert.equal(seriesToken(0), "--series-1");
  assert.equal(seriesToken(8), "--series-1");
});
