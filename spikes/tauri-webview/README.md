# Spike A02-T05..T07: Tauri 2.12 webview (IPC, WebGL2, accessibility)

> **Status:** Throwaway spike for the A02 prototype gate. Deleted or archived after the gate. Not production code.
> **Tasks:** A02-T05 (IPC), A02-T06 (WebGL2), A02-T07 (accessibility) in [implementation-plan §2](../../docs/planning/implementation-plan.md). **Decisions under test:** [ADR-0002](../../docs/decisions/0002-desktop-shell-tauri.md), [ADR-0003](../../docs/decisions/0003-frontend-react-and-react-aria.md). **Design:** [09 §2](../../docs/architecture/09-desktop-app.md#2-ipc-design).

One Tauri 2.12.4 app (Rust host in `src-tauri/`, Vite + TypeScript frontend in `src/`) with one page per suite. The host takes `--autorun <suite>` and `--out <file>`: the suite runs without interaction, the host adds OS, Tauri and webview versions, writes the JSON report and exits. CI uses the same mode. The `spike-a02-tauri` workflow builds the app on Linux, Windows and macOS, runs the IPC suite on each runner, and keeps the builds (`.deb`, `.app`, `.exe`) as artifacts for 30 days.

The spike has its own `pnpm-workspace.yaml`, so it stays out of the repository's pnpm workspace and its lint jobs.

## Run it

```bash
pnpm install
pnpm tauri build --no-bundle
./src-tauri/target/release/spike-tauri-webview --autorun ipc --out results/ipc.json
```

`pnpm tauri dev` runs it interactively; each suite has a button.

## A02-T05: IPC (PB-09)

`src/ipc.ts` against the host commands in `src-tauri/src/main.rs`:

| Test | What it does |
|---|---|
| `ping` | 500 round trips of a command with no payload |
| `binary1MB`, `binary10MB` | A command returning `tauri::ipc::Response` with 1 MB (100 runs) or 10 MB (30 runs), the bulk-data path of 09 §2 |
| `json1MB` | The same 1 MB returned as a JSON number array (10 runs), for contrast |
| `channel30Hz1KB` | 300 binary Channel messages of 1 KB at 30 Hz (the meter rate of 09 §2): one-way latency |
| `channelBurst64KB` | 500 binary Channel messages of 64 KB sent as fast as possible: throughput |

One-way Channel latency compares the host's wall clock with `performance.timeOrigin + performance.now()`. The binary test checks that the two clocks agree: the host's timestamp must fall between the request and the response (`clockCheckMinMs`, both values ≥ 0). Round-trip times use `performance.now()`, whose resolution in WebView2 is about 0.1 ms.

### Results (Windows, 2026-10-19)

Machine: the owner's laptop (i7-14650HX, 16 GB, Windows 11 build 26300, on AC power). Tauri 2.12.4, WebView2 155.0.4283.62, release build. Three runs on all cores and three runs pinned to 4 E-cores (`start /affinity F0000`; the WebView2 child processes inherit the mask, which was checked). The 4 E-cores are a conservative stand-in for the 4-core reference laptop of 12 §8. Raw reports are in [`results/`](results/).

| Test | All cores (3 runs) | 4 E-cores (3 runs) |
|---|---|---|
| Command round trip, p50 / p95 | 0.9 / 1.2 ms | 1.7–2.0 / 2.0–2.6 ms |
| **1 MB binary, p50 / p95** | **10.0–10.4 / 11.5–11.8 ms** | **13.9–15.5 / 15.7–20.7 ms** |
| 1 MB binary, max | 12.6–13.7 ms | 18.2–23.3 ms |
| 10 MB binary, p50 / p95 | 80.5–81.6 / 84.5–90.1 ms | 103–129 / 122–150 ms |
| 1 MB as JSON, p50 | 34.3–35.0 ms | 51.3–53.9 ms |
| Channel 1 KB at 30 Hz, one-way p50 / p95 / max | 1.8–1.9 / 2.1–2.4 / 2.6–6.1 ms | 3.0–3.7 / 3.9–5.4 / 6.7–9.8 ms |
| Channel burst of 64 KB messages, throughput | 147–156 MB/s | 49–56 MB/s |

**PB-09 (≤ 30 ms for a 1 MB binary view) is met on Windows.** The all-core p95 is 39% of the budget. On 4 E-cores it is 52–69% of the budget: above the 50% headroom line, but this run is the CPU-limited re-measurement that the headroom rule of 12 §8 asks for, and it still meets the budget. Other observations:

- Rust-to-webview throughput is about 100 MB/s for responses (only that direction was measured). A binary response costs about 10 ms per MB, so views and tiles should stay well under 1 MB; an M4 view for a 2,560-pixel-wide plot is 40 KB.
- JSON costs 3.4 times as much as binary for the same bytes, which confirms the 09 §2 rule "no JSON on hot paths".
- Channel messages arrive about 2 ms after they are sent. A burst queues: at 150 MB/s, 500 messages of 64 KB arrive 147 ms (p50) after their send stamp.
- **A strict CSP must allow the IPC origins.** With `default-src 'self'` and no `connect-src ipc: http://ipc.localhost`, Tauri silently falls back to its postMessage IPC: responses arrive as JSON arrays instead of `ArrayBuffer`s, and the binary path is lost. `tauri.conf.json` sets `connect-src` explicitly. The production CSP (09 §4) needs a test for this.

macOS and Linux numbers come from the CI runners (workflow summary and artifacts). They are virtual machines, not reference hardware: Linux runs WebKitGTK under Xvfb with software rendering.
