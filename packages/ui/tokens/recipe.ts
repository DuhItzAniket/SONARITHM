// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * The token recipe (design-language §5, §7, §8, §9): the only place with raw colour, size and
 * duration values (coding-standards §12.6). `generate.ts` turns it into
 * `src/generated/tokens.css` and `src/generated/tokens.json`.
 *
 * Themes come from three inputs (§5.1): the base (a warm neutral hue at very low chroma), the
 * accent (hue and chroma) and the contrast level. The contrast level picks the lightness
 * ladder: at high contrast the surfaces sit closer to the extreme, text and borders move to
 * the other end, neutrals lose their chroma and the accent gains chroma. Every value is a
 * proposal until the B02 design review (B02-T09); the contrast pairs below are the gate.
 */

import type { Oklch } from "./color.ts";

/** Theme IDs: the values of the app's `Theme` setting other than `system` (IPC `Theme`). */
export const THEMES = [
  "dark",
  "light",
  "highContrastDark",
  "highContrastLight",
] as const;

export type ThemeId = (typeof THEMES)[number];

/** The colour scheme of each theme, for `color-scheme` and the selection rules. */
export const SCHEME: Readonly<Record<ThemeId, "dark" | "light">> = {
  dark: "dark",
  light: "light",
  highContrastDark: "dark",
  highContrastLight: "light",
};

export const SURFACES = [
  "surface-0",
  "surface-1",
  "surface-2",
  "surface-3",
] as const;

export const SIGNALS = [
  "signal-playhead",
  "signal-warning",
  "signal-error",
  "signal-ok",
] as const;

/** The categorical series colours of design-language §6, in Okabe–Ito order. */
export const SERIES = [
  "series-1",
  "series-2",
  "series-3",
  "series-4",
  "series-5",
  "series-6",
  "series-7",
  "series-8",
] as const;

/** The semantic colour tokens of design-language §5.2, in table order, then the series (§6). */
export const COLOR_TOKENS = [
  ...SURFACES,
  "text-1",
  "text-2",
  "text-3",
  "border",
  "border-strong",
  "accent",
  "focus",
  ...SIGNALS,
  ...SERIES,
] as const;

export type ColorToken = (typeof COLOR_TOKENS)[number];

/** The generator's inputs (§5.1). */
export const INPUTS = {
  /** Warm neutral; chroma at or below 0.006 so surfaces read as gray. */
  base: { hue: 85, chroma: 0.004 },
  /** Teal-cyan: not indigo or purple (DL-X01), clear of the signal hues. */
  accent: { hue: 195, chroma: 0.095 },
} as const;

/** A neutral at lightness `l`: the base hue and chroma. */
function neutral(l: number): Oklch {
  return { l, c: INPUTS.base.chroma, h: INPUTS.base.hue };
}

/** An achromatic neutral, for high contrast. */
function gray(l: number): Oklch {
  return { l, c: 0, h: 0 };
}

/** An accent colour at lightness `l`, with the accent chroma scaled by `scale`. */
function accent(l: number, scale: number): Oklch {
  return { l, c: INPUTS.accent.chroma * scale, h: INPUTS.accent.hue };
}

/** A colour with its own hue: signals, and the high-contrast focus rings. */
function fixed(l: number, c: number, h: number): Oklch {
  return { l, c, h };
}

/**
 * The colour of each token in each theme. Colours outside sRGB are gamut-mapped by the
 * generator (CSS Color 4), so the emitted value can have less chroma than written here.
 */
export const RECIPE: Readonly<
  Record<ColorToken, Readonly<Record<ThemeId, Oklch>>>
