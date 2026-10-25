# Technology Stack

> **Status:** Baseline v1. Versions verified against crates.io, npm and GitHub on **2026-10-18** by the kickoff research ([notes/04](../research/notes/04-rust-core-stack.md), [notes/05](../research/notes/05-desktop-frontend-stack.md)).
> **Owner:** Architecture lead. Exact versions are pinned by `Cargo.lock` and `pnpm-lock.yaml` from A01. This page records the **intended baseline and the reasoning**. Update it in the PR that changes a major dependency.

## 1. Why this stack

| Requirement | Consequence |
|---|---|
| Real-time audio with no glitches, plus deterministic offline export | **Rust**: no GC pauses, memory safety, an own render graph ([ADR-0001](../decisions/0001-rust-workspace-core.md), [ADR-0004](../decisions/0004-audio-backend-and-render-graph.md)) |
| One core for desktop, CLI, Python and web | Rust workspace → PyO3 wheels, wasm-bindgen, Tauri in-process |
| Screen-reader-complete UI on three OSes | **System webview (Tauri) + React Aria**; native Rust GUIs are not accessible enough yet ([ADR-0002](../decisions/0002-desktop-shell-tauri.md), [ADR-0003](../decisions/0003-frontend-react-and-react-aria.md)) |
| Zero-copy data exchange with the Python research ecosystem | **Apache Arrow** internally ([ADR-0006](../decisions/0006-arrow-data-model.md)) |
| Clean licensing for wide adoption | Permissive dependencies only in default features; cargo-deny allowlist ([supply chain](supply-chain-and-licensing.md)) |

**Ring legend** (ThoughtWorks-style): **Adopt** = default choice; **Trial** = use behind an abstraction and validate; **Assess** = investigate before using; **Avoid** = do not use (reason given).

## 2. Rust core

