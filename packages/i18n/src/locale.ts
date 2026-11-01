// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * A locale tag that `Intl` and Fluent accept, from whatever the system reports (F-210).
 *
 * POSIX locale names are not BCP 47 tags: `C` and `POSIX` (with or without
 * `.UTF-8` or an `@modifier`) name no language, and `en_US.UTF-8` or `de_DE@euro`
 * carry an encoding, a modifier and an underscore. WebKitGTK reports the process locale
 * as `navigator.language`, so under `LANG=C` a page sees "C", and
 * `new Intl.NumberFormat("C")` throws a `RangeError`. This helper drops the encoding and
 * the modifier, turns underscores into hyphens, maps C and POSIX to `fallback`, and returns
 * the canonical tag, or `fallback` for anything `Intl.getCanonicalLocales` rejects.
 */
export function normalizeLocale(
  tag: string | null | undefined,
  fallback: string,
): string {
  const base =
    (tag ?? "").split(/[.@]/, 1)[0]?.trim().replaceAll("_", "-") ?? "";
  if (base === "" || /^(c|posix)$/i.test(base)) {
    return fallback;
  }
  try {
    return Intl.getCanonicalLocales(base)[0] ?? fallback;
  } catch {
    return fallback;
  }
}
