// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Localizer } from "@sonarithm/i18n";
import { SOURCE_LOCALE, createLocalizer } from "@sonarithm/i18n";

// Stories show catalogue text, never literals (coding-standards §12.7): the same files the
// app and the Rust announcement templates use.
const files = import.meta.glob<string>(
  "../../../crates/sonarithm-i18n/locales/en-US/*.ftl",
  { query: "?raw", import: "default", eager: true },
);

/** A localizer over the English catalogue, for stories and browser tests. */
export function storyLocalizer(): Localizer {
  const sources = Object.keys(files)
    .sort()
    .map((name) => files[name] ?? "");
  return createLocalizer(SOURCE_LOCALE, sources);
}
