// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * The fixed regions in their F6 order (design-language §4.1, keyboard-and-navigation §3).
 * The order is the same in every layout and theme (A11Y-07, KB-05).
 */
export const REGIONS = [
  "top",
  "project",
  "workspace",
  "inspector",
  "status",
] as const;

/** One of {@link REGIONS}. */
export type RegionId = (typeof REGIONS)[number];

/** The regions the Narrow layout shows one at a time. */
export const SWITCHABLE = ["project", "workspace", "inspector"] as const;

/** One of {@link SWITCHABLE}. */
export type SwitchableRegion = (typeof SWITCHABLE)[number];

/** Whether a region is one the Narrow layout switches between. */
export function isSwitchable(region: RegionId): region is SwitchableRegion {
  return (SWITCHABLE as readonly string[]).includes(region);
}

/**
 * The region F6 (or Shift+F6, `backward`) moves to from `current`, wrapping. From outside
 * every region, F6 goes to the first and Shift+F6 to the last.
 */
export function nextRegion(
  current: RegionId | null,
  backward: boolean,
): RegionId {
  const count = REGIONS.length;
  const at = current === null ? (backward ? 0 : -1) : REGIONS.indexOf(current);
  const index = (at + (backward ? -1 : 1) + count) % count;
  return REGIONS[index] ?? "top";
}

/** The layouts of design-language §4.3, by window width in CSS pixels. */
export type Layout = "wide" | "medium" | "narrow";

/** Wide from 1280 px, Medium from 960 px, Narrow below (design-language §4.3). */
export function layoutFor(width: number): Layout {
  if (width >= 1280) {
    return "wide";
  }
  return width >= 960 ? "medium" : "narrow";
}

/** A side region's width limits and default, in CSS pixels (design-language §4.1). */
export const SIDE_WIDTH = {
  project: { min: 200, max: 400, initial: 264 },
  inspector: { min: 280, max: 480, initial: 320 },
} as const;
