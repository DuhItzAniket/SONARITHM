// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The spec editor (B05-T03; keyboard-and-navigation §5.9, §5.10; A11Y-05, A11Y-29, FR-112).
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";

/** A spec whose pitch names a field the data does not have (SNR-SPEC-070). */
const BROKEN = `{
  "specVersion": "0.4",
  "mode": "analysis",
  "tracks": [
    {
      "id": "temperature",
      "mark": "tone",
      "channels": {
        "pitch": {
          "field": "tempx"
        }
      }
    }
  ]
}`;

interface Mock {
  commands(): string[];
  specText(): string | null;
  settings(): { editor: { autoComplete: boolean } };
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);
const specText = async (page: Page) =>
  (await host(page)).evaluate((m) => m.specText());
const sent = async (page: Page, command: string) =>
  (await host(page)).evaluate(
    (m, wanted) => m.commands().filter((c) => c === wanted).length,
    command,
  );

const polite = (page: Page) =>
  page.locator('[data-announcer] [aria-live="polite"]');
const editor = (page: Page) =>
  page.getByRole("textbox", { name: "Spec (JSON)" });
const summary = (page: Page) => page.locator("#spec-summary");

/** The platform's Primary modifier as the app sees it (KB-07; WebKit reports a Mac). */
async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

/** Opens the Spec tab from the workspace's tab list and waits for the editor to load. */
async function openSpecTab(page: Page) {
  await page.getByRole("tab", { name: "Plot" }).focus();
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("tab", { name: "Spec" })).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(editor(page)).toBeVisible();
  // React Aria gives a tab panel with nothing tabbable a tabindex of 0 and drops it when a
  // mutation observer sees the editor; until then a Tab lands on the panel, which WebKit then
  // blurs (F-228). Wait for the panel to settle.
  await expect(
    page.getByRole("tabpanel", { name: "Spec" }),
  ).not.toHaveAttribute("tabindex");
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript((spec) => {
    Reflect.set(window, "__SONARITHM_MOCK_SPEC__", spec);
  }, BROKEN);
  await page.goto("/e2e.html");
  await expect(
    page.getByRole("main").getByText("No dataset is open."),
  ).toBeVisible();
});

test("pb_07_codemirror_loads_only_when_the_spec_tab_opens", async ({
  page,
}) => {
  const loaded = () =>
    page.evaluate(() =>
      performance
        .getEntriesByType("resource")
        .some((entry) => entry.name.includes("@codemirror")),
    );
  expect(await loaded()).toBe(false);
  await openSpecTab(page);
  expect(await loaded()).toBe(true);
});

test("t_12_edit_the_spec_find_and_fix_a_diagnostic", async ({
  page,
  browserName,
}) => {
  const mod = await primary(page);
  await openSpecTab(page);
  // Tab from the tab list reaches the editor: one Tab stop, named, with its summary.
  await page.keyboard.press("Tab");
  await expect(editor(page)).toBeFocused();
  await expect(summary(page)).toHaveText("1 error, 0 warnings, 0 notes.");
  await expect(editor(page)).toHaveAccessibleDescription(
    "1 error, 0 warnings, 0 notes.",
  );
  // Typing and validating never spoke.
  await expect(polite(page)).toHaveText("");

  // F8 goes to the diagnostic and says it, with its place, code and message.
  await page.keyboard.press("F8");
  await expect(polite(page)).toHaveText(
    'Error, line 10, column 11. SNR-SPEC-070: The data has no field "tempx". Name a field of the data, or declare it in `data.fields`.',
  );
  // The mark is a solid underline, not colour alone (DL-X08).
  await expect(page.locator(".cm-lintRange-error")).toHaveCSS(
    "text-decoration-style",
    "solid",
  );

  // Fix it from the caret: replace the property with a field the data has.
  await page.keyboard.press("Shift+End");
  await page.keyboard.press("Backspace");
  await page.keyboard.type('"field": "temperature"');
  await expect(summary(page)).toHaveText("No problems.");

  // Primary+Enter applies the spec and says what the compile made.
  await page.keyboard.press(`${mod}+Enter`);
  await expect(polite(page)).toHaveText(
    "Spec applied. Compiled 1 track, 12.5 seconds.",
  );
  expect(await specText(page)).toContain('"field": "temperature"');
  await expect(editor(page)).toBeFocused();

  // Firefox skips only this scan: Playwright 1.65's Firefox crashes when axe runs after a
  // named lookup (F-226; remove with #487).
  if (browserName === "firefox") {
    test.info().annotations.push({
      type: "fixme",
      description:
        "axe scan skipped in Firefox: microsoft/playwright#43967 (Mozilla bug 2090055); remove with #487",
    });
  } else {
    expect(await seriousViolations(page)).toEqual([]);
  }
});

