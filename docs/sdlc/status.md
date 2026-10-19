# Project Status

> **Last updated:** 2026-10-18 · **Current stage:** A (A01 in progress) · **Next gate:** Gate A (alpha), target Feb 2027
> **Owner:** Project lead. **Update rule:** every PR that changes a task's or phase's state updates this file ([sdlc/README](README.md)).

## 1. Summary

| Item | State |
|---|---|
| Overall health | **Green**: A01 merged (#1, `02a44f3`); all CI checks green on Windows, macOS and Linux |
| Current focus | A02 (spikes) and A03 (core types) in parallel; tasks tracked on the [project board](https://github.com/users/DuhItzAniket/projects/2). Owner: funding for paid BLV usability sessions (OD-10) |
| Latest release | none |
| Blocking decisions | none for A01 (OD-01 and OD-09 closed 2026-10-18) |
| Top risks | R-08 scope creep (20), R-09 bus factor (20), R-02 screen readers in the webview (15), R-22 no macOS/Linux hardware (9) |

## 2. Phase board

Status values: `Not started` · `In progress` · `In review` · `Done` · `Blocked` · `Deferred`.

| Stage | Phase | Name | Status | Progress | Notes |
|---|---|---|---|---|---|
| — | — | Kickoff: research and documentation baseline | **Done** | 100% | 2026-10-18; see [worklog](worklog/2026-10-18-project-kickoff.md) |
| A | A01 | Repository bootstrap and governance | **Done** | 100% | 2026-10-19; [phase review](reviews/A01-phase-review.md), [retro](retrospectives/A01-retro.md). See [A01 tasks](#a01-tasks) |
| A | A02 | Technical spikes and prototype gate | **In review** | ≈80% | All spikes built and measured on Windows and CI; gate report draft: Conditional go recommended. PB-01 fails on Windows shared mode; screen-reader, macOS and Linux manual steps pending. See [A02 tasks](#a02-tasks) |
| A | A03 | Core types, Spec v0.1, Score IR | **Done** | 100% | 2026-10-19; [phase review](reviews/A03-phase-review.md), [retro](retrospectives/A03-retro.md). See [A03 tasks](#a03-tasks) |
| A | A04 | Data layer | **Done** | 100% | 2026-10-19; [phase review](reviews/A04-phase-review.md), [retro](retrospectives/A04-retro.md). See [A04 tasks](#a04-tasks) |
| A | A05 | Render engine, audio backend, export | **In progress** | ≈5% | T01, T03 and T04 in review. See [A05 tasks](#a05-tasks) |
| A | A06 | NUMISONA v1 | **In progress** | ≈30% | T01, T02, T03 and T05 in review. See [A06 tasks](#a06-tasks) |
| A | A07 | CLI, Python, v0.1.0-alpha | Not started | 0% | |
| A | A08 | Research instrumentation v0, ICAD 2027 | Not started | 0% | P01 without participant studies (OD-04) |
| B | B01 | Desktop shell, IPC, app core | Not started | 0% | |
| B | B02 | Design system and application frame | Not started | 0% | |
| B | B03 | AUDAXIS v1 | Not started | 0% | |
| B | B04 | Visualization and linked cursor | Not started | 0% | |
| B | B05 | Inspector, spec editor, projects | Not started | 0% | |
| B | B06 | SONOTRACE v1 | Not started | 0% | |
| B | B07 | Conformance, tutorial, co-design, v0.5.0-beta | Not started | 0% | JOSS P02 |
| C | C01 | PLOTSONA exact ingestion | Not started | 0% | |
| C | C02 | PLOTSONA raster digitizer | Not started | 0% | |
| C | C03 | CURVASONA | Not started | 0% | |
| C | C04 | AXONOTE | Not started | 0% | |
| C | C05 | ECHOMETRIC v1 | Not started | 0% | CHI 2028 P05 drafted and submitted (est. Sep 2027) |
| C | C06 | Hardening and v1.0 GA | Not started | 0% | P05 artifact package |
| D | D01 | TONALENS | Not started | 0% | |
| D | D02 | RESONUM | Not started | 0% | |
| D | D03 | Spatial and multichannel | Not started | 0% | |
| D | D04 | ML extraction and AI assist | Not started | 0% | |
| D | D05 | Plugins and WebAssembly | Not started | 0% | |
| D | D06 | Research studies programme | Not started | 0% | |

### A01 tasks

"Local" means the acceptance check passes on the Windows development machine; no task is Done until its CI run on GitHub is green. Details: [worklog](worklog/2026-10-18-a01-repository-bootstrap.md).

| Task | Status | Notes |
|---|---|---|
| A01-T01 Licence decision (ADR-0010, OD-09) | **Done** | 2026-10-18: code `MIT OR Apache-2.0`, docs and data `CC-BY-4.0`, DCO; ADR-0010 accepted |
| A01-T02 Repository, `main`, ruleset | **Done** | `main` ruleset active (PR required, 1 review + code owners, linear history, no force-push or deletion, 6 required checks, admin bypass per the interim rule). Squash-only merges, sign-off kept |
| A01-T03 Cargo workspace skeleton | **Done** | 11 crates build and test on 3 OSes (#1); all `license = "MIT OR Apache-2.0"` |
| A01-T04 Toolchain, rustfmt, clippy, editorconfig | **Done** | fmt and clippy `-D warnings` green in CI; Rust 1.99.0 pinned; MSRV 1.88 job green |
| A01-T05 `deny.toml` | **Done** | `cargo deny check` green in CI, including our own crates' licences |
| A01-T06 `xtask` (`ci`, `check-deps`, `codegen`, `wasm-check`) | **Done** | `check-deps` job green; rejects engine→engine and core→cpal (unit and fixture tests) |
| A01-T07 `ci.yml` | **Done** | Green on #1 across ubuntu, windows and macos; wall time about 1–2 min (budget ≤ 15 min) |
| A01-T08 PR title, DCO, Scorecard, CodeQL, Dependabot, label sync | **Done** | pr-title, dco and CodeQL green on #1; Scorecard green on `main`; 52 labels synced with `gh`; Dependabot alerts and security updates on |
| A01-T09 pnpm workspace skeleton | **Done** | `frontend` CI job green (install, lint, typecheck) |
| A01-T10 GitHub Project and issues | **Done** | [SONARITHM Roadmap](https://github.com/users/DuhItzAniket/projects/2): fields Status, Phase and Size; milestones A01–A08; one issue per Stage A task (82) |
| A01-T11 Secret scanning, vulnerability reporting, Discussions | **Done** | Secret scanning + push protection on; private vulnerability reporting and Discussions enabled; workflow token read-only by default. Reported-content moderation is unavailable for personal repositories; CODE_OF_CONDUCT.md now names the alternative channels |
| A01-T12 Docs CI | **Done** | markdown, internal-links, reuse and schema-diff green; external links advisory on PRs, failing only on the weekly run (`lychee.toml`) |
| A01-T13 Licence files, REUSE | **Done** | Licence files, `REUSE.toml` and SPDX headers; `reuse` check green in CI |

### A02 tasks

Work runs in a separate agent session in parallel with A03. The spikes are throwaway code in `spikes/`. Details: [gate report](reviews/A02-prototype-gate.md), [worklog](worklog/2026-10-19-a02-spikes.md).

| Task | Status | Notes |
|---|---|---|
| A02-T01 cpal output + loopback latency harness | **Done** (Windows) | Merged in #9. PB-01 **fails**: p95 123.1 ms at the 256 setting (F-031). macOS and Linux pending (R-22) |
| A02-T02 Owned render graph behind a draft `RenderGraph` | In review | #98: 361× real time offline, 0 deadline misses in 120 s (F-034) |
| A02-T03 Firewheel 0.14 behind the same contract; Miri | In review | #103: bit-identical output at equal CPU (F-035); Miri passes with Miri's allocator (F-037) |
| A02-T04 Offline determinism across OSes | In review | #104: bit-identical on Linux, Windows, macOS AArch64 (F-039) |
| A02-T05 Tauri 2.12 Channel and binary IPC | In review | #101: 1 MB p95 11.5–11.8 ms on HW-01, 7–8 ms on all CI runners (F-040). Real macOS and Linux hardware pending |
| A02-T06 WebGL2 1M-point waveform and spectrogram | In review | #102: 60 fps in WebView2 on both GPUs (F-043). WKWebView and WebKitGTK pending |
| A02-T07 React Aria prototype, screen-reader walkthroughs, OD-05 prototype | In review (prototype); walkthroughs pending | #107: prototype, UIA smoke test (F-045) and walkthrough scripts. NVDA, Narrator, Orca and VoiceOver runs are owner or volunteer tasks |
| A02-T08 Gate report; ADR-0002/0003/0004 recommendations | In review | Gate report draft; ADR-0004 amend; ADR-0002 and ADR-0003 stay provisional until the manual steps |

### A03 tasks

Work runs in a separate agent session in parallel with A02 (spikes). Each task lands through its own small PR; a task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A03-T01 Units newtypes | **Done** | Merged in #4: `Hz`, `Cents`, `Midi`, `Db`, `Seconds`, `Frames`, `SampleRate`, `Pan`; `sonarithm_core::math` over `libm`; round-trip property tests |
| A03-T02 IDs, provenance, diagnostics, error-code registry | **Done** | #12 (IDs, provenance, diagnostics), #6 (code registry and `cargo xtask codes`) |
| A03-T03 Score IR, canonical hash, review snapshots | **Done** | #106 (Score IR types and invariants), #111 (canonical hash and review snapshots; F-019) |
| A03-T04 Capability traits, registry | **Done** | #10, with `CompileContext`; [ADR-0014](../decisions/0014-capability-trait-signatures.md) accepted in #117 |
| A03-T05 RFC-0001 Spec v0.1 | **Done** | [RFC-0001](../rfcs/0001-spec-v0.1.md) accepted by the owner on 2026-10-19 with all 21 proposed resolutions; spec v0.1 frozen ([ADR-0015](../decisions/0015-spec-v0.1-frozen.md)) |
| A03-T06 Spec types, JSON Schema export | **Done** | #66: serde + schemars types per RFC-0001; `schemas/sonarithm-spec-0.1.schema.json` from `cargo xtask codegen`, diffed by `docs / schema-diff` |
| A03-T07 Semantic validation | **Done** | #96 (parse layers and semantic rules, a failing and a passing test each), #97 (miette reports with spans from JSON pointers) |
| A03-T08 Canonicalization, spec hash, migrations | **Done** | #99: RFC 8785 canonical form (Appendix B vectors pass), defaults as `Info` (`SNR-SPEC-080`), spec hash pinned on 3 OSes, migration framework; F-020 |
| A03-T09 Compiler skeleton, `CompileContext` | **Done** | #100: `Compiler<D>` runs every stage with no-op test engines; an inline-source spec compiles end to end to an empty Score |
| A03-T10 Spec parser fuzz target, proptest generators | **Done** | #105: `spec_parse` and `spec_migrate` targets, `nightly.yml`, valid-spec generators; 1 h run without a panic (see the phase review). #128 lets other crates reuse core's strategies |
| A03-T11 Doc-example validation test | **Done** | #97: every JSON example in 04 validates (4 specs, 9 fragments, the §2 skeleton) |

### A04 tasks

Work runs in a separate agent session, in parallel with A03, through small PRs into `crates/sonarithm-data`. A task is Done when its PRs are merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A04-T01 `DataBundle`/`Table`/`FieldMeta` on Arrow; `snr.*` metadata | Done | `FieldMeta` and the `SNR-DATA` code table (#113); tables and bundles (#114); Parquet and Arrow IPC round trips in T03 |
| A04-T02 CSV/TSV and JSON/NDJSON sources | Done | Encoding, delimiter and header sniffing; 95% type rule; corpus of 23 messy CSVs; JSON records, columns and rows at a pointer (#133) |
| A04-T03 Parquet and Arrow IPC sources | Done | Metadata and provenance columns survive both formats (property test) (#116) |
| A04-T04 XLSX/ODS via calamine | Done | Sheet and range selection, cached formula values, dates through number formats; workbook suite built in the tests (#135) |
| A04-T05 Type, role and concept inference | Done | Corpus of 27 files, 74 fields: precision 100% type, 98.5% role (one known miss), 100% concept (#136) |
| A04-T06 Provenance columns and propagation | Done | Canonical `__snr_prov`/`__snr_conf` at ingest; property tests for weakest class and lowest confidence; propagation table in 03 §4 (#138) |
| A04-T07 Transforms with a sandboxed rhai subset | Done | All eight transforms with unit and property tests; rhai subset fuel-limited (10 000 operations per row); provenance propagated per 03 §4 (#139) |
| A04-T08 M4 and LTTB downsampling; tile cache | Done | M4 pixel-exact against the full polyline and the paper's reference query (property test); LTTB keeps ends and threshold; LRU tile cache per generation (#141) |
| A04-T09 Data fingerprinting | Done | Canonical IPC (one batch, zeroed null slots); corpus digests pinned and checked on three OSes in CI (#142) |
| A04-T10 Benchmarks (PB-05); CSV and XLSX fuzzers | Done | PB-05 met: 0.51 s on HW-01 (34%), 0.91 s on HW-01e (F-070); fuzz targets for CSV, JSON, XLSX/ODS and Parquet/IPC in `nightly.yml`; fuzzing found allocation and panic issues in arrow-ipc and calamine, now validated or contained (F-071); a slow Parquet page header contained in a follow-up (#146) (#143) |

### A05 tasks

Work runs in a separate agent session through small PRs into `crates/sonarithm-dsp`, `-render`, `-audio` and `-export`, following [ADR-0004](../decisions/0004-audio-backend-and-render-graph.md) and [ADR-0016](../decisions/0016-windows-low-latency-backend.md). A task is Done when its PRs are merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A05-T01 DSP nodes (oscillators, ADSR, filters, panner) | In review | Oscillators: sine (IEEE-arithmetic polynomial, F-039), triangle, PolyBLEP saw and square, with spectral tests. ADSR with raised-cosine ramps of at least 5 ms and the SDR-22 conformance tests. One-pole and RBJ biquad filters with denormal flushing, constant-power panner, linear parameter smoother |
| A05-T02 ISO 226:2023 table; EBU R128 meter | Not started | |
| A05-T03 `RenderPlan` and plan builder | In review | Score → frames, frequencies, linear gains and pan gains; taps expanded; voice pools at twice the polyphony; `SNR-RENDER` code table; plan snapshot and polyphony property tests |
| A05-T04 Render graph, voice pool, scheduler, transport | In review | `Graph::process`: voice pools with oldest-first stealing, four buses at the sound-design §6.2 levels, sample-accurate dispatch (bit-identical output at blocks of 1 to 4096 frames), play, pause, stop, seek and loop with crossfades, speed without pitch change, live cues; provisional earcons |
| A05-T05 RT communication (rtrb, triple_buffer, basedrop) | Not started | |
| A05-T06 `AudioBackend` + `CpalBackend` | Not started | |
| A05-T07 `OfflineBackend`; WAV and FLAC writers | Not started | |
| A05-T08 Render manifest + WebVTT sidecar | Not started | |
| A05-T09 Master limiter + start-up gain ramp | Not started | |
| A05-T10 Allocation check in debug; 10-min soak; xrun counting | Not started | `assert_no_alloc` is BSD-1-Clause (F-038); licence question raised with the coordinator |
| A05-T11 Golden-audio harness; cross-OS CI job (OD-08) | Not started | |
| A05-T12 Benchmarks (PB-03) | Not started | |

### A06 tasks

Work runs in a separate agent session through one PR per task into `crates/engines/sonarithm-numisona`. A task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A06-T01 Scales | In review | `linear`, `log`, `symlog`, `sqrt`, `pow`, `quantile`, `band`, `ordinal`; `domain` auto, zero-based or explicit; `nice`; monotonicity and range property tests |
| A06-T02 Pitch spaces and range defaults (SDR-08, SDR-09) | In review | Log-frequency (default), linear, mel, ERB-rate and Bark spaces; 400–1400 Hz default range, 65–1480 Hz expanded; ranges in Hz, MIDI or cents; `sdr_08_*` and `sdr_09_*` tests |
| A06-T03 `concepts.toml`, polarity resolution, ambiguity diagnostics (SDR-12..15) | In review | 16 rows, each citing evidence (Walker 2007 counts where they exist); explicit → profile → concept → generic resolution, one-key flip; `SNR-ENG-020`…`023`; F-080 |
| A06-T04 `tone` and `glide` encoders, redundant duration and taps (A11Y-13), categorical timbre | Not started | |
| A06-T05 Psychophysical scaling (SDR-11) | In review | `f = f_ref · (d / d_ref)^(1/m)`, labelled as guidance; reproduces the Smith & Walker (2005) dollar mapping; range from the law or clamped to a given range |
| A06-T06 `auto` mark with density switching (SDR-20, experimental) | Not started | |
| A06-T07 Templates | Not started | |
| A06-T08 Erie importer (subset) | In review | Written from Erie's documentation; 8-spec gallery subset imports as valid v0.1 specs with a report of 19 approximated or unmapped constructs (`SNR-ENG-080`, `081`); F-081 |
| A06-T09 Rule-conformance suite and compile benchmarks (PB-04) | Not started | |

## 3. Current iteration

| Iteration | Dates | Goal | Tasks |
|---|---|---|---|
| I-00 | 2026-10-18 → 2026-11-01 | Owner decisions + A01 start | A01-T01, A01-T02, A01-T10; A01-T03..T09 and A01-T12 started early |

## 4. Engine maturity

| Engine | Maturity | Next milestone |
|---|---|---|
| NUMISONA | Designed | A06 |
| AUDAXIS | Designed | B03 |
| SONOTRACE | Designed | B06 |
| PLOTSONA | Designed | C01 |
| CURVASONA | Designed | C03 |
| AXONOTE | Designed | C04 |
| ECHOMETRIC | Designed | A08 (harness), C05 |
| TONALENS | Designed | D01 |
| RESONUM | Designed | D02 |

Maturity scale: `Designed` → `Prototype` → `Alpha` → `Beta` → `Stable` → `Research-grade` (for engines whose defaults await studies).

## 5. Research track snapshot

| Item | State |
|---|---|
| Findings logged | Background findings from the kickoff research ([findings-log](../research-paper/findings-log.md)) |
| Hypotheses | H-01..H-12 proposed ([hypotheses](../research-paper/hypotheses.md)) |
| Next paper | P01, ICAD 2027 (deadline est. Mar 2027) |
| Ethics | Not pursued: the owner works alone (OD-04, 2026-10-19); participant studies deferred, revisit at Gate B |

## 6. Change history

| Date | Change |
|---|---|
| 2026-10-18 | Status board created at the documentation baseline |
| 2026-10-18 | A01 started: workspace, tooling, xtask, workflows and pnpm skeleton built locally (T03–T09, T12) |
| 2026-10-18 | Licence decided (T01); licence files and REUSE (T13); GitHub ruleset, security and repo settings (T02, T11); A01 PR opened |
| 2026-10-19 | A03 started: unit newtypes (T01) and RFC-0001 (T05) |
| 2026-10-19 | RFC-0001 accepted with all 21 resolutions; spec v0.1 frozen (ADR-0015) |
| 2026-10-19 | A01 done: project board and Stage A issues (T10); phase review and retrospective |
| 2026-10-18 | A01 PR #1 merged as `02a44f3` with all checks green; owner context recorded (RV University, Windows-only hardware) |
| 2026-10-19 | A02 spikes measured (T01 merged in #9; T02–T07 in review); gate report draft recommends a conditional go; PB-01 fails on Windows shared mode |
| 2026-10-19 | A04 started: `FieldMeta` with `snr.*` metadata and the `SNR-DATA` code table (T01) |
| 2026-10-19 | A05 started: oscillators in `sonarithm-dsp` (T01) |
| 2026-10-19 | A03 done: all eleven tasks merged; phase review and retrospective; ADR-0014 and ADR-0015 accepted |
| 2026-10-19 | A06 started: `sonarithm-numisona` crate with scales (T01) |
| 2026-10-19 | A04 built: sources, inference, provenance, transforms, downsampling, fingerprints, PB-05 benchmark and fuzzers (T01–T10 in review) |
| 2026-10-19 | A04 done: all ten tasks merged (#113, #114, #116, #133, #135, #136, #138, #139, #141, #142, #143); phase review and retro |
