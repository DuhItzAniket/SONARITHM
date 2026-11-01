// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The app under a POSIX "C" locale (F-210): WebKitGTK reports the process locale as
// navigator.language, and uPlot formats with it when its module loads. The page must start
// and the plot must draw anyway.
import { expect, test } from "@playwright/test";

test("posix_c_locale_the_app_starts_and_the_plot_draws", async ({ page }) => {
  await page.addInitScript(() => {
    // Configurable, as the real getter on Navigator.prototype is.
    Object.defineProperty(navigator, "language", {
      configurable: true,
      get: () => "C",
    });
    Object.defineProperty(navigator, "languages", {
      configurable: true,
      get: () => ["C"],
    });
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 1000 });
  });
  const errors: string[] = [];
  page.on("pageerror", (error) => {
    errors.push(error.message);
  });
  await page.goto("/e2e.html");
  await page.getByRole("tab", { name: "Plot" }).click();
  await expect(page.getByRole("img", { name: /temperature/ })).toBeVisible();
  expect(await page.evaluate(() => navigator.language)).toBe("en-US");
  expect(errors).toEqual([]);
});
