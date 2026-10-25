// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * Colour arithmetic for the token generator (design-language §5.1): OKLCH and OKLab, sRGB,
 * gamut mapping and the WCAG 2.2 contrast ratio.
 *
 * OKLab uses Björn Ottosson's published sRGB matrices ("A perceptual color space for image
 * processing", 2020). Gamut mapping is the CSS Color Module Level 4 algorithm: a binary search
 * on chroma that keeps lightness and hue, accepting a clipped colour within a deltaEOK of 0.02.
 */

/** A colour in OKLCH: lightness 0–1, chroma, hue in degrees. */
export interface Oklch {
  readonly l: number;
  readonly c: number;
  readonly h: number;
}

/** A colour in OKLab. */
export interface Oklab {
  readonly l: number;
  readonly a: number;
  readonly b: number;
}

/** Gamma-encoded sRGB, nominally 0–1 per channel; values outside are out of gamut. */
export type Rgb = readonly [number, number, number];

const DEGREE = Math.PI / 180;

/** How far outside 0–1 a channel may be and still count as in gamut. */
const GAMUT_EPSILON = 0.000075;

/** CSS Color 4: the largest deltaEOK at which a clipped colour is accepted. */
const JND = 0.02;

/** CSS Color 4: the chroma resolution of the search. */
const SEARCH_EPSILON = 0.0001;

export function oklchToOklab(color: Oklch): Oklab {
  return {
    l: color.l,
    a: color.c * Math.cos(color.h * DEGREE),
    b: color.c * Math.sin(color.h * DEGREE),
  };
}

export function oklabToOklch(color: Oklab): Oklch {
  const c = Math.hypot(color.a, color.b);
  const h = c < 1e-9 ? 0 : (Math.atan2(color.b, color.a) / DEGREE + 360) % 360;
  return { l: color.l, c, h };
}

function encode(linear: number): number {
  const magnitude = Math.abs(linear);
  const encoded =
    magnitude <= 0.0031308
      ? 12.92 * magnitude
      : 1.055 * magnitude ** (1 / 2.4) - 0.055;
  return Math.sign(linear) * encoded;
}

function decode(encoded: number): number {
  const magnitude = Math.abs(encoded);
  const linear =
    magnitude <= 0.04045
      ? magnitude / 12.92
      : ((magnitude + 0.055) / 1.055) ** 2.4;
  return Math.sign(encoded) * linear;
}

export function oklabToSrgb(color: Oklab): Rgb {
  const l = (color.l + 0.3963377774 * color.a + 0.2158037573 * color.b) ** 3;
  const m = (color.l - 0.1055613458 * color.a - 0.0638541728 * color.b) ** 3;
  const s = (color.l - 0.0894841775 * color.a - 1.291485548 * color.b) ** 3;
  return [
    encode(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
    encode(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
    encode(-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s),
  ];
}

export function srgbToOklab(color: Rgb): Oklab {
  const r = decode(color[0]);
  const g = decode(color[1]);
  const b = decode(color[2]);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return {
    l: 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    a: 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    b: 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  };
}

export function oklchToSrgb(color: Oklch): Rgb {
  return oklabToSrgb(oklchToOklab(color));
}

export function inGamut(color: Rgb): boolean {
  return color.every(
    (channel) => channel >= -GAMUT_EPSILON && channel <= 1 + GAMUT_EPSILON,
  );
}

function clamp(channel: number): number {
  return Math.min(1, Math.max(0, channel));
}

export function clip(color: Rgb): Rgb {
  return [clamp(color[0]), clamp(color[1]), clamp(color[2])];
}

function deltaEOK(a: Oklab, b: Oklab): number {
  return Math.hypot(a.l - b.l, a.a - b.a, a.b - b.b);
}

/** `color` mapped into sRGB with the CSS Color 4 gamut-mapping algorithm. */
export function gamutMap(color: Oklch): Rgb {
  if (color.l >= 1) {
    return [1, 1, 1];
  }
  if (color.l <= 0) {
    return [0, 0, 0];
  }
  const origin = oklchToSrgb(color);
  if (inGamut(origin)) {
    return clip(origin);
  }
  let clipped = clip(origin);
  if (deltaEOK(srgbToOklab(clipped), oklchToOklab(color)) < JND) {
    return clipped;
  }
  let min = 0;
  let max = color.c;
  let minInGamut = true;
  let current = color;
  while (max - min > SEARCH_EPSILON) {
    const chroma = (min + max) / 2;
    current = { ...color, c: chroma };
    const rgb = oklchToSrgb(current);
    if (minInGamut && inGamut(rgb)) {
      min = chroma;
      continue;
    }
    clipped = clip(rgb);
    const error = deltaEOK(srgbToOklab(clipped), oklchToOklab(current));
    if (error < JND) {
      if (JND - error < SEARCH_EPSILON) {
        return clipped;
      }
      minInGamut = false;
      min = chroma;
    } else {
      max = chroma;
    }
  }
  return clip(oklchToSrgb(current));
}

/** `color` as `#RRGGBB`, clipped to sRGB as a browser renders it. */
export function toHex(color: Rgb): string {
  const hex = clip(color)
    .map((channel) =>
      Math.round(channel * 255)
        .toString(16)
        .padStart(2, "0"),
    )
    .join("");
  return `#${hex.toUpperCase()}`;
}

/** WCAG 2.2 relative luminance of an `#RRGGBB` colour. */
export function relativeLuminance(hex: string): number {
  const channel = (offset: number): number =>
    decode(Number.parseInt(hex.slice(offset, offset + 2), 16) / 255);
  return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
}

/** WCAG 2.2 contrast ratio of two `#RRGGBB` colours, 1–21. */
export function contrastRatio(a: string, b: string): number {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

function trim(value: number, digits: number): string {
  return String(Number(value.toFixed(digits)));
}

/** `color` as a CSS `oklch()` value: four decimals for L and C, two for hue. */
export function formatOklch(color: Oklch): string {
  const hue = color.c === 0 ? 0 : color.h;
  return `oklch(${trim(color.l, 4)} ${trim(color.c, 4)} ${trim(hue, 2)})`;
}

/** Parses a value written by {@link formatOklch}. */
export function parseOklch(css: string): Oklch {
  const match = /^oklch\(([\d.]+) ([\d.]+) ([\d.]+)\)$/.exec(css);
  if (match === null) {
    throw new Error(`not an oklch() value: ${css}`);
  }
  const [, l = "", c = "", h = ""] = match;
  return { l: Number(l), c: Number(c), h: Number(h) };
}
