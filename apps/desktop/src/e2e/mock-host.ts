// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  CursorOrigin,
  CursorState,
  DatasetSummary,
  DiagnosticInfo,
  EffectiveSetting,
  Event,
  MockHandlers,
  Request,
  Settings,
  ToggleTarget,
  TransportState,
} from "@sonarithm/ipc";
import {
  createMockTransport,
  encodeView,
  EVENT_CHANNELS,
  IpcFailure,
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
  muted: false,
  excludedTracks: [],
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
  choices: {},
  braille: { mode: "review" },
};

const MODALITIES = ["sonification", "speech", "braille", "visual"] as const;
let announced = 0;

/** The effective modalities and verbosity: the toggles' choices over the settings. */
function effective(): EffectiveSetting[] {
  const { choices } = settings;
  const layer = (chosen: unknown) =>
    chosen === undefined || chosen === null
      ? ("settings" as const)
      : ("session" as const);
  return [
    {
      key: "interaction.verbosity",
      value: choices.verbosity ?? settings.verbosity,
      layer: layer(choices.verbosity),
    },
    ...MODALITIES.map((modality) => ({
      key: `interaction.modalities.${modality}`,
      value: choices[modality] ?? settings.modalities[modality],
      layer: layer(choices[modality]),
    })),
  ];
}

/** Says `text` as the app core would; `assertive` for an error. */
function say(text: string, assertive = false): void {
  announced += 1;
  emit({
    event: "announcement",
    seq: announced,
    speech: text,
    display: text,
    class: "status",
    assertive,
    interrupt: false,
    selfVoiced: false,
    node: null,
    muted: false,
  });
}

/**
 * `modality_toggle` as the app core does it (B03-T09): flips the effective value, refuses
 * to leave no non-visual output (SNR-APP-094), and says what changed.
 */
function toggle(target: ToggleTarget) {
  const now = (key: string) =>
    effective().find((s) => s.key === `interaction.${key}`)?.value;
  if (target === "verbosity") {
    const next = now("verbosity") === "verbose" ? "terse" : "verbose";
    settings = {
      ...settings,
      choices: { ...settings.choices, verbosity: next },
    };
    say(next === "verbose" ? "Verbose announcements" : "Terse announcements");
    return { reply: "effective" as const, settings: effective() };
  }
  const on = now(`modalities.${target}`) !== true;
  const after = { ...settings.choices, [target]: on };
  const audible = (["speech", "sonification", "braille"] as const).some(
    (m) => after[m] ?? settings.modalities[m],
  );
  if (!audible) {
    say(
      "Speech, sonification and braille cannot all be off. Turn another one on first.",
      true,
    );
    throw new IpcFailure({
      code: "SNR-APP-094",
      message:
        "Speech, sonification and braille cannot all be off. Nothing was changed.",
      hint: null,
      pointer: null,
    });
  }
  settings = { ...settings, choices: after };
  if (target === "sonification") {
    transport = { ...transport, muted: !on };
  }
  const name = {
    speech: "Speech",
    sonification: "Sonification",
    braille: "Braille",
    visual: "Visual highlight",
  }[target];
  say(`${name} ${on ? "on" : "off"}`);
  return { reply: "effective" as const, settings: effective() };
}

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
    dataType: "Float64",
    fieldType: name === "time" ? "temporal" : "quantitative",
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

/**
 * The one cursor as Rust would hold it (B04-T06), without a compiled Score: the path is the
 * dataset alone. Every change goes out as a `cursor` event.
 */
let cursor: CursorState = {
  seq: 0,
  dataset: null,
  row: null,
  x: null,
  node: null,
  path: [],
  excluded: [],
  origin: null,
  braille: null,
};

function moveCursor(row: number, origin: CursorOrigin): CursorState {
  cursor = {
    seq: cursor.seq + 1,
    dataset: dataset?.id ?? null,
    row,
    x: null,
    node: null,
    path: dataset === null ? [] : [{ level: "dataset", label: dataset.name }],
    excluded: [],
    origin,
    braille: null,
  };
  emit({ event: "cursor", ...cursor });
  return cursor;
}

/**
 * A plot tile of the mock dataset (B04-T06): `pxWidth` rows spread evenly over the table,
 * with the columns `row`, `x` (time) and `y` (temperature), as `view_get` sends them. It
 * thins the rows rather than computing M4, which is enough for the cursor's tests.
 */
