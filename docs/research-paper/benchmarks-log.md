# Benchmarks log

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead (records); architecture lead (measurements).
> **Related:** [architecture overview §7](../architecture/01-overview.md) · [evaluation plan](evaluation-plan.md#technical-evaluation) · [roadmap](../planning/roadmap.md) · [findings log](findings-log.md) · [ADR-0013](../decisions/0013-determinism-and-provenance.md) · [datasets](datasets.md) (DS-006)

This log records engineering metrics per phase and release against the budgets. It is append-only: add rows, never overwrite them. Each row links to the manifest or CI run that produced it. A result that misses a budget, or changes by more than the run-to-run spread, is also logged as a finding (`F-###`, type `benchmark`). Engine numbers feed P01 and P07.

## Budgets

Targets for reference hardware, from [architecture overview §7](../architecture/01-overview.md) and the phase exit criteria in the [roadmap](../planning/roadmap.md). Budgets are re-baselined at each stage gate; a change is recorded in the change log below.

| Metric | Budget | Verified in |
|---|---|---|
| Interactive response, key press → cue onset | ≤ 50 ms p95 at 48 kHz with a 256-frame buffer | A02, B03 |
| Audio robustness | 0 xruns in a 10-minute soak at 128-frame buffers | A05 |
| Offline render speed (real-time factor) | ≥ 50× real time for a 4-track, 10k-event Score | A05 |
| Compile time | ≤ 200 ms for 100k rows × 4 tracks; ≤ 1 s for 1M rows | A06 |
| Determinism | Identical Score IR across OS/CPU; null-test residual ≤ −90 dBFS; LUFS Δ ≤ 0.1 | A05 |
| Data load | 1M-row CSV in ≤ 1.5 s | A04 |
| Stream endurance | 1 kHz input stream for 1 hour with no xruns and bounded memory | B06 |
| UI frame time | 60 fps playhead; ≤ 8 ms scripting per frame with a 1M-point series | B04 |
| Startup | Cold start to interactive ≤ 2 s | B01 |
| Installer size | ≤ 40 MB without ML models | C06 |
| Privacy | Zero network calls without explicit consent | B01 |

## Reference hardware

To be defined at A02 (open question OQ-R5 in the [evaluation plan](evaluation-plan.md#open-questions)). Give each machine an ID and keep its record here; rows cite the ID.

| HW ID | Machine | CPU | RAM | OS and version | Audio device / host | GPU (UI metrics) |
|---|---|---|---|---|---|---|
| | | | | | | |

## Audio engine

| Phase / release | Date | Tag or commit | HW ID | Buffer @ rate | Latency p50 (ms) | Latency p95 (ms) | Xruns in soak (duration) | Real-time factor (×) | Null residual (dBFS) | LUFS Δ | Score IR identical across OS | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| A02 spike (Windows) | | | | | | | | | | | | |
| A02 spike (macOS) | | | | | | | | | | | | |
| A02 spike (Linux) | | | | | | | | | | | | |
| A05 | | | | | | | | | | | | |
| A07 `v0.1.0-alpha` | | | | | | | | | | | | |
| A08 `v0.2.0-alpha` | | | | | | | | | | | | |
| B03 | | | | | | | | | | | | |
| B06 (1-hour stream) | | | | | | | | | | | | |
| Gate B `v0.5.0-beta` | | | | | | | | | | | | |
| Gate C `v1.0.0` | | | | | | | | | | | | |

## Compiler and data

| Phase / release | Date | Tag or commit | HW ID | Compile 100k × 4 (ms) | Compile 1M rows (ms) | 1M-row CSV load (s) | Evidence |
|---|---|---|---|---|---|---|---|
| A04 | | | | | | | |
| A06 | | | | | | | |
| A07 `v0.1.0-alpha` | | | | | | | |
| A08 `v0.2.0-alpha` | | | | | | | |
| Gate B `v0.5.0-beta` | | | | | | | |
| Gate C `v1.0.0` | | | | | | | |

## Desktop application

| Phase / release | Date | Tag or commit | HW ID | Webview | Cold start (s) | Playhead fps | Scripting per frame, 1M points (ms) | Installer size (MB) | Network calls without consent | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| A02 spike (WebView2) | | | | | | | | | | |
| A02 spike (WKWebView) | | | | | | | | | | |
| A02 spike (WebKitGTK) | | | | | | | | | | |
| B01 | | | | | | | | | | |
| B04 | | | | | | | | | | |
| Gate B `v0.5.0-beta` | | | | | | | | | | |
| Gate C `v1.0.0` | | | | | | | | | | |

## Continuous integration

The A01 research hook records CI times as a baseline.

| Phase / release | Date | Commit | Workflow | Wall time Linux (min) | Wall time Windows (min) | Wall time macOS (min) | Evidence |
|---|---|---|---|---|---|---|---|
| A01 baseline | | | `ci.yml` | | | | |

## Method notes

- **Environment.** Record the HW ID, OS version, audio host and driver, buffer size, sample rate, power plan, and whether other load was present. Close other audio applications.
- **Repetition.** Report the median and the stated percentile over at least the run count given in the benchmark script. Write the run count in the evidence link or manifest.
- **Latency.** Measure key press to cue onset on a loopback capture: timestamp a synthetic key event, detect the cue onset in the captured audio, and compute the difference. Report p50 and p95. Use the harness from A02 and B03 so that results are comparable across phases.
- **Soak.** Ten minutes of continuous playback of the benchmark Score at 128-frame buffers; count xruns as the backend reports them (cpal provides xrun reporting). B06 uses a 1 kHz input stream for one hour and also records peak memory.
- **Real-time factor.** Audio duration divided by wall-clock time for an offline render of the 4-track, 10k-event Score in DS-006. State the thread count.
- **Compile time.** Criterion benchmarks on the DS-006 inputs; report the median.
- **Determinism.** Compare Score IR hashes across the CI matrix. For audio, compute the null-test residual and the integrated-loudness difference (EBU R128) between renders, as [ADR-0013](../decisions/0013-determinism-and-provenance.md) defines them. Store SHA-256 hashes of renders for same-platform regression checks.
- **UI.** Measure playhead frame rate and scripting time per frame with a 1M-point series, downsampled in Rust, in each webview (WebView2, WKWebView, WebKitGTK).
- **Startup.** Cold start to interactive after a reboot or with caches cleared; state which.
- **Installer.** Size of each platform installer without ML models.
- **Evidence column.** Link the CI run, the committed benchmark spec and the manifest. A row without evidence is not valid.

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline tables and budgets from architecture overview §7 and roadmap exit criteria. | Research lead |
