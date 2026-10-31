// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The data table over a million rows (B04-T05; A11Y-03, A11Y-24, keyboard §5.6): headers in
// the accessibility tree, the true row number as the row header, paging across a window
// edge, whole-table jumps, "Go to row…" and sorting, each said in words.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

const TOTAL = 1_000_000;

/** The platform's Primary modifier as the app sees it (KB-07; WebKit reports a Mac). */
async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

const grid = (page: Page) =>
  page.getByRole("grid", { name: "Rows of Readings" });

const panel = (page: Page) => page.getByRole("tabpanel", { name: "Table" });

const polite = (page: Page) =>
  page.locator('[data-announcer] [aria-live="polite"]');

/** The focused cell's row header and column index. */
const focusedCell = (page: Page) =>
  page.evaluate(() => {
    const active = document.activeElement;
    const row = active?.closest('[role="row"]');
    return {
      row: row?.querySelector('[role="rowheader"]')?.textContent ?? null,
      column:
        active?.closest("[aria-colindex]")?.getAttribute("aria-colindex") ??
        null,
    };
  });

test.beforeEach(async ({ page }) => {
  await page.addInitScript((rows) => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows });
  }, TOTAL);
  await page.goto("/e2e.html");
  await page.getByRole("tab", { name: "Table" }).click();
  await expect(grid(page)).toBeVisible();
});

test("a11y_03_the_table_has_its_headers_the_row_number_and_provenance", async ({
  page,
}) => {
  await expect(
    grid(page).getByRole("columnheader").allTextContents(),
  ).resolves.toEqual(["Row", "time (s)", "temperature (°C)", "Provenance"]);
  await expect(grid(page).getByRole("rowheader").first()).toHaveText("1");
  await expect(
    panel(page).getByText("Rows 1 to 1,000 of 1,000,000", { exact: true }),
  ).toBeVisible();
  // Only the visible rows are in the DOM.
  expect(await grid(page).getByRole("row").count()).toBeLessThan(100);
  // Row 10 is estimated (mock-host.ts), in the words speech uses (A11Y-24).
  const tenth = grid(page)
    .getByRole("row")
    .filter({
      has: page.getByRole("rowheader", { name: "10", exact: true }),
    });
  await expect(tenth).toContainText(
    "estimated from a chart image, confidence 0.82",
  );
  await expect(grid(page).getByRole("row").nth(1)).toContainText("exact");
  expect(await seriousViolations(page)).toEqual([]);
});

test("b04_t05_go_to_row_then_page_across_a_window_edge", async ({ page }) => {
  await grid(page).getByRole("rowheader").first().focus();
  await page.keyboard.press(`${await primary(page)}+Shift+P`);
  await page.keyboard.type("Go to row");
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog", { name: "Go to row" });
  await expect(dialog).toBeVisible();
  const field = dialog.getByRole("textbox", { name: "Row number" });
  await field.fill("1000");
  await page.keyboard.press("Enter");
  await expect(dialog).toBeHidden();
  await expect
    .poll(() => focusedCell(page))
    .toEqual({ row: "1,000", column: "1" });

  // Down from the window's last row loads the next window, in the same column.
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowDown");
  await expect
    .poll(() => focusedCell(page))
    .toEqual({ row: "1,001", column: "2" });
  await expect(
    panel(page).getByText("Rows 1,001 to 2,000 of 1,000,000", { exact: true }),
  ).toBeVisible();
  await expect(polite(page)).toHaveText("Rows 1,001 to 2,000 of 1,000,000");

  // And back up.
  await page.keyboard.press("ArrowUp");
  await expect
    .poll(() => focusedCell(page))
    .toEqual({ row: "1,000", column: "2" });
  await expect(polite(page)).toHaveText("Rows 1 to 1,000 of 1,000,000");
});

test("b04_t05_primary_end_and_home_reach_the_whole_table", async ({ page }) => {
  await grid(page).getByRole("rowheader").first().focus();
  const key = await primary(page);
  await page.keyboard.press(`${key}+End`);
  await expect
    .poll(() => focusedCell(page))
    .toEqual({
      row: "1,000,000",
      column: "4",
    });
  await expect(polite(page)).toHaveText(
    "Rows 999,001 to 1,000,000 of 1,000,000",
  );
  await page.keyboard.press(`${key}+Home`);
  await expect.poll(() => focusedCell(page)).toEqual({ row: "1", column: "1" });
  await expect(polite(page)).toHaveText("Rows 1 to 1,000 of 1,000,000");
});

test("b04_t05_enter_on_a_header_sorts_and_says_so", async ({ page }) => {
  const header = grid(page).getByRole("columnheader", { name: "time (s)" });
  await header.focus();
  await page.keyboard.press("Enter");
  await expect(header).toHaveAttribute("aria-sort", "ascending");
  await expect(polite(page)).toHaveText("Sorted by time (s), ascending");
  await page.keyboard.press("Enter");
  await expect(header).toHaveAttribute("aria-sort", "descending");
  await expect(polite(page)).toHaveText("Sorted by time (s), descending");
  // Sorting starts again at row 1; the mock reverses the order when descending.
  await expect(grid(page).getByRole("rowheader").first()).toHaveText("1");
  await expect(grid(page).getByRole("row").nth(1)).toContainText("99,999.9");
});
