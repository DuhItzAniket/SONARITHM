// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * The derivation of the categorical series colours (design-language §6; B04-T01). It starts
 * from the Okabe–Ito palette (Okabe and Ito, *Color Universal Design*, 2008) and changes
 * only lightness in OKLCH, so every series keeps its hue and the hue order:
 *
 * 1. Each colour moves away from the plot backgrounds (`surface-0` and `surface-1`) until it
 *    has its contrast target: 3:1 in the standard themes (WCAG 2.2 1.4.11, graphical
 *    objects) and 4.5:1 in high contrast, where the user chose maximum contrast. A
 *    high-contrast series never has less contrast than its standard counterpart. Black is
 *    the theme's neutral `text-2` in every theme: dark themes need it for contrast (§6), and
 *    DL-X07 rules out pure black in the default light theme.
 * 2. While the closest pair of series under any of protanopia, deuteranopia and tritanopia
 *    (and normal vision) is nearer than {@link SERIES_SEPARATION}, one member of that pair
 *    moves a further {@link STEP} away from the background, which only raises its contrast.
 *    The move that separates the pairs most is taken; when no single step helps at once,
 *    the member already farther from the background moves, widening the lightness gap.
 *
 * Achromatopsia is not a goal: eight hues cannot all differ in lightness at 3:1, and every
 * series also has its own dash pattern, marker and direct label (§6, DL-X08). The recipe
 * holds the result; `node tokens/series-print.ts` prints it, and a test checks that the two agree.
 */

import type { Oklch } from "./color.ts";
import { resolve } from "./build.ts";
import { contrastRatio, oklabToOklch, srgbToOklab } from "./color.ts";
import { deltaEOK, parseHex, simulate } from "./cvd.ts";

/** The Okabe–Ito palette in its published order. */
export const OKABE_ITO = [
  "#E69F00",
  "#56B4E9",
  "#009E73",
  "#F0E442",
  "#0072B2",
  "#D55E00",
  "#CC79A7",
  "#000000",
] as const;

/** The deficiencies the series must stay apart under. */
const DICHROMACIES = ["protanopia", "deuteranopia", "tritanopia"] as const;

/**
 * The smallest OKLab distance between two series, with or without a dichromacy: 1.5 times
 * the about 0.02 that is just noticeable. A proposal (INFER) until checked with CVD viewers;
 * 0.04 is not reachable in the light theme without pushing colours to black.
 */
export const SERIES_SEPARATION = 0.03;

/** The lightness step of the separation loop. */
const STEP = 0.01;

/** What the derivation needs to know about a theme. */
export interface SeriesTheme {
  readonly dark: boolean;
  readonly highContrast: boolean;
  /** The plot backgrounds, as `#RRGGBB`. */
  readonly backgrounds: readonly string[];
  /** The theme's `text-2`, which replaces black. */
  readonly neutral: string;
  /**
   * The least contrast each series must have, from the standard counterpart of a
   * high-contrast theme, so high contrast never lowers a pair.
   */
  readonly floors?: readonly number[];
}

/** The colour the generator emits for `color`, so every check sees the rendered value. */
const hexOf = (color: Oklch): string => resolve(color).hex;
const toOklch = (hex: string): Oklch =>
  oklabToOklch(srgbToOklab(parseHex(hex)));

/** The contrast target of a theme's series. */
export function seriesTarget(theme: SeriesTheme): number {
  return theme.highContrast ? 4.5 : 3;
}

/** The closest pair of `hexes` under normal vision or any dichromacy: distance and pair. */
export function closestSeries(hexes: readonly string[]): {
  readonly distance: number;
  readonly pair: readonly [number, number];
} {
  let distance = Number.POSITIVE_INFINITY;
  let pair: [number, number] = [0, 0];
  for (const deficiency of [null, ...DICHROMACIES]) {
    const seen = hexes.map((hex) =>
      deficiency === null ? hex : simulate(hex, deficiency),
    );
    for (let a = 0; a < seen.length; a += 1) {
      for (let b = a + 1; b < seen.length; b += 1) {
        const d = deltaEOK(seen[a] ?? "", seen[b] ?? "");
        if (d < distance) {
          distance = d;
          pair = [a, b];
        }
      }
    }
  }
  return { distance, pair };
}

