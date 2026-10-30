// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/** Joins class names, skipping empty ones. */
export function cx(
  ...names: readonly (string | false | null | undefined)[]
): string {
  return names.filter(Boolean).join(" ");
}
