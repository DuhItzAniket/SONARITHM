// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The hard-coded UI string lint (B02-T08, NFR-09): `sonarithm/no-literal-ui-string` fails
// on seeded hard-coded strings and passes catalogue lookups.
import { test } from "node:test";

import { RuleTester } from "eslint";

import rule from "../eslint/no-literal-ui-string.js";

const tester = new RuleTester({
  languageOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    parserOptions: { ecmaFeatures: { jsx: true } },
  },
});

const literal = { messageId: "literal" };

test("nfr_09 the lint fails on a hard-coded UI string", () => {
  tester.run("no-literal-ui-string", rule, {
    valid: [
      'const a = <Button>{t.format("ui-stop-all-audio")}</Button>;',
      'const b = <IconButton aria-label={t.format("ui-command-palette")} />;',
      "const c = <span>{value} · {unit}</span>;",
      'const d = <div className="toolbar" data-testid="transport" />;',
      "const e = <span>  </span>;",
      'const f = <span>{"–"}</span>;',
    ],
    invalid: [
      { code: "const a = <Button>Stop all audio</Button>;", errors: [literal] },
      {
        code: 'const b = <IconButton aria-label="Play" />;',
        errors: [literal],
      },
      { code: 'const c = <span>{"Paused"}</span>;', errors: [literal] },
      {
        code: "const d = <input placeholder={`Find value`} />;",
        errors: [literal],
      },
      {
        code: 'const e = <img alt={"Waveform"} title="Plot" />;',
        errors: [literal, literal],
      },
    ],
  });
});
