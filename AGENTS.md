# AGENTS.md

Instructions for AI coding agents (and a quick reference for humans) working in the SONARITHM repository. This file follows the [agents.md](https://agents.md/) format. Nested `AGENTS.md` files in crates or apps (added as they are created) override this one for their subtree: the closest file wins.

## Project in one paragraph

SONARITHM turns graphs, charts and numerical data into **structured, navigable sound**. A Rust workspace compiles a declarative **spec** (JSON) plus data (Apache Arrow) into a canonical **Score IR**. One render graph plays the Score in real time (cpal) and renders it offline to WAV/FLAC/MIDI with a provenance manifest. The same core powers a Tauri 2 + React 19 + React Aria desktop app, a CLI, Python bindings (PyO3) and, later, WebAssembly. Nine named **engines** (NUMISONA, AUDAXIS, SONOTRACE, PLOTSONA, CURVASONA, AXONOTE, ECHOMETRIC, TONALENS, RESONUM) are capability providers inside that one compiler. Blind and low-vision screen-reader users are a primary audience. Accessibility and evidence-backed sound design are architecture, not polish.

## Current state

- **Documentation baseline only (2026-10-18).** No Cargo workspace or frontend exists yet. Phase **A01** creates them. Check [docs/sdlc/status.md](docs/sdlc/status.md) before starting any work.
- Commands below marked *(after A01)* do not work until the workspace exists.

## Read before you change anything

1. [docs/sdlc/status.md](docs/sdlc/status.md): current phase and task states.
2. The phase in [docs/planning/roadmap.md](docs/planning/roadmap.md) and the task row in [docs/planning/implementation-plan.md](docs/planning/implementation-plan.md) (its **Read** column lists the docs you need).
3. The relevant document in [docs/architecture/](docs/architecture/README.md) and any ADR it cites in [docs/decisions/](docs/decisions/README.md).
4. [docs/research/design-rules.md](docs/research/design-rules.md) if you touch any sound default, mapping, navigation or announcement behaviour.

## Repository map

| Path | What lives there |
|---|---|
| `crates/` | Rust library crates (`sonarithm-core`, `-spec`, `-data`, `-compiler`, `-dsp`, `-render`, `-audio`, `-export`, `-stream`, `-speech`, `-app`, facade `sonarithm`) *(after A01)* |
| `crates/engines/` | One crate per engine (`sonarithm-numisona`, …) |
| `apps/cli/`, `apps/desktop/` | CLI binary `sonarithm`; Tauri host (`src-tauri/`) + React UI (`src/`) |
| `bindings/python/`, `bindings/wasm/` | PyO3 package; WASM build (Stage D) |
| `packages/` | Frontend workspace packages: `@sonarithm/ui`, `@sonarithm/viz`, `@sonarithm/ipc` |
| `schemas/` | **Generated** JSON Schemas; never edit by hand |
| `tests/golden/` | Golden Score snapshots and reference audio |
| `xtask/` | `cargo xtask` automation (codegen, check-deps, golden updates) |
| `docs/` | All documentation ([map](docs/README.md)) |

Full layout and dependency rules: [docs/architecture/02-repository-and-crate-layout.md](docs/architecture/02-repository-and-crate-layout.md).

## Commands *(after A01)*

| Purpose | Command |
|---|---|
| Everything CI runs, locally | `cargo xtask ci` |
| Build / test | `cargo build --workspace` · `cargo nextest run --workspace` |
| Lint and format | `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` |
| Licences, advisories, bans | `cargo deny check` |
| Architecture dependency rules | `cargo xtask check-deps` |
| Regenerate schemas and TS types | `cargo xtask codegen` |
| Review snapshot changes | `cargo insta review` |
| Update golden audio (needs a reason in the PR) | `cargo xtask golden --update` |
| WASM build check of core crates | `cargo xtask wasm-check` |
| Frontend | `pnpm install` · `pnpm -r lint` · `pnpm -r test` · `pnpm -C apps/desktop tauri dev` |
| Python bindings | `maturin develop -m bindings/python/Cargo.toml` · `pytest bindings/python/tests` |

## Non-negotiable rules

1. **Spec in, Score out.** All sound comes from a spec compiled to a Score IR. Do not add code paths that synthesize sound from data without going through the compiler. The Score IR is the canonical artifact, and tests snapshot it ([ADR-0005](docs/decisions/0005-declarative-spec-and-score-ir.md)).
2. **The audio thread is sacred.** In the audio callback and everything it calls: no allocation or deallocation, no locks, no I/O or syscalls, no logging, no panics, no unbounded loops. Use `rtrb`, `triple_buffer` and `basedrop`; preallocate voices. Debug builds run under `allocation-counter` ([06-audio-engine](docs/architecture/06-audio-engine.md#12-real-time-rules-summary)).
3. **Determinism.** No `HashMap` iteration in outputs (use `BTreeMap` or sorted `Vec`). No wall-clock reads in compile or render paths. Every random source is seeded from the spec seed. f64 for time, f32 for samples ([ADR-0013](docs/decisions/0013-determinism-and-provenance.md)).
4. **Engines never depend on each other.** They communicate through `sonarithm-core` types only. `cargo xtask check-deps` enforces this and the other dependency rules. Only `sonarithm-audio` may depend on cpal; core crates must build for `wasm32-unknown-unknown`.
5. **Research-backed defaults carry rule IDs.** When you implement or change a default (pitch range, polarity, rate, ticks, reference tone, loudness), cite the rule (`SDR-xx` / `A11Y-xx`) in a doc comment and in the test name. **Never change a default without evidence.** Follow the change control in [design-rules.md](docs/research/design-rules.md). Rules marked `INFER` must stay flagged as experimental.
6. **Accessibility is part of done** for any UI change: keyboard-operable, correct accessible name, role and state, focus visible, no colour-only meaning, high-contrast theme works, screen-reader announcement checked. Use React Aria Components; do not hand-roll ARIA where RAC has a pattern ([08](docs/architecture/08-interaction-and-accessibility.md), [keyboard map](docs/design/keyboard-and-navigation.md)).
7. **Never lose provenance.** Data values carry a provenance class and confidence. Transforms propagate them, and announcements qualify estimates ([03 §4](docs/architecture/03-data-model.md#4-provenance-and-confidence)).
8. **Licences.** Do not add a dependency under GPL, AGPL, SSPL or BSL, or LGPL in default features. Known traps: `evalexpr` (AGPL since 12.0), `anomaly_detection` (GPL), `mp3lame-encoder` (LGPL, feature only), `oxisynth` (LGPL), nih-plug VST3 (GPLv3), ChartGemma (GPL), Moondream 3 (BSL), WebPlotDigitizer code (AGPL: re-implement from docs, never copy). Run `cargo deny check` ([supply chain](docs/engineering/supply-chain-and-licensing.md)).
9. **Privacy.** No telemetry and no network calls without an explicit consent gate. No absolute user paths, personal data or secrets in logs, manifests, projects or test fixtures ([ADR-0012](docs/decisions/0012-privacy-and-telemetry.md)).
10. **Generated files are generated.** Do not edit `schemas/**` or `packages/ipc/src/generated/**` by hand. Change the Rust types and run `cargo xtask codegen`.
11. **No silent contract changes.** Changes to the spec language, Score IR snapshot format, IPC protocol, project file format or plugin WIT need an RFC or ADR ([docs/rfcs](docs/rfcs/README.md)).

## Code style (summary)

- **Rust:** edition 2024; core MSRV 1.88; `thiserror` in libraries, `miette` diagnostics with codes `SNR-<AREA>-<NNN>`; no `unwrap`/`expect` outside tests; `unsafe` only in allow-listed crates with `// SAFETY:` comments; public items documented; units as newtypes (`Hz`, `Cents`, `Db`, `Seconds`, `Frames`).
- **TypeScript/React:** strict TS; React Aria Components for widgets; Zustand for UI state, XState for transport and export machines; Rust is authoritative for session, transport and cursor state; hot paths (playhead, meters) bypass React via `requestAnimationFrame`; CSS Modules + design tokens only (no hard-coded colours or spacing); every string externalized.
- **UI look:** follow [design-language.md](docs/design/design-language.md). Its anti-patterns (gradients, glassmorphism, emoji icons, oversized rounded cards, default shadcn/Tailwind look, decorative motion) are review blockers.
- Details: [docs/engineering/coding-standards.md](docs/engineering/coding-standards.md).

## Tests you must add

| Change | Required tests |
|---|---|
| Spec or compiler logic | Unit tests + Score snapshot (insta) + property tests for invariants |
| A research default | A conformance test named after the rule (`sdr_17_dynamic_reference_tone`) |
| DSP or render | Golden-audio test (tolerance) and, for RT code, a check under `allocation-counter` |
| Parser or importer | Unit tests + a fuzz target |
| UI component | Storybook story + axe check + keyboard test; a manual screen-reader note in the PR |
| Performance-sensitive code | A criterion benchmark against its PB budget |

Strategy and tolerances: [docs/engineering/testing-strategy.md](docs/engineering/testing-strategy.md).

## Commits and pull requests

- **Conventional Commits** with scopes from crate areas or engine IDs: `feat(numisona): add ERB pitch space`, `fix(render): crossfade on seek`, `docs(adr): 0014 …`.
- **DCO sign-off** on every commit: `git commit -s`.
- Small PRs (≈ ≤ 400 changed lines, excluding generated files and snapshots), one task per PR, title prefixed with the task ID in the body (`Implements A03-T06`).
- Fill in the PR template's Definition of Done checklist ([definition-of-ready-and-done.md](docs/sdlc/definition-of-ready-and-done.md)).
- Never force-push to `main`, disable or skip CI checks, bypass hooks, or merge your own PR without the required review.

## Documentation duties (same PR as the change)

| When you… | Update |
|---|---|
| Start, finish or block a task | [docs/sdlc/status.md](docs/sdlc/status.md) |
| Finish a working session | A worklog entry in [docs/sdlc/worklog/](docs/sdlc/worklog/README.md) |
| Measure something, or something surprises you | A finding in [docs/research-paper/findings-log.md](docs/research-paper/findings-log.md) |
| Touch a performance budget | [docs/research-paper/benchmarks-log.md](docs/research-paper/benchmarks-log.md) |
| Change a requirement's design, test or status | [docs/sdlc/traceability-matrix.md](docs/sdlc/traceability-matrix.md) |
| Make a significant decision | A new ADR in [docs/decisions/](docs/decisions/README.md) |
| Change behaviour described in an architecture doc | That architecture doc |
| Ship user-visible changes (before A07 automation) | [CHANGELOG.md](CHANGELOG.md) |

## When you are unsure

- Do not invent research numbers, citations, versions or deadlines. Mark gaps as `TODO(verify)` and say so in the PR.
- If a task needs a decision that is not documented, add it to [docs/planning/open-decisions.md](docs/planning/open-decisions.md) and stop that part of the work.
- If two documents disagree, an accepted ADR wins over an architecture doc, and an architecture doc wins over a plan. Report the conflict in the PR.

## Engine quick reference

| Engine | Crate | Role |
|---|---|---|
| NUMISONA | `sonarithm-numisona` | Data → auditory channels (pitch, duration, timbre, pan…), scales, polarity, templates |
| AUDAXIS | `sonarithm-audaxis` | Navigation tree, context tracks (ticks, reference tone, earcons), announcements, spatial |
| SONOTRACE | `sonarithm-sonotrace` | Transport, live streams, monitoring mode |
| PLOTSONA | `sonarithm-plotsona` (+ `-ml`) | Chart ingestion (exact-first) and chart-type templates |
| CURVASONA | `sonarithm-curvasona` | Math functions, critical-point cues, audification |
| AXONOTE | `sonarithm-axonote` | Musical quantization (presentation mode only), instruments, MIDI/OSC |
| ECHOMETRIC | `sonarithm-echometric` | Pattern detection, descriptions, perceptual lint, evaluation harness |
| TONALENS | `sonarithm-tonalens` | Multivariate lens, multi-stream separation, granular density |
| RESONUM | `sonarithm-resonum` | Model-based sonification (excitation-driven) |
