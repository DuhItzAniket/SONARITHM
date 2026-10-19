# A04 retrospective

> **Date:** 2026-10-19. **Phase:** A04. **Participants:** @DuhItzAniket (owner), coordinator session, A04 agent session. **Phase review:** [A04-phase-review.md](../reviews/A04-phase-review.md).
> **Related:** [retrospectives README](README.md)

## Previous action items

From the [A01 retrospective](A01-retro.md), as they applied to A04:

| Action | Owner | Outcome |
|---|---|---|
| Coordinator merges in strict stack order, with the phase session rebasing after each merge | Coordinator, A04 session | Done, then replaced: the fast-track (2026-10-19) moved A04 to one PR per task, landed by the phase session with auto-merge |
| Link every PR to its task issue | A04 session | Done: each task PR closes its issue (#46 to #54); #45 closed with a comment linking #113 and #114 |
| Phase sessions edit only their own `status.md` rows | A04 session | Done, but adjacent phase rows still conflict on every rebase; resolved by script |

## Keep

- Building every task on its own branch, so each could be reviewed and landed alone.
- One type inference for text from every format (CSV, JSON, spreadsheets): the same messy column behaves the same everywhere.
- Fuzzing third-party decoders before trusting them. It found arrow-ipc and calamine crashes in minutes (F-071).
- A trial rebase onto the next phase's unmerged head before it lands, so the real rebase holds no surprises.

## Stop

- Splitting one task over several PRs. T02 and T07 needed folding when the fast-track arrived.
- Picking finding IDs from "the next free number" while other phases append in parallel: A04's IDs moved twice (F-046 → F-050 → F-070).

## Start

- Reserve a findings ID block per phase when the phase starts, in the findings-log header.
- Script the recurring rebase conflicts (Cargo.lock, adjacent `status.md` rows, generated code tables) from the first cycle.
- Run new fuzz targets on Linux before calling a decoder safe; libFuzzer on Windows cannot exercise panic containment. The two-minute smoke runs on the T10 pull request found a crash and a timeout in `columnar_read` that the short Windows runs had not.

## Metrics

| Metric | Value |
|---|---|
| Task PRs | 9 (#116, #133, #135, #136, #138, #139, #141, #142, #143), plus #113 and #114 for T01 |
| Tests in `sonarithm-data` | 129 (nextest, all targets) |
| PB-05 | 0.506 s on HW-01, 0.912 s on HW-01e, budget 1.5 s |

## Action items

| Action | Owner | Due |
|---|---|---|
| Add per-phase finding ID blocks to the findings-log header when a phase starts | Coordinator | Next phase kickoff |
| Fix or contain the slow Parquet page header (#146) | Data-layer owner | Before Stage A ends |
| Triage the Linux nightly fuzz results for the data targets | Data-layer owner | A05 |
