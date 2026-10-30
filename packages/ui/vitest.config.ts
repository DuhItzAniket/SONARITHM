// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

import { THEMES } from "./src/foundations/themes.ts";

// Browser tests (B02-T03; testing-strategy §12). `storybook-<theme>`: every story renders and
// passes axe in Chromium, once per theme (the seeded stories run only with SNR_SEEDED=1, for the negative test).
// `visual`: screenshot baselines, taken only in CI's pinned Linux Chromium image so font
// rendering matches (`.github/workflows/ci.yml`, job `frontend-browser`). The Node unit tests
// stay on `node --test`.
const seeded = process.env.SNR_SEEDED === "1";

/** A fresh browser config per project: Vitest names (mutates) the instances it is given. */
function browser() {
  return {
    enabled: true,
    headless: true,
    provider: playwright(),
    instances: [{ browser: "chromium" as const }],
  };
}

export default defineConfig({
  test: {
    projects: [
      // One project per theme, so every story passes axe in all four (B02-T04).
      ...THEMES.map((theme) => ({
        extends: true as const,
        define: {
          __SNR_THEME__: JSON.stringify(theme),
          // React Stately's Virtualizer reads this when NODE_ENV is "test"; it keeps real
          // virtualization on, as in the app.
          "process.env.VIRT_ON": JSON.stringify("1"),
        },
        plugins: [
          storybookTest({
            configDir: ".storybook",
            tags: seeded ? { include: ["seeded"] } : { exclude: ["seeded"] },
          }),
        ],
        test: { name: `storybook-${theme}`, browser: browser() },
      })),
      {
        extends: true,
        plugins: [react()],
        test: {
          name: "visual",
          include: ["test/visual/**/*.test.tsx"],
          browser: { ...browser(), viewport: { width: 1024, height: 768 } },
        },
      },
    ],
  },
});
