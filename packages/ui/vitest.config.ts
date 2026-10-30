// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

// Browser tests (B02-T03; testing-strategy §12). `storybook`: every story renders and passes
// axe in Chromium (the seeded stories run only with SNR_SEEDED=1, for the negative test).
// `visual`: screenshot baselines, taken only in CI's pinned Linux Chromium image so font
// rendering matches (`.github/workflows/ci.yml`, job `frontend-browser`). The Node unit tests
// stay on `node --test`.
const seeded = process.env.SNR_SEEDED === "1";

const browser = {
  enabled: true,
  headless: true,
  provider: playwright(),
  instances: [{ browser: "chromium" as const }],
};

export default defineConfig({
  test: {
    projects: [
      {
        extends: true,
        plugins: [
          storybookTest({
            configDir: ".storybook",
            tags: seeded ? { include: ["seeded"] } : { exclude: ["seeded"] },
          }),
        ],
        test: { name: "storybook", browser },
      },
      {
        extends: true,
        plugins: [react()],
        test: {
          name: "visual",
          include: ["test/visual/**/*.test.tsx"],
          browser: { ...browser, viewport: { width: 1024, height: 768 } },
        },
      },
    ],
  },
});