test("enter_on_a_diagnostic_goes_to_its_place_in_the_editor", async ({
  page,
}) => {
  await openSpecTab(page);
  await expect(summary(page)).toHaveText("1 error, 0 warnings, 0 notes.");
  await page.getByRole("tab", { name: "Announcements" }).focus();
  await page.keyboard.press("ArrowRight");
  const grid = page.getByRole("grid", { name: "Diagnostics" });
  await expect(grid).toBeVisible();
  const row = grid.getByRole("row");
  await expect(row).toHaveCount(1);
  await expect(row).toContainText("Error");
  await expect(row).toContainText("SNR-SPEC-070");
  await expect(row).toContainText("Line 10, column 11");
  await page.keyboard.press("Tab");
  await expect(row).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(editor(page)).toBeFocused();
  await expect(polite(page)).toHaveText(
    /^Error, line 10, column 11\. SNR-SPEC-070/,
  );
  // The caret is at the diagnostic: F8 from here wraps round to the same one.
  await page.keyboard.press("F8");
  await expect(polite(page)).toHaveText(/^Error, line 10, column 11\./);
});

test("undo_says_which_history_it_stepped", async ({ page }) => {
  const mod = await primary(page);
  await openSpecTab(page);
  await page.keyboard.press("Tab");
  await expect(editor(page)).toBeFocused();
  // Apply once, so the spec history has an edit.
  await page.keyboard.press(`${mod}+End`);
  await page.keyboard.type("\n");
  await page.keyboard.press(`${mod}+Enter`);
  await expect(polite(page)).toHaveText(
    /^Spec applied, not compiled: 1 error\./,
  );
  expect(await sent(page, "spec_set")).toBe(1);

  // Typing, then Primary+Z: the editor's own undo, said as a text edit.
  await page.keyboard.type(" ");
  await expect(summary(page)).toHaveText("Checking the spec.");
  // CodeMirror takes Primary from navigator.platform, the app from the user agent. On a real
  // Mac both say Cmd; Playwright's WebKit on Windows reports a Mac agent on a Win32 platform.
  const textMod = await page.evaluate(() =>
    navigator.platform.includes("Mac") ? "Meta" : "Control",
  );
  await page.keyboard.press(`${textMod}+z`);
  await expect(polite(page)).toHaveText("Undid a text edit.");
  expect(await sent(page, "history_undo")).toBe(0);

  // No typing left to undo: Primary+Z (the app's) steps the spec history, and says so.
  await page.keyboard.press(`${mod}+z`);
  await expect(polite(page)).toHaveText("Undid spec change: Set spec");
  expect(await sent(page, "history_undo")).toBe(1);

  // Outside the editor, Redo is the spec history's.
  await page.getByRole("tab", { name: "Spec" }).focus();
  await page.keyboard.press(mod === "Meta" ? "Meta+Shift+z" : "Control+y");
  await expect(polite(page)).toHaveText("Redid spec change: Set spec");
  expect(await sent(page, "history_redo")).toBe(1);
});

