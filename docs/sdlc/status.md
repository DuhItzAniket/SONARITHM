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
| A | A05 | Render engine, audio backend, export | **In progress** | ≈60% | T01–T05 and T07–T09 merged; bundle 2 (T06, T13) and bundle 3 (T10–T12) in review. See [A05 tasks](#a05-tasks) |
| A | A06 | NUMISONA v1 | **Done** | 100% | 2026-10-19; [phase review](reviews/A06-phase-review.md). See [A06 tasks](#a06-tasks) |
| A | A07 | CLI, Python, v0.1.0-alpha | **In progress** | ≈15% | #156 fixed (#159); the facade in review. See [A07 tasks](#a07-tasks) |
| A | A08 | Research instrumentation v0, ICAD 2027 | **In progress** | ≈25% | P01 without participant studies (OD-04). See [A08 tasks](#a08-tasks) |
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
| A05-T01 DSP nodes (oscillators, ADSR, filters, panner) | Done (#118, #119, #120) | Oscillators: sine (IEEE-arithmetic polynomial, F-039), triangle, PolyBLEP saw and square, with spectral tests. ADSR with raised-cosine ramps of at least 5 ms and the SDR-22 conformance tests. One-pole and RBJ biquad filters with denormal flushing, constant-power panner, linear parameter smoother |
| A05-T02 ISO 226:2023 table; EBU R128 meter | Done (#160) | ISO 226:2023 Formulas (1) and (2) with Table 1 (checked against the standard's official preview); compensation on by default in the plan (SDR-24, 60 phon provisional), never boosting; `LoudnessMeter` over ebur128 passes EBU Tech 3341 cases 1–5 within 0.1 LU. Annex B comparison TODO(verify) |
| A05-T03 `RenderPlan` and plan builder | Done (#122) | Score → frames, frequencies, linear gains and pan gains; taps expanded; voice pools at twice the polyphony; `SNR-RENDER` code table; plan snapshot and polyphony property tests |
| A05-T04 Render graph, voice pool, scheduler, transport | Done (#126) | `Graph::process`: voice pools with oldest-first stealing, four buses at the sound-design §6.2 levels, sample-accurate dispatch (bit-identical output at blocks of 1 to 4096 frames), play, pause, stop, seek and loop with crossfades, speed without pitch change, live cues; provisional earcons |
| A05-T05 RT communication (rtrb, triple_buffer, basedrop) | Done (#160) | `realtime(graph)` splits into `Controller` and `Processor`: rtrb command ring (1024, ≤ 64 per block, coalescing on overflow), plan swap with crossfade through `basedrop::Owned`, `EngineState` through `triple_buffer`; shuttle test of the protocol over 200 random schedules |
| A05-T06 `AudioBackend` + `CpalBackend` | In review | `AudioBackend`, `CpalBackend` (silent smoke test on HW-01: Realtek speakers, 480-frame buffer, 0 late callbacks), timer-driven `NullBackend`, and `Player`, which rebuilds after device loss and keeps the position, paused (HS-05). Manual hot-plug script written; the owner's run is pending |
| A05-T07 `OfflineBackend`; WAV and FLAC writers | Done (#160) | `OfflineRender` drives `Graph::process` in fixed blocks; `write_audio` streams 24-bit or float WAV (hound) and 24-bit FLAC (flacenc). Files decode losslessly in hound, claxon and ffmpeg 9.0.1 |
| A05-T08 Render manifest + WebVTT sidecar | Done (#160) | `export()` writes audio, `<stem>.render.json` and `<stem>.cues.vtt`, with one static normalization gain (−16 LUFS within −1 dBTP by default, sound-design §6.3). The manifest validates against its generated schema, keeps no directories, and is byte-identical across runs; ffmpeg 9.0.1 reads the sidecar |
| A05-T09 Master limiter + start-up gain ramp | Done (#160) | Look-ahead true-peak limiter (−1 dBTP ceiling, 2 ms look-ahead, latency compensated offline); start-up ramp from −20 dB over 500 ms for playback. Stress Scores measure −1.19 and −1.16 dBTP with ebur128 (F-050) |
| A05-T10 Allocation check in debug; 10-min soak; xrun counting | Not started | `assert_no_alloc` is BSD-1-Clause (F-038); licence question raised with the coordinator |
| A05-T11 Golden-audio harness; cross-OS CI job (OD-08) | Not started | |
| A05-T12 Benchmarks (PB-03) | In review | criterion bench and `render_speed` on the DS-006 Score: 91–96× on HW-01, **43–46× on HW-01e (4 E-cores)**, so PB-03 is not declared met under 12 §8 (F-053). Three optimizations landed while measuring |
| A05-T13 `WasapiBackend` (ADR-0016) | In review | RAW shared mode by default (RAW confirmed on HW-01), exclusive opt-in, latency tool and per-device calibration table. Loopback self-test: p95 104.0 ms RAW, 145.6 ms default (F-052); exclusive mode quirk on the Realtek driver (F-051). **Owner step:** cable test and NVDA check (`manual-tests/a05-cable-latency.md`), issue #123 stays open |

### A06 tasks

Work runs in a separate agent session through one PR per task into `crates/engines/sonarithm-numisona`. A task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A06-T01 Scales | **Done** | Merged in #134. `linear`, `log`, `symlog`, `sqrt`, `pow`, `quantile`, `band`, `ordinal`; `domain` auto, zero-based or explicit; `nice`; monotonicity and range property tests |
| A06-T02 Pitch spaces and range defaults (SDR-08, SDR-09) | **Done** | Merged in #137. Log-frequency (default), linear, mel, ERB-rate and Bark spaces; 400–1400 Hz default range, 65–1480 Hz expanded; ranges in Hz, MIDI or cents; `sdr_08_*` and `sdr_09_*` tests |
| A06-T03 `concepts.toml`, polarity resolution, ambiguity diagnostics (SDR-12..15) | **Done** | Merged in #144. 16 rows, each citing evidence (Walker 2007 counts where they exist); explicit → profile → concept → generic resolution, one-key flip; `SNR-ENG-020`…`023`; F-080 |
| A06-T04 `tone` and `glide` encoders, redundant duration and taps (A11Y-13), categorical timbre | **Done** | Merged in #151. `Numisona` engine and encoders with `RowRef` and confidence; filter, time order and pace; redundant taps and duration; timbre per category; missing values; `TrackRequest` gains substrate, fields, missing and mode; synthetic example datasets; snapshots; F-082 |
| A06-T05 Psychophysical scaling (SDR-11) | **Done** | Merged in #140. `f = f_ref · (d / d_ref)^(1/m)`, labelled as guidance; reproduces the Smith & Walker (2005) dollar mapping; range from the law or clamped to a given range |
| A06-T06 `auto` mark with density switching (SDR-20, experimental) | **Done** | Merged in #153. Tones up to 10 events/s, a glide above; every use reported as experimental (`SNR-ENG-011`, H-02); `density` module |
| A06-T07 Templates | **Done** | Merged in #154. `line-overview`, `bar-discrete`, `scatter-xy`, `histogram`, `box-five-number`, `heatmap-rowscan` with slot binding; each renders every compatible example dataset through the render engine; the compiler compiles `DataBundle` with spec transforms |
| A06-T08 Erie importer (subset) | **Done** | Merged in #150. Written from Erie's documentation; 8-spec gallery subset imports as valid v0.1 specs with a report of 19 approximated or unmapped constructs (`SNR-ENG-080`, `081`); F-081 |
| A06-T09 Rule-conformance suite and compile benchmarks (PB-04) | **Done** | Merged with the phase review. A named test for every rule NUMISONA owns, checked against the catalogue; the Erie gallery subset renders; SDR-21 volatility advice; PB-04: 35 ms (HW-01) and 54 ms (HW-01e) for 100k rows × 4 tracks; F-083, F-084 |

### A07 tasks

Work runs in a separate agent session through one PR per task, based on `main`. `render`, `quick` and `play` wait for A05-T06 to T08. A task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A07 prerequisite #156 `without_inference()` keeps declared fields | **Done** | Merged in #159. Declared `fields` are written into the schema without inference; NUMISONA's example datasets declare their roles and read without inference (data unchanged, same fingerprints) |
| A07-T01 Facade crate API, rustdoc examples | In review | `Sonarithm` (platform + NUMISONA), `Spec`, `compile`/`compile_with` (host datasets through `Platform::with_dataset`, `SNR-COMP-026`), `quick`, summary and L1/L2 description; doc examples run as tests; builds for wasm32 without default features. Rendering joins when A05's export crate lands |
| A07-T02 CLI commands, `--json`, `--plain` | In progress | `render`, `validate`, `inspect`, `lint` (stub), `quick` (with `-o`), `schema`, `spec migrate`; exit codes 0–3; assert_cmd + insta snapshots; CLI render bytes equal the facade's. `play` and `devices` wait for A05's audio backend |
| A07-T03 Python package (PyO3/maturin abi3) | In progress | `sonarithm._core` + Python layer, abi3 ≥ 3.10; pyarrow, pandas, polars, dict, records, arrays and paths; `render`, `wav`, notebook `play` and `<audio>`; 19 pytest tests incl. Score and WAV parity with the facade; `wheels.yml` builds and tests wheels on 3 OSes.  |
| A07-T04 Examples gallery | **Done** | Merged in #175. DS-001 to DS-005 as CSV from seeded generators (`cargo xtask gallery`, checked in CI), a spec each, README and notebook; the `ci / gallery` job renders every spec to FLAC (artifact `gallery-renders`); facade and notebook tests; licences recorded in datasets.md (CC BY 4.0) |
| A07-T05 release-plz, git-cliff, wheels workflow | In review | `release-plz.toml` (one version group, facade tag `vX.Y.Z`, Keep a Changelog via embedded git-cliff), `release-plz.yml` (release PR, release on merge, dry run), `wheels.yml` publish (Trusted Publishing, TestPyPI rehearsal), `release-binaries.yml` (auditable CLI, SBOM, notices, attestations). Dry run: `cargo publish --workspace --dry-run` packaged and verify-built all 11 crates. Owner steps: release-process.md |
| A07-T06 Zenodo, `CITATION.cff` | In review | `.zenodo.json` (software, description, keywords, related repository) and `CITATION.cff` in step; `cargo xtask citation` checks both in CI and `--release DATE` sets version and date. Author names and ORCID: owner (#129) |
| A07-T07 Quickstart tutorial | Not started | Three-person timing test: owner (#129) |
| A07-T08 Release v0.1.0-alpha | Not started | Publishing needs the owner's go |

### A08 tasks

Work runs in a separate agent session through one PR per task into `crates/engines/sonarithm-echometric`, in parallel with A05 and A07. No participant data is collected (OD-04). A task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A08-T01 Stimulus generators | **Done** | Merged in #163. GEN-001 to GEN-004 with published parameters verified against the papers (F-110 to F-113); pinned manifests on three OSes |
| A08-T02 Replication gallery v0 | Not started | Needs A05's export (WAV and manifest) on main |
| A08-T03 Study runner, `sonarithm study run` | In review (library) | `harness::study`: protocol files (`sonarithm-study/0`), seeded trial order, scoring against the stimulus's ground truth, the JSONL log (`sonarithm-study-log/0`), pause and resume from the log; `SNR-ENG-600`–`621`. The subcommand follows A07's CLI |
| A08-T04 Technical benchmark report | Not started | Needs A05 and A07 on main |
| A08-T05 P01 draft | In review | Draft v0 in `papers/P01-icad-2027-toolkit/src/paper.md` with authorship, artifact, AI-usage and submission records; every citation checked against its source. Submission is the owner's act; the ICAD 2027 call is not published yet |
| A08-T06 Release v0.2.0-alpha | Not started | Release PR and checklist only; the owner tags |

## 3. Current iteration

| Iteration | Dates | Goal | Tasks |
|---|---|---|---|
| I-00 | 2026-10-18 → 2026-11-01 | Owner decisions + A01 start | A01-T01, A01-T02, A01-T10; A01-T03..T09 and A01-T12 started early |

## 4. Engine maturity

| Engine | Maturity | Next milestone |
|---|---|---|
| NUMISONA | Alpha | A07 (`v0.1.0-alpha`); lint warnings move to ECHOMETRIC in C05 |
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
| 2026-10-19 | A06: T01–T05 and T08 merged (scales, pitch spaces, concept table, encoders, psychophysical scaling, Erie importer); the compiler compiles `DataBundle` with transforms (#152) |
| 2026-10-19 | A06 done: templates (T07, #154), the auto mark (T06, #153), the rule-conformance suite and PB-04 (T09); [phase review](reviews/A06-phase-review.md) |
| 2026-10-19 | A04 built: sources, inference, provenance, transforms, downsampling, fingerprints, PB-05 benchmark and fuzzers (T01–T10 in review) |
| 2026-10-19 | A04 done: all ten tasks merged (#113, #114, #116, #133, #135, #136, #138, #139, #141, #142, #143); phase review and retro |
| 2026-10-19 | A08 started: `sonarithm-echometric` with stimulus generators GEN-001 to GEN-004 (T01) |
| 2026-10-19 | A07 started: `without_inference()` keeps declared fields (#156) |
| 2026-10-21 | A08: T01 merged (#163), engine code blocks (#161); P01 draft v0 in review (T05) |
| 2026-10-21 | A07: #156 fixed (#159); facade API (T01) in review |
