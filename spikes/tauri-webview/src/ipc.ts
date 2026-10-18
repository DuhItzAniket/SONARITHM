// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// A02-T05: Tauri IPC benchmark (PB-09: a 1 MB binary view in ≤ 30 ms on every OS; 09 §2).

import { Channel, invoke } from "@tauri-apps/api/core";
import { summarize, timed, type Summary } from "./stats";

const MB = 1024 * 1024;
/** Wall clock in ms, comparable with the host's `epoch_ms` (both read the system clock). */
const wall = () => performance.timeOrigin + performance.now();

/** A binary `ipc::Response` round trip. Also checks that host and webview clocks agree: the
 *  host's stamp must fall between the request and the response. */
async function binary(bytes: number, n: number) {
  let early = Infinity;
  let late = Infinity;
  const times = await timed(n, 3, async () => {
    const t0 = wall();
    const buf = await invoke<ArrayBuffer>("bench_binary", { bytes });
    const t1 = wall();
    if (buf.byteLength !== Math.max(bytes, 8)) throw new Error(`got ${buf.byteLength} bytes`);
    const stamp = new DataView(buf).getFloat64(0, true);
    early = Math.min(early, stamp - t0);
    late = Math.min(late, t1 - stamp);
  });
  return { ...summarize(times), clockCheckMinMs: [Math.round(early * 1e3) / 1e3, Math.round(late * 1e3) / 1e3] };
}

interface ChannelResult extends Summary {
  messages: number;
  mbPerS: number;
  msgsPerS: number;
}

/** `count` binary Channel messages of `size` bytes: one-way latency and throughput. */
function channel(size: number, count: number, intervalUs: number): Promise<ChannelResult> {
  return new Promise((resolve, reject) => {
    const latency: number[] = [];
    let firstSent = 0;
    const ch = new Channel<ArrayBuffer>();
    ch.onmessage = (msg) => {
      const now = wall();
      const sent = new DataView(msg).getFloat64(8, true);
      if (latency.length === 0) firstSent = sent;
      latency.push(now - sent);
      if (latency.length === count) {
        const seconds = (now - firstSent) / 1000;
        resolve({
          ...summarize(latency),
          messages: count,
          mbPerS: Math.round(((size * count) / MB / seconds) * 10) / 10,
          msgsPerS: Math.round(count / seconds),
        });
      }
    };
    setTimeout(() => reject(new Error(`channel: ${latency.length} of ${count} messages in 60 s`)), 60_000);
    invoke("bench_channel", { channel: ch, size, count, intervalUs }).catch(reject);
  });
}

export async function runIpc(progress: (s: string) => void): Promise<Record<string, unknown>> {
  const r: Record<string, unknown> = {};
  progress("Command round trip");
  r.ping = summarize(await timed(500, 20, () => invoke("bench_ping")));
  progress("Binary response, 1 MB");
  r.binary1MB = await binary(MB, 100);
  progress("Binary response, 10 MB");
  r.binary10MB = await binary(10 * MB, 30);
  progress("JSON response, 1 MB");
  r.json1MB = summarize(await timed(10, 1, () => invoke("bench_json", { bytes: MB })));
  progress("Channel burst, 500 × 64 KB");
  r.channelBurst64KB = await channel(64 * 1024, 500, 0);
  progress("Channel at 30 Hz, 300 × 1 KB");
  r.channel30Hz1KB = await channel(1024, 300, 33_333);
  return r;
}
