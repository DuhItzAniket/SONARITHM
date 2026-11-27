// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The braille line (B03-T10; keyboard-and-navigation §10, §14; ADR-0017 "Braille"): B turns
// it on, it follows the one cursor, Shift+B switches to the sparkline, a routing press
// moves the cursor silently (origin braille), and Preferences holds the cell count.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

interface Mock {
  commands(): string[];
  cursor(): { row: number | null; origin: string | null };
  settings(): {
    modalities: Record<string, boolean>;
    braille: { mode: string; cells?: number | null };
  };
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);

const explorer = (page: Page) =>
  page.getByRole("application", { name: "Chart explorer, Readings" });

/** The field's caret, as a routing key or the cursor keys leave it. */
const caret = (page: Page, name: string) =>
  page
    .getByRole("textbox", { name })
    .evaluate((field: HTMLInputElement) => field.selectionStart);

async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 50 });
  });
  await page.goto("/e2e.html");
  await page.getByRole("tab", { name: "Plot" }).click();
  await expect(explorer(page)).toBeVisible();
});

test("b03_t10_b_shows_the_braille_line_and_it_follows_the_cursor", async ({
  page,
}) => {
  const line = page.getByRole("textbox", { name: "Braille line" });
  // Braille is off in the mock host: the field is there, empty and says so, and it is
  // not a Tab stop (#513 review).
  await expect(line).toHaveValue("");
  await expect(line).toHaveAttribute("aria-disabled", "true");
  await expect(line).toHaveAttribute("tabindex", "-1");
  await expect(line).toHaveAccessibleDescription(/Braille output is off/);
  await explorer(page).focus();
  await page.keyboard.press("b");
  await page.keyboard.press("ArrowRight");
  await expect(line).toHaveValue("15.1 °C day 2");
  await expect(line).toHaveAttribute("readonly", "");
  await expect(line).toHaveAccessibleDescription(/The cursor's value/);
  await page.keyboard.press("ArrowRight");
  await expect(line).toHaveValue("15.2 °C day 3");
  // The field never takes the focus; the explorer keeps it.
  await expect(explorer(page)).toBeFocused();
  expect(await seriousViolations(page)).toEqual([]);
});

test("b03_t10_shift_b_switches_to_the_sparkline_with_the_caret_on_the_point", async ({
  page,
}) => {
  await explorer(page).focus();
  await page.keyboard.press("b");
  await page.keyboard.press("End");
  await page.keyboard.press("Shift+B");
  await expect
    .poll(async () =>
      (await host(page)).evaluate((mock) => mock.settings().braille.mode),
    )
    .toBe("sparkline");
  const sparkline = page.getByRole("textbox", { name: "Braille sparkline" });
  await expect(sparkline).toHaveValue("⣀⣠⣤⣴⣶⣾⣿⣷⣦⣄");
  // Row 50 is index 49: its cell is 49 % 10 in the mock's line.
  await expect.poll(() => caret(page, "Braille sparkline")).toBe(9);
});

test("b03_t10_a_routing_press_moves_the_cursor_silently", async ({ page }) => {
  await explorer(page).focus();
  await page.keyboard.press("b");
  await page.keyboard.press("Shift+B");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  const sparkline = page.getByRole("textbox", { name: "Braille sparkline" });
  await expect(sparkline).toBeVisible();
  // A routing key: the screen reader sets the caret on the routed cell through the
  // accessibility API, which reaches the page as a selection change, not a key press.
  await sparkline.focus();
  await sparkline.evaluate((field: HTMLInputElement) => {
    field.setSelectionRange(0, 0);
  });
  await expect
    .poll(async () => (await host(page)).evaluate((mock) => mock.cursor()))
    .toEqual(expect.objectContaining({ row: 0, origin: "braille" }));
  const commands = await (await host(page)).evaluate((mock) => mock.commands());
  expect(commands).toContain("braille_route");
  // A routing move is not announced (OD-30): the explorer's speech commands were not used.
  expect(commands).not.toContain("nav_where");
});

test("b03_t10_preferences_hold_the_braille_cell_count", async ({ page }) => {
  await page.keyboard.press(`${await primary(page)}+Comma`);
  const dialog = page.getByRole("dialog", { name: "Preferences" });
  const cells = dialog.getByRole("textbox", { name: "Braille display cells" });
  await expect(cells).toHaveValue("");
  await cells.fill("40");
  await cells.press("Tab");
  await expect
    .poll(async () =>
      (await host(page)).evaluate((mock) => mock.settings().braille.cells),
    )
    .toBe(40);
  await cells.fill("");
  await cells.press("Tab");
  await expect
    .poll(async () =>
      (await host(page)).evaluate(
        (mock) => mock.settings().braille.cells ?? null,
      ),
    )
    .toBe(null);
});

const routes = async (page: Page) =>
  (await (await host(page)).evaluate((mock) => mock.commands())).filter(
    (command) => command === "braille_route",
  ).length;

test("b03_t10_arrow_keys_in_the_field_review_and_do_not_route", async ({
  page,
}) => {
  // Keyboard §14 (#513 review): arrows, Home and End move the caret to read the line; only
  // a caret change with no such key, a routing key, moves the cursor. TODO(verify) with a
  // real display.
  await explorer(page).focus();
  await page.keyboard.press("b");
  await page.keyboard.press("Shift+B");
  await page.keyboard.press("ArrowRight");
  const sparkline = page.getByRole("textbox", { name: "Braille sparkline" });
  await expect(sparkline).toBeVisible();
  await sparkline.focus();
  // Chromium moves the caret in a read-only field with these keys; WebKit (with a Mac's
  // keys here) may not move it at all, which sends nothing either.
  for (const key of ["ArrowLeft", "Home", "End", "ArrowLeft"]) {
    await page.keyboard.press(key);
  }
  // Past the review keys' 150 ms window, any route they caused would have been sent.
  await page.waitForTimeout(200);
  expect(await routes(page)).toBe(0);
  // A routing key still routes.
  await sparkline.evaluate((field: HTMLInputElement) => {
    field.setSelectionRange(0, 0);
  });
  await expect.poll(() => routes(page)).toBe(1);
});

test("b03_t10_turning_braille_off_from_the_field_keeps_the_focus", async ({
  page,
}) => {
  // #513 review: the palette returns the focus to the braille field it opened from, which
  // stays mounted when braille goes off, so the focus never falls to the page.
  await explorer(page).focus();
  await page.keyboard.press("b");
  await page.keyboard.press("ArrowRight");
  const line = page.getByRole("textbox", { name: "Braille line" });
  await expect(line).toHaveValue("15.1 °C day 2");
  await line.focus();
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  await page.keyboard.press(`${mac ? "Meta" : "Control"}+Shift+P`);
  const palette = page.getByRole("dialog", { name: "Command palette" });
  await expect(palette).toBeVisible();
  await page.keyboard.type("Braille on or off");
  await page.keyboard.press("Enter");
  await expect(palette).toBeHidden();
  await expect(line).toHaveValue("");
  await expect(line).toHaveAttribute("aria-disabled", "true");
  await expect
    .poll(() => page.evaluate(() => document.activeElement?.tagName))
    .not.toBe("BODY");
  await expect(line).toBeFocused();
});
