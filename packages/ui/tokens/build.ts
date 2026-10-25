// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * Builds the token files from the recipe: resolves every colour into sRGB, checks the
 * design-language §5.3 pairs, and renders the CSS custom properties and the JSON record.
 * Pure functions; `generate.ts` writes the files and the tests compare them.
 */

import {
  contrastRatio,
  formatOklch,
  gamutMap,
  inGamut,
  oklchToSrgb,
  parseOklch,
  srgbToOklab,
  oklabToOklch,
  toHex,
} from "./color.ts";
import type { Oklch } from "./color.ts";
import {
  COLOR_TOKENS,
  FORCED_COLORS,
  INPUTS,
  MOTION_TOKENS,
  PAIRS,
  RECIPE,
  SCHEME,
  SHADOW_OVERLAY,
  STATIC_TOKENS,
  THEMES,
} from "./recipe.ts";
import type { ColorToken, ThemeId } from "./recipe.ts";

/** One resolved colour token. */
export interface TokenValue {
  /** The emitted CSS value, inside sRGB. */
  readonly css: string;
  /** The sRGB colour a browser renders for `css`. */
  readonly hex: string;
  /** Whether the recipe colour was outside sRGB and its chroma was reduced. */
  readonly gamutMapped: boolean;
}

export interface Theme {
  readonly id: ThemeId;
  readonly scheme: "dark" | "light";
  readonly colors: Readonly<Record<ColorToken, TokenValue>>;
  readonly shadowOverlay: string;
}

export interface PairResult {
  readonly theme: ThemeId;
  readonly foreground: ColorToken;
  readonly background: ColorToken;
  /** WCAG 2.2 contrast ratio of the rendered colours. */
  readonly ratio: number;
  readonly minimum: number;
}

/** The step by which an emitted chroma is lowered until the rounded value is in gamut. */
const CHROMA_STEP = 0.0001;

/** The CSS value of `color`: as written when in sRGB, else gamut-mapped and rounded inside. */
export function resolve(color: Oklch): TokenValue {
  if (inGamut(oklchToSrgb(color))) {
    const css = formatOklch(color);
    return {
      css,
      hex: toHex(oklchToSrgb(parseOklch(css))),
      gamutMapped: false,
    };
  }
  const mapped = oklabToOklch(srgbToOklab(gamutMap(color)));
  let emitted = parseOklch(formatOklch(mapped));
  while (!inGamut(oklchToSrgb(emitted)) && emitted.c > 0) {
    emitted = parseOklch(
      formatOklch({ ...emitted, c: Math.max(0, emitted.c - CHROMA_STEP) }),
    );
  }
  return {
    css: formatOklch(emitted),
    hex: toHex(oklchToSrgb(emitted)),
    gamutMapped: true,
  };
}

function mapTokens<T>(
  value: (token: ColorToken) => T,
): Readonly<Record<ColorToken, T>> {
  return Object.fromEntries(
    COLOR_TOKENS.map((token) => [token, value(token)]),
  ) as Record<ColorToken, T>;
}

export function buildThemes(): readonly Theme[] {
  return THEMES.map((id) => ({
    id,
    scheme: SCHEME[id],
    colors: mapTokens((token) => resolve(RECIPE[token][id])),
    shadowOverlay: SHADOW_OVERLAY[id],
  }));
}

/** Every §5.3 pair in every theme, with its rendered contrast ratio. */
export function checkPairs(themes: readonly Theme[]): readonly PairResult[] {
  return themes.flatMap((theme) =>
    PAIRS.flatMap((rule) =>
      rule.foregrounds.flatMap((foreground) =>
        rule.backgrounds.map((background) => ({
          theme: theme.id,
          foreground,
          background,
          ratio: contrastRatio(
            theme.colors[foreground].hex,
            theme.colors[background].hex,
          ),
          minimum: rule.minimum,
        })),
      ),
    ),
  );
}

/**
 * Which `data-theme` values a selection rule matches: a theme ID, or `os` for every other
 * value (`system`, no attribute, or a value this version does not know), which follows the OS.
 */
export type ThemeAttribute = "os" | ThemeId;

/** A media condition the selection rules use. */
export type MediaCondition = "prefers-light" | "prefers-more-contrast";

/**
 * One theme-selection rule. The rules apply in order and a later match wins, so an
 * engine that does not support a media feature keeps the earlier theme.
 */
export interface SelectionRule {
  /** All must match; empty means always. */
  readonly media: readonly MediaCondition[];
  /** `any` matches every attribute value. */
  readonly attributes: "any" | readonly ThemeAttribute[];
  readonly theme: ThemeId;
}

/**
 * How the theme is chosen (design-language §5.4). The app sets `data-theme` on the root
 * element from the `theme` setting; `system` or no attribute follows the OS. On top of dark
 * or light, `prefers-contrast: more` selects the high-contrast theme of that scheme.
 */
export const SELECTION: readonly SelectionRule[] = [
  { media: [], attributes: "any", theme: "dark" },
  { media: ["prefers-light"], attributes: ["os"], theme: "light" },
  { media: [], attributes: ["light"], theme: "light" },
  {
    media: ["prefers-more-contrast"],
    attributes: ["os", "dark"],
    theme: "highContrastDark",
  },
  {
    media: ["prefers-more-contrast", "prefers-light"],
    attributes: ["os"],
    theme: "highContrastLight",
  },
  {
    media: ["prefers-more-contrast"],
    attributes: ["light"],
    theme: "highContrastLight",
  },
  { media: [], attributes: ["highContrastDark"], theme: "highContrastDark" },
  { media: [], attributes: ["highContrastLight"], theme: "highContrastLight" },
];

