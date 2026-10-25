// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * Writes `src/generated/tokens.css` and `src/generated/tokens.json` from the recipe
 * (`pnpm -C packages/ui tokens`). Refuses to write if any design-language §5.3 pair is below
 * its minimum. The package tests fail when the committed files differ from this output.
 */

import { writeFileSync } from "node:fs";

import { generate } from "./build.ts";

export const CSS_FILE = new URL("../src/generated/tokens.css", import.meta.url);
export const JSON_FILE = new URL(
  "../src/generated/tokens.json",
  import.meta.url,
);

if (import.meta.main) {
  const { css, json, failures } = generate();
  if (failures.length > 0) {
    for (const pair of failures) {
      console.error(
        `${pair.theme}: ${pair.foreground} on ${pair.background} is ${pair.ratio.toFixed(2)}:1, below ${String(pair.minimum)}:1`,
      );
    }
    process.exit(1);
  }
  writeFileSync(CSS_FILE, css);
  writeFileSync(JSON_FILE, json);
}
