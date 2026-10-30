// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Page } from "@playwright/test";

/** One stop in the Tab order. */
export interface FocusStop {
  /** Tag, ID and text, for messages. */
  readonly label: string;
}

/** Some tabbable elements cannot be reached with Tab: a focus trap (WCAG 2.1.2, A11Y-05). */
export class FocusTrapError extends Error {
  override readonly name = "FocusTrapError";
}

/** A focused element shows no focus indicator (WCAG 2.4.7). */
export class FocusNotVisibleError extends Error {
  override readonly name = "FocusNotVisibleError";
}

const TABBABLE = [
  "a[href]",
  "button:not([disabled])",
  'input:not([disabled]):not([type="hidden"])',
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[tabindex]",
  '[contenteditable="true"]',
].join(", ");

/** Widgets that are one Tab stop with arrow keys inside (APG composite widgets). */
const COMPOSITE = [
  "toolbar",
  "radiogroup",
  "tablist",
  "grid",
  "treegrid",
  "tree",
  "listbox",
  "menu",
  "menubar",
]
  .map((role) => `[role="${role}"]`)
  .join(", ");

/**
 * A keyboard-only flow (testing-strategy §13.1): presses Tab from the start of the page,
 * with no pointer event, until every Tab stop has had focus (a composite widget such as a
 * toolbar counts once) or twice their number of presses has passed. Throws {@link FocusTrapError} if an element was never reached and
 * {@link FocusNotVisibleError} if a focused element showed no indicator. Returns the stops
 * in order.
 */
export async function tabThrough(page: Page): Promise<FocusStop[]> {
  const count = await page.evaluate(
    ({ selector, composite }) => {
      const all = [...document.querySelectorAll<HTMLElement>(selector)].filter(
        (el) => el.tabIndex >= 0 && el.getClientRects().length > 0,
      );
      // A composite widget is one Tab stop; arrows move inside it (APG).
      const keys = new Map<Element, string>();
      all.forEach((el, i) => {
        const group = el.closest(composite);
        if (group === null) {
          el.dataset.snrTab = String(i);
        } else {
          const key = keys.get(group) ?? `g${String(keys.size)}`;
          keys.set(group, key);
          el.dataset.snrTab = key;
        }
      });
      if (document.activeElement instanceof HTMLElement) {
        document.activeElement.blur();
      }
      return new Set(all.map((el) => el.dataset.snrTab)).size;
    },
    { selector: TABBABLE, composite: COMPOSITE },
  );
  const reached = new Set<string>();
  const stops: FocusStop[] = [];
  for (let press = 0; press < count * 2 + 2 && reached.size < count; press++) {
    await page.keyboard.press("Tab");
    const stop = await page.evaluate(() => {
      const el = document.activeElement;
      if (!(el instanceof HTMLElement) || el === document.body) {
        return null;
      }
      const style = getComputedStyle(el);
      const visible =
        (style.outlineStyle !== "none" && style.outlineWidth !== "0px") ||
        style.boxShadow !== "none";
      const text = el.getAttribute("aria-label") ?? el.textContent;
      return {
        tab: el.dataset.snrTab ?? null,
        label: `${el.tagName.toLowerCase()}#${el.id} "${text.trim()}"`,
        visible,
      };
    });
    if (stop === null) {
      continue;
    }
    if (!stop.visible) {
      throw new FocusNotVisibleError(`no visible focus on ${stop.label}`);
    }
    if (stop.tab !== null) {
      reached.add(stop.tab);
    }
    stops.push({ label: stop.label });
  }
  if (reached.size < count) {
    const missed = await page.evaluate(
      (seen) =>
        [...document.querySelectorAll<HTMLElement>("[data-snr-tab]")]
          .filter((el) => !seen.includes(el.dataset.snrTab ?? ""))
          .map(
            (el) =>
              `${el.tagName.toLowerCase()} "${(el.getAttribute("aria-label") ?? el.textContent).trim()}"`,
          ),
      [...reached],
    );
    throw new FocusTrapError(
      `${String(count - reached.size)} of ${String(count)} tabbable elements were never reached: ${missed.join(", ")}`,
    );
  }
  return stops;
}

/**
 * Moves the focus out of every region, as before start-up placed it (B01-T07), so a test
 * of the F6 cycle starts from the same point at every breakpoint.
 */
export async function startOutsideRegions(page: Page): Promise<void> {
  await page.evaluate(() => {
    const active = document.activeElement;
    if (active instanceof HTMLElement) {
      active.blur();
    }
  });
}