/** The series colours of a theme, as OKLCH with the source chroma (the generator gamut-maps). */
export function deriveSeries(theme: SeriesTheme): Oklch[] {
  const target = seriesTarget(theme);
  const away = theme.dark ? 1 : -1;
  const ratio = (color: Oklch) =>
    Math.min(...theme.backgrounds.map((bg) => contrastRatio(hexOf(color), bg)));
  const colors = OKABE_ITO.map((hex, i) => {
    const start = toOklch(i === 7 ? theme.neutral : hex);
    const goal = Math.max(target, theme.floors?.[i] ?? 0);
    let l = round(start.l);
    // A margin, so rounding in the emitted value cannot drop it under the target.
    while (ratio({ ...start, l }) < goal + 0.02 && l > 0 && l < 1) {
      l = round(l + away * 0.005);
    }
    return { ...start, l };
  });
  for (let iteration = 0; iteration < 200; iteration += 1) {
    const { distance, pair } = closestSeries(colors.map(hexOf));
    if (distance >= SERIES_SEPARATION) {
      break;
    }
    let best: number | null = null;
    let bestDistance = distance;
    for (const k of pair) {
      const current = colors[k];
      if (current === undefined) continue;
      const l = round(current.l + away * STEP);
      if (l <= 0.05 || l >= 0.97) continue;
      const trial = colors.map((c, i) => (i === k ? { ...c, l } : c));
      const moved = closestSeries(trial.map(hexOf)).distance;
      if (moved > bestDistance) {
        bestDistance = moved;
        best = k;
      }
    }
    if (best === null) {
      // No single step widens the pair at once: move the member already farther from
      // the background, which widens the pair's lightness gap.
      const [a, b] = pair;
      const la = colors[a]?.l ?? 0;
      const lb = colors[b]?.l ?? 0;
      const farther = (la - lb) * away > 0 ? a : b;
      const l = round((colors[farther]?.l ?? 0) + away * STEP);
      if (l <= 0.05 || l >= 0.97) {
        break;
      }
      best = farther;
    }
    const chosen = colors[best];
    if (chosen !== undefined) {
      colors[best] = { ...chosen, l: round(chosen.l + away * STEP) };
    }
  }
  return colors;
}

function round(l: number): number {
  return Math.round(l * 1000) / 1000;
}

/** The least contrast of `color` on the theme's plot backgrounds. */
export function seriesContrast(theme: SeriesTheme, color: Oklch): number {
  return Math.min(
    ...theme.backgrounds.map((bg) => contrastRatio(hexOf(color), bg)),
  );
}

/** The series of every theme, standard themes first so high contrast can use their floors. */
export function deriveAll(
  themes: Readonly<
    Record<
      "dark" | "light" | "highContrastDark" | "highContrastLight",
      Omit<SeriesTheme, "floors">
    >
  >,
): Record<keyof typeof themes, Oklch[]> {
  const dark = deriveSeries(themes.dark);
  const light = deriveSeries(themes.light);
  // The highest standard ratio, so the high-contrast series is at least as strong on every
  // background as the standard one is on any.
  const floors = (base: SeriesTheme, colors: Oklch[]) =>
    colors.map((c) =>
      Math.max(...base.backgrounds.map((bg) => contrastRatio(hexOf(c), bg))),
    );
  return {
    dark,
    light,
    highContrastDark: deriveSeries({
      ...themes.highContrastDark,
      floors: floors(themes.dark, dark),
    }),
    highContrastLight: deriveSeries({
      ...themes.highContrastLight,
      floors: floors(themes.light, light),
    }),
  };
}

/** The derivation's view of the built themes (their plot backgrounds and `text-2`). */
export function seriesThemes(
  themes: readonly {
    readonly id: "dark" | "light" | "highContrastDark" | "highContrastLight";
    readonly scheme: "dark" | "light";
    readonly colors: Readonly<
      Record<"surface-0" | "surface-1" | "text-2", { readonly hex: string }>
    >;
  }[],
): Readonly<
  Record<
    "dark" | "light" | "highContrastDark" | "highContrastLight",
    Omit<SeriesTheme, "floors">
  >
> {
  const view = (
    id: "dark" | "light" | "highContrastDark" | "highContrastLight",
  ) => {
    const theme = themes.find((t) => t.id === id);
    if (theme === undefined) {
      throw new Error(`no ${id} theme`);
    }
    return {
      dark: theme.scheme === "dark",
      highContrast: id.startsWith("high"),
      backgrounds: [
        theme.colors["surface-0"].hex,
        theme.colors["surface-1"].hex,
      ],
      neutral: theme.colors["text-2"].hex,
    };
  };
  return {
    dark: view("dark"),
    light: view("light"),
    highContrastDark: view("highContrastDark"),
    highContrastLight: view("highContrastLight"),
  };
}
