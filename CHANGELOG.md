# Changelog

All notable changes to SONARITHM are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) with the pre-1.0 rules in the [release process](docs/engineering/release-process.md#versioning-policy). The spec version (`specVersion`) is versioned separately.

From phase A07, release-plz maintains this file: it generates each release's entries with git-cliff from Conventional Commit messages and proposes them in the release pull request. Edit wording in that pull request, not after the release. Until A07, entries are written by hand.

## [Unreleased]

### Added

- The navigation cursor (B03-T02): every movement verb of the keymap (step, jump by 10 %, first and last, in and out, lateral to another series, maximum and minimum, insights, landmarks, go to x, find value, Enter) and "where am I". Moves never wrap: at an edge the cursor stays and reports the boundary earcon; a verb that does not apply reports why.
- The navigation tree (B03-T01): the `sonarithm-audaxis` engine crate builds a NavTree (dataset, series, segments, points with their data rows, axes, insights and filters) from each compiled Score, and the compiler returns it beside the Score (`Compiled::nav`). The tree types live in `sonarithm_core::nav`.
- Message catalogues and locale formatting (B02-T08): the `sonarithm-i18n` crate with Fluent catalogues shared with the desktop UI, ICU4X number and date formatting that matches `Intl.NumberFormat`, spoken unit names (symbol, terse and verbose), and the `en-XA` pseudo-locale with a check for hard-coded, concatenated and truncated strings; the `@sonarithm/i18n` package and the `sonarithm/no-literal-ui-string` ESLint rule.
- Documentation baseline (2026-10-18), written before any code:
  - Research: the kickoff research report, seven research notes, and the design-rules catalogue (`SDR-##`, `A11Y-##`).
  - Product: the engine catalogue (nine engines on one platform).
  - Architecture: the overview, the repository and crate layout, and the component documents.
  - Decisions: initial architecture decision records in MADR 4 format, and the RFC process.
  - Planning: the roadmap (stages A to D, phases A01 to D06), implementation plan, risk register and open decisions.
  - SDLC: the process overview, live status, Definition of Ready and Done, and traceability matrix.
  - Engineering: tech stack, coding standards, testing strategy, development environment, git workflow, CI/CD, release process, and supply chain and licensing policy.
  - Design: the accessibility conformance plan and design documentation.
  - Research-paper track: the findings log and publication planning.
  - Community and governance: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant 3.0 by reference), `SECURITY.md`, `GOVERNANCE.md`, `CITATION.cff`, issue forms, pull request template, `CODEOWNERS` and the label taxonomy.
  - Agent instructions: `AGENTS.md` and `CLAUDE.md`.
  - Repository hygiene: `.gitignore`, `.gitattributes` and `.editorconfig`.
- Repository bootstrap (phase A01):
  - Cargo workspace with 11 skeleton crates, pinned toolchain, workspace lints, `rustfmt`, `clippy` and `cargo-deny` configuration, and `cargo xtask` automation (`ci`, `check-deps`, `codegen`, `wasm-check`).
  - pnpm workspace with placeholder frontend packages, ESLint and Prettier.
  - GitHub Actions workflows for CI, documentation checks, PR titles, DCO, CodeQL and OpenSSF Scorecard, plus Dependabot configuration.
- Licensing ([ADR-0010](docs/decisions/0010-licensing.md)): code under `MIT OR Apache-2.0`; documentation, example datasets and study stimuli under `CC-BY-4.0`. Adds `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSES/`, `REUSE.toml`, SPDX headers and a REUSE check in CI.
- Core types, spec v0.1 and Score IR (phase A03):
  - `sonarithm-core`: unit newtypes with platform-independent maths (`libm`), IDs and row references, provenance, diagnostics with a compile-time-checked code registry, the Score IR with invariant checks, a canonical SHA-256 hash and rounded review snapshots, and the engine capability traits with the registry and `CompileContext`.
  - `sonarithm-spec`: spec v0.1 as frozen by [RFC-0001](docs/rfcs/0001-spec-v0.1.md), with the generated JSON Schema in `schemas/`, validation with `SNR-SPEC` codes, JSON pointers and miette reports, RFC 8785 canonicalization, the spec hash, defaults reported as `Info`, and a migration framework.
  - `sonarithm-compiler`: the compile pipeline skeleton.
  - `cargo xtask codes` and the generated [error code registry](docs/reference/error-codes.md); fuzz targets for the spec parser and migrations, run nightly by `nightly.yml`.
