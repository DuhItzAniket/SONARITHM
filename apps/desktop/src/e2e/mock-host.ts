// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type {
  Channel,
  CursorOrigin,
  CursorState,
  DatasetSummary,
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

function moveCursor(
  row: number,
  origin: CursorOrigin,
  datasetId: string | null = dataset?.id ?? null,
): CursorState {
  cursor = {
    seq: cursor.seq + 1,
    dataset: datasetId,
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
 * The function commands (C03-T08b), simplified: an expression reads unless its brackets
 * do not balance or it ends with an operator, and every function opens with the same four
 * critical points. The app core's own tests cover reading and sampling; the E2E checks
 * what the UI does with the replies.
 */
function checkExpression(expr: string): {
  ok: boolean;
  canonical: string | null;
  diagnostics: {
    severity: string;
    code: string;
    rules: string[];
    message: string;
    pointer: string | null;
  }[];
  span: [number, number] | null;
} {
  const text = expr.trim().replace(/\s+/g, " ");
  // `@` stands for an error with no place (a budget or a variable count), `#` for a reply
  // with no diagnostic at all.
  if (text.includes("@")) {
    return {
      ok: false,
      canonical: null,
      diagnostics: [
        {
          severity: "error",
          code: "SNR-ENG-404",
          rules: [],
          message: "Too many variables.",
          pointer: null,
        },
      ],
      span: null,
    };
  }
  if (text.includes("#")) {
    return { ok: false, canonical: null, diagnostics: [], span: [1, 1] };
  }
  let depth = 0;
  let stop: number | null = null;
  for (let at = 0; at < expr.length; at++) {
    const char = expr.charAt(at);
    depth += char === "(" ? 1 : char === ")" ? -1 : 0;
    if (depth < 0) {
      stop = at;
      break;
    }
  }
  if (stop === null && (depth > 0 || /[-+*/^]$/.test(text))) {
    stop = expr.length;
  }
  if (stop === null) {
    return { ok: true, canonical: text, diagnostics: [], span: null };
  }
  return {
    ok: false,
    canonical: null,
    diagnostics: [
      {
        severity: "error",
        code: "SNR-ENG-400",
        rules: [],
        // As the host words it: the place is part of the message, counted from 1.
        message: `The expression does not read here (at character ${String(stop + 1)}).`,
        pointer: null,
      },
    ],
    // Like the host's span: in characters as the message counts them, from 1.
    span: [stop + 1, 1],
  };
}

function openFunction(args: {
  readonly expr: string;
  readonly domain: readonly [number, number];
  readonly samples: number | null;
}) {
  const canonical = checkExpression(args.expr).canonical ?? args.expr;
  const rows = args.samples ?? 1000;
  const field = (name: string) => ({
    name,
    dataType: "Float64",
    fieldType: "quantitative",
    role: name === "x" ? "x" : null,
    unit: null,
    concept: null,
  });
  const [min, max] = args.domain;
  return {
    reply: "function" as const,
    dataset: {
      id: "f1",
      name: `f(x) = ${canonical}`,
      rows,
      fields: [field("x"), field("y"), field("dy")],
      diagnostics: [],
    },
    function: {
      canonical,
      summary: `f(x) = ${canonical} from x = ${String(min)} to ${String(max)}: 2 zeros, 1 local maximum and 1 horizontal asymptote.`,
      criticalPoints: [
        {
          kind: "zero",
          x: -1,
          end: null,
          y: 0,
          row: 10,
          text: "zero at x = -1",
        },
        {
          kind: "local maximum",
          x: 0,
          end: null,
          y: 1,
          row: 20,
          text: "local maximum at x = 0, y = 1",
        },
        { kind: "zero", x: 1, end: null, y: 0, row: 30, text: "zero at x = 1" },
        {
          kind: "horizontal asymptote",
          x: max,
          end: null,
          y: 0,
          row: null,
          text: "horizontal asymptote y = 0",
        },
      ],
    },
  };
}

/**
 * The SDR-31 calculator over a series of 1000 samples at 100 Hz (10 s of data): T/k seconds
 * in the band [k/T, k·fs/2], with 461 or 462 when part or all of it is outside hearing.
 * The default k is 400, whose band, 40 Hz to 20 kHz, is all audible.
 */
function audifyReadout(wanted: number | null) {
  const fsHz = 100;
  const samples = 1000;
  const seconds = samples / fsHz;
  const k = wanted ?? 400;
  const band: [number, number] = [k / seconds, (k * fsHz) / 2];
  const low = Math.max(band[0], 20);
  const high = Math.min(band[1], 20_000);
  const audibleHz: [number, number] | null = low < high ? [low, high] : null;
  const codes =
    audibleHz === null
      ? ["SNR-ENG-462"]
      : band[0] < 20 || band[1] > 20_000
        ? ["SNR-ENG-461"]
        : [];
  return {
    reply: "audify_readout" as const,
    k,
    fsHz,
    samples,
    series: "y",
    durationS: seconds / k,
    bandHz: band,
    audibleHz,
    codes,
  };
}

const handlers: MockHandlers = {
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
    ...moveCursor(request.args.row, request.args.origin, request.args.dataset),
  }),
  function_check: (request) => ({
    reply: "function_check" as const,
    ...checkExpression(request.args.expr),
  }),
  function_open: (request) => openFunction(request.args),
  audify_calculate: (request) => audifyReadout(request.args.k),
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
  /** The args of the UI's `cursor_set` requests, oldest first, as JSON. */
  cursorSets(): string[] {
    return host.requests.flatMap((request) =>
      request.command === "cursor_set" ? [JSON.stringify(request.args)] : [],
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
