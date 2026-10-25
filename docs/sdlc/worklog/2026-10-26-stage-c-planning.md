# 2026-10-26: Stage C planning

> **Date(s):** 2026-10-26. **Phase:** Stage C planning (C01–C06). **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [roadmap Stage C](../../planning/roadmap.md#4-stage-c--bring-any-chart-general-availability) · [implementation plan §4](../../planning/implementation-plan.md#4-stage-c--bring-any-chart-task-level) · [Stage B planning](2026-10-21-stage-b-planning.md)

## Context and phase

- Phase: Stage C planning, ahead of Gate B. Stage A's code is complete; Stage B has 21 of 67 tasks done, and its Rust lane has two tasks left (B03-T08, B06-T07). Gate U is still closed: it waits on the owner's Windows walkthroughs (HP-01) and the ADR-0002/0003 confirmation (HP-04).
- Issue(s) / PR(s): the Stage C issues and milestones C01–C06 listed in this PR; this PR.
- Starting point: the implementation plan had one epic row per Stage C phase. Docs only: no cargo, pnpm or maturin command ran, because the owner was using the laptop for other heavy work.

## Goals

- Break C01–C06 into task rows with acceptance tests, sizes, dependencies, the docs to read, the SDR and A11Y rules each must satisfy, and what each is blocked on.
- Split the work into lanes: work that can start before Stage B's UI exists, work that needs it, and owner-only items.
- Plan around the licence traps (AGENTS.md rule 8) and keep provenance and consent rules visible (rules 7 and 9).
- Re-estimate Stage C from the actual pace of Stages A and B, and create the milestones and issues.

## What was done

- **Implementation plan §4:** 68 task rows (C01 13, C02 11, C03 9, C04 10, C05 12, C06 13). Lane R has 46 tasks, Lane U 12, and 10 wait only on owner items. The section also has how Stage C runs, a critical-path diagram, the new human-paced items HP-18 to HP-29, a licence-trap and provenance table, and the carry-over from Stages A and B. Stage D keeps its epic row (now §5).
- **Roadmap:** a Stage C re-estimate of about 16–18 agent session-days (Lane R about 12–14), size lines per phase, and Gate C's date tied to the human-paced items (TODO(verify)).
- **Records:** open decisions OD-20 to OD-28; risks R-28 to R-32, with R-07 and R-13 updated; finding F-150 (Stage B's Rust-lane pace); the midly, Highcharts and OCR-model traps in the supply-chain policy; the Stage C rows of the status board; B07-T10 now reviews the Stage C rows instead of writing them.
- **GitHub:** milestones C01–C06; one issue per task with labels `stage/c`, `area/*`, `engine/*`, `kind/*`, and `status/blocked` on the 22 issues that wait for Gate U or the owner; each issue on the board with Phase (C01–C06 options added, existing values kept), Size and Status.

## Decisions

| Decision | Why | Record |
|---|---|---|
| One RFC (RFC-0004, spec v0.2) for `template`, `instruments` and `lint` and the chart calibration and correction shapes, before any code reads them | RFC-0001 reserves these keys; one RFC avoids three spec versions in a stage (R-31) | C01-T01; proposed in the PR body |
| A licence check task at the start of each phase that adds dependencies | AGENTS.md rule 8; midly turned out to be `Unlicense`, which the allowlist does not contain | Plan §4.4; OD-21 |
| Static chart parsing only: no script execution and no remote fetch | 12 §6 rules out script execution from files; ADR-0012 rules out network calls without consent | C01-T06; OD-25 (coordinator); proposed ADR-0021 |
| A clean-room procedure before the raster digitizer's code | WebPlotDigitizer is AGPL; the plan re-implements its documented workflow | C02-T01; R-28; proposed ADR-0022 |
| Office chart caches are `exact`, SVG and PDF geometry `vector` | 10 §3 and the roadmap's C01 exit; a `sonarithm-core` doc comment says otherwise and is fixed in C01-T03 | Plan §4.4 |
| No network, `ml_estimate` or `cloud_estimate` work in Stage C | They are D04 scope; Stage C stays offline, OSC output is the only socket and it waits for a user action | Plan §4.1 |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| The board's Phase field had no C options | Options are added per stage | Added C01–C06 through the GraphQL mutation with the existing option IDs passed, so no value was lost |
| The A11Y task suite has no chart-import, correction or function task | It was written for Stage B | C01-T13, C02-T11 and C03-T08 add tasks T-16 to T-18 when they land |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Stage B Lane R tasks closed | 21 of 23 on two calendar days | GitHub close times, 2026-10-21 and 2026-10-25 | F-150 |

## Findings to log

- F-150: Stage B's Rust lane closed 21 of its 23 tasks on two calendar days, about four to five tasks per session-day ([findings log](../../research-paper/findings-log.md#implementation-findings)).

## Next steps

- [ ] Coordinator: decide OD-20 (start Lane R before Gate B) and OD-25 (static chart parsing); review RFC-0004 when C01-T01's PR opens.
- [ ] Owner: the licence decisions OD-21 (MIDI file library), OD-22 (SoundFonts) and OD-23 (OCR model), and OD-24, OD-26 to OD-28 when their phases near (#129).
- [ ] Sessions: one build-heavy session on PLOTSONA (C01-T02 onward) while the owner's other work runs; a light docs session for C01-T01 (RFC-0004), C02-T01 and C04-T01.
