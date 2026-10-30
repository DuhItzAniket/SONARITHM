// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * Colour-vision deficiency simulation for the design review (B02-T09, DL-X08, A11Y-25).
 *
 * Machado, Oliveira and Fernandes, "A Physiologically-based Model for Simulation of Color
 * Vision Deficiency" (IEEE TVCG 15(6), 2009): the severity-1.0 matrices for protanopia,
 * deuteranopia and tritanopia, applied to linear sRGB. Achromatopsia is modelled as WCAG
 * relative luminance alone. The simulation answers two review questions: do indicators
 * (focus, accent, strong borders) keep their contrast for CVD viewers, and which signal
 * colours collapse into each other, which must therefore never be told apart by colour
 * alone (DL-X08).
 */

import type { Rgb } from "./color.ts";
import { srgbToOklab, toHex } from "./color.ts";

/** A colour-vision deficiency type. */
export type Deficiency =
  "protanopia" | "deuteranopia" | "tritanopia" | "achromatopsia";

/** Every simulated deficiency. */
export const DEFICIENCIES: readonly Deficiency[] = [
  "protanopia",
  "deuteranopia",
  "tritanopia",
  "achromatopsia",
];

type Matrix = readonly [
  readonly [number, number, number],
  readonly [number, number, number],
  readonly [number, number, number],
];

/** Machado et al. 2009, severity 1.0, for linear RGB. */
const MACHADO: Readonly<Record<Exclude<Deficiency, "achromatopsia">, Matrix>> =
  {
    protanopia: [
      [0.152286, 1.052583, -0.204868],
      [0.114503, 0.786281, 0.099216],
      [-0.003882, -0.048116, 1.051998],
    ],
    deuteranopia: [
      [0.367322, 0.860646, -0.227968],
      [0.280085, 0.672501, 0.047413],
      [-0.01182, 0.04294, 0.968881],
    ],
    tritanopia: [
      [1.255528, -0.076749, -0.178779],
      [-0.078411, 0.930809, 0.147602],
      [0.004733, 0.691367, 0.3039],
    ],
  };

const toLinear = (c: number): number =>
  c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
const toGamma = (c: number): number =>
  c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055;
const clamp = (c: number): number => Math.min(1, Math.max(0, c));

/** `#RRGGBB` as gamma-encoded sRGB. */
export function parseHex(hex: string): Rgb {
  const channel = (offset: number): number =>
    Number.parseInt(hex.slice(offset, offset + 2), 16) / 255;
  return [channel(1), channel(3), channel(5)];
}

/** How `hex` looks to a viewer with `deficiency`, as `#RRGGBB`. */
export function simulate(hex: string, deficiency: Deficiency): string {
  const linear = parseHex(hex).map(toLinear) as unknown as Rgb;
  let out: Rgb;
  if (deficiency === "achromatopsia") {
    const y = 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
    out = [y, y, y];
  } else {
    const m = MACHADO[deficiency];
    out = m.map(
      (row) => row[0] * linear[0] + row[1] * linear[1] + row[2] * linear[2],
    ) as unknown as Rgb;
  }
  return toHex(out.map((c) => toGamma(clamp(c))) as unknown as Rgb);
}

/** The OKLab distance between two colours (deltaEOK); about 0.02 is just noticeable. */
export function deltaEOK(a: string, b: string): number {
  const la = srgbToOklab(parseHex(a));
  const lb = srgbToOklab(parseHex(b));
  return Math.hypot(la.l - lb.l, la.a - lb.a, la.b - lb.b);
}
