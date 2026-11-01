// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The one cursor in the table and the readout (B04-T06; A11Y-10, A11Y-11, T-10): the focused
// row is the cursor, the readout and breadcrumb follow it, a cursor moved elsewhere selects
// its row without moving the focus, and the readout never speaks.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

interface Mock {
  moveCursor(row: number, origin: "table" | "plot" | "nav"): unknown;
  cursor(): { row: number | null; origin: string | null };
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);

const grid = (page: Page) =>
  page.getByRole("grid", { name: "Rows of Readings" });
const panel = (page: Page) => page.getByRole("tabpanel", { name: "Table" });
const readout = (page: Page) => page.locator("[data-cursor-readout]");

/** The focused element's row header, or null outside the grid. */
const focusedRow = (page: Page) =>
  page.evaluate(
    () =>
      document.activeElement
        ?.closest('[role="row"]')
        ?.querySelector('[role="rowheader"]')?.textContent ?? null,
  );

/** The row headers of the selected rows. */
const selectedRows = (page: Page) =>
  page.evaluate(() =>
    [...document.querySelectorAll('[role="row"][aria-selected="true"]')].map(
      (row) => row.querySelector('[role="rowheader"]')?.textContent ?? "",
    ),
  );

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 1_000_000 });
  });
  await page.goto("/e2e.html");
  await page.getByRole("tab", { name: "Table" }).click();
  await expect(grid(page)).toBeVisible();
});

test("b04_t06_t_10_the_focused_row_is_the_cursor_and_the_readout_follows", async ({
  page,
}) => {
  await grid(page).getByRole("rowheader").first().focus();
  await expect(readout(page)).toContainText("Data row 1");
  await page.keyboard.press("ArrowDown");
  await expect(readout(page)).toContainText("Data row 2");
  await expect.poll(() => selectedRows(page)).toEqual(["2"]);
  // The cursor follows the focus; the focus never follows the cursor.
  expect(await focusedRow(page)).toBe("2");
  const trail = page.getByRole("navigation", { name: "Cursor position" });
  await expect(trail.locator('[aria-current="page"]')).toHaveText(
    "Dataset Readings",
  );
  // The readout is not a live region: speech is the announcement's job (A11Y-17).
  expect(
    await readout(page).evaluate(
      (element) => element.closest("[aria-live]") === null,
    ),
  ).toBe(true);
  await expect(
    page.locator('[data-announcer] [aria-live="polite"]'),
  ).not.toContainText("Data row");
  expect(await seriousViolations(page)).toEqual([]);
});

test("b04_t06_a_cursor_moved_elsewhere_selects_its_row_and_never_moves_focus", async ({
  page,
}) => {
  await grid(page).getByRole("rowheader").first().focus();
  await expect(readout(page)).toContainText("Data row 1");
  // The plot moves the cursor to row 6: the table selects it, the focus stays on row 1.
  await (await host(page)).evaluate((m) => m.moveCursor(5, "plot"));
  await expect.poll(() => selectedRows(page)).toEqual(["6"]);
  await expect(readout(page)).toContainText("Data row 6");
  expect(await focusedRow(page)).toBe("1");

  // A row in another window is not loaded under the focus…
  await (await host(page)).evaluate((m) => m.moveCursor(5_000, "plot"));
  await expect(readout(page)).toContainText("Data row 5,001");
  await expect(
    panel(page).getByText("Rows 1 to 1,000 of 1,000,000", { exact: true }),
  ).toBeVisible();
  expect(await focusedRow(page)).toBe("1");
  // …but once the focus has left the table, its window comes in, selected.
  await page.getByRole("tab", { name: "Table" }).focus();
  await expect(
    panel(page).getByText("Rows 5,001 to 6,000 of 1,000,000", { exact: true }),
  ).toBeVisible();
  await expect.poll(() => selectedRows(page)).toEqual(["5,001"]);
  await expect(page.getByRole("tab", { name: "Table" })).toBeFocused();
});

/** The readout's row and the crosshair's x, read in one evaluation: the same frame. */
const shown = (page: Page) =>
  page.evaluate(() => {
    const line = document.querySelector<HTMLElement>("[data-plot-cursor]");
    return {
      readout:
        document.querySelector("[data-cursor-readout] p")?.textContent ?? null,
      crosshair:
        line === null || line.hidden ? null : (line.dataset.cursorX ?? null),
    };
  });

test("b04_t06_t_10_the_table_and_the_plot_show_the_same_cursor", async ({
  page,
}) => {
  // Table to plot: the focused row's x is where the plot's crosshair stands.
  await grid(page).getByRole("rowheader").first().focus();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  await expect(readout(page)).toContainText("Data row 3");
  await page.getByRole("tab", { name: "Plot" }).click();
  const plot = page.getByRole("img", { name: /temperature/ });
  await expect(plot).toBeVisible();
  await expect
    .poll(() => shown(page))
    .toEqual({ readout: "Data row 3", crosshair: "0.2" });

  // Plot to table: a click moves the cursor (origin plot) to the nearest drawn row.
  const over = page.locator(".u-over");
  const box = await over.boundingBox();
  if (box === null) throw new Error("no plotting area");
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
  await expect
    .poll(
      async () => (await (await host(page)).evaluate((m) => m.cursor())).origin,
    )
    .toBe("plot");
  const row = (await (await host(page)).evaluate((m) => m.cursor())).row ?? 0;
  const number = (row + 1).toLocaleString("en-US");
  // Both views follow the one cursor event in the same frame.
  await expect
    .poll(() => shown(page))
    .toEqual({ readout: `Data row ${number}`, crosshair: String(row / 10) });
  await page.getByRole("tab", { name: "Table" }).click();
  await expect.poll(() => selectedRows(page)).toEqual([number]);
  expect(await seriousViolations(page)).toEqual([]);
});
