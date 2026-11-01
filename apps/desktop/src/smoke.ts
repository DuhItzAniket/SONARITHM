// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { IpcTransport } from "@sonarithm/ipc";
import { connect, readView } from "@sonarithm/ipc";
import { invoke } from "@tauri-apps/api/core";

/** What the host passes to a smoke run (`src-tauri/src/smoke.rs`). */
export interface SmokeConfig {
  /** The fixture CSV the host wrote. */
  readonly dataset: string;
  readonly rows: number;
  /** When the host started, in milliseconds since the Unix epoch (PB-07). */
  readonly startedAtMs: number | null;
}

/** One check's outcome. */
interface Check {
  readonly name: string;
  readonly ok: boolean;
  readonly detail: string;
}

/** The smoke configuration the host injected, if this is a smoke run. */
export function smokeConfig(scope: object): SmokeConfig | null {
  const value: unknown = Reflect.get(scope, "__SONARITHM_SMOKE__");
  if (typeof value !== "object" || value === null) {
    return null;
  }
  const { dataset, rows, startedAtMs } = value as Record<string, unknown>;
  return typeof dataset === "string" && typeof rows === "number"
    ? {
        dataset,
        rows,
        startedAtMs: typeof startedAtMs === "number" ? startedAtMs : null,
      }
    : null;
}

/**
 * How long a smoke run may take before its watchdog reports the check still waiting:
 * well under CI's 5-minute step limit, and well over a normal run (a few seconds).
 */
export const SMOKE_WATCHDOG_MS = 120_000;

/** The binary view must reach this size to count as the F-042 check (1 MB). */
const BINARY_BYTES = 1_000_000;

/**
 * The CI smoke test (B01-T03): the empty workspace is shown and named, the protocol
 * handshake passes, and a 1 MB view arrives as an `ArrayBuffer` under the app's CSP
 * (F-042). The host adds its own checks, writes the report and exits.
 *
 * A check that never settles would hold the run until CI's step limit with nothing to read.
 * A watchdog sends a failing report after `SMOKE_WATCHDOG_MS` that names the check still
 * waiting and keeps the checks done so far. `public/smoke-guard.js` covers the time before
 * this function starts.
 */
export async function runSmoke(
  transport: IpcTransport,
  config: SmokeConfig,
): Promise<void> {
  Reflect.set(window, "__SONARITHM_SMOKE_STARTED__", true);
  const checks: Check[] = [];
  let waiting = "start";
  let reported = false;
  const send = async (results: object) => {
    if (reported) return;
    reported = true;
    await invoke("smoke_report", { results });
  };
  const watchdog = setTimeout(() => {
    void send({
      ok: false,
      checks: [
        ...checks,
        {
          name: waiting,
          ok: false,
          detail: `still waiting after ${String(SMOKE_WATCHDOG_MS / 1000)} s (watchdog)`,
        },
      ],
    });
  }, SMOKE_WATCHDOG_MS);
  const check = async (name: string, run: () => Promise<string>) => {
    waiting = name;
    try {
      checks.push({ name, ok: true, detail: await run() });
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      checks.push({ name, ok: false, detail });
    }
  };

  await check("bundled-origin", () => {
    const { protocol, hostname } = window.location;
    const bundled =
      (protocol === "tauri:" && hostname === "localhost") ||
      hostname === "tauri.localhost";
    return bundled
      ? Promise.resolve(`${protocol}//${hostname}`)
      : Promise.reject(new Error(`loaded from ${protocol}//${hostname}`));
  });

  await check("empty-workspace", async () => {
    const main = await waitFor(() =>
      document.querySelector('main [data-state="ready"]')?.closest("main"),
    );
    const labelId = main.getAttribute("aria-labelledby") ?? "";
    const name = document.getElementById(labelId)?.textContent ?? "";
    if (name === "") {
      throw new Error("the workspace region has no accessible name");
    }
    return name;
  });

  // PB-07 (B01-T07): from the host's start to an interactive workspace, which is the
  // focus in a region (09 §7). Reported, not enforced here: ci.yml runs a debug build on
  // shared runners, and desktop-nightly a release build. The benchmarks log records the
  // HW-01 runs.
  let pb07: Pb07 | null = null;
  await check("pb-07-cold-start", async () => {
    pb07 = await measurePb07(config);
    return `${pb07.coldStartMs.toFixed(0)} ms to an interactive workspace (budget ${String(PB07_BUDGET_MS)} ms)`;
  });

  await check("binary-view", async () => {
    const client = await connect(transport);
    const dataset = await client.call({
      command: "dataset_open",
      args: { path: config.dataset },
    });
    const buffer = await client.binary({
      command: "view_get",
      args: {
        dataset: dataset.id,
        x: "x",
        y: "y",
        range: null,
        pxWidth: Math.floor(config.rows * 0.9),
        method: "lttb",
      },
    });
    if (buffer.byteLength < BINARY_BYTES) {
      throw new Error(`only ${String(buffer.byteLength)} bytes`);
    }
    const view = readView(buffer);
    return `${String(buffer.byteLength)} bytes, ${String(view.rows)} rows`;
  });

  // PB-09 (B01-T04): a 1 MB binary view, request to ArrayBuffer, p95 over repeated calls.
  // Reported, not enforced here: CI runs a debug build on shared runners, so the number is
  // an upper bound. The benchmarks log records the release-build runs (HW-01).
  let pb09: Pb09 | null = null;
  await check("pb-09-binary-view", async () => {
    pb09 = await measurePb09(transport, config);
    return `p95 ${pb09.p95Ms.toFixed(1)} ms over ${String(pb09.samples)} calls of ${String(pb09.bytes)} bytes (budget ${String(PB09_BUDGET_MS)} ms)`;
  });

  // B03-T04: the native announcer route raises a UI Automation notification on Windows
  // (`UiaRaiseNotificationEvent` returns success); other hosts refuse, and the announcer
  // falls back to live regions.
  await check("native-announcer", async () => {
    const windows = navigator.userAgent.includes("Windows");
    try {
      await invoke("announce_native", {
        text: "SONARITHM smoke test",
        assertive: false,
      });
    } catch (error) {
      if (windows) {
        throw error;
      }
      return "not available on this platform (live regions are used)";
    }
    if (!windows) {
      throw new Error("a native route answered outside Windows");
    }
    return "UI Automation notification raised";
  });

  clearTimeout(watchdog);
  const results = { ok: checks.every((c) => c.ok), checks, pb07, pb09 };
  await send(results);
}

