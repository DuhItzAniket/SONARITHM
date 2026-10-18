# Project Status

> **Last updated:** 2026-10-18 · **Current stage:** A (A01 in progress) · **Next gate:** Gate A (alpha), target Feb 2027
> **Owner:** Project lead. **Update rule:** every PR that changes a task's or phase's state updates this file ([sdlc/README](README.md)).

## 1. Summary

| Item | State |
|---|---|
| Overall health | **Green**: A01 workspace, CI, licensing and GitHub settings in place; first CI run is on the A01 pull request |
| Current focus | Get the A01 PR green and merged; then A01-T10 (project board) and A02/A03. Owner decisions OD-02, OD-04, OD-10 |
| Latest release | none |
| Blocking decisions | none for A01 (OD-01 and OD-09 closed 2026-10-18) |
| Top risks | R-08 scope creep (20), R-09 bus factor (20), R-02 screen readers in the webview (15) |

## 2. Phase board

Status values: `Not started` · `In progress` · `In review` · `Done` · `Blocked` · `Deferred`.

| Stage | Phase | Name | Status | Progress | Notes |
|---|---|---|---|---|---|
| — | — | Kickoff: research and documentation baseline | **Done** | 100% | 2026-10-18; see [worklog](worklog/2026-10-18-project-kickoff.md) |
| A | A01 | Repository bootstrap and governance | **In progress** | ≈80% | T01, T02, T11 done; T03–T09, T12, T13 pass locally and await their first CI run on the A01 PR; T10 not started. See [A01 tasks](#a01-tasks) |
| A | A02 | Technical spikes and prototype gate | Not started | 0% | |
| A | A03 | Core types, Spec v0.1, Score IR | Not started | 0% | RFC-0001 to open |
| A | A04 | Data layer | Not started | 0% | |
| A | A05 | Render engine, audio backend, export | Not started | 0% | |
| A | A06 | NUMISONA v1 | Not started | 0% | |
| A | A07 | CLI, Python, v0.1.0-alpha | Not started | 0% | |
| A | A08 | Research instrumentation v0, ICAD 2027 | Not started | 0% | Ethics application filed by the research lane, RL-T02 (Dec 2026) |
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
| A01-T03 Cargo workspace skeleton | In progress | Local: 11 crates build and test, all `license = "MIT OR Apache-2.0"` |
| A01-T04 Toolchain, rustfmt, clippy, editorconfig | In progress | Local: fmt and clippy `-D warnings` clean. Rust 1.99.0 pinned, MSRV 1.88 |
| A01-T05 `deny.toml` | In progress | Local: `cargo deny check` passes; a GPL test crate is rejected |
| A01-T06 `xtask` (`ci`, `check-deps`, `codegen`, `wasm-check`) | In progress | Local: `check-deps` fails on engine→engine and core→cpal (unit and fixture tests) |
| A01-T07 `ci.yml` | In progress | Written; needs a green run on a PR (≤ 15 min) |
| A01-T08 PR title, DCO, Scorecard, CodeQL, Dependabot, label sync | In progress | Workflows written, first run on the A01 PR. 52 labels synced from `labels.yml` with `gh` (a sync workflow is optional); Dependabot alerts and security updates on |
| A01-T09 pnpm workspace skeleton | In progress | Local: `pnpm install` and `pnpm -r lint` pass |
| A01-T10 GitHub Project and issues | Not started | |
| A01-T11 Secret scanning, vulnerability reporting, Discussions | **Done** | Secret scanning + push protection on; private vulnerability reporting and Discussions enabled; workflow token read-only by default. Reported-content moderation is a web-UI setting, still to enable |
| A01-T12 Docs CI | In progress | Local: markdownlint and offline link check pass on the baseline, with listed exceptions; `reuse` job added |
| A01-T13 Licence files, REUSE | In progress | `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSES/`, `REUSE.toml`, SPDX headers; local `reuse lint`: 167/167 files compliant. Awaits CI |

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
| Ethics | Not started (OD-04) |

## 6. Change history

| Date | Change |
|---|---|
| 2026-10-18 | Status board created at the documentation baseline |
| 2026-10-18 | A01 started: workspace, tooling, xtask, workflows and pnpm skeleton built locally (T03–T09, T12) |
| 2026-10-18 | Licence decided (T01); licence files and REUSE (T13); GitHub ruleset, security and repo settings (T02, T11); A01 PR opened |
