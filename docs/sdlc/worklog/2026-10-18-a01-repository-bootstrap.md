# 2026-10-18: A01 repository bootstrap (workspace, tooling, CI skeleton)

> **Date(s):** 2026-10-18. **Phase:** A01. **Author:** written by an AI agent (Claude Code, a session started from the kickoff session); human review pending (@DuhItzAniket).
> **Related:** [worklog conventions](README.md) · [status](../status.md) · [roadmap A01](../../planning/roadmap.md#a01--repository-bootstrap-and-governance) · [implementation plan §2](../../planning/implementation-plan.md)

## Context and phase

- Phase: A01 — [roadmap entry](../../planning/roadmap.md#a01--repository-bootstrap-and-governance)
- Issue(s) / PR(s): none yet; nothing is committed. The first PR will run the new workflows.
- Starting point: the documentation baseline was on `main`; no Cargo workspace, frontend workspace or workflows existed. The licence is undecided (ADR-0010 proposed, OD-01 open).

## Goals

- Tasks A01-T03 to A01-T09 and A01-T12 from the [implementation plan](../../planning/implementation-plan.md): Cargo workspace, toolchain and lint configuration, `deny.toml`, `xtask`, the GitHub workflows and Dependabot, the pnpm workspace skeleton and docs CI.
- Out of scope: licence files and `license` fields (T01, T13), the ruleset (T02), the project board (T10) and repository settings (T11).

## What was done

| Area | Files | Result on the Windows development machine |
|---|---|---|
| Cargo workspace (T03) | `Cargo.toml`, `Cargo.lock`, `crates/sonarithm{,-core,-spec,-data,-compiler,-dsp,-render,-audio,-export}`, `apps/cli` (binary `sonarithm`), `xtask` | `cargo build --workspace` and `cargo test --workspace` pass |
| Toolchain and lints (T04) | `rust-toolchain.toml` (1.99.0, rustfmt, clippy, wasm32 target), `rustfmt.toml`, `clippy.toml`, `[workspace.lints]` exactly as coding-standards §2.1 | `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo hack check --rust-version --workspace --all-targets --locked` passes on Rust 1.88 for all 11 crates |
| Supply chain (T05) | `deny.toml` | `cargo deny check` passes (cargo-deny 0.20.2); a scratch crate depending on a `GPL-3.0-only` crate fails `licenses` |
| xtask (T06) | `.cargo/config.toml` (alias), `xtask/src/{main,ci,check_deps,codegen}.rs` | `cargo xtask check-deps` passes; 17 unit tests, including an on-disk fixture workspace where engine→engine and core→cpal dependencies fail |
| CI (T07, T08, T12) | `.github/workflows/{ci,pr-title,dco,scorecard,codeql,docs}.yml`, `.github/dependabot.yml`, `.markdownlint-cli2.jsonc` | Not run yet. Every action is pinned to a full commit SHA looked up with `gh api` on 2026-10-18; top-level `permissions: {}`; least privilege per job. markdownlint-cli2 0.23.2 (the version the action bundles) and lychee 0.24.2 (`--offline --include-fragments`) pass locally on all Markdown |
| Frontend workspace (T09) | `package.json`, `pnpm-workspace.yaml`, `pnpm-lock.yaml`, `eslint.config.js`, `prettier.config.js`, `tsconfig.base.json`, `packages/{ui,viz,ipc}` | `pnpm install` and `pnpm -r lint` pass; `pnpm -r typecheck` passes. A throwaway probe file confirmed that type-aware and jsx-a11y rules fire |

`cargo xtask` tasks: `ci` (fmt, clippy, tests via nextest when installed, doctests, rustdoc with `-D warnings`, wasm32 build, cargo-deny, dependency rules; all steps run and failures are summarised), `check-deps`, `codegen` (stub), `wasm-check` (listed in AGENTS.md; the crate list lives once in `xtask/src/ci.rs` and CI calls it).

## Decisions

| Decision | Why | Record |
|---|---|---|
| Pin Rust **1.99.0** in `rust-toolchain.toml` | Current stable, matching [tech-stack](../../engineering/tech-stack.md) "1.99.x (CI)" | No ADR needed: local choice |
| `publish = false` for every crate, inherited from `[workspace.package]`; `[licenses.private] ignore = true` in `deny.toml` | Nothing may be published before ADR-0010 is accepted, and cargo-deny would otherwise fail the unlicensed workspace crates. Remove both when T13 adds the licence | No ADR needed; revisit in T13 |
| `check-deps` checks the 02 §3 rules plus the layering graph; dev-dependencies are exempt; heavy crates (wasmtime, tract-onnx, ort, rumqttc, polars, mp3lame-encoder) are restricted to their leaf crates, optional where 02 §4 makes them features | Dev-dependencies never ship and do not affect the wasm32 build (and `proptest` pulls `tempfile`, which core must not reach in shipped code). An unknown workspace crate fails, so a new crate forces an update of 02 §3 and the rule table | No ADR needed: implements 02 §3 |
| `#![deny(missing_docs)]` in every library crate from A01 | Empty crates comply for free; coding-standards §2.2 requires it before publication anyway | No ADR needed |
| MSRV job uses **cargo-hack** `--rust-version` | Checks each crate against its own `rust-version`, so leaf crates with higher MSRVs (desktop 1.90, plugin host 1.96) work without workflow changes; ci-cd.md left the tool to A01 | No ADR needed: tool choice delegated to A01 |
| cargo-deny runs through `taiki-e/install-action` instead of `EmbarkStudios/cargo-deny-action` | Same binary and toolchain as `cargo xtask ci`; avoids the action's container and its own Rust toolchain. ci-cd.md shows the action only as an illustration | No ADR needed |
| DCO is a workflow (`dco.yml`), not the DCO GitHub App | No third-party app install; ci-cd.md allows either | No ADR needed |
| **Dependabot**, not Renovate, with grouped weekly updates and a 7-day cooldown | Recommendation in [supply chain](../../engineering/supply-chain-and-licensing.md#dependency-updates-dependabot-or-renovate) | No ADR needed |
| ESLint **9.39** (not 10), TypeScript **6.0** (not 7), typescript-eslint 8.71, jsx-a11y 6.10.2, react-hooks 7.1.1, Prettier 3.9, pnpm **12.14.0** | eslint-plugin-jsx-a11y's peer range ends at ESLint 9; typescript-eslint supports TypeScript < 6.1. Versions sit in a pnpm catalog | Record in tech-stack.md (not edited by this session) |
| Node `engines`: `>=24.11.0`; CI uses Node 24 (Active LTS) | 24.11.0 opened the 24 LTS line; the range also admits the Node 25 development machine (dev-environment.md). Move CI to Node 26 once it enters LTS | No ADR needed |
| Add a minimal `frontend` job (ubuntu: install, lint) to `ci.yml` now | A01-T09 requires `pnpm -r lint` in CI; ci-cd.md schedules the full `frontend` job for B01 | Conflict reported below |
| markdownlint: MD013 and MD060 off (house style), MD024 siblings only, MD041 off; MD022, MD032, MD033, MD036 and two research notes disabled as **baseline exceptions** | The docs CI must be green on the baseline, and this session may not edit other docs. Each exception is marked in `.markdownlint-cli2.jsonc` for removal after the cleanup listed below | No ADR needed |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| The pinned toolchain install failed with rename errors, then left a toolchain without `cargo.exe` | Two concurrent rustup auto-installs of the same toolchain (a `cargo build` and a status command) raced in `~/.rustup/downloads` | `rustup toolchain uninstall 1.99.0`, then a single `rustup toolchain install` from the repository root. Suggest a troubleshooting entry in dev-environment.md: run `rustup toolchain install` once before the first build |
| `pnpm install` first failed to resolve the self-installed pnpm 12.14.0 | pnpm switching to the `packageManager` version (local 12.11.2); appeared transient | Second run succeeded |
| clippy `collapsible_if` and `unnecessary_wraps` in xtask | New lints in 1.99 | Let chains (stable since 1.88, so within MSRV); `#[expect(..., reason)]` on the codegen stub |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Workspace build from clean (debug) | 37 s | Windows 11, Rust 1.99.0, 11 crates (xtask dependencies dominate) | pending (CI baseline goes to the benchmarks log after the first PR) |
| `cargo nextest run --workspace` | 0.39 s, 18 tests | Same machine | pending |
| `cargo install --locked cargo-deny` | 4 min | Same machine | not a project budget |
| CI wall time | not measured | Needs the first PR (A01-T07 target ≤ 15 min) | [benchmarks log](../../research-paper/benchmarks-log.md), after the run |

## Inconsistencies found (not fixed here)

This session was limited to the files above, so these are reported for the owner or the kickoff session:

1. **AGENTS.md "Current state"** still says no Cargo workspace or frontend exists, and its commands are marked *(after A01)*.
2. **[02 §1](../../architecture/02-repository-and-crate-layout.md#1-repository-tree-target)** does not list `.cargo/config.toml` (needed for the `cargo xtask` alias), `.markdownlint-cli2.jsonc`, `eslint.config.js`, `prettier.config.js` or `tsconfig.base.json`. It also lists `.github/` contents loosely; `label-sync.yml` (A01-T08) is not created yet.
3. **[ci-cd.md](../../engineering/ci-cd.md)**: the `frontend` job is scheduled for B01, but A01-T09 requires `pnpm -r lint` in CI (a minimal job now exists). The illustrative `deny` job uses cargo-deny-action; the real one uses install-action. The `msrv` tool is now cargo-hack. The `wasm32` job calls `cargo xtask wasm-check`.
4. **[coding-standards §7](../../engineering/coding-standards.md#7-documentation)** expects a crate `README.md` used as the crate doc; the A01 crates use `//!` comments until they have content.
5. **[tech-stack](../../engineering/tech-stack.md)** should record ESLint 9 (blocked on jsx-a11y for 10), TypeScript 6.0 (typescript-eslint range), pnpm 12.14 and cargo-hack.
6. **Markdown defects in the baseline** (behind the markdownlint exceptions):
   - Template placeholders written as bare `<name>` are swallowed by GitHub as unknown HTML tags, so readers do not see them: `docs/research-paper/experiments/EXP-000-template.md`, `docs/sdlc/reviews/_phase-review-template.md`, `docs/sdlc/reviews/_gate-review-template.md`, `docs/sdlc/worklog/_template.md`, `docs/sdlc/retrospectives/_template.md`, `docs/rfcs/0000-template.md`, and `docs/design/keyboard-and-navigation.md` lines 372–377. Wrap them in backticks.
   - Headings or paragraphs directly followed by a list, without a blank line (MD022/MD032): `docs/README.md`, ADRs 0001–0004, `docs/design/design-language.md`, `docs/architecture/05-compiler-and-score-ir.md`, `docs/architecture/08-interaction-and-accessibility.md`, `docs/product/requirements.md`, `docs/research-paper/papers/P01-icad-2027-toolkit/outline.md`.
   - Bold lines used as headings (MD036): `docs/design/design-language.md`, `docs/sdlc/definition-of-ready-and-done.md`, `docs/sdlc/reviews/_phase-review-template.md`.
   - `docs/research/notes/06-chart-extraction-ai.md` line 177: the source line directly under a table is rendered as a broken table row (needs a blank line). The same note and `01-tools-landscape.md` contain bare URLs.
7. **CodeQL Rust support level** was not re-verified (ci-cd.md asks to confirm it when enabling). TODO(verify) on the first run.
8. The Scorecard badge in README.md (A01 target in supply-chain-and-licensing.md) is not added; README.md was outside this session's files.

## Next steps

- [ ] Owner: decide the licence (A01-T01), then add licence fields, files and REUSE (T13), remove `publish = false` handling as appropriate, and drop `[licenses.private]` from `deny.toml`.
- [ ] Owner: commit this work with DCO sign-off on a branch and open the first PR; confirm every workflow runs green on three OSes and record CI wall time in the benchmarks log (T07 acceptance, A01 research hook).
- [ ] Owner: configure the `main` ruleset with the required checks named in ci-cd.md (T02).
- [ ] Add `label-sync.yml` (rest of T08) and update the docs listed under "Inconsistencies", then re-enable the markdownlint baseline exceptions.
- [ ] After Node 26 enters LTS (expected late October 2026), move CI to it.

## Time spent (optional)

One working session.
