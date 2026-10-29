# 2026-10-29: Gate U opened provisionally

> **Date(s):** 2026-10-29. **Phase:** Stage B (B01–B07), affecting Stage C's Lane U. **Author:** AI agent (Claude Code, Claude Opus 5.5) in a session started by the coordinator session, to be reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [OD-15](../../planning/open-decisions.md#closed-decisions) · [implementation plan §3.1](../../planning/implementation-plan.md#31-how-stage-b-runs) · [A02 gate report](../reviews/A02-prototype-gate.md#decision) · [Stage B planning worklog](2026-10-21-stage-b-planning.md)

## Context and phase

- Stage B's Rust lane was complete, and the shell and UI tasks (Lane U) still waited for Gate U: the owner's Windows walkthroughs (HP-01) and ADR-0002 and ADR-0003 confirmed for Windows (HP-04), per OD-15 of 2026-10-21.
- The owner decided in the coordinator session: "Open now + I drive NVDA".

## What was done

- **OD-15 amended:** Gate U is open provisionally, with three conditions:
  - the thin host;
  - one announcer interface for both OD-05 routes;
  - axe, keyboard tests and the A02 UI Automation tree dump in CI for every component.

  The NVDA walkthrough becomes a checkpoint that the coordinator runs through desktop control, with NVDA's Speech Viewer as the transcript. The owner's Narrator walkthrough comes before B02-T10 and before any component sign-off. Linux with Orca stays due by B03's exit.
- **Docs updated to match:**
  - R-33 (rework if a walkthrough fails);
  - the Gate U definition and the HP-01 and HP-04 rows in the plan;
  - B02-T03 (UIA tree dump in CI), B02-T10 (HP-01 first) and B03-T04 (both routes behind one interface);
  - the Stage C Lane U wording, the roadmap, the A02 gate report, ADR-0002's and ADR-0003's Confirmation sections (decisions unchanged), the ADR index and the status board.
- **Labels:**
  - `status/blocked` removed from the 28 issues that were blocked only by Gate U.
  - It stays on owner-only items, on items behind another owner step (directly or through a dependency) and on B02-T10.
  - The affected issue texts now describe the provisional gate.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Keep `status/blocked` on B07-T05 (#252), B07-T06 (#253) and C06-T03 (#347) although their own blocker cell says only Gate U | Each depends on a task behind another owner step: the matrix and co-design (HP-10, HP-09) or signing (HP-23) | OD-15 (amended); the PR body |
| Keep B03-T04 (#214) blocked | It also lists HP-01: OD-05 closes from the walkthrough evidence. Building both routes behind one interface could start earlier if the coordinator lifts the label | OD-15 (amended) |

## Next steps

- [ ] Coordinator: run the NVDA walkthrough (WALKTHROUGH.md §1) through desktop control and record the Speech Viewer transcript in the A02 gate report.
- [ ] Owner: the Narrator walkthrough (§2) before B02-T10.
- [ ] Coordinator: confirm ADR-0002 and ADR-0003 for Windows (HP-04) once both are in, which makes Gate U final.
