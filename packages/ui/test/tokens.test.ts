// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// The generated design tokens (B02-T01): the committed files match the generator, every
// design-language §5.3 pair meets WCAG 2.2 in every theme, the themes follow the setting and
// the OS, and forced colours and reduced motion are handled (A11Y-25, FR-117, DL-X02, DL-X03,
// DL-X07).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  SELECTION,
  buildThemes,
  checkPairs,
  generate,
  renderCss,
} from "../tokens/build.ts";
import type { MediaCondition, ThemeAttribute } from "../tokens/build.ts";
import {
  contrastRatio,
  inGamut,
  oklchToSrgb,
  parseOklch,
  toHex,
} from "../tokens/color.ts";
import { generatedFiles } from "../tokens/generate.ts";
import {
  COLOR_TOKENS,
  FORCED_COLORS,
  MOTION_TOKENS,
  PAIRS,
  SHADOW_OVERLAY,
  STATIC_TOKENS,
  THEMES,
} from "../tokens/recipe.ts";
import type { ThemeId } from "../tokens/recipe.ts";

const themes = buildThemes();

/** The colour a browser renders for an emitted value, computed again from the CSS text. */
function rendered(css: string): string {
  const color = parseOklch(css);
  assert.ok(inGamut(oklchToSrgb(color)), `${css} is outside sRGB`);
  return toHex(oklchToSrgb(color));
}

function theme(id: ThemeId): (typeof themes)[number] {
  const found = themes.find((t) => t.id === id);
  assert.ok(found, id);
  return found;
}

test("generated files are committed and current", () => {
  assert.deepEqual(generate().failures, []);
  const hint = "run `pnpm -C packages/ui tokens` and commit the result";
  for (const [file, contents] of generatedFiles()) {
    assert.equal(
      readFileSync(file, "utf8"),
      contents,
      `${file.pathname}: ${hint}`,
    );
  }
});

function checkMinimum(minimum: 4.5 | 3): void {
  const rules = PAIRS.filter((rule) => rule.minimum === minimum);
  assert.ok(rules.length > 0);
  for (const t of themes) {
    for (const rule of rules) {
      for (const fg of rule.foregrounds) {
        for (const bg of rule.backgrounds) {
          const ratio = contrastRatio(
            rendered(t.colors[fg].css),
            rendered(t.colors[bg].css),
          );
          assert.ok(
            ratio >= minimum,
            `${t.id}: ${fg} on ${bg} is ${ratio.toFixed(2)}:1, below ${String(minimum)}:1`,
          );
        }
      }
    }
  }
}

test("a11y_25 text pairs meet 4.5 to 1 in every theme", () => {
  checkMinimum(4.5);
});

test("a11y_25 non-text and focus pairs meet 3 to 1 in every theme", () => {
  checkMinimum(3);
});

test("a11y_25 high contrast never lowers a pair", () => {
  const pairs = checkPairs(themes);
  const ratio = (id: ThemeId, fg: string, bg: string): number => {
    const pair = pairs.find(
      (p) => p.theme === id && p.foreground === fg && p.background === bg,
    );
    assert.ok(pair);
    return pair.ratio;
  };
  const counterparts: readonly [ThemeId, ThemeId][] = [
    ["dark", "highContrastDark"],
    ["light", "highContrastLight"],
  ];
  for (const [standard, high] of counterparts) {
    for (const pair of pairs.filter((p) => p.theme === standard)) {
      const raised = ratio(high, pair.foreground, pair.background);
      assert.ok(
        raised >= pair.ratio,
        `${high}: ${pair.foreground} on ${pair.background} is ${raised.toFixed(2)}:1, below ${standard}'s ${pair.ratio.toFixed(2)}:1`,
      );
    }
  }
});

test("dl_x07 default themes avoid pure black and pure white", () => {
  for (const id of ["dark", "light"] as const) {
    for (const token of COLOR_TOKENS) {
      const hex = theme(id).colors[token].hex;
      assert.ok(hex !== "#000000" && hex !== "#FFFFFF", `${id} ${token}`);
    }
  }
});

test("dl_x02 every colour token is opaque", () => {
  for (const t of themes) {
    for (const token of COLOR_TOKENS) {
      assert.match(
        t.colors[token].css,
        /^oklch\([\d.]+ [\d.]+ [\d.]+\)$/,
        `${t.id} ${token} has no alpha`,
      );
    }
  }
});

test("dl_x03 radii stay at or below 4px and the overlay shadow is neutral", () => {
  const radii = STATIC_TOKENS.filter(([name]) => name.startsWith("radius-"));
  assert.deepEqual(
    radii.map(([name]) => name),
    ["radius-0", "radius-1", "radius-2"],
  );
  for (const [, value] of radii) {
    assert.ok(Number.parseFloat(value) <= 4, value);
  }
  for (const shadow of Object.values(SHADOW_OVERLAY)) {
    // Coloured shadows are glow (DL-X01); only a neutral shadow or none.
    assert.match(shadow, /^(none|0 \d+px \d+px oklch\(0 0 0 \/ [\d.]+\))$/);
  }
});

