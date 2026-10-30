// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import AxeBuilder from "@axe-core/playwright";
import type { Page } from "@playwright/test";

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
