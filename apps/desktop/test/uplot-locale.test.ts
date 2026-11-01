// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// F-210: uPlot 1.6.32 builds `new Intl.NumberFormat(navigator.language)` when its module
// loads (uPlot.esm.js:442). With navigator.language "C", as WebKitGTK reports under LANG=C,
// loading uPlot throws; loading the desktop's locale shim first prevents it.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

/** uPlot's ES module, loaded afresh: a query string gives a new module instance. */
async function loadUplot(tag: string): Promise<unknown> {
  // uPlot is @sonarithm/viz's dependency, so it resolves from that package.
  const require = createRequire(
    new URL("../../../packages/viz/package.json", import.meta.url),
  );
  const file = require.resolve("uplot/dist/uPlot.esm.js");
  return import(`${pathToFileURL(file).href}?${tag}`);
}

void test("f_210 uplot loads under a posix C locale once the shim has run", async () => {
  // Just enough of a browser for uPlot's module scope: it reads window, document and
  // navigator when a window exists.
  const scope = globalThis as Record<string, unknown>;
  scope.window = globalThis;
  scope.document = {};
  scope.devicePixelRatio = 1;
  scope.matchMedia = () => ({
    matches: false,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  });
  scope.dispatchEvent = () => true;
  scope.addEventListener = () => undefined;
  scope.removeEventListener = () => undefined;
  Object.defineProperty(navigator, "language", {
    configurable: true,
    get: () => "C",
  });
  Object.defineProperty(navigator, "languages", {
    configurable: true,
    get: () => ["C"],
  });

  // Without the shim: the crash CI's Linux smoke run reported.
  await assert.rejects(() => loadUplot("without-shim"), RangeError);

  // With the shim imported first, as main.tsx does.
  await import("../src/locale.ts");
  assert.equal(navigator.language, "en-US");
  await assert.doesNotReject(() => loadUplot("with-shim"));
});
