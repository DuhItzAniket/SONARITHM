// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The negative test of the axe gate (B02-T03, testing-strategy §13.1): runs only the seeded
// stories and succeeds only if each failed because axe found its violation. A run that fails
// for another reason (no browser, a build error) or that passes fails this check.
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const out = join(mkdtempSync(join(tmpdir(), "snr-seeded-")), "report.json");
const vitest = join(
  dirname(createRequire(import.meta.url).resolve("vitest/package.json")),
  "vitest.mjs",
);
const run = spawnSync(
  process.execPath,
  [
    vitest,
    "run",
    "--project",
    "storybook",
    "--reporter=json",
    `--outputFile=${out}`,
  ],
  {
    env: { ...process.env, SNR_SEEDED: "1" },
    stdio: "inherit",
  },
);

interface Assertion {
  readonly fullName: string;
  readonly status: string;
  readonly failureMessages: readonly string[];
}
interface Report {
  readonly testResults: readonly {
    readonly assertionResults: readonly Assertion[];
  }[];
}

const report = JSON.parse(readFileSync(out, "utf8")) as Report;
const tests = report.testResults.flatMap((file) => file.assertionResults);
const problems: string[] = [];
if (run.status === 0) {
  problems.push("the seeded run passed");
}
if (tests.length === 0) {
  problems.push("no seeded story ran");
}
for (const test of tests) {
  const message = test.failureMessages.join("\n");
  if (test.status !== "failed" || !message.includes("button-name")) {
    problems.push(
      `${test.fullName}: ${test.status}, not an axe button-name failure`,
    );
  }
}
if (problems.length > 0) {
  console.error(
    `The axe gate did not catch the seeded violation:\n${problems.join("\n")}`,
  );
  process.exit(1);
}
console.log(
  `The axe gate caught every seeded violation (${String(tests.length)}).`,
);
