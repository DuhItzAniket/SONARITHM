// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Open chart image and calibrate it by keyboard alone (C02-T10; A11Y-05, A11Y-26, A11Y-29;
// ADR-0017 "Raster calibration", keyboard-and-navigation §5.11): the crosshair's sliders
// say where they are, anchors are added at the crosshair, the checks are said when they
// change, and reading the chart says its extraction summary.
import type { Locator, Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

/**
 * The axe check of the calibration dialog. Playwright 1.65's Firefox (build 1572) crashes the
 * page when axe reads `ariaLabelledByElements` after a named lookup has read it in
 * Playwright's own world (microsoft/playwright#43967, Mozilla bug 2090055; F-226): here the
 * next lookup after the scan crashed. Firefox skips only this scan, with a fixme note in the
 * report; every other step still runs in Firefox, and Chromium and WebKit run the scan, as
 * `audio.spec.ts` does. Remove the skip with #487.
 *
 * TODO(#487): use the shared `expectNoSeriousViolations` from `./axe.ts` once #498 lands, so
 * the workaround is removed in one place.
 */
async function expectNoSeriousViolations(page: Page, browserName: string) {
  if (browserName === "firefox") {
    test.info().annotations.push({
      type: "fixme",
      description:
        "axe scan skipped in Firefox (F-226): microsoft/playwright#43967 (Mozilla bug 2090055); remove with #487",
    });
    return;
  }
  expect(await seriousViolations(page)).toEqual([]);
}

const polite = (page: Page) =>
  page.locator('[data-announcer] [aria-live="polite"]');

const commands = (page: Page) =>
  page.evaluate(() =>
    (
      Reflect.get(window, "__SONARITHM_MOCK__") as { commands(): string[] }
    ).commands(),
  );

/** Opens "Open chart image…" from the palette and names the file. */
async function openImage(page: Page): Promise<void> {
  await page.keyboard.press(`${await primary(page)}+Shift+P`);
  // WebKit drops keys typed before the palette is open (as palette.spec.ts waits).
  await expect(
    page.getByRole("dialog", { name: "Command palette" }),
  ).toBeVisible();
  await page.keyboard.type("Open chart image");
  await page.keyboard.press("Enter");
  const field = page.getByRole("textbox", { name: "Chart image file" });
  await expect(field).toBeFocused();
  await page.keyboard.type("C:/charts/rain.png");
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("dialog", { name: "Calibrate rain.png" }),
  ).toBeVisible();
}

/** Types a number into the field named `name`, replacing what it holds. */
async function setNumber(
  page: Page,
  name: string,
  value: string,
): Promise<void> {
  const field = page
    .getByRole("textbox", { name, exact: true })
    .or(page.getByRole("spinbutton", { name, exact: true }));
  await field.focus();
  // WebKit ignores Primary+A in these fields; select the old text, then type over it.
  await field.selectText();
  await page.keyboard.type(value);
  await page.keyboard.press("Tab");
}

/** Picks `option` in the select named `name` with the keyboard. */
async function choose(page: Page, name: string, option: string): Promise<void> {
  await page.getByRole("button", { name: new RegExp(name) }).focus();
  await page.keyboard.press("Enter");
  // The selected option's name also carries its check mark, as in audio.spec.ts.
  await page.getByRole("option", { name: option }).focus();
  await page.keyboard.press("Enter");
}

/** The new anchor's value field: a number field or, for dates and labels, a text field. */
const valueField = (scope: Locator) =>
  scope
    .getByRole("textbox", { name: "Value at the crosshair", exact: true })
    .or(scope.getByRole("spinbutton", { name: "Value at the crosshair" }));

async function addAnchor(
  page: Page,
  axis: "x axis" | "y axis",
  at: readonly [number, number],
  value: string,
): Promise<void> {
  await setNumber(page, "Pixel x", String(at[0]));
  await setNumber(page, "Pixel y", String(at[1]));
  await choose(page, "Axis of the new anchor", axis);
  await setNumber(page, "Value at the crosshair", value);
  await page
    .getByRole("button", { name: "Add anchor at the crosshair" })
    .focus();
  await page.keyboard.press("Enter");
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 36 });
  });
  await page.goto("/e2e.html");
  // Keys pressed before the app has loaded its dataset are lost (WebKit).
  await expect(
    page.getByRole("application", { name: /^Chart explorer/ }),
  ).toBeVisible();
});

