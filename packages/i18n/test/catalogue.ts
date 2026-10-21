// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Test helper: the catalogue files of a locale, read from the Rust crate that owns them.
import { readdirSync, readFileSync } from "node:fs";

const LOCALES = new URL(
  "../../../crates/sonarithm-i18n/locales/",
  import.meta.url,
);

/** The contents of every `.ftl` file of `locale`, in file-name order. */
export function catalogueSources(locale: string): string[] {
  const dir = new URL(`${locale}/`, LOCALES);
  return readdirSync(dir)
    .filter((file) => file.endsWith(".ftl"))
    .sort()
    .map((file) => readFileSync(new URL(file, dir), "utf8"));
}

/** The message IDs (not terms) defined in `sources`, sorted. */
export function messageIds(sources: readonly string[]): string[] {
  const ids: string[] = [];
  for (const source of sources) {
    for (const match of source.matchAll(/^([a-zA-Z][\w-]*)\s*=/gm)) {
      if (match[1] !== undefined) {
        ids.push(match[1]);
      }
    }
  }
  return ids.sort();
}
