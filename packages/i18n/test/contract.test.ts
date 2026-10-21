// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The TypeScript half of the Rust–TypeScript contract (B02-T08, NFR-09): every case in
// crates/sonarithm-i18n/tests/fixtures/contract.json renders to its expected text here, as
// it does in the crate's tests/contract.rs.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { PSEUDO_LOCALE, SOURCE_LOCALE, createLocalizer } from "../src/index.ts";
import type { MessageArgs } from "../src/index.ts";
import { catalogueSources } from "./catalogue.ts";

interface ContractCase {
  readonly locale: string;
  readonly id: string;
  readonly args: MessageArgs;
  readonly expected: string;
}

const fixture = JSON.parse(
  readFileSync(
    new URL(
      "../../../crates/sonarithm-i18n/tests/fixtures/contract.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { cases: ContractCase[] };

const english = catalogueSources(SOURCE_LOCALE);

test("nfr_09 contract cases render as expected in TypeScript", () => {
  assert.ok(fixture.cases.length >= 20);
  const failures: string[] = [];
  for (const { locale, id, args, expected } of fixture.cases) {
    // Only English has a catalogue so far; en-GB, en-IN and the pseudo-locale read it.
    const actual = createLocalizer(locale, english).format(id, args);
    if (actual !== expected) {
      failures.push(
        `${locale} ${id}: ${JSON.stringify(actual)} != ${JSON.stringify(expected)}`,
      );
    }
  }
  assert.deepEqual(failures, []);
});

test("the pseudo-locale is flagged", () => {
  assert.equal(createLocalizer(PSEUDO_LOCALE, english).pseudo, true);
  assert.equal(createLocalizer(SOURCE_LOCALE, english).pseudo, false);
});

test("errors are thrown, not rendered", () => {
  const localizer = createLocalizer(SOURCE_LOCALE, english);
  assert.throws(() => localizer.format("no-such-message"), /no message/);
  assert.throws(
    () => localizer.format("common-position", { index: { number: 1 } }),
    /formatting/,
  );
  assert.throws(
    () => localizer.format("common-number", { value: { number: Number.NaN } }),
    /not a finite number/,
  );
});
