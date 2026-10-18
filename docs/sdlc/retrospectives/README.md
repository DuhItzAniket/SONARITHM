# Retrospectives

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> **Related:** [retrospective template](_template.md) · [reviews](../reviews/README.md) · [Definition of Ready and Done](../definition-of-ready-and-done.md#4-definition-of-done-phase) · [worklog](../worklog/README.md)

A retrospective looks at how we worked during a phase: what to keep, stop and start. A phase review looks at what was delivered against the roadmap. Both are required to close a phase.

## Conventions

| Item | Rule |
|---|---|
| When | After every phase, before or alongside the phase review. A stage gate can add a stage-wide retrospective. |
| Effort | Lightweight: about 30 minutes, or a short written round when contributors are asynchronous. |
| Who | Everyone who worked on the phase. Work done by AI agents is discussed by the humans who directed and reviewed it. |
| File name | `<PHASE-ID>-retro.md`, for example `A03-retro.md`; stage-wide: `gate-<STAGE>-retro.md`. |
| Format | Copy [`_template.md`](_template.md): keep, stop, start, metrics, action items. |
| Tone | Blameless. Describe systems, tools and process, not people. |
| Privacy | No personal data or health information. Accessibility needs of contributors are discussed only with their consent and are not recorded by name. |

## Action items

- Every action item has one owner and a due date or target phase. "The team" is not an owner.
- Action items that change code, CI or docs become issues and are linked from the retrospective.
- The next retrospective starts by reviewing the previous one's action items: done, dropped (with a reason) or carried over.
- A process change agreed in a retrospective is made in the document that owns the process (for example this folder, [git-workflow](../../engineering/git-workflow.md) or the [Definition of Ready and Done](../definition-of-ready-and-done.md)), not only recorded here.
