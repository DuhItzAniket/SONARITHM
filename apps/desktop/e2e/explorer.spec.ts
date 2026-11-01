// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The chart explorer and keymap v0 (B03-T09; keyboard-and-navigation §4.2, §5.4, §5.5; 08
// §5): one named and described Tab stop in the Plot tab whose keys send the app core's
// verbs, move the one cursor, toggle the modalities and work only inside it (KB-02).
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

interface Mock {
  navVerbs(): string[];
  commands(): string[];
  settings(): {
    verbosity: string;
    modalities: Record<string, boolean>;
  };
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);

const navVerbs = async (page: Page) =>
  (await host(page)).evaluate((mock) => mock.navVerbs());

const explorer = (page: Page) =>
  page.getByRole("application", { name: "Chart explorer, Readings" });

const readout = (page: Page) => page.locator("[data-cursor-readout]");

/**
 * The modifiers as the app sees them (KB-07): it picks its bindings from the user agent,
 * and Playwright's WebKit project reports a Mac one, whatever OS runs the test. Jump is
 * Ctrl on Windows and Linux and Option on macOS; Primary is Ctrl or Cmd.
 */
async function modifiers(
  page: Page,
): Promise<{ jump: "Alt" | "Control"; primary: "Meta" | "Control" }> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac
    ? { jump: "Alt", primary: "Meta" }
    : { jump: "Control", primary: "Control" };
}

/** Every movement key of keymap v0 and the verb it sends (keyboard §4.2). */
const keys = (jump: string): readonly (readonly [string, object])[] => [
  ["ArrowRight", { verb: "next" }],
  ["ArrowLeft", { verb: "previous" }],
  [`${jump}+ArrowRight`, { verb: "jumpForward" }],
  [`${jump}+ArrowLeft`, { verb: "jumpBack" }],
  ["Home", { verb: "first" }],
  ["End", { verb: "last" }],
  ["ArrowDown", { verb: "into" }],
  ["Enter", { verb: "activate" }],
  ["ArrowUp", { verb: "out" }],
  ["Backspace", { verb: "out" }],
  ["PageDown", { verb: "nextSeries" }],
  ["PageUp", { verb: "previousSeries" }],
  [`${jump}+ArrowUp`, { verb: "maximum" }],
  [`${jump}+ArrowDown`, { verb: "minimum" }],
  ["n", { verb: "nextInsight" }],
  ["Shift+N", { verb: "previousInsight" }],
  ["x", { verb: "landmark", landmark: "xAxis" }],
  ["y", { verb: "landmark", landmark: "yAxis" }],
  ["i", { verb: "landmark", landmark: "insights" }],
  ["f", { verb: "landmark", landmark: "filters" }],
  ["d", { verb: "landmark", landmark: "dataset" }],
];

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 50 });
  });
  await page.goto("/e2e.html");
  await page.getByRole("tab", { name: "Plot" }).click();
  await expect(explorer(page)).toBeVisible();
});

test("b03_t09_the_explorer_is_a_named_and_described_tab_stop", async ({
  page,
}) => {
  await expect(explorer(page)).toHaveAccessibleDescription(
    "Arrow keys move through the chart. Press H for its keys.",
  );
  await expect(explorer(page)).toHaveAttribute("tabindex", "0");
  await expect(explorer(page)).not.toHaveAttribute("aria-activedescendant");
  // The global command reaches it from anywhere (keyboard §5.1).
  await page.getByRole("tab", { name: "Plot" }).focus();
  const { primary } = await modifiers(page);
  await page.keyboard.press(`${primary}+Shift+C`);
  await expect(explorer(page)).toBeFocused();
  expect(await seriousViolations(page)).toEqual([]);
});

test("b03_t09_every_movement_key_sends_its_verb", async ({ page }) => {
  const table = keys((await modifiers(page)).jump);
  await explorer(page).focus();
  for (const [key] of table) {
    await page.keyboard.press(key);
  }
  await expect
    .poll(() => navVerbs(page))
    .toEqual(table.map(([, verb]) => JSON.stringify(verb)));
  // The explorer never took the focus away from itself.
  await expect(explorer(page)).toBeFocused();
});

test("a11y_10_explorer_moves_move_the_one_cursor", async ({ page }) => {
  await explorer(page).focus();
  await page.keyboard.press("ArrowRight");
  await expect(readout(page)).toContainText("Data row 2");
  await page.keyboard.press("End");
  await expect(readout(page)).toContainText("Data row 50");
  // At the end, a step stays: the app core says "Last" and plays E-BOUNDARY.
  await page.keyboard.press("ArrowRight");
  await expect(readout(page)).toContainText("Data row 50");
});

test("kb_02_explorer_letters_do_nothing_outside_the_explorer", async ({
  page,
}) => {
  await page.getByRole("tab", { name: "Plot" }).focus();
  for (const key of ["n", "x", "w", "s", "Shift+T"]) {
    await page.keyboard.press(key);
  }
  expect(await navVerbs(page)).toEqual([]);
  const commands = await (await host(page)).evaluate((mock) => mock.commands());
  expect(commands).not.toContain("nav_where");
  expect(commands).not.toContain("settings_set");
});

test("b03_t09_where_am_i_and_repeat_reach_the_app_core", async ({ page }) => {
  await explorer(page).focus();
  await page.keyboard.press("w");
  await page.keyboard.press("r");
  await expect
    .poll(async () =>
      (await host(page)).evaluate((mock) =>
        mock.commands().filter((c) => c !== "cursor_get"),
      ),
    )
    .toEqual(expect.arrayContaining(["nav_where", "announce_repeat"]));
});

test("b03_t09_go_to_asks_for_an_x_and_returns_the_focus", async ({ page }) => {
  await explorer(page).focus();
  await page.keyboard.press("g");
  const dialog = page.getByRole("dialog", { name: "Go to an x value" });
  await expect(dialog).toBeVisible();
  await dialog.getByRole("textbox", { name: "X value" }).fill("2.5");
  await page.keyboard.press("Enter");
  await expect(dialog).toBeHidden();
  await expect
    .poll(() => navVerbs(page))
    .toEqual([JSON.stringify({ verb: "goTo", x: { number: 2.5 } })]);
  // React Aria returns the focus in an animation frame (F-176), so poll for it.
  await expect
    .poll(() => explorer(page).evaluate((e) => e === document.activeElement))
    .toBe(true);
});

test("b03_t09_modality_and_verbosity_toggles_save_the_settings", async ({
  page,
}) => {
  const settings = async () =>
    (await host(page)).evaluate((mock) => mock.settings());
  await explorer(page).focus();
  await page.keyboard.press("s");
  await expect
    .poll(async () => (await settings()).modalities.sonification)
    .toBe(false);
  await page.keyboard.press("Shift+T");
  await expect.poll(async () => (await settings()).verbosity).toBe("verbose");
  await page.keyboard.press("s");
  await expect
    .poll(async () => (await settings()).modalities.sonification)
    .toBe(true);
  // Each toggle says what it did, politely.
  await expect(
    page.locator('[data-announcer] [aria-live="polite"]'),
  ).toHaveText("Sonification on");
});

test("b03_t09_h_lists_the_explorer_keys_first", async ({ page }) => {
  await explorer(page).focus();
  await page.keyboard.press("h");
  const reference = page.getByRole("dialog");
  await expect(reference).toBeVisible();
  await expect(reference).toContainText("Next at this level");
  await expect(reference).toContainText("Where am I");
});
