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
| Top risks | R-08 scope creep (20), R-09 bus factor (20), R-24 human-paced validation lags agent-paced building (16), R-02 screen readers in the webview (15), R-22 no macOS/Linux hardware (9) |

## 2. Phase board

Status values: `Not started` · `In progress` · `In review` · `Done` · `Blocked` · `Deferred`.

| Stage | Phase | Name | Status | Progress | Notes |
|---|---|---|---|---|---|
| — | — | Kickoff: research and documentation baseline | **Done** | 100% | 2026-10-18; see [worklog](worklog/2026-10-18-project-kickoff.md) |
| A | A01 | Repository bootstrap and governance | **Done** | 100% | 2026-10-19; [phase review](reviews/A01-phase-review.md), [retro](retrospectives/A01-retro.md). See [A01 tasks](#a01-tasks) |
| A | A02 | Technical spikes and prototype gate | **Waiting on owner** | ≈90% | All spikes merged and measured on Windows and CI; gate report on main (Conditional go). PB-01 fails on Windows shared mode (deferred to B03, #186). Remaining: the owner's screen-reader walkthroughs and the Linux and macOS hardware runs (#129), which also gate ADR-0002/0003 (OD-15). See [A02 tasks](#a02-tasks) |
| A | A03 | Core types, Spec v0.1, Score IR | **Done** | 100% | 2026-10-19; [phase review](reviews/A03-phase-review.md), [retro](retrospectives/A03-retro.md). See [A03 tasks](#a03-tasks) |
| A | A04 | Data layer | **Done** | 100% | 2026-10-19; [phase review](reviews/A04-phase-review.md), [retro](retrospectives/A04-retro.md). See [A04 tasks](#a04-tasks) |
| A | A05 | Render engine, audio backend, export | **Done** | 100% | 2026-10-21; [phase review](reviews/A05-phase-review.md). PB-01, PB-02 and PB-03 decisions and the cable test (#123) are with the owner. See [A05 tasks](#a05-tasks) |
| A | A06 | NUMISONA v1 | **Done** | 100% | 2026-10-19; [phase review](reviews/A06-phase-review.md). See [A06 tasks](#a06-tasks) |
| A | A07 | CLI, Python, v0.1.0-alpha | **In progress** | ≈90% | T01–T07 merged (#159, #162, #169, #171, #172, #174, #175, #176, #180). T08, the v0.1.0-alpha release, is prepared in #183 and waits for the owner's publishing steps (#129). See [A07 tasks](#a07-tasks) |
| A | A08 | Research instrumentation v0, ICAD 2027 | **Done** (owner steps pending) | 100% | 2026-10-21; [phase review](reviews/A08-phase-review.md). P01 submission or deferral and the v0.2.0-alpha tag are the owner's at Gate A. See [A08 tasks](#a08-tasks) |
| B | B01 | Desktop shell, IPC, app core | **In progress** | ≈25% | Planned 2026-10-21: 9 [tasks](../planning/implementation-plan.md#b01--desktop-shell-ipc-and-app-core), issues [#191](https://github.com/DuhItzAniket/SONARITHM/issues/191)–[#199](https://github.com/DuhItzAniket/SONARITHM/issues/199). Lane R: T01 (#270, `sonarithm-app`, [ADR-0017](../decisions/0017-ipc-protocol-v1.md)) and T06 (#272, network-deny harness) done; T02 in review. The rest waits for Gate U: the Windows walkthroughs and ADR-0002/0003 confirmed for Windows (OD-15, [#129](https://github.com/DuhItzAniket/SONARITHM/issues/129)). See [B01 tasks](#b01-tasks) |
| B | B02 | Design system and application frame | In progress | ≈25% | T08 (catalogues, locale formatting, pseudo-locale) done (#260); T01 (OKLCH tokens) done (#281); T02 (IBM Plex fonts, type scale) done (#284). Planned 2026-10-21: 11 [tasks](../planning/implementation-plan.md#b02--design-system-and-application-frame), issues [#200](https://github.com/DuhItzAniket/SONARITHM/issues/200)–[#210](https://github.com/DuhItzAniket/SONARITHM/issues/210). Lane R now: T01, T02, T08; the rest waits for Gate U; T10 needs the owner's component checks (HP-07), T11 needs OD-03 |
| B | B03 | AUDAXIS v1 | In progress | ≈60% | T01, T02, T03, T05, T06 and T11 done (#262, #264, #267, #269, #275, #266). Planned 2026-10-21: 14 [tasks](../planning/implementation-plan.md#b03--audaxis-v1-navigation-context-and-speech), issues [#211](https://github.com/DuhItzAniket/SONARITHM/issues/211)–[#224](https://github.com/DuhItzAniket/SONARITHM/issues/224). Lane R now: T01–T03, T05–T08, T11; T12 waits for [#186](https://github.com/DuhItzAniket/SONARITHM/issues/186) and [#123](https://github.com/DuhItzAniket/SONARITHM/issues/123); T13 needs NVDA, JAWS, VoiceOver and Orca runs (HP-08) and the Linux and macOS checks (HP-02, HP-03; OD-15) |
| B | B04 | Visualization and linked cursor | **In progress** | ≈10% | Planned 2026-10-21: 8 [tasks](../planning/implementation-plan.md#b04--visualization-and-linked-cursor), issues [#225](https://github.com/DuhItzAniket/SONARITHM/issues/225)–[#232](https://github.com/DuhItzAniket/SONARITHM/issues/232). Lane R: T03 in review (view tiles and table pages); the rest waits for Gate U |
| B | B05 | Inspector, spec editor, projects | **In progress** | ≈10% | Planned 2026-10-21: 8 [tasks](../planning/implementation-plan.md#b05--inspector-spec-editor-and-project-files), issues [#233](https://github.com/DuhItzAniket/SONARITHM/issues/233)–[#240](https://github.com/DuhItzAniket/SONARITHM/issues/240). Lane R: T01 (spec patches) and T04 (project bundles, [RFC-0002](../rfcs/0002-project-format-v1.md)) in review; the rest waits for Gate U |
| B | B06 | SONOTRACE v1 | **In progress** | ≈40% | Planned 2026-10-21: 7 [tasks](../planning/implementation-plan.md#b06--sonotrace-v1-transport-streams-and-monitoring), issues [#241](https://github.com/DuhItzAniket/SONARITHM/issues/241)–[#247](https://github.com/DuhItzAniket/SONARITHM/issues/247). Lane R now: all but T06. T01 (#261) and T03 (#263) done; T02 in review. See [B06 tasks](#b06-tasks) |
| B | B07 | Conformance, tutorial, co-design, v0.5.0-beta | Not started | 0% | Planned 2026-10-21: 10 [tasks](../planning/implementation-plan.md#b07--accessibility-conformance-tutorial-and-co-design-round-1), issues [#248](https://github.com/DuhItzAniket/SONARITHM/issues/248)–[#257](https://github.com/DuhItzAniket/SONARITHM/issues/257). JOSS P02 (calendar-gated, OD-17). Co-design round 1 needs HP-09 (OD-16); `v0.5.0-beta` waits for [#185](https://github.com/DuhItzAniket/SONARITHM/issues/185) and [#187](https://github.com/DuhItzAniket/SONARITHM/issues/187) |
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

### B02 tasks

Lane R tasks run in agent sessions now; the Lane U tasks wait for Gate U ([implementation plan §3.1](../planning/implementation-plan.md#31-how-stage-b-runs)).

| Task | Status | Notes |
|---|---|---|
| B02-T01 OKLCH token generator: dark, light and high-contrast themes, forced colors | **Done** (#281) | `packages/ui/tokens` generates `@sonarithm/ui/tokens.css` (four themes chosen by `data-theme` and the OS, forced colours, reduced motion, spacing, radius and motion tokens) and `tokens.json`; every design-language §5.3 pair passes in every theme (F-131); Stylelint rejects colour literals, raw spacing and durations, and DL-X01 to DL-X03 |
| B02-T02 IBM Plex fonts, type scale and tabular numerals | **Done** (#284) | IBM's unmodified WOFF2 files (Sans and Mono at 400, 500, 600) in `packages/ui/fonts` with IBM's licence, `LICENSES/OFL-1.1.txt` and a `REUSE.toml` annotation for those files only, as the supply-chain policy allows for fonts (coordinator, 2026-10-25). Also: `fonts.css`, type and numeric tokens, the `snr-numeric` class, `licenses.json` for the licences view, and the font test page. Plex needs no `tnum`: its default figures are tabular in every weight, and every weight has `zero` (F-132). IBM's npm packages are not used because their install script sends telemetry (F-133). Stylelint enforces the type scale and readouts (DL-X10) |
| B02-T08 Fluent catalogues, locale formatting, pseudo-locale check | **Done** (#260) | `sonarithm-i18n` (Fluent catalogues in `locales/`, ICU4X numbers and dates, unit names, `en-XA`) and `@sonarithm/i18n`; the Rust–TypeScript contract passes 24 shared cases on both sides (F-130); `sonarithm/no-literal-ui-string` lint. Built ahead of B01-T02: it needs no generated IPC type |

### B03 tasks

AUDAXIS runs in one agent session (Lane R: T01–T03, T05, T06, T11), one PR per task into `crates/engines/sonarithm-audaxis`.

| Task | Status | Notes |
|---|---|---|
| B03-T01 `sonarithm-audaxis` crate and NavTree builder | **Done** (#262) | NavTree types in `sonarithm_core::nav`; `ContextProvider::nav_tree` (ADR-0014's additive hook); the compiler returns the tree in `Compiled::nav`; snapshots for the six templates and reachability property tests. Even segments are provisional (OD-18). The facade, CLI `inspect` and Python expose the tree once AUDAXIS joins the default engines with its context tracks (T05), so no Score hash changes here |
| B03-T02 Cursor, movement verbs and where am I | **Done** (#264) | `nav::Cursor` with every verb of keyboard §4.2 (local, lateral, spatial, targeted, Enter); boundary and invalid outcomes keep the cursor and name their earcon; where am I at every node. Verb-by-level matrix snapshot, property tests over random verb sequences, no-dead-end and key-reachability tests |
| B03-T11 Stereo placement and the never-pan-alone guard | **Done** (#266) | The `Spatial` shaper (positions kept in [−1, 1]; constant-power panning in the render engine) and the A11Y-14 guard over every compiled Score: `SNR-ENG-100` to `102`; binaural and multichannel fall back to stereo (`SNR-ENG-103`). Mono-downmix conformance test and the `stereo-placement` golden case |
| B03-T03 Announcement composer and queue | **Done** (#267) | `announce::Composer` over the shared catalogue (`announce.ftl`): parts in the listener's order, terse and verbose, spoken unit names with a symbol display form, mandatory provenance qualifiers, where am I, pause; `AnnouncementQueue` with the §9.1 classes, supersede, priority and a history linked to nodes. Snapshots at both verbosities; every announcement passes the pseudo-locale check |
| B03-T06 Earcon set with text equivalents | **Done** (#275) | The render engine's `EarconSound` (compiled and interactive earcons, redesigned per keyboard §11) with `Command::Earcon` on the earcon bus; AUDAXIS's `earcons::CATALOGUE` with text equivalents, defaults and `E-GAP` for missing values. `earcons` golden reference updated; listening check HP-11 with the owner |
| B03-T05 Context tracks: tick track, dynamic reference tone, spoken legend | **Done** (#269) | `context` module: ticks with the render engine's new `click` instrument (SDR-16; nice step, OD-19), the dynamic reference tone (SDR-17; no static first-value reference anywhere), the spoken legend (SDR-18) as speech in the Score and `Composer::legend` for the app; context providers now receive the whole resolved spec. Golden case `context-cues`. AUDAXIS joins the facade's default engines in a separate step, because that changes every default render |
| B03-T07 Speech and sonification sequencing; self-voicing | In review ([#217](https://github.com/DuhItzAniket/SONARITHM/issues/217)) | App-core session. `sonarithm_app::speech::Sequencer` over AUDAXIS's queue on the transport's control thread: cue, then speech; only cues while a key repeats; speech held during playback except errors and alarms; a response during playback pauses it, or ducks with "duck instead of pause"; the current value on pause (A11Y-09). A −20 dB ducker stage in the render graph (`GainTarget::Duck`, measured within 0.05 dB). `sonarithm-speech`: the `Speaker` trait, a silent `Recorder`, and `System` over `tts` behind the opt-in `system` feature. WebVTT cues sequenced by estimated speech length. `announce_repeat` and the `announcement` event; the app compiles with AUDAXIS registered (F-142) |

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
| A02-T02 Owned render graph behind a draft `RenderGraph` | **Done** (merged in #98) | #98: 361× real time offline, 0 deadline misses in 120 s (F-034) |
| A02-T03 Firewheel 0.14 behind the same contract; Miri | **Done** (merged in #110, replacing #103) | #103: bit-identical output at equal CPU (F-035); Miri passes with Miri's allocator (F-037) |
| A02-T04 Offline determinism across OSes | **Done** (merged in #104) | #104: bit-identical on Linux, Windows, macOS AArch64 (F-039) |
| A02-T05 Tauri 2.12 Channel and binary IPC | **Done** on Windows and CI (merged in #101); real macOS and Linux hardware pending (owner, #129) | #101: 1 MB p95 11.5–11.8 ms on HW-01, 7–8 ms on all CI runners (F-040). Real macOS and Linux hardware pending |
| A02-T06 WebGL2 1M-point waveform and spectrogram | **Done** in WebView2 (merged in #102); WKWebView and WebKitGTK pending (owner, #129) | #102: 60 fps in WebView2 on both GPUs (F-043). WKWebView and WebKitGTK pending |
| A02-T07 React Aria prototype, screen-reader walkthroughs, OD-05 prototype | Prototype **done** (merged in #107); screen-reader walkthroughs pending (owner, #129) | #107: prototype, UIA smoke test (F-045) and walkthrough scripts. NVDA, Narrator, Orca and VoiceOver runs are owner or volunteer tasks |
| A02-T08 Gate report; ADR-0002/0003/0004 recommendations | Gate report merged (#108, #112); ADR-0004 confirmed and ADR-0016 accepted (#109); ADR-0002/0003 provisional until the walkthroughs (OD-15) | Gate report draft; ADR-0004 amend; ADR-0002 and ADR-0003 stay provisional until the manual steps |

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
| A05-T06 `AudioBackend` + `CpalBackend` | Done (#168) | `AudioBackend`, `CpalBackend` (silent smoke test on HW-01: Realtek speakers, 480-frame buffer, 0 late callbacks), timer-driven `NullBackend`, and `Player`, which rebuilds after device loss and keeps the position, paused (HS-05). Manual hot-plug script written; the owner's run is pending |
| A05-T07 `OfflineBackend`; WAV and FLAC writers | Done (#160) | `OfflineRender` drives `Graph::process` in fixed blocks; `write_audio` streams 24-bit or float WAV (hound) and 24-bit FLAC (flacenc). Files decode losslessly in hound, claxon and ffmpeg 9.0.1 |
| A05-T08 Render manifest + WebVTT sidecar | Done (#160) | `export()` writes audio, `<stem>.render.json` and `<stem>.cues.vtt`, with one static normalization gain (−16 LUFS within −1 dBTP by default, sound-design §6.3). The manifest validates against its generated schema, keeps no directories, and is byte-identical across runs; ffmpeg 9.0.1 reads the sidecar |
| A05-T09 Master limiter + start-up gain ramp | Done (#160) | Look-ahead true-peak limiter (−1 dBTP ceiling, 2 ms look-ahead, latency compensated offline); start-up ramp from −20 dB over 500 ms for playback. Stress Scores measure −1.19 and −1.16 dBTP with ebur128 (F-050) |
| A05-T10 Allocation check in debug; 10-min soak; xrun counting | Done (#178) | `allocation-counter` (MIT/Apache) replaces `assert_no_alloc` (BSD-1-Clause, F-038; F-054): no allocation in a scripted session or in device callbacks; `no-alloc` CI job on three OSes. Xruns reach the engine state. 10-minute silent soaks on HW-01: 0 underruns with WASAPI RAW (480-frame period) and exclusive (144 frames), 0 late callbacks with cpal. **PB-02 not declared met at 128 frames:** the driver has no 128-frame period (F-030); the render takes at most 14% of a 128-frame block (F-055) |
| A05-T11 Golden-audio harness; cross-OS CI job (OD-08) | Done (#178) | `cargo xtask golden` with six cases in `tests/golden/`; OD-08 decided (hashes and metrics, no audio in git); `golden` and `golden-compare` CI jobs; real-time path equal to the offline render bit for bit (null-device test). Cross-OS result: pending the PR's CI run |
| A05-T12 Benchmarks (PB-03) | Done (#178) | criterion bench and `render_speed` on the DS-006 Score: 91–96× on HW-01, **43–46× on HW-01e (4 E-cores)**, so PB-03 is not declared met under 12 §8 (F-053). Three optimizations landed while measuring. **PB-03 met on HW-01e on 2026-10-25 (60×; #185, F-134)** |
| A05-T13 `WasapiBackend` (ADR-0016) | Done (#168); owner step open (#123) | RAW shared mode by default (RAW confirmed on HW-01), exclusive opt-in, latency tool and per-device calibration table. Loopback self-test: p95 104.0 ms RAW, 145.6 ms default (F-052); exclusive mode quirk on the Realtek driver (F-051). **Owner step:** cable test and NVDA check (`manual-tests/a05-cable-latency.md`), issue #123 stays open |

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
| A07-T01 Facade crate API, rustdoc examples | **Done** (merged in #162) | `Sonarithm` (platform + NUMISONA), `Spec`, `compile`/`compile_with` (host datasets through `Platform::with_dataset`, `SNR-COMP-026`), `quick`, summary and L1/L2 description; doc examples run as tests; builds for wasm32 without default features. Rendering joins when A05's export crate lands |
| A07-T02 CLI commands, `--json`, `--plain` | **Done** (merged in #169 and #174) | Part 1 merged in #169 (`render`, `validate`, `inspect`, `lint` stub, `quick -o`, `schema`, `spec migrate`). Part 2: `play` and `devices` on A05's player (text-line announcements, keys, `--device`, `--speed`, `null` backend), and `quick` plays. Screen-reader check of `play` in a terminal: owner (#129) |
| A07-T03 Python package (PyO3/maturin abi3) | **Done** | Merged in #171. `sonarithm._core` + Python layer, abi3 ≥ 3.10; pyarrow, pandas, polars, dict, records, arrays and paths; `render`, `wav`, notebook `play`; wheels built and 19 pytest tests passed on Linux, Windows and macOS with Python 3.10 and 3.14 (wheels run 39503116734), including Score and WAV parity with the facade |
| A07-T04 Examples gallery | **Done** | Merged in #175. DS-001 to DS-005 as CSV from seeded generators (`cargo xtask gallery`, checked in CI), a spec each, README and notebook; the `ci / gallery` job renders every spec to FLAC (artifact `gallery-renders`); facade and notebook tests; licences recorded in datasets.md (CC BY 4.0) |
| A07-T05 release-plz, git-cliff, wheels workflow | **Done** | Merged in #176. `release-plz.toml` (one version group, facade tag `vX.Y.Z`, Keep a Changelog via embedded git-cliff), `release-plz.yml` (release PR, release on merge, dry run), `wheels.yml` publish (Trusted Publishing, TestPyPI rehearsal), `release-binaries.yml` (auditable CLI, SBOM, notices, attestations). Dry run: `cargo publish --workspace --dry-run` packaged and verify-built all 11 crates. Owner steps: release-process.md |
| A07-T06 Zenodo, `CITATION.cff` | **Done** (merged in #172); author names and ORCID are the owner's (#129) | `.zenodo.json` (software, description, keywords, related repository) and `CITATION.cff` in step; `cargo xtask citation` checks both in CI and `--release DATE` sets version and date. Author names and ORCID: owner (#129) |
| A07-T07 Quickstart tutorial | **Done** (merged in #180); the three-person timing test is the owner's (#129) | `docs/tutorials/quickstart.md`: install (Python or CLI), CSV to FLAC, read the description and the applied defaults; every command checked. The three-person timing test (NFR-12) is the owner's; its protocol is in the tutorial (#129) |
| A07-T08 Release v0.1.0-alpha | Not started | Publishing needs the owner's go |

### A08 tasks

Work runs in a separate agent session through one PR per task into `crates/engines/sonarithm-echometric`, in parallel with A05 and A07. No participant data is collected (OD-04). A task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| A08-T01 Stimulus generators | **Done** | Merged in #163. GEN-001 to GEN-004 with published parameters verified against the papers (F-110 to F-113); pinned manifests on three OSes |
| A08-T02 Replication gallery v0 | **Done** | Merged in #177. `replication/`: Smith & Walker context cues (6 specs), Nees & Walker density × reversals (16), Tucker Brown transits (18), Erie subset (9); every render's record committed and checked on three OSes by `cargo xtask replication`; deviations documented per group (F-114) |
| A08-T03 Study runner, `sonarithm study run` | **Done** | Merged in #166 (library) and #179 (the subcommands). `sonarithm study check` and `study run` with the pilot protocol `examples/studies/pilot-001` run end to end with scripted answers; screen-reader walkthrough is an owner step (`docs/engineering/manual-tests/a08-study-run-screen-reader.md`). Library `harness::study`: protocol files (`sonarithm-study/0`), seeded trial order, scoring against the stimulus's ground truth, the JSONL log (`sonarithm-study-log/0`), pause and resume from the log; `SNR-ENG-600`–`621` |
| A08-T04 Technical benchmark report | **Done** | Merged in #189. [Report](../research-paper/benchmark-report-a08.md): end-to-end export 131.7× (HW-01) and 70.9× (HW-01e) real time (F-115); golden cases and 49 gallery renders byte-identical on three OSes (F-116); budget decisions #185, #186, #187 |
| A08-T05 P01 draft | **Done** (draft) | Merged in #165. Draft v0 in `papers/P01-icad-2027-toolkit/src/paper.md` with authorship, artifact, AI-usage and submission records; every citation checked against its source. Submission is the owner's act; the ICAD 2027 call is not published yet |
| A08-T06 Release v0.2.0-alpha | Prepared | [Release checklist](releases/v0.2.0-alpha.md). The release PR comes from release-plz after v0.1.0-alpha; the owner tags and publishes |

### B01 tasks

Lane R work runs in one agent session (app core and SONOTRACE), one PR per task, alongside the AUDAXIS session. Lane U tasks (T03–T05, T07–T09) keep `status/blocked` until Gate U. A task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| B01-T01 `sonarithm-app` headless app core | **Done** (#270) | `App::dispatch` over the IPC protocol of [ADR-0017](../decisions/0017-ipc-protocol-v1.md): datasets, spec with undo tokens, compile, transport on a control thread that owns A05's `Player` (F-140), export jobs, versioned settings with crash-safe writes, devices. 16 headless scenarios on the null device (open → compile → play → seek → stop → export; A11Y-15; device loss; v0 settings migration; fault injection; no absolute path in any reply, error or event) |
| B01-T02 Typed IPC: ts-rs codegen, `IpcTransport` | In review | `cargo xtask codegen` writes `packages/ipc/src/generated/protocol.ts` (ts-rs 12) and a Rust-encoded binary-view fixture; `REPLY_KINDS` and `EVENT_CHANNELS` generated from the Rust tables; `@sonarithm/ipc` has `IpcTransport`, `connect` (fails fast on a protocol mismatch), `createMockTransport` and `readView`; a contract test sends every command through `dispatch`; Node test runner on the TypeScript sources in CI |
| B01-T06 Network-deny integration test | **Done** (#272) | `cargo xtask network-deny` runs the app-core tests under strace, `sandbox-exec` or WFP auditing and fails on any outbound attempt; `--self-test` must catch a deliberate probe; `network-deny` CI job on three OSes; mechanisms in testing-strategy §10 |

### B04 tasks

| Task | Status | Notes |
|---|---|---|
| B04-T03 M4 and LTTB view tiles and table pages | In review | `view_get` and `table_page` through `App::dispatch_binary`: M4 tiles from A04's tile cache (pixel-exact through the protocol, property test), LTTB overviews, sorted table pages with `__snr_row`, ISO 8601 dates and validity masks; unique table IDs per dataset so cached tiles never mix |

### B05 tasks

| Task | Status | Notes |
|---|---|---|
| B05-T01 Spec patch model with undo and redo | In review | `spec_apply` takes a JSON Patch (RFC 6902), applies it all or nothing as one undoable edit and returns the result's validation diagnostics; `spec_validate` checks text without changing the session; `SNR-APP-040`. Property test: apply, then undo, gives back the first spec, byte for byte |
| B05-T04 Project bundles, autosave, recovery | In review | `.sonarithm` bundles per [RFC-0002](../rfcs/0002-project-format-v1.md) and [ADR-0018](../decisions/0018-project-format-v1.md) in `sonarithm-app::project`: the manifest first with a SHA-256 per member; stored Parquet read in place (condition B, F-141); linked data only as a regular data file whose fingerprint matches (condition A); zip-slip, member, size and ratio limits. Autosaves (the newest 5, at most every 30 s, 2 GiB in all, TODO(verify) in B05-T07) are recovered on the next start. The `project_*` commands, a recent list by file name, `SNR-APP-030` to `035`, and the `project_open` fuzz target |

### B06 tasks

Lane R work runs in the app-core and SONOTRACE session, one PR per task. T06 (the stream UI) waits for Gate U. A task is Done when its PR is merged with CI green.

| Task | Status | Notes |
|---|---|---|
| B06-T01 `StreamTable` | **Done** | Merged in #261. `sonarithm_data::StreamTable`: sequence numbers, row or time-window capacity with a hard row limit, shared-buffer snapshots, `SNR-DATA-071` when a reader falls behind; property tests for bounded memory, contiguity and snapshot consistency; provenance kept in canonical form |
| B06-T02 Stream adapters with consent | **Done** (#265) | `sonarithm-stream`: WebSocket and OSC (default), serial and MQTT (features), file tail; `Consent::user_approved` is the only way to `connect`; JSON Schema subset or inferred message schema; bounded `StreamTable`; reconnect with backoff; pause and stop. Loopback tests for WebSocket (with a reconnect), OSC and tail, a consent test, three decoder fuzz targets in `nightly.yml` |
| B06-T03 `sonarithm-sonotrace` live and transport | **Done** | Merged in #263. New engine crate: the `stream` source over a connected window (`SNR-ENG-200`), `live::Live` window encoding that emits each new row's events once (deterministic replay test, property test), scrub grains at the cursor (Score snapshot, `a11y_09_*`), long-series windows. No `encode_incremental` on the core trait (an RFC if B06-T07 needs it) |
| B06-T04 Monitoring soundscape | **Done** (#278) | `sonotrace::monitor`, a context provider for `mode: "monitoring"`: a triangle bed on the monitor bus whose brightness, never its level, rises on detector events and settles back (SDR-35); `E-STREAM` per event, `E-ALARM` only for an alarm configured with `salience: "high"`; the comfort share reported as an experimental `Info` diagnostic (SDR-39, INFER); `SNR-ENG-240`, `241`; golden case `monitoring-soundscape` (F-143). Every value is a proposal until the B06 evaluation (sound-design §10.7) |
| B06-T05 Announcement rate limits, coalescing, pause, stop and hide | In review ([#245](https://github.com/DuhItzAniket/SONARITHM/issues/245)) | `sonotrace::rate_limit::RateLimiter`: at most one notice per stream every 5 s, coalesced into a summary, events older than 10 s dropped, errors and alarms never limited or dropped (TODO(verify): the §9.1 proposal values, B07). Through the app core: `stream_offer` (a one-time token; nothing opens) and `stream_connect` (the user's approval opens one connection), `stream_pause`, `stream_resume`, `stream_hide`, `stream_stop`, `stream_list`; connection changes become rate-limited stream announcements from the `stream.ftl` catalogue; `SNR-APP-070` to `072` |

## 3. Current iteration

| Iteration | Dates | Goal | Tasks |
|---|---|---|---|
| I-00 | 2026-10-18 → 2026-11-01 | Owner decisions + A01 start | A01-T01, A01-T02, A01-T10; A01-T03..T09 and A01-T12 started early |

## 4. Engine maturity

| Engine | Maturity | Next milestone |
|---|---|---|
| NUMISONA | Alpha | A07 (`v0.1.0-alpha`); lint warnings move to ECHOMETRIC in C05 |
| AUDAXIS | Prototype | B03 (NavTree in review; cursor, announcements and context next) |
| SONOTRACE | Prototype | B06 (streams, live encoding and scrub grains in review; monitoring and rate limits next) |
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
| 2026-10-19 | A05: all thirteen tasks implemented; T01, T03 and T04 merged, the rest landing in three bundles; PB-02 and PB-03 not declared met on HW-01 (F-053, F-055); owner steps listed in the A05 worklog |
| 2026-10-21 | A05 done: all thirteen tasks merged (#118–#120, #122, #126, #160, #168, #178); [phase review](reviews/A05-phase-review.md) with the gate summary; owner decisions pending for PB-01, PB-02 and PB-03 |
| 2026-10-21 | A08 done: generators (#163), code blocks (#161), P01 draft (#165), study runner (#166, #179), replication gallery (#177), benchmark report (#189); [phase review](reviews/A08-phase-review.md); v0.2.0-alpha checklist prepared |
| 2026-10-21 | Stage B planned to task level: 67 tasks, issues [#191](https://github.com/DuhItzAniket/SONARITHM/issues/191)–[#257](https://github.com/DuhItzAniket/SONARITHM/issues/257), milestones B01–B07; Lane R (23 tasks) can start, Lane U waits for the A02 walkthroughs (Gate U); owner items HP-01 to HP-17; OD-15 to OD-17, R-24 to R-27, F-120 ([worklog](worklog/2026-10-21-stage-b-planning.md)) |
| 2026-10-21 | B02 started: Fluent catalogues, locale formatting and the pseudo-locale check (T08) in review |
| 2026-10-25 | B02-T02 merged (#284). PB-03 met on HW-01e: the benchmark Score renders at 60× real time on four E-cores with bit-identical output (#185, F-134); the A05 waiver is closed |
| 2026-10-25 | B02-T01 merged (#281); B02-T02 (IBM Plex fonts, type scale, DL-X10 lint) in review after the coordinator confirmed OFL-1.1 for fonts is existing policy |
| 2026-10-25 | B03-T06 merged (#275); B02-T01 (OKLCH tokens, Stylelint) in review; B02-T02 blocked: OFL-1.1 for the IBM Plex fonts is not yet in `REUSE.toml` or `LICENSES/` (owner decision) |
| 2026-10-21 | B03-T05 merged (#269); T06 (earcon set) in review |
| 2026-10-21 | B03-T03 merged (#267); T05 (context cues) in review |
| 2026-10-21 | B03-T11 merged (#266); T03 (announcement composer and queue) in review |
| 2026-10-21 | B03-T02 merged (#264); T11 (stereo placement and the A11Y-14 guard) in review |
| 2026-10-21 | B03-T01 merged (#262); T02 (cursor, verbs, where am I) in review |
| 2026-10-21 | B02-T08 merged (#260); B03 started: `sonarithm-audaxis` and the NavTree (T01) in review; OD-18 (even segment size) opened |
| 2026-10-21 | OD-15 closed by the coordinator: Gate U opens after the Windows walkthroughs (HP-01) and ADR-0002/0003 confirmed for Windows (HP-04); the Linux session (HP-02) moves to B03's exit; the Tauri host stays thin |