> = {
  "surface-0": {
    dark: neutral(0.185),
    light: neutral(0.97),
    highContrastDark: gray(0.14),
    highContrastLight: gray(1),
  },
  "surface-1": {
    dark: neutral(0.215),
    light: neutral(0.99),
    highContrastDark: gray(0.14),
    highContrastLight: gray(1),
  },
  "surface-2": {
    dark: neutral(0.255),
    light: neutral(0.95),
    highContrastDark: gray(0.2),
    highContrastLight: gray(0.96),
  },
  "surface-3": {
    dark: neutral(0.3),
    light: neutral(0.915),
    highContrastDark: gray(0.28),
    highContrastLight: gray(0.91),
  },
  "text-1": {
    dark: neutral(0.93),
    light: neutral(0.23),
    highContrastDark: gray(0.99),
    highContrastLight: gray(0.14),
  },
  "text-2": {
    dark: neutral(0.79),
    light: neutral(0.4),
    highContrastDark: gray(0.95),
    highContrastLight: gray(0.22),
  },
  "text-3": {
    dark: neutral(0.69),
    light: neutral(0.49),
    highContrastDark: gray(0.9),
    highContrastLight: gray(0.3),
  },
  border: {
    dark: neutral(0.33),
    light: neutral(0.88),
    highContrastDark: gray(0.82),
    highContrastLight: gray(0.4),
  },
  "border-strong": {
    dark: neutral(0.57),
    light: neutral(0.58),
    highContrastDark: gray(0.95),
    highContrastLight: gray(0.22),
  },
  accent: {
    dark: accent(0.75, 1),
    light: accent(0.5, 0.9),
    highContrastDark: accent(0.87, 1.25),
    highContrastLight: accent(0.42, 1),
  },
  // The standard rings follow the accent; they are told apart from it by the 2 px offset in
  // the surface colour (§5.2). High contrast uses a hue nothing else uses.
  focus: {
    dark: accent(0.83, 1.15),
    light: accent(0.45, 0.95),
    highContrastDark: fixed(0.94, 0.15, 105),
    highContrastLight: fixed(0.42, 0.19, 262),
  },
  "signal-playhead": {
    dark: fixed(0.96, 0.02, 95),
    light: fixed(0.26, 0.01, 85),
    highContrastDark: gray(0.99),
    highContrastLight: gray(0.14),
  },
  "signal-warning": {
    dark: fixed(0.8, 0.14, 70),
    light: fixed(0.57, 0.115, 70),
    highContrastDark: fixed(0.86, 0.14, 75),
    highContrastLight: fixed(0.5, 0.12, 70),
  },
  "signal-error": {
    dark: fixed(0.7, 0.17, 25),
    light: fixed(0.53, 0.18, 25),
    highContrastDark: fixed(0.8, 0.12, 25),
    highContrastLight: fixed(0.48, 0.18, 25),
  },
  "signal-ok": {
    dark: fixed(0.77, 0.14, 150),
    light: fixed(0.53, 0.125, 150),
    highContrastDark: fixed(0.86, 0.16, 150),
    highContrastLight: fixed(0.46, 0.13, 150),
  },
  // Series (§6, B04-T01): Okabe–Ito with only lightness changed, written out by
  // `node tokens/series-print.ts` (the derivation in `series.ts`, checked by a test): at least 3:1 on the plot
  // backgrounds (4.5:1 in high contrast) and apart under every dichromacy.
  // Okabe–Ito orange.
  "series-1": {
    dark: fixed(0.753, 0.1576, 76.77),
    light: fixed(0.418, 0.1576, 76.77),
    highContrastDark: fixed(0.753, 0.1576, 76.77),
    highContrastLight: fixed(0.418, 0.1576, 76.77),
  },
  // Okabe–Ito sky blue.
  "series-2": {
    dark: fixed(0.735, 0.1174, 236.18),
    light: fixed(0.64, 0.1174, 236.18),
    highContrastDark: fixed(0.735, 0.1174, 236.18),
    highContrastLight: fixed(0.56, 0.1174, 236.18),
  },
  // Okabe–Ito bluish green.
  "series-3": {
    dark: fixed(0.62, 0.1295, 165.46),
    light: fixed(0.62, 0.1295, 165.46),
    highContrastDark: fixed(0.62, 0.1295, 165.46),
    highContrastLight: fixed(0.535, 0.1295, 165.46),
  },
  // Okabe–Ito yellow.
  "series-4": {
    dark: fixed(0.902, 0.1721, 105.04),
    light: fixed(0.357, 0.1721, 105.04),
    highContrastDark: fixed(0.902, 0.1721, 105.04),
    highContrastLight: fixed(0.352, 0.1721, 105.04),
  },
  // Okabe–Ito blue.
  "series-5": {
    dark: fixed(0.532, 0.1313, 244.05),
    light: fixed(0.532, 0.1313, 244.05),
    highContrastDark: fixed(0.572, 0.1313, 244.05),
    highContrastLight: fixed(0.512, 0.1313, 244.05),
  },
  // Okabe–Ito vermillion.
  "series-6": {
    dark: fixed(0.621, 0.1705, 47.51),
    light: fixed(0.381, 0.1705, 47.51),
    highContrastDark: fixed(0.621, 0.1705, 47.51),
    highContrastLight: fixed(0.376, 0.1705, 47.51),
  },
  // Okabe–Ito reddish purple.
  "series-7": {
    dark: fixed(0.679, 0.1177, 346.32),
    light: fixed(0.659, 0.1177, 346.32),
    highContrastDark: fixed(0.679, 0.1177, 346.32),
    highContrastLight: fixed(0.579, 0.1177, 346.32),
  },
  // Okabe–Ito black, as the theme's neutral text-2 (DL-X07).
  "series-8": {
    dark: fixed(0.79, 0.0036, 67.77),
    light: fixed(0.399, 0.0043, 67.7),
    highContrastDark: fixed(0.949, 0, 89.88),
    highContrastLight: fixed(0.222, 0, 89.88),
  },
};

/**
 * The overlay shadow (§8): the only shadow token, neutral, for menus, popovers, tooltips and
 * dialogs. High contrast relies on the `border-strong` outline instead.
 */
export const SHADOW_OVERLAY: Readonly<Record<ThemeId, string>> = {
  dark: "0 2px 8px oklch(0 0 0 / 0.4)",
  light: "0 2px 8px oklch(0 0 0 / 0.15)",
  highContrastDark: "none",
  highContrastLight: "none",
};

