// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { EffectiveSetting } from "@sonarithm/ipc";

/**
 * Whether the visual highlight is on in the app core's effective settings (V, keyboard
 * §5.5, B03-T09). Missing means on: the default (A11Y-10).
 */
export function visualOn(settings: readonly EffectiveSetting[]): boolean {
  const visual = settings.find(
    (setting) => setting.key === "interaction.modalities.visual",
  );
  return visual?.value !== false;
}