function viewTile(
  pxWidth: number,
  range: { readonly start: number; readonly end: number } | null,
): ArrayBuffer {
  const total = dataset?.rows ?? 0;
  // x is row / 10, so a range of x is a range of rows (B04-T07).
  const first = range === null ? 0 : Math.max(0, Math.ceil(range.start * 10));
  const last =
    range === null
      ? total - 1
      : Math.min(total - 1, Math.floor(range.end * 10));
  const span = Math.max(0, last - first);
  const count = Math.min(span + 1, Math.max(1, pxWidth));
  const rows = Array.from({ length: count }, (_, i) =>
    count === 1 ? first : first + Math.round((i * span) / (count - 1)),
  );
  return encodeView({
    kind: 1,
    generation: 1,
    columns: [
      { name: "row", dataType: "i64", values: rows.map((r) => BigInt(r)) },
      { name: "x", dataType: "f64", values: rows.map((r) => r / 10) },
      { name: "y", dataType: "f64", values: rows.map((r) => 15 + (r % 7)) },
    ],
  });
}

/**
 * The explorer's verbs (B03-T09), simplified: the steps along a level move the row and stop
 * at the ends with a boundary; every other verb keeps the row. The app core's own tests
 * cover what each verb does; the E2E checks which verb each key sends.
 */
function navMove(verb: string): {
  row: number;
  outcome: "moved" | "boundary";
  edge: string | null;
} {
  const total = dataset?.rows ?? 0;
  const at = cursor.row ?? 0;
  const jump = Math.max(1, Math.round(total / 10));
  const target =
    {
      next: at + 1,
      previous: at - 1,
      jumpForward: at + jump,
      jumpBack: at - jump,
      first: 0,
      last: total - 1,
    }[verb] ?? at;
  if (target < 0 || target >= total) {
    return {
      row: at,
      outcome: "boundary",
      edge: target < 0 ? "first" : "last",
    };
  }
  return { row: target, outcome: "moved", edge: null };
}

/**
 * The chart image of the calibration flow (C02-T10): a 200 by 120 line chart, axes at
 * x = 20 and y = 100. The mock fits numeric anchors with a straight line and refuses ticks
 * that do not change in one direction, as the app core does; it reads no dates or labels
 * (the app core's tests cover those).
 */
const CHART_IMAGE = `data:image/svg+xml;utf8,${encodeURIComponent(
  '<svg xmlns="http://www.w3.org/2000/svg" width="200" height="120">' +
    '<rect width="200" height="120" fill="white"/>' +
    '<path d="M20 10V100H190" fill="none" stroke="black"/>' +
    '<polyline points="20,90 60,70 100,40 140,55 180,20" fill="none" stroke="steelblue" stroke-width="2"/>' +
    "</svg>",
)}`;

interface MockAnchor {
  readonly at: readonly number[];
  readonly value: unknown;
}

/** A least-squares line through numeric anchors along `coordinate`, or a refusal. */
function mockFit(
  anchors: readonly MockAnchor[],
  coordinate: 0 | 1,
):
  | { readonly fit: (p: number) => number }
  | { readonly refused: string }
  | null {
  const points = anchors.flatMap((a) =>
    typeof a.value === "number"
      ? [[a.at[coordinate] ?? 0, a.value] as const]
      : [],
  );
  if (points.length < 2) return null;
  const sorted = [...points].sort((p, q) => p[0] - q[0]);
  const rising = sorted.every(
    (p, i) => i === 0 || p[1] >= (sorted[i - 1]?.[1] ?? 0),
  );
  const falling = sorted.every(
    (p, i) => i === 0 || p[1] <= (sorted[i - 1]?.[1] ?? 0),
  );
  if (!rising && !falling) {
    return {
      refused: "The tick values do not change in one direction along the axis.",
    };
  }
  const n = points.length;
  const mp = points.reduce((s, p) => s + p[0], 0) / n;
  const mv = points.reduce((s, p) => s + p[1], 0) / n;
  const spp = points.reduce((s, p) => s + (p[0] - mp) ** 2, 0);
  const spv = points.reduce((s, p) => s + (p[0] - mp) * (p[1] - mv), 0);
  const slope = spp === 0 ? 0 : spv / spp;
  return { fit: (p) => mv + slope * (p - mp) };
}

