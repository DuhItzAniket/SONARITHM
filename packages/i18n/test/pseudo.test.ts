// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The pseudo-locale check in the UI package (B02-T08, NFR-09): every message renders as
// one marked message, and the check catches seeded concatenated, hard-coded and truncated
// strings, as in crates/sonarithm-i18n/tests/pseudo.rs.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  PSEUDO_LOCALE,
  SOURCE_LOCALE,
  checkPseudo,
  createLocalizer,
} from "../src/index.ts";
import type { MessageArgs } from "../src/index.ts";
import { catalogueSources, messageIds } from "./catalogue.ts";

const english = catalogueSources(SOURCE_LOCALE);
const pseudo = createLocalizer(PSEUDO_LOCALE, english);

function sampleArgs(style: string): MessageArgs {
  return {
    value: { number: 28.7, minimumFractionDigits: 1, maximumFractionDigits: 1 },
    style,
    count: { number: 3 },
    index: { number: 2 },
    total: { number: 5 },
    unit: "furlongs",
    // Data arguments have no letters, so the test sees only catalogue text.
    text: "12.5",
    first: "1",
    rest: "2",
    from: "3",
    to: "4",
    qualifier: "5",
    confidence: { number: 0.82 },
    key: "↓",
    kind: "point",
    detail: "7",
    class: "cv_estimate",
    axis: "x",
    included: "yes",
    edge: "last",
    reason: "one-series",
    hours: { number: 14 },
    minutes: { number: 2 },
    seconds: { number: 11 },
    date: "6",
    time: "7",
    field: "8",
    series: "9",
    range: "10",
    low: { number: 400 },
    high: { number: 1400 },
    polarity: "positive",
    mode: "analysis",
    ticks: "yes",
    reference: "yes",
    name: "11",
    state: "on",
    stream: "13",
    event: "14",
    condition: "15",
  };
}

test("nfr_09 every message passes the pseudo-locale check", () => {
  const ids = messageIds(english);
  assert.ok(ids.length > 0);
  for (const id of ids) {
    for (const style of ["symbol", "terse", "verbose"]) {
      const text = pseudo.format(id, sampleArgs(style));
      assert.equal(checkPseudo(text), null, `${id}: ${text}`);
      assert.doesNotMatch(text.replaceAll("furlongs", ""), /[A-Za-z]/, id);
    }
  }
});

test("nfr_09 pseudo check catches a seeded concatenated string", () => {
  const count = pseudo.format("common-point-count", { count: { number: 3 } });
  const position = pseudo.format("common-position", {
    index: { number: 2 },
    total: { number: 5 },
  });
  assert.equal(checkPseudo(`${count}, ${position}`), "concatenated");
  assert.equal(checkPseudo(`Series: ${count}`), "concatenated");
});

test("nfr_09 pseudo check catches hard-coded and truncated strings", () => {
  assert.equal(checkPseudo("Stop all audio"), "unmarked");
  const text = pseudo.format("ui-stop-all-audio");
  assert.equal(checkPseudo(Array.from(text).slice(0, 8).join("")), "truncated");
  assert.equal(checkPseudo("⟦a ⟦b~⟧~~⟧"), null);
});
