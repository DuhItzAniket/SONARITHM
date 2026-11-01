// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The command palette and the keyboard reference (B02-T07; KB-01, KB-09, A11Y-30).
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

const palette = (page: Page) =>
  page.getByRole("dialog", { name: "Command palette" });

/**
 * The platform's Primary modifier as the app sees it (KB-07): the app picks its bindings
 * from the user agent, and Playwright's WebKit project reports a Mac one, so Primary is
 * Cmd (Meta) there and Ctrl elsewhere, whatever OS runs the test.
 */
async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

async function openPalette(page: Page): Promise<void> {
  await page.keyboard.press(`${await primary(page)}+Shift+P`);
  await expect(palette(page)).toBeVisible();
}

/** Closes every open dialog with Escape. */
async function closeDialogs(page: Page): Promise<void> {
  for (let i = 0; i < 3 && (await page.getByRole("dialog").count()) > 0; i++) {
    // Escape closes the dialog that has the focus; React Aria moves it in after an
    // animation frame (F-176), so wait for that first.
    await expect
      .poll(() =>
        page.evaluate(
          () => document.activeElement?.closest('[role="dialog"]') != null,
        ),
      )
      .toBe(true);
    await page.keyboard.press("Escape");
  }
  await expect(page.getByRole("dialog")).toHaveCount(0);
}

test.beforeEach(async ({ page }) => {
  await page.goto("/e2e.html");
  await expect(
    page.getByRole("main").getByText("No dataset is open."),
  ).toBeVisible();
});

test("kb_01_every_registered_command_runs_from_the_palette", async ({
  page,
}) => {
  const ids = await page.evaluate(() =>
    (Reflect.get(window, "__SONARITHM_COMMANDS__") as () => string[])(),
  );
  expect(ids.length).toBeGreaterThan(0);
  for (const id of ids) {
    await openPalette(page);
    const input = palette(page).getByRole("searchbox");
    // Keyboard only: arrows move the active result until it is this command.
    let previous: string | null = null;
    for (let i = 0; i <= ids.length; i++) {
      await page.keyboard.press("ArrowDown");
      // The active result updates after the key press; wait for it to move.
      await expect
        .poll(() => input.getAttribute("aria-activedescendant"))
        .not.toBe(previous);
      previous = await input.getAttribute("aria-activedescendant");
      const key = await page
        .locator(`[id="${previous ?? ""}"]`)
        .getAttribute("data-key");
      if (key === id) break;
    }
    await page.keyboard.press("Enter");
    await expect
      .poll(() =>
        page.evaluate(() => document.documentElement.dataset.lastCommand),
      )
      .toBe(id);
    // These open a dialog of their own, just after the palette closes: the palette again,
    // the key references (help.explorer is the explorer's, B03-T09) and Preferences.
    if (
      [
        "app.palette",
        "help.keyboard",
        "help.explorer",
        "app.preferences",
      ].includes(id)
    ) {
      await expect(page.getByRole("dialog")).toHaveCount(1);
    }
    await closeDialogs(page);
    await page.evaluate(() => {
      delete document.documentElement.dataset.lastCommand;
    });
  }
});

test("a11y_30_palette_results_are_announced", async ({ page }) => {
  await openPalette(page);
  const polite = page.locator('[data-announcer] [aria-live="polite"]');
  await expect(polite).toHaveText(/^\d+ commands$/);
  await page.keyboard.type("region");
  await expect(polite).toHaveText("3 commands");
  await expect(palette(page).getByRole("menuitem")).toHaveCount(3);
  const input = palette(page).getByRole("searchbox");
  await expect(input).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await expect
    .poll(() => input.getAttribute("aria-activedescendant"))
    .not.toBeNull();
  const active = await input.getAttribute("aria-activedescendant");
  await expect(page.locator(`[id="${active ?? ""}"]`)).toHaveRole("menuitem");
  expect(await seriousViolations(page)).toEqual([]);
});

test("kb_09_each_result_shows_its_current_binding", async ({ page }) => {
  await openPalette(page);
  await page.keyboard.type("next region");
  await expect(palette(page).getByRole("menuitem")).toContainText("F6");
});

test("kb_09_shift_f1_lists_the_bindings_of_the_focused_scope", async ({
  page,
}) => {
  await page.keyboard.press("F6");
  await page.keyboard.press("Shift+F1");
  const reference = page.getByRole("dialog", { name: "Keyboard shortcuts" });
  await expect(reference).toBeVisible();
  await expect(reference).toContainText("Next region");
  await expect(reference).toContainText("Shift+F6");
  expect(await seriousViolations(page)).toEqual([]);
  await page.keyboard.press("Escape");
  await expect(reference).toBeHidden();
});