| Area | Choice | Version (2026-10-18) | Ring | Licence | Introduced | Notes / alternatives |
|---|---|---|---|---|---|---|
| Toolchain | Rust stable, edition 2024 | 1.99.x (CI); core MSRV 1.88 | Adopt | — | A01 | `rust-toolchain.toml` pins CI. Leaf crates declare higher MSRVs |
| Audio I/O | **cpal** | 0.18.2 | Adopt | Apache-2.0 | A05 | Native PipeWire/PulseAudio hosts, xrun reporting. No exclusive mode (#106). 0.19 will break the API, so it sits behind `AudioBackend` |
| Render graph | **Own graph** (`sonarithm-render`) | — | Adopt | — | A05 | Firewheel 0.14 **Trial** in A02 behind the same trait. kira 0.12 / rodio 0.23 Assess (CLI preview only) |
| Synthesis | **fundsp** | 0.23 | Adopt | MIT/Apache | A05 | Seeded noise, PolyBLEP, FIR resampling. Custom granular/wavetable/modal nodes |
| Resampling / FFT | rubato; rustfft; realfft | 5.0.1; 6.4.1; 3.5.0 | Adopt | MIT | A05/C03 | |
| Loudness | ebur128 | 0.1.10 | Adopt | MIT | A05 | Metering and golden tolerance (ΔLUFS) |
| Spatial | sofar (SOFA HRTF) | 0.4.0 | Trial | MIT/Apache | D03 | hrtf 0.9 Assess; audionimbus (Steam Audio) Assess |
| RT communication | rtrb; triple_buffer; basedrop; atomic_float; no_denormals; allocation-counter (debug and CI; replaces assert_no_alloc, F-038) | 0.4.0; 9.0.0; 0.1.3; 1.1; 0.3; 0.8.1 | Adopt (basedrop Trial) | MIT/Apache (triple_buffer MPL-2.0) | A05 | crossbeam-channel only off the RT path |
| Decode | symphonia | 0.6.1 | Adopt | MPL-2.0 | C01 | Audio input for samples and audification sources |
| Export | hound (WAV); flacenc (FLAC); midly (SMF) | 3.5.1; 0.5.1; 0.5.3 | Adopt (flacenc Trial) | Apache/MIT | A05/C04 | MP3 via mp3lame-encoder is **LGPL**, optional feature only |
| Live MIDI / OSC | midir; rosc | 0.11.1; 0.11.4 | Adopt | MIT/Apache | C04 | |
| Speech | tts | 0.26.3 | Trial | MIT | B03 | Last release Jul 2024; repo active. Self-voicing only, behind `sonarithm_speech::Speaker` and the opt-in `system` feature (B03-T07); on Linux it binds speech-dispatcher (needs libspeechd-dev) |
| Data model | arrow + parquet; bytes | 60; 1.12 | Adopt | Apache-2.0; MIT | A04 | The split crates (`arrow-array`, `-data`, `-schema`, `-select`, `-ipc`, `-ord`, `-row`) instead of the `arrow` umbrella; parquet without its async (tokio) default and with pure-Rust codecs. zstd (C) is the default-on `zstd` feature of `sonarithm-data`, off in the wasm32 build |
| File formats | csv; encoding_rs + chardetng; calamine; jiff (+ chrono interop) | 1.4; 0.8.42 + 1.0.0; 0.37.0; 0.2.38 | Adopt | MIT/Unlicense; MIT/Apache (+ BSD-3-Clause data tables) | A04 | encoding_rs and chardetng (both by H. Sivonen) decode and guess non-UTF-8 CSV. calamine sits behind the default `xlsx` feature. jiff without default features: civil date-time parsing only, no time-zone database and no clock |
| DataFrame adapter | polars | 0.55.2 | Trial (feature) | MIT | A04 | Rust API breaks without deprecation |
| Streaming | tokio; tokio-tungstenite; serialport; rumqttc | 1.53; 0.30; 4.10.1; 0.25.1 | Adopt (rumqttc Trial) | MIT/Apache (serialport MPL-2.0) | B06 | |
| Expressions | exmex (behind a facade) | 0.21 | Trial | MIT/Apache | C03 | Symbolic derivatives. Single maintainer, hence the facade |
| Scripting | rhai | 1.26.1 | Adopt | MIT/Apache | A04 | Sandboxed spec expressions (`filter`, `derive`): a raw engine without default features (no runtime RNG, no time package), user functions, modules, closures and custom syntax compiled out, fuel-limited per row. Its dependency `smartstring` is unmaintained (RUSTSEC-2026-0249, ignored in `deny.toml`, tracked in #115) |
| Analytics | statrs; ndarray; augurs | 0.19.1; 0.17.2; 0.10.2 | Adopt / Trial (augurs) | MIT/Apache | C05 | linfa 0.8, changepoint Assess |
| Chart ML | tract-onnx (default); ort (opt-in) | 0.23.9; 2.0.0-rc.14 | Trial | MIT/Apache | D04 | ort is still RC; use `load-dynamic` |
| Python | pyo3 + arrow-pyarrow + maturin | 0.29.3; 60.0.0; 1.15 | Adopt | MIT/Apache; Apache-2.0; MIT/Apache | A07 | abi3 wheels (CPython 3.10+); Arrow PyCapsule interop through arrow-pyarrow, which fixes PyO3 at 0.29 (F-101); numpy is not needed |
| WebAssembly | wasm-bindgen (+ cli) | 0.2.129 | Adopt | MIT/Apache | A01 (build check), D05 | wasm-pack 0.15 Assess (maintainer churn) |
| C ABI | cbindgen | 0.29.4 | Assess | MPL-2.0 (tool) | post-1.0 | uniffi 0.32 for mobile later |
| Plugins | wasmtime (+ WIT, WASI 0.3) | 49 | Trial | Apache-2.0 | D05 | MSRV 1.96, leaf crate only. extism Assess |
| Serialization / schema | serde; serde_json; schemars | 1.0.229; —; 1.2.3 | Adopt | MIT/Apache | A03 | |
| TS codegen | ts-rs | 12.0.1 | Adopt | MIT | B01 | specta/tauri-specta 2.0.0-rc.25 Trial ([ADR-0011](../decisions/0011-typed-ipc-ts-rs.md)) |
| Errors / diagnostics | thiserror; miette; anyhow (apps only) | 2.0.21; 7.6.0; — | Adopt | MIT/Apache | A03 | |
| Logging | tracing (+ tracing-subscriber, tracing-appender) | 0.1.44 | Adopt | MIT | A03 | Never on the audio thread |
| i18n | fluent-bundle; ICU4X (`icu_decimal`, `icu_datetime`); @fluent/bundle (UI) | 0.16.0; 2.3.0; 0.19.1 | Adopt | Apache/MIT; Unicode-3.0; Apache-2.0 | B02 | `sonarithm-i18n` and `@sonarithm/i18n` (B02-T08); ICU4X `sync` for shareable payloads |
| CLI | clap | — | Adopt | MIT/Apache | A07 | |

## 3. Desktop and frontend

| Area | Choice | Version | Ring | Licence | Introduced | Notes |
|---|---|---|---|---|---|---|
| Shell | **Tauri** | 2.12.x (2.12.3 latest patch) | Adopt (provisional) | MIT/Apache | B01 | Confirmed at the A02 gate. **Electron 44/45** is the fallback |
| Language / build | TypeScript (strict); Vite | — | Adopt | — | B01 | |
| Framework | **React** + React Compiler | 19.2 | Adopt | MIT | B01 | |
| Components | **React Aria Components** | RAC 1.17+ | Adopt | Apache-2.0 | B02 | Single primitive layer; Base UI only for trivial primitives if ever needed |
| App state | Zustand; XState | —; v5 | Adopt | MIT | B01 | Rust is authoritative for session and transport |
| Styling | CSS Modules + cascade layers + OKLCH token generator (`packages/ui/tokens`, B02-T01) | — | Adopt | — | B02 | Tailwind v4 only with a fully custom theme (not planned) |
| Plots | uPlot + custom WebGL2 layer | uPlot latest | Adopt | MIT | B04 | ECharts 6 Assess for breadth charts; WebGPU optional |
| Spec editor | CodeMirror 6 | — | Adopt | MIT | B05 | Monaco rejected as heavier |
| Layout | Fixed regions + splitters | — | Adopt | — | B02 | dockview's keyboard docking is Enterprise-only, hence fixed regions |
| Icons | Lucide | — | Adopt | ISC (+ Feather MIT) | B02 | Labelled, sparing |
| Fonts | IBM Plex Sans + IBM Plex Mono (WOFF2, bundled unmodified in `packages/ui/fonts`, 400/500/600) | Sans 3.005 (`@ibm/plex-sans` 1.1.0), Mono 2.005 (`@ibm/plex-mono` 2.5.0) | Adopt | OFL-1.1 | B02 | Alternative: Source Sans 3 + JetBrains Mono |
| Package manager | pnpm (workspaces) | 12.x | Adopt | MIT | A01 | |
| JS runtime (tooling and CI) | Node.js, an even-numbered **LTS** line | Node 24 LTS; move to Node 26 once it enters LTS (expected late Oct 2026; verify in A01) | Adopt | MIT | A01 | Odd lines (such as the Node 25 on the baseline dev machine) never become LTS. Range pinned in the `engines` field |
| Lint and format (frontend) | ESLint + typescript-eslint + jsx-a11y (strict) + react-hooks; Prettier; Stylelint 17.16 (no hard-coded colours or spacing; design-language anti-patterns) | Stylelint 17.16 | Adopt | MIT | A01/B02 | Biome rejected: weaker accessibility rule coverage ([coding-standards §12.2](coding-standards.md#122-linting-and-formatting)) |
| Python tooling | ruff (lint and format); mypy (stubs); pytest | — | Adopt | MIT | A07 | NumPy-style docstrings |

## 4. Quality, testing and supply chain

| Area | Choice | Version | Ring | Phase |
|---|---|---|---|---|
| Test runner | cargo-nextest | 0.9.148 | Adopt | A01 |
| Snapshots | insta | 1.49 | Adopt | A03 |
| Property tests | proptest | 1.12 | Adopt | A03 |
| Benchmarks | criterion (no default features: no plotting or rayon) | 0.8.2 | Adopt (divan Assess) | A04 (PB-05 data load), A05 |
| Fuzzing | cargo-fuzz (libfuzzer-sys 0.4 in each crate's `fuzz/` harness) | 0.13.2 | Adopt | A03 (spec), A04 (data importers) |
| Test fixtures | zip (dev-only, no default features: builds XLSX and ODS packages in the data tests; the version calamine uses) | 9.0 | Adopt | A04 |
| Concurrency / UB | Miri; shuttle | —; 0.9.7 | Adopt / Trial (loom Assess) | A05 |
| Coverage | cargo-llvm-cov (candidate) | — | Trial | A05 |
| Platform-independent maths | libm (pure-Rust) for every value that reaches the Score | — | Adopt | A03 |
| Licences / advisories / bans | cargo-deny | 0.20.2 | Adopt | A01 |
| Advisories | cargo-audit | 0.22.2 | Adopt | A01 |
| API compatibility | cargo-semver-checks | 0.51 | Adopt | A07 |
| Unused deps | cargo-machete | 0.9.2 | Adopt | A01 |
| Supply-chain review | cargo-vet | 0.10.2 | Assess | post-1.0 |
| SBOM | cargo-cyclonedx; cargo-auditable | — | Adopt | C06 |
| Release automation | release-plz (+ git-cliff) | — | Adopt | A07 |
| Frontend unit / visual | Vitest 4 (browser mode) | 4.x | Adopt | B02 |
| Component workbench + a11y | Storybook 10 + a11y addon (axe) | 10.x | Adopt | B02 |
| Cross-engine UI tests | Playwright (with Tauri IPC mocks) | — | Adopt | B02 |
| Real-app E2E | WebdriverIO `@wdio/tauri-service` | — | Trial | B05 |
| Security CI | OpenSSF Scorecard; CodeQL; Dependabot or Renovate | — | Adopt | A01 |

## 5. Avoid list

| Package | Reason | Instead |
|---|---|---|
| evalexpr ≥ 12.0.0 | **AGPL-3.0-only** since 2024-10-17 | exmex (facade) |
| fasteval, meval | Unmaintained (2020, 2018) | exmex |
| anomaly_detection | GPL-3.0 | augurs, statrs, own detectors |
| glicol, knyst, oddio | Stale (2024-04, 2024-11, 2023-10) | own graph + fundsp |
| synthizer-rs | Archived | — |
| dasp | No release since 2020-05 (usable, but not as a foundation) | fundsp |
| abi_stable, libloading plugins | Dormant; unsafe ABI | wasmtime components |
| nih-plug VST3 bindings | GPLv3 obligations | No VST3 hosting/exporting in the core |
| mp3lame-encoder (default) | LGPL-3.0 | Optional `mp3` feature only |
| oxisynth | LGPL-2.1 | rustysynth (MIT) |
| ChartGemma code; Moondream 3 weights | GPL-3.0; BSL-1.1 | Apache/MIT models (Qwen2.5-VL, Qwen3-VL, Florence-2, DePlot) |
| WebPlotDigitizer code | AGPL-3.0 frontend; closed AI Assist | Re-implement from public documentation |
| Slint (for the main UI) | GPLv3 or commercial; incomplete text accessibility | Tauri + React Aria |
| Tauri JSON events on hot paths | "Not designed for low latency or high throughput" | Channels + binary `ipc::Response` |

## 6. Review cadence

At every stage gate the architecture lead:

1. re-checks the maintenance status of every **Trial** and **Adopt** dependency (last release, open security advisories, maintainer changes);
2. re-evaluates **Assess** items against their trigger conditions;
3. updates this page and the [risk register](../planning/risk-register.md).

Major upgrades of cpal, Tauri, React or Arrow each get an issue with a migration checklist.
