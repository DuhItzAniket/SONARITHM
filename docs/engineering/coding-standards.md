# Coding standards

> **Status:** Baseline v1 (2026-10-18). **Owner:** Architecture lead.
> **Related:** [01-overview](../architecture/01-overview.md) (principles P1–P10) · [02-repository-and-crate-layout](../architecture/02-repository-and-crate-layout.md) (crates, dependency rules, features, naming) · [testing-strategy](testing-strategy.md) · [git-workflow](git-workflow.md) · [tech-stack](tech-stack.md) · [design-rules](../research/design-rules.md) · [AGENTS.md](../../AGENTS.md)

These rules apply to every contributor, human or AI agent. Where a rule is enforced by a tool, the tool configuration is the source of truth and this document explains the intent. Where this document and an architecture document disagree, the architecture document wins and this file is corrected in the same PR.

Workspace configuration files named here (`Cargo.toml`, `clippy.toml`, `rustfmt.toml`, `.editorconfig`, `eslint.config.js`, `tsconfig.json`) are created in phase A01. Until then, the snippets below are the target.

## Contents

1. [Rust toolchain, edition and MSRV](#1-rust-toolchain-edition-and-msrv)
2. [Lints](#2-lints)
3. [Formatting and files](#3-formatting-and-files)
4. [Error handling and diagnostics](#4-error-handling-and-diagnostics)
5. [Unsafe code](#5-unsafe-code)
6. [API design](#6-api-design)
7. [Documentation](#7-documentation)
8. [Feature flags and dependencies](#8-feature-flags-and-dependencies)
9. [Logging and tracing](#9-logging-and-tracing)
10. [Determinism](#10-determinism)
11. [Real-time audio rules](#11-real-time-audio-rules)
12. [TypeScript and React](#12-typescript-and-react)
13. [Python bindings](#13-python-bindings)
14. [Commits and pull requests](#14-commits-and-pull-requests)

---

## 1. Rust toolchain, edition and MSRV

| Item | Rule |
|---|---|
| Edition | `edition = "2024"` for every crate, set once in `[workspace.package]` and inherited. |
| Resolver | `resolver = "3"` in `[workspace]` (MSRV-aware resolution; required explicitly in a virtual manifest). |
| Toolchain | `rust-toolchain.toml` pins a stable release for CI reproducibility. `rustup` installs it automatically on first `cargo` call. Nightly is used only for Miri and cargo-fuzz, never to build shipped artifacts. |
| Core MSRV | **1.88** for every crate unless the crate declares otherwise. Set as `rust-version = "1.88"` in `[workspace.package]` and inherited with `rust-version.workspace = true`. |
| Leaf-crate MSRV | A leaf crate may declare a higher `rust-version` when a dependency requires it (for example `sonarithm-desktop` at 1.90 for Tauri, `sonarithm-plugin-host` at 1.96 for wasmtime; see [02 §2](../architecture/02-repository-and-crate-layout.md#2-crate-catalogue)). A crate with a higher MSRV must never become a dependency of a crate with a lower one. |
| CI | The MSRV job builds each crate with its declared `rust-version`; the main jobs use the pinned stable. See [ci-cd](ci-cd.md). |

**Raising the core MSRV** needs a PR that states the reason (the dependency or language feature), updates the MSRV column in [02 §2](../architecture/02-repository-and-crate-layout.md#2-crate-catalogue) and [01 §8](../architecture/01-overview.md#8-technology-summary) through the normal change process, and carries a changelog entry. The core MSRV must stay well behind latest stable; at baseline it trails by 11 releases ([04-rust-core-stack §12](../research/notes/04-rust-core-stack.md#12-quality-tooling-nextest-insta-proptest-criterion-vs-divan-cargo-fuzz-deny-audit-vet-semver-checks-machete-miri-loom-rust-2024-edition-and-msrv-norms)).

## 2. Lints

### 2.1 Workspace lint table

Every crate inherits the workspace lints with `[lints] workspace = true`. Cargo does not allow a crate to combine `workspace = true` with its own entries, so crate-specific adjustments are made with inner attributes in `lib.rs` or `main.rs` (an attribute overrides a `deny` from the command line but not a `forbid`).

```toml
# Cargo.toml (workspace root), target for A01
[workspace.lints.rust]
unsafe_code = "deny"                  # backstop; crates on the unsafe register opt in locally (section 5)
unsafe_op_in_unsafe_fn = "deny"       # warn-by-default in edition 2024; we deny
unreachable_pub = "warn"              # keep the public surface intentional

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
# Noisy pedantic lints are allowed here one by one, each with a comment giving the reason.
# The initial allow list is set in A01 and reviewed at every phase review.

# Restriction lints we opt into
unwrap_used = "deny"
expect_used = "deny"
undocumented_unsafe_blocks = "deny"   # every `unsafe {}` needs a `// SAFETY:` comment
allow_attributes_without_reason = "warn"
panic = "warn"
dbg_macro = "warn"
todo = "warn"
unimplemented = "warn"
print_stdout = "warn"
print_stderr = "warn"
```

```toml
# clippy.toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-dbg-in-tests = true
allow-print-in-tests = true
```

Clippy reads the MSRV from each crate's `rust-version`, so `clippy.toml` does not repeat it.

### 2.2 How lint levels are used

- CI runs `cargo clippy --workspace --all-targets -- -D warnings`, so every `warn` blocks a merge. `warn` exists so that local work-in-progress builds are not interrupted; `deny` marks things that are never acceptable in committed library code.
- `unwrap_used` and `expect_used` are denied outside tests (`#[cfg(test)]`, `#[test]` and integration tests). In library code, return a `Result`. Binaries may use `expect` for start-up invariants with a local `#[allow(clippy::expect_used, reason = "...")]`.
- Every `#[allow]` or `#[expect]` carries `reason = "..."`. Prefer `#[expect(...)]`, which fails when the lint no longer fires.
- Lints that are noisy only in one domain are allowed at crate level with a reason. Example: DSP crates allow `clippy::cast_precision_loss` for `usize` → `f32` conversions of buffer indices.
- `missing_docs`: published library crates declare `#![deny(missing_docs)]` in `lib.rs`. It is mandatory for `sonarithm-core` and `sonarithm-spec` from A03 ([roadmap](../planning/roadmap.md#a03--core-types-spec-v01-and-score-ir)) and for every other library crate before its first publication. Apps, `xtask` and test-only crates are exempt.
- Pedantic includes `missing_errors_doc` and `missing_panics_doc`. Keep them on for library crates: public fallible functions document their errors.

## 3. Formatting and files

| Item | Rule |
|---|---|
| Rust | `rustfmt` with stable options only (the toolchain is stable): `style_edition = "2024"`, `newline_style = "Unix"`, defaults otherwise. CI runs `cargo fmt --all --check`. |
| TypeScript, CSS, JSON, YAML, Markdown | Prettier (section 12.2). |
| Encoding and line endings | UTF-8, LF, final newline. `.editorconfig` sets indentation (4 spaces for Rust and Python, 2 for TS, JSON, YAML, CSS). `.gitattributes` sets `* text=auto eol=lf`. Windows contributors should set `core.autocrlf=false`. |
| File headers | SPDX licence identifiers per the REUSE policy in [supply-chain-and-licensing](supply-chain-and-licensing.md). No author names or dates in source files; Git holds history. |
| Module layout | One concept per module. Prefer `foo.rs` + `foo/` over `foo/mod.rs` for new modules. Modules are snake_case nouns ([02 §5](../architecture/02-repository-and-crate-layout.md#5-naming-conventions)). |
| Generated code | Lives in clearly named files or directories (`schemas/`, `packages/ipc/src/generated/`), carries a "generated, do not edit" header, and is regenerated by `cargo xtask` commands. CI fails if regeneration produces a diff. |

## 4. Error handling and diagnostics

### 4.1 Rules

| Context | Rule |
|---|---|
| Library crates | Errors are typed enums derived with `thiserror`, named `<Crate>Error` (for example `SpecError`, `AudioError`), marked `#[non_exhaustive]`. Variants carry structured fields, not preformatted strings. |
| User-facing diagnostics | Problems the user can fix (invalid spec, unreadable column, lint findings) are reported as `Diagnostic` values with a stable code, severity, message, optional rule IDs and location ([05 §7](../architecture/05-compiler-and-score-ir.md#7-diagnostics)). They are rendered with `miette` at the edges (CLI, Python, desktop). Which library crates depend on `miette` directly is settled in A03; `sonarithm-core` stays free of rendering concerns. |
| Applications | `anyhow` is allowed only in `apps/*` and `xtask`. Library crates never return `anyhow::Error` or `Box<dyn Error>`. |
| Panics | Library code does not panic on any input. No `unwrap`, `expect`, `panic!`, `todo!` or `unimplemented!` outside tests. `unreachable!` is allowed only for an invariant proven in the same function, with a comment stating the proof. Indexing that could go out of bounds is replaced by iterators, `get`, or a length check at the function boundary. Arithmetic on untrusted sizes (row counts, frame counts) uses `checked_*` or `saturating_*`. |
| Debug assertions | `debug_assert!` is encouraged for internal invariants. It must not be the only guard against invalid external input. |
| Error messages | State the cause and the fix, carry units, no exclamation marks ([design-language](../design/design-language.md)). Never include absolute user paths or data values beyond what is needed to locate the problem (P8). |

### 4.2 Diagnostic codes

Format: `SNR-<AREA>-<NNN>`, for example `SNR-SPEC-012`. Lint diagnostics also carry the design-rule ID they enforce: `SDR-19 / SNR-LINT-003` ([02 §5](../architecture/02-repository-and-crate-layout.md#5-naming-conventions)).

- A code, once released, is never reused or renumbered. A retired code stays in the registry marked `RETIRED`.
- Each code has a registry entry (code, area, one-line meaning, help text, rule IDs). The registry lives next to the code that emits it; the docs site renders it as reference. The registry mechanism is built in A03.
- Initial areas (extend through a PR that updates this table):

| Area | Owner crate | Area | Owner crate |
|---|---|---|---|
| `SPEC` | `sonarithm-spec` | `RENDER` | `sonarithm-render` |
| `DATA` | `sonarithm-data` | `AUDIO` | `sonarithm-audio` |
| `COMP` | `sonarithm-compiler` | `EXPORT` | `sonarithm-export` |
| `LINT` | ECHOMETRIC lint | `APP` | `sonarithm-app` |
| `ENG` | engine crates (message names the engine) | `CLI`, `PY` | `sonarithm-cli`, `sonarithm-py` |

## 5. Unsafe code

`unsafe` is **forbidden by default**. Every crate not listed in the register below starts `lib.rs` (or `main.rs`) with `#![forbid(unsafe_code)]`. The workspace `unsafe_code = "deny"` is the backstop.

**Unsafe register.** A crate may contain `unsafe` only within the scope listed here. Adding a crate or widening a scope needs an ADR.

| Crate | Permitted scope | Verification |
|---|---|---|
| `sonarithm-render` | Real-time primitives not covered by `rtrb`, `triple_buffer`, `basedrop` or `atomic_float` | Miri, plus shuttle or loom for concurrent code |
| `sonarithm-dsp` | SIMD kernels behind safe wrappers, each with a scalar reference implementation | Miri on the scalar path and on wrappers; equivalence property tests |
| `sonarithm-audio` | Platform FFI not covered by cpal (for example a future WASAPI exclusive-mode path) | Integration tests on each OS; Miri does not cover FFI |
| `sonarithm-py` | FFI at the Arrow C Data Interface boundary where the safe API cannot express the transfer | pytest round-trip tests; Miri does not cover FFI |

`sonarithm-core`, `sonarithm-spec`, `sonarithm-data`, `sonarithm-compiler` and all engine crates never contain `unsafe`.

Rules inside the permitted scope:

1. Keep `unsafe` in a small private module that exposes a safe API. The module's doc comment states the invariants it relies on.
2. Every `unsafe` block has a `// SAFETY:` comment that explains why each precondition holds at that point (enforced by `clippy::undocumented_unsafe_blocks`). Every `unsafe fn` has a `# Safety` rustdoc section.
3. One unsafe operation per block where practical (`clippy::multiple_unsafe_ops_per_block` may be enabled per crate).
4. Pure-Rust unsafe code is covered by a Miri test (`cargo +nightly miri test -p <crate>`) that runs in CI on PRs touching the crate (a `miri` job joins `ci.yml` when the first registered unsafe code lands). Lock-free code also gets a shuttle or loom model test ([testing-strategy §8](testing-strategy.md#8-concurrency-and-unsafe-code)).
5. PRs that add or change `unsafe` need a second reviewer listed for that path in CODEOWNERS.

## 6. API design

Follow the Rust API Guidelines. Project-specific rules:

| Topic | Rule |
|---|---|
| Units | Physical and musical quantities are newtypes in `sonarithm-core`: `Hz`, `Cents`, `Db`, `Seconds`, `Frames` ([02 §2](../architecture/02-repository-and-crate-layout.md#2-crate-catalogue)). Never pass a bare `f64` meaning seconds or decibels across a public API. Conversions are explicit methods (`Seconds::to_frames(rate)`), and each conversion documents its rounding rule. Seconds → frames rounds half to even ([05 §4.1](../architecture/05-compiler-and-score-ir.md#41-representation-choices)); no ad-hoc `as` casts. |
| Numeric types | Score-level time and pitch are `f64`; audio samples are `f32`; absolute sample positions are integer frames (section 10). |
| Construction | Types with more than three optional fields get a builder (`ScoreTrack::builder()`), with validation in `build()` returning a `Result`. Public structs that may grow fields are `#[non_exhaustive]` and constructed through builders or constructors. |
| Spec-facing types | Enums and structs that mirror the spec (marks, channels, scales, modes) are `#[non_exhaustive]`, derive `serde` and `schemars`, and follow the spec's naming (Enge et al. vocabulary, SDR-04). Spec compatibility follows [04-sonification-spec §7](../architecture/04-sonification-spec.md#7-versioning-and-compatibility). |
| IDs | Strongly typed IDs (`TrackId`, `EngineId`, `RowRef`), never raw `usize` or `String` across crate boundaries. |
| Traits | Engine capabilities are traits in `sonarithm-core`. Volatile third-party crates (cpal, Firewheel, Tauri, ML runtimes) sit behind project traits (P10). Third-party types never appear in the public API of `sonarithm-core`, `sonarithm-spec` or the facade. |
| Ownership | Accept borrowed inputs (`&str`, `&[T]`, `impl AsRef<Path>`); return owned outputs. Avoid `Rc`/`RefCell` in library APIs. |
| Async | The compiler, engines, render and DSP crates are synchronous and wasm-compatible ([02 §3](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced)). Async (`tokio`) appears only where 02 permits it. |
| Naming | Crates, engine IDs, modules and types follow [02 §5](../architecture/02-repository-and-crate-layout.md#5-naming-conventions). |
| Stability | Published crates run `cargo-semver-checks` in CI ([ci-cd](ci-cd.md)). Until v1.0 all library crates share one workspace version. |

## 7. Documentation

- Every public item has a rustdoc comment: one summary sentence, then details, `# Errors`, `# Panics` (should say "never" for library code), `# Safety` where applicable, and `# Examples` for entry points.
- Rustdoc examples compile and run. CI runs `cargo test --doc` separately, because `cargo nextest run` does not execute doctests. Examples use `?` with a `fn main() -> Result<…>` wrapper rather than `unwrap`.
- CI builds docs with `RUSTDOCFLAGS="-D warnings"`, which rejects broken intra-doc links.
- **Cite design rules.** Where a default value or behaviour implements a rule from the [design-rules catalogue](../research/design-rules.md), the doc comment names the rule ID, so a reader can trace the default to its evidence:

  ```rust
  /// Default pitch range for single-series values.
  ///
  /// Default per SDR-08 (400–1400 Hz, expandable to 65–1480 Hz).
  pub const DEFAULT_PITCH_RANGE: HzRange = HzRange::new(Hz(400.0), Hz(1400.0));
  ```

  The same ID appears in the conformance test name ([testing-strategy §6](testing-strategy.md#6-design-rule-conformance-tests)) and in the [traceability matrix](../sdlc/traceability-matrix.md).
- Each crate has a `README.md` used as the crate-level doc (`#![doc = include_str!("../README.md")]`) with purpose, feature flags and one example.
- Architecture-level explanation belongs in `docs/architecture/`, not in rustdoc. Link to it.

## 8. Feature flags and dependencies

| Rule | Detail |
|---|---|
| Additive only | A feature adds capability. It never removes API or changes the output of an existing code path. The same spec and data produce the same Score whatever features are enabled. |
| Licence-clean defaults | Default features are permissively licensed. Copyleft code (for example LGPL MP3 encoding) is only reachable through non-default features (P9, [02 §4](../architecture/02-repository-and-crate-layout.md#4-feature-flags-planned)). |
| Naming | Short lowercase names matching the capability: `asio`, `jack`, `polars`, `xlsx`, `mp3`. The facade's `full` enables everything non-copyleft. |
| Heavy dependencies | ML runtimes, wasmtime, MQTT, SQL and C-library codecs live in leaf crates behind features ([02 §3 rule 5](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced)). |
| CI | Each library crate builds with `--no-default-features` and with `--all-features` (excluding features that need SDKs unavailable in CI, such as `asio`). |
| Dependency declarations | Versions live in `[workspace.dependencies]`; crates use `dep.workspace = true`. New dependencies need a justification in the PR (maintenance, licence, MSRV) and must pass `cargo deny check` ([supply-chain-and-licensing](supply-chain-and-licensing.md)). `cargo machete` keeps unused dependencies out. |
| Dependency direction | The rules in [02 §3](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced) are enforced by `cargo xtask check-deps` (A01). |

## 9. Logging and tracing

- Library crates instrument with `tracing` (spans for pipeline stages, events for notable conditions). They never install a subscriber and never print. Subscribers are set up by apps (CLI, desktop) and by the Python module's init.
- **Never call `tracing` macros on the audio thread.** RT code pushes fixed-size records (xrun, fault code, overflow count) into an `rtrb` ring, and a non-RT thread drains it and emits `tracing` events (section 11).
- Levels: `error` for failures the user must see; `warn` for degraded behaviour; `info` for lifecycle (project opened, render finished); `debug` for decisions (which defaults applied); `trace` for per-item detail. Hot loops log at `trace` or not at all.
- **Privacy (P8).** Logs never contain dataset values or column contents at `info` or above, never absolute user paths outside `debug`, and never anything sent over the network. Telemetry rules are in [ADR-0012](../decisions/0012-privacy-and-telemetry.md).
- Use structured fields (`tracing::info!(track = %id, events = n, "encoded")`) rather than formatted strings.

## 10. Determinism

SDR-01 requires that spec plus data fully determine every render. The Score IR must be byte-identical across OS and CPU; audio must match within tolerance ([01 §7](../architecture/01-overview.md#7-quality-attributes-and-budgets), [ADR-0013](../decisions/0013-determinism-and-provenance.md), [05 §4.2](../architecture/05-compiler-and-score-ir.md#42-invariants-property-tested)). These rules apply to every compile, analysis, plan and render path, real time and offline.

| Source of variance | Rule |
|---|---|
| Collection order | Never iterate a `HashMap` or `HashSet` to produce output (Score, snapshots, manifests, diagnostics order, files). Use `BTreeMap`/`BTreeSet`, `IndexMap`/`IndexSet`, or sort explicitly. Hash maps are fine for lookups. |
| Hashing | Never use `std::collections::hash_map::DefaultHasher` or `RandomState` for anything persisted or compared: its algorithm is unspecified and may change between Rust releases. Fingerprints and spec hashes use SHA-256 over a canonical form; seed derivation (`seed ⊕ hash(track_id)`, [05 §3](../architecture/05-compiler-and-score-ir.md#3-engine-registry-and-capability-traits)) uses a fixed, documented hash function. |
| Randomness | All randomness is seeded from the spec seed and declared (SDR-02). Use a seeded, portable generator (for example `rand_chacha`), one stream per track via `CompileContext::rng(track_id)`. Never `thread_rng`, `rand::random` or OS entropy in compile or render paths. |
| Time | No wall clock (`SystemTime::now`, `Instant::now`) in compile or render paths. Render time comes from the frame counter. If a manifest records a creation time, that field is excluded from comparisons. |
| Numeric types | `f64` for time and Score-level values; `f32` for audio samples; integer frames (`u64`) for absolute sample positions. Never accumulate time by repeated float addition; compute `t = start + i × step` or count frames. |
| Float ordering | Sort floats with `f64::total_cmp`. Ties are broken by a stable key (insertion order, track ID). Use stable sorts. |
| Transcendental math | `std` float functions such as `powf`, `exp`, `ln` and `sin` have platform-dependent precision. Values that end up in the Score IR are computed with a platform-independent implementation (a pure-Rust libm behind a `sonarithm-core` math module; choice confirmed in A03 and ADR-0013). |
| Parallelism | Results never depend on thread count or scheduling. Parallel stages partition work deterministically and reduce in a fixed order. The offline renderer is single-threaded per Score ([06 §8](../architecture/06-audio-engine.md#8-determinism)). |
| CPU features | No `-C target-cpu=native` in distributed or golden-test builds. SIMD paths have scalar equivalents tested against them. |
| Denormals | FTZ/DAZ is set identically in the real-time callback and the offline driver (section 11.4), so both paths compute the same samples. |
| Environment | Locale, timezone, current directory and environment variables never influence compile or render output. Paths in manifests are relative or redacted ([06 §7](../architecture/06-audio-engine.md#7-export)). |

## 11. Real-time audio rules

The audio thread is sacred (P3). These rules apply to everything reachable from the backend callback: the render graph `process()`, voice processing, scheduler dispatch, command handling, telemetry writes and every function they call. The same `process()` drives offline export (P4), so these rules also keep exports deterministic. Architecture: [06-audio-engine](../architecture/06-audio-engine.md), [ADR-0004](../decisions/0004-audio-backend-and-render-graph.md).

### 11.1 Never on the audio thread

| Forbidden | Includes | Instead |
|---|---|---|
| Allocation and deallocation | `Box::new`, `Vec::push` beyond capacity, `collect`, `to_vec`, `clone` of heap types, `String`, `format!`, `Arc::new`, dropping the last `Arc`/`Box`/`Vec` | Preallocate at plan time; reuse buffers; retire heap objects through `basedrop` so the free happens on the collector thread |
| Locks | `Mutex`, `RwLock`, `Condvar`, `try_lock`, `OnceLock` initialisation, anything that can block or invert priority | Wait-free queues (`rtrb`), `triple_buffer`, atomics (`atomic_float`) |
| Blocking channels | `std::sync::mpsc`, crossbeam channels, `tokio` channels | `rtrb` SPSC rings with fixed capacity |
| System calls and I/O | File, network, `println!`, thread spawn, `sleep`, `Instant::now`, `SystemTime::now`, environment reads | Use the timestamps the backend passes to the callback; move I/O to non-RT threads |
| Logging | `tracing` macros, `log`, `eprintln!` | Fixed-size records into an `rtrb` ring drained off-thread (section 9) |
| Panics | `unwrap`, `expect`, unchecked indexing, overflow on frame arithmetic, division by a value not validated at plan time | Iterators (`zip`, `chunks_exact`), `get` with a silent fallback, validation at plan-build time |
| Unbounded work | Loops over the Score, retries, searches without a bound, draining a queue without a cap | Every loop is bounded by block size, voice-pool size or a fixed per-block cap (for example at most `MAX_COMMANDS_PER_BLOCK` commands drained per block) |
| Unknown code | Plugin calls (wasmtime), user scripts (rhai), expression evaluation (exmex), `dyn` callbacks supplied from outside the render crate | Plugins and scripts run at compile time only (Stage D rule); expressions are compiled into plan data before playback |
| Late planning | FFT planning, resampler construction, fundsp graph construction | Plan off-thread; call fundsp `allocate()` before handing a node to the RT thread; use the `*_with_scratch` and preallocated-output APIs of rustfft, realfft and rubato |

### 11.2 Patterns

| Need | Pattern |
|---|---|
| Control → audio commands | `rtrb::Producer<Command>` where `Command` is `Copy` and fixed-size (play, pause, seek in frames, set gain, trigger cue, stop all). Capacity and overflow policy per [06 §4](../architecture/06-audio-engine.md#4-real-time-communication); the control thread coalesces on overflow and never blocks the audio thread. |
| New plan or buffer | Build the `RenderPlan` on a worker thread, wrap it in `basedrop::Shared`, send it through an `rtrb` ring. The audio thread swaps it in at a block boundary (with crossfade). The old plan's handle is dropped on the audio thread, and `basedrop` defers the free to the collector thread. |
| Audio → UI state | `triple_buffer` for latest-value state (sample position, host time of the block, playing flag, xrun count, meters). The UI needs only the latest value. |
| Discrete audio-thread events | An `rtrb` ring of fixed-size records (xrun, marker crossed, fault code). |
| Continuous parameters | `atomic_float` values written by the control thread, read once per block, then smoothed (section 11.3). |
| Live data | Ingest and encoding run on `tokio` tasks and worker threads; events reach the audio thread through a timestamped `rtrb` ring with look-ahead. The audio thread never parses data. |
| Voices | A voice pool preallocated at plan time from the Score's maximum polyphony plus headroom ([05 §5](../architecture/05-compiler-and-score-ir.md#5-from-score-to-renderplan)). Voice stealing follows a deterministic rule. |
| Block size | Scratch buffers are sized for the largest block the backend can deliver. If a callback delivers more frames than planned, process in sub-blocks; never resize on the audio thread. |
| Thread priority | Real-time promotion through cpal's `realtime` feature ([04-rust-core-stack §3](../research/notes/04-rust-core-stack.md#3-real-time-safety-rtrb-ringbuf-crossbeam-triple_buffer-basedrop-assert_no_alloc-no_denormals-audio_thread_priority-lock-free-uiaudio-patterns)). |
| Faults | RT code is panic-free by construction. As a last resort, the callback wrapper catches an unwind, outputs silence and raises a fatal flag; the control thread tears the stream down ([06 §11](../architecture/06-audio-engine.md#11-failure-handling)). This is not a control-flow mechanism. |

### 11.3 Parameter smoothing and envelopes

- Every parameter that reaches the audio thread from outside (gain, pan, speed, brightness) is smoothed per sample or per block with a one-pole or linear ramp. The smoothing time is a named constant with its rationale in the doc comment.
- Note onsets and offsets always have ramps; zero-length envelopes are forbidden (SDR-22).
- Seeks, loop wraps and plan swaps crossfade rather than cut ([06 §5](../architecture/06-audio-engine.md#5-scheduler-and-transport)).
- No full-scale onsets: master gain start-up and limiter behaviour follow [06 §10](../architecture/06-audio-engine.md#10-hearing-safety-and-levels).

### 11.4 Denormals and numeric safety

- Flush denormals (FTZ/DAZ) for the duration of each callback, using `no_denormals` or an equivalent guard. The offline driver sets the same mode (section 10).
- Filters and feedback paths that can decay towards zero are tested with silent input for long durations to catch denormal slowdowns and NaN propagation.
- A NaN or infinity reaching the output is a bug. Debug builds assert on non-finite output samples; release builds clamp at the limiter and raise a fault record.

### 11.5 Enforcement

1. **`assert_no_alloc` in debug builds.** The callback body runs inside an `assert_no_alloc` scope. The `AllocDisabler` global allocator is installed in the test and app binaries (a library cannot set the global allocator). CI runs a debug-build soak that fails on any allocation in the callback ([testing-strategy §9](testing-strategy.md#9-performance-and-soak-tests)).
2. **Marking.** Functions callable from the audio thread carry a `# Real-time safety` rustdoc section saying "RT-safe". Code that is only RT-safe under conditions (for example "after `prepare()`") says so. Modules that contain only RT code are named `rt` or documented as RT-only.
3. **No RT-only output branches.** There is no `if realtime { … }` that changes rendered samples. Real-time and offline differ only in the driver (P4).
4. **Soak and xrun tracking.** The device soak on reference hardware ([testing-strategy §9.3](testing-strategy.md#93-soak-tests)) must report zero xruns at the budgeted buffer size.

### 11.6 Reviewing RT code

A PR that touches `sonarithm-render`, `sonarithm-dsp`, `sonarithm-audio` or any function marked RT-safe gets this review:

- [ ] The RT call graph from `process()` is identified, and every function in it is marked RT-safe.
- [ ] No hidden allocation: `collect`, `to_vec`, `clone`, `format!`, `?` converting into an allocating error type, or a `Drop` impl that frees memory.
- [ ] No locks, blocking channels, syscalls, logging or wall-clock reads (section 11.1).
- [ ] Every loop has a stated bound.
- [ ] No panicking operations; indices and divisors are validated at plan-build time.
- [ ] External parameters are smoothed; envelopes have non-zero ramps (SDR-22).
- [ ] The `assert_no_alloc` CI run exercises the changed path (add a test case if it does not).
- [ ] Golden audio is unchanged, or the change is intentional and justified ([testing-strategy §5](testing-strategy.md#5-golden-audio-tests)).
- [ ] Render benchmarks are within the regression threshold ([testing-strategy §9](testing-strategy.md#9-performance-and-soak-tests)).
- [ ] For changes to the scheduler, voice pool, plan swap or backend: a local soak result at 128-frame buffers is attached to the PR.
- [ ] Any `unsafe` is within the register (section 5) and has Miri or model-test coverage.

## 12. TypeScript and React

Scope: `apps/desktop/src` and `packages/*` ([02 §1](../architecture/02-repository-and-crate-layout.md#1-repository-tree-target)). Architecture: [09-desktop-app](../architecture/09-desktop-app.md), [08-interaction-and-accessibility](../architecture/08-interaction-and-accessibility.md).

### 12.1 TypeScript

- `tsconfig` base: `"strict": true`, `"noUncheckedIndexedAccess": true`, `"exactOptionalPropertyTypes": true`, `"noImplicitOverride": true`, `"noFallthroughCasesInSwitch": true`, `"verbatimModuleSyntax": true`, `"isolatedModules": true`. Packages extend the base and do not loosen it.
- No `any`; use `unknown` and narrow. No non-null assertions (`!`) except in tests. No `@ts-ignore`; `@ts-expect-error` only with a reason.
- IPC types come from `@sonarithm/ipc`, generated by ts-rs from Rust ([01 §8](../architecture/01-overview.md#8-technology-summary)). Never hand-write a type that mirrors a Rust type, and never edit generated files.
- Components call the typed client in `@sonarithm/ipc`, never `invoke("command_name")` directly. The client is the thin, shell-agnostic bridge that keeps an Electron fallback a port rather than a rewrite ([ADR-0002](../decisions/0002-desktop-shell-tauri.md)).

### 12.2 Linting and formatting

**Choice: ESLint (flat config) with typescript-eslint (type-checked rule sets), eslint-plugin-jsx-a11y (strict), eslint-plugin-react-hooks, plus Prettier for formatting.**

Reason: accessibility is a deciding criterion for this project, and we want the complete jsx-a11y rule set together with typescript-eslint's type-aware rules (floating promises, unsafe `any` flows). Biome is faster and needs less configuration, but it reimplements a subset of these rules and has narrower type-aware coverage. Revisit if Biome reaches parity on both; record the decision in [tech-stack](tech-stack.md).

- CI runs `eslint --max-warnings 0` and `prettier --check`.
- Enable jsx-a11y `control-has-associated-label` in addition to the strict preset.
- Stylelint checks CSS Modules (section 12.6).

### 12.3 React and React Aria

- React 19 function components only. The React Compiler is enabled ([05-desktop-frontend-stack §10](../research/notes/05-desktop-frontend-stack.md#10-synthesis-1-recommended-stack-for-sonarithm)); do not add `useMemo`/`useCallback` by reflex.
- **React Aria Components (RAC) is the single primitive layer.** Where RAC has a component for a pattern (Button, ToggleButton, Slider, NumberField, Tree, Table, GridList, Toolbar, Menu, Dialog, ComboBox, Tabs, drag and drop), use it. Never hand-roll `role`, `aria-*` attributes, focus management or keyboard handling for a pattern RAC covers. Do not mix in Radix or Base UI.
- Application code imports primitives from `@sonarithm/ui`, which wraps RAC with tokens and project defaults. Importing RAC directly in `apps/desktop` needs a reason in review.
- Custom widgets are allowed only where RAC has no equivalent (for example the plot keyboard explorer). They follow the WAI-ARIA APG pattern, are built with React Aria hooks, are specified in [keyboard-and-navigation](../design/keyboard-and-navigation.md), and need a manual screen-reader check before merge.
- Every interactive control has an accessible name. **No icon-only button without an accessible name** (`aria-label` or visually hidden text) and a tooltip; icons themselves are `aria-hidden`.
- Sliders and numeric readouts announce value, unit and range (`formatOptions` with the unit style).
- Status messages go through the managed announcer ([08 §4](../architecture/08-interaction-and-accessibility.md#4-announcement-pipeline)), never through ad-hoc `aria-live` elements. Announcements must not steal focus (A11Y-17).
- Keyboard shortcuts are registered in the central keymap, never in component-level `keydown` handlers. Single-key shortcuts are chart-scoped or remappable and avoid screen-reader modifier combinations (A11Y-06).

### 12.4 State

| State | Owner | Rule |
|---|---|---|
| Audio, transport, session, project, undo/redo, profiles and settings | **Rust** (`sonarithm-app`) | Authoritative. The UI renders the last state Rust reported and never assumes a transition succeeded before the authoritative state arrives ([06 §5](../architecture/06-audio-engine.md#5-scheduler-and-transport)). Undo/redo lives in the Rust command bus. Settings persist through Rust profiles, not `localStorage`. |
| Transport, render/export and spec-validation flows | **XState v5** machines | Mirror the Rust state machine so impossible UI states cannot be represented. Machines are pure and unit-tested. |
| UI-only state (panel layout, selection, focus region, open dialogs) | **Zustand** stores | One store per concern. No server data cached in Zustand beyond the latest snapshot Rust sent. |
| Component-local state | `useState`/`useReducer` | Only when nothing else needs it. |

### 12.5 Hot paths bypass React

- The playhead is drawn in a `requestAnimationFrame` loop that extrapolates from the clock anchors Rust sends over a Channel ([06 §9](../architecture/06-audio-engine.md#9-latency-and-clock-synchronization)), writing to a canvas or a CSS transform through refs. No React state update per frame.
- Meters and waveform or spectrogram views draw to canvas or WebGL2 from binary Channel frames or `ipc::Response` buffers. No JSON events on hot paths.
- Hot-path code reads Zustand through `store.subscribe` (transient subscriptions), not through hooks.
- Downsampling (M4, LTTB) happens in Rust; the UI never receives more points than it can draw.

### 12.6 Styling

- CSS Modules with cascade layers, driven by the token layer in `@sonarithm/ui` ([design-language](../design/design-language.md)). No CSS-in-JS runtime, no Tailwind defaults.
- **No hard-coded colours, spacing, radii, font sizes or durations** in component CSS or inline styles. Use semantic tokens (`var(--surface-1)`, `var(--space-2)`). Only the token generator contains raw values. Stylelint enforces this (`color-no-hex`, `color-named: never`, and a strict-value rule for spacing properties) from B02.
- Every component works in dark, light and high-contrast themes, under `forced-colors: active` and `prefers-contrast: more`. Canvas layers redraw with system colours under forced colours.
- Motion is for meaning only (playhead, meters, short state transitions) and honours `prefers-reduced-motion`.
- Use logical properties (`margin-inline-start`) so RTL layouts work.

### 12.7 Strings and internationalisation

- No user-facing string literals in components. Strings come from Fluent message catalogues with the same message IDs as the Rust side ([08 §9](../architecture/08-interaction-and-accessibility.md#9-localization-of-announcements)).
- Never build sentences by concatenation; use message arguments and plural forms. Announcement text is composed from structured parts so verbosity and order stay configurable (A11Y-12).
- Numbers, units and dates use locale-aware formatting. RAC components receive the locale through `I18nProvider`.
- Copy follows the design language: terse, unit-bearing, sentence case, no exclamation marks.

## 13. Python bindings

Scope: `bindings/python` (crate `sonarithm-py`, Python package `sonarithm`, native module `sonarithm._core`; [02 §2](../architecture/02-repository-and-crate-layout.md#2-crate-catalogue)).

| Topic | Rule |
|---|---|
| Layering | The Rust crate is thin: it converts types and calls the facade. No business logic in the binding ([02 §3 rule 6](../architecture/02-repository-and-crate-layout.md#3-dependency-rules-enforced)). PyO3 types never leak into other crates. The public API is re-exported from a pure-Python `sonarithm/__init__.py`; `_core` is private. |
| Wheels | abi3 wheels built with maturin. The minimum CPython version is decided in A07 ([release-process](release-process.md)); code and docs assume Python 3.10 or newer until then. |
| Typing | The package ships `py.typed` and a `.pyi` stub for `_core` covering every public symbol. CI checks stubs against the built module with `python -m mypy.stubtest sonarithm` and type-checks the pure-Python layer with mypy in strict mode. |
| Data interop | Accept pandas, Polars and pyarrow inputs through the Arrow C Data Interface (zero-copy where possible) and NumPy arrays through rust-numpy. Return audio as `numpy.ndarray` of `float32` with shape `(frames, channels)`. Do not require pandas or Polars at import time. |
| Naming and signatures | PEP 8 names. Required arguments positional; optional arguments keyword-only. Units in parameter names where ambiguous (`duration_s`, `gain_db`). |
| Docstrings | NumPy docstring style for every public function and class, including `Parameters`, `Returns`, `Raises` and `Examples`. Examples run as doctests in CI. Cite design-rule IDs where a default implements one, as in Rust docs. |
| Errors | A base `sonarithm.SonarithmError` with subclasses per area (`SpecError`, `DataError`, `RenderError`). Exceptions carry the diagnostic `code` (`SNR-…`), rule IDs and location as attributes. |
| Concurrency | Release the GIL (detach from the interpreter) around compile and render calls. |
| Side effects | No printing. Diagnostics are returned or raised; logs go through Python's `logging`. No network access (P8). |
| Notebook display | Rendered results implement IPython rich display for audio playback (A07). |
| Tooling | ruff for lint and format, mypy for types, pytest for tests ([testing-strategy §11](testing-strategy.md#11-cli-and-python-tests)); confirm in [tech-stack](tech-stack.md). |

## 14. Commits and pull requests

Conventional Commits PR titles with crate or engine scopes (`feat(numisona): add mel scale`), DCO sign-off, squash merges and branch naming are defined in [git-workflow](git-workflow.md). The checklist a PR must satisfy is the task-level [Definition of Done](../sdlc/definition-of-ready-and-done.md). Contribution process: [CONTRIBUTING.md](../../CONTRIBUTING.md).
