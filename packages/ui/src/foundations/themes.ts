// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/** The values of the `theme` setting that select a theme (design-language §5.4). */
export const THEMES = [
  "dark",
  "light",
  "highContrastDark",
  "highContrastLight",
] as const;

/** One of {@link THEMES}. */
export type Theme = (typeof THEMES)[number];
