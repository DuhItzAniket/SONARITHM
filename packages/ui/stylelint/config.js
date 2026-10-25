// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Stylelint rules for SONARITHM CSS (coding-standards §12.6, design-language §3). Component CSS
// uses semantic tokens only; the generated token file is the one place with raw values, so it
// is ignored. The root stylelint.config.js re-exports this config for the whole workspace.

/** Functions that write a colour literal; components use semantic tokens (DL-P07). */
const COLOR_FUNCTIONS = [
  "rgb",
  "rgba",
  "hsl",
  "hsla",
  "hwb",
  "lab",
  "lch",
  "oklab",
  "oklch",
  "color",
  "color-mix",
  "light-dark",
];

/** Gradients and gradient text (DL-X01). */
const GRADIENTS = [
  "linear-gradient",
  "radial-gradient",
  "conic-gradient",
  "repeating-linear-gradient",
  "repeating-radial-gradient",
  "repeating-conic-gradient",
];

/** Frosted blur (DL-X02) and glow (DL-X01). */
const EFFECTS = ["blur", "drop-shadow"];

/** One radius value: zero or a radius token up to radius-2 (DL-X03). */
const RADIUS = String.raw`(0|var\(--radius-[0-2]\))`;

/** @type {import("stylelint").Config} */
export default {
  ignoreFiles: [
    "**/src/generated/**",
    "**/node_modules/**",
    "**/dist/**",
    "**/storybook-static/**",
  ],
  rules: {
    // DL-P07: no colour literals.
    "color-no-hex": true,
    "color-named": "never",
    "function-disallowed-list": [...COLOR_FUNCTIONS, ...GRADIENTS, ...EFFECTS],
    // DL-X02 frosted panels; DL-X01 glow on text.
    "property-disallowed-list": [
      "backdrop-filter",
      "-webkit-backdrop-filter",
      "text-shadow",
    ],
    // DL-X01 gradient text.
    "declaration-property-value-disallowed-list": {
      "/^(-webkit-)?background-clip$/": ["text"],
    },
    "declaration-property-value-allowed-list": {
      // DL-X01 coloured shadows and DL-X03 soft shadows: only the overlay token (§8).
      "box-shadow": ["none", "var(--shadow-overlay)"],
      // DL-X03: radii come from the radius tokens, at most 4 px.
      "/radius$/": [`/^${RADIUS}( ${RADIUS}){0,3}$/`],
    },
    // Spacing and durations come from tokens (coding-standards §12.6).
    "declaration-property-unit-disallowed-list": {
      "/^(margin|padding|gap|row-gap|column-gap|inset)/": ["px", "rem", "em"],
      "/^(transition|animation)/": ["ms", "s"],
    },
  },
};
