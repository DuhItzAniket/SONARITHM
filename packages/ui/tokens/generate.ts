// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * Writes the generated files of `@sonarithm/ui` (`pnpm -C packages/ui tokens`):
 * `src/generated/tokens.css` and `tokens.json` from the recipe, and `src/generated/fonts.css`,
 * `src/generated/licenses.json` and `font-test/index.html` from the font table. Refuses to
 * write if any design-language §5.3 pair is below its minimum. The package tests fail when
 * the committed files differ from this output.
 */

import { readFileSync, writeFileSync } from "node:fs";

import { generate } from "./build.ts";
import {
  LICENSE_FILE,
  renderFontTest,
  renderFontsCss,
  renderLicenses,
} from "./fonts.ts";

export const CSS_FILE = new URL("../src/generated/tokens.css", import.meta.url);
export const JSON_FILE = new URL(
  "../src/generated/tokens.json",
  import.meta.url,
);
export const FONTS_CSS_FILE = new URL(
  "../src/generated/fonts.css",
  import.meta.url,
);
export const LICENSES_FILE = new URL(
  "../src/generated/licenses.json",
  import.meta.url,
);
export const FONT_TEST_FILE = new URL(
  "../font-test/index.html",
  import.meta.url,
);
export const FONT_DIR = new URL("../fonts/", import.meta.url);

/** Every generated file and its expected contents. */
export function generatedFiles(): readonly (readonly [URL, string])[] {
  const { css, json } = generate();
  const license = readFileSync(new URL(LICENSE_FILE.file, FONT_DIR), "utf8");
  return [
    [CSS_FILE, css],
    [JSON_FILE, json],
    [FONTS_CSS_FILE, renderFontsCss()],
    [LICENSES_FILE, renderLicenses(license)],
    [FONT_TEST_FILE, renderFontTest()],
  ];
}

if (import.meta.main) {
  const { failures } = generate();
  if (failures.length > 0) {
    for (const pair of failures) {
      console.error(
        `${pair.theme}: ${pair.foreground} on ${pair.background} is ${pair.ratio.toFixed(2)}:1, below ${String(pair.minimum)}:1`,
      );
    }
    process.exit(1);
  }
  for (const [file, contents] of generatedFiles()) {
    writeFileSync(file, contents);
  }
}
