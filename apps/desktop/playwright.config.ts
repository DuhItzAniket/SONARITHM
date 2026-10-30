// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { defineConfig, devices } from "@playwright/test";

// Cross-engine E2E with the in-memory host (B02-T03; testing-strategy §12): the real UI in
// Chromium, WebKit and Firefox, served by Vite's dev server. CI runs it in the pinned
// Playwright image; locally, `pnpm -C apps/desktop test:e2e --project chromium`.
const PORT = 1430;
const ci = process.env.CI !== undefined;

export default defineConfig({
  testDir: "e2e",
  forbidOnly: ci,
  retries: 0,
  workers: 2,
  reporter: "list",
  use: { baseURL: `http://localhost:${String(PORT)}` },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
  ],
  webServer: {
    command: `pnpm exec vite --port ${String(PORT)} --strictPort`,
    port: PORT,
    reuseExistingServer: !ci,
  },
});
