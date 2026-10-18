# A02 prototype gate: technical spikes

> **Status:** Draft (2026-10-19). **Gate:** A02 prototype gate. **Release:** none. **Decision:** Conditional go, recommended; awaiting the owner's sign-off and the pending manual steps.
> **Related:** [reviews README](README.md#the-a02-prototype-gate-report) · [roadmap A02](../../planning/roadmap.md#a02--technical-spikes-and-prototype-gate) · [implementation plan §2](../../planning/implementation-plan.md) · [risk register](../../planning/risk-register.md) · [findings F-030 to F-045](../../research-paper/findings-log.md#implementation-findings) · [benchmarks log](../../research-paper/benchmarks-log.md) · [worklog](../worklog/2026-10-19-a02-spikes.md)

This report follows the [gate review template](_gate-review-template.md) and adds the spike sections that the [reviews README](README.md#the-a02-prototype-gate-report) requires for A02. The spikes live in `spikes/` and are throwaway code. Each has a README with its method and raw results, and each landed (or is landing) through its own pull request:

| Spike | Task | Pull request | Code |
|---|---|---|---|
| Output latency | A02-T01 | [#9](https://github.com/DuhItzAniket/SONARITHM/pull/9) (merged, `705dc67`) | `spikes/cpal-latency/` |
| Owned render graph | A02-T02 | [#98](https://github.com/DuhItzAniket/SONARITHM/pull/98) | `spikes/render-graph/` |
| Firewheel behind the same contract, Miri | A02-T03 | [#103](https://github.com/DuhItzAniket/SONARITHM/pull/103), stacked on #98 | `spikes/render-graph/src/fw.rs` |
| Cross-OS determinism | A02-T04 | [#104](https://github.com/DuhItzAniket/SONARITHM/pull/104), stacked on #103 | `spikes/render-graph/src/determinism.rs`, `.github/workflows/spike-a02-determinism.yml` |
| Tauri IPC | A02-T05 | [#101](https://github.com/DuhItzAniket/SONARITHM/pull/101) | `spikes/tauri-webview/`, `.github/workflows/spike-a02-tauri.yml` |
| WebGL2 frame times | A02-T06 | [#102](https://github.com/DuhItzAniket/SONARITHM/pull/102), stacked on #101 | `spikes/tauri-webview/src/webgl.ts` |
| Accessibility prototype and walkthrough scripts | A02-T07 | [#107](https://github.com/DuhItzAniket/SONARITHM/pull/107), stacked on #102 | `spikes/tauri-webview/src/a11y/`, `spikes/tauri-webview/WALKTHROUGH.md` |

All measurements were taken on the owner's laptop (HW-01) or on GitHub-hosted runners (CI-LNX, CI-WIN, CI-MAC); the hardware records are in the [benchmarks log](../../research-paper/benchmarks-log.md#reference-hardware). HW-01 is much faster than the Windows reference machine. Where a result exceeded 50% of its budget, it was measured again with the CPU limited to 4 E-cores (HW-01e), as the headroom rule of [12 §8](../../architecture/12-cross-cutting-concerns.md#8-performance-budgets) requires. No number in this report is estimated; everything not measured is marked **pending**.

## Decision

| Item | Value |
|---|---|
| Stage | A · A02 prototype gate (inside Stage A) |
| Gate date | 2026-10-19 (draft) |
| Release | none |
| Decision | **Conditional go** (recommended; owner to decide) |

The Windows results support the Rust core, the owned render graph and the Tauri + React Aria stack:

- the render graph is fast, allocation-free and deterministic across three OSes and two CPU architectures;
- IPC and WebGL2 meet PB-08 and PB-09 in WebView2.

One budget fails: **PB-01 interactive latency**. Through cpal in WASAPI shared mode, key press to cue onset is about 123 ms p95 on the development laptop against 50 ms. That is a backend problem, not a graph problem (F-031). The shell and frontend decisions cannot be confirmed yet, because the deciding evidence is manual and on other OSes: WebKitGTK frame rates, Orca, VoiceOver, and NVDA and Narrator walkthroughs. A02 can close for the render-graph choice now. ADR-0002 and ADR-0003 remain provisional until the conditions below are met.

Conditions (conditional go):

| Condition | Owner | Due | Issue |
|---|---|---|---|
| NVDA and Narrator walkthroughs on HW-01 (WALKTHROUGH.md §1–2); OD-05 recommendation from steps 11–15 | Owner | Before B01 starts | to open |
| Ubuntu live-USB session on HW-01: WebKitGTK IPC and WebGL2 on the Intel and discrete GPUs, Orca walkthrough (WALKTHROUGH.md §3). A Linux failure that cannot be fixed within one week triggers the Electron fallback (ADR-0002) | Owner | Before B01 starts | to open |
| macOS session (lab Mac or volunteer): WKWebView IPC and WebGL2, App Nap check, VoiceOver walkthrough (WALKTHROUGH.md §4) | Owner (arranges access) | By B03 (R-22) | to open |
| A latency plan for PB-01 in A05: a WASAPI RAW or exclusive-mode backend option, a per-device latency calibration, and an acoustic or cable confirmation of the A02 numbers | Audio lead | A05 planning | to open |
| Licence decision for `assert_no_alloc` (BSD-1-Clause) before A05 uses it (F-038) | Owner | A05 start | to open |

## Go/no-go criteria

**Stage-specific criteria** (verbatim from the [roadmap](../../planning/roadmap.md#a02--technical-spikes-and-prototype-gate)):

| # | Criterion | Met | Evidence |
|---|---|---|---|
| 1 | "A gate report in `docs/sdlc/reviews/A02-prototype-gate.md` with measurements." | Yes (draft) | This report; F-030 to F-045 |
| 2 | "ADR-0002 (shell), ADR-0004 (render graph) and ADR-0003 (frontend) are confirmed or amended." | Partly | Recommendations in [ADR decisions](#adr-decisions): ADR-0004 amend; ADR-0002 and ADR-0003 pending the manual steps |
| 3 | "If Linux fails (4) or (5), the Electron fallback is triggered per ADR-0002." | Not yet evaluated | Linux spikes (4) and (5) are pending; WebKitGTK IPC on CI-LNX meets PB-09 (F-040) |

**Deliverables** (roadmap numbering) and **task acceptance** (implementation plan): the pass / pending / fail matrix.

| Roadmap deliverable / task | Acceptance criterion | Windows | macOS | Linux | Verdict |
|---|---|---|---|---|---|
| (1) / A02-T01 cpal output latency | p50/p95 at 128/256/512 frames on each OS | **Fail** against PB-01: p95 123.1 ms at 256 (all sizes within 1 ms; period fixed at 480) | Pending (needs a loopback device, R-22) | Pending (harness not ported) | **Fail** (Windows); pending elsewhere |
| (2) / A02-T02 owned graph | Benchmark Score in real time and offline; CPU per block | Pass: 361× offline, p99 24 µs per 256-frame block, 0 deadline misses in 120 s | Builds and renders on CI-MAC (T04) | Builds and renders on CI-LNX (T04) | **Pass** |
| (2) / A02-T03 Firewheel | Same benchmark; comparison table (CPU, xruns, code size, Miri, ergonomics) | Pass: bit-identical output, equal CPU; table in [#103](https://github.com/DuhItzAniket/SONARITHM/pull/103) | — | — | **Pass** |
| (2) Miri on the graph | UB audit where feasible | Pass with Miri's allocator (all tests). A report with a custom global allocator is reproduced without Firewheel (F-037) | — | Linux-target run with `assert_no_alloc`: TODO(verify), in progress at the time of writing | **Pass** (with F-037 caveat) |
| (6) / A02-T04 determinism | Null residual and ΔLUFS against PB-06 | Pass: bit-identical (owned and Firewheel); `f32::sin` variant −144.5 dBFS | Pass: bit-identical; `f32::sin` variant −150.5 dBFS | Reference | **Pass** |
| (3) / A02-T05 Tauri IPC | PB-09 on all 3 OSes | Pass: 1 MB p95 11.5–11.8 ms on HW-01, 15.7–20.7 ms on HW-01e, 7.8 ms on CI-WIN | Pass on CI-MAC (VM): p95 8 ms; real Mac pending | Pass on CI-LNX (VM, Xvfb): p95 7 ms; real hardware pending | **Pass** on CI and HW-01; real macOS and Linux hardware pending |
| (4) / A02-T06 WebGL2 | fps and frame-time distributions (PB-08) in WebView2, WKWebView, WebKitGTK (Intel/AMD/NVIDIA) | Pass on Intel UHD and RTX 4050: 60 fps, ≤ 0.6 ms script (recommended path) | Pending | Pending (owner live USB: Intel, discrete GPU; no AMD GPU available) | **Pass** (WebView2); pending elsewhere |
| (5) / A02-T07 screen readers | Pass/fail matrix per AT; OD-05 recommendation | Prototype built, UIA smoke test done (F-045); NVDA and Narrator pending | VoiceOver pending | Orca pending | **Pending** |
| A02-T08 gate report | Report merged; ADR confirmations recorded; risks updated | This report (draft); proposed risk changes below | — | — | **In review** |

**Gate Definition of Done** (the A02 subset):

- [ ] Every A02 task meets its acceptance criterion: no. T01 fails PB-01; T07 and the macOS/Linux parts of T01, T05 and T06 are pending.
- [x] Risk register reviewed for R-01 to R-05 and R-22 (proposals below; the register changes when the owner signs off).
- [ ] Release cut: not applicable (A02 releases nothing).
- [ ] Budgets measured on reference hardware: no reference machine is available (R-22). HW-01 results above 50% of a budget were re-measured on HW-01e.
- [x] Next-phase effects recorded (A05, B01–B04 below).

## Spike 1: output latency (A02-T01)

**Method.** A control-thread trigger stands in for the key press. The next cpal output callback starts a 1 kHz cue at frame 0 of its buffer, with 5 ms ramps per SDR-22. A WASAPI loopback capture of the same endpoint finds the onset. All timestamps are QPC readings, so trigger, callback and capture share one clock (clock check 0.00 ms). There were 200 trials per mode, with seeded random intervals of 400–700 ms. **Limits:** the loopback tap is the audio engine's output, so the acoustic latency is at least the measured value; the webview input path is excluded.

**Results** (HW-01, Realtek "Speakers", 48 kHz, enhancements as installed):

| Mode | Callback period | p50 | p95 | p99 | max | cpal-reported output latency |
|---|---|---|---|---|---|---|
| cpal `Fixed(128)` | 480 | 116.8 ms | 123.7 ms | 125.9 ms | 126.1 ms | 31.8 ms |
| cpal `Fixed(256)` | 480 | 116.4 ms | **123.1 ms** | 123.8 ms | 124.6 ms | 31.9 ms |
| cpal `Fixed(512)` | 480 | 117.2 ms | 124.0 ms | 125.2 ms | 126.3 ms | 31.8 ms |
| cpal `Default` | 480 | 116.9 ms | 124.0 ms | 125.5 ms | 126.3 ms | 31.7 ms |
| Direct WASAPI, RAW stream option | 480 | 79.2 ms | 83.7 ms | 84.2 ms | 84.4 ms | 12.0 ms (different formula) |

**Verdict: PB-01 fails** on Windows shared mode via cpal 0.18, by about 2.5×. The findings:

- **F-030:** the driver offers one engine period, so buffer requests have no effect.
- **F-031:** latency.
- **F-032:** the OEM APO adds about 38 ms and 20 dB of gain after our limiter.
- **F-033:** cpal's playback timestamp is about 80 ms early.

**Pending:**

- an acoustic or cable confirmation (the built-in microphone's echo cancellation removed the cue);
- a rerun with enhancements off;
- macOS and Linux.

### Options for PB-01

The measured floor on HW-01 is about 117 ms p50 and 123 ms p95 through cpal, and 84 ms p95 with the RAW stream. These are the ways forward, for the A05 plan:

| Option | What it changes | Evidence so far | Cost and risk |
|---|---|---|---|
| **RAW stream mode** (`AUDCLNT_STREAMOPTIONS_RAW`) | Bypasses OEM enhancement APOs (about 38 ms on HW-01) and their extra gain | Measured: p50 79.2 ms, p95 83.7 ms. That alone still misses 50 ms | Needs a second WASAPI backend (`wasapi` crate) or upstream cpal work (cpal #1173). The driver must support RAW processing |
| **WASAPI exclusive mode** (`wasapi` crate) | Bypasses the shared audio engine: the period becomes the driver's own minimum | Not measured. Loopback capture cannot observe an exclusive stream, so it needs a cable or acoustic setup (P6) | Takes the device from every other app, including the screen reader's speech output on the same device. That is a serious accessibility cost, so only an opt-in mode |
| **ASIO** (cpal `asio` feature, opt-in build) | A low-latency driver model, where the device has an ASIO driver | Not measured. HW-01's Realtek driver offers no ASIO driver; generic wrappers exist. TODO(verify): the licence terms of the Steinberg ASIO SDK, which cpal's build needs, under `deny.toml` and ADR-0010 | Opt-in build only; the same device-exclusivity concern as above |
| **USB audio interface** | Hardware with small shared-mode periods or a native low-latency driver; it also gives the cable loopback that PB-01 sign-off needs | Not measured | Hardware cost; not something to assume for users. It is the reference setup for measurements |
| **`IAudioClient3` small shared periods** | Shared mode at 128–256 frames where the driver allows it, without exclusivity | Not available on HW-01 (minimum = maximum = 480) | Driver-dependent; cpal does not use it yet (cpal #1343) |
| **Revised budget** | For example, PB-01 ≤ 50 ms p95 for a low-latency mode (exclusive, ASIO or a USB interface), and a documented, higher shared-mode figure | Shared mode measured 84–124 ms p95 here; other machines are unknown | Needs change control and evidence: what latency still supports interactive exploration (A11Y and SDR rules). Not a decision for this report |
| **Latency compensation** (calibrate per device) | Aligns the playhead and visual cues with the sound (F-033); it does not shorten key-to-sound time | — | Cheap; worth doing regardless |

**Recommendation.** Keep PB-01 as it is for now, and plan three things in A05:

1. a `wasapi`-crate backend behind `AudioBackend`, with RAW mode on by default and exclusive mode as an opt-in "low-latency mode";
2. per-device latency calibration;
3. a cable measurement with a USB interface (P6), before any decision to revise the budget.

The exclusive-mode accessibility cost (it can silence the screen reader's speech on the same device) needs its own test with NVDA.

## Spike 2: render graph, owned versus Firewheel (A02-T02, T03)

**Method.** Both graphs sit behind one draft contract: a control half and a real-time `GraphProcessor` (F-036). They render a seeded benchmark Score with the DS-006 shape (4 tracks, 10,000 notes, 60 s) using the same voice, bus and limiter code. The two are measured:

- offline, 3 runs per block size;
- in real time through cpal for 120 s, looping the Score;
- in debug builds under `assert_no_alloc`;
- under Miri.

| Criterion | Owned graph | Firewheel 0.14.0 |
|---|---|---|
| Output | Reference | Bit-identical (null residual −∞) |
| Offline real-time factor, 128 / 256 / 512 frames | 349× / 361× / 352× | 331× / 337× / 332× |
| Real time p99 per callback (HW-01 / HW-01e) | 85.7 / 151.5 µs | 85.4 / 124.0 µs |
| Deadline misses (both runs) | 0 | 0 |
| Allocation in `process` | none | none with `BufferOutOfSpaceMode::Panic`; the default allocates on the audio thread |
| Our code for the benchmark | 190 lines | 240 lines |
| Third-party code | `libm` | 43 crates, 28,100 lines, 81 `unsafe` occurrences |
| Licences, advisories | pass | pass |
| Miri (Windows target, Miri's allocator) | pass | pass |

**Verdict: pass for both.** Firewheel brings graph compilation, mixing and a correct sample-accurate scheduler (F-035), but no voices. For this workload it measurably beats the owned graph on nothing, and it adds pre-1.0 churn, a large dependency tree and an RT-unsafe default. The ADR-0004 condition for adopting it ("only if it beats the owned graph without safety concerns") is not met.

**Miri** (F-037): a Stacked Borrows and a Tree Borrows report appear only when a custom global allocator wraps `System` on the Windows target. A 15-line program without Firewheel or `assert_no_alloc` reproduces them, and they vanish on the Linux target. With Miri's own allocator, every spike test passes on the Windows target (62 minutes for 0.1 s of benchmark audio per test). The Linux-target run with `assert_no_alloc` was still running when this draft was written: TODO(verify).

## Spike 3: Tauri IPC (A02-T05)

**Method.** A Tauri 2.12.4 app with benchmark commands. Each runs unattended (`--autorun ipc`) on HW-01 (3 runs on all cores, 3 on HW-01e) and once on each CI runner.

| Test | HW-01 (WebView2 155) | HW-01e | CI-WIN (WebView2 154) | CI-MAC (WKWebView) | CI-LNX (WebKitGTK 2.52.6, Xvfb) |
|---|---|---|---|---|---|
| **1 MB binary `ipc::Response`, p95** | **11.5–11.8 ms** | **15.7–20.7 ms** | **7.8 ms** | **8 ms** | **7 ms** |
| 1 MB binary, p50 | 10.0–10.4 ms | 13.9–15.5 ms | 7 ms | 3 ms | 5 ms |
| 10 MB binary, p50 | 80.5–81.6 ms | 103–129 ms | 55.8 ms | 12 ms | 43 ms |
| 1 MB as JSON, p50 | 34.3–35.0 ms | 51.3–53.9 ms | 28.7 ms | 22 ms | 39 ms |
| Channel 1 KB at 30 Hz, one-way p50 / p95 | 1.8–1.9 / 2.1–2.4 ms | 3.0–3.7 / 3.9–5.4 ms | 0.97 / 1.4 ms | 2.9 / 4.8 ms (8 msg/s delivered) | 0.6 / 1.1 ms |
| Channel 64 KB burst | 147–156 MB/s | 49–56 MB/s | 103 MB/s | 192 MB/s | 171 MB/s |

**Verdict: PB-09 (≤ 30 ms) is met** on HW-01, including the CPU-limited rerun (F-040), and on all three CI runners, which are virtual machines. Also found:

- **F-041:** JSON costs 3.4× as much as binary.
- **F-042:** a CSP without the IPC origins silently downgrades binary IPC to JSON.
- **F-041:** the macOS runner delivered the 30 Hz stream at 8 messages per second, an App Nap hypothesis to check on a real Mac.

**Pending:** real macOS and Linux hardware (WALKTHROUGH.md M1, L1).

## Spike 4: WebGL2 frame times (A02-T06)

**Method.** A 2016 × 630 WebGL2 canvas, 600 frames per case. Each frame records the frame interval, the script time and the GPU time (timer queries). The GPU is selected with `--force_high_performance_gpu` / `--force_low_power_gpu`; the renderer string confirms it (F-044).

| Case | Intel UHD: fps / script max / GPU p50 | RTX 4050: fps / script max / GPU p50 |
|---|---|---|
| Playhead over a static 1M-point waveform | 59.7 / 0.5 ms / 0 ms | 60.0 / 0.2 ms / 0 ms |
| 1M points reduced by M4 per pixel column | 60.0 / 0.5 ms / 1.68 ms | 60.0 / 0.4 ms / 0.23 ms |
| Spectrogram 2048 × 512, one column per frame | 60.0 / 0.4 ms / 1.70 ms | 60.0 / 0.6 ms / 0.19 ms |
| Full 1M-point redraw every frame | **37** / 0.4 ms / 26.1 ms | 59.6 / 0.4 ms / 0.59 ms |

**Verdict: PB-08 is met in WebView2 on both GPUs** for the planned design. Data is drawn once or reduced in Rust, and the playhead never redraws the data; script time stays below 8% of the budget, so the headroom rule needs no CPU-limited rerun. A raw 1M-point redraw every frame is GPU-bound on the integrated GPU, so continuous zoom must use the M4 path (F-043).

**Pending:** WKWebView and WebKitGTK. The live USB covers the Intel GPU and the discrete GPU via nouveau; the proprietary NVIDIA driver needs a dual boot; no AMD GPU is available.

## Spike 5: screen readers (A02-T07)

**Prototype.** React 19.3 with React Aria Components 1.22: Tree, Toolbar with a toggle button, Slider, RadioGroup, and a chart explorer with `role="application"` scoped to the chart. There are three announcement routes for OD-05:

- a managed live-region pair with clear-then-set and supersede;
- a native UI Automation notification raised by the host (Windows);
- both at once.

**Smoke test** (automated, F-045):

- UI Automation exposes the expected roles.
- `UiaRaiseNotificationEvent` succeeds.
- Three points need a person to judge:
  - the Tree is a *tree grid* whose row names repeat the expand button's label;
  - `SliderOutput` is a live region;
  - React Aria adds live regions of its own.

**Verdict: pending.** No screen reader was run. The step-by-step scripts with expected results are in `spikes/tauri-webview/WALKTHROUGH.md`: NVDA and Narrator, Orca, VoiceOver.

## Spike 6: determinism (A02-T04)

**Method.** The benchmark Score is rendered offline at 256-frame blocks on CI-LNX, CI-WIN, CI-MAC (AArch64) and HW-01, in three variants: owned graph with polynomial sines, owned graph with `f32::sin`, and Firewheel with polynomial sines. The renders are compared with the Linux set.

| Variant | HW-01 | CI-WIN | CI-MAC |
|---|---|---|---|
| Owned, polynomial | bit-identical | bit-identical | bit-identical |
| Owned, `f32::sin` | −144.5 dBFS, ΔLUFS 0.0000 | −144.5 dBFS, ΔLUFS 0.0000 | −150.5 dBFS, ΔLUFS 0.0000 |
| Firewheel, polynomial | bit-identical | bit-identical | bit-identical |
| Score bytes | identical | identical | identical |

**Verdict: PB-06 is met everywhere** (F-039). Bit identity holds when the render path avoids the platform maths library.

## Performance budgets

| Budget | Target | Measured | Verdict | Re-baseline proposal |
|---|---|---|---|---|
| PB-01 interactive response | ≤ 50 ms p95 (256 frames, 48 kHz) | 123.1 ms p95, loopback, cpal shared mode, HW-01; 83.7 ms with RAW | **Fail** | None yet: see [Options for PB-01](#options-for-pb-01). Keep the budget; A05 plans a backend that can meet it, and measures with a cable |
| PB-02 audio robustness | 0 xruns in 10 min at 128 frames | Not in A02 (A05). 0 deadline misses in 120 s at the 480-frame period | — | Note: on HW-01's driver a 128-frame soak runs at 480 frames (F-030) |
| PB-03 offline render speed | ≥ 50× | 361× owned, 337× Firewheel (256 frames, 1 thread) | Preview pass (A05 owns it) | — |
| PB-06 determinism | Identical Score; residual ≤ −90 dBFS; ΔLUFS ≤ 0.1 | Identical Score; bit-identical audio; −144.5 dBFS worst with `f32::sin` | **Pass** | — |
| PB-08 UI frame time | 60 fps playhead; ≤ 8 ms script with 1M points | 60 fps; ≤ 0.6 ms script (WebView2, both GPUs) | **Pass** (WebView2); pending elsewhere | — |
| PB-09 IPC bulk transfer | ≤ 30 ms for 1 MB on all OSes | p95 11.5–11.8 ms (HW-01), 15.7–20.7 ms (HW-01e), 7–8 ms (CI ×3) | **Pass** (CI and HW-01); real macOS and Linux hardware pending | — |

## ADR decisions

| ADR | Recommendation | Reason |
|---|---|---|
| [ADR-0004](../../decisions/0004-audio-backend-and-render-graph.md) own render graph over a cpal-backed `AudioBackend` | **Amend**: confirm the owned graph and record the A02 evidence. Amend the backend part: cpal 0.18 in WASAPI shared mode cannot meet PB-01 on the development laptop, so A05 adds a WASAPI backend option (RAW stream, exclusive mode via `wasapi`) and a per-device latency calibration behind the same `AudioBackend` trait. Also record that the `RenderGraph` contract has a control half and an RT processor | F-030 to F-036, F-039 |
| [ADR-0002](../../decisions/0002-desktop-shell-tauri.md) Tauri 2 shell | **Keep provisional; no amendment yet.** The Windows evidence supports it (PB-08, PB-09), and IPC passes on all CI runners. The deciding Linux evidence (WebKitGTK frame rates on real GPUs, Orca) is pending; the Electron trigger is not evaluated | F-040 to F-044 |
| [ADR-0003](../../decisions/0003-frontend-react-and-react-aria.md) React + React Aria Components | **Keep provisional.** The prototype is built and the UI Automation tree is as expected; the screen-reader walkthroughs, which are the confirmation the ADR names, are pending | F-045 |

Recorded (the coordinator reviews these with the approval the owner delegated):

- ADR-0004's status line is now `accepted`.
- The backend amendment is the new [ADR-0016](../../decisions/0016-windows-low-latency-backend.md) (proposed); the ADRs are immutable once accepted.
- 06 §3.1, §6 and §9 describe the two-half contract, the proposed `WasapiBackend` and the latency calibration.
- ADR-0002 and ADR-0003 stay `accepted (provisional)`; the ADR index records their evidence status.

## Risk register review

Proposed changes; the register changes when the owner signs off.

| Risk ID | Previous rating | Proposed rating | Action |
|---|---|---|---|
| R-01 WebKitGTK performance, Orca | 3 × 4 = 12 | 12 (unchanged) | Pending the live-USB session. IPC on WebKitGTK meets PB-09 on CI-LNX |
| R-02 screen readers in the webview | 3 × 5 = 15 | 15 (unchanged) | Pending the walkthroughs. The UIA smoke test raised three points to check (F-045) |
| R-03 latency over 50 ms on Windows shared mode | 3 × 3 = 9 | **5 × 3 = 15** | The trigger is met (p95 123 ms, F-031). Mitigations: RAW or exclusive backend, latency calibration, pre-rendered cues; an acoustic confirmation first |
| R-04 render graph concurrency bugs or UB | 2 × 4 = 8 | 2 × 4 = 8 (unchanged) | Miri passes with Miri's allocator; the remaining report is a std/Miri issue on Windows (F-037). Run Miri on the Linux target in CI from A05 |
| R-05 cpal API churn | 4 × 2 = 8 | 8 (unchanged) | Still relevant: cpal lacks RAW and exclusive modes (cpal #106, #1173), so a second backend is likely |
| R-22 no macOS or Linux hardware | 3 × 3 = 9 | 9 (unchanged) | It held back every macOS and Linux manual item in this gate |

New risks raised at this gate:

| Risk | Rating | Owner | Mitigation |
|---|---|---|---|
| Platform audio processing (OEM enhancement APOs) changes the sonification after SONARITHM's limiter: on HW-01, +20.6 dB of gain, a 200 ms tail and 38 ms of latency (F-032). This undermines loudness calibration (SDR-24), hearing safety (06 §10) and latency | 3 × 4 = 12 | Audio lead | RAW stream mode where the driver supports it; a loopback self-test that detects added gain or delay; user guidance to disable enhancements |
| cpal's reported playback time is far from the observed onset (F-033), so the playhead and cue sync drift by tens of milliseconds | 3 × 2 = 6 | Audio lead | Per-device latency calibration; verify acoustically before B04 |

## Pending owner and other-OS steps

Everything below is scripted in `spikes/tauri-webview/WALKTHROUGH.md`, which has the commands, expected results and a results table.

| # | Step | Who | Where | Covers |
|---|---|---|---|---|
| P1 | NVDA walkthrough, steps 1–20 | Owner | HW-01 | ADR-0003, OD-05, R-02 |
| P2 | Narrator walkthrough, steps 1–19 | Owner | HW-01 | OD-05 (the native route), R-02 |
| P3 | Ubuntu live USB: IPC (L1), WebGL2 on Intel (L2), the discrete GPU via `DRI_PRIME` (L3), without DMABUF (L4), X11 (L5) | Owner | HW-01 | ADR-0002 Linux trigger, PB-08, PB-09, R-01 |
| P4 | Orca walkthrough | Owner | HW-01 live USB | ADR-0002 Linux trigger, R-01, R-02 |
| P5 | macOS: IPC (M1), App Nap check (M2), WebGL2 (M3), VoiceOver (M4) | Volunteer or lab Mac | Mac | ADR-0002, ADR-0003, R-22 |
| P6 | Acoustic or cable latency confirmation: `spikes/cpal-latency` with `--capture` on a USB audio interface | Owner (needs an interface) | HW-01 | PB-01, F-031 to F-033 |
| P7 | Latency rerun with audio enhancements off (Settings → Sound) | Owner | HW-01 | F-032 |
| P8 | WebKitGTK with the proprietary NVIDIA driver (installed Ubuntu, dual boot) | Owner | HW-01 | R-01 (NVIDIA case) |
| P9 | macOS and Linux output latency (loopback device and a harness port) | Volunteer | Mac, Linux | (1) on all OSes |

## Research and publications

| Item | Status | Link |
|---|---|---|
| Findings logged in A02 | 16 (F-030 to F-045; IDs F-030 to F-049 reserved for A02) | [findings log](../../research-paper/findings-log.md#implementation-findings) |
| Benchmarks rows | A02 audio and desktop rows; hardware IDs HW-01, HW-01e, CI-LNX, CI-WIN, CI-MAC | [benchmarks log](../../research-paper/benchmarks-log.md) |
| Engine paper (P07) | Latency, render-graph and determinism numbers seed it (roadmap research hook) | F-031, F-034, F-035, F-039 |

## Next phases

| Phase | Effect of this gate | Reason |
|---|---|---|
| A05 | Add: a WASAPI backend option (RAW or exclusive) behind `AudioBackend`, per-device latency calibration, an acoustic latency check; adopt the two-half `RenderGraph` contract; settle the `assert_no_alloc` licence; run Miri on the Linux target | F-030 to F-038 |
| A05 | Keep oscillators table-based or polynomial (or `libm`) where byte-identical audio is wanted; re-check determinism with fundsp and SIMD | F-039 |
| B01 | Test that the CSP keeps the binary IPC path; keep JSON off hot paths; disable App Nap for the control and telemetry threads if P5 confirms it | F-041, F-042 |
| B02, B03 | Check the tree-grid naming, the slider output live region and React Aria's own announcer against the walkthrough results | F-045 |
| B04 | Use M4 tiles for zoom over large series; offer a discrete-GPU option | F-043, F-044 |

**Spike disposition.** The roadmap says the spikes are "deleted or archived after the gate". Proposal:

1. Once the pending results are in, tag the last commit that has `spikes/` (for example `a02-spikes`).
2. Then delete `spikes/` and the two temporary workflows in one cleanup PR.

The READMEs and this report keep the method and the numbers.

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | @DuhItzAniket | | Go / Conditional go / No-go |
| Architecture lead | @DuhItzAniket | | |
| Research lead | @DuhItzAniket | | |
| Accessibility owner | @DuhItzAniket | | |
