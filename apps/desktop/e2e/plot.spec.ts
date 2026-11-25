// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Zoom and pan without dragging (B04-T07; A11Y-26, WCAG 2.5.7, 2.5.8): the toolbar buttons,
// the keyboard commands, a drag that zooms without moving the cursor, and target sizes.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

interface Mock {
  viewRanges(): ({ start: number; end: number } | null)[];
  commands(): string[];
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);

/** The platform's Primary modifier as the app sees it (KB-07; WebKit reports a Mac). */
async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

const toolbar = (page: Page) =>
  page.getByRole("toolbar", { name: "Plot view" });
const range = (page: Page) => page.locator("[data-plot-range]");
const lastRange = async (page: Page) =>
  (await host(page)).evaluate((m) => m.viewRanges().at(-1) ?? null);

test.beforeEach(async ({ page }) => {
  // 1,000 rows: x (time) runs from 0 to 99.9 s in the mock host.
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 1000 });
  });
  await page.goto("/e2e.html");
  await page.getByRole("tab", { name: "Plot" }).click();
  await expect(page.getByRole("img", { name: /temperature/ })).toBeVisible();
  await expect(range(page)).toHaveText("Showing time (s) from 0 to 99.9");
});

test("a11y_26_the_toolbar_zooms_and_pans_without_a_drag", async ({ page }) => {
  const button = (name: string) => toolbar(page).getByRole("button", { name });
  await expect(button("Zoom out")).toBeDisabled();
  await button("Zoom in").focus();
  await page.keyboard.press("Enter");
  // Around the middle (no cursor yet): half the range.
  await expect(range(page)).toHaveText(
    "Showing time (s) from 24.975 to 74.925",
  );
  const requested = await lastRange(page);
  expect(requested?.start).toBeCloseTo(24.975, 6);
  expect(requested?.end).toBeCloseTo(74.925, 6);
  await expect(
    page.locator('[data-announcer] [aria-live="polite"]'),
  ).toHaveText("Showing time (s) from 24.975 to 74.925");
  // Pan right a quarter of the view, then reset to all of it.
  await button("Pan right").click();
  await expect(range(page)).toHaveText(
    "Showing time (s) from 37.463 to 87.413",
  );
  await button("Reset zoom").click();
  await expect(range(page)).toHaveText("Showing time (s) from 0 to 99.9");
  await expect.poll(() => lastRange(page)).toBeNull();
  expect(await seriousViolations(page)).toEqual([]);
});

test("a11y_26_the_zoom_commands_work_from_the_keyboard", async ({ page }) => {
  const key = await primary(page);
  await toolbar(page).getByRole("button", { name: "Zoom in" }).focus();
  await page.keyboard.press(`${key}+Equal`);
  await expect(range(page)).toHaveText(
    "Showing time (s) from 24.975 to 74.925",
  );
  await page.keyboard.press(`${key}+Minus`);
  await expect(range(page)).toHaveText("Showing time (s) from 0 to 99.9");
  await page.keyboard.press(`${key}+Equal`);
  await page.keyboard.press(`${key}+Digit0`);
  await expect(range(page)).toHaveText("Showing time (s) from 0 to 99.9");
});

test("a11y_26_a_drag_zooms_and_does_not_move_the_cursor", async ({ page }) => {
  const over = page.locator(".u-over");
  const box = await over.boundingBox();
  if (box === null) throw new Error("no plotting area");
  const y = box.y + box.height / 2;
  await page.mouse.move(box.x + box.width * 0.25, y);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.5, y, { steps: 5 });
  await page.mouse.up();
  await expect(range(page)).not.toHaveText("Showing time (s) from 0 to 99.9");
  const zoomed = await lastRange(page);
  expect(zoomed).not.toBeNull();
  expect(zoomed?.start).toBeGreaterThan(20);
  expect(zoomed?.end).toBeLessThan(55);
  // A drag is not a click: the cursor did not move.
  expect(
    await (
      await host(page)
    ).evaluate((m) => m.commands().includes("cursor_set")),
  ).toBe(false);
});

test("a11y_26_wcag_2_5_8_every_target_is_at_least_24_px", async ({ page }) => {
  const buttons = toolbar(page).getByRole("button");
  const count = await buttons.count();
  expect(count).toBe(5);
  for (let i = 0; i < count; i += 1) {
    const box = await buttons.nth(i).boundingBox();
    expect(box?.width ?? 0).toBeGreaterThanOrEqual(24);
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(24);
  }
});
