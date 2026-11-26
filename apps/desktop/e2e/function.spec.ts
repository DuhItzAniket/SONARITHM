// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Functions in the desktop app (C03-T08b; ADR-0017 "Functions"): the New function dialog
// with live expression checks, the critical-point list on Rust's one cursor (A11Y-10,
// A11Y-24) and the audification calculator, whose all-inaudible readout is "will not
// compile" (SDR-31). Keyboard only; axe on every state.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

interface Mock {
  commands(): string[];
  cursorSets(): string[];
  moveCursor(row: number, origin: string, dataset: string): void;
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);

const dialog = (page: Page) =>
  page.getByRole("dialog", { name: "New function" });

const expression = (page: Page) =>
  dialog(page).getByRole("textbox", { name: "Expression in x" });

const points = (page: Page) =>
  page.getByRole("grid", { name: "Critical points" });

async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

/** Runs "New function…" from the command palette, keyboard only. */
async function newFunction(page: Page): Promise<void> {
  await page.keyboard.press(`${await primary(page)}+Shift+P`);
  const palette = page.getByRole("dialog", { name: "Command palette" });
  await expect(palette).toBeVisible();
  await page.keyboard.type("New function");
  await page.keyboard.press("Enter");
  await expect(dialog(page)).toBeVisible();
  // React Aria moves the focus in after an animation frame (F-176).
  await expect(expression(page)).toBeFocused();
}

/** Opens `x^3 - 3*x` on [-2, 2] through the dialog. */
async function openCubic(page: Page): Promise<void> {
  await newFunction(page);
  await page.keyboard.type("x^3 - 3*x");
  await expect(expression(page)).toHaveAccessibleDescription(
    "Reads as: x^3 - 3*x",
  );
  await page.keyboard.press("Enter");
  await expect(dialog(page)).toHaveCount(0);
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 50 });
  });
  await page.goto("/e2e.html");
  await expect(page.getByRole("tab", { name: "Plot" })).toBeVisible();
});

test("c03_t08b_the_expression_is_checked_after_a_pause_and_marks_where_it_stops", async ({
  page,
}) => {
  await newFunction(page);
  await page.keyboard.type("x^2 +");
  // The mark appears after the pause, as text linked to the field; it is not a live
  // region, so typing is never interrupted (keyboard §9).
  await expect(expression(page)).toHaveAttribute("aria-invalid", "true");
  await expect(expression(page)).toHaveAccessibleDescription(
    /The expression does not read here\. \(at character 6\)/,
  );
  await expect(dialog(page).getByRole("alert")).toHaveCount(0);
  expect(await seriousViolations(page)).toEqual([]);
  // Enter with the mark says it and opens nothing.
  await page.keyboard.press("Enter");
  await expect(dialog(page)).toBeVisible();
  await expect(
    page.locator('[data-announcer] [aria-live="polite"]'),
  ).toHaveText(/The expression does not read here\. \(at character 6\)/);
  expect(
    (await (await host(page)).evaluate((mock) => mock.commands())).filter(
      (command) => command === "function_open",
    ),
  ).toEqual([]);
  await page.keyboard.type(" 1");
  await expect(expression(page)).not.toHaveAttribute("aria-invalid", "true");
  await expect(expression(page)).toHaveAccessibleDescription(
    "Reads as: x^2 + 1",
  );
});

test("c03_t08b_an_error_with_no_place_names_no_character", async ({ page }) => {
  await newFunction(page);
  // The mock host answers `@` with a refusal that has no span (a variable count).
  await page.keyboard.type("x @ 2");
  await expect(expression(page)).toHaveAttribute("aria-invalid", "true");
  await expect(expression(page)).toHaveAccessibleDescription(
    /The expression cannot be used: Too many variables\./,
  );
  await expect(expression(page)).not.toHaveAccessibleDescription(
    /at character/,
  );
  // And `#` with no diagnostic at all: a plain sentence, no stray position.
  await page.keyboard.press("ControlOrMeta+a");
  await page.keyboard.type("x #");
  await expect(expression(page)).toHaveAccessibleDescription(
    /The expression does not read\./,
  );
  await expect(expression(page)).not.toHaveAccessibleDescription(
    /at character/,
  );
});

test("c03_t08b_an_empty_number_field_is_refused_in_words", async ({ page }) => {
  await newFunction(page);
  await page.keyboard.type("x^2");
  await expect(expression(page)).toHaveAccessibleDescription("Reads as: x^2");
  const from = dialog(page).getByRole("textbox", { name: "From x" });
  await from.focus();
  await page.keyboard.press("ControlOrMeta+a");
  await page.keyboard.press("Backspace");
  await page.keyboard.press("Enter");
  await expect(dialog(page)).toBeVisible();
  await expect(from).toHaveAttribute("aria-invalid", "true");
  await expect(from).toHaveAccessibleDescription(/Enter a number for From x\./);
  expect(
    (await (await host(page)).evaluate((mock) => mock.commands())).filter(
      (command) => command === "function_open",
    ),
  ).toEqual([]);
  // A number clears the error once the field commits it (on leaving it, as React Aria's
  // NumberField does).
  await page.keyboard.type("-3");
  await page.keyboard.press("Tab");
  await expect(from).not.toHaveAttribute("aria-invalid", "true");
});

