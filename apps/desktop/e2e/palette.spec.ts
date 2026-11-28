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

/**
 * Poll every 20 ms: a command runs a tick after Enter, and the default back-off (100, 250,
 * 500 ms…) made each step of the walks wait most of a second.
 */
const STEP_POLL = [20];

/** The id of the palette's active result. */
async function activeKey(page: Page): Promise<string | null> {
  const input = palette(page).getByRole("searchbox");
  const active = await input.getAttribute("aria-activedescendant");
  if (active === null || active === "") return null;
  return page.locator(`[id="${active}"]`).getAttribute("data-key");
}

/** Each command's id and its name in the palette, in the palette's order. */
async function commandNames(
  page: Page,
): Promise<{ readonly id: string; readonly name: string }[]> {
  await openPalette(page);
  const names = await palette(page)
    .getByRole("menuitem")
    .evaluateAll((items) =>
      items.map((item) => {
        // The name is the item's content without its key binding, which is hidden.
        const copy = item.cloneNode(true) as HTMLElement;
        for (const hidden of copy.querySelectorAll('[aria-hidden="true"]')) {
          hidden.remove();
        }
        return {
          id: item.getAttribute("data-key") ?? "",
          name: copy.textContent.trim(),
        };
      }),
    );
  await page.keyboard.press("Escape");
  await expect(palette(page)).toBeHidden();
  return names;
}

test("kb_01_every_registered_command_runs_from_the_palette", async ({
  page,
}) => {
  const ids = await page.evaluate(() =>
    (Reflect.get(window, "__SONARITHM_COMMANDS__") as () => string[])(),
  );
  expect(ids.length).toBeGreaterThan(0);
  const commands = await commandNames(page);
  // The palette lists every registered command, in the registry's order.
  expect(commands.map((c) => c.id)).toEqual(ids);
  // Each command is reached as users mostly do: type its name, take the match, Enter. The
  // walk is linear in the count (the arrow walk is its own test, below). A step takes about
  // 0.7 s locally, 0.5 s of it React Aria's pause after typing; 2 s per command covers CI
  // runners up to three times slower (Firefox on CI ran 1.8 times Chromium's time).
  test.setTimeout(30_000 + ids.length * 2_000);
  for (const { id, name } of commands) {
    await openPalette(page);
    // The name arrives as one input, as from a paste or dictation: key by key, each
    // character re-filters the list, which tests the filter, not the command.
    await page.keyboard.insertText(name);
    // The filter leaves this command and any whose name contains its name (Stop, Stop all
    // sound), in the palette's order. Wait for exactly those, then arrow to this one.
    const matching = commands.filter((c) =>
      c.name.toLowerCase().includes(name.toLowerCase()),
    );
    await expect
      .poll(() => palette(page).getByRole("menuitem").count(), {
        message: `typing "${name}" leaves ${String(matching.length)} results`,
        intervals: STEP_POLL,
      })
      .toBe(matching.length);
    // Typing makes the first match active, 500 ms later: React Aria's Autocomplete waits
    // so a screen reader can say the typed text first. That wait is most of each step.
    // Each arrow then moves one down, so wait for each step.
    const place = matching.findIndex((c) => c.id === id);
    for (let i = 0; i <= place; i++) {
      if (i > 0) await page.keyboard.press("ArrowDown");
      await expect
        .poll(() => activeKey(page), {
          message: `typing "${name}" reaches ${id}`,
          timeout: 10_000,
          intervals: STEP_POLL,
        })
        .toBe(matching[i]?.id);
    }
    await page.keyboard.press("Enter");
    await expect
      .poll(
        () => page.evaluate(() => document.documentElement.dataset.lastCommand),
        {
          message: `${id} runs from the palette`,
          timeout: 10_000,
          intervals: STEP_POLL,
        },
      )
      .toBe(id);
    // These open a dialog of their own, just after the palette closes: the palette again,
    // the key references (help.explorer is the explorer's, B03-T09), Preferences, and
    // the explorer's value dialogs (G, Primary+F) and New function (C03-T08b).
    if (
      [
        "app.palette",
        "help.keyboard",
        "help.explorer",
        "app.preferences",
        "nav.goTo",
        "nav.findValue",
        "data.newFunction",
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

test("kb_01_the_arrows_walk_the_whole_palette_list", async ({ page }) => {
  const ids = await page.evaluate(() =>
    (Reflect.get(window, "__SONARITHM_COMMANDS__") as () => string[])(),
  );
  test.setTimeout(30_000 + ids.length * 500);
  await openPalette(page);
  // Keyboard only, once down the list: each arrow makes the next command active, and the
  // focus stays in the search field.
  for (const id of ids) {
    await page.keyboard.press("ArrowDown");
    await expect
      .poll(() => activeKey(page), {
        message: `the arrows reach ${id}`,
        intervals: STEP_POLL,
      })
      .toBe(id);
  }
  await expect(palette(page).getByRole("searchbox")).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(palette(page)).toBeHidden();
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
