// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The application frame (B02-T06): landmarks, the F6 order at every breakpoint, keyboard
// splitters, collapsed regions in the cycle, and no focus trap (A11Y-05, A11Y-07, KB-05).
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

import { seriousViolations } from "./axe.ts";
import { tabThrough } from "./keyboard.ts";

const ORDER = ["top", "project", "workspace", "inspector", "status"];

const SIZES = [
  { layout: "wide", width: 1440, height: 900 },
  { layout: "medium", width: 1100, height: 800 },
  { layout: "narrow", width: 800, height: 600 },
] as const;

/** The region that holds the focus. */
function focusedRegion(page: Page): Promise<string | null> {
  return page.evaluate(
    () =>
      document.activeElement?.closest<HTMLElement>("[data-region]")?.dataset
        .region ?? null,
  );
}

/** The regions F6 (or Shift+F6) visits, starting outside every region. */
async function cycle(page: Page, key: "F6" | "Shift+F6"): Promise<string[]> {
  const seen: string[] = [];
  for (const press of ORDER.map(() => key)) {
    await page.keyboard.press(press);
    seen.push((await focusedRegion(page)) ?? "none");
  }
  return seen;
}

for (const size of SIZES) {
  test.describe(`${size.layout} layout`, () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize({ width: size.width, height: size.height });
      await page.goto("/e2e.html");
      await expect(
        page.getByRole("main").getByText("No dataset is open."),
      ).toBeVisible();
    });

    test("a11y_07_the_regions_are_landmarks", async ({ page }) => {
      // Narrow shows one of Project, Workspace and Inspector; the others are hidden from
      // everyone until the region switcher or F6 shows them.
      const includeHidden = size.layout === "narrow";
      await expect(page.getByRole("banner")).toBeAttached();
      for (const [role, name] of [
        ["navigation", "Project"],
        ["main", "Workspace"],
        ["complementary", "Inspector"],
        ["contentinfo", "Status"],
      ] as const) {
        await expect(
          page.getByRole(role, { name, includeHidden }),
        ).toBeAttached();
      }
    });

    test("kb_05_f6_order_is_the_same_at_every_breakpoint", async ({ page }) => {
      expect(await cycle(page, "F6")).toEqual(ORDER);
      expect(await cycle(page, "Shift+F6")).toEqual([
        "inspector",
        "workspace",
        "project",
        "top",
        "status",
      ]);
    });

    test("a11y_05_no_focus_trap_and_focus_is_visible", async ({ page }) => {
      await tabThrough(page);
    });

    test("a11y_31_no_serious_axe_violation", async ({ page }) => {
      expect(await seriousViolations(page)).toEqual([]);
    });
  });
}

test("narrow: F6 into a hidden region shows it", async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto("/e2e.html");
  const inspector = page.getByRole("complementary", { name: "Inspector" });
  await expect(inspector).toBeHidden();
  for (let i = 0; i < 4; i++) {
    await page.keyboard.press("F6");
  }
  await expect(inspector).toBeVisible();
  expect(await focusedRegion(page)).toBe("inspector");
});

test("wide: splitters resize by keyboard and Enter collapses into Expand", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/e2e.html");
  const project = page.getByRole("navigation", { name: "Project" });
  const splitter = page.getByRole("separator", { name: "Resize Project" });
  await page.keyboard.press("F6");
  await page.keyboard.press("F6");
  // Tab from the project region's first stop reaches its splitter.
  while (!(await splitter.evaluate((el) => el === document.activeElement))) {
    await page.keyboard.press("Tab");
  }
  await expect(splitter).toHaveAttribute("aria-valuenow", "264");
  const before = (await project.boundingBox())?.width ?? 0;
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  await expect(splitter).toHaveAttribute("aria-valuenow", "280");
  expect((await project.boundingBox())?.width ?? 0).toBeGreaterThan(before);
  await page.keyboard.press("Enter");
  const expand = page.getByRole("button", { name: "Expand Project" });
  await expect(expand).toBeFocused();
  // The collapsed region stays in the F6 cycle as its Expand control.
  await page.keyboard.press("F6");
  await page.keyboard.press("Shift+F6");
  await expect(expand).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(splitter).toBeAttached();
});
