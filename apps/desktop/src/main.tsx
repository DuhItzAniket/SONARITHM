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
const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(
    <StrictMode>
      <LocalizerContext value={uiLocalizer()}>
        <App transport={transport} announcer={announcer} />
      </LocalizerContext>
    </StrictMode>,
  );
}
const smoke = smokeConfig(window);
if (smoke !== null) {
  void runSmoke(transport, smoke);
}
