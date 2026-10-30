// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import "@sonarithm/ui/tokens.css";
import "@sonarithm/ui/fonts.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import {
  createLiveRegionAnnouncer,
  createRoutedAnnouncer,
} from "./announcer/announcer.ts";
import { mountLiveRegions } from "./announcer/live-regions.ts";
import { tauriNativeAnnouncer } from "./announcer/native.ts";
import { App } from "./app/App.tsx";
import { currentPlatform } from "./commands/chords.ts";
import { CommandsContext } from "./commands/context.tsx";
import { DEFAULT_KEYMAP } from "./commands/keymap.ts";
import { CommandRegistry } from "./commands/registry.ts";
import { LocalizerContext, uiLocalizer } from "./i18n.ts";
import { tauriTransport } from "./ipc/tauri-core.ts";
import { installAcceleratorGuard } from "./keyboard/accelerators.ts";
import { runSmoke, smokeConfig } from "./smoke.ts";

installAcceleratorGuard(window);
const transport = tauriTransport();
// The managed announcer (08 §4, B03-T04): live regions by default, the native route by
// setting (OD-05).
const announcer = createRoutedAnnouncer({
  live: createLiveRegionAnnouncer(mountLiveRegions(document)),
  native: tauriNativeAnnouncer(),
});
// The command registry (B02-T07): the v0 keymap; user remapping arrives with profiles.
const registry = new CommandRegistry(
  DEFAULT_KEYMAP,
  currentPlatform(navigator.userAgent),
);
if (import.meta.env.DEV) {
  // For the E2E tests: which commands are registered, and which one ran last.
  Reflect.set(window, "__SONARITHM_COMMANDS__", () =>
    registry.available().map(({ spec }) => spec.id),
  );
  registry.onRun((id) => {
    document.documentElement.dataset.lastCommand = id;
  });
}
const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(
    <StrictMode>
      <LocalizerContext value={uiLocalizer()}>
        <CommandsContext value={registry}>
          <App transport={transport} announcer={announcer} />
        </CommandsContext>
      </LocalizerContext>
    </StrictMode>,
  );
}
const smoke = smokeConfig(window);
if (smoke !== null) {
  void runSmoke(transport, smoke);
}