/** PB-07: 2 s from launch to an interactive workspace (12 §8). */
const PB07_BUDGET_MS = 2000;

/** The PB-07 measurement the smoke report carries. */
interface Pb07 {
  readonly coldStartMs: number;
  readonly budgetMs: number;
  readonly withinBudget: boolean;
}

/** Waits until the start-up focus is in a region, then measures from the host's start. */
async function measurePb07(config: SmokeConfig): Promise<Pb07> {
  if (config.startedAtMs === null) {
    throw new Error("the host passed no start time");
  }
  await waitFor(() => document.activeElement?.closest("[data-region]"));
  const coldStartMs = Date.now() - config.startedAtMs;
  return {
    coldStartMs,
    budgetMs: PB07_BUDGET_MS,
    withinBudget: coldStartMs <= PB07_BUDGET_MS,
  };
}

/** PB-09: 30 ms p95 for a 1 MB binary view (12 §8). */
const PB09_BUDGET_MS = 30;
/** Timed calls after one warm-up call. */
const PB09_SAMPLES = 30;

/** The PB-09 measurement the smoke report carries. */
interface Pb09 {
  readonly bytes: number;
  readonly samples: number;
  readonly p50Ms: number;
  readonly p95Ms: number;
  readonly maxMs: number;
  readonly budgetMs: number;
  readonly withinBudget: boolean;
}

/**
 * Times `view_get` round trips of an M4 view of every fixture row (one row per pixel), about
 * 1.2 MB. With more rows per pixel, M4 keeps only the rows that are a pixel's first, last,
 * minimum or maximum, which on the fixture's smooth sine is about two of every four, so the
 * view would fall under 1 MB. M4 tiles are cached in the app core after the warm-up call,
 * so the timed calls measure encoding, the IPC transfer and the arrival as an `ArrayBuffer`.
 */
async function measurePb09(
  transport: IpcTransport,
  config: SmokeConfig,
): Promise<Pb09> {
  const client = await connect(transport);
  const dataset = await client.call({
    command: "dataset_open",
    args: { path: config.dataset },
  });
  const request = {
    command: "view_get",
    args: {
      dataset: dataset.id,
      x: "x",
      y: "y",
      range: null,
      pxWidth: config.rows,
      method: "m4",
    },
  } as const;
  let bytes = (await client.binary(request)).byteLength;
  if (bytes < BINARY_BYTES) {
    throw new Error(`the view is only ${String(bytes)} bytes`);
  }
  const times: number[] = [];
  for (let i = 0; i < PB09_SAMPLES; i += 1) {
    const start = performance.now();
    bytes = (await client.binary(request)).byteLength;
    times.push(performance.now() - start);
  }
  times.sort((a, b) => a - b);
  const at = (q: number) =>
    times[Math.min(times.length - 1, Math.ceil(q * times.length) - 1)] ?? 0;
  const p95Ms = at(0.95);
  return {
    bytes,
    samples: times.length,
    p50Ms: at(0.5),
    p95Ms,
    maxMs: times.at(-1) ?? 0,
    budgetMs: PB09_BUDGET_MS,
    withinBudget: p95Ms <= PB09_BUDGET_MS,
  };
}

/** Resolves with `find()`'s element once it exists; rejects after 10 s. */
async function waitFor(
  find: () => Element | null | undefined,
): Promise<Element> {
  for (let waited = 0; waited < 10_000; waited += 50) {
    const element = find();
    if (element !== null && element !== undefined) {
      return element;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error("timed out");
}
