// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Localizer } from "@sonarithm/i18n";
import { SOURCE_LOCALE, createLocalizer } from "@sonarithm/i18n";
import { createContext, use } from "react";

// The catalogues the Rust announcement templates use too (12 §5), bundled at build time.
const files = import.meta.glob<string>(
  "../../../crates/sonarithm-i18n/locales/en-US/*.ftl",
  { query: "?raw", import: "default", eager: true },
);

/** The UI's localizer. The locale setting arrives with B07-T07 (OD-12). */
export function uiLocalizer(): Localizer {
  const sources = Object.keys(files)
    .sort()
    .map((name) => files[name] ?? "");
  return createLocalizer(SOURCE_LOCALE, sources);
}

/** The localizer for components below the app root. */
export const LocalizerContext = createContext<Localizer | null>(null);

/** Formats catalogue messages; throws outside the app root. */
export function useLocalizer(): Localizer {
  const localizer = use(LocalizerContext);
  if (localizer === null) {
    throw new Error("useLocalizer outside LocalizerContext");
  }
  return localizer;
}
