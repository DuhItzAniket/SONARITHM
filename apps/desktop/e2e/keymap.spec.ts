// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Remapping (B03-T09; keyboard-and-navigation §13): Preferences › Keyboard's group switch
// unbinds the explorer's single keys, the recorder refuses reserved combinations, and a
// saved key works in the explorer while the old one does nothing.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

interface Mock {
  navVerbs(): string[];
  commands(): string[];
  settings(): {
    keymap: Record<string, string>;
    keyboard: { singleKeyCommands: boolean };
  };
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);

const settings = async (page: Page) =>
  (await host(page)).evaluate((mock) => mock.settings());

const explorer = (page: Page) =>
  page.getByRole("application", { name: "Chart explorer, Readings" });

/** Primary as the app sees it: Cmd where the user agent is a Mac (WebKit), else Ctrl. */
async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

async function openKeyboard(page: Page) {
  await page.keyboard.press(`${await primary(page)}+Comma`);
  const dialog = page.getByRole("dialog", { name: "Preferences" });
  await expect(dialog).toBeVisible();
  return dialog;
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_MOCK_DATASET__", { rows: 50 });
  });
  await page.goto("/e2e.html");
  await page.getByRole("tab", { name: "Plot" }).click();
  await expect(explorer(page)).toBeVisible();
});

test("kb_02_the_group_switch_turns_the_explorers_single_keys_off", async ({
  page,
}) => {
  const dialog = await openKeyboard(page);
  await dialog
    .getByRole("switch", { name: "Single-key commands in the chart explorer" })
    .click();
  await expect
    .poll(async () => (await settings(page)).keyboard.singleKeyCommands)
    .toBe(false);
  expect(await seriousViolations(page)).toEqual([]);
  await page.keyboard.press("Escape");
  await explorer(page).focus();
  await page.keyboard.press("n");
  await page.keyboard.press("w");
  await page.keyboard.press("ArrowRight");
  // Letters do nothing; the arrow still moves.
  await expect
    .poll(async () => (await host(page)).evaluate((mock) => mock.navVerbs()))
    .toEqual([JSON.stringify({ verb: "next" })]);
  const commands = await (await host(page)).evaluate((mock) => mock.commands());
  expect(commands).not.toContain("nav_where");
});

test("keyboard_13_the_recorder_refuses_reserved_keys_and_saves_a_free_one", async ({
  page,
}) => {
  const dialog = await openKeyboard(page);
  await dialog
    .getByRole("button", { name: "Change keys for Where am I" })
    .click();
  const recorder = page.getByRole("dialog", {
    name: "Change keys for Where am I",
  });
  const field = recorder.getByRole("textbox", { name: "New keys" });
  await field.focus();
  await page.keyboard.press("F6");
  await expect(recorder).toContainText("F6 cannot be used: it is reserved.");
  await expect(recorder.getByRole("button", { name: "Save" })).toBeDisabled();
  await page.keyboard.press("Shift+W");
  await expect(recorder).toContainText("is free.");
  await recorder.getByRole("button", { name: "Save" }).click();
  await expect
    .poll(async () => (await settings(page)).keymap["nav.whereAmI"])
    .toBe("Shift+W");
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await explorer(page).focus();
  await page.keyboard.press("w");
  await page.keyboard.press("Shift+W");
  await expect
    .poll(async () =>
      (await host(page)).evaluate(
        (mock) => mock.commands().filter((c) => c === "nav_where").length,
      ),
    )
    .toBe(1);
});
