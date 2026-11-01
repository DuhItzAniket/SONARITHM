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
| HW-01 | Owner's laptop, the development machine of [12 §8](../architecture/12-cross-cutting-concerns.md#8-performance-budgets) (much faster than the Windows reference machine) | Intel i7-14650HX, 8 P-cores + 8 E-cores, 24 threads | 16 GB | Windows 11 Home 10.0.26300, AC power, Balanced plan | Realtek(R) Audio, WASAPI shared, 48 kHz; OEM enhancement APO active (Nahimic installed) | Intel UHD Graphics (default) and NVIDIA RTX 4050 Laptop 6 GB; 2560 × 1600 panel at 60 Hz |
| HW-01e | HW-01 pinned to 4 E-cores (`start /affinity F0000`; child processes inherit it). A conservative stand-in for the 4-core reference laptop, used for the 12 §8 headroom rule | 4 Gracemont E-cores | 16 GB | As HW-01 | As HW-01 | As HW-01 |
| CI-LNX | GitHub-hosted runner, virtual machine | x86-64 | — | `ubuntu-24.04` image 20261012 | none | none (Xvfb, software rendering) |
| CI-WIN | GitHub-hosted runner, virtual machine | x86-64 | — | `windows-2025-vs2026` image 20261007 | none | none |
| CI-MAC | GitHub-hosted runner, virtual machine | Apple silicon (AArch64) | — | `macos-26-arm64` image 20261013 | none | none |

No reference-class machine is available yet (risk R-22). Rows on HW-01 apply the headroom rule of 12 §8.

## Audio engine

