# Git workflow

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> Related: [CONTRIBUTING.md](../../CONTRIBUTING.md) · [GOVERNANCE.md](../../GOVERNANCE.md) · [CI/CD](ci-cd.md) · [Release process](release-process.md) · [Definition of Ready and Done](../sdlc/definition-of-ready-and-done.md) · [CODEOWNERS](../../.github/CODEOWNERS)

This document defines how changes reach `main`: branches, commit messages, sign-off, review, merging and tags. The rationale is in [research notes 07 §1](../research/notes/07-engineering-publication.md) and the [research report](../research/research-report.md#process-and-release-automation). Repository settings described here are applied in phase A01.

## Summary

| Topic | Rule |
|---|---|
| Model | Trunk-based development on `main` |
| Branches | Short-lived, named `<kind>/<phase>-<short-desc>` |
| Commits | Conventional Commits 1.0.0, DCO sign-off on every commit |
| PR title | A Conventional Commit header; it becomes the squash commit |
| Merge | Squash merge only; linear history |
| Review | One maintainer approval; code-owner approval on core, real-time audio and accessibility-critical paths. Suspended during the [fast-track](#review-rules) |
| Release branches | None before 1.0; after 1.0 only for patch backports |
| Tags | `vX.Y.Z[-pre]` for the workspace; per-crate tags after 1.0 |
| Signing | Commit signing recommended, not required |

## Trunk-based development

`main` is the only long-lived branch, and it is always releasable. Work happens on short-lived branches that merge back within days. Incomplete features stay behind feature flags or unexposed APIs instead of on long-running branches. GitFlow's long-lived `develop` branch is not used; its author no longer recommends GitFlow for continuously delivered software ([notes 07 §1](../research/notes/07-engineering-publication.md)).

The `main` ruleset (A01):

- Changes arrive only through pull requests; direct pushes are blocked.
- Required status checks must pass ([CI/CD](ci-cd.md#required-status-checks)).
- Linear history is required; force pushes and deletion are blocked.
- Code-owner review is required for the paths listed in [CODEOWNERS](../../.github/CODEOWNERS) (suspended during the [fast-track](#review-rules)).
- Merge commits and rebase merges are disabled in repository settings; only squash merging is allowed.
- Branches are deleted automatically after merge.

## Branches

Name branches `<kind>/<phase>-<short-desc>`:

- `<kind>` is a Conventional Commit type (`feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`), or `spike` for throwaway A02 prototypes.
- `<phase>` is the lowercase roadmap phase ID from the [roadmap](../planning/roadmap.md). Omit it for work outside a phase.
- `<short-desc>` is two to five lowercase words joined by hyphens.

| Example | Meaning |
|---|---|
| `feat/a06-mel-scale` | NUMISONA mel pitch unit, phase A06 |
| `fix/b03-focus-after-dialog` | AUDAXIS focus bug, phase B03 |
| `spike/a02-cpal-loopback` | Throwaway latency spike |
| `chore/bump-actions` | Maintenance outside a phase |

Bot branches (`dependabot/...`, release-plz's release branches) are exempt.

Keep branches short-lived: a few days is the target. Update a branch with `git rebase origin/main` rather than merging `main` into it. Because merges are squashed, intermediate history on the branch does not reach `main`.

## Commit messages

All commits follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/). The PR title is checked by `pr-title.yml` ([CI/CD](ci-cd.md#workflow-inventory)), and because it becomes the squash commit, it is the message release-plz and git-cliff read.

```text
<type>(<scope>)!: <description>

<body>

<footers>
```

- **Description:** imperative mood, lower case, no full stop, header at most 72 characters.
- **Body:** why the change is needed and what it affects. Wrap at 72 characters.
- **Footers:** `Closes #123`, `Refs #123`, `BREAKING CHANGE: <migration>`, `Co-authored-by: ...`, `Signed-off-by: ...`.

### Types

The changelog mapping is the intended git-cliff configuration; `cliff.toml` is created in A07.

| Type | Use for | Changelog section |
|---|---|---|
| `feat` | A user-visible capability | Added |
| `fix` | A bug fix | Fixed |
| `perf` | Performance improvement, no behaviour change | Changed |
| `refactor` | Restructuring, no behaviour change | Hidden |
| `docs` | Documentation only | Hidden, unless the scope is user-facing docs |
| `test` | Tests, fixtures, golden files, benchmarks | Hidden |
| `build` | Build system, Cargo or pnpm configuration | Hidden |
| `ci` | Workflows | Hidden |
| `chore` | Maintenance | Hidden |
| `revert` | Reverts an earlier commit | Matches the reverted entry |

Security fixes use `fix` and are also listed under the `Security` section of the release's changelog entry.

### Scopes

A scope is optional for repository-wide changes. Use one scope per commit; if a change spans several areas, choose the main one or omit the scope. Scopes map to `area/*` and `engine/*` labels ([labels.yml](../../.github/labels.yml)).

| Scope | Covers | Label |
|---|---|---|
| `core` | `crates/sonarithm-core` | `area/core` |
| `facade` | `crates/sonarithm` (public facade) | `area/core` |
| `spec` | `crates/sonarithm-spec` | `area/spec` |
| `schemas` | Generated `schemas/` | `area/spec` |
| `data` | `crates/sonarithm-data` | `area/data` |
| `compiler` | `crates/sonarithm-compiler` | `area/compiler` |
| `dsp` | `crates/sonarithm-dsp` | `area/dsp` |
| `render` | `crates/sonarithm-render` | `area/render` |
| `audio` | `crates/sonarithm-audio` | `area/audio` |
| `speech` | `crates/sonarithm-speech` | `area/audio` |
| `export` | `crates/sonarithm-export` | `area/export` |
| `stream` | `crates/sonarithm-stream` | `area/stream` |
| `app` | `crates/sonarithm-app` | `area/app` |
| `desktop` | `apps/desktop/src-tauri` (Tauri host) | `area/app` |
| `ui` | `apps/desktop/src`, `packages/ui` | `area/desktop-ui` |
| `viz` | `packages/viz` | `area/desktop-ui` |
| `ipc` | `packages/ipc`, IPC command definitions | `area/desktop-ui` |
| `cli` | `apps/cli` | `area/cli` |
| `python` | `bindings/python` | `area/python` |
| `wasm` | `bindings/wasm` | `area/wasm` |
| `plugins` | `crates/sonarithm-plugin-host`, `wit/` | `area/plugins` |
| `xtask` | `xtask/` | `area/ci` |
| `ci` | `.github/workflows/`, CI configuration | `area/ci` |
| `release` | Release configuration and release PRs | `area/ci` |
| `deps` | Dependency updates | `dependencies` |
| `docs` | Documentation structure and governance files | `area/docs` |
| `adr` | `docs/decisions/` (for example `docs(adr): 0014 ...`) | `area/docs` |
| `rfc` | `docs/rfcs/` | `area/docs` |
| `research` | `docs/research/`, `docs/research-paper/` | `research` |
| `design` | `docs/design/` | `area/docs` |
| `a11y` | Cross-cutting accessibility changes | `a11y` |
| `numisona`, `audaxis`, `sonotrace`, `plotsona`, `curvasona`, `axonote`, `echometric`, `tonalens`, `resonum` | The engine crate `crates/engines/sonarithm-<engine>` (PLOTSONA also covers `sonarithm-plotsona-ml`) | `engine/<engine>` |

Engine scopes use the lowercase engine id, as in specs and crate names ([engine catalogue](../product/engines.md#naming-and-usage-rules)).

### Breaking changes

Mark a breaking change with `!` after the scope **and** a `BREAKING CHANGE:` footer that tells users how to migrate:

```text
feat(spec)!: rename `tracks[].channel` to `tracks[].channels`

BREAKING CHANGE: specs using `channel` must rename it to `channels`.
The v0.2 migration rewrites old specs automatically.

Signed-off-by: ...
```

Apply the `breaking-change` label to the PR. Before 1.0, breaking changes are allowed in minor releases but must always be marked and documented ([release process](release-process.md#versioning-policy)). Changes to the spec format also follow the spec's own versioning policy, which is independent of crate versions ([crate layout §6](../architecture/02-repository-and-crate-layout.md#6-versioning)).

## DCO sign-off

Every commit carries a `Signed-off-by:` trailer certifying the [Developer Certificate of Origin 1.1](https://developercertificate.org/). The DCO check (A01) inspects every commit in a PR.

```sh
git commit -s                      # sign off a new commit
git commit --amend -s --no-edit    # sign off the last commit
git rebase --signoff origin/main   # sign off every commit on the branch
git push --force-with-lease        # update your own branch after rewriting it
```

Force pushing is fine on your own feature branch and never allowed on `main`.

The repository's squash-merge message setting is "pull request title and commit details", so the sign-offs of squashed commits are preserved in the commit on `main` (A01).

AI agents may prepare commits, but the `Signed-off-by:` trailer is added only by the human who reviewed the change ([AI policy](../../CONTRIBUTING.md#ai-assisted-contributions)).

## Pull request size

| Changed lines (excluding generated files, lockfiles, snapshots, golden files) | Guidance |
|---|---|
| Up to about 400 | Normal |
| About 400 to 800 | Explain in the summary why it cannot be split |
| More than about 800 | Split it, unless the change is mechanical (a rename or a generated update) |

Open a draft PR early to get feedback on direction. A sequence of small dependent PRs is better than one large one; say in each PR which one it builds on.

## Review rules

- **Fast-track (owner decision, 2026-10-19).** To finish Stage A's code quickly, the `main` ruleset requires no approving review and no code-owner review. The required status checks, linear history, DCO sign-off and resolved conversations still gate every merge, and PRs merge through GitHub auto-merge (squash). For this push, one PR per task replaces the size guideline above. Only people with write access can merge, so external PRs still need the lead. The rules below apply again when the owner ends the fast-track.
- **Approvals.** One approval from a maintainer.
- **Code owners.** Paths that [CODEOWNERS](../../.github/CODEOWNERS) assigns need their owner's approval. The ruleset requires it for:
  - core and compiler crates: `sonarithm-core`, `sonarithm-spec`, `sonarithm-compiler`, `schemas/`;
  - real-time audio crates: `sonarithm-dsp`, `sonarithm-render`, `sonarithm-audio`;
  - accessibility-critical code: `sonarithm-audaxis`, `sonarithm-speech`, `apps/desktop/`, `packages/ui/`;
  - supply chain and policy: `.github/`, `deny.toml`, `SECURITY.md`, `GOVERNANCE.md`.
- **Stale approvals** are dismissed when new commits are pushed.
- **Conversations** must be resolved before merging.
- **Interim rule.** GitHub does not allow authors to approve their own PRs. While the project lead is the only maintainer, the lead merges their own PRs through the ruleset's bypass list after all required checks pass. External PRs always need the lead's approval ([governance](../../GOVERNANCE.md#decision-making)). This lowers the OpenSSF Scorecard Code-Review score until a second maintainer joins.

What reviewers check, in addition to correctness:

| Check | Reference |
|---|---|
| Definition of Done items are met or justified as N/A | [Definition of Ready and Done](../sdlc/definition-of-ready-and-done.md) |
| Tests cover the behaviour; golden-file changes have a stated reason | [Testing strategy](testing-strategy.md) |
| No allocation, locks, I/O or logging on the audio thread | [Architecture principle P3](../architecture/01-overview.md#2-architectural-principles) |
| Keyboard, screen-reader and axe checks were done for UI changes | [CONTRIBUTING](../../CONTRIBUTING.md#accessibility-expectations-for-ui-changes) |
| New dependencies have permitted licences and, if significant, an ADR | [Supply chain and licensing](supply-chain-and-licensing.md) |
| Code follows the standards | [Coding standards](coding-standards.md) |

Response targets are in [CONTRIBUTING](../../CONTRIBUTING.md#review-and-response-targets).

## Merging

- **Squash merge only.** The squash commit message is the PR title plus the commit details. Edit the title before merging if it is not a good changelog line.
- **Auto-merge** may be enabled on a PR once it is approved; it merges when the checks pass.
- **Merge queue (later).** When several maintainers merge daily and "require branches to be up to date" becomes a bottleneck, enable GitHub's merge queue. It needs the repository to be owned by an organisation; check the current GitHub plan rules when adopting it. `ci.yml` listens for the `merge_group` event from the start so the switch needs no workflow change.

## Release branches

- **Before 1.0:** none. Releases are cut from `main`, which is always releasable.
- **After 1.0:** a `release/vX.Y` branch is created from the release tag only when a patch must be backported to the latest stable minor and `main` has moved on. Fixes land on `main` first and are cherry-picked with `git cherry-pick -x`. See the [hotfix procedure](release-process.md#hotfix-procedure).

## Tags

| Tag | Used for | From |
|---|---|---|
| `vX.Y.Z[-pre]`, for example `v0.1.0-alpha` | The workspace release. All library crates share one version before 1.0 ([crate layout §6](../architecture/02-repository-and-crate-layout.md#6-versioning)); one tag covers the crates, CLI, Python package and, from Stage B, the desktop app | A07 |
| `<crate>-vX.Y.Z` | Per-crate releases after 1.0, created by release-plz when crates version independently | After 1.0 |

- Tags are created by release automation from the merged release PR, never by hand on an arbitrary commit ([release process](release-process.md)).
- Release tags are protected by the ruleset, and stable releases are immutable once published.
- Research notes 07 §3 suggest a separate `app-v*` tag for desktop releases. It is adopted only if the desktop app needs its own release cadence; that is decided before the first desktop beta (B07).
- The spec version (`specVersion`) is recorded in the spec and schema, not as a git tag.

## Commit signing

Signing commits is **recommended** and not required, so that first-time contributors are not blocked. Maintainers are expected to sign. Squash merges made through the GitHub web interface are signed by GitHub.

```sh
# SSH signing (git 2.34 or later)
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

Add the same key to GitHub as a signing key so that commits show as verified. Requiring signed commits on `main` is reconsidered when the maintainer council forms.

## Quick reference

```sh
git switch -c feat/a06-mel-scale origin/main
# ...edit, test...
git commit -s -m "feat(numisona): add mel pitch unit"
git fetch origin && git rebase origin/main
git push -u origin feat/a06-mel-scale
# Open a PR with a Conventional Commit title and complete the template.
```
