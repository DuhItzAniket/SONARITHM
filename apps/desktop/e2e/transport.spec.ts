// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The transport toolbar (B01-T05; keyboard §5.3): one Tab stop with arrows inside (APG
// Toolbar), buttons that follow the app core's state, and nothing playable before a Score.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";
import { startOutsideRegions } from "./keyboard.ts";

const toolbar = (page: Page) =>
  page.getByRole("toolbar", { name: "Transport" });

async function loadScore(page: Page, durationS: number): Promise<void> {
  await page.evaluate((seconds) => {
    (
      Reflect.get(window, "__SONARITHM_MOCK__") as {
        loadScore(durationS: number): void;
      }
    ).loadScore(seconds);
  }, durationS);
}

test.beforeEach(async ({ page }) => {
  await page.goto("/e2e.html");
  await expect(
    page.getByRole("main").getByText("No dataset is open."),
  ).toBeVisible();
});

test("a11y_15_nothing_is_playable_before_a_score_is_loaded", async ({
  page,
}) => {
  const bar = toolbar(page);
  await expect(bar.getByRole("button", { name: "Play" })).toBeDisabled();
  await expect(bar.getByRole("button", { name: "Stop" })).toBeDisabled();
  await expect(bar.getByRole("button", { name: "Loop" })).toBeDisabled();
});

test("b01_t05_toolbar_arrows_move_inside_and_tab_leaves", async ({ page }) => {
  await loadScore(page, 4);
  const bar = toolbar(page);
  const play = bar.getByRole("button", { name: "Play" });
  await expect(play).toBeEnabled();

  // From no focus (start-up puts it in the project region, B01-T07), F6 reaches the top
  // region, whose first Tab stop is the toolbar.
  await startOutsideRegions(page);
  await page.keyboard.press("F6");
  await expect(play).toBeFocused();

  // Play: the button becomes Pause once the app core reports playing, and keeps focus.
  await page.keyboard.press("Enter");
  const pause = bar.getByRole("button", { name: "Pause" });
  await expect(pause).toBeFocused();

  // Arrows move inside the toolbar.
  await page.keyboard.press("ArrowRight");
  await expect(bar.getByRole("button", { name: "Stop" })).toBeFocused();
  await page.keyboard.press("ArrowRight");
  const loop = bar.getByRole("button", { name: "Loop" });
  await expect(loop).toBeFocused();
  await page.keyboard.press("Space");
  await expect(loop).toHaveAttribute("aria-pressed", "true");
  await page.keyboard.press("ArrowLeft");
  await expect(bar.getByRole("button", { name: "Stop" })).toBeFocused();

  // Tab leaves the toolbar instead of visiting each button.
  await page.keyboard.press("Tab");
  const inside = await bar.evaluate((element) =>
    element.contains(document.activeElement),
  );
  expect(inside).toBe(false);

  // Stop from the toolbar: the app core's state comes back as stopped.
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Enter");
  await expect(bar.getByRole("button", { name: "Play" })).toBeVisible();
  // The focus stays on Stop: no button is disabled under it.
  await expect(bar.getByRole("button", { name: "Stop" })).toBeFocused();

  expect(await seriousViolations(page)).toEqual([]);
});
