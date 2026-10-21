# 2026-10-21: Stage B planning

> **Date(s):** 2026-10-21. **Phase:** Stage B planning (B01–B07). **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [roadmap Stage B](../../planning/roadmap.md#3-stage-b--accessible-desktop-explore-by-ear) · [implementation plan §3](../../planning/implementation-plan.md#3-stage-b--accessible-desktop-task-level) · [A02 gate](../reviews/A02-prototype-gate.md) · [A05 phase review](../reviews/A05-phase-review.md)

## Context and phase

- Phase: Stage B planning, before B01 starts. Stage A's code is complete on `main`; the `v0.1.0-alpha` release PR (#183) waits for the owner's publishing steps.
- Issue(s) / PR(s): Stage B issues [#191](https://github.com/DuhItzAniket/SONARITHM/issues/191)–[#257](https://github.com/DuhItzAniket/SONARITHM/issues/257); milestones B01–B07; this PR.
- Starting point: the implementation plan had a one-line task list per Stage B phase. ADR-0002 (Tauri) and ADR-0003 (React Aria) were still `accepted (provisional)`, waiting on the owner's A02 walkthroughs (#129). The A05 budget follow-ups #185, #186 and #187 had no Stage B home.

## Goals

- Break B01–B07 into task rows with acceptance tests, sizes, dependencies, the docs to read and the A11Y and SDR rules each task must satisfy.
- Create milestones B01–B07 and one issue per task, as for Stage A.
- Re-estimate Stage B from Stage A's actual pace, keep human-paced work separate, and mark which tasks wait on the owner.

## What was done

- **Implementation plan §3:** 67 task rows (B01 9, B02 11, B03 14, B04 8, B05 8, B06 7, B07 10), each with its rules and budgets and a "Blocked on" column. Two lanes: Lane R (23 Rust and framework-agnostic tasks, which can start now) and Lane U (37 shell and UI tasks behind "Gate U", the A02 gate's conditions for B01). Seven more tasks wait only on owner items. A critical-path diagram, the 17 human-paced items (HP-01 to HP-17) and the carry-over issues (#185, #186, #187, #123, #92, #94, #95).
- **Roadmap:** a Stage B re-estimate (≈ 15–19 agent session-days, about two calendar weeks with two sessions), re-estimated phase sizes with their owner gates, a note that the Gantt is the baseline, and B07's exit and Gate B tied to OD-16 and OD-17.
- **Records:** OD-15 to OD-17; R-24 to R-27, with the R-10 and R-22 triggers tied to the tasks that need them; finding F-120 (Stage A's pace).
- **Consistency fixes:** testing-strategy §13.3 and the conformance plan §8.1 still called co-design round 1 an ethics-approved research activity; both now follow OD-04. The research lane's RL-T01 and RL-T04 deadlines point at B07-T04.
- **GitHub:** milestones B01–B07; issues [#191](https://github.com/DuhItzAniket/SONARITHM/issues/191)–[#257](https://github.com/DuhItzAniket/SONARITHM/issues/257) with labels, milestones and the board's Phase, Size and Status fields (B01–B07 added to the Phase field, Stage A values unchanged); blocked tasks carry `status/blocked`; #185, #186 and #187 joined their Stage B milestones and link the tasks that wait for them.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Split Stage B into Lane R and Lane U | The A02 gate holds Tauri and React work until the walkthroughs; the Rust work does not depend on either ADR | Implementation plan §3.1; no ADR needed |
| Keep the A02 gate conditions as written; propose opening Lane U after HP-01 alone | Changing a gate condition and ADR-0002's confirmation timing is the coordinator's call | OD-15 (open) |
| Project format v1 needs an RFC before B05-T04's code | AGENTS.md rule 11 | B05-T04 row |
| Sizes stay S, M and L as relative effort; estimates in session-days | Stage A's code finished in about 55 hours against a four-month baseline window | F-120; plan §1.1 |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| Four tasks depended on later-numbered tasks in the same phase | Draft order | Renumbered inside B03, B04 and B07 so every dependency comes first |
| Adding project-field options can drop existing values | The GraphQL mutation replaces the option list | Passed the existing option IDs; #55 kept its A05 value |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Stage A phase tasks closed | 69 of 78 in about 55 hours | GitHub close times, milestones A01–A08 | F-120 |
| Pull requests merged in Stage A | 89 in under three days | 2026-10-18 18:24 to 2026-10-21 02:40 UTC | F-120 |

## Findings to log

- F-120: agent sessions closed 69 of the 78 Stage A phase tasks in about 55 hours, and the nine still open all wait on the owner ([findings log](../../research-paper/findings-log.md#implementation-findings)).

## Next steps

- [ ] Coordinator: decide OD-15 (open Lane U after HP-01 alone) and start the two Lane R sessions (plan §3.1).
- [ ] Owner: HP-01 and HP-02 (A02 walkthroughs, #129), which open Gate U.
- [ ] Owner: decide OD-16 and OD-17 before B07; start HP-09 (policy, budget, recruiting) early, because it sets Gate B's date.
