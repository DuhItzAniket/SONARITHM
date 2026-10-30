// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// `pnpm test:stories`: the story tests once per theme (`storybook-<theme>` projects in
// vitest.config.ts), one theme after another. Storybook's Vitest plugin shares one dependency
// cache between projects, and parallel projects race on it on Windows (EPERM on rename);
// sequential runs also keep one headless browser alive at a time. Extra arguments go to
// Vitest. Fails if any theme fails.
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";

import { THEMES } from "../src/foundations/themes.ts";

const vitest = join(
  dirname(createRequire(import.meta.url).resolve("vitest/package.json")),
  "vitest.mjs",
);

const failed = THEMES.filter((theme) => {
  const run = spawnSync(
    process.execPath,
    [
      vitest,
      "run",
      "--project",
      `storybook-${theme}`,
      ...process.argv.slice(2),
    ],
    { stdio: "inherit" },
  );
  return run.status !== 0;
});

if (failed.length > 0) {
  console.error(`Story tests failed in: ${failed.join(", ")}`);
  process.exit(1);
}
