# A01 retrospective

> **Date:** 2026-10-19. **Phase:** A01. **Participants:** @DuhItzAniket (owner), coordinator agent.
> **Related:** [A01 phase review](../reviews/A01-phase-review.md)

## Keep

- Writing the documentation baseline first. Every A01 task had a clear spec and acceptance criteria to check against.
- One aggregate `all-checks-pass` gate. New CI jobs need no ruleset change.
- Verifying replacements (licence texts via `reuse download`, action SHAs via `gh api`, dead-link fixes by hand) instead of guessing.

## Stop

- Deep PR stacks. A five-deep stack had to be rebuilt after every squash merge.
- Treating external link failures as blocking on pull requests.

## Start

- Coordinator merges in strict stack order, with the phase session rebasing after each merge.
- Linking every PR to its task issue on the [project board](https://github.com/users/DuhItzAniket/projects/2).
- Phase sessions append to `status.md` only in their own phase rows, to keep conflicts trivial.

## Action items

| Action | Owner | Due |
|---|---|---|
| Tell the A02 and A03 sessions their task issue numbers | Coordinator | 2026-10-19 |
| Re-check reported-content moderation if the repository moves to an organization | Project lead | When it happens |