- Data layer (phase A04, `sonarithm-data`): Arrow-backed tables and bundles with `snr.*` field metadata; CSV/TSV, JSON/NDJSON, Parquet, Arrow IPC and XLSX/ODS sources with encoding, dialect, header and type sniffing; field type, role and concept inference reported as diagnostics; provenance columns and their propagation; eight transforms with a sandboxed, fuel-limited rhai expression subset; M4 and LTTB downsampling with a tile cache; canonical data fingerprints; a PB-05 benchmark and fuzz targets.
- Render engine (phase A05):
  - `sonarithm-dsp` oscillators: sine, triangle, and band-limited (PolyBLEP) saw and square, computed with IEEE 754 arithmetic only so renders stay bit-identical across platforms.
  - `sonarithm-dsp` envelopes: ADSR with raised-cosine onset and offset ramps of at least 5 ms, about 40 ms for notes (SDR-22); retriggers and releases never step.
  - `sonarithm-dsp` filters (one-pole low-pass; RBJ low-, high- and band-pass), a constant-power panner and a linear parameter smoother (10 ms).
  - `sonarithm-render` plan builder: a Score becomes a `RenderPlan` in frames, with resolved frequencies, gains, pan gains, instruments and voice-pool sizes; unknown instruments raise `SNR-RENDER-001` and fall back to the mallet preset.
  - `sonarithm-render` render graph: preallocated voice pools, data, context, earcon and monitoring buses, a sample-accurate scheduler and a transport (play, pause, stop, seek, loop, speed and live cues) with crossfades instead of cuts. Earcon sounds are provisional until B03.
  - `sonarithm-render` real-time channels: a wait-free command ring that coalesces on overflow, plan swaps with a crossfade whose old graph is freed off the audio thread, and an engine-state triple buffer for telemetry.
  - Offline rendering and audio export: `OfflineRender` plays a plan through the same render path in fixed blocks, and `sonarithm-export` writes 24-bit PCM or 32-bit float WAV and 24-bit FLAC.
  - Hearing safety on the master: a look-ahead true-peak limiter at −1 dBTP, and a start-up ramp from −20 dB over 500 ms whenever a playback stream opens.
  - ISO 226:2023 equal-loudness compensation, on by default when a plan is built (SDR-24), and an EBU R128 loudness and true-peak meter for exports.
  - Exports write a render manifest (`*.render.json`, with a generated JSON Schema) and a WebVTT cue sidecar (`*.cues.vtt`) next to the audio, and normalize loudness with one static gain.
  - `sonarithm-audio`: the `AudioBackend` trait, a cpal backend, a timer-driven null device for tests, and a `Player` that recovers from device loss at the same transport position, paused until the user resumes.
  - A Windows WASAPI backend (ADR-0016): RAW shared mode by default, so OEM enhancement processing no longer adds latency and gain after the limiter, with exclusive mode as an opt-in; a latency measurement tool and per-device latency calibration.
  - Render benchmarks (PB-03) on a reference benchmark Score; faster voices and limiter. The mallet preset's brightness now tilts its partials instead of filtering them.
  - A golden-audio harness (`cargo xtask golden`, six cases in `tests/golden/`) and CI jobs that render on Windows, Linux and macOS and compare the results (PB-06).
  - An audio-thread allocation check in debug builds and CI (`no-alloc` job), xrun counting from the device to the engine state, and a `soak` program that plays the benchmark Score on a device, silently by default, and reports xruns, late callbacks and allocating callbacks.
