// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// System locale names that Intl rejects (F-210; #436's Linux smoke run under LANG=C).
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  SOURCE_LOCALE,
  createLocalizer,
  normalizeLocale,
} from "../src/index.ts";

void test("posix C and POSIX locales fall back, with or without an encoding", () => {
  for (const tag of [
    "C",
    "C.UTF-8",
    "POSIX",
    "posix",
    "C@euro",
    "",
    null,
    undefined,
  ]) {
    assert.equal(
      normalizeLocale(tag, SOURCE_LOCALE),
      SOURCE_LOCALE,
      String(tag),
    );
  }
});

void test("posix locale names become bcp 47 tags", () => {
  assert.equal(normalizeLocale("en_US.UTF-8", SOURCE_LOCALE), "en-US");
  assert.equal(normalizeLocale("de_DE@euro", SOURCE_LOCALE), "de-DE");
  assert.equal(normalizeLocale("pt_BR", SOURCE_LOCALE), "pt-BR");
  assert.equal(normalizeLocale("fr", SOURCE_LOCALE), "fr");
});

void test("a tag intl rejects falls back instead of throwing", () => {
  assert.equal(normalizeLocale("not a tag!", SOURCE_LOCALE), SOURCE_LOCALE);
  assert.equal(normalizeLocale("x", SOURCE_LOCALE), SOURCE_LOCALE);
  // Every result is one Intl accepts.
  for (const tag of ["C", "en_US.UTF-8", "de_DE@euro", "not a tag!"]) {
    assert.doesNotThrow(
      () => new Intl.NumberFormat(normalizeLocale(tag, SOURCE_LOCALE)),
    );
  }
});

void test("a localizer made with a posix locale formats numbers", () => {
  const t = createLocalizer("C.UTF-8", ["n = { $n }"]);
  assert.equal(t.locale, SOURCE_LOCALE);
  assert.equal(t.format("n", { n: { number: 1234.5 } }), "1,234.5");
});
