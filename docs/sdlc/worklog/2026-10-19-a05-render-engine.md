# 2026-10-19: A05 render engine, audio backends and export

> **Date(s):** 2026-10-19. **Phase:** A05. **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [status](../status.md#a05-tasks) · [ADR-0004](../../decisions/0004-audio-backend-and-render-graph.md) · [ADR-0016](../../decisions/0016-windows-low-latency-backend.md)

## Context and phase

- Phase: A05, [roadmap entry](../../planning/roadmap.md).
- Issues: #55 to #65, #67 and #123 (milestone 5). The tasks form one dependency chain, merged in order:
  - [#118](https://github.com/DuhItzAniket/SONARITHM/pull/118), [#119](https://github.com/DuhItzAniket/SONARITHM/pull/119) and [#120](https://github.com/DuhItzAniket/SONARITHM/pull/120): T01 oscillators, envelopes, filters;
  - [#122](https://github.com/DuhItzAniket/SONARITHM/pull/122) T03 plan builder; [#126](https://github.com/DuhItzAniket/SONARITHM/pull/126) T04 render graph;
  - [#160](https://github.com/DuhItzAniket/SONARITHM/pull/160), bundle 1: T05, T07, T09, T02, T08;
  - [#168](https://github.com/DuhItzAniket/SONARITHM/pull/168), bundle 2: T06, T13;
  - bundle 3 (this entry's PR): T12, T11, T10.

  The owner approved bundling the rest of the chain into three PRs on 2026-10-19 (one commit per task inside each).
- Starting point: A03 (spec, Score IR) and A04 (data) were in progress in other sessions. `sonarithm-dsp`, `-render`, `-audio` and `-export` were empty crates. The owner has one Windows laptop (HW-01), so macOS and Linux run only on CI runners.

## Goals

- Build the render path of ADR-0004: DSP nodes, the render plan, the graph with its voice pools, scheduler and transport, and the real-time channels.
- Add the backends: cpal, the WASAPI backend of ADR-0016, offline rendering, and WAV and FLAC export with a render manifest.
- Add hearing safety on the master, the golden-audio harness, the benchmarks and the allocation check.
- Measure the A05 exit criteria on HW-01 and log every result.

## What was done

- **T01:** oscillators with an IEEE-arithmetic sine polynomial and no `f32::sin`, which a clippy `disallowed-methods` rule enforces; ADSR with raised-cosine ramps and the SDR-22 conformance tests; filters, a constant-power panner and a smoother.
- **T02:** ISO 226:2023 compensation (Table 1 checked against the standard's official preview) and an EBU R128 meter that passes EBU Tech 3341 cases 1–5.
- **T03 to T05:** `RenderPlan`; `Graph` with four buses, sample-accurate dispatch (bit-identical at any block size) and crossfaded seek and loop; `realtime()` with an rtrb command ring, basedrop plan swaps and a triple-buffer engine state, tested with shuttle.
- **T06, T13:** the `AudioBackend` trait, `CpalBackend`, a timer-driven `NullBackend`, a `Player` that recovers from device loss, and `WasapiBackend` (RAW shared mode by default, exclusive mode opt-in) with a latency tool and a calibration table. Manual scripts: [device hot-plug](../../engineering/manual-tests/a05-device-hotplug.md) and [cable latency](../../engineering/manual-tests/a05-cable-latency.md).
- **T07 to T09:** offline rendering, 24-bit and float WAV, 24-bit FLAC, the render manifest with a generated schema, the WebVTT cue sidecar, and a 4× oversampled true-peak limiter with a start-up ramp.
- **T11, T12:** `cargo xtask golden` with six cases and the `golden` and `golden-compare` CI jobs (OD-08: hashes and metrics, no audio in git); criterion and `render_speed` benchmarks for PB-03, with three optimizations.
- **T10:** an allocation check for the audio thread (`allocation-counter`), xrun forwarding from the device to the engine state, the `soak` program, the `no-alloc` CI job, and a 10-minute `alloc-soak` job in `nightly.yml`.

## Decisions

| Decision | Why | Record |
|---|---|---|
| The allocation check uses `allocation-counter` 0.8.1 (MIT OR Apache-2.0) instead of `assert_no_alloc` | `assert_no_alloc`'s BSD-1-Clause licence is not in `deny.toml`, and licence changes belong to the owner | F-038, F-054; approved by the coordinator under the owner's delegation, with three conditions: no shipped dependency (enforced by `check-deps`), control tests, and AGENTS.md rule 2 unchanged except for the tool name |
| Envelope ramps have a 5 ms floor | 06 §3 wins over the 2 ms proposed in sound-design §5 (an architecture doc wins over a design proposal) | 06 note; SDR-22 tests |
| No FTZ/DAZ mode; the DSP flushes its own state below 1e-20 | `no_denormals` does not build for wasm32 and needs `unsafe` | coding-standards §11.4 note |
| Plan swaps pass `basedrop::Owned<Graph>`, not `Shared<RenderPlan>` | The new graph adopts the old transport and levels, and the old graph fades out before the collector frees it | 06 §4 note |
| OD-08: golden audio is stored as hashes and metrics, not audio files | Renders are bit-identical across OSes (F-039), so hashes catch every change without binary files in git | open-decisions OD-08 (closed) |
| ISO 226 compensation is on by default at 60 phon | SDR-24; the level is provisional until the B03 studies | SDR-24 tests; status T02 |
| The paced null device counts a block that is ready after its deadline as an underrun; `soak --strict` gates allocations and faults, not xruns | It gives a 128-frame deadline test where the driver has no 128-frame period, while CI runners' timer jitter cannot fail the build | F-055; ci-cd `no-alloc` row |
| A local `Score` strategy instead of core's `test-support` feature | Enabling it from dev-dependencies pulls proptest and tempfile into core's graph, which `check-deps` rejects | Reported to the coordinator (A03 follow-up) |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| **Hearing safety:** the latency tool ran for about 90 s with cues at −36 dBFS in non-RAW shared mode, which the OEM processing raises to about −16 dBFS (F-032) | The tool capped the level at −30 dBFS whatever the mode; the brief's −30 dBFS limit assumed RAW mode | `MAX_LEVEL_PROCESSED` (−50 dBFS) now applies whenever RAW mode is not in effect; the earlier RAW runs used −36 dBFS, and every device run after the fix was a silent soak (−120 dB) |
| The hot-plug example played at −39 dBFS through cpal, which cannot use RAW mode, so about −19 dBFS after the OEM processing | Written before the non-RAW cap existed | Lowered to −50 dBFS in the T06 PR before it was opened; the owner has not run it yet |
| Exclusive mode stalled after start | The Realtek driver needs the buffer filled before `Start` and signals two or three events per period | Prefill before start; queue depth from the device clock (F-051) |
| Exclusive-mode soak reports late callbacks with no xrun | The same event clustering makes some callback intervals longer than 1.5 periods | Late callbacks in exclusive mode are read together with xruns (F-055) |
| Scratch builds failed | Windows path length limits in the default target directory | `CARGO_TARGET_DIR` inside the worktree |
| Struct-update syntax on `PlanConfig` and `StreamConfig` failed outside their crates | Both are `#[non_exhaustive]` | Constructors and builders: `PlanConfig::at_rate`, `StreamConfig::with_buffer` and `on_device` |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Offline render speed (PB-03, ≥ 50×) | 91.4× (criterion), 92–96× (`render_speed`); 43–46× on four E-cores | DS-006 Score, 60 s, 256-frame blocks, 48 kHz, HW-01 and HW-01e | Benchmarks log; F-053 |
| Key-to-onset latency, p95 (loopback, software path) | 104.0 ms RAW; 145.6 ms default shared | WASAPI, 480-frame period, 200 cues, HW-01 | Benchmarks log; F-052 |
| True peak under stress | −1.186 and −1.162 dBTP | Limiter stress Scores, ebur128 | F-050 |
| Device soak (PB-02) | 0 underruns: WASAPI RAW (480-frame period), exclusive (144 frames); cpal 0 late callbacks. Null device at 128 frames: 194 and 153 missed timer deadlines | 10 min each, HW-01 and HW-01e, silent | Benchmarks log; F-055 |
| Render time per 128-frame block | p99.9 106–120 µs, max 382 µs (14% of the block) | HW-01, playback graph, DS-006 Score | F-055 |
| Allocations on the audio thread | 0 | Debug builds: scripted session; null device; null soak | F-054 |
| Cross-OS null residual (PB-06) | Pending the T11 PR's CI run | `golden-compare` | — |

## Findings to log

- F-050 to F-055: logged in the [findings log](../../research-paper/findings-log.md).

## A05 gate summary

Budgets from 12 §8, measured on HW-01 (and HW-01e, four E-cores, for the headroom rule). The owner decides each before `v0.1.0-alpha` is tagged.

| Budget | Target | Measured | Recommendation |
|---|---|---|---|
| PB-01 key press → cue onset | ≤ 50 ms p95 at 256 frames, 48 kHz | Loopback self-test through `Command::Cue`, 200 cues, software path only: RAW shared p50 99.2 ms, p95 104.0 ms; default shared p95 145.6 ms (F-052). A02's direct RAW renderer: p95 83.7 ms (F-032). The driver's shared period is fixed at 480 frames (F-030), and exclusive mode queues about 100 ms (F-051). Not yet measured acoustically (cable test, #123) | **Defer to B03**, where 12 §8 places PB-01's final check. The engine's share is small: a 128-frame block renders in at most 0.4 ms, and a command waits at most one 10 ms period. The rest is this driver's Windows audio path. Decide after the cable test and one run on a device with small shared periods (a USB interface, or a driver that offers `IAudioClient3` periods). If those still exceed 50 ms, re-baseline PB-01 per device class in 12 §8 rather than waive it |
| PB-02 audio robustness | 0 xruns in 10 min at 128 frames | 0 underruns in 10-minute soaks at every period HW-01 offers: WASAPI RAW 480 frames, exclusive 144 frames; cpal 0 late callbacks. No 128-frame period exists on this driver (F-030, F-055) | **Accept for v0.1.0-alpha at the device's periods**; rerun at 128 frames on reference hardware or a USB interface, TODO(verify) |
| PB-03 offline render speed | ≥ 50× real time (4 tracks, 10k events) | 91.4× (criterion, 656 ms per 60 s render) and 92–96× (`render_speed`) on HW-01; 43–46× on HW-01e, the CPU-limited rerun that 12 §8 requires above half the budget (F-053) | **Waive for v0.1.0-alpha, with a follow-up before Gate B.** Reason: only the stand-in for the reference machine misses, by about 10%. No reference-class machine exists (R-22). At 43×, a 60 s export takes about 1.4 s, which does not block alpha users. Follow-up: render voices in SIMD batches (voices dominate at about 10 ns per voice-sample, F-053), then rerun on reference hardware |
| PB-06 determinism | Null residual ≤ −90 dBFS, ΔLUFS ≤ 0.1 across OSes | Six golden cases identical at every block size on HW-01; the cross-OS comparison is the `golden-compare` job of the bundle 3 PR. A02 showed bit-identical renders across three OSes (F-039) | **Accept if `golden-compare` passes** on the bundle 3 PR |

## Next steps

These are the owner's manual steps for the A05 gate.

- [ ] Run the [device hot-plug script](../../engineering/manual-tests/a05-device-hotplug.md) (@DuhItzAniket).
- [ ] Run the [cable latency test](../../engineering/manual-tests/a05-cable-latency.md) and the NVDA check; #123 stays open until then (@DuhItzAniket).
- [ ] PB-02 at 128 frames: run `soak` on a device with 128-frame periods, for example a USB audio interface (@DuhItzAniket).
- [ ] Decide PB-01, PB-02 and PB-03 using the gate summary above (@DuhItzAniket).
- [ ] Acknowledge the OD-08 decision (@DuhItzAniket). Optional: allowing BSD-1-Clause in `deny.toml` later (F-038; `allocation-counter` needs no change).
- [ ] Compare the ISO 226 implementation with the standard's Annex B, TODO(verify) (anyone with the full standard).
- [ ] Record the cross-OS golden result from the T11 PR's CI run in the benchmarks log (agent).
- [ ] Draft the A05 phase review once these steps are done (agent).
