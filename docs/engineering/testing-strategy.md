# Testing strategy

> **Status:** Baseline v1 (2026-10-18). **Owner:** Architecture lead (accessibility testing: QA).
> **Related:** [01-overview §7](../architecture/01-overview.md#7-quality-attributes-and-budgets) (budgets) · [05 §4](../architecture/05-compiler-and-score-ir.md#4-score-ir) (Score IR) · [06 §8](../architecture/06-audio-engine.md#8-determinism) (determinism) · [coding-standards](coding-standards.md) · [ci-cd](ci-cd.md) · [dev-environment](dev-environment.md) · [design-rules](../research/design-rules.md) · [accessibility-conformance-plan](../design/accessibility-conformance-plan.md) · [traceability-matrix](../sdlc/traceability-matrix.md)

This document defines what we test, with which tools, where the tests live and when they run. Commands marked "after A01" (or a later phase) depend on the workspace and harnesses created in that phase.

## Contents

1. [Principles](#1-principles)
2. [The test pyramid for a sound engine](#2-the-test-pyramid-for-a-sound-engine)
3. [Unit and property tests](#3-unit-and-property-tests)
4. [Score IR snapshot tests](#4-score-ir-snapshot-tests)
5. [Golden-audio tests](#5-golden-audio-tests)
6. [Design-rule conformance tests](#6-design-rule-conformance-tests)
7. [Fuzzing](#7-fuzzing)
8. [Concurrency and unsafe code](#8-concurrency-and-unsafe-code)
9. [Performance and soak tests](#9-performance-and-soak-tests)
10. [Integration tests through the headless app core](#10-integration-tests-through-the-headless-app-core)
11. [CLI and Python tests](#11-cli-and-python-tests)
12. [Frontend tests](#12-frontend-tests)
13. [Accessibility testing](#13-accessibility-testing)
14. [Latency measurement](#14-latency-measurement)
15. [Research validation](#15-research-validation)
16. [Coverage targets](#16-coverage-targets)
17. [Flaky tests](#17-flaky-tests)
18. [Test data management](#18-test-data-management)
19. [Test types by phase](#19-test-types-by-phase)

---

## 1. Principles

1. **The Score IR is the canonical test artifact.** Spec in, Score out (P1). We snapshot the Score, which is exact and reviewable, and compare audio only within tolerances, because floating-point and SIMD results differ across CPUs ([research report, "Testing a sound engine"](../research/research-report.md#testing-a-sound-engine)).
2. **Test the shipping code path.** Golden audio is rendered by the offline driver, which calls the same `process()` as the device callback (P4).
3. **Determinism is tested, not assumed.** The same spec, data, seed and engine versions must produce a byte-identical serialized Score on every OS in the CI matrix ([05 §4.2](../architecture/05-compiler-and-score-ir.md#42-invariants-property-tested), hypothesis H-08).
4. **Every evidence-backed default has a test named after its rule.** A reader can go from `SDR-08` to the code, the test and the evidence ([section 6](#6-design-rule-conformance-tests)).
5. **Automation cannot judge what a screen reader says.** A manual screen-reader matrix is mandatory every release ([section 13](#13-accessibility-testing)).
6. **Tests are hermetic.** No network, no wall clock, no OS entropy, no audio device (except the dedicated latency and soak jobs). Randomness in tests is seeded and the seed is printed on failure.

## 2. The test pyramid for a sound engine

From the base (many, fast, every PR) to the top (few, slow, scheduled or manual):

| Layer | What it proves | Tooling | Runs |
|---|---|---|---|
| Unit | Functions and types behave as documented | `cargo nextest run`, `cargo test --doc` | Every PR, 3 OSes |
| Property | Spec → Score invariants hold for generated inputs | proptest | Every PR; more cases nightly |
| Score IR snapshots | The compiler's output is exactly what was reviewed | insta | Every PR, 3 OSes |
| Cross-OS Score determinism | Byte-identical Score on Windows, macOS, Linux | full-precision Score hashes checked in the `golden` CI job | Every PR, 3 OSes |
| Golden audio | Rendered audio matches reference within tolerance | harness (A05), ebur128 | Every PR, 3 OSes |
| Design-rule conformance | Defaults and behaviours match SDR/A11Y rules | unit, snapshot, frontend tests named by rule ID | Every PR |
| Fuzzing | Parsers and importers never panic, hang or explode in memory | cargo-fuzz | Nightly; longer runs before releases |
| Concurrency and unsafe | Lock-free protocols and unsafe code are sound | Miri, shuttle (loom optional), `allocation-counter` | Allocation check every PR; Miri and shuttle on PRs touching the crates and nightly |
| Integration | App behaviour end to end without a webview | headless app core (B01), facade before that | Every PR |
| CLI and Python | Public interfaces behave and stay stable | assert_cmd + insta; pytest on wheels | Every PR |
| Frontend | Components, flows and visuals in every webview engine | Vitest 4, Storybook 10, Playwright, WebdriverIO | Every PR (real-app E2E nightly) |
| Performance and soak | Budgets met, no regressions, no xruns | criterion, soak harness, loopback harness | Nightly on hosted runners (relative); reference hardware (absolute), started by a maintainer |
| Manual accessibility | Real assistive technology works | screen-reader matrix, Chartability | Every release |
| Research validation | Replicated designs still render as published | replication gallery | Every PR (gallery), per study |

## 3. Unit and property tests

**Unit tests** live in `#[cfg(test)] mod tests` blocks next to the code. Run them with `cargo nextest run` (after A01). Doctests run separately with `cargo test --doc`, because nextest does not execute them.

**Property tests** (proptest) cover the invariants that define a correct compiler ([05 §4.2](../architecture/05-compiler-and-score-ir.md#42-invariants-property-tested)):

| Invariant | Property |
|---|---|
| Monotonic time | Events in each track are sorted by start time; `t ≥ 0`; glides are monotonic in time; `t + dur ≤ header.duration`. |
| Ranges respected | Every pitch lies within the resolved channel range after polarity is applied; gains and pans lie in their declared ranges. |
| Polarity flip symmetry | Flipping polarity maps each pitch `x` to `lo + hi − x` and leaves every other field unchanged; flipping twice is the identity (SDR-15). |
| Determinism | Compiling the same input twice yields identical serialized bytes; compiling with 1 and N worker threads yields identical bytes. |
| Mode safety | `mode = Analysis` ⇒ no quantization shaper ran (SDR-10). |
| Time conversion | `Seconds → Frames` uses round-half-to-even; converting back stays within half a frame. |
| Spec round trip | Parse → serialize → parse gives an equal spec; migrations are idempotent. |
| Transforms (A04) | Filter, sort, aggregate and resample obey their algebraic properties; M4 downsampling preserves per-column min and max. |

Rules:

- Strategies for spec fragments and `DataBundle`s are shared through a non-default `test-support` feature of the crate that owns the types, so no extra crate is needed. Other crates enable it from their dev-dependencies (for example `sonarithm-core = { workspace = true, features = ["test-support"] }`). Because `cargo metadata` unifies those features, a `test-support` dependency must not reach a crate the dependency rules forbid; proptest is therefore built without its `fork` and `timeout` features (#121).
- proptest regression files (`proptest-regressions/`) are committed.
- PR runs use the default case count; the nightly job raises it with `PROPTEST_CASES`.

## 4. Score IR snapshot tests

The Score snapshot is the project's primary regression artifact.

- **Format.** JSON through insta, with floats rounded to 1 µs for time, 0.01 cents for pitch and 0.01 dB for gain. Large Scores are snapshotted as summaries (counts, extents, hashes of event arrays) plus the first and last N events ([05 §4.3](../architecture/05-compiler-and-score-ir.md#43-snapshot-format)).
- **Strict determinism check.** Rounding makes snapshots readable; it does not replace exactness. Each corpus case's manifest also stores the SHA-256 of the full-precision serialized Score, and the `golden` CI job ([ci-cd](ci-cd.md)) recomputes it on every OS. A mismatch on any OS fails the job, which proves byte-identical output across platforms.
- **Location.** The workspace corpus (spec + data reference + snapshot per case) lives under `tests/golden/` ([02 §1](../architecture/02-repository-and-crate-layout.md#1-repository-tree-target)). Crate-level snapshots stay in insta's default `snapshots/` directories next to their tests.
- **Corpus.** At least one case per mark type, channel, scale, context cue and engine as each lands, plus every spec example in the docs (A03 exit: 100% of documented spec examples validate).
- **Release independence.** The Score header records the engine versions, which are the crate versions (ADR-0013), so a raw Score snapshot or a pinned Score hash changes with every release. Snapshots and pinned hashes write engine versions as `*` first (NUMISONA's `release_independent`, the facade's summary snapshot, `tests/golden/parity`), and the CLI snapshots filter the version, so a release PR changes no snapshot. Determinism across platforms is still checked on the full Score.
- **CI behaviour.** CI runs with `INSTA_UPDATE=no`, so a mismatch fails without writing files.
- **Updating.** Run `cargo insta review` locally, accept only changes you can explain, and state the reason in the PR. Snapshot changes and unrelated code changes do not share a PR when the snapshot diff is large.
- **IR version.** A change to the snapshot format bumps `ir_version` ([02 §6](../architecture/02-repository-and-crate-layout.md#6-versioning)) and regenerates all snapshots in a PR that contains nothing else, so the diff is purely mechanical.

## 5. Golden-audio tests

Golden-audio tests compare offline renders with committed reference renders. The harness lands in A05.

### 5.1 Fixed conditions

| Condition | Value |
|---|---|
| Renderer | `OfflineBackend` (same graph and `process()` as real time) |
| Sample rate | 48 kHz unless a case states otherwise |
| Block size | 256 frames unless a case states otherwise |
| Noise and grains | Seeded from the case's spec seed (SDR-02) |
| Denormals | FTZ/DAZ as in the real-time callback |
| Reference format | WAV, 32-bit float |

Every condition is recorded in the case manifest and in the render manifest ([06 §7](../architecture/06-audio-engine.md#7-export)).

### 5.2 Comparison and tolerances

| Metric | Pass condition | Notes |
|---|---|---|
| Length | Exact, in frames | |
| Null-test residual | Peak of (render − reference) ≤ −90 dBFS | Budget from [01 §7](../architecture/01-overview.md#7-quality-attributes-and-budgets). RMS residual is reported alongside for diagnosis. |
| Integrated loudness | Δ ≤ 0.1 LU (ebur128) | |
| Spectral centroid | Frame-wise Δ within the case's tolerance | Tolerance is per case in the manifest; the provisional default is 1% relative over non-silent frames, confirmed in A05. |
| Same-platform hash | SHA-256 recorded for the platform that produced the reference | Informational: a hash change within tolerance is reported in the job summary, not failed. |

Additional golden checks:

- **Block-size invariance.** The same case rendered at 64, 256 and 1024 frames must pass the tolerances above against the 256-frame reference. This proves sample-accurate scheduling.
- **Real-time path equivalence.** The real-time graph driven by a scripted fake device produces the same samples as the offline driver, within tolerance.
- **Cross-OS null test.** References are produced on one designated platform (the Linux x86_64 CI runner). The other OSes compare against them, which is the A05 exit criterion.

### 5.3 Golden-file policy

| Rule | Detail |
|---|---|
| Location | `tests/golden/`, one directory per case with its manifest. |
| Manifest | `case.json`: the Score source (`score.json` in the case directory, or a generator and seed), sample rate, block size, tolerances, the reference fingerprint with its origin platform, and the reason for the last update. Engine versions and `ir_version` are inside the Score. |
| Storage | **Decided in A05 (OD-08): no reference audio in git.** Renders are bit-identical across OSes (F-039), so each manifest records the reference render's SHA-256 and metrics (length, integrated loudness, true and sample peak, RMS, per-second spectral centroid). CI renders every case on the three OSes, checks each against its manifest, compares Windows and macOS with Linux (PB-06), and keeps the renders as workflow artifacts for listening. |
| Size | References are short (seconds, not minutes). Coverage comes from many small cases. |
| Updates | Only through `cargo xtask golden --update` (after A05), optionally limited to named cases. The command re-renders on the reference platform, rewrites manifests and prints metric deltas. Never edit or replace reference files by hand. |
| Review | The PR states why the output changed and includes the metric deltas. A reviewer listens to every changed case (headphones, low gain) before approving. |
| Isolation | Golden updates are kept apart from unrelated changes so the reason is unambiguous. |

## 6. Design-rule conformance tests

Every rule in the [design-rules catalogue](../research/design-rules.md) that code implements has at least one test named after its ID.

| Item | Convention |
|---|---|
| Rust test name | `<rule id in snake case>_<behaviour>`: `sdr_08_default_pitch_range_is_400_to_1400_hz`, `a11y_14_pitch_only_information_survives_mono_downmix` |
| Frontend test name | `describe("A11Y-05: keyboard operable", …)` |
| Location | A `conformance` test module in the crate or package that owns the rule (Owner column of the catalogue) |
| Doc comment | Cites the rule ID and strength: `/// SDR-22 (MUST): every note has non-zero onset and offset ramps.` |
| Lint rules | Each lint rule has a positive test (fires) and a negative test (does not fire) (C05 exit) |
| `INFER` rules | Tests assert the behaviour is present and flagged experimental (for example SDR-20, A06) |
| Manual-only rules | Rules that cannot be automated (A11Y-32 screen-reader matrix, A11Y-34 co-design) appear as checklist items in release and gate reviews, not as tests |
| Changing a default | Per catalogue rule 1, the PR states the old and new value, links the finding (`F-###`) or experiment (`EXP-###`), and updates the test in the same PR |

The [traceability matrix](../sdlc/traceability-matrix.md) maps rule → implementation → test. Checking test names against the matrix is automated in A06, whose exit criterion requires a test for every SDR rule NUMISONA owns.

## 7. Fuzzing

cargo-fuzz (libFuzzer, nightly toolchain) on Linux CI runners. Harnesses live in a `fuzz/` directory inside the owning crate (the cargo-fuzz default), so no new top-level directory is needed.

| Target | Crate | Phase |
|---|---|---|
| Spec parser and semantic validation | `sonarithm-spec` | A03 (exit: ≥ 1 h with no panic) |
| Spec migrations | `sonarithm-spec` | A03 |
| CSV/TSV, JSON/NDJSON, Parquet, Arrow IPC importers | `sonarithm-data` | A04 |
| XLSX/ODS importer | `sonarithm-data` | A04 |
| Erie spec importer | `sonarithm-spec` | A06 |
| Project bundle loader (`.sonarithm`) | `sonarithm-app` | B05 |
| Office chart XML, SVG, vector PDF, Vega/Plotly/Highcharts JSON importers | `sonarithm-plotsona` | C01 |
| Raster image input | `sonarithm-plotsona` | C02 |
| Expression parser facade | `sonarithm-curvasona` | C03 |

Requirements for every target: no panic, no hang (per-input timeout), bounded memory (`-rss_limit_mb`), and a round-trip assertion where one exists (parse → serialize → parse).

Schedule: 30 minutes per target in `nightly.yml` ([ci-cd](ci-cd.md)), a two-minute smoke run on pull requests that touch `sonarithm-core` or `sonarithm-spec`, and at least 1 hour per parser target before each release (manual dispatch of `nightly.yml` with `seconds: 3600`). Run locally from the owning crate with `cargo +nightly fuzz run <target>`. On Windows, cargo-fuzz needs AddressSanitizer: put the MSVC directory that holds `clang_rt.asan_dynamic-x86_64.dll` (`VC/Tools/MSVC/<version>/bin/Hostx64/x64`) on `PATH`, otherwise the target exits with `STATUS_DLL_NOT_FOUND`; `-s none` does not link with MSVC. Contributors changing a parser or importer run its target locally before opening the PR. Every crash is minimized and committed as a regular regression test in the owning crate.

The `sonarithm-data` targets (`csv_read`, `json_read`, `xlsx_read`, `columnar_read`) also check that every table they load survives an Arrow IPC round trip with the same fingerprint. The data layer turns panics of third-party decoders (arrow-ipc, parquet, calamine) into errors, so `xlsx_read` and `columnar_read` replace libFuzzer's abort-on-panic hook and only panics that escape `read_bytes` count as crashes. On Windows, libFuzzer treats any Rust panic as a crash even when it is caught, so run those two targets on Linux; on Windows the ASan runtime DLL (`clang_rt.asan_dynamic-x86_64.dll`, in the MSVC `bin\Hostx64\x64` folder) must be on `PATH`.

## 8. Concurrency and unsafe code

| Tool | Scope | When |
|---|---|---|
| Miri (`cargo +nightly miri test -p <crate>`) | Pure-Rust `unsafe` in the crates on the [unsafe register](coding-standards.md#5-unsafe-code), and our use of lock-free crates in `sonarithm-render` | PRs touching those crates (a `miri` job joins `ci.yml` when the first registered unsafe code or RT protocol lands, A05); nightly |
| shuttle (default) or loom (small primitives) | Command-queue overflow and coalescing; plan swap with the basedrop collector (no use-after-free, no free on the RT thread); telemetry writes and reads; live-event ordering; compiler cancellation | PRs touching those modules; nightly with more iterations |
| `allocation-counter` (debug build; replaces `assert_no_alloc`, F-038) | The render callback driven through a scripted session: play, seek, loop, plan swap, cue triggers, gain changes, stop | Every PR from A05 (`no-alloc` job in [ci-cd](ci-cd.md)) |

Production code reaches atomics and thread primitives through a small `sync` shim module that switches to shuttle or loom types under a `cfg`, so model tests exercise the real code.

## 9. Performance and soak tests

### 9.1 Benchmarks and budgets

criterion benchmarks live in crate `benches/` directories and, for cross-crate scenarios, in the workspace `benches/` ([02 §1](../architecture/02-repository-and-crate-layout.md#1-repository-tree-target)). Budgets come from [01 §7](../architecture/01-overview.md#7-quality-attributes-and-budgets); reference hardware is defined in [12-cross-cutting-concerns §8](../architecture/12-cross-cutting-concerns.md#8-performance-budgets).

| Budget | Benchmark | Introduced |
|---|---|---|
| CI duration baseline | CI timing record in the benchmarks log | A01 |
| Channel throughput and binary IPC latency | IPC spike benchmark | A02 (budget set), B01 |
| 1M-row CSV load ≤ 1.5 s | `sonarithm-data` load bench | A04 |
| Offline render ≥ 50× real time (4 tracks, 10k events) | render bench | A05 |
| Compile ≤ 200 ms (100k rows × 4 tracks), ≤ 1 s (1M rows) | compile bench | A06 |
| Cold start ≤ 2 s | startup bench | B01 |
| 60 fps playhead, ≤ 8 ms scripting per frame with 1M points | frontend perf test in each webview | B04 |
| Key press → cue onset ≤ 50 ms p95 | loopback harness ([section 14](#14-latency-measurement)) | A02, B03 |
| Installer ≤ 40 MB without ML models | release CI size check | C06 |

### 9.2 Regression policy

- Hosted CI runners are too noisy for absolute numbers. They run benchmarks nightly (`nightly.yml`) for relative comparison only. Budget verification runs on reference hardware, started by a maintainer on a trusted ref; self-hosted runners never run untrusted PR code ([ci-cd](ci-cd.md)).
- The nightly job compares each budgeted benchmark with the baseline from `main`. A median slowdown greater than **10%**, outside criterion's noise interval, fails the job and opens an issue. The threshold is provisional and is calibrated in A05 after the runner noise floor is measured.
- PRs that touch hot paths (`sonarithm-render`, `sonarithm-dsp`, compiler stages, data loading) include before and after numbers from the relevant benchmark.
- Missing an absolute budget is a gate blocker unless the gate review records an accepted exception.
- Phase-end numbers are recorded in the [benchmarks log](../research-paper/benchmarks-log.md).

### 9.3 Soak tests

| Soak | Condition | Pass | Runs |
|---|---|---|---|
| Device soak (A05) | 10 min at 128-frame buffers on reference hardware, representative Score | 0 xruns | Started by a maintainer on a trusted ref: before every release, and after merges that change the scheduler, voice pool, plan swap or backend |
| Allocation soak (A05) | Debug build, `allocation-counter`, scripted session on a timer-driven null device | No allocation in the callback | Short session on every PR (`no-alloc` job); full 10 minutes nightly ([06 §12](../architecture/06-audio-engine.md#12-real-time-rules-summary)) |
| Stream soak (B06) | 1 kHz input stream sonified for 1 h | No xruns, bounded memory (RSS sampled) | Memory and throughput nightly on hosted runners; xrun check on reference hardware before releases. As built (B06-T07): the `stream_soak` example of `sonarithm-app` plays a synthetic 1 kHz `tail` stream live through the app core and judges memory, rows and keeping up (`--strict`); the nightly `stream-soak` job runs it for 10 minutes on three OSes on the null device, and `--output system --max-xruns 0` is the device run |

## 10. Integration tests through the headless app core

The headless app core (`sonarithm-app`, B01) holds all non-UI desktop behaviour, so most app behaviour is testable without a webview ([01 §4](../architecture/01-overview.md#4-containers-c4-level-2)). Integration tests live in the workspace `tests/` directory.

- **Before B01**: integration tests drive the facade (`sonarithm`): spec + data → Score → rendered file + manifest.
- **From B01**: scenarios drive `sonarithm-app` with a null audio backend: open dataset → compile → plan → play → seek → export; undo/redo; autosave and crash recovery; settings and profiles.
- **Network deny (B01)**: the scenarios run under a harness that fails on any network connection attempt (P8): `cargo xtask network-deny`, in the `network-deny` CI job on three OSes. It builds the `sonarithm-app` test executables and runs each under the OS's mechanism:

  | OS | Mechanism | What counts as an attempt |
  |---|---|---|
  | Linux | `strace -f -e trace=connect,sendto,sendmsg,sendmmsg` | A traced call with an `AF_INET` or `AF_INET6` address (Unix sockets do not count) |
  | macOS | `sandbox-exec` with `(deny network-outbound (remote ip "*:*"))`, so attempts also fail | A kernel `deny(1) network-outbound` line for the process in `log show` |
  | Windows | Windows Filtering Platform connection auditing (`auditpol`, needs an elevated shell, as CI runners have) | An outbound 5156 or 5157 event for the process in the Security log |

  `--self-test` is the negative test: an ignored probe (`network_probe_connects`) connects to 192.0.2.1 (TEST-NET-1) over TCP and UDP, and the harness fails unless it sees those attempts, so a mechanism that stops working cannot pass silently. The self-test reads the OS log for up to about a minute, because the unified log and the Security log persist entries late on busy runners. On macOS, if the log still has no entry, the probe's own report that the sandbox refused its connections (EPERM) counts, since that shows the sandbox is enforcing. B01-T09 runs the same mechanisms against the packaged app.
- **Project round trip (B05)**: a saved project reopens bit-identically; every inspector edit round-trips through the spec without loss.
- **IPC contract**: ts-rs types are regenerated in CI and must produce no diff; Tauri command handlers are tested with typed payloads; frontend tests use the same generated types through `mockIPC`.

## 11. CLI and Python tests

**CLI** (`sonarithm`, A07): `assert_cmd` runs the binary; `insta` snapshots stdout, stderr and the exit code. Tests set `NO_COLOR=1` and a fixed terminal width so miette output is stable. Cases cover `render` (file plus manifest), `validate` (diagnostic codes), `--deny-warnings` exit codes, `schema` output equal to the committed `schemas/`, and `inspect` output.

**Python** (A07):

- pytest runs against **built wheels** installed into a fresh virtual environment on each OS, for the minimum and the newest supported Python (3.10 and 3.14 in `wheels.yml`; the abi3 floor is 3.10, see [release-process](release-process.md)). Editable installs (`maturin develop`) are for local work only.
- A job without optional dependencies proves `import sonarithm` works without pandas or Polars; a second job installs pandas, Polars and pyarrow and tests zero-copy input.
- Tests cover output dtype and shape (`float32`, `(frames, channels)`), exception classes and their `code` attributes, GIL release (a threaded smoke test), docstring doctests, and stub accuracy with `python -m mypy.stubtest sonarithm`.
- Example notebooks execute headless in CI.

## 12. Frontend tests

| Layer | Tooling | Scope |
|---|---|---|
| Unit | Vitest 4 | Zustand stores, XState machines (model-based tests of transport and export machines), formatters, keymap resolution, token contrast calculations |
| Component | Vitest 4 browser mode with `toMatchScreenshot` | Component behaviour and visuals. Baselines are taken on one pinned Linux Chromium image to avoid font-rendering differences, plus a small per-OS smoke set |
| Workbench and a11y | Storybook 10 with the Vitest addon and the accessibility addon (axe-core) | Every component story, in every state (focus, disabled, error) and theme (dark, light, high contrast, forced colours). CI fails on serious and critical violations |
| Cross-engine E2E | Playwright with Tauri `mockIPC` in Chromium, WebKit and Firefox; `@axe-core/playwright` on key flows | User flows and visual regression without the Rust backend |
| Real-app E2E | WebdriverIO `@wdio/tauri-service` on Windows, Linux and macOS, in `nightly.yml` | The packaged app with the real core (Trial from B05 per [tech-stack](tech-stack.md)). The official `tauri-driver` does not support macOS. The embedded provider on macOS 26 with Tauri 2.12 is unverified and is checked when the tool is adopted |
| Performance | Browser performance traces in WebView2, WKWebView and WebKitGTK | B04 budgets |

Source: [05-desktop-frontend-stack §9](../research/notes/05-desktop-frontend-stack.md#9-frontend-testing-vitest-playwright-tauri-webdriver-per-os-axe-core-storybook-vs-ladle-visual-regression).

**How it runs (B02-T03).** CI's `frontend-browser` job runs in the `mcr.microsoft.com/playwright` image whose tag matches the `playwright` version in `pnpm-workspace.yaml`:

| Command | What it checks |
|---|---|
| `pnpm -C packages/ui test:stories` | Every Storybook story (`packages/ui/src/**/*.stories.tsx`) renders in Chromium and passes axe, once per theme (Vitest projects `storybook-dark`, `-light`, `-highContrastDark`, `-highContrastLight`, run one after another by `test/run-stories.ts`, because parallel projects race on Storybook's shared dependency cache on Windows); Storybook's `a11y.test: "error"` turns any violation into a failed test, and each component's keyboard test is its story's `play` function |
| `pnpm -C packages/ui test:seeded` | Negative test: only the stories tagged `seeded` run, and the command succeeds only if each failed with its axe rule (`button-name`) |
| `pnpm -C packages/ui test:visual` | `toMatchScreenshot` baselines in `packages/ui/test/visual/__screenshots__`, taken in the CI image only. Without committed baselines the job writes them, uploads them as the `visual-baselines` artifact and fails, so a person reviews them before they are committed |
| `pnpm -C apps/desktop test:e2e` | Playwright in Chromium, WebKit and Firefox against `e2e.html`: the real UI with Tauri's `mockIPC` answering from `@sonarithm/ipc`'s in-memory host. `e2e/axe.ts` (serious and critical WCAG 2.2 A and AA violations) and `e2e/keyboard.ts` (`tabThrough`: Tab only, every tabbable element reached, focus visible at each stop), each with a seeded negative test (`e2e/gates.spec.ts`) |
| `apps/desktop/scripts/uia-check.ps1` | In the Windows `desktop` job: the A02 UI Automation tree dump (F-045) of the real app in WebView2, failing on a missing expected role and name (`apps/desktop/test/uia/*.json`) or an unnamed interactive element. The dump is uploaded as `uia-tree-windows` |

Locally, `pnpm -C packages/ui storybook` opens the workbench; the browser tests need `pnpm exec playwright install chromium` once. Screenshot tests do not run locally, because fonts render differently per OS.

## 13. Accessibility testing

### 13.1 Automated (every PR)

- Lint: jsx-a11y strict rules ([coding-standards §12.2](coding-standards.md#122-linting-and-formatting)) and `sonarithm/no-literal-ui-string` for hard-coded UI text ([coding-standards §12.7](coding-standards.md#127-strings-and-internationalisation)).
- Localization: the pseudo-locale check (`en-XA`) over every catalogue message, in Rust and in `@sonarithm/i18n` (`pnpm -r test`), and the Rust–TypeScript message contract on shared cases ([12 §5](../architecture/12-cross-cutting-concerns.md#5-internationalization-and-localization)).
- axe-core in Storybook and Playwright, failing on serious and critical violations.
- Token tests assert WCAG 2.2 contrast for every theme pairing: 4.5:1 for text, 3:1 for non-text and focus indicators (`packages/ui/test/tokens.test.ts`, from B02-T01). Seeded CSS shows that Stylelint rejects the detectable design-language anti-patterns (`packages/ui/test/stylelint.test.ts`).
- Keyboard-only Playwright flows (no pointer events) assert focus order, no traps and visible focus.
- Rust-side A11Y conformance tests (for example A11Y-14 mono survival, A11Y-16 announcement rate limits, A11Y-20 speech/sonification sequencing).

Automation does not detect wrong announcements, wrong reading order, missing context or broken screen-reader interaction modes. That is what the manual matrix is for.

### 13.2 Manual screen-reader matrix (every release, A11Y-32)

| Screen reader | OS | Webview | Configurations |
|---|---|---|---|
| NVDA (installed, not portable) | Windows 11 | WebView2 | Audio ducking off, "duck when outputting speech and sounds", "always duck"; sound split on and off |
| JAWS | Windows 11 | WebView2 | Default settings |
| Narrator | Windows 11 | WebView2 | Default settings |
| VoiceOver | macOS (current release) | WKWebView | Default settings |
| Orca | Linux, X11 session | WebKitGTK | Default settings |
| Orca | Linux, Wayland session | WebKitGTK | Default settings |
| Braille | One refreshable braille display, with NVDA or VoiceOver | — | Review line and sparkline output (A11Y-21) |

Procedure:

1. Run the scripted task suite (from B03: find the maximum, compare two series, read the value at x) plus the flows changed in the release.
2. Record pass, fail and notes per cell, as defined in the [accessibility conformance plan](../design/accessibility-conformance-plan.md). The results are linked from the release's gate or release review.
3. File each failure as an issue with a severity. An open critical accessibility issue blocks the release (B07 exit).

### 13.3 Audits and conformance

- **Chartability**: the 14 critical tests every release and all 50 heuristics at major versions (A11Y-33).
- **ACR**: v0 in B07 and v1 in C06, against WCAG 2.2 AA via WCAG2ICT (A11Y-31).
- **Co-design rounds** (A11Y-34) are not tests. While there is no ethics route ([OD-04](../planning/open-decisions.md#closed-decisions)), they run as paid product usability feedback, and no research data is published from them. Their findings are logged in the [findings log](../research-paper/findings-log.md) and triaged into issues.
- **Optional automation**: Guidepup for NVDA and VoiceOver smoke tests is assessed in B03; its maturity is unverified.

## 14. Latency measurement

Budget: key press → cue onset ≤ 50 ms p95 at 48 kHz with a 256-frame buffer ([01 §7](../architecture/01-overview.md#7-quality-attributes-and-budgets), [06 §9](../architecture/06-audio-engine.md#9-latency-and-clock-synchronization)). Measured by the loopback harness in A02 and B03.

1. **Device setup.** Use a physical loopback cable from an output to an input on the same audio interface for sign-off numbers. A virtual loopback device measures only the software path and is for harness development.
2. **Stimulus.** The harness triggers the interactive path and records the host timestamp. In A02 and A05 the trigger enters through the core's command path. From B03 it is a synthetic key event injected into the webview, so UI handling and IPC are included.
3. **Capture.** Record the input stream with cpal's host timestamps. Detect cue onset by threshold crossing or cross-correlation with the known cue waveform.
4. **Trials.** At least 200 trials with randomized inter-trial intervals, so triggers do not lock to block boundaries. Report p50, p95, p99 and maximum.
5. **Interpretation.** The result includes capture-path latency, so it is an upper bound on output latency unless input latency is separately calibrated. Record OS, host API, device, buffer size, sample rate and cpal's reported input and output latencies with every run.
6. **Logging.** Results go to the [benchmarks log](../research-paper/benchmarks-log.md), and each measurement campaign produces a finding (`F-###`), as A02 requires.

## 15. Research validation

- **Replication gallery (A08).** Each gallery item (a committed spec plus its manifest) is a regression test: its Score snapshot must match and its audio must pass the golden tolerances. A08 exit requires every item to render reproducibly.
- **Stimulus generators (A08).** ECHOMETRIC generators are deterministic for a seed (snapshot tests) and statistically correct (property tests, for example a requested SNR is met within tolerance).
- **Detectors (C05).** Precision and recall on labelled synthetic data, with thresholds per detector.
- **Extraction corpora (C01, C02).** Exact-source extraction ≤ 0.5% relative error; raster median ≤ 2%; provenance class asserted per point.
- **Experiment runner.** Log records validate against their schema; stimulus order is deterministic for a seed.
- A change that alters a gallery output for a perceptual reason links a finding in the [findings log](../research-paper/findings-log.md).

## 16. Coverage targets

Coverage is a floor that flags untested areas, not a goal.

| Scope | Line coverage |
|---|---|
| `sonarithm-core`, `sonarithm-spec`, `sonarithm-compiler` | ≥ 85% |
| Engine crates | ≥ 75% |
| Other library crates (`data`, `dsp`, `render`, `audio`, `export`, `stream`) | Reported; a target is set at the crate's first phase review |
| UI glue (`apps/desktop`, Tauri command wrappers, binding wrappers, `xtask`) | No target |

- Measured with a cargo coverage tool chosen in A05 (cargo-llvm-cov is the candidate); frontend coverage comes from Vitest and is informational.
- CI reports coverage on PRs (a coverage job is added to `ci.yml` in A05). Targets are checked at phase reviews and become a CI gate for the named crates once first met.
- Exclusions (generated code, test support) are listed in the coverage configuration with reasons.
- Do not write tests only to raise the number. Property, snapshot and golden tests count.

## 17. Flaky tests

A flaky test passes and fails on the same commit and environment.

1. **Quarantine within one working day.** Move the test to the nextest quarantine group, which is excluded from required checks but still runs nightly, and open an issue with the logs and an owner.
2. **Fix or delete within 10 working days.** A quarantined test that is not fixed in that time is deleted, and the gap is recorded in the issue.
3. **No retries in the default profile.** nextest retries are allowed only in the quarantine profile. Retries hide flakiness.
4. **Determinism tests are never retried.** A flaky snapshot, golden or cross-OS hash test is a determinism bug and is treated as high priority.
5. **No sleeps.** Timing-dependent logic uses simulated clocks and scripted devices.
6. **Visual tests.** Screenshot baselines come from the pinned image; a test that is flaky only because of rendering noise gets a reviewed threshold, not a retry.

## 18. Test data management

| Data | Rule |
|---|---|
| Example datasets | `examples/`, licensed CC BY 4.0, each with a README giving source, licence, provenance and intended use (A07). |
| Synthetic data | Preferred over committed files. Generators are seeded and live in code (proptest strategies, ECHOMETRIC stimulus generators). Large fixtures are generated at test time rather than committed. |
| Golden references | `tests/golden/`, under the policy in [section 5.3](#53-golden-file-policy). |
| Fuzz corpora | Small seed corpora are committed with the harness; grown corpora live in CI caches; crash reproducers become regression tests. |
| Chart corpora (C01, C02) | Only charts with redistributable licences, with ground truth committed alongside. |
| Personal and study data | Never committed. No personal data in fixtures. Study participant data follows the ethics approval and stays out of the repository. |
| Licensing | All test data carries SPDX licence information per [supply-chain-and-licensing](supply-chain-and-licensing.md). |

## 19. Test types by phase

| Phase | Test types introduced |
|---|---|
| A01 | CI skeleton on 3 OSes: fmt, clippy, nextest, doctests, rustdoc warnings, cargo-deny, MSRV build, wasm32 build of core crates, Markdown lint and internal link check, `cargo xtask check-deps`; CI time baseline |
| A02 | Spike measurements: loopback latency harness v0, cross-OS determinism check, Miri on the graph spike, IPC throughput benchmark, WebGL2 fps, manual screen-reader walkthrough |
| A03 | Unit and property tests on core and spec; Score IR snapshots (insta); cross-OS Score hash job; schema regeneration diff; spec parser fuzzing; doc coverage (`missing_docs`) |
| A04 | Transform property tests; importer fuzzing; M4 reference test; fingerprint stability across OSes; CSV load benchmark |
| A05 | Golden-audio harness and policy; block-size invariance; allocation check (`allocation-counter`); device and allocation soaks; Miri and shuttle on RT protocols; render benchmark; regression policy calibration; coverage tool |
| A06 | Design-rule conformance tests (SDR, NUMISONA); Erie import tests and fuzzing; compile benchmark; coverage gates for core, spec and compiler |
| A07 | CLI tests (assert_cmd + insta); Python tests on wheels; stubtest; notebook execution; quickstart test with 3 people (manual) |
| A08 | Replication gallery regression tests; stimulus generator determinism and statistics |
| B01 | Headless app-core integration tests; network-deny test; IPC contract tests; startup benchmark; frontend lint and typecheck |
| B02 | Vitest unit and browser-mode screenshots; Storybook with axe; Playwright with `mockIPC`; token contrast tests; manual NVDA and VoiceOver component checks |
| B03 | AUDAXIS A11Y conformance tests; scripted screen-reader task suite; interactive latency sign-off; speech/sonification sequencing tests |
| B04 | Frontend performance tests in all webviews; cursor-sync tests |
| B05 | Inspector ↔ spec round-trip properties; undo/redo coverage; project reopen bit-identity; project bundle fuzzing; WebdriverIO real-app E2E |
| B06 | Stream soak (1 h); announcement rate-limit tests; transport machine model tests |
| B07 | Full manual screen-reader matrix; Chartability audit; ACR v0 |
| C01 | Exact-extraction corpus tests; Office XML, SVG, PDF and chart-JSON importer fuzzing |
| C02 | Raster digitizer accuracy corpus; provenance and confidence labelling tests |
| C03 | 40-curve function suite; audification speed-up tests; expression parser fuzzing |
| C04 | MIDI export round trip; quantization-absent-in-Analysis test (SDR-10) |
| C05 | Detector precision and recall; lint rule positive and negative tests |
| C06 | Performance pass against all budgets; installer size check; signed-artifact release smoke tests; ACR v1 |

Commands for running each layer locally are in [dev-environment](dev-environment.md#common-commands).
