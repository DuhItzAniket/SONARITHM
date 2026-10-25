// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/ui`: design tokens and the React Aria Components wrappers that the
 * desktop app imports its primitives from (coding-standards §12.3, design-language.md).
 *
 * The tokens are CSS: import `@sonarithm/ui/tokens.css` once, before any component CSS. It
 * defines the semantic colour, spacing, radius and motion custom properties in the `tokens`
 * cascade layer; the host picks the theme with `data-theme` on the root element
 * (design-language §5.4). `pnpm -C packages/ui tokens` regenerates it from `tokens/recipe.ts`.
 * Components arrive in B02-T04.
 */
export {};
