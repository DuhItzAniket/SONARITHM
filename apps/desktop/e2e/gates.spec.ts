// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Negative tests of the gates (B02-T03): a seeded focus trap fails the keyboard-only flow,
// and a seeded unnamed button fails the axe check.
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";
import { FocusTrapError, tabThrough } from "./keyboard.ts";

test.beforeEach(async ({ page }) => {
  await page.goto("/e2e/fixtures/focus-trap.html");
});

test("a seeded focus trap fails the keyboard-only flow", async ({ page }) => {
  await expect(tabThrough(page)).rejects.toBeInstanceOf(FocusTrapError);
});

test("a seeded unnamed button fails the axe check", async ({ page }) => {
  const violations = await seriousViolations(page);
  expect(violations).toContainEqual(expect.stringMatching(/^button-name: /));
});