test("fr_117 every theme defines every token inside srgb", () => {
  assert.deepEqual(
    themes.map((t) => t.id),
    [...THEMES],
  );
  for (const t of themes) {
    assert.deepEqual(Object.keys(t.colors), [...COLOR_TOKENS]);
    for (const token of COLOR_TOKENS) {
      assert.equal(rendered(t.colors[token].css), t.colors[token].hex);
    }
  }
});

test("fr_117 theme ids match the app's theme setting", () => {
  const protocol = readFileSync(
    new URL("../../ipc/src/generated/protocol.ts", import.meta.url),
    "utf8",
  );
  const match = /export type Theme = ([^;]+);/.exec(protocol);
  assert.ok(match?.[1], "Theme in the generated IPC types");
  const values = [...match[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
  assert.deepEqual(values, ["system", ...THEMES]);
});

/** A `data-theme` value as the app could set it; `null` is no attribute. */
type DataTheme = string | null;

function isThemeId(value: DataTheme): value is ThemeId {
  return (THEMES as readonly (string | null)[]).includes(value);
}

/** The theme the selection rules pick: the last matching rule wins. */
function pick(
  dataTheme: DataTheme,
  media: ReadonlySet<MediaCondition>,
): ThemeId | undefined {
  const attribute: ThemeAttribute = isThemeId(dataTheme) ? dataTheme : "os";
  let picked: ThemeId | undefined;
  for (const rule of SELECTION) {
    const attributeMatches =
      rule.attributes === "any" || rule.attributes.includes(attribute);
    if (attributeMatches && rule.media.every((m) => media.has(m))) {
      picked = rule.theme;
    }
  }
  return picked;
}

/** design-language §5.4: the explicit theme wins; dark and light gain high contrast. */
function expected(
  dataTheme: DataTheme,
  media: ReadonlySet<MediaCondition>,
): ThemeId {
  if (dataTheme === "highContrastDark" || dataTheme === "highContrastLight") {
    return dataTheme;
  }
  const light =
    dataTheme === "light" ||
    (dataTheme !== "dark" && media.has("prefers-light"));
  if (media.has("prefers-more-contrast")) {
    return light ? "highContrastLight" : "highContrastDark";
  }
  return light ? "light" : "dark";
}

test("fr_117 the theme follows the setting and the os", () => {
  // No attribute, the `system` setting and a value this version does not know follow the OS.
  const attributes: readonly DataTheme[] = [null, "system", "sepia", ...THEMES];
  const conditions: readonly MediaCondition[][] = [
    [],
    ["prefers-light"],
    ["prefers-more-contrast"],
    ["prefers-light", "prefers-more-contrast"],
  ];
  for (const attribute of attributes) {
    for (const condition of conditions) {
      const media = new Set(condition);
      assert.equal(
        pick(attribute, media),
        expected(attribute, media),
        `${attribute ?? "no attribute"} with ${condition.join(", ") || "no preference"}`,
      );
    }
  }
});

test("fr_117 every selection rule is in the stylesheet, in order", () => {
  const css = renderCss(themes);
  let from = css.indexOf("@layer theme {");
  assert.ok(from > 0);
  for (const rule of SELECTION) {
    const head =
      rule.media.length === 0
        ? rule.attributes === "any"
          ? ":where(:root) {\n      color-scheme"
          : `:where(:root[data-theme="${String(rule.attributes[0])}"])`
        : "@media (prefers-";
    const at = css.indexOf(head, from);
    assert.ok(at > from, `rule for ${rule.theme} after offset ${String(from)}`);
    from = at + 1;
  }
  assert.ok(from < css.indexOf("@layer system {"));
});

test("a11y_25 forced colours map every token to a system colour", () => {
  const systemColors = new Set([
    "Canvas",
    "CanvasText",
    "GrayText",
    "Highlight",
    "HighlightText",
    "ButtonFace",
    "ButtonText",
    "ButtonBorder",
  ]);
  assert.deepEqual(Object.keys(FORCED_COLORS), [...COLOR_TOKENS]);
  for (const value of Object.values(FORCED_COLORS)) {
    assert.ok(systemColors.has(value), value);
  }
  const css = renderCss(themes);
  const system = css.slice(css.indexOf("@layer system {"));
  assert.match(system, /@media \(forced-colors: active\)/);
  for (const token of COLOR_TOKENS) {
    assert.ok(system.includes(`--${token}: ${FORCED_COLORS[token]};`), token);
  }
  assert.ok(system.includes("--shadow-overlay: none;"));
});

test("a11y_25 reduced motion sets every duration to zero", () => {
  const durations = STATIC_TOKENS.filter(([, value]) =>
    value.endsWith("ms"),
  ).map(([name]) => name);
  assert.deepEqual(durations, [...MOTION_TOKENS]);
  const css = renderCss(themes);
  const reduced = css.slice(
    css.indexOf("@media (prefers-reduced-motion: reduce)"),
  );
  for (const token of MOTION_TOKENS) {
    assert.ok(reduced.includes(`--${token}: 0ms;`), token);
  }
  for (const [, value] of STATIC_TOKENS.filter(([n]) =>
    n.startsWith("motion-"),
  )) {
    // DL-P09: 120 ms is the maximum.
    assert.ok(Number.parseFloat(value) <= 120, value);
  }
});
