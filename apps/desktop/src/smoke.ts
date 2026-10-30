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
  const { dataset, rows } = value as Record<string, unknown>;
  return typeof dataset === "string" && typeof rows === "number"
    ? { dataset, rows }
    : null;
}

/** The binary view must reach this size to count as the F-042 check (1 MB). */
const BINARY_BYTES = 1_000_000;

/**
 * The CI smoke test (B01-T03): the empty workspace is shown and named, the protocol
 * handshake passes, and a 1 MB view arrives as an `ArrayBuffer` under the app's CSP
 * (F-042). The host adds its own checks, writes the report and exits.
 */
export async function runSmoke(
  transport: IpcTransport,
  config: SmokeConfig,
): Promise<void> {
  const checks: Check[] = [];
  const check = async (name: string, run: () => Promise<string>) => {
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

  const results = { ok: checks.every((c) => c.ok), checks };
  await invoke("smoke_report", { results });
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
