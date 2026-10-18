// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import { invoke } from "@tauri-apps/api/core";
import { runIpc } from "./ipc";
import { runWebgl } from "./webgl";

type Suite = (progress: (s: string) => void) => Promise<Record<string, unknown>>;
const suites: Record<string, Suite> = { ipc: runIpc, webgl: runWebgl };

const status = document.querySelector<HTMLElement>("#status");
const output = document.querySelector<HTMLElement>("#output");
const say = (s: string) => {
  if (status) status.textContent = s;
};

async function run(name: string) {
  const suite = suites[name];
  let results: Record<string, unknown>;
  try {
    if (!suite) throw new Error(`unknown suite ${name}`);
    say(`Running ${name}…`);
    results = { suite: name, ok: true, userAgent: navigator.userAgent, ...(await suite(say)) };
    say(`${name} finished.`);
  } catch (e) {
    results = { suite: name, ok: false, error: String(e) };
    say(`${name} failed: ${String(e)}`);
  }
  if (output) output.textContent = JSON.stringify(results, null, 2);
  await invoke("report", { results });
}

for (const button of document.querySelectorAll<HTMLButtonElement>("button[data-suite]")) {
  button.addEventListener("click", () => void run(button.dataset.suite ?? ""));
}

const auto = await invoke<string | null>("autorun");
if (auto) void run(auto);