/** A contrast requirement from design-language §5.3. */
export interface PairRule {
  readonly foregrounds: readonly ColorToken[];
  readonly backgrounds: readonly ColorToken[];
  /** WCAG 2.2: 4.5 for text, 3 for non-text UI, graphical objects and focus indicators. */
  readonly minimum: 4.5 | 3;
}

/** The pair list of design-language §5.3, checked in every theme. */
export const PAIRS: readonly PairRule[] = [
  { foregrounds: ["text-1", "text-2"], backgrounds: SURFACES, minimum: 4.5 },
  // Tertiary text is kept off surface-3 for margin (§5.3).
  {
    foregrounds: ["text-3"],
    backgrounds: ["surface-0", "surface-1", "surface-2"],
    minimum: 4.5,
  },
  { foregrounds: ["border-strong"], backgrounds: SURFACES, minimum: 3 },
  { foregrounds: ["focus"], backgrounds: SURFACES, minimum: 3 },
  { foregrounds: ["accent"], backgrounds: SURFACES, minimum: 3 },
  // Signals are drawn as marks with an icon or text; signal text uses text-1 (§5.3).
  { foregrounds: SIGNALS, backgrounds: SURFACES, minimum: 3 },
  // Series marks on the plot backgrounds (§6; WCAG 1.4.11). High contrast aims at 4.5:1.
  {
    foregrounds: SERIES,
    backgrounds: ["surface-0", "surface-1"],
    minimum: 3,
  },
];

/**
 * Forced colours (§5.4): every colour token as a CSS system colour. Signals lose their hue;
 * each one already carries an icon or text (DL-X08).
 */
export const FORCED_COLORS: Readonly<Record<ColorToken, string>> = {
  "surface-0": "Canvas",
  "surface-1": "Canvas",
  "surface-2": "Canvas",
  "surface-3": "Canvas",
  "text-1": "CanvasText",
  "text-2": "CanvasText",
  "text-3": "CanvasText",
  border: "CanvasText",
  "border-strong": "CanvasText",
  accent: "Highlight",
  focus: "Highlight",
  "signal-playhead": "Highlight",
  "signal-warning": "CanvasText",
  "signal-error": "CanvasText",
  "signal-ok": "CanvasText",
  // Plots in forced colours tell series apart by dash, marker and label (§6).
  "series-1": "CanvasText",
  "series-2": "CanvasText",
  "series-3": "CanvasText",
  "series-4": "CanvasText",
  "series-5": "CanvasText",
  "series-6": "CanvasText",
  "series-7": "CanvasText",
  "series-8": "CanvasText",
};

/**
 * Tokens that do not change with the theme: spacing and sizing (§7.2), radii (§7.3), lines
 * (§5.2, §8) and motion (§9). Sizes that hold text are in rem, so text scaling and zoom
 * apply (A11Y-25); 1 rem is 16 px at the default scale.
 */
export const STATIC_TOKENS: readonly (readonly [string, string])[] = [
  ["space-1", "0.25rem"],
  ["space-2", "0.5rem"],
  ["space-3", "0.75rem"],
  ["space-4", "1rem"],
  ["space-5", "1.5rem"],
  ["space-6", "2rem"],
  ["row-compact", "1.5rem"],
  ["row-default", "1.75rem"],
  ["bar-toolbar", "2.5rem"],
  // WCAG 2.5.8 target size (A11Y-26).
  ["target-min", "1.5rem"],
  // Nothing exceeds radius-2 (DL-X03).
  ["radius-0", "0"],
  ["radius-1", "2px"],
  ["radius-2", "4px"],
  ["border-width", "1px"],
  // Focus rings: at least 2 px with a 2 px offset (DL-P05, A11Y-05).
  ["focus-ring-width", "2px"],
  ["focus-ring-offset", "2px"],
  // 120 ms is the maximum (DL-P09).
  ["motion-fast", "80ms"],
  ["motion-base", "120ms"],
  // Typography (§7.1): IBM Plex, bundled by fonts.css; the fallbacks only cover a failed load.
  ["font-sans", '"IBM Plex Sans", system-ui, sans-serif'],
  ["font-mono", '"IBM Plex Mono", ui-monospace, monospace'],
  // The type scale, size over line height in px: 11/16, 12/16, 13/20, 15/20, 18/24.
  ["type-11-size", "0.6875rem"],
  ["type-11-line", "1rem"],
  ["type-12-size", "0.75rem"],
  ["type-12-line", "1rem"],
  ["type-13-size", "0.8125rem"],
  ["type-13-line", "1.25rem"],
  ["type-15-size", "0.9375rem"],
  ["type-15-line", "1.25rem"],
  ["type-18-size", "1.125rem"],
  ["type-18-line", "1.5rem"],
  // No light or thin weights: they break up on dark surfaces (§7.1).
  ["weight-regular", "400"],
  ["weight-medium", "500"],
  ["weight-semibold", "600"],
  // Figures for numbers (§7.1, DL-P04): Plex's default figures are already tabular (F-132).
  ["numeric", "tabular-nums lining-nums slashed-zero"],
];

/** Motion tokens set to 0 ms under `prefers-reduced-motion: reduce` (§9). */
export const MOTION_TOKENS = ["motion-fast", "motion-base"] as const;
