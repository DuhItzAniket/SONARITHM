// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The announcement routes under test (08 §4, OD-05): a managed pair of ARIA live regions,
// a native UIA notification (Windows only, through the host), or both.

import { invoke } from "@tauri-apps/api/core";

export type Route = "live" | "native" | "both";
export type Priority = "polite" | "assertive";

export interface LogEntry {
  at: number;
  route: Route;
  priority: Priority;
  text: string;
  result: string;
}

const regions: Partial<Record<Priority, HTMLElement>> = {};
let pending: number | undefined;
const listeners = new Set<(e: LogEntry) => void>();

/** Creates the two live regions once. They exist before any announcement, which screen
 *  readers need in order to track them. */
export function mountAnnouncer() {
  for (const priority of ["polite", "assertive"] as const) {
    if (regions[priority]) continue;
    const el = document.createElement("div");
    el.className = "visually-hidden";
    el.setAttribute("aria-live", priority);
    el.setAttribute("aria-atomic", "true");
    el.dataset.announcer = priority;
    document.body.append(el);
    regions[priority] = el;
  }
}

/** Clear-then-set: empty the region, then write the text after a short delay, so the same
 *  text is announced again. A newer announcement cancels a pending one (supersede, A11Y-17). */
function live(text: string, priority: Priority) {
  const el = regions[priority];
  if (!el) return "no live region";
  window.clearTimeout(pending);
  el.textContent = "";
  pending = window.setTimeout(() => {
    el.textContent = text;
  }, 80);
  return "live region";
}

async function native(text: string, priority: Priority) {
  try {
    await invoke("announce_native", { text, assertive: priority === "assertive" });
    return "UIA notification";
  } catch (e) {
    return `native failed: ${String(e)}`;
  }
}

export async function announce(text: string, route: Route, priority: Priority = "polite") {
  const results: string[] = [];
  if (route !== "native") results.push(live(text, priority));
  if (route !== "live") results.push(await native(text, priority));
  const entry = { at: performance.now(), route, priority, text, result: results.join(" + ") };
  for (const l of listeners) l(entry);
}

export function onAnnounce(listener: (e: LogEntry) => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
