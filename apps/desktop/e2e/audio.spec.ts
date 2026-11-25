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
    audio: {
      device: string | null;
      volumeDb: number;
      calibratedLatencyS: Record<string, number>;
    };
    gpu: string;
  };
}

/**
 * The axe check of the Preferences dialog after its controls were used. Playwright 1.65's
 * Firefox (build 1572) crashes the page when axe reads `ariaLabelledByElements` after a named
 * lookup has read it in Playwright's own world (microsoft/playwright#43967, Mozilla bug
 * 2090055; F-226). Firefox skips only this scan, with a fixme note in the report; every other
 * step still runs in Firefox, and Chromium and WebKit run the scan. Remove the skip when
 * Playwright ships Firefox build 1575 or later (#487).
 */
async function expectNoSeriousViolations(page: Page, browserName: string) {
  if (browserName === "firefox") {
    test.info().annotations.push({
      type: "fixme",
      description:
        "axe scan skipped in Firefox: microsoft/playwright#43967 (Mozilla bug 2090055); remove with #487",
    });
    return;
  }
  expect(await seriousViolations(page)).toEqual([]);
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
  browserName,
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

  await expectNoSeriousViolations(page, browserName);

  // Escape closes it and the focus goes back where it was. React Aria restores the focus in
  // the animation frame after the dialog unmounts, so wait for it (WebKit in CI read too early).
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect.poll(() => focused(page)).toBe(opener);
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

test("b04_t04_preferences_store_the_measured_latency_of_the_selected_device", async ({
  page,
  browserName,
}) => {
  await page.keyboard.press(`${await primary(page)}+Comma`);
  const dialog = page.getByRole("dialog", { name: "Preferences" });
  await expect(dialog).toBeVisible();
  // ADR-0016: for the system default device (Speakers in the mock), in milliseconds.
  const field = dialog.getByRole("textbox", {
    name: "Measured output latency",
  });
  await expect(field).toBeEnabled();
  await field.fill("71");
  await page.keyboard.press("Tab");
  await expect
    .poll(async () => (await audio(page)).calibratedLatencyS)
    .toEqual({ speakers: 0.071 });
  // Emptying the field removes it, so the playhead uses the system's estimate again.
  await field.fill("");
  await page.keyboard.press("Tab");
  await expect
    .poll(async () => (await audio(page)).calibratedLatencyS)
    .toEqual({});
  await expectNoSeriousViolations(page, browserName);
});