test("c03_t08b_every_control_has_a_name_and_the_dialog_returns_the_focus", async ({
  page,
}) => {
  await newFunction(page);
  for (const name of ["From x", "To x", "Samples"]) {
    await expect(dialog(page).getByRole("textbox", { name })).toBeVisible();
  }
  await expect(
    dialog(page).getByRole("textbox", { name: "From x" }),
  ).toHaveAccessibleDescription("The x range the function is sampled over.");
  await page.keyboard.press("Escape");
  await expect(dialog(page)).toHaveCount(0);
});

test("a11y_24_opening_a_function_lists_its_critical_points", async ({
  page,
}) => {
  await openCubic(page);
  // The function becomes the shown dataset; the Inspector lists its points as text.
  const inspector = page.getByRole("complementary", { name: "Inspector" });
  await expect(
    inspector.getByText(/^f\(x\) = x\^3 - 3\*x from x = -10 to 10/),
  ).toBeVisible();
  // Opening says the function's summary, politely.
  await expect(
    page.locator('[data-announcer] [aria-live="polite"]'),
  ).toHaveText(/^f\(x\) = x\^3 - 3\*x from x = -10 to 10/);
  await expect(points(page).getByRole("row")).toHaveText([
    "zero at x = -1",
    "local maximum at x = 0, y = 1",
    "zero at x = 1",
    "horizontal asymptote y = 0",
  ]);
  expect(await seriousViolations(page)).toEqual([]);
});

test("a11y_10_enter_on_a_point_moves_the_one_cursor_and_the_list_marks_it", async ({
  page,
}) => {
  await openCubic(page);
  const rows = points(page).getByRole("row");
  await rows.first().focus();
  await page.keyboard.press("ArrowDown");
  await expect(rows.nth(1)).toBeFocused();
  // Moving through the list sends nothing and marks nothing.
  expect(
    await (await host(page)).evaluate((mock) => mock.cursorSets()),
  ).toEqual([]);
  await expect(points(page).locator('[aria-current="true"]')).toHaveCount(0);
  await page.keyboard.press("Enter");
  await expect
    .poll(async () => (await host(page)).evaluate((mock) => mock.cursorSets()))
    .toEqual([JSON.stringify({ dataset: "f1", row: 20, origin: "nav" })]);
  await expect(rows.nth(1)).toHaveAttribute("aria-current", "true");
  // The cursor moved by another view: the mark follows, the focus stays.
  await (
    await host(page)
  ).evaluate((mock) => {
    mock.moveCursor(30, "plot", "f1");
  });
  await expect(rows.nth(2)).toHaveAttribute("aria-current", "true");
  await expect(rows.nth(1)).not.toHaveAttribute("aria-current");
  await expect(rows.nth(1)).toBeFocused();
  // A feature without a row says so and moves nothing.
  await page.keyboard.press("End");
  await page.keyboard.press("Enter");
  await expect(
    page.locator('[data-announcer] [aria-live="polite"]'),
  ).toHaveText("horizontal asymptote y = 0 has no row to go to.");
  expect(
    await (await host(page)).evaluate((mock) => mock.cursorSets()),
  ).toHaveLength(1);
});

test("sdr_31_the_calculator_reads_out_and_an_inaudible_band_will_not_compile", async ({
  page,
}) => {
  await openCubic(page);
  const inspector = page.getByRole("complementary", { name: "Inspector" });
  await inspector
    .getByRole("button", { name: "Audification calculator" })
    .press("Enter");
  const readout = inspector.getByRole("definition");
  await expect(readout).toHaveText([
    "0.025 s",
    "40 to 20,000 Hz",
    "40 to 20,000 Hz",
  ]);
  const k = inspector.getByRole("textbox", { name: "Speed-up k" });
  await expect(k).toHaveValue("400");
  // The seismic preset: part of the band is above hearing.
  await inspector
    .getByRole("button", { name: "Use the seismic speed-up (k = 2200)" })
    .press("Enter");
  await expect(k).toHaveValue("2,200");
  await expect(
    inspector.getByText(/^Part of the band is outside 20 Hz to 20 kHz/),
  ).toBeVisible();
  // A tiny k puts the whole band below hearing: an error, never a normal readout.
  await k.fill("0.001");
  await k.press("Enter");
  await expect(
    inspector.getByText(/^Will not compile: the whole band is outside/),
  ).toBeVisible();
  await expect(readout.nth(2)).toHaveText("None");
  // Small band edges keep their digits (0.0001 Hz to 0.05 Hz, not 0 to 0.1).
  await expect(readout.nth(1)).toHaveText("0.0001 to 0.05 Hz");
  expect(await seriousViolations(page)).toEqual([]);
});
