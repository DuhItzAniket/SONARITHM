// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { defineConfig, devices } from "@playwright/test";

// Cross-engine E2E with the in-memory host (B02-T03; testing-strategy §12): the real UI in
// Chromium, WebKit and Firefox, served by Vite's dev server. CI runs it in the pinned
// Playwright image; locally, `pnpm -C apps/desktop test:e2e --project chromium`.
//
// Each run starts its own dev server and never attaches to one already listening: several
// worktrees share one machine, and another worktree's server would serve its own code and
// stop when its run ends. A taken port fails the start instead (`--strictPort`); to run two
// worktrees at once, give one another port with SONARITHM_E2E_PORT.
const ci = process.env.CI !== undefined;
const PORT = Number(process.env.SONARITHM_E2E_PORT ?? 1430);
if (!Number.isInteger(PORT) || PORT < 1024 || PORT > 65535) {
  throw new Error(
    "SONARITHM_E2E_PORT must be a whole number from 1024 to 65535",
  );
}

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
    reuseExistingServer: false,
  },
});
