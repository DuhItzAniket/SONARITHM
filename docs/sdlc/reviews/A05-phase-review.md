# A05 phase review: Render engine, audio backend and export

> **Status:** Draft (2026-10-21), final when the owner decides the four budgets below. **Phase:** A05. **Author:** A05 agent session (Claude Code) for the project lead. **Reviewers:** coordinator session (owner's delegate), @DuhItzAniket.
> **Related:** [reviews README](README.md) · [roadmap A05](../../planning/roadmap.md#a05--render-engine-audio-backend-and-export) · [worklog: A05 render engine](../worklog/2026-10-19-a05-render-engine.md) · [06 audio engine](../../architecture/06-audio-engine.md) · [ADR-0004](../../decisions/0004-audio-backend-and-render-graph.md) · [ADR-0016](../../decisions/0016-windows-low-latency-backend.md)

## Summary

| Item | Value |
|---|---|
| Phase | A05 · Render engine, audio backend and export ([roadmap](../../planning/roadmap.md#a05--render-engine-audio-backend-and-export)) |
| Planned window | 2026-11-29 – 2026-12-27 (4 weeks) |
| Actual window | 2026-10-19 – 2026-10-21 |
| Outcome | **Done.** The owner decided PB-01, PB-02, PB-03 and PB-06 on 2026-10-21 (gate summary below) |

A05 delivered:

- the DSP nodes;
- the render plan and the render graph with its voice pools, scheduler and transport;
- the real-time channels between the control and audio threads;
- the offline renderer with WAV and FLAC writers, the render manifest and the WebVTT sidecar;
- the true-peak limiter, the start-up ramp and ISO 226 compensation;
- the cpal and WASAPI backends with device-loss recovery and latency calibration;
- the golden-audio harness, the benchmarks and the allocation checks.

Renders are bit-identical across three OSes and every block size. The audio thread allocates nothing.

Two budgets are not met as written on the owner's only machine: PB-02 and PB-03. PB-01, which belongs to B03, is far from its target on this laptop's audio driver. The owner accepted the recommendations below on 2026-10-21. The cable-latency confirmation of ADR-0016 (#123) is an owner step.

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | 0 xruns in a 10-min soak at 128-frame buffers on reference hardware. | Partially met; **accepted by the owner** for `v0.1.0-alpha` (PB-02), with the 128-frame rerun in #187 | 0 underruns in 10-minute soaks at every period HW-01 offers: WASAPI RAW 480 frames, exclusive 144 frames; cpal 0 late callbacks. The driver has no 128-frame period (F-030), and no reference-class machine exists (R-22). The worst 128-frame block renders in 382 µs of 2,667 µs (F-055) |
| 2 | Offline render ≥ 50× real time on the benchmark Score. | Partially met; **waived by the owner** for `v0.1.0-alpha` (PB-03), with the follow-up in #185 | 91.4× (criterion) and 92–96× (`render_speed`) on HW-01; 43–46× on HW-01e, the CPU-limited rerun that 12 §8 requires above half the budget (F-053) |
| 3 | The null-test residual between OSes is ≤ −90 dBFS for the golden set. | Met | All six golden cases bit-identical on Windows and macOS against Linux: residual at the −240 dBFS floor, ΔLUFS 0.0000 (`golden-compare` on #178, F-056) |
| 4 | No allocations on the audio thread (debug assertion run in CI). | Met | `no-alloc` job green on ubuntu, windows and macos (#178), and the nightly `alloc-soak` job green on its PR run. `allocation-counter` replaces `assert_no_alloc` (F-038, F-054) |

Waivers granted by the owner on 2026-10-21:

| Criterion | Reason | Owner | Follow-up |
|---|---|---|---|
| 2 (PB-03) for `v0.1.0-alpha` | Only the stand-in for the reference machine misses, by about 10%. At 43× a 60 s export takes about 1.4 s | @DuhItzAniket | SIMD voice batching and a rerun on reference hardware before Gate B (#185) |

## Gate summary: budgets and recommendations

Budgets from [12 §8](../../architecture/12-cross-cutting-concerns.md#8-performance-budgets). Each one needs the owner's decision before `v0.1.0-alpha` is tagged.

| Budget | Target | Measured | Recommendation | Decision |
|---|---|---|---|---|
| PB-01 key press → cue onset | ≤ 50 ms p95 at 256 frames, 48 kHz | Loopback self-test through `Command::Cue`, 200 cues, software path only: RAW shared p50 99.2 ms, p95 104.0 ms; default shared p95 145.6 ms (F-052). A02's direct RAW renderer: p95 83.7 ms (F-032). The shared period is fixed at 480 frames (F-030); exclusive mode queues about 100 ms (F-051). Not yet measured acoustically (#123) | **Defer to B03**, where 12 §8 places PB-01's final check. The engine's share is small: a 128-frame block renders in at most 0.4 ms, and a command waits at most one 10 ms period. Decide after the cable test and one run on a device with small shared periods. If those still exceed 50 ms, re-baseline PB-01 per device class rather than waive it | **Deferred to B03** by the owner, 2026-10-21 (#186, #123) |
| PB-02 audio robustness | 0 xruns in 10 min at 128 frames | 0 underruns at the device's periods (480 and 144 frames); 769,982 callbacks with no allocation (F-055) | **Accept for `v0.1.0-alpha` at the device's periods**; rerun at 128 frames on a USB interface or reference hardware | **Accepted** by the owner, 2026-10-21; rerun #187 |
| PB-03 offline render speed | ≥ 50× real time (4 tracks, 10k events) | 91.4× on HW-01; 43–46× on HW-01e (F-053) | **Waive for `v0.1.0-alpha`**, with the follow-up before Gate B (above) | **Waived for `v0.1.0-alpha`** by the owner, 2026-10-21; follow-up #185 |
| PB-06 determinism | Null residual ≤ −90 dBFS; ΔLUFS ≤ 0.1 | Bit-identical on three OSes and two architectures (F-056) | **Accept**: `golden-compare` passed | **Accepted** by the owner, 2026-10-21 |

## Definition of Done (phase)

- [x] All issues in the phase are closed (#55 to #65 and #67 by their PRs), except #123. #123 waits only on the owner's cable test and NVDA check.
- [x] No open critical bug from the phase.
- [x] Docs match what was built:
  - implementation notes in 06 and 05 §5;
  - 02 §3 rules 3, 5 and 7;
  - coding-standards §11.4 and §11.5, testing-strategy §5, §8 and §9, ci-cd;
  - AGENTS.md rule 2 (tool name only).
- [x] Deviations recorded:
  - no FTZ/DAZ mode; the DSP flushes its own state;
  - plan swaps pass `Owned<Graph>`;
  - a 5 ms envelope floor (06 §3 wins over the sound-design proposal);
  - `allocation-counter` in place of `assert_no_alloc`.
- [x] [status.md](../status.md) updated.
- [x] Findings logged: F-050 to F-056. Measurements in the benchmarks log.
- [x] Traceability matrix:
  - FR-100, NFR-03 and NFR-10 set to Done;
  - FR-050, FR-071 and FR-101 continue in B01, C04 and B03;
  - NFR-01 records PB-01, PB-02 and PB-03.
- [x] Risk register: R-03 updated with the A05 latency results.
- [x] Coverage targets: not measured (no coverage tooling in CI yet). Every module has unit tests; the RT protocol has a shuttle test; renders have golden cases and snapshots.
- [x] Retrospective: folded into "Lessons" below (one agent session ran the phase).

## Metrics against budget

| Metric | Budget | Measured | Conditions | Evidence | Verdict |
|---|---|---|---|---|---|
| PB-01 key press → onset, p95 | ≤ 50 ms | 104.0 ms (RAW), 145.6 ms (default) | HW-01, WASAPI shared, 480-frame period, loopback, 200 cues | F-052, benchmarks log | Outside; deferred to B03 by the owner (#186) |
| PB-02 underruns, 10 min | 0 at 128 frames | 0 at 480 and 144 frames; 128 not available | HW-01, silent soaks | F-055, benchmarks log | Not measurable at 128 frames |
| PB-03 render speed | ≥ 50× | 91.4× (HW-01), 43–46× (HW-01e) | DS-006 Score, 256-frame blocks, 1 thread | F-053, benchmarks log | Outside on HW-01e; waived for `v0.1.0-alpha` by the owner (#185) |
| PB-06 null residual, ΔLUFS | ≤ −90 dBFS, ≤ 0.1 | Identical (−240 dBFS floor), 0.0000 | CI-LNX, CI-WIN, CI-MAC, six cases | F-056, benchmarks log | Within |
| NFR-10 true peak under stress | ≤ −1 dBTP | −1.186 and −1.162 dBTP | Two stress Scores, ebur128 meter | F-050 | Within |

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| Oscillators incl. PolyBLEP, envelopes with mandatory ramps (SDR-22), filters, constant-power panner (T01) | `sonarithm-dsp` | #118, #119, #120 |
| ISO 226 loudness compensation (SDR-24) and EBU R128 meter (T02) | `sonarithm-dsp::loudness`, `::meter` | #160 |
| `RenderPlan` and plan builder (T03) | `sonarithm-render::plan` | #122 |
| Render graph, voice pool, sample-accurate scheduler, transport (T04) | `sonarithm-render::graph` | #126 |
| `rtrb` command queue, `triple_buffer` telemetry, `basedrop` plan swap (T05) | `sonarithm-render::rt` | #160 |
| `AudioBackend` trait, cpal backend, device hot-plug (T06) | `sonarithm-audio` | #168 |
| Offline backend; WAV via hound, FLAC via flacenc (T07) | `sonarithm-render::offline`, `sonarithm-export` | #160 |
| Render manifest with spec hash and data fingerprint; WebVTT sidecar (T08) | `sonarithm-export` | #160 |
| Master limiter and start-up ramp (T09) | `sonarithm-dsp::limiter`, graph master | #160 |
| Allocation check in debug builds; 10-minute soak; xrun counting (T10) | `no_alloc` tests, `soak` example, `no-alloc` and nightly `alloc-soak` jobs | #178 |
| Golden-audio harness with tolerance comparison; cross-OS CI (T11) | `cargo xtask golden`, `tests/golden/` | #178 |
| Benchmarks (T12) | criterion bench, `render_speed`, `block_times` | #178 |
| WASAPI backend, latency report and calibration (T13, ADR-0016) | `sonarithm-audio::wasapi_backend`, `latency`, `calibration` | #168 |

**Deferred or dropped**

| Item | Reason | Where it goes |
|---|---|---|
| A 128-frame device soak | HW-01's driver has no 128-frame period | Owner step on other hardware (PB-02, #187) |
| Acoustic latency and NVDA coexistence | Needs a cable and the owner | #123 (owner checklist #129) |
| SIMD voice rendering | PB-03 on the reference stand-in | Before Gate B (#185) |
| `IAudioClient3` small shared periods | Not offered by HW-01's driver | When a device that offers them is available |
| Latency tooling on macOS and Linux | No hardware (R-22) | R-22 follow-up |
| ISO 226 Annex B comparison; EBU Tech 3342 cases | Need the full standard and the test signals | TODO(verify) |
| Listening checks: limiter pumping, the new mallet tilt | Need a listener | Owner or B03 listening sessions |
| `audio_thread_priority` 0.38 | cpal 0.18 pins 0.35; two versions fail to link on Windows | Dependabot ignore until cpal moves (#173) |

## Decisions

| ADR / decision | Title | Status |
|---|---|---|
| [ADR-0004](../../decisions/0004-audio-backend-and-render-graph.md) | Audio backend and render graph | Accepted; implemented with the two-half contract |
| [ADR-0016](../../decisions/0016-windows-low-latency-backend.md) | Windows low-latency backend | Accepted; implemented, with RAW mode confirmed on HW-01. Acoustic confirmation pending (#123) |
| [ADR-0013](../../decisions/0013-determinism-and-provenance.md) | Determinism and provenance | Accepted; met for the render path (F-056) |
| OD-08 | Golden audio stored as hashes and metrics, no audio in git | Decided in A05; owner acknowledgement pending |
| F-038 | `allocation-counter` in place of `assert_no_alloc` | Approved by the coordinator under the owner's delegation; `deny.toml` unchanged |

## Risks

| Risk ID | Change | Note |
|---|---|---|
| R-03 | Mitigation and trigger updated | Interactive latency stays above 50 ms on Windows shared mode, in RAW mode too; exclusive mode gains nothing on HW-01 (F-051, F-052) |
| R-22 | No change | Cross-OS determinism is covered on CI runners; device tests ran on Windows only |

## Findings and research hooks

| Item | Status | Link |
|---|---|---|
| F-050 · The master limiter keeps stress Scores at −1.16 dBTP or below | Logged | [findings log](../../research-paper/findings-log.md) |
| F-051 · WASAPI exclusive mode on HW-01's Realtek driver clusters events and queues about 100 ms | Logged | as above |
| F-052 · RAW shared mode: 104 ms p95 key-to-onset; prediction 83 ms early | Logged | as above |
| F-053 · PB-03: 91–93× on HW-01, 44× on four E-cores | Logged | as above |
| F-054 · `allocation-counter` replaces `assert_no_alloc` | Logged | as above |
| F-055 · Device soaks: no underrun at HW-01's periods; no 128-frame period | Logged | as above |
| F-056 · Golden cases bit-identical on three OSes and two architectures | Logged | as above |
| P07 (engine paper) | Benchmarks, determinism and the soak results are ready for it | [benchmarks log](../../research-paper/benchmarks-log.md) |

## Accessibility

No UI changed. The engine makes these choices for blind and low-vision users:

- **Streams never open loud.** Playback starts 20 dB down and rises over 500 ms. A true-peak limiter holds the master at −1 dBTP, and HW-01's OEM processing is bypassed in RAW mode (HS-02, NFR-10).
- **Device loss pauses, never jumps.** After a device is lost, playback rebuilds on the default device at the same position and stays paused until the user resumes (HS-05). It reports the change in words for the B-stage announcement.
- **Exclusive mode is opt-in.** It silences a screen reader on the same device; the NVDA coexistence check is part of #123.

## Lessons

- **Bundling a dependency chain saves merge waits.** The thirteen tasks form one chain, and one PR at a time took a CI cycle each. Bundling the last nine into three PRs, one commit per task, landed them in three cycles.
- **A clean textual merge of `Cargo.lock` can still be stale.** One entry lacked the version qualifier that a second `thiserror` required. CI's `--locked` would have failed. After a rebase, check every branch with `cargo metadata --locked --offline`.
- **Device budgets meet the driver first.** This laptop's driver fixes the shared period at 10 ms and adds about 80 ms in RAW mode, so PB-01 and PB-02 at 128 frames cannot be met or measured here. Budgets that name a buffer size may need device classes.
- **Hearing safety needs the processed path in mind.** One latency run played cues at −36 dBFS without RAW mode, which the OEM processing raised to about −16 dBFS for 90 s. Non-RAW output is now capped at −50 dBFS, and every later device run was silent.

## Follow-up actions

| Action | Owner | Due | Issue |
|---|---|---|---|
| Decide PB-01, PB-02, PB-03 and PB-06 (gate summary above) | @DuhItzAniket | Before `v0.1.0-alpha` | Done 2026-10-21 |
| Cable latency test and NVDA check (ADR-0016) | @DuhItzAniket | Before B03 | #123, checklist #129 |
| Device hot-plug script | @DuhItzAniket | — | checklist #129 |
| SIMD voice rendering and a PB-03 rerun on reference hardware | Render maintainer | Before Gate B | #185 |
| A 128-frame soak on a capable device | @DuhItzAniket | Before Gate B | — |
| Remove the Dependabot ignore for `audio_thread_priority` when cpal requires a newer version | Maintainer | With the next cpal upgrade | #173 |

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | @DuhItzAniket | | |