test("c02_t10_a_chart_image_is_calibrated_and_read_by_keyboard_alone", async ({
  page,
  browserName,
}) => {
  await openImage(page);
  const dialog = page.getByRole("dialog", { name: "Calibrate rain.png" });
  await expect(
    dialog.getByRole("img", {
      name: "rain.png, a chart image of 200 by 120 pixels",
    }),
  ).toBeVisible();
  await expectNoSeriousViolations(page, browserName);

  // The crosshair's sliders: arrows step a pixel, and each says where it is.
  const x = dialog.getByRole("slider", { name: "Crosshair x (pixels)" });
  await x.focus();
  await page.keyboard.press("Home");
  await expect(x).toHaveAttribute("aria-valuetext", "Pixel 0");
  await page.keyboard.press("ArrowRight");
  await expect(x).toHaveAttribute("aria-valuetext", "Pixel 1");

  await addAnchor(page, "x axis", [20, 100], "0");
  // Add clears the value field for the next anchor (the #517 review).
  await expect(valueField(dialog)).toHaveValue("");
  await addAnchor(page, "x axis", [180, 100], "10");
  await addAnchor(page, "y axis", [20, 100], "0");
  await addAnchor(page, "y axis", [20, 10], "100");
  await expect(dialog.getByText("x1: pixel 20, 100, value 0")).toBeVisible();
  await expect(dialog.getByText("y2: pixel 20, 10, value 100")).toBeVisible();
  // The checks, shown and said once both axes fit.
  await expect(
    dialog.getByText("x axis: 2 anchors fit within 0%"),
  ).toBeVisible();
  await expect(polite(page)).toContainText("y axis: 2 anchors fit within 0%");
  // Once calibrated, the crosshair's slider says the value it is over too, in the same
  // moment as the step, and its text does not change afterwards (the #517 review).
  await setNumber(page, "Pixel x", "100");
  await x.focus();
  await expect(x).toHaveAttribute("aria-valuetext", /^Pixel 100, value 5/);
  await page.keyboard.press("ArrowRight");
  const stepped = await x.getAttribute("aria-valuetext");
  expect(stepped).toMatch(/^Pixel 101, value 5\.0/);
  await page.evaluate(
    () =>
      new Promise((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(resolve));
      }),
  );
  await expect(x).toHaveAttribute("aria-valuetext", stepped ?? "");
  expect(
    (await commands(page)).filter((c) => c === "chart_calibration_check")
      .length,
  ).toBeLessThanOrEqual(8);

  // The area to read: the anchors' area by default, the others by keyboard.
  const anchorsArea = dialog.getByRole("radio", { name: "The anchors' area" });
  await expect(anchorsArea).toBeChecked();
  await anchorsArea.focus();
  await page.keyboard.press("ArrowDown");
  await expect(
    dialog.getByRole("radio", {
      name: "The plot area found from the axis lines",
    }),
  ).toBeChecked();
  await page.keyboard.press("ArrowUp");
  await expect(anchorsArea).toBeChecked();

  await dialog.getByRole("button", { name: "Read the chart" }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(polite(page)).toContainText(
    "36 points: 0 exact, 0 calibrated, 36 estimated",
  );
  expect(await commands(page)).toContain("chart_calibration_apply");
});

test("c02_t10_ticks_out_of_order_are_said_and_reading_waits_for_two_anchors", async ({
  page,
}) => {
  await openImage(page);
  const dialog = page.getByRole("dialog", { name: "Calibrate rain.png" });
  await dialog.getByRole("button", { name: "Read the chart" }).focus();
  await page.keyboard.press("Enter");
  await expect(dialog.getByRole("alert")).toHaveText(
    "Add at least two anchors on each axis, then read the chart.",
  );
  // Said once, by the alert, not also by the live region (the #517 review).
  await expect(polite(page)).not.toContainText("Add at least two anchors");
  await addAnchor(page, "y axis", [20, 100], "0");
  await addAnchor(page, "y axis", [20, 50], "100");
  await addAnchor(page, "y axis", [20, 10], "50");
  await expect(polite(page)).toContainText(
    "The y axis cannot be calibrated: The tick values do not change in one direction",
  );
  // A deleted anchor is said, and the axis is checked again. The focus goes to the next
  // anchor's Delete, else the previous one's, else Add anchor (the #517 review).
  await dialog.getByRole("button", { name: "Delete y2" }).focus();
  await page.keyboard.press("Enter");
  await expect(polite(page)).toContainText("y2 deleted.");
  await expect(dialog.getByRole("button", { name: "Delete y2" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(dialog.getByRole("button", { name: "Delete y1" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(valueField(dialog)).toBeFocused();
});

test("c02_t10_escape_ends_the_session_and_returns_the_focus", async ({
  page,
}) => {
  await openImage(page);
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect.poll(() => commands(page)).toContain("chart_raster_close");
});
