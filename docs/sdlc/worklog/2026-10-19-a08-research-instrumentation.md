# 2026-10-19: A08 research instrumentation (ECHOMETRIC harness v0)

> **Date(s):** 2026-10-19–2026-10-21. **Phase:** A08. **Author:** A08 agent session (Claude Code), written by an AI agent, reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [A08 phase review](../reviews/A08-phase-review.md)

## Context and phase

- Phase: A08 — [roadmap entry](../../planning/roadmap.md#a08--research-instrumentation-v0-first-paper)
- PRs: #161, #163, #165, #166, #177, #179, #189 and this close-out PR.
- Starting point: A06 done; A05 and A07 in flight in parallel sessions; no ECHOMETRIC crate.

## Goals

- Stimulus generators GEN-001 to GEN-004, a replication gallery, a study runner with `sonarithm study run`, a technical benchmark report, a P01 draft and the `v0.2.0-alpha` release checklist, under the fast-track protocol, with no participant data (OD-04).

## What was done

- `sonarithm-echometric` with the four generators, each checked against its paper (#163; F-110 to F-113). A subagent read the primary sources and recorded every parameter with its page.
- Engines share `SNR-ENG` in blocks of 100 numbers (#161, issue #157); ECHOMETRIC has 600–699.
- P01 draft v0 with authorship, artifact, AI-usage and submission records; six incomplete bibliography records completed, with errata in the notes (#165).
- The study runner library: protocols, seeded schedules, scoring, the JSONL log, pause and resume (#166).
- The replication gallery `replication/` with 49 renders and `cargo xtask replication` (#177; F-114).
- `sonarithm study check` and `study run` with the pilot protocol (#179).
- The technical benchmark report (#189; F-115, F-116).
- This phase review, the release checklist and this worklog.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Engine code blocks instead of per-engine sub-areas | No code-format change, so no RFC | #161; coding-standards §4.2 |
| Generators re-create published designs and flag every departure | Evidence first: claim reproduction of stimuli, not results | Module docs; F-110 to F-113 |
| Score hashes in gallery records with engine versions as `*` | Release bumps must not churn records (F-102) | `replication/README.md` |
| Gallery folder `replication/` and task `cargo xtask replication` | A07's examples gallery owns `cargo xtask gallery` | #177 |
| Line-based dialogue for `study run` (no raw keys) | Screen readers read lines in order; braille shows the current line | 07, ECHOMETRIC |
| No hand-made version-bump PR for `v0.2.0-alpha` | release-plz publishes on merge; `v0.1.0-alpha` comes first | [release checklist](../releases/v0.2.0-alpha.md) |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| PR title check failed on #161 | 73 characters, limit 72 | Shortened the title |
| Gallery renders could not wait for export | A05's export was still in review | Built on a temporary local merge, then rebuilt the branch on main |
| Parallel renders failed on Windows with "file in use" | Threads staged the same data files | Stage each group's data once before the renders |
| Every rebase conflicted on status, traceability, CHANGELOG and findings | Several sessions append to the same tables | Union resolution by first-cell keys |
| The laptop ran out of memory with four sessions compiling | Parallel builds | `CARGO_BUILD_JOBS=6` and at most two building sessions (coordinator) |
| The `dry-run` release check fails on many PRs | A release-plz template error in the release configuration (not A08's) | Not required; reported to the coordinator |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| End-to-end export of the replication gallery | 131.7× real time (HW-01), 70.9× (HW-01e) | Release, one thread, 5 runs, `0103de8` on `2af548f` | Benchmarks log "A08"; F-115 |
| Cross-OS identity | 49 renders and 6 golden cases byte-identical | CI-WIN, CI-LNX, CI-MAC | F-116 |

## Findings to log

- F-110 to F-116 — [findings log](../../research-paper/findings-log.md)

## Next steps

- [ ] Owner: screen-reader walkthrough of `sonarithm study run` (manual test script).
- [ ] Owner: Gate A decision on P01; check the ICAD 2027 call.
- [ ] Owner: release `v0.1.0-alpha`, then `v0.2.0-alpha` with the checklist.
- [ ] A08 session: `cargo clean` in the worktree after the last merge.
