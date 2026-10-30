// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The empty workspace in Chromium, WebKit and Firefox with the in-memory host (B02-T03):
// it is named, passes axe and has no focus trap.
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";
import { tabThrough } from "./keyboard.ts";

test.beforeEach(async ({ page }) => {
  await page.goto("/e2e.html");
  await expect(
    page.getByRole("main").getByText("No dataset is open."),
  ).toBeVisible();
});

test("the workspace is a named main region", async ({ page }) => {
  await expect(page.getByRole("main", { name: "Workspace" })).toBeVisible();
  await expect(
    page.getByRole("heading", { level: 1, name: "Workspace" }),
  ).toBeVisible();
});

test("a11y_31_the_workspace_has_no_serious_axe_violation", async ({ page }) => {
  expect(await seriousViolations(page)).toEqual([]);
});

test("a11y_05_the_workspace_has_no_focus_trap", async ({ page }) => {
  await tabThrough(page);
});
