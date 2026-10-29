// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import "@sonarithm/ui/tokens.css";
import "@sonarithm/ui/fonts.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./app/App.tsx";
import { LocalizerContext, uiLocalizer } from "./i18n.ts";
import { tauriTransport } from "./ipc/tauri-core.ts";
import { installAcceleratorGuard } from "./keyboard/accelerators.ts";
import { runSmoke, smokeConfig } from "./smoke.ts";

installAcceleratorGuard(window);
const transport = tauriTransport();
const root = document.getElementById("root");
if (root !== null) {
  createRoot(root).render(
    <StrictMode>
      <LocalizerContext value={uiLocalizer()}>
        <App transport={transport} />
      </LocalizerContext>
    </StrictMode>,
  );
}
const smoke = smokeConfig(window);
if (smoke !== null) {
  void runSmoke(transport, smoke);
}
