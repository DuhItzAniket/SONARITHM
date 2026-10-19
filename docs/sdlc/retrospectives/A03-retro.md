# A03 retrospective

> **Date:** 2026-10-19. **Phase:** A03. **Participants:** @DuhItzAniket (owner), coordinator session, A03 agent session.
> **Related:** [A03 phase review](../reviews/A03-phase-review.md) · [worklog](../worklog/2026-10-19-a03-core-types-spec-score.md)

## Keep

- Writing the RFC first and implementing it as written. Accepting all 21 resolutions in one decision kept T06–T11 free of rework.
- Pinning determinism with tests that run on all three CI OSes: `libm` bit patterns, the canonical Score hash, the spec hash, and the per-track random streams.
- A failing and a passing test for every validation rule, with the passing case required to produce no diagnostic at all.
- Generating what can drift (the JSON Schema, the code registry) and diffing it in CI.
- Property tests found a real bug in a dependency's default configuration (F-019) within the first runs.

## Stop

- Stacks deeper than two PRs. The A03 stack reached nine, and each squash merge cost a rebase and a CI cycle.
- Treating the rollup's `all-checks-pass` from a cancelled duplicate run as a real failure.
- Writing generated-file and lockfile edits in many stacked PRs at once; they conflict on every merge.

## Start

- Retarget the child PR to `main` before merging its parent (now the coordinator's rule).
- Land a phase's PRs before merging another phase's PRs that touch the same shared files (the lockfile, code-table lists), or batch them.
- Record each phase's finding-ID range up front (A02: F-030 … F-049, A03: F-019 … F-029) so parallel sessions never collide.
- Run long fuzz campaigns locally when CI would make the phase wait, and confirm them on Linux CI afterwards.

## Action items

| Action | Owner | Due |
|---|---|---|
| Make `docs / schema-diff` a required check | Coordinator | Next ruleset change |
| Evaluate Dependabot #124 (`sha2`) and #125 (`rand_chacha`) against the pinned hashes and streams | A03 session | Done 2026-10-19: both safe; #125 needed a one-line import fix |
| Keep stacks at most two PRs deep in A04–A08 | All sessions | Ongoing |
