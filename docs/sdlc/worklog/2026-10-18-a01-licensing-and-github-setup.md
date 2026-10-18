# 2026-10-18: A01 licensing, GitHub settings and first pull request

> **Date(s):** 2026-10-18. **Phase:** A01. **Author:** project lead, written by an AI agent (Claude Code), reviewed by @DuhItzAniket.
> **Related:** [worklog conventions](README.md) · [A01 bootstrap entry](2026-10-18-a01-repository-bootstrap.md) · [status](../status.md)

## Context and phase

- Phase: A01 ([roadmap entry](../../planning/roadmap.md#a01--repository-bootstrap-and-governance))
- Starting point: the documentation baseline was on `main` (`5c521d8`). The A01 workspace, tooling and workflows were built locally but uncommitted. The licence was undecided.

## Goals

- Record the owner's licence decision and finish A01-T13 (licence files and REUSE).
- Configure the GitHub repository (A01-T02, T11, label sync from T08).
- Open the A01 pull request so the new workflows run for the first time.

## What was done

- **Licence (T01).** The owner chose `MIT OR Apache-2.0` for code and `CC-BY-4.0` for documentation, example datasets and study stimuli, with DCO sign-off. ADR-0010 was accepted, OD-01 and OD-09 were closed, and README, CONTRIBUTING, CITATION.cff and the supply-chain policy were updated.
- **Licence files and REUSE (T13).**
  - Licence files: `LICENSE-MIT` (notice "SONARITHM contributors"), `LICENSE-APACHE`, and `LICENSES/{MIT,Apache-2.0,CC-BY-4.0}.txt`, downloaded with `reuse download`.
  - Annotations: `REUSE.toml` covers docs (CC BY 4.0) and configuration and lockfiles (code licence).
  - SPDX headers on all 20 Rust, TypeScript and JavaScript source files.
  - Licence fields: `license = "MIT OR Apache-2.0"` in every crate and `package.json`. cargo-deny now checks the workspace crates' own licences (`[licenses.private] ignore = false`).
  - CI: a `reuse` job in `docs.yml`, pinned to `fsfe/reuse-action` v6.0.0 at its commit SHA.
- **GitHub settings (T02, T11, part of T08):**
  - Repository metadata: description and topics.
  - Discussions enabled.
  - Merging: squash only, with the PR title and commit details kept so DCO sign-offs survive; branches deleted after merge; sign-off required for web commits.
  - Security: private vulnerability reporting, Dependabot alerts and security updates. Secret scanning and push protection were already on.
  - Actions: the default workflow token is read-only.
  - The `main` ruleset (id 25350043) blocks deletion and force-push, requires linear history and a pull request (1 approval, code-owner review, stale approvals dismissed, threads resolved, squash only), and requires six checks: `all-checks-pass`, `pr-title`, `dco`, `markdown`, `internal-links`, `reuse`. Repository admins can bypass when merging a pull request, per the interim review rule in GOVERNANCE.md.
  - Labels: 52 synced from `.github/labels.yml` with `gh label create --force`.

## Decisions

| Decision | Why | Record |
|---|---|---|
| Copyright notice "SONARITHM contributors" | Contributors keep copyright; no single-person notice in shared files | [ADR-0010](../../decisions/0010-licensing.md) |
| Required checks configured before their first run | Bootstrap exception to the rule in ci-cd.md: the A01 PR is the first run. Admin bypass covers a misnamed check | [ci-cd.md](../../engineering/ci-cd.md#required-status-checks) |
| Labels synced once with `gh` instead of a workflow | Enough until the taxonomy changes; a sync workflow stays optional | No ADR needed |
| Keep `publish = false` until A07 | Nothing is ready for crates.io; release-plz arrives in A07 | No ADR needed |

## Problems and how they were solved

| Problem | Cause | Resolution |
|---|---|---|
| `reuse` failed to start (encoding module) | The `reuse` package needs an optional dependency on Windows | Ran it as `uvx --from "reuse[charset-normalizer]" reuse` |
| `reuse lint` scanned pnpm's per-package `node_modules` | On Windows the tool did not apply git's ignore rules to those folders | Linted a clean copy of the committable files: 167/167 compliant. CI runs on Ubuntu without `node_modules` |

## Measurements

| What | Value | Conditions | Logged in |
|---|---|---|---|
| Local CI suite (`cargo xtask ci`) | Passes | Windows 11, Rust 1.99.0 | Pending the first GitHub run |

## Next steps

- [ ] Watch the A01 PR's first CI run and fix failures (agent).
- [ ] Review and merge the A01 PR (@DuhItzAniket).
- [ ] Enable reported-content moderation in the repository's moderation settings (web UI only) (@DuhItzAniket).
- [ ] A01-T10: GitHub Project board and Stage A issues (agent, with the owner's go-ahead).
- [ ] Start A02 spikes and A03 core types after merge.
