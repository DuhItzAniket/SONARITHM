// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// DL-X05 (B02-T09): no emoji in the UI. Every UI string lives in a Fluent catalogue, so a
// catalogue without pictographs means no emoji labels, icons or decoration. Screen readers
// read emoji aloud by their Unicode names, which is noise in a command name or announcement.
import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import { test } from "node:test";

import { catalogueSources } from "./catalogue.ts";

const LOCALES = new URL(
  "../../../crates/sonarithm-i18n/locales/",
  import.meta.url,
);

/** Pictographs, emoji presentation selectors and the keycap mark. */
const EMOJI = /\p{Extended_Pictographic}|\uFE0F|\u20E3/u;

test("dl_x05 emoji detector finds emoji", () => {
  for (const sample of ["\u{1F50A} Play", "Done \u2705", "1\uFE0F\u20E3"]) {
    assert.ok(EMOJI.test(sample), sample);
  }
  for (const sample of ["Play", "Ctrl+Shift+P", "−6 dB", "→", "×"]) {
    assert.ok(!EMOJI.test(sample), sample);
  }
});

test("dl_x05 no catalogue contains emoji", () => {
  const found: string[] = [];
  for (const locale of readdirSync(LOCALES)) {
    for (const source of catalogueSources(locale)) {
      for (const line of source.split("\n")) {
        if (!line.startsWith("#") && EMOJI.test(line)) {
          found.push(`${locale}: ${line.trim()}`);
        }
      }
    }
  }
  assert.deepEqual(found, []);
});
