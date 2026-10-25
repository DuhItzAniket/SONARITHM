// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The CSS lint (B02-T01, coding-standards §12.6): seeded design-language anti-patterns fail,
// token-based CSS passes, and only the generated token file is exempt.
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import stylelint from "stylelint";

import config from "../stylelint/config.js";
import { CSS_FILE } from "../tokens/generate.ts";

/** The rules `code` breaks, sorted and without duplicates. */
async function broken(code: string): Promise<string[]> {
  const result = await stylelint.lint({
    code,
    codeFilename: fileURLToPath(
      new URL("../src/components/sample.module.css", import.meta.url),
    ),
    config,
  });
  const rules = (result.results[0]?.warnings ?? []).map((w) => w.rule);
  return [...new Set(rules)].sort();
}

test("dl_x01 gradients, glow and gradient text are rejected", async () => {
  assert.deepEqual(
    await broken(
      ".a { background: linear-gradient(var(--accent), var(--focus)); }",
    ),
    ["function-disallowed-list"],
  );
  assert.deepEqual(
    await broken(
      ".a { background-image: radial-gradient(var(--accent), var(--surface-0)); }",
    ),
    ["function-disallowed-list"],
  );
  assert.deepEqual(await broken(".a { background-clip: text; }"), [
    "declaration-property-value-disallowed-list",
  ]);
  assert.deepEqual(await broken(".a { box-shadow: 0 0 12px var(--accent); }"), [
    "declaration-property-value-allowed-list",
  ]);
  assert.deepEqual(await broken(".a { text-shadow: 0 0 4px var(--focus); }"), [
    "property-disallowed-list",
  ]);
  assert.deepEqual(
    await broken(".a { filter: drop-shadow(0 0 6px var(--accent)); }"),
    ["function-disallowed-list"],
  );
});

test("dl_x02 frosted and blurred panels are rejected", async () => {
  assert.deepEqual(await broken(".a { backdrop-filter: blur(12px); }"), [
    "function-disallowed-list",
    "property-disallowed-list",
  ]);
  assert.deepEqual(
    await broken(".a { -webkit-backdrop-filter: saturate(2); }"),
    ["property-disallowed-list"],
  );
  assert.deepEqual(
    await broken(
      ".a { background: color-mix(in oklch, var(--surface-1), transparent 40%); }",
    ),
    ["function-disallowed-list"],
  );
});

test("dl_x03 large radii and soft shadows are rejected", async () => {
  assert.deepEqual(await broken(".a { border-radius: 12px; }"), [
    "declaration-property-value-allowed-list",
  ]);
  assert.deepEqual(
    await broken(".a { border-top-left-radius: var(--radius-3); }"),
    ["declaration-property-value-allowed-list"],
  );
  assert.deepEqual(
    await broken(".a { box-shadow: 0 8px 24px var(--shadow-colour); }"),
    ["declaration-property-value-allowed-list"],
  );
});

test("dl_p07 colour literals are rejected", async () => {
  assert.deepEqual(await broken(".a { color: #ffffff; }"), ["color-no-hex"]);
  assert.deepEqual(await broken(".a { color: white; }"), ["color-named"]);
  assert.deepEqual(await broken(".a { color: oklch(0.9 0 0); }"), [
    "function-disallowed-list",
  ]);
  assert.deepEqual(await broken(".a { border-color: rgb(0 0 0); }"), [
    "function-disallowed-list",
  ]);
});

test("spacing and durations must come from tokens", async () => {
  assert.deepEqual(await broken(".a { padding: 8px; }"), [
    "declaration-property-unit-disallowed-list",
  ]);
  assert.deepEqual(await broken(".a { margin-inline-start: 0.5rem; }"), [
    "declaration-property-unit-disallowed-list",
  ]);
  assert.deepEqual(await broken(".a { transition: opacity 200ms ease-out; }"), [
    "declaration-property-unit-disallowed-list",
  ]);
});

test("token-based css passes", async () => {
  const css = `
.panel {
  background: var(--surface-1);
  color: var(--text-1);
  border-block-end: var(--border-width) solid var(--border);
  padding: var(--space-2) var(--space-3);
  gap: var(--space-1);
  border-radius: var(--radius-1);
}
.overlay {
  box-shadow: var(--shadow-overlay);
  border-radius: var(--radius-2) var(--radius-2) 0 0;
  transition: opacity var(--motion-fast) ease-out;
}
.icon {
  color: currentColor;
  background: transparent;
}
.control:focus-visible {
  outline: var(--focus-ring-width) solid var(--focus);
  outline-offset: var(--focus-ring-offset);
}
@media (forced-colors: active) {
  .control { border-color: ButtonBorder; }
}
`;
  assert.deepEqual(await broken(css), []);
});

test("only the generated token file is exempt", async () => {
  const generated = await stylelint.lint({
    files: fileURLToPath(CSS_FILE),
    config,
  });
  assert.equal(generated.results[0]?.ignored, true);
  const withoutIgnore = await stylelint.lint({
    files: fileURLToPath(CSS_FILE),
    config: { ...config, ignoreFiles: [] },
  });
  // The raw values there would fail the rules anywhere else.
  assert.ok((withoutIgnore.results[0]?.warnings.length ?? 0) > 0);
});
