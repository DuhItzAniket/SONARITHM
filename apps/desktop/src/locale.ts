// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { SOURCE_LOCALE, normalizeLocale } from "@sonarithm/i18n";

/** What `fixNavigatorLocale` reads and replaces. */
interface LocaleSource {
  readonly language: string;
  readonly languages: readonly string[];
}

/**
 * Makes `navigator.language` and `navigator.languages` tags that `Intl` accepts.
 * WebKitGTK reports the process locale, so under `LANG=C` they are "C", and uPlot 1.6.32
 * runs `new Intl.NumberFormat(navigator.language)` when its module loads, which throws and
 * stops the page. Only invalid values are replaced, with `normalizeLocale`'s result; valid
 * ones are left as they are. Returns whether anything changed.
 */
export function fixNavigatorLocale(target: LocaleSource): boolean {
  const language = normalizeLocale(target.language, SOURCE_LOCALE);
  const languages = [
    ...new Set([
      language,
      ...target.languages.map((tag) => normalizeLocale(tag, SOURCE_LOCALE)),
    ]),
  ];
  const unchanged =
    language === target.language &&
    languages.length === target.languages.length &&
    languages.every((tag, i) => tag === target.languages[i]);
  if (unchanged) {
    return false;
  }
  try {
    Object.defineProperty(target, "language", {
      configurable: true,
      get: () => language,
    });
    Object.defineProperty(target, "languages", {
      configurable: true,
      get: () => Object.freeze([...languages]),
    });
  } catch {
    // A webview that will not let the value be replaced: nothing more can be done here,
    // and the page must not stop for it.
    return false;
  }
  return true;
}

// The first import of `main.tsx`, so this runs before any module that reads the locale.
fixNavigatorLocale(navigator);
