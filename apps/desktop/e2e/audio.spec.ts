// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Devices, sonification volume and stop all audio (B01-T08; A11Y-15, T-13, FR-118).
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";
import { startOutsideRegions } from "./keyboard.ts";

interface Mock {
  commands(): string[];
  settings(): {
    audio: { device: string | null; volumeDb: number };
    gpu: string;
  };
}

const host = (page: Page) =>
  page.evaluateHandle(() => Reflect.get(window, "__SONARITHM_MOCK__") as Mock);

const stopAllCount = async (page: Page) =>
  (await host(page)).evaluate(
    (m) => m.commands().filter((c) => c === "stop_all").length,
  );

const audio = async (page: Page) =>
  (await host(page)).evaluate((m) => m.settings().audio);

const focused = (page: Page) =>
  page.evaluate(() => document.activeElement?.outerHTML ?? "");

/** The platform's Primary modifier as the app sees it (KB-07; WebKit reports a Mac). */
async function primary(page: Page): Promise<"Meta" | "Control"> {
  const mac = await page.evaluate(() => navigator.userAgent.includes("Mac"));
  return mac ? "Meta" : "Control";
}

test.beforeEach(async ({ page }) => {
  await page.goto("/e2e.html");
  await expect(
    page.getByRole("main").getByText("No dataset is open."),
  ).toBeVisible();
});

test("t_13_stop_all_audio_from_every_region_and_from_a_dialog", async ({
  page,
}) => {
  const key = `${await primary(page)}+Period`;
  await startOutsideRegions(page);
  for (let region = 1; region <= 5; region += 1) {
    await page.keyboard.press("F6");
    const before = await focused(page);
    await page.keyboard.press(key);
    await expect.poll(() => stopAllCount(page)).toBe(region);
    // Stopping never moves the focus.
    expect(await focused(page)).toBe(before);
  }
  // Inside a modal dialog, stop all audio still runs (keyboard §2), and the dialog stays.
  await page.keyboard.press(`${await primary(page)}+Comma`);
  const dialog = page.getByRole("dialog", { name: "Preferences" });
  await expect(dialog).toBeVisible();
  await page.keyboard.press(key);
  await expect.poll(() => stopAllCount(page)).toBe(6);
  await expect(dialog).toBeVisible();
});

test("fr_118_preferences_choose_the_device_and_set_the_volume_by_keyboard", async ({
  page,
}) => {
  const opener = await focused(page);
  await page.keyboard.press(`${await primary(page)}+Comma`);
  const dialog = page.getByRole("dialog", { name: "Preferences" });
  await expect(dialog).toBeVisible();

  // The output device: a select of the system default and the listed devices.
  const device = dialog.getByRole("button", { name: /Output device/ });
  await expect(device).toBeEnabled();
  await device.focus();
  await page.keyboard.press("Enter");
  await page.getByRole("option", { name: "Headphones" }).press("Enter");
  await expect.poll(async () => (await audio(page)).device).toBe("headphones");

  // The volume: decibels in the slider's value text; a step down is saved as −1 dB.
  const slider = dialog.getByRole("slider", { name: "Sonification volume" });
  await expect(slider).toHaveAttribute("aria-valuetext", /0 dB/);
  await slider.focus();
  await page.keyboard.press("ArrowLeft");
  await expect.poll(async () => (await audio(page)).volumeDb).toBe(-1);
  await expect(
    dialog.getByText("the system volume does not change", { exact: false }),
  ).toBeVisible();

  expect(await seriousViolations(page)).toEqual([]);

  // Escape closes it and the focus goes back where it was.
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  expect(await focused(page)).toBe(opener);
});

test("f_044_the_gpu_switch_saves_the_setting_or_says_why_it_cannot", async ({
  page,
}) => {
  await page.keyboard.press(`${await primary(page)}+Comma`);
  const dialog = page.getByRole("dialog", { name: "Preferences" });
  const gpu = dialog.getByRole("switch", {
    name: "Use the high-performance GPU for plots",
  });
  await expect(gpu).toBeVisible();
  const windows = await page.evaluate(() =>
    navigator.userAgent.includes("Windows"),
  );
  if (windows) {
    await expect(gpu).toHaveAccessibleDescription(
      "Takes effect after a restart.",
    );
    await expect(gpu).toBeEnabled();
    await gpu.focus();
    await page.keyboard.press("Space");
    await expect
      .poll(async () => (await host(page)).evaluate((m) => m.settings().gpu))
      .toBe("highPerformance");
  } else {
    // Only WebView2 can be asked; the reason is visible text, not a tooltip.
    await expect(gpu).toBeDisabled();
    const reason =
      "Only Windows lets SONARITHM choose the GPU; this system makes its own choice.";
    await expect(gpu).toHaveAccessibleDescription(reason);
    await expect(dialog.getByText(reason)).toBeVisible();
  }
  expect(await seriousViolations(page)).toEqual([]);
});

test("f_044_a_gpu_fallback_at_start_up_is_said_in_words", async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "__SONARITHM_START__", { gpuFallback: true });
  });
  await page.goto("/e2e.html");
  await expect(
    page
      .getByRole("contentinfo")
      .getByText("The high-performance GPU setting will apply after SONARITHM"),
  ).toBeVisible();
});