test("a_change_outside_the_editor_shows_in_it_and_keeps_unapplied_edits", async ({
  page,
}) => {
  const mod = await primary(page);
  await openSpecTab(page);
  await page.keyboard.press("Tab");
  await page.keyboard.press(`${mod}+End`);
  await page.keyboard.type("\n");
  await page.keyboard.press(`${mod}+Enter`);
  await expect(polite(page)).toHaveText(/^Spec applied/);
  const applied = await specText(page);

  // With nothing unapplied, an undo made outside the editor shows in it at once.
  await page.getByRole("tab", { name: "Spec" }).focus();
  await page.keyboard.press(`${mod}+z`);
  await expect(polite(page)).toHaveText("Undid spec change: Set spec");
  expect(await specText(page)).toBe(BROKEN);
  await expect(editor(page)).toHaveText(/tempx/);
  expect((await editor(page).textContent())?.length).toBeLessThan(
    applied?.length ?? 0,
  );

  // With unapplied edits, they are kept and the editor says the spec changed.
  await page.keyboard.press("Tab");
  await page.keyboard.press(`${mod}+End`);
  await page.keyboard.type(" ");
  await page.getByRole("tab", { name: "Spec" }).focus();
  await page.keyboard.press(mod === "Meta" ? "Meta+Shift+z" : "Control+y");
  await expect(
    page.getByText(
      "The spec changed outside the editor. Your unapplied edits are kept.",
    ),
  ).toBeVisible();
  const load = page.getByRole("button", { name: "Load the current spec" });
  await load.focus();
  await page.keyboard.press("Enter");
  await expect(load).toBeHidden();
  expect(await editor(page).textContent()).toBe(
    (await specText(page))?.replaceAll("\n", ""),
  );
});

test("b05_t03_preferences_turn_on_completion_while_typing_by_keyboard", async ({
  page,
}) => {
  const mod = await primary(page);
  await page.keyboard.press(`${mod}+Comma`);
  const dialog = page.getByRole("dialog", { name: "Preferences" });
  const toggle = dialog.getByRole("switch", {
    name: "Show completions while typing",
  });
  await expect(toggle).not.toBeChecked();
  await expect(toggle).toHaveAccessibleDescription(
    /completions open only when you ask/,
  );
  await toggle.focus();
  await page.keyboard.press("Space");
  await expect(toggle).toBeChecked();
  await expect
    .poll(async () =>
      (await host(page)).evaluate((m) => m.settings().editor.autoComplete),
    )
    .toBe(true);
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();

  // Now typing a key opens the list on its own.
  await openSpecTab(page);
  await page.keyboard.press("Tab");
  await page.keyboard.press(`${mod}+Home`);
  await page.keyboard.press("End");
  await page.keyboard.press("Enter");
  await page.keyboard.type('"na');
  await expect(page.getByRole("listbox")).toBeVisible();
});

test("completion_opens_on_request_and_tab_leaves_the_editor", async ({
  page,
}) => {
  const mod = await primary(page);
  await openSpecTab(page);
  await page.keyboard.press("Tab");
  await expect(editor(page)).toBeFocused();
  // Into the top-level object, after its brace: typing alone opens no list.
  await page.keyboard.press(`${mod}+Home`);
  await page.keyboard.press("End");
  await page.keyboard.press("Enter");
  await page.keyboard.type('"na');
  await expect(page.getByRole("listbox")).toHaveCount(0);
  await page.keyboard.press(`${mod}+i`);
  const list = page.getByRole("listbox");
  await expect(list).toBeVisible();
  const option = list.getByRole("option", { name: /"name"/ });
  await expect(option).toHaveAttribute("aria-selected", "true");
  // CodeMirror ignores Enter for 75 ms after the list opens (its interactionDelay), so a
  // key pressed while typing fast is not taken as a choice.
  await page.waitForTimeout(150);
  await page.keyboard.press("Enter");
  await expect(editor(page)).toContainText('"name": ');
  await expect(list).toHaveCount(0);
  // Tab is not indentation: it leaves the editor (keyboard §5.9).
  await page.keyboard.press("Tab");
  await expect(editor(page)).not.toBeFocused();
});
