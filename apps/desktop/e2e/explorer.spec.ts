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
  toggles(): string[];
  transport(): { muted: boolean };
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

test("kb_01_the_explorer_is_reached_from_any_tab", async ({ page }) => {
  // From the Table tab, "Go to the chart explorer" selects Plot and focuses the explorer.
  await page.getByRole("tab", { name: "Table" }).click();
  await expect(explorer(page)).toHaveCount(0);
  const { primary } = await modifiers(page);
  await page.keyboard.press(`${primary}+Shift+C`);
  await expect(explorer(page)).toBeFocused();
  // F1 in the explorer is its own help (keyboard §5.4).
  await page.keyboard.press("F1");
  await expect(page.getByRole("dialog")).toContainText("Next at this level");
});

test("b03_t09_h_lists_the_explorer_keys_first", async ({ page }) => {
  await explorer(page).focus();
  await page.keyboard.press("h");
  const reference = page.getByRole("dialog");
  await expect(reference).toBeVisible();
  await expect(reference).toContainText("Next at this level");
  await expect(reference).toContainText("Where am I");
});

// The toggles (keyboard §5.5): the app core flips the effective value, saves it and says
// it; the mock host answers as the app core does (its Rust tests assert the effects).
const polite = (page: Page) =>
  page.locator('[data-announcer] [aria-live="polite"]');
const assertive = (page: Page) =>
  page.locator('[data-announcer] [aria-live="assertive"]');

test("b03_t09_each_toggle_key_flips_its_modality_in_the_app_core", async ({
  page,
}) => {
  await explorer(page).focus();
  for (const key of ["b", "v", "Shift+T", "v", "b"]) {
    await page.keyboard.press(key);
  }
  await expect
    .poll(async () => (await host(page)).evaluate((mock) => mock.toggles()))
    .toEqual(["braille", "visual", "verbosity", "visual", "braille"]);
  // The base settings are not written: the choice is the app core's top layer.
  const commands = await (await host(page)).evaluate((mock) => mock.commands());
  expect(commands).not.toContain("settings_set");
});

test("b03_t09_speech_off_is_said_before_it_goes_quiet", async ({ page }) => {
  await explorer(page).focus();
  await page.keyboard.press("t");
  await expect(polite(page)).toHaveText("Speech off");
  await page.keyboard.press("t");
  await expect(polite(page)).toHaveText("Speech on");
});

test("a11y_10_the_highlight_follows_the_visual_toggle", async ({ page }) => {
  const crosshair = page.locator("[data-plot-cursor]");
  await explorer(page).focus();
  await page.keyboard.press("ArrowRight");
  await expect(crosshair).toHaveJSProperty("hidden", false);
  await page.keyboard.press("v");
  await expect(polite(page)).toHaveText("Visual highlight off");
  await expect(crosshair).toHaveJSProperty("hidden", true);
  await page.keyboard.press("v");
  await expect(crosshair).toHaveJSProperty("hidden", false);
});

test("b03_t09_the_last_non_visual_output_cannot_be_turned_off", async ({
  page,
}) => {
  // Braille is off in the mock host: speech off, then sonification off, leaves nothing.
  await explorer(page).focus();
  await page.keyboard.press("t");
  await expect(polite(page)).toHaveText("Speech off");
  await page.keyboard.press("s");
  await expect(assertive(page)).toHaveText(
    "Speech, sonification and braille cannot all be off. Turn another one on first.",
  );
  const muted = await (
    await host(page)
  ).evaluate((mock) => mock.transport().muted);
  expect(muted).toBe(false);
});
