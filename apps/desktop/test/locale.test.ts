// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// navigator.language under LANG=C (F-210; #436's Linux smoke run).
import assert from "node:assert/strict";
import { test } from "node:test";

import { fixNavigatorLocale } from "../src/locale.ts";

const fake = (language: string, languages: string[]) => ({
  language,
  languages,
});

void test("a posix C locale becomes the app's fallback before intl reads it", () => {
  const nav = fake("C", ["C"]);
  assert.equal(fixNavigatorLocale(nav), true);
  assert.equal(nav.language, "en-US");
  assert.deepEqual([...nav.languages], ["en-US"]);
  assert.doesNotThrow(() => new Intl.NumberFormat(nav.language));
});

void test("posix names in the language list become tags", () => {
  const nav = fake("C.UTF-8", ["de_DE@euro", "POSIX", "en_US.UTF-8"]);
  fixNavigatorLocale(nav);
  assert.equal(nav.language, "en-US");
  assert.deepEqual([...nav.languages], ["en-US", "de-DE"]);
});

void test("a valid locale is left as it is", () => {
  const nav = fake("fr-FR", ["fr-FR", "en"]);
  assert.equal(fixNavigatorLocale(nav), false);
  assert.equal(nav.language, "fr-FR");
});
