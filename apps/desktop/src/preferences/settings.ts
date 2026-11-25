// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Settings } from "@sonarithm/ipc";

/** The settings in a `settings` reply, without its `reply` tag, ready for `settings_set`. */
export function settingsOf(
  reply: Settings & { readonly reply?: string },
): Settings {
  return Object.fromEntries(
    Object.entries(reply).filter(([key]) => key !== "reply"),
  ) as Settings;
}
