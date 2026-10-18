# <PHASE-ID> phase review: <phase name>

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead. This is the phase review template. In a copy, replace this block with: `> **Status:** Draft | Final (YYYY-MM-DD). **Phase:** <PHASE-ID>. **Author:** <role>. **Reviewers:** <roles>.`
> **Related:** [reviews README](README.md) · [roadmap](../../planning/roadmap.md) · [Definition of Done, phase level](../definition-of-ready-and-done.md#4-definition-of-done-phase)

<!--
Save as docs/sdlc/reviews/<PHASE-ID>-phase-review.md (A02: A02-prototype-gate.md, with the spike sections from the README).
Copy exit criteria verbatim from the roadmap. Every "Met" needs an evidence link.
-->

## Summary

| Item | Value |
|---|---|
| Phase | <PHASE-ID> · <name> — [roadmap](../../planning/roadmap.md) |
| Planned window | <YYYY-MM-DD> – <YYYY-MM-DD> |
| Actual window | <YYYY-MM-DD> – <YYYY-MM-DD> |
| Outcome | <Done / Done with waivers / Not done> |

<Three to five lines: what the phase delivered, what it did not, and anything the next phase must know.>

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | <criterion> | Met / Partially met / Not met / Waived | <CI run, benchmark entry, test name, manual record, ADR> |

Waivers:

| Criterion | Reason | Owner | Follow-up issue |
|---|---|---|---|
| | | | |

## Definition of Done (phase)

- [ ] All issues in the phase are closed or deferred with a target phase.
- [ ] No open critical bug from the phase, or an accepted exception is recorded below.
- [ ] Architecture and engineering docs match what was built; deviations are recorded as ADRs or roadmap changes.
- [ ] [status.md](../status.md) updated.
- [ ] Findings logged; measurements in the benchmarks log; research hooks satisfied or deferred.
- [ ] Traceability matrix updated.
- [ ] Risk register updated.
- [ ] Coverage targets checked for crates introduced or changed.
- [ ] Retrospective held: <link>.

## Metrics against budget

| Metric | Budget | Measured | Conditions (hardware, OS, buffer, sample rate, commit) | Evidence | Verdict |
|---|---|---|---|---|---|
| <metric> | <from 01-overview §7 or the roadmap> | <value with unit> | | | Within / Outside / Not measured |

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| | | |

**Deferred or dropped**

| Item | Reason | New target phase | Issue |
|---|---|---|---|
| | | | |

## Decisions

| ADR / RFC | Title | Status |
|---|---|---|
| | | |

## Risks

| Risk ID | Change (new / raised / lowered / retired) | Note |
|---|---|---|
| | | |

## Findings and research hooks

| Item | Status | Link |
|---|---|---|
| F-### <title> | Logged | [findings log](../../research-paper/findings-log.md) |
| Research hook from the roadmap | Satisfied / Deferred (reason) | |

## Accessibility

<For phases that touch UI or assistive-technology behaviour: manual checks done, axe status, A11Y rules implemented, open accessibility issues by severity. Otherwise: "Not applicable".>

## Lessons

- <one line each; discussion goes to the retrospective>

## Follow-up actions

| Action | Owner | Due | Issue |
|---|---|---|---|
| | | | |

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | | | Accept / Accept with waivers / Reject |
| Architecture lead | | | |
| Research lead (if research hooks) | | | |
| Accessibility owner (if UI) | | | |