const handlers: MockHandlers = {
  chart_raster_open: (request) => {
    const name = request.args.path.split(/[\\/]/).pop() ?? "chart.png";
    return {
      reply: "raster_chart" as const,
      id: 1,
      name,
      widthPx: 200,
      heightPx: 120,
      image: CHART_IMAGE,
      plotArea: { min: [20, 10], max: [190, 100] },
    };
  },
  chart_calibration_check: (request) => {
    const { calibration, at } = request.args;
    const diagnostics: DiagnosticInfo[] = [];
    const axes = [calibration.x, calibration.y].map((axis, index) => {
      const fitted = mockFit(axis.anchors, index === 0 ? 0 : 1);
      if (fitted !== null && "refused" in fitted) {
        diagnostics.push({
          severity: "error",
          code: "SNR-ENG-322",
          rules: [],
          message: `The ${index === 0 ? "x" : "y"} axis cannot be calibrated: ${fitted.refused}`,
          pointer: null,
        });
        return null;
      }
      return fitted;
    });
    const info = (index: 0 | 1) =>
      axes[index] === null || axes[index] === undefined
        ? null
        : {
            anchors: (index === 0 ? calibration.x : calibration.y).anchors
              .length,
            residual: 0,
            confidence: 1,
          };
    const read = (index: 0 | 1) => {
      const fitted = axes[index];
      return at === null ||
        fitted === null ||
        fitted === undefined ||
        !("fit" in fitted)
        ? null
        : Math.round(fitted.fit(at[index]) * 1000) / 1000;
    };
    // Every pixel's value along each fitted axis (the mock image is 200 by 120).
    const along = (index: 0 | 1, length: number) => {
      const fitted = axes[index];
      return fitted === null || fitted === undefined || !("fit" in fitted)
        ? null
        : Array.from(
            { length },
            (_, i) => Math.round(fitted.fit(i + 0.5) * 1000) / 1000,
          );
    };
    return {
      reply: "calibration_check" as const,
      diagnostics,
      x: info(0),
      y: info(1),
      valueAt: at === null ? null : { x: read(0), y: read(1) },
      values: { x: along(0, 200), y: along(1, 120) },
    };
  },
  chart_calibration_apply: () => ({
    reply: "dataset" as const,
    ...(dataset ?? {
      id: "d1",
      name: "rain.png",
      rows: 0,
      fields: [],
      diagnostics: [],
    }),
    name: "rain.png",
    diagnostics: [
      {
        severity: "info",
        code: "SNR-ENG-385",
        rules: [],
        message:
          "36 points: 0 exact, 0 calibrated, 36 estimated, median confidence 0.82",
        pointer: null,
      },
    ],
  }),
  chart_raster_close: (request) => ({
    reply: "raster_closed" as const,
    id: request.args.id,
  }),
  nav_move: (request) => {
    const step = navMove(request.args.verb.verb);
    return {
      reply: "nav" as const,
      cursor: moveCursor(step.row, "nav"),
      outcome: step.outcome,
      edge: step.edge,
      reason: null,
      included: null,
      earcon: step.outcome === "boundary" ? "E-BOUNDARY" : null,
      highlight: null,
      announced: true,
    };
  },
  nav_where: () => ({ reply: "announce" as const, queued: true }),
  modality_toggle: (request) => toggle(request.args.modality),
  choices_reset: () => {
    settings = { ...settings, choices: {} };
    transport = { ...transport, muted: false };
    say("Toggles reset to your profile's defaults.");
    return { reply: "effective" as const, settings: effective() };
  },
  settings_effective: () => ({
    reply: "effective" as const,
    settings: effective(),
  }),
  announce_repeat: () => ({ reply: "announce" as const, queued: true }),
  cursor_set: (request) => ({
    reply: "cursor" as const,
    ...moveCursor(request.args.row, request.args.origin),
  }),
  cursor_get: () => ({ reply: "cursor" as const, ...cursor }),
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
    view_get: (request) =>
      request.command === "view_get"
        ? viewTile(request.args.pxWidth, request.args.range)
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
  /** Moves the cursor as another view would, such as the plot (B04-T06). */
  moveCursor,
  /** The cursor as the mock host holds it. */
  cursor(): CursorState {
    return cursor;
  },
  /** The x ranges of the plot's `view_get` requests, oldest first (`null` for all of x). */
  viewRanges(): ({ start: number; end: number } | null)[] {
    return host.requests.flatMap((request) =>
      request.command === "view_get" ? [request.args.range] : [],
    );
  },
  /** The commands the UI sent through `ipc_dispatch`, oldest first. */
  commands(): string[] {
    return host.requests.map((request) => request.command);
  },
  /** The verbs of the explorer's `nav_move` requests, oldest first, as JSON. */
  navVerbs(): string[] {
    return host.requests.flatMap((request) =>
      request.command === "nav_move" ? [JSON.stringify(request.args.verb)] : [],
    );
  },
  /** What the explorer's `modality_toggle` requests flipped, oldest first. */
  toggles(): string[] {
    return host.requests.flatMap((request) =>
      request.command === "modality_toggle" ? [request.args.modality] : [],
    );
  },
  /** The transport as the mock host holds it. */
  transport(): TransportState {
    return transport;
  },
  /** The settings as the UI last saved them. */
  settings(): Settings {
    return settings;
  },
});
