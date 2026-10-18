# Phase and gate reviews

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> **Related:** [phase review template](_phase-review-template.md) · [gate review template](_gate-review-template.md) · [roadmap](../../planning/roadmap.md) · [Definition of Ready and Done](../definition-of-ready-and-done.md) · [status](../status.md) · [retrospectives](../retrospectives/README.md) · [risk register](../../planning/risk-register.md)

Reviews are the formal record that a phase or a stage met its exit criteria. The roadmap defines the criteria; a review shows the evidence and records the decision. Retrospectives (how we worked) are separate and live in [retrospectives/](../retrospectives/README.md).

## Review types

| Type | When | Template | File name |
|---|---|---|---|
| Phase review | At the end of every phase (A01 … D06) | [_phase-review-template.md](_phase-review-template.md) | `<PHASE>-phase-review.md`, for example `A03-phase-review.md` |
| Stage gate review | At the end of every stage (A, B, C, D) | [_gate-review-template.md](_gate-review-template.md) | `gate-<STAGE>.md`, for example `gate-A.md` |
| A02 prototype gate | At the end of A02 | Phase review template plus the spike sections listed below | `A02-prototype-gate.md` (replaces a separate A02 phase review) |

### The A02 prototype gate report

A02 retires the highest technical risks before the stack is committed ([roadmap A02](../../planning/roadmap.md#a02--technical-spikes-and-prototype-gate)). Its report lives here as `A02-prototype-gate.md` and adds, to the phase review template:

- one section per spike (1–6) with the measurement method, the raw results per OS and webview, and a pass or fail against the budget;
- the decision on ADR-0002 (shell), ADR-0003 (frontend) and ADR-0004 (render graph): confirmed or amended, with links;
- whether the Electron fallback is triggered (required if Linux fails spike 4 or 5);
- the `F-###` finding recorded for every measurement.

## Process

1. **Draft.** The phase owner opens a PR with the review once the last exit criterion is met, or when the phase is time-boxed out. Status: `Draft`.
2. **Evidence.** Every exit criterion links to evidence: a CI run permalink, a benchmarks-log entry, a test name, a manual check record or an ADR. "Done" without a link is not evidence.
3. **Review.** Reviewers: the project lead and the architecture lead; the research lead when the phase has research hooks; the accessibility owner for phases that change UI or assistive-technology behaviour.
4. **Final.** Merging the PR makes the review final. [status.md](../status.md) marks the phase done only after that.
5. **Exceptions.** An unmet criterion is either fixed, or waived with a reason, an owner and a follow-up issue. Waivers are listed in the next gate review.

A gate review follows the same steps and ends with an explicit **Go**, **Conditional go** (with dated conditions) or **No-go** decision, signed off by the project lead.

## Rules

- Reviews are written within one week of the last exit criterion being met.
- Copy exit criteria verbatim from the roadmap, so the review can be checked against it.
- Numbers carry units, conditions and hardware. Budgets come from [01-overview §7](../../architecture/01-overview.md#7-quality-attributes-and-budgets).
- Keep lessons short in the review; the discussion belongs in the retrospective.
- A final review is not edited. Corrections are added as a dated addendum.
