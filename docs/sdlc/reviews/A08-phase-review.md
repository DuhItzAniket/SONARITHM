# A08 phase review: research instrumentation v0

> **Status:** Final (2026-10-21). **Phase:** A08. **Author:** A08 agent session (Claude Code) for the project lead. **Reviewers:** coordinator session (owner's delegate), @DuhItzAniket.
> **Related:** [reviews README](README.md) · [roadmap A08](../../planning/roadmap.md#a08--research-instrumentation-v0-first-paper) · [worklog: A08](../worklog/2026-10-19-a08-research-instrumentation.md) · [07, ECHOMETRIC](../../architecture/07-engines.md) · [benchmark report](../../research-paper/benchmark-report-a08.md) · [v0.2.0-alpha checklist](../releases/v0.2.0-alpha.md)

## Summary

| Item | Value |
|---|---|
| Phase | A08 · Research instrumentation v0 (first paper) ([roadmap](../../planning/roadmap.md#a08--research-instrumentation-v0-first-paper)) |
| Planned window | after A07, about 4 weeks |
| Actual window | 2026-10-19 – 2026-10-21 |
| Outcome | **Done with one criterion pending the owner** (P01 submission or deferral is a Gate A decision) |

A08 made SONARITHM a research instrument built around ECHOMETRIC:

- four stimulus generators that re-create published designs from a seed;
- a replication gallery of 49 renders that reproduce byte for byte on three operating systems;
- a study runner, as a library and as `sonarithm study run`, with a pilot protocol;
- a technical benchmark report;
- a draft of paper P01.

Checking the published parameters against the papers corrected the project's notes in several places (F-110 to F-113). Two published stimulus details cannot be reproduced yet (F-114). No participant data was collected: the owner works without an ethics route (OD-04). The `v0.2.0-alpha` release is prepared as a checklist; tagging and publishing are the owner's, after `v0.1.0-alpha`.

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | Every gallery item renders reproducibly from a committed spec with its manifest. | Met | `replication/`: 49 specs, each exported with its render manifest; the xtask test `the_replication_gallery_renders_reproducibly` compares every record (WAV SHA-256, loudness, release-independent Score hash) on Windows, Linux and macOS in CI (#177; F-116) |
| 2 | P01 is submitted (or consciously deferred at the gate with a recorded reason). | **Pending the owner** | Draft v0 merged (#165); every citation checked against its source. The ICAD 2027 call is not published (checked 2026-10-19). Submitting or deferring is the owner's decision at Gate A ([submission record](../../research-paper/papers/P01-icad-2027-toolkit/submission.md)) |

Waivers: none. Criterion 2 is not waived; it waits for the owner and the ICAD 2027 call.

## Definition of Done (phase)

- [x] All issues in the phase are closed, or stay open for the owner: #85 (#163), #86 (#177), #87 (#166, #179), #88 (#189) and #157 (#161) are closed. #89 (P01) stays open until submission or deferral; #90 (release) stays open until the owner tags.
- [x] No open critical bug from the phase.
- [x] Docs match what was built:
  - 07, ECHOMETRIC: generators, study runner, the normative log schema.
  - 11 §2: the `study` commands; `sonarithm gallery render` was not built, replaced by `cargo xtask replication`.
  - 02 layout: `replication/`.
  - The evaluation plan: the gallery table and the implemented log schema.
  - coding-standards §4.2: engine code blocks.
- [x] [status.md](../status.md) updated.
- [x] Findings logged: F-110 to F-116. Measurements in the benchmarks log.
- [x] Traceability matrix: FR-084, NFR-01, NFR-03.
- [x] Risk register: no change made here; see "Risks" for proposals.
- [x] Coverage targets: not measured (no coverage tooling in CI). Every module has unit tests; invariants have property tests; manifests and a session log are pinned snapshots; the CLI has end-to-end tests.
- [x] Retrospective: folded into "Lessons" below (one agent session ran the phase).

## Metrics against budget

| Metric | Budget | Measured | Conditions | Evidence | Verdict |
|---|---|---|---|---|---|
| Export of the replication gallery, end to end, one thread | — (new) | 131.7× real time (HW-01), 70.9× (HW-01e) | Release, Rust 1.99.0, 5 runs each, commit `0103de8` on `2af548f` | F-115 | — |
| Offline audio across OS | null residual ≤ −90 dBFS | byte-identical (−∞) for 49 gallery renders and A05's 6 golden cases | CI-WIN, CI-LNX, CI-MAC | F-116 | Within |
| Score IR across OS | identical | identical (release-independent hashes, 49 renders) | as above | F-116 | Within |
| Stimulus generators across OS | identical | pinned manifests match on three OSes | as above | #163 | Within |

The engine budgets are A05's; the [benchmark report](../../research-paper/benchmark-report-a08.md) restates them for P01 and P07 with the owner's decisions of 2026-10-21: PB-01 not met on HW-01, deferred to B03 ([#186](https://github.com/DuhItzAniket/SONARITHM/issues/186)); PB-03 not met on the four-E-core stand-in, waived for v0.1.0-alpha and due before Gate B ([#185](https://github.com/DuhItzAniket/SONARITHM/issues/185)); PB-02 accepted with a 128-frame rerun ([#187](https://github.com/DuhItzAniket/SONARITHM/issues/187)); PB-06 accepted.

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| Stimulus generators: reversals × density, price series, SNR transits, correlation triplets (T01) | `sonarithm_echometric::harness::generators` | #163 |
| Engine code blocks in `SNR-ENG` (issue #157) | `xtask/src/codes.rs`, coding-standards §4.2 | #161 |
| Replication gallery: Smith & Walker context cues, Nees & Walker density, Tucker Brown transits, Erie examples (T02) | `replication/`, `cargo xtask replication` | #177 |
| Study runner with structured logs (T03) | `harness::study`; `sonarithm study check` and `study run`; `examples/studies/pilot-001` | #166, #179 |
| Technical benchmarks: latency, real-time factor, determinism (T04) | [benchmark report](../../research-paper/benchmark-report-a08.md) | #189 |
| Paper P01 drafted (T05) | `docs/research-paper/papers/P01-icad-2027-toolkit/` | #165 |
| Release v0.2.0-alpha (T06) | [release checklist](../releases/v0.2.0-alpha.md) | this PR (checklist only) |

**Deferred or dropped**

| Item | Reason | New target phase | Issue |
|---|---|---|---|
| Dombois audification gallery item | Needs the `audify` mark | C03 | — |
| P01 submission | Owner's act; the ICAD 2027 call is not published | Gate A | #89 |
| Tagging and publishing v0.2.0-alpha | Owner's go, after v0.1.0-alpha (A07-T08) | Gate A | #90 |
| Short note ramps and a broadband click source (F-114) | Spec v0.1 has no envelope setting; the DSP has no noise source | A spec minor version; DSP | — |
| Sharing the device lookup between `study run` and `play` | See "Decisions": the playback loops stay separate; only opening a device by name or ID is worth sharing | CLI follow-up | — |
| Running studies with participants | No ethics route (OD-04) | Gate B review | RL-T02 |

## Decisions

| ADR / RFC | Title | Status |
|---|---|---|
| — | No new ADR. Engine code blocks (#161) change no format; the study log is research infrastructure, documented normatively in 07 | — |
| [ADR-0013](../../decisions/0013-determinism-and-provenance.md) | Determinism and provenance | Accepted; strengthened by F-116 |
| OD-04 | Owner works alone without ethics review | Closed; A08 collected no participant data |
| — | `sonarithm study run` keeps its own playback loop instead of `sonarithm play`'s (#174) | Decided 2026-10-21, no ADR needed (CLI-internal). `play` announces "Playing …" just before the sound and "Finished at …" after it, enters raw keyboard mode with transport keys (pause, step, seek) and paces its silent output in real time. In a trial, the announcement would overlap the stimulus for screen-reader users (A11Y-20 sequences speech and sound), seeking inside a stimulus is not part of any protocol (a replay plays the whole stimulus and is logged), raw mode would break the line-by-line dialogue, and the scripted pilot needs an unpaced silent output. The loop is about 40 lines; only the device lookup is shared work |

## Risks

| Risk ID | Change | Note |
|---|---|---|
| R-06 | Proposed: lower likelihood 3 → 2 at Gate A | Golden cases and 49 gallery renders are byte-identical across Windows, Linux and macOS AArch64 (F-116) |
| R-11 | No change | The study runner exists; using it with participants waits for an ethics route |
| R-12 | No change | P01 draft v0 exists; the ICAD 2027 call is not published |
| — | New, for the register at Gate A: replication stimuli depend on unpublished details | Smith & Walker's series values and beep timing, Nees & Walker's shapes, Tucker Brown's transit positions are not published (F-110 to F-112); the gallery documents its reconstructions |

## Findings and research hooks

| Item | Status | Link |
|---|---|---|
| F-110 · Nees & Walker crossed 0–3 reversals with 1–8 points/s; no generating function | Logged | [findings log](../../research-paper/findings-log.md) |
| F-111 · Smith & Walker's single series is unpublished; "200–1900 Hz" is rounded | Logged | as above |
| F-112 · Tucker Brown's SNR = t_d·√N_t/σ; positions unpublished | Logged | as above |
| F-113 · Elmquist's added-noise range is stated two ways; r ≈ 0.95 reproduced | Logged | as above |
| F-114 · The gallery reproduces every stated pitch, timing and count, except short ramps and broadband clicks | Logged | as above |
| F-115 · End-to-end export at about 130× (HW-01) and 70× (HW-01e) | Logged | as above |
| F-116 · 49 gallery renders byte-identical on three OSes | Logged | as above |
| P01 (ICAD 2027) | Draft v0; Gate A decision | [outline](../../research-paper/papers/P01-icad-2027-toolkit/outline.md) |
| P03 (benchmark harness) | The harness part needs no participants and is ready: GEN-001 to GEN-004, the study runner | [publication plan](../../research-paper/publication-plan.md) |
| H-01, H-02 | Prepared: GEN-001/GEN-002 and the study runner implement their stimuli and tasks; no study can run (OD-04) | [hypotheses](../../research-paper/hypotheses.md) |

## Accessibility

`sonarithm study run` is a terminal dialogue designed for screen readers: one plain line per prompt, answers as typed lines, no raw keyboard mode, no redraws, no colour; with `--json` the dialogue moves to standard error. The CLI tests check those rules on the text. The walkthrough with NVDA is an owner step ([manual test script](../../engineering/manual-tests/a08-study-run-screen-reader.md)); an agent cannot do it.

## Lessons

- Reading the primary sources early paid off: the kickoff notes' stimulus summaries were incomplete in several places (Nees & Walker's four reversal levels, Smith & Walker's single series queried at every hour, Elmquist's two statements of the added noise), and the corrections went into the generators before any gallery item depended on them.
- Parallel sessions collide on names as well as files: A07's examples gallery took `cargo xtask gallery` while the replication gallery waited for A05's export, so the replication gallery moved to `replication/`. Claim shared names (xtask tasks, top-level folders, CLI commands) in the coordinator's notes before building on them.
- Score hashes that include crate versions make every release churn pinned records (F-102); storing a release-independent hash in the gallery records avoided that.
- Shared docs (status, traceability, CHANGELOG, findings) conflicted on almost every rebase; keeping each row keyed by its first cell and resolving by union kept it mechanical.
- Measuring on a quiet machine mattered: the first timing run predated A05's voice optimizations, and re-measuring on the merged code moved the result from 99.5× to 131.7×.

## Follow-up actions

| Action | Owner | Due | Issue |
|---|---|---|---|
| Screen-reader walkthrough of `sonarithm study run` (NVDA) | Owner | Before v0.2.0-alpha | #87 (comment) |
| Gate A decision on P01: submit, merge with P03, or defer; check the ICAD 2027 call | Owner | Gate A | #89 |
| Tag v0.1.0-alpha, then v0.2.0-alpha, after the release checklist | Owner | Gate A | #90 |
| Envelope settings in a spec minor version, and a click or noise source (F-114) | Architecture lead | Spec v0.2 / DSP | — |
| Share the device lookup (by name or ID) between `study run` and `play` | CLI owner | A CLI follow-up | — |
| Dombois audification gallery item | CURVASONA | C03 | — |

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | @DuhItzAniket | | |
