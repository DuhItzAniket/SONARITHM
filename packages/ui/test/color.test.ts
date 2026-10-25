// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The token generator's colour arithmetic (B02-T01): OKLab against published reference
// values, round trips, CSS Color 4 gamut mapping and the WCAG 2.2 contrast ratio.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  contrastRatio,
  formatOklch,
  gamutMap,
  inGamut,
  oklabToOklch,
  oklabToSrgb,
  oklchToOklab,
  oklchToSrgb,
  parseOklch,
  srgbToOklab,
  toHex,
} from "../tokens/color.ts";
import type { Oklch, Rgb } from "../tokens/color.ts";

function close(actual: number, expected: number, tolerance: number): void {
  assert.ok(
    Math.abs(actual - expected) <= tolerance,
    `${String(actual)} is not within ${String(tolerance)} of ${String(expected)}`,
  );
}

test("srgb primaries convert to their published oklch values", () => {
  // Reference values from Björn Ottosson's OKLab definition, as used in CSS Color 4.
  const cases: readonly [Rgb, Oklch][] = [
    [[1, 0, 0], { l: 0.62796, c: 0.25768, h: 29.2339 }],
    [[0, 1, 0], { l: 0.86644, c: 0.29483, h: 142.4953 }],
    [[0, 0, 1], { l: 0.45201, c: 0.31321, h: 264.052 }],
  ];
  for (const [rgb, expected] of cases) {
    const actual = oklabToOklch(srgbToOklab(rgb));
    close(actual.l, expected.l, 1e-4);
    close(actual.c, expected.c, 1e-4);
    close(actual.h, expected.h, 1e-3);
  }
  close(srgbToOklab([1, 1, 1]).l, 1, 1e-6);
  close(srgbToOklab([0, 0, 0]).l, 0, 1e-12);
});

test("srgb to oklab and back is lossless at 8-bit precision", () => {
  // The published matrices have ten digits, so the round trip is exact to about 1e-6,
  // far below one 8-bit step (0.0039).
  for (const r of [0, 0.02, 0.3, 0.71, 1]) {
    for (const g of [0, 0.5, 1]) {
      for (const b of [0, 0.04, 0.9]) {
        const back = oklabToSrgb(srgbToOklab([r, g, b]));
        close(back[0], r, 1e-5);
        close(back[1], g, 1e-5);
        close(back[2], b, 1e-5);
      }
    }
  }
  const lch: Oklch = { l: 0.6, c: 0.1, h: 200 };
  const round = oklabToOklch(oklchToOklab(lch));
  close(round.c, lch.c, 1e-12);
  close(round.h, lch.h, 1e-9);
});

test("gamut mapping keeps in-gamut colours and brings others inside", () => {
  const inside: Oklch = { l: 0.75, c: 0.095, h: 195 };
  assert.deepEqual(gamutMap(inside), oklchToSrgb(inside).map(Number));
  assert.deepEqual(gamutMap({ l: 1.2, c: 0.1, h: 30 }), [1, 1, 1]);
  assert.deepEqual(gamutMap({ l: 0, c: 0.1, h: 30 }), [0, 0, 0]);
  for (const hue of [0, 60, 120, 195, 262, 320]) {
    const wanted: Oklch = { l: 0.7, c: 0.37, h: hue };
    assert.equal(inGamut(oklchToSrgb(wanted)), false);
    const mapped = oklabToOklch(srgbToOklab(gamutMap(wanted)));
    // CSS Color 4: lightness is kept and the result is within a JND of the boundary.
    close(mapped.l, wanted.l, 0.02);
    assert.ok(mapped.c < wanted.c);
  }
});

test("hex output clips to srgb", () => {
  assert.equal(toHex([1, 0.5, 0]), "#FF8000");
  assert.equal(toHex([1.2, -0.1, 0.0]), "#FF0000");
});

test("wcag 2.2 contrast ratio", () => {
  close(contrastRatio("#000000", "#FFFFFF"), 21, 1e-9);
  close(contrastRatio("#FFFFFF", "#000000"), 21, 1e-9);
  close(contrastRatio("#777777", "#777777"), 1, 1e-12);
  // #767676 on white is the usual example of the lightest gray that passes 4.5:1.
  close(contrastRatio("#767676", "#FFFFFF"), 4.54, 0.005);
  assert.ok(contrastRatio("#777777", "#FFFFFF") < 4.5);
});

test("oklch values format and parse", () => {
  assert.equal(
    formatOklch({ l: 0.185, c: 0.004, h: 85 }),
    "oklch(0.185 0.004 85)",
  );
  assert.equal(formatOklch({ l: 0.14, c: 0, h: 85 }), "oklch(0.14 0 0)");
  assert.equal(
    formatOklch({ l: 0.456_321, c: 0.077_849, h: 195.4449 }),
    "oklch(0.4563 0.0778 195.44)",
  );
  assert.deepEqual(parseOklch("oklch(0.83 0.1092 195)"), {
    l: 0.83,
    c: 0.1092,
    h: 195,
  });
  assert.throws(() => parseOklch("#FFFFFF"));
});
