// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import AxeBuilder from "@axe-core/playwright";
import type { Page } from "@playwright/test";
import { expect, test } from "@playwright/test";

/** The WCAG 2.2 A and AA rules (A11Y-31). */
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];

/**
 * axe's serious and critical violations on the page (testing-strategy §13.1), as
 * `rule: target` lines; empty when the page passes.
 */
export async function seriousViolations(page: Page): Promise<string[]> {
  const results = await new AxeBuilder({ page }).withTags(TAGS).analyze();
  return results.violations
    .filter((v) => v.impact === "serious" || v.impact === "critical")
    .flatMap((v) => v.nodes.map((node) => `${v.id}: ${node.target.join(" ")}`));
}

/**
 * The axe check after named lookups among labelled controls. Playwright 1.65's Firefox
 * (build 1572) crashes the page when axe reads `ariaLabelledByElements` after a named lookup
 * has read it in Playwright's own world (microsoft/playwright#43967, Mozilla bug 2090055;
 * F-226). Firefox skips only this scan, with a fixme note in the report; every other step
 * still runs in Firefox, and Chromium and WebKit run the scan. Remove the skip when
 * Playwright ships Firefox build 1575 or later (#487).
 */
export async function expectNoSeriousViolations(
  page: Page,
  browserName: string,
): Promise<void> {
  if (browserName === "firefox") {
    test.info().annotations.push({
      type: "fixme",
      description:
        "axe scan skipped in Firefox (F-226): microsoft/playwright#43967 (Mozilla bug 2090055); remove with #487",
    });
    return;
  }
  expect(await seriousViolations(page)).toEqual([]);
}