- NUMISONA, the parameter-mapping engine (phase A06):
  - `sonarithm-numisona` scales: `linear`, `log`, `symlog`, `sqrt`, `pow`, `quantile`, `band` and `ordinal`, with `auto`, `zero-based` and explicit domains, `nice` rounding, and off-scale reporting for clamped values.
  - `sonarithm-numisona` pitch spaces and ranges: log-frequency (the default, SDR-09), linear frequency, mel, ERB-rate and Bark; the 400–1400 Hz default range and the 65–1480 Hz expanded range (SDR-08); ranges in hertz, MIDI notes or cents.
  - `sonarithm-numisona` psychophysical pitch scale, `f = f_ref · (d / d_ref)^(1/m)` (SDR-11), with exponents labelled as guidance.
  - `sonarithm-numisona` concept table (`data/concepts.toml`) with evidence on every row, polarity resolution (explicit, profile, concept, generic; SDR-12..14), the profile's one-key flip (SDR-15), and `SNR-ENG` diagnostics that explain every automatic choice.
  - `sonarithm-numisona` `tone` and `glide` encoders (the `Numisona` engine): every note keeps its row and confidence; track filters, time order and the three paces; redundant tap-count and duration encodings (A11Y-13); one timbre per category (SDR-25); missing values silenced or interpolated; warnings for loudness on data (SDR-23), pan as the only carrier (A11Y-14) and fast tones (SDR-19); synthetic example datasets.
  - `sonarithm-numisona` templates (SDR-03): `line-overview`, `bar-discrete`, `scatter-xy`, `histogram`, `box-five-number` and `heatmap-rowscan`, bound to any dataset with a compatible schema.
  - `sonarithm-numisona` `auto` mark (experimental, SDR-20): tones up to about 10 events per second, a glide above, reported as experimental every time.
  - Encoders receive the spec's substrate, field semantics, missing-value treatment and mode alongside the track (`TrackRequest`).
  - `sonarithm-numisona` Erie importer (subset, SDR-05): Erie specs become spec v0.1 documents, with a report of every construct that is approximated or has no equivalent.
  - `sonarithm-compiler` compiles the data layer's `DataBundle`: spec transforms run in `sonarithm-data`, and the `Platform` engine reads `inline` records and `file` sources relative to a base directory.
  - `sonarithm-numisona` rule-conformance suite: one named test for every design rule NUMISONA owns, checked against the design-rules catalogue; the Erie gallery subset renders; volatility advice for series with many trend reversals (SDR-21); the PB-04 compile benchmark (35 ms for 100k rows × 4 tracks on the development laptop).
