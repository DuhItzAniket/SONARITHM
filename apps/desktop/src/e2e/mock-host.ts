// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  Event,
  MockHandlers,
  Request,
  TransportState,
} from "@sonarithm/ipc";
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
/** The mock transport: no Score until a test loads one, so nothing is playable at start. */
let transport: TransportState = {
  status: "stopped",
  positionS: 0,
  durationS: 0,
  speed: 1,
  loopRange: null,
  loaded: false,
  device: null,
  live: false,
  xruns: 0,
};

function reply(next: TransportState) {
  transport = next;
  return { reply: "transport" as const, ...next };
}

const handlers: MockHandlers = {
  transport_state: () => reply(transport),
  transport_play: () =>
    reply({ ...transport, status: "playing", device: "Mock output" }),
  transport_pause: () =>
    reply(
      transport.status === "playing"
        ? { ...transport, status: "paused" }
        : transport,
    ),
  transport_stop: () =>
    reply({ ...transport, status: "stopped", positionS: 0 }),
  transport_set_loop: (request) =>
    reply({ ...transport, loopRange: request.args.range }),
};
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

/** Sends an event to the Tauri channels subscribed to its channel, as the host's router does. */
function emit(event: Event): void {
  const channel = EVENT_CHANNELS[event.event];
  for (const { channel: wanted, to } of subscribers.values()) {
    if (wanted === channel) {
      to.onmessage(event);
    }
  }
}

Reflect.set(window, "__SONARITHM_MOCK__", {
  /** Loads a Score of `durationS` seconds and reports it, as a compile would. */
  loadScore(durationS: number) {
    transport = { ...transport, loaded: true, durationS };
    emit({ event: "clock", transport, hostTimeS: null, seq: 1 });
  },
  emit,
});