const MEDIA_QUERY: Readonly<Record<MediaCondition, string>> = {
  "prefers-light": "(prefers-color-scheme: light)",
  "prefers-more-contrast": "(prefers-contrast: more)",
};

function selector(attributes: SelectionRule["attributes"]): string {
  if (attributes === "any") {
    return ":where(:root)";
  }
  const explicit = (id: ThemeId): string => `[data-theme="${id}"]`;
  const parts = attributes.map((attribute) =>
    attribute === "os"
      ? `:root:not(${THEMES.map(explicit).join(", ")})`
      : `:root${explicit(attribute)}`,
  );
  return `:where(${parts.join(", ")})`;
}

function indent(lines: readonly string[], depth: number): string[] {
  const pad = "  ".repeat(depth);
  return lines.map((line) => (line === "" ? line : pad + line));
}

function block(head: string, body: readonly string[]): string[] {
  return [`${head} {`, ...indent(body, 1), "}"];
}

function themeDeclarations(theme: Theme): string[] {
  return [
    `color-scheme: ${theme.scheme};`,
    ...COLOR_TOKENS.map((token) => `--${token}: ${theme.colors[token].css};`),
    `--shadow-overlay: ${theme.shadowOverlay};`,
  ];
}

/** The generated stylesheet: custom properties in the `tokens` cascade layer. */
export function renderCss(themes: readonly Theme[]): string {
  const byId = new Map(themes.map((theme) => [theme.id, theme]));
  const themeLayer: string[] = [
    ...block(
      ":where(:root)",
      STATIC_TOKENS.map(([name, value]) => `--${name}: ${value};`),
    ),
  ];
  for (const rule of SELECTION) {
    const theme = byId.get(rule.theme);
    if (theme === undefined) {
      throw new Error(`no theme ${rule.theme}`);
    }
    const rules = block(selector(rule.attributes), themeDeclarations(theme));
    themeLayer.push(
      "",
      ...(rule.media.length === 0
        ? rules
        : block(
            `@media ${rule.media.map((m) => MEDIA_QUERY[m]).join(" and ")}`,
            rules,
          )),
    );
  }
  const systemLayer = [
    ...block(
      "@media (forced-colors: active)",
      block(":where(:root)", [
        ...COLOR_TOKENS.map((token) => `--${token}: ${FORCED_COLORS[token]};`),
        "--shadow-overlay: none;",
      ]),
    ),
    "",
    ...block(
      "@media (prefers-reduced-motion: reduce)",
      block(
        ":where(:root)",
        MOTION_TOKENS.map((token) => `--${token}: 0ms;`),
      ),
    ),
  ];
  const lines = [
    // The header of the generated file, not of this one.
    // REUSE-IgnoreStart
    "/* SPDX-FileCopyrightText: 2026 SONARITHM contributors */",
    "/* SPDX-License-Identifier: MIT OR Apache-2.0 */",
    // REUSE-IgnoreEnd
    "",
    "/*",
    " * Generated by `pnpm -C packages/ui tokens` from packages/ui/tokens/recipe.ts. Do not edit.",
    " * Design tokens of design-language §5, §7, §8 and §9. The `theme` sublayer picks a theme",
    " * from the root element's `data-theme` attribute and the OS preferences; the `system`",
    " * sublayer, later and so stronger, applies forced colours and reduced motion.",
    " */",
    "",
    ...block("@layer tokens", [
      "@layer theme, system;",
      "",
      ...block("@layer theme", themeLayer),
      "",
      ...block("@layer system", systemLayer),
    ]),
  ];
  return `${lines.join("\n")}\n`;
}

function round(value: number, digits: number): number {
  return Number(value.toFixed(digits));
}

/** The JSON record: inputs, every resolved token and every pair with its ratio. */
export function renderJson(
  themes: readonly Theme[],
  pairs: readonly PairResult[],
): string {
  const record = {
    generatedBy: "pnpm -C packages/ui tokens (packages/ui/tokens/recipe.ts)",
    inputs: INPUTS,
    themes: Object.fromEntries(
      themes.map((theme) => [
        theme.id,
        {
          scheme: theme.scheme,
          colors: theme.colors,
          shadowOverlay: theme.shadowOverlay,
        },
      ]),
    ),
    forcedColors: FORCED_COLORS,
    contrast: pairs.map((pair) => ({ ...pair, ratio: round(pair.ratio, 2) })),
  };
  return `${JSON.stringify(record, null, 2)}\n`;
}

/** Everything `generate.ts` writes, plus the pairs below their minimum. */
export function generate(): {
  readonly css: string;
  readonly json: string;
  readonly failures: readonly PairResult[];
} {
  const themes = buildThemes();
  const pairs = checkPairs(themes);
  return {
    css: renderCss(themes),
    json: renderJson(themes, pairs),
    failures: pairs.filter((pair) => pair.ratio < pair.minimum),
  };
}
