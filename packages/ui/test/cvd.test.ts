// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The colour-vision deficiency pass of the B02 design review (B02-T09, A11Y-25, DL-X08):
// indicators keep 3:1 against every surface for protanopes, deuteranopes, tritanopes and
// achromats in every theme; signal colours are not assumed to stay apart, so the review
// requires text or an icon wherever a signal colour carries meaning.
import assert from "node:assert/strict";
import { test } from "node:test";

import { buildThemes } from "../tokens/build.ts";
import { contrastRatio } from "../tokens/color.ts";
import { DEFICIENCIES, deltaEOK, simulate } from "../tokens/cvd.ts";
import { SIGNALS, SURFACES } from "../tokens/recipe.ts";

const themes = buildThemes();

/** Tokens that show state or position on their own: they must stay visible (WCAG 1.4.11). */
const INDICATORS = ["focus", "accent", "border-strong"] as const;

test("cvd simulation leaves greys unchanged", () => {
  for (const deficiency of DEFICIENCIES) {
    for (const grey of ["#000000", "#808080", "#ffffff"]) {
      assert.ok(
        deltaEOK(simulate(grey, deficiency), grey) < 0.005,
        `${deficiency} ${grey}`,
      );
    }
  }
});

test("cvd simulation moves red and green together for deuteranopes", () => {
  const red = simulate("#d03030", "deuteranopia");
  const green = simulate("#30a030", "deuteranopia");
  assert.ok(
    deltaEOK(red, green) < deltaEOK("#d03030", "#30a030") / 2,
    `${red} ${green}`,
  );
});

test("a11y_25 indicators keep 3 to 1 under every cvd type", () => {
  const failures: string[] = [];
  for (const theme of themes) {
    for (const deficiency of DEFICIENCIES) {
      for (const indicator of INDICATORS) {
        for (const surface of SURFACES) {
          const ratio = contrastRatio(
            simulate(theme.colors[indicator].hex, deficiency),
            simulate(theme.colors[surface].hex, deficiency),
          );
          if (ratio < 3) {
            failures.push(
              `${theme.id} ${deficiency} ${indicator}/${surface} ${ratio.toFixed(2)}`,
            );
          }
        }
      }
    }
  }
  assert.deepEqual(failures, []);
});

test("dl_x08 some signal pairs collapse under cvd so signals need text", () => {
  // The review's premise, checked: if this ever stops holding, the DL-X08 rule still stands,
  // but the review record's CVD table is out of date and must be regenerated.
  let closest = Number.POSITIVE_INFINITY;
  for (const theme of themes) {
    for (const deficiency of DEFICIENCIES) {
      for (const [i, a] of SIGNALS.entries()) {
        for (const b of SIGNALS.slice(i + 1)) {
          closest = Math.min(
            closest,
            deltaEOK(
              simulate(theme.colors[a].hex, deficiency),
              simulate(theme.colors[b].hex, deficiency),
            ),
          );
        }
      }
    }
  }
  assert.ok(closest < 0.02, `closest signal pair ΔEOK ${closest.toFixed(3)}`);
});
