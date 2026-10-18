# A01 phase review: Repository bootstrap and governance

> **Status:** Final (2026-10-19). **Phase:** A01. **Author:** coordinator agent (Claude Code) for the project lead. **Reviewers:** @DuhItzAniket.
> **Related:** [reviews README](README.md) · [roadmap A01](../../planning/roadmap.md#a01--repository-bootstrap-and-governance) · [A01 retrospective](../retrospectives/A01-retro.md) · [worklog: bootstrap](../worklog/2026-10-18-a01-repository-bootstrap.md) · [worklog: licensing and GitHub setup](../worklog/2026-10-18-a01-licensing-and-github-setup.md)

## Summary

| Item | Value |
|---|---|
| Phase | A01 · Repository bootstrap and governance ([roadmap](../../planning/roadmap.md#a01--repository-bootstrap-and-governance)) |
| Planned window | 2026-11-01 – 2026-11-11 |
| Actual window | 2026-10-18 – 2026-10-19 |
| Outcome | **Done** |

A01 delivered:

- the Cargo and pnpm workspaces, `cargo xtask` automation and CI on three OSes;
- licensing (MIT OR Apache-2.0 for code, CC BY 4.0 for docs and data) with REUSE compliance;
- the `main` ruleset with six required checks, security settings, and labels;
- the GitHub Project board with one issue per Stage A task.

It started two weeks before its planned window because the documentation baseline finished early. One governance item could not be delivered as planned, because GitHub doesn't offer it for personal repositories: "report to repository maintainers". The Code of Conduct names the alternative channels instead. A02 and A03 started in parallel during A01's last day.

## Exit criteria

| # | Criterion (verbatim from the roadmap) | Result | Evidence |
|---|---|---|---|
| 1 | A PR to `main` runs all required checks green on Linux, Windows and macOS. | Met | #1: 22 of 22 checks green, including `clippy and test` on ubuntu, windows and macos and the `all-checks-pass` gate |
| 2 | Branch protection blocks direct pushes. | Met | `main` ruleset (id 25350043): pull request required, linear history, no force-push or deletion, admin bypass only through a pull request |
| 3 | Scorecard runs. | Met | `scorecard.yml` completed successfully on the push of #1 to `main` |
| 4 | Every root governance file exists. | Met | README, AGENTS, CLAUDE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, GOVERNANCE, CHANGELOG, CITATION.cff, LICENSE-MIT, LICENSE-APACHE, REUSE.toml |
| 5 | ADR-0010 (licence) is **Accepted**. | Met | [ADR-0010](../../decisions/0010-licensing.md), accepted 2026-10-18 |

Waivers: none.

## Definition of Done (phase)

- [x] All issues in the phase are closed. A01-T10 closes with this review's PR.
- [x] No open critical bug from the phase.
- [x] Docs match what was built. Deviations recorded:
  - external links are advisory on PRs ([ci-cd](../../engineering/ci-cd.md));
  - the conduct reporting channels changed (CODE_OF_CONDUCT).
- [x] [status.md](../status.md) updated.
- [x] Findings logged: none produced (bootstrap work). Measurements: CI wall time, below.
- [x] Traceability matrix: no requirement changed status in A01.
- [x] Risk register updated: R-11 reframed, R-14 mitigation updated, R-22 opened.
- [x] Coverage targets: not applicable (skeleton crates only).
- [x] Retrospective held: [A01-retro.md](../retrospectives/A01-retro.md).

## Metrics against budget

| Metric | Budget | Measured | Conditions | Evidence | Verdict |
|---|---|---|---|---|---|
| CI wall time on a PR | ≤ 15 min (A01-T07) | About 1–2 min per `ci.yml` run | GitHub-hosted runners, 3-OS matrix, rust-cache warm | Runs on #1 | Within |

## Scope

**Delivered**

| Deliverable (from the roadmap) | Where | PRs |
|---|---|---|
| Cargo workspace skeleton, toolchain, lints, `deny.toml`, `xtask` | Root, `crates/`, `apps/cli`, `xtask/` | #1 |
| CI, PR-title, DCO, Scorecard, CodeQL, docs workflows; Dependabot | `.github/` | #1 |
| pnpm workspace skeleton | `package.json`, `packages/` | #1 |
| Licence files and REUSE | `LICENSE-*`, `LICENSES/`, `REUSE.toml` | #1 |
| Ruleset, secret scanning, vulnerability reporting, Discussions, labels | Repository settings | Applied directly, recorded in the worklog |
| GitHub Project board, milestones A01–A08, Stage A issues | [SONARITHM Roadmap](https://github.com/users/DuhItzAniket/projects/2) | This PR records it |
| Owner decisions OD-01, OD-09 (licences), OD-04 (solo research path) | `docs/planning/open-decisions.md` | #1, #2, this PR |

**Deferred or dropped**

| Item | Reason | Where it goes |
|---|---|---|
| Reported-content moderation | Available only for organization-owned repositories | Enabled if the project moves to an organization ([GOVERNANCE.md](../../../GOVERNANCE.md)) |
| A label-sync workflow | One-off `gh` sync was enough | Optional, when the taxonomy changes |

## Lessons for the next phases

1. **Stacked PRs and squash merges conflict.** After each squash merge the next PR must be rebased, so stacks stay at most two PRs deep.
2. **External link checks are noisy.** Publishers block bots and hosts flake, so the check is advisory on PRs and authoritative only in the weekly run.
3. **`reuse lint` on Windows doesn't apply git's ignore rules** to pnpm's per-package `node_modules`. Lint a clean copy locally; CI on Ubuntu is authoritative.
