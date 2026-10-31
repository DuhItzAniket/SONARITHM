// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The categorical series colours (B04-T01; design-language §6, DL-X08, A11Y-25): Okabe–Ito
// with only lightness changed, on the plot backgrounds at 3:1 (4.5:1 in high contrast), and
// apart under every dichromacy; the recipe holds what the derivation gives.
import assert from "node:assert/strict";
import { test } from "node:test";

import { buildThemes } from "../tokens/build.ts";
import { contrastRatio, oklabToOklch, srgbToOklab } from "../tokens/color.ts";
import { parseHex } from "../tokens/cvd.ts";
import { RECIPE, SERIES } from "../tokens/recipe.ts";
import {
  OKABE_ITO,
  SERIES_SEPARATION,
  closestSeries,
  deriveAll,
  seriesThemes,
} from "../tokens/series.ts";

const themes = buildThemes();

const hueOf = (hex: string) => oklabToOklch(srgbToOklab(parseHex(hex)));

test("dl_x08 the series keep the okabe ito hues in their order", () => {
  // The chromatic seven (series-8 is a neutral). Only lightness is changed; the CSS gamut
  // mapping's final clip and 8-bit rounding move a dark hue a little (8.6° at most, light
  // vermillion), never past a neighbour (design-language §6: keep hue order).
  const order = (hexes: readonly string[]) =>
    hexes
      .map((hex, i) => ({ i, h: hueOf(hex).h }))
      .sort((a, b) => a.h - b.h)
      .map(({ i }) => i);
  const source = OKABE_ITO.slice(0, 7);
  for (const theme of themes) {
    const rendered = SERIES.slice(0, 7).map((token) => theme.colors[token].hex);
    assert.deepEqual(order(rendered), order(source), theme.id);
    rendered.forEach((hex, i) => {
      const drift = Math.abs(
        ((hueOf(hex).h - hueOf(source[i] ?? "").h + 540) % 360) - 180,
      );
      assert.ok(
        drift < 10,
        `${theme.id} series-${String(i + 1)}: ${drift.toFixed(1)}°`,
      );
    });
  }
});

test("a11y_25 the series meet 3 to 1 on the plot backgrounds and 4_5 in high contrast", () => {
  for (const theme of themes) {
    const minimum = theme.id.startsWith("high") ? 4.5 : 3;
    for (const token of SERIES) {
      for (const background of ["surface-0", "surface-1"] as const) {
        const ratio = contrastRatio(
          theme.colors[token].hex,
          theme.colors[background].hex,
        );
        assert.ok(
          ratio >= minimum,
          `${theme.id} ${token} on ${background}: ${ratio.toFixed(2)}:1`,
        );
      }
    }
  }
});

test("dl_x08 the series stay apart under every dichromacy", () => {
  for (const theme of themes) {
    const { distance, pair } = closestSeries(
      SERIES.map((token) => theme.colors[token].hex),
    );
    assert.ok(
      distance >= SERIES_SEPARATION,
      `${theme.id}: series-${String(pair[0] + 1)} and series-${String(pair[1] + 1)} are ${distance.toFixed(3)} apart`,
    );
  }
});

test("the recipe's series are what the derivation gives", () => {
  const derived = deriveAll(seriesThemes(themes));
  for (const [theme, colors] of Object.entries(derived)) {
    colors.forEach((color, i) => {
      const token = SERIES[i];
      assert.ok(token !== undefined);
      const written = RECIPE[token][theme as keyof typeof derived];
      assert.ok(
        Math.abs(written.l - color.l) < 0.0005,
        `${theme} ${token}: the recipe has L ${String(written.l)}, the derivation ${String(color.l)}; run node tokens/series-print.ts`,
      );
    });
  }
});
