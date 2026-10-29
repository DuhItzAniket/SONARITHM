// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Event, IpcTransport } from "@sonarithm/ipc";
import { Channel, invoke } from "@tauri-apps/api/core";

import { createTauriTransport } from "./transport.ts";

/** The transport over Tauri's real JavaScript API; only the app's entry point uses it. */
export function tauriTransport(): IpcTransport {
  return createTauriTransport({
    invoke: (command, args) => invoke(command, args),
    channel(onMessage) {
      const channel = new Channel<Event>();
      channel.onmessage = onMessage;
      return channel;
    },
  });
}
