// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  DatasetSummary,
  Event,
  MockHandlers,
  Request,
  Settings,
  TransportState,
} from "@sonarithm/ipc";
import {
  createMockTransport,
  encodeView,
  EVENT_CHANNELS,
} from "@sonarithm/ipc";
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

/** The mock settings (B01-T08): the app core's defaults. */
let settings: Settings = {
  settingsVersion: 1,
  audio: {
    device: null,
    bufferFrames: null,
    volumeDb: 0,
    calibratedLatencyS: {},
  },
  theme: "system",
  verbosity: "terse",
  modalities: {
    sonification: true,
    speech: true,
    braille: false,
    visual: true,
  },
  keymap: {},
  consent: { updateCheck: null },
  speech: {
    route: "screenReader",
    duckInsteadOfPause: false,
    rateWpm: 180,
    announcer: "liveRegion",
  },
  gpu: "default",
};

const DEVICES = [
  { id: "speakers", name: "Speakers", isDefault: true },
  { id: "headphones", name: "Headphones", isDefault: false },
];

/**
 * A dataset for the data table (B04-T05), when a test sets
 * `window.__SONARITHM_MOCK_DATASET__ = { rows }` before the page loads. Row `i` has time
 * `i / 10` s and a temperature; every tenth row is estimated from a chart image, with a
 * confidence. Sorting by any field reverses the order when descending, which is the order
 * these values have; the mock does not sort by value.
 */
function mockDataset(): DatasetSummary | null {
  const wanted: unknown = Reflect.get(window, "__SONARITHM_MOCK_DATASET__");
  const rows =
    typeof wanted === "object" && wanted !== null
      ? Number(Reflect.get(wanted, "rows"))
      : Number.NaN;
  if (!Number.isInteger(rows) || rows < 0) {
    return null;
  }
  const field = (name: string, unit: string) => ({
    name,
    dataType: "f64",
    fieldType: null,
    role: null,
    unit,
    concept: null,
  });
  return {
    id: "mock-1",
    name: "Readings",
    rows,
    fields: [field("time", "s"), field("temperature", "°C")],
    diagnostics: [],
  };
}
const dataset = mockDataset();

/** A table page of the mock dataset, encoded as the app core does (ADR-0017 §6). */
function tablePage(args: {
  readonly offset: number;
  readonly limit: number;
  readonly sort: { readonly descending: boolean } | null;
}): ArrayBuffer {
  const total = dataset?.rows ?? 0;
  const count = Math.max(0, Math.min(args.limit, total - args.offset));
  const index = Array.from({ length: count }, (_, i) =>
    args.sort?.descending === true
      ? total - 1 - (args.offset + i)
      : args.offset + i,
  );
  return encodeView({
    kind: 3,
    generation: 1,
    columns: [
      {
        name: "__snr_row",
        dataType: "i64",
        values: index.map((i) => BigInt(i)),
      },
      { name: "time", dataType: "f64", values: index.map((i) => i / 10) },
      {
        name: "temperature",
        dataType: "f64",
        values: index.map((i) => 15 + (i % 7)),
      },
      {
        name: "__snr_prov",
        dataType: "provenance",
        values: index.map((i) => (i % 10 === 9 ? 2 : 0)),
      },
      {
        name: "__snr_conf",
        dataType: "confidence",
        values: index.map(() => 0.82),
        validity: index.map((i) => i % 10 === 9),
      },
    ],
  });
}

const handlers: MockHandlers = {
  dataset_list: () => ({
    reply: "datasets" as const,
    datasets: dataset === null ? [] : [dataset],
  }),
  stop_all: () =>
    reply(
      transport.status === "playing"
        ? { ...transport, status: "paused" }
        : transport,
    ),
  settings_get: () => ({ reply: "settings" as const, ...settings }),
  settings_set: (request) => {
    settings = request.args.settings;
    return { reply: "settings" as const, ...settings };
  },
  devices_list: () => ({
    reply: "devices" as const,
    devices: DEVICES.map((device) => ({
      ...device,
      selected: device.id === (settings.audio.device ?? "speakers"),
    })),
  }),
  device_select: (request) => {
    settings = {
      ...settings,
      audio: { ...settings.audio, device: request.args.id },
    };
    return reply({ ...transport, device: request.args.id ?? "Speakers" });
  },
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
const host = createMockTransport(handlers, {
  binary: {
    table_page: (request) =>
      request.command === "table_page"
        ? tablePage(request.args)
        : new ArrayBuffer(0),
  },
});

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
    emit({
      event: "clock",
      transport,
      hostTimeS: null,
      outputLatencyS: null,
      seq: 1,
    });
  },
  emit,
  /** The commands the UI sent through `ipc_dispatch`, oldest first. */
  commands(): string[] {
    return host.requests.map((request) => request.command);
  },
  /** The settings as the UI last saved them. */
  settings(): Settings {
    return settings;
  },
});
