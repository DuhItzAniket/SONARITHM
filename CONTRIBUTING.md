# Contributing to SONARITHM

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> Related: [README](README.md) · [AGENTS.md](AGENTS.md) · [Governance](GOVERNANCE.md) · [Code of conduct](CODE_OF_CONDUCT.md) · [Security policy](SECURITY.md) · [Git workflow](docs/engineering/git-workflow.md) · [Definition of Ready and Done](docs/sdlc/definition-of-ready-and-done.md)

SONARITHM turns graphs, charts and numerical data into structured, navigable sound. It is one deterministic Rust core shared by a Tauri 2 desktop app, a CLI and Python bindings ([architecture overview](docs/architecture/01-overview.md)).

**Project state (2026-10-18).** Only documentation exists. Phase A01 of the [roadmap](docs/planning/roadmap.md) creates the Cargo workspace, CI and repository settings. Commands marked *available after A01* (or a later phase) do not work yet.

Everyone who takes part agrees to the [code of conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

| Contribution | Where to start | Notes |
|---|---|---|
| Code: Rust core, engines, CLI, Python bindings, desktop UI | Issues labelled `status/ready`, `good first issue` or `help wanted` | Read [AGENTS.md](AGENTS.md) and the [coding standards](docs/engineering/coding-standards.md) first. |
| Accessibility testing: screen-reader, braille, magnifier, switch, voice-control and keyboard-only users, especially blind and low-vision (BLV) people | [Accessibility issue form](https://github.com/DuhItzAniket/SONARITHM/issues/new?template=accessibility_issue.yml) | No coding needed. See [Testing with assistive technology](#testing-with-assistive-technology). |
| Research: findings, hypotheses, replications of classic designs, design-rule changes | [Research question form](https://github.com/DuhItzAniket/SONARITHM/issues/new?template=research_question.yml) | See [Research contributions](#research-contributions). |
| Documentation | Issues labelled `kind/docs` | The docs follow Diátaxis ([docs index](docs/README.md)). |
| Triage and review | Issues labelled `status/needs-triage`; open pull requests | Reproducing a bug on another OS, or confirming what a screen reader announces, is real work and is credited. |
| Example datasets and specs | Open a feature request first | Datasets are licensed separately from code (CC BY 4.0, see [supply chain and licensing](docs/engineering/supply-chain-and-licensing.md)). Never personal data. |
| Security reports | [SECURITY.md](SECURITY.md) | Never in public issues, discussions or pull requests. |

### Testing with assistive technology

- Report what you hear, feel on a braille display, or cannot reach, with your own setup. The [accessibility issue form](https://github.com/DuhItzAniket/SONARITHM/issues/new?template=accessibility_issue.yml) asks for the assistive technology and version, OS, braille display, steps, and the expected versus actual announcement.
- If the issue form is hard to use with your setup, start a [discussion](https://github.com/DuhItzAniket/SONARITHM/discussions) in plain text. A maintainer will turn it into an issue and credit you if you want.
- Until the desktop app exists (Stage B), useful testing covers the CLI, the Python package in notebooks, and whether the documentation reads well with a screen reader.
- **Formal co-design is paid.** Co-design rounds (roadmap B07 and C06) are compensated and run under ethics approval ([A11Y-34](docs/research/design-rules.md)). Volunteer reports through issues are welcome, but they never replace paid study participation, and they are product feedback, not research data: they are not analysed for publications.
- With your agreement, release notes credit you by GitHub handle.

## Development setup

The toolchains (Rust pinned by `rust-toolchain.toml`, Node with pnpm, Python with maturin, and the Tauri platform dependencies) are described in [docs/engineering/dev-environment.md](docs/engineering/dev-environment.md). [AGENTS.md](AGENTS.md) holds the canonical build, test and lint commands.

The local checks every PR must pass (*available after A01*):

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --doc --locked
cargo deny check
cargo xtask check-deps
```

`cargo xtask ci` runs the local equivalent of all required checks in one command (*available after A01*). Frontend checks are added in B01 and Python checks in A07. [CI/CD](docs/engineering/ci-cd.md) lists the full set by phase.

## Workflow

1. **Start from an issue.** Typos and small doc fixes can go straight to a PR. Anything else needs an issue that meets the [Definition of Ready](docs/sdlc/definition-of-ready-and-done.md). Say in the issue that you want to work on it, and a maintainer will assign it. Architectural or cross-cutting work needs an ADR or RFC first (see [Where decisions live](#where-decisions-live)).
2. **Branch.** External contributors work on a fork. Name branches `<kind>/<phase>-<short-desc>`, for example `feat/a06-mel-scale`. See [git workflow](docs/engineering/git-workflow.md#branches).
3. **Commit** with Conventional Commit messages and a DCO sign-off (below).
4. **Open a pull request** using the template. The PR title must be a Conventional Commit header, because it becomes the squash-merge commit and feeds the changelog. Link the issue and the roadmap phase ID, and the task ID from the [implementation plan](docs/planning/implementation-plan.md) if there is one (for example `Implements A03-T06`). Draft PRs are welcome for early feedback.
5. **CI and review.** All required checks must pass. A maintainer reviews and squash-merges.

## Commit messages

SONARITHM uses [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

```text
<type>(<scope>)!: <description in imperative mood, lower case, no full stop>

<body: why the change is needed, what it affects>

<footers: Closes #123, BREAKING CHANGE: ..., Signed-off-by: ...>
```

| Type | Use for |
|---|---|
| `feat` | A user-visible capability |
| `fix` | A bug fix |
| `perf` | A performance improvement with no behaviour change |
| `refactor` | Restructuring with no behaviour change |
| `docs` | Documentation only |
| `test` | Tests, fixtures, golden files |
| `build`, `ci`, `chore` | Build system, workflows, maintenance |
| `revert` | Reverting an earlier commit |

**Scopes** are crate areas or lowercase engine ids:

- Crate areas: `core`, `spec`, `data`, `compiler`, `dsp`, `render`, `audio`, `export`, `stream`, `speech`, `app`, `facade`, `cli`, `desktop`, `ui`, `viz`, `ipc`, `python`, `wasm`, `plugins`, `xtask`, `schemas`.
- Engines: `numisona`, `audaxis`, `sonotrace`, `plotsona`, `curvasona`, `axonote`, `echometric`, `tonalens`, `resonum` ([engine catalogue](docs/product/engines.md)).
- Repository: `ci`, `deps`, `release`, `docs`, `adr`, `rfc`, `research`, `design`, `a11y`.

Examples: `feat(numisona): add mel pitch unit`, `fix(render): ramp voice release to avoid clicks`, `docs(research): log finding on tick-track density`.

Mark breaking changes with `!` after the scope and a `BREAKING CHANGE:` footer that explains the migration. The full scope table, label mapping and rules are in [git workflow](docs/engineering/git-workflow.md#commit-messages).

## Developer Certificate of Origin (DCO)

Every commit must carry a `Signed-off-by:` line. It certifies that you have the right to submit the change under the project's licence, as set out in the [Developer Certificate of Origin 1.1](https://developercertificate.org/). The project uses a DCO, not a CLA.

```sh
git commit -s -m "fix(spec): reject negative durations"
# Add sign-offs to every commit already on your branch:
git rebase --signoff origin/main
```

The sign-off uses the name and address from your git configuration. To keep your personal address private, configure git with the no-reply address GitHub provides in your email settings. The DCO check (*available after A01*) fails a PR that has an unsigned commit.

A sign-off is a human certification. An AI agent must not add a `Signed-off-by:` line for a person who has not reviewed the change.

## Licence of contributions

By signing off, you agree that your contribution is licensed under the project's licence, inbound equal to outbound. Per [ADR-0010](docs/decisions/0010-licensing.md), code is licensed `MIT OR Apache-2.0`, and documentation, example datasets and study stimuli are licensed `CC-BY-4.0`. New source files carry the SPDX header `SPDX-License-Identifier: MIT OR Apache-2.0` and `SPDX-FileCopyrightText: 2026 SONARITHM contributors` (use the current year). See [supply chain and licensing](docs/engineering/supply-chain-and-licensing.md).

Do not copy code from projects under incompatible licences. In particular, PLOTSONA's raster digitizer re-implements WebPlotDigitizer's documented behaviour; WebPlotDigitizer's AGPL source must never be copied.

## Pull request expectations

- **One concern per PR.** Aim for about 400 changed lines or fewer, excluding generated files, lockfiles, snapshots and golden files. Split larger work into a sequence of PRs.
- **Definition of Done.** The PR template checklist is the [Definition of Done](docs/sdlc/definition-of-ready-and-done.md): tests, docs, a changelog-worthy title, ADR or RFC links, golden-file reasons, accessibility checks, real-time rules, dependency licences, research findings and status updates. Mark items that do not apply as N/A; do not delete them.
- **Generated files are regenerated, never edited by hand.** This covers `schemas/` (from A03) and `packages/ipc/src/generated/` (from B01).
- **Golden files change only on purpose.** Regenerate them with `cargo xtask golden --update` (*available after A05*), listen to audible changes, and state in the PR why the expected output changed ([testing strategy](docs/engineering/testing-strategy.md)).
- **AI-authored PRs** are reviewed by a human maintainer like any other PR.
- **Describe how it sounds.** For changes that affect audio, say what a listener hears before and after, or state "no audible change".
- **Keep the branch green.** Fix failing checks; do not ask reviewers to merge around them.

## Review and response targets

These are targets, not guarantees. While the project has one maintainer they are best effort, and they are reviewed at each stage gate.

| Event | Target |
|---|---|
| First response to a new issue (triage label or question) | 5 business days |
| First review of a ready (non-draft) PR | 5 business days |
| Follow-up review after you push changes | 3 business days |
| Security reports | See [SECURITY.md](SECURITY.md#response-targets) |
| PR with no author activity | Maintainers may close it after 30 days; you can reopen it |

Review rules: one maintainer approval is required, plus a code-owner approval for paths that [CODEOWNERS](.github/CODEOWNERS) marks, which include the core, real-time audio and accessibility-critical code. Details are in [git workflow](docs/engineering/git-workflow.md#review-rules), which also describes the current fast-track: the maintainer's own PRs merge on green checks without a review.

## Accessibility expectations for UI changes

Accessibility is architecture in SONARITHM ([principle P5](docs/architecture/01-overview.md#2-architectural-principles)). A UI change is not done until:

- It uses or extends the React Aria-based components in `@sonarithm/ui` instead of hand-rolled widgets.
- Every function is reachable and operable by keyboard alone, with a visible focus indicator and no keyboard trap. Single-key shortcuts can be switched off or remapped (WCAG 2.1.4).
- Status changes are announced without moving focus (WCAG 4.1.3). Speech and sonification are sequenced, not overlapped ([A11Y-20](docs/research/design-rules.md)).
- No information is carried only by colour, pitch, pan or position ([A11Y-14](docs/research/design-rules.md), WCAG 1.3.3). Every sonified insight has a non-audio equivalent ([A11Y-24](docs/research/design-rules.md)).
- High-contrast and forced-colours themes, text scaling and reduced motion are respected ([A11Y-25](docs/research/design-rules.md)).
- You have tested it with the keyboard and with a screen reader: changed components with NVDA (Windows) or VoiceOver (macOS), new components and custom widgets with both, and Orca for Linux-specific changes. The axe checks in Storybook and the affected Playwright flows report no serious or critical violations. Record what you tested in the PR.

The complete UI checklist (contrast ratios, forced colours, reduced motion, strings, stories) is in the [Definition of Done §3.2](docs/sdlc/definition-of-ready-and-done.md#32-user-interface).

The full screen-reader matrix (NVDA, JAWS, Narrator, VoiceOver, Orca on X11 and Wayland, and a braille display) runs per release, not per PR ([A11Y-32](docs/research/design-rules.md), [accessibility conformance plan](docs/design/accessibility-conformance-plan.md)).

## Real-time audio and determinism

- **The audio thread is sacred** ([principle P3](docs/architecture/01-overview.md#2-architectural-principles)): no allocation, locks, I/O, logging or unbounded work in the real-time callback. Debug builds and the `no-alloc` CI job enforce the allocation rule with `allocation-counter` (A05).
- **Renders are deterministic** ([SDR-01, SDR-02](docs/research/design-rules.md)): no wall-clock time, unseeded randomness or thread-order-dependent mixing in the compiler or offline renderer. See [testing strategy](docs/engineering/testing-strategy.md).

## Research contributions

SONARITHM is also a research instrument. Research contributions follow the same PR workflow, plus:

- **Findings** go into the [findings log](docs/research-paper/findings-log.md) with the next free `F-###` ID, the evidence (citation with DOI or URL, or a SONARITHM measurement with its spec and render manifest), and the affected rule IDs.
- **Design-rule changes** (`SDR-##`, `A11Y-##`) are made only through a PR that cites evidence, and rules are never renumbered ([design rules](docs/research/design-rules.md)). A change to a default states the old value and the new one.
- **Hypotheses and studies** are discussed through the research question form first. Any study that involves people needs ethics approval *before* any data collection.
- **Never commit participant data** or anything that could identify a participant. Study data lives outside this repository under the approved data-management plan.
- The research-paper track is described in [docs/research-paper/README.md](docs/research-paper/README.md).

## AI-assisted contributions

AI coding tools are allowed. Disclosure is required, because the project publishes in venues such as JOSS that require an AI usage disclosure ([research notes 07 §6](docs/research/notes/07-engineering-publication.md)).

- **Disclose** in the PR template which tools you used and for what (for example "generated the first draft of the tests" or "suggested the refactor"). A `Co-authored-by:` trailer for the tool is optional.
- **You are the author.** You have read, understood and tested every line, and you can explain it in review. Core design decisions are made by people.
- **Licences still apply.** Do not submit generated output that reproduces code from incompatible sources.
- **Agents follow [AGENTS.md](AGENTS.md).** Agent-authored commits are signed off only by the human who reviewed them (see [DCO](#developer-certificate-of-origin-dco)).
- **No sensitive data in third-party AI services.** Never paste participant data, security reports or unpublished vulnerability details into an AI service.
- Maintainers may use AI tools to help review, but approval is a human decision.

The research-paper track aggregates these disclosures for each paper's AI usage statement.

## Where decisions live

| What | Where | Process |
|---|---|---|
| Architecture decisions (technology choices, new crates or top-level directories, significant dependencies) | [docs/decisions/](docs/decisions/README.md), MADR 4 format | ADR PR, see [governance](GOVERNANCE.md#decision-making) |
| Spec format, plugin API, file formats, a new engine, cross-cutting changes | [docs/rfcs/](docs/rfcs/README.md) | RFC PR with a 7-day final comment period, then an ADR records the outcome |
| Design rules | [docs/research/design-rules.md](docs/research/design-rules.md) | PR citing evidence |
| Open questions | [docs/planning/open-decisions.md](docs/planning/open-decisions.md) | Resolved by ADR, RFC or PR |
| Scope and schedule | [roadmap](docs/planning/roadmap.md), [status](docs/sdlc/status.md) | PR editing both |
| Who decides | [GOVERNANCE.md](GOVERNANCE.md) | |

## Getting help

- Usage questions and ideas: [GitHub Discussions](https://github.com/DuhItzAniket/SONARITHM/discussions).
- Bugs, features, accessibility problems and research questions: the [issue forms](https://github.com/DuhItzAniket/SONARITHM/issues/new/choose).
- Vulnerabilities: [SECURITY.md](SECURITY.md).
