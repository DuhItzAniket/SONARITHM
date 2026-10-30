// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The announcer in the real UI with the in-memory host (B03-T04; 08 §4): app-core
// announcements reach the live regions and the log, and announcing never moves focus.
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

/** Pushes an `announcement` event as the app core would. */
async function announce(
  page: Page,
  fields: { seq: number; speech: string; display: string } & Partial<{
    assertive: boolean;
    selfVoiced: boolean;
  }>,
): Promise<void> {
  await page.evaluate((f) => {
    const mock = Reflect.get(window, "__SONARITHM_MOCK__") as {
      emit(event: unknown): void;
    };
    mock.emit({
      event: "announcement",
      class: "navigation",
      interrupt: false,
      node: null,
      assertive: false,
      selfVoiced: false,
      ...f,
    });
  }, fields);
}

const polite = (page: Page) =>
  page.locator('[data-announcer] [aria-live="polite"]');
const assertive = (page: Page) =>
  page.locator('[data-announcer] [aria-live="assertive"]');

test.beforeEach(async ({ page }) => {
  await page.goto("/e2e.html");
  await expect(
    page.getByRole("main").getByText("No dataset is open."),
  ).toBeVisible();
});

test("a11y_17_announcing_never_moves_focus", async ({ page }) => {
  await page.keyboard.press("F6");
  await page.keyboard.press("F6");
  await page.keyboard.press("F6");
  const before = await page.evaluate(() => document.activeElement?.outerHTML);
  await announce(page, {
    seq: 1,
    speech: "28.7 degrees Celsius",
    display: "28.7 °C",
  });
  await expect(polite(page)).toHaveText("28.7 degrees Celsius");
  expect(await page.evaluate(() => document.activeElement?.outerHTML)).toBe(
    before,
  );
});

test("announcements reach the log with their display text", async ({
  page,
}) => {
  await announce(page, {
    seq: 1,
    speech: "Paused. 28.7 degrees Celsius",
    display: "Paused. 28.7 °C",
  });
  await announce(page, {
    seq: 2,
    speech: "Audio device lost.",
    display: "Audio device lost.",
    assertive: true,
  });
  await expect(assertive(page)).toHaveText("Audio device lost.");
  const log = page.getByRole("log", { name: "Announcements" });
  await expect(log.getByRole("listitem")).toHaveText([
    "Paused. 28.7 °C",
    "Audio device lost.",
  ]);
  await expect(log).toHaveAttribute("aria-live", "off");
});

test("a11y_19_self_voiced_announcements_are_logged_not_spoken", async ({
  page,
}) => {
  await announce(page, {
    seq: 1,
    speech: "Spoken by SONARITHM",
    display: "Spoken by SONARITHM",
    selfVoiced: true,
  });
  await expect(
    page.getByRole("log", { name: "Announcements" }).getByRole("listitem"),
  ).toHaveText(["Spoken by SONARITHM"]);
  await expect(polite(page)).toHaveText("");
});