- CLI, Python package and first release (phase A07):
  - `sonarithm` facade: `Sonarithm` with the default engines (the data platform and NUMISONA), `Spec` (from text, values or files; validation and migration), `compile` and `compile_with` for data the caller holds, the `quick` builder over the `line-overview` template, and `Compiled` with the Score, its diagnostics, a summary and a deterministic L1/L2 description; rustdoc examples run as tests.
  - `sonarithm-compiler`: `Platform::with_dataset` reads `dataset` sources from bundles the host provides (`SNR-COMP-026` for an unknown ID).
  - `sonarithm` CLI: `validate`, `inspect` (`--describe`, `--tree`), `lint` (a stub that lists the compiler's warnings until C05), `quick`, `schema` and `spec migrate`, with `--json`, `--plain` for screen readers, `--deny-warnings`, `--verbose`, specs from standard input, and exit codes 0 to 3.
  - Spec diagnostics that point at a missing top-level key name the location in their help instead of quoting the whole spec.
  - `sonarithm render` and `quick -o`, and the facade's `render_to_file` and `render_wav`: offline WAV or FLAC with the render manifest (canonical spec, data source, provenance mix) and the cue sidecar, at the spec's sample rate and loudness target; `--seed`, `--data`, `--sample-rate`, `--preserve-level`, `--no-cues`.
  - `sonarithm-compiler`: the compile result carries the data's provenance mix for render manifests.
  - Examples gallery (`examples/`): five synthetic datasets (DS-001 to DS-005, CC BY 4.0) with a spec each, a README and a notebook; CI regenerates the data and renders every spec.
  - Release automation: release-plz (one version for every crate, changelog from Conventional Commits), PyPI Trusted Publishing for the wheels, and CLI archives with an SBOM, third-party notices and build-provenance attestations.
  - Quickstart tutorial (`docs/tutorials/quickstart.md`): from install to a first FLAC in about five minutes, in Python or on the command line.
  - `sonarithm` Python package (abi3 wheels for CPython 3.10 and later): `quick`, `compile`, `Spec` as dicts or JSON, `dataset`, diagnostics as dicts, `render` to WAV or FLAC with the manifest, `wav()` and notebook `play()`, a text-first notebook display with an `<audio>` element, and typed stubs; data from pyarrow, pandas, polars, dicts, records, arrays or files, with the GIL released while compiling.
  - `sonarithm-data`: `read_batches` reads Arrow handed over in memory like a file, with one layout for text and without library metadata, so the same values give the same fingerprint from any library.
  - Citation metadata for Zenodo (`.zenodo.json`) alongside `CITATION.cff`, kept in step with the release version by `cargo xtask citation`.
  - `sonarithm play` and `sonarithm devices`: real-time playback with text-line announcements (space, arrows, Home, q), `--device`, `--speed` and a silent `null` backend; `quick` without `-o` plays.
- ECHOMETRIC research instrumentation (phase A08):
  - `sonarithm-echometric` stimulus generators, each deterministic from its parameters and a seed, with the published parameters documented next to SONARITHM's: GEN-001 trend reversals × density (Nees & Walker 2008), GEN-002 price series with click and reference-tone columns (Smith & Walker 2005), GEN-003 SNR-controlled transit light curves (Tucker Brown et al. 2022) and GEN-004 correlation triplets (Elmquist et al. 2024). Every stimulus carries its ground truth, a canonical CSV with a SHA-256 digest, a provenance manifest and notes on where it departs from the published design.
  - `sonarithm-echometric` study runner library: TOML protocol files (`sonarithm-study/0`) with conditions, blocks and generator parameters, checked with `SNR-ENG-600`–`607` diagnostics; a seeded trial order with counterbalancing; responses scored against each stimulus's ground truth (point estimation, trend identification, counting, choice); a JSONL log (`sonarithm-study-log/0`) with pseudonymous codes and no paths; pause, and resumption from the log in a new run.
  - `sonarithm study check` and `sonarithm study run`: check a protocol and its condition specs, then run a session with one plain line per prompt and typed answers (screen-reader friendly), replays, pauses, breaks, `--resume`, the silent `--audio null` output, and audio context recorded only as entered; with the pilot protocol `examples/studies/pilot-001`. `sonarithm-echometric` is now published with the CLI.
  - Replication gallery (`replication/`): Smith & Walker's context-cue conditions, Nees & Walker's density × reversals cells, Tucker Brown et al.'s transit curves and the Erie gallery subset, rendered from committed specs with their records; `cargo xtask replication` regenerates the stimuli and checks every render on three operating systems, and `--timings` measures them.
- SONOTRACE streams (phase B06, Lane R):
  - `sonarithm-data` `StreamTable`: bounded, append-only storage for live data with row sequence numbers, a row or time-window capacity with a hard row limit, cheap snapshots that later appends never change, and `SNR-DATA-071` when rows are dropped before a reader read them; provenance columns kept in canonical form.
  - `sonarithm-sonotrace`, the SONOTRACE engine: the spec's `stream` source reads the window of a connected stream; live encoding emits each new row's events exactly once and replays deterministically; scrub grains preview a track's value at the cursor; long series play in windows.

### Fixed

- `sonarithm-data`: `ReadOptions::without_inference()` keeps the field semantics declared in `fields` (type, unit, concept, role, label, description) instead of dropping them (#156).

[Unreleased]: https://github.com/DuhItzAniket/SONARITHM/commits/main
