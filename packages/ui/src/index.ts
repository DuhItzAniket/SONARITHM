// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/ui`: design tokens and the React Aria Components wrappers that the
 * desktop app imports its primitives from (coding-standards §12.3, design-language.md).
 *
 * The tokens are CSS: import `@sonarithm/ui/tokens.css` and `@sonarithm/ui/fonts.css` once,
 * before any component CSS. `tokens.css` defines the semantic colour, spacing, radius, motion
 * and type custom properties in the `tokens` cascade layer; the host picks the theme with
 * `data-theme` on the root element (design-language §5.4). `fonts.css` loads the bundled IBM
 * Plex files and defines the `snr-numeric` class for readouts (§7.1, DL-X10).
 * `@sonarithm/ui/licenses.json` holds the font notices for the licences view (12 §9).
 * `pnpm -C packages/ui tokens` regenerates all three. Components arrive in B02-T04.
 */
export {};
