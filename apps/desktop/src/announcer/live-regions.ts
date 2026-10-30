// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { LiveRegionSink } from "./announcer.ts";
import styles from "./announcer.module.css";

/**
 * Adds the managed live-region pair to `doc` once and returns its sink (08 §4). The
 * regions exist, empty, from start-up, because a screen reader only watches live regions
 * that were present before their content changed. They are visually hidden; the
 * announcement log in the Status region is the visible record (A11Y-24).
 */
export function mountLiveRegions(doc: Document): LiveRegionSink {
  const container = doc.createElement("div");
  container.className = styles.hidden ?? "";
  container.dataset.announcer = "";
  const polite = doc.createElement("div");
  polite.setAttribute("aria-live", "polite");
  polite.setAttribute("aria-atomic", "true");
  const assertive = doc.createElement("div");
  assertive.setAttribute("aria-live", "assertive");
  assertive.setAttribute("aria-atomic", "true");
  container.append(polite, assertive);
  doc.body.append(container);
  return {
    set(politeness, text) {
      (politeness === "assertive" ? assertive : polite).textContent = text;
    },
  };
}
