// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Channel, Event, MockHandlers, Request } from "@sonarithm/ipc";
import { createMockTransport, EVENT_CHANNELS } from "@sonarithm/ipc";
import { mockIPC } from "@tauri-apps/api/mocks";

/**
 * Tauri's `mockIPC` answering the host's commands from `@sonarithm/ipc`'s in-memory host,
 * so Playwright drives the real UI code, including `createTauriTransport`, in any browser
 * (testing-strategy §12). Flows add handlers here as they need them.
 *
 * Tests push app-core events with `window.__SONARITHM_MOCK__.emit(event)`: each goes to
 * the Tauri channels subscribed to its ADR-0017 channel, as the host's router does.
 */
const handlers: MockHandlers = {};
const host = createMockTransport(handlers);

/** A Tauri channel as the mock receives it: its handler is readable. */
interface MockChannel {
  readonly onmessage: (event: Event) => void;
}

const subscribers = new Map<number, { channel: Channel; to: MockChannel }>();
let nextId = 1;

mockIPC(async (command, payload) => {
  const args = (payload ?? {}) as {
    request?: Request;
    channel?: Channel;
    onEvent?: MockChannel;
    id?: number;
  };
  switch (command) {
    case "ipc_dispatch":
      return args.request === undefined ? null : host.invoke(args.request);
    case "ipc_binary":
      return args.request === undefined ? null : host.binary(args.request);
    case "ipc_subscribe": {
      const id = nextId++;
      if (args.channel !== undefined && args.onEvent !== undefined) {
        subscribers.set(id, { channel: args.channel, to: args.onEvent });
      }
      return id;
    }
    case "ipc_unsubscribe":
      if (args.id !== undefined) {
        subscribers.delete(args.id);
      }
      return null;
    case "announce_native":
      // No native route in the browser: the announcer falls back to live regions.
      throw new Error("No native announcer in the mock host.");
    default:
      return null;
  }
});

Reflect.set(window, "__SONARITHM_MOCK__", {
  emit(event: Event) {
    const channel = EVENT_CHANNELS[event.event];
    for (const { channel: wanted, to } of subscribers.values()) {
      if (wanted === channel) {
        to.onmessage(event);
      }
    }
  },
});