| Phase / release | Date | Tag or commit | HW ID | Buffer @ rate | Latency p50 (ms) | Latency p95 (ms) | Xruns in soak (duration) | Real-time factor (×) | Null residual (dBFS) | LUFS Δ | Score IR identical across OS | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| A02 spike (Windows) | 2026-10-19 | [#9](https://github.com/DuhItzAniket/SONARITHM/pull/9) `705dc67`; [#98](https://github.com/DuhItzAniket/SONARITHM/pull/98), [#103](https://github.com/DuhItzAniket/SONARITHM/pull/103) | HW-01 | 480-frame period (256 requested) @ 48 kHz, cpal 0.18.2 | 116.4 | 123.1 | 0 deadline misses, 0 late callbacks (120 s, owned graph and Firewheel); 7 and 12 late callbacks, 0 misses on HW-01e | 361 owned, 337 Firewheel (256-frame blocks, 1 thread) | −∞ against CI-LNX (bit-identical) | 0.0000 | yes | Latency: loopback method, software path only, F-031. Graph: F-034, F-035. Determinism: F-039 ([#104](https://github.com/DuhItzAniket/SONARITHM/pull/104)) |
| A02 spike (Windows, RAW stream) | 2026-10-19 | [#9](https://github.com/DuhItzAniket/SONARITHM/pull/9) `705dc67` | HW-01 | 480-frame period @ 48 kHz, direct WASAPI with `AUDCLNT_STREAMOPTIONS_RAW` | 79.2 | 83.7 | — | — | — | — | — | Bypasses the OEM enhancement APO; not available through cpal 0.18 (F-032) |
| A02 spike (macOS) | 2026-10-19 | [#104](https://github.com/DuhItzAniket/SONARITHM/pull/104) | CI-MAC | — | pending | pending | — | — | −∞ against CI-LNX (bit-identical) | 0.0000 | yes | Latency needs a loopback device and a port of the harness (R-22); determinism F-039 |
| A02 spike (Linux) | 2026-10-19 | [#104](https://github.com/DuhItzAniket/SONARITHM/pull/104) | CI-LNX | — | pending | pending | — | — | reference | reference | yes | Latency harness not ported; determinism F-039 |
| A05 (Windows, WASAPI loopback) | 2026-10-19 | A05-T13 PR | HW-01 | 480-frame period, 1056-frame buffer @ 48 kHz, `WasapiBackend` RAW shared | 99.2 | 104.0 | 0 xruns (200 cues, about 95 s) | — | — | — | — | Loopback self-test through `Command::Cue`; non-RAW shared: p50 138.5, p95 145.6. Calibration offset 82.6 ms (RAW). Software path only (F-052); cable run pending (`docs/engineering/manual-tests/a05-cable-latency.md`) |
| A05 (Windows, offline render) | 2026-10-19 | A05-T12 PR | HW-01 / HW-01e | 256-frame blocks @ 48 kHz, 1 thread | — | — | — | 91.4 (criterion), 92–96 (`render_speed`); **43–46 on HW-01e** | — | — | — | DS-006 benchmark Score (4 tracks, 10,000 events, 60 s). Above 50% of the PB-03 budget on HW-01, and below 50× on the CPU-limited rerun (F-053) |
| A05 (Windows, device soak) | 2026-10-19 | A05-T10 PR | HW-01 | 480-frame period, 1056-frame buffer @ 48 kHz, `WasapiBackend` RAW shared | — | — | 0 (10 min, 60,002 callbacks) | — | — | — | — | Silent (−120 dB master), DS-006 Score looped; 0 late callbacks, 0 allocating callbacks (F-055) |
| A05 (Windows, device soak) | 2026-10-19 | A05-T10 PR | HW-01 | 480 frames (128 requested) @ 48 kHz, cpal 0.18.2 | — | — | not reported by cpal's WASAPI output (10 min, 60,001 callbacks) | — | — | — | — | 0 late callbacks, 0 allocating callbacks (F-055) |
| A05 (Windows, device soak) | 2026-10-19 | A05-T10 PR | HW-01 | 144-frame period @ 48 kHz, `WasapiBackend` exclusive | — | — | 0 (10 min, 200,031 callbacks) | — | — | — | — | 20,396 late callbacks from the driver's event clustering, which also queues about 100 ms (F-051); 0 allocating callbacks (F-055) |
| A05 (null device, 128 frames) | 2026-10-19 | A05-T10 PR | HW-01 / HW-01e | 128 frames @ 48 kHz, timer-driven null device at real-time priority | — | — | 194 / 153 missed timer deadlines (10 min each, about 225,000 callbacks) | — | — | — | — | The same loop without rendering misses 0.05% of deadlines (worst wake-up 3.4 ms late); per-block render p99.9 106–120 µs, max 382 µs of 2,667 µs on HW-01 (`block_times`). HW-01's driver has no 128-frame period (F-030), so PB-02 at 128 frames is not declared met (F-055) |
| A05 (cross-OS golden) | 2026-10-21 | [#178](https://github.com/DuhItzAniket/SONARITHM/pull/178) | CI-LNX, CI-WIN, CI-MAC | 256-frame blocks @ 48 kHz (offline) | — | — | — | — | −240 (identical; the comparison's floor) against CI-LNX, all six cases | 0.0000 | yes | `golden-compare` in CI run 39509526921; PB-06 met (F-056) |
| A05 | | | | | | | | | | | | |
| A08 (replication gallery, end to end) | 2026-10-21 | A08-T02 branch `0103de8` (on `2af548f`); A08-T04 PR | HW-01; HW-01e | offline, 256-frame blocks @ 48 kHz, one thread | — | — | — | 131.7 (HW-01; 130.5–132.3), 70.9 (HW-01e; 70.3–71.3): compile + render + 24-bit WAV + loudness meter, 443.8 s of audio in 49 renders, 5 runs each | −∞ across CI-WIN, CI-LNX, CI-MAC (WAV SHA-256 identical) | 0 | yes (release-independent hashes) | `cargo run --release -p xtask -- replication --timings`; raw tables in [benchmarks/a08-replication-timings](benchmarks/a08-replication-timings/); [report](benchmark-report-a08.md); F-115, F-116 |
| B (PB-03 rerun, #185) | 2026-10-25 | #185 PR | HW-01 / HW-01e | 256-frame blocks @ 48 kHz, 1 thread | — | — | — | **HW-01e: 59–64 (`render_speed`, 3 rounds), 60.4 (criterion, 0.99 s)**; A05 binary alternated with it: 40–41 and 39–40 (1.51–1.55 s). HW-01: 107 (`render_speed`), 87 (criterion, 689 ms); A05 binary: 72 and 62 (974 ms) | Identical to the A05 render (FNV-1a over the 60 s benchmark render) | — | yes | DS-006 benchmark Score. PB-03 met on the four-E-core stand-in. Another build session was using the machine during the measurements, so both sides of each A/B run below A05's 43–46× and 91×. F-134 |
| A07 `v0.1.0-alpha` | | | | | | | | | | | | |
| A08 `v0.2.0-alpha` | | | | | | | | | | | | |
| B03 | | | | | | | | | | | | |
| B06 (1-hour stream) | 2026-10-25 | B06-T07 PR | HW-01 | 256-frame blocks @ 48 kHz, timer-driven null device | — | — | 148 late blocks on the null device (1 h; not device underruns) | — | — | — | — | PB-11 on the null device: memory flat after the warm-up (26.1 → 26.2 MiB peak), 3,601,204 rows written and received, playback kept up; a 10 s, 10,000-row window compiles in 8.2 ms median (F-144); samples in [benchmarks/b06-stream-soak](benchmarks/b06-stream-soak/). The device xrun run is an owner step (#129) |
| Gate B `v0.5.0-beta` | | | | | | | | | | | | |
| Gate C `v1.0.0` | | | | | | | | | | | | |

## Compiler and data

| Phase / release | Date | Tag or commit | HW ID | Compile 100k × 4 (ms) | Compile 1M rows (ms) | 1M-row CSV load (s) | Evidence |
|---|---|---|---|---|---|---|---|
| A04 | 2026-10-19 | A04 PRs (`feat/a04-bench-fuzz`) | HW-01; HW-01e | — | — | 0.506 (HW-01), 0.912 (HW-01e); 0.433 and 0.792 without semantic inference | PB-05 met: 34% of 1.5 s on HW-01, under the 12 §8 headroom threshold; 61% on the 4-E-core stand-in. criterion `pb05_csv_ingest_1m_rows_5_columns`, 46 MB synthetic input, F-070 |
| A06 | 2026-10-19 | A06-T09 PR (`benches/compile.rs`) | HW-01; HW-01e | 35.1 (HW-01); 54.3 (HW-01e) | 1 track: 135 (HW-01), 192 (HW-01e); 4 tracks: 362 (HW-01), 545 (HW-01e) | — | Criterion 0.8.2, release, Rust 1.99.0, 20 samples, medians; data preloaded (PB-05 covers loading); 4 tone tracks, default mappings. F-083 |
| A07 `v0.1.0-alpha` | | | | | | | |
| A08 `v0.2.0-alpha` | | | | | | | |
| Gate B `v0.5.0-beta` | | | | | | | |
| Gate C `v1.0.0` | | | | | | | |

## Desktop application

| Phase / release | Date | Tag or commit | HW ID | Webview | Cold start (s) | Playhead fps | Scripting per frame, 1M points (ms) | Installer size (MB) | Network calls without consent | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| A02 spike (WebView2) | 2026-10-19 | [#101](https://github.com/DuhItzAniket/SONARITHM/pull/101), [#102](https://github.com/DuhItzAniket/SONARITHM/pull/102) | HW-01 (Intel UHD and RTX 4050) | WebView2 155.0.4283.62 | — | 60 (panel limit) on both GPUs; full 1M redraw every frame: 37 on Intel UHD | ≤ 0.6 | — | — | PB-09: 1 MB binary p95 11.5–11.8 ms on HW-01, 15.7–20.7 ms on HW-01e, 7.8 ms on CI-WIN (F-040, F-041); PB-08: F-043 |
| A02 spike (WKWebView) | 2026-10-19 | [#101](https://github.com/DuhItzAniket/SONARITHM/pull/101) | CI-MAC | WKWebView (WebKit 21624.5.1.11.3) | — | pending | pending | — | — | PB-09: 1 MB binary p95 8 ms (F-040). Frame rates need a GPU: owner or volunteer step |
| A02 spike (WebKitGTK) | 2026-10-19 | [#101](https://github.com/DuhItzAniket/SONARITHM/pull/101) | CI-LNX | WebKitGTK 2.52.6 (Xvfb) | — | pending | pending | — | — | PB-09: 1 MB binary p95 7 ms (F-040). Frame rates on Intel, AMD and NVIDIA GPUs: owner live-USB step |
| B01 (B01-T04) | pending | — | CI-WIN, CI-MAC, CI-LNX (debug build); HW-01 release run pending (owner) | per CI job | — | — | — | — | — | PB-09: the desktop smoke run reports the p50, p95 and max of 30 `view_get` round trips of a cached 1.2 MB M4 view (`pb09` in `smoke.json`, CI step summary). No number yet: filled in from the first CI run, and on HW-01 by the owner |
| B01 (B01-T07) | pending | — | CI-WIN, CI-MAC, CI-LNX (debug in ci.yml, release in desktop-nightly); HW-01 release run pending (owner) | per CI job | pending: `pb07` in `smoke.json`, from the host's start to the first focus in a region (an upper bound; process creation before `main` not counted) | — | — | — | — | PB-07: no number yet. Filled in from CI's smoke runs and the owner's HW-01 run |
| B01 | | | | | | | | | | |
| B04 (B04-T08), WebView2 | 2026-11-01 | [debug](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649561), [release](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649558) at `e1f0fedf` (#439) | CI-WIN (windows-latest); HW-01 run pending (owner, #129) | WebView2 | — | 64.0 debug, 64.0 release (p95 frame interval 15.7 ms) | debug: p50 5.9, p95 15.6, max 19.1 (35 of 60 redraws before the 8 s cap); release: p50 5.4, p95 13.7, max 21.8 (44 redraws, capped) | — | — | PB-08 smoke check `pb-08-dense-layer`, `backend` webgl2. Over the 8 ms budget on a shared runner, an upper bound (F-212) |
| B04 (B04-T08), WKWebView | 2026-11-01 | [debug](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649561), [release](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649558) at `e1f0fedf` (#439) | CI-MAC (macos-latest) | WKWebView | — | 59.0 debug, 60.1 release (p95 interval 19 ms) | debug: p50 10, p95 26, max 29; release: p50 11, p95 21.0, max 32 (60 redraws each) | — | — | As above, `backend` webgl2. Over budget on a shared runner (F-212) |
| B04 (B04-T08), WebKitGTK | 2026-11-01 | [debug](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649561), [release](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649558) at `e1f0fedf` (#439) | CI-LNX (ubuntu-24.04, Xvfb) | WebKitGTK | — | 60.2 debug, 60.3 release (p95 interval 18 ms) | debug: p50 93, p95 110, max 181; release: p50 70, p95 87, max 147 (60 redraws each) | — | — | As above. `backend` reports webgl2, but Xvfb has no GPU, so WebGL is drawn in software: an upper bound, not a WebKitGTK figure (F-212). Real-GPU Linux run: HP-02 |
| Gate B `v0.5.0-beta` | | | | | | | | | | |
| Gate C `v1.0.0` | | | | | | | | | | |

## Chart extraction accuracy

PLOTSONA's accuracy on the DS-007 corpus, scored by the harness in `crates/engines/sonarithm-plotsona/src/accuracy.rs` (the CI job `chart-accuracy` uploads the reports). Errors are relative to the chart's largest value for calibrated and estimated tiers, and to each value for exact tiers.

| Phase | Date | Commit | Corpus | Charts | Worst error | Lowest RMS-F1 | Bar | Evidence |
|---|---|---|---|---|---|---|---|---|
| C01 (exact and vector tiers) | 2026-10-26 | [#385](https://github.com/DuhItzAniket/SONARITHM/pull/385) | DS-007 Office, chart JSON, SVG, vector PDF | 67 | 0.147 % (largest error, vector) | 0.9986 | 0.5 % per value | [F-167](findings-log.md#f-167--all-67-corpus-charts-extract-with-the-right-class-within-015--and-an-rms-f1-of-at-least-0998) |
| C02 (raster tier, clean renders) | 2026-10-30 | C02-T08 PR | DS-007 raster: line and bar charts at 1x, 1.5x, 2x and 1x crisp | 24 renders | 0.14 % (median per render) | 0.994 | 2 % median | [F-183](findings-log.md#f-183--clean-raster-line-and-bar-charts-read-within-014--median-error-at-every-resolution-but-jpeg-artefacts-break-automatic-series-colours) |
| C02 (raster tier, JPEG q90 and q60) | 2026-10-30 | C02-T08 PR | The same renders re-encoded in memory | 36 renders | bars mostly within 0.12 %; most line renders lose series | down to 0 | reported only | F-183 |

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
- **End-to-end export (A08).** `cargo run --release -p xtask -- replication --timings` times, per replication-gallery spec, the compile (including the CSV load) and the export (plan, offline render, WAV file, EBU R128 and true-peak metering, manifest) on one thread; the real-time factor is audio duration over their sum. Run it five times on all cores and five times pinned to the four E-cores (`ProcessorAffinity = 0xF0000`), with no compiler running, and report medians and ranges ([report](benchmark-report-a08.md#method)).
- **Compile time.** Criterion benchmarks on the DS-006 inputs; report the median.
- **Determinism.** Compare Score IR hashes across the CI matrix. For audio, compute the null-test residual and the integrated-loudness difference (EBU R128) between renders, as [ADR-0013](../decisions/0013-determinism-and-provenance.md) defines them. Store SHA-256 hashes of renders for same-platform regression checks.
- **UI.** Measure playhead frame rate and scripting time per frame with a 1M-point series, downsampled in Rust, in each webview (WebView2, WKWebView, WebKitGTK). B04-T08: the desktop smoke run's `pb-08-dense-layer` check does this in the app's own webview. It redraws the dense layer 60 times (each a zoom, so the data is drawn again) and times the script per frame, then moves a playhead by transform for 120 frames over the drawn layer and takes the frame rate from the animation frames' timestamps. Each phase stops after 8 s (`cutShort`). It reports `backend` (`webgl2` or `canvas2d`), so a software fallback is visible. Record only measured values; CI's numbers are upper bounds.
- **Startup.** Cold start to interactive after a reboot or with caches cleared; state which.
- **Installer.** Size of each platform installer without ML models.
- **Chart extraction accuracy.** `cargo test -p sonarithm-plotsona --lib accuracy -- --nocapture`; set `SONARITHM_ACCURACY_REPORT` to write the tables. Raster renders are read through anchors taken from the SVG's axes and scaled to the image; raster series are paired with truth series by least error, because the raster tier names them `Series 1`, `Series 2`…
- **Evidence column.** Link the CI run, the committed benchmark spec and the manifest. A row without evidence is not valid.

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline tables and budgets from architecture overview §7 and roadmap exit criteria. | Research lead |
| 2026-10-19 | A02 spike rows; hardware IDs HW-01, HW-01e, CI-LNX, CI-WIN and CI-MAC. | A02 session (AI agent), for review |
| 2026-10-21 | A08 row: end-to-end export of the replication gallery; method note; [A08 report](benchmark-report-a08.md). | A08 session (AI agent), for review |
| 2026-10-25 | PB-03 rerun after the voice-loop work (#185): met on HW-01e. | AI agent (Claude Code), for review |
| 2026-10-25 | B06 row: the 1-hour stream soak on the null device (PB-11 memory and throughput; F-144). | Lane R session (AI agent), for review |
| 2026-10-30 | Chart extraction accuracy section: the C01 row (F-167) and the C02 raster rows (F-183). | PLOTSONA session (AI agent), for review |
