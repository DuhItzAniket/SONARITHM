// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Client } from "@sonarithm/ipc";
import { useCallback, useEffect } from "react";

import type { CommandRegistry } from "./registry.ts";

const PREFIX = "interaction.keymap.";

/**
 * Applies the user's keymap to the command registry (B03-T09; keyboard §13): the
 * effective keymap of the settings and the active profile, as the app core resolves it
 * (`settings_effective`; a project's keys never apply), and the group switch from the
 * settings. Returns a function that loads them again, for after Preferences saves.
 */
export function useKeymapSettings(
  client: Client | null,
  registry: CommandRegistry,
): () => void {
  const reload = useCallback(() => {
    if (client === null) return;
    void Promise.all([
      client.call({ command: "settings_get" }),
      client.call({ command: "settings_effective" }),
    ])
      .then(([settings, effective]) => {
        const keymap = Object.fromEntries(
          effective.settings
            .filter((setting) => setting.key.startsWith(PREFIX))
            .map((setting) => [
              setting.key.slice(PREFIX.length),
              typeof setting.value === "string" ? setting.value : "",
            ]),
        );
        registry.configure(keymap, settings.keyboard.singleKeyCommands);
      })
      .catch(() => undefined);
  }, [client, registry]);
  useEffect(() => {
    reload();
  }, [reload]);
  return reload;
}
