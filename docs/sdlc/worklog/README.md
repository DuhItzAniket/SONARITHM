# Engineering worklog

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> **Related:** [entry template](_template.md) · [status](../status.md) · [Definition of Ready and Done](../definition-of-ready-and-done.md) · [reviews](../reviews/README.md) · [findings log](../../research-paper/findings-log.md) · [benchmarks log](../../research-paper/benchmarks-log.md)

The worklog is the project's engineering journal: what was attempted, what happened, and why. It gives phase reviews, papers and future contributors a dated record that commit messages and PR descriptions are too short to hold. Human contributors and AI coding agents both write entries.

## Conventions

| Item | Rule |
|---|---|
| Granularity | One file per working session, or one per week for short, related sessions. Pick one per workstream and keep to it. |
| File name | `YYYY-MM-DD-<slug>.md`, dated by the first day the entry covers. The slug is lowercase kebab-case and names the work, ideally with the phase: `2026-11-03-a01-ci-matrix.md`. |
| Template | Copy [`_template.md`](_template.md). Delete sections that do not apply rather than leaving them empty, except "Next steps". |
| Length | Short. Bullets over prose. Link to PRs, issues, CI runs and commits instead of pasting their content. |
| Immutability | An entry is append-only once its PR is merged. Corrections go in a dated addendum at the end of the entry. |
| Authorship | Record the author's role or handle, and for agent-written entries, that an agent wrote it and which human reviewed it. |
| Privacy | No personal data, credentials, tokens, absolute user paths or unpublished study data. Refer to study participants only by study ID. |

## What goes where

The worklog is a journal, not the system of record. Content that has a canonical home goes there, and the worklog links to it.

| Content | Canonical home | In the worklog |
|---|---|---|
| A decision with lasting effect | [ADR](../../decisions/README.md) (or an [RFC](../../rfcs/README.md) first) | One line and the ADR link |
| A research-relevant observation | [Findings log](../../research-paper/findings-log.md) (`F-###`) | The F-ID and context |
| A benchmark or measurement campaign | [Benchmarks log](../../research-paper/benchmarks-log.md) | Raw numbers while work is in progress; the final figures go to the log |
| Phase progress | [status.md](../status.md) | Optional reference |
| A problem future contributors will hit | [dev-environment troubleshooting](../../engineering/dev-environment.md#troubleshooting) or an issue | How it was found and solved |

## When to write

- At the end of a working session or week on roadmap work.
- When a session produced a decision, a measurement or a solved problem, even if it was short.
- Before handing work to someone else (person or agent), so the next session can start from the "Next steps" section.

## Index

Entries are listed by file name order; no separate index is maintained.
