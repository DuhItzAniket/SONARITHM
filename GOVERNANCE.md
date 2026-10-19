# Governance

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> Related: [Contributing](CONTRIBUTING.md) · [Code of conduct](CODE_OF_CONDUCT.md) · [Security policy](SECURITY.md) · [Decisions (ADRs)](docs/decisions/README.md) · [RFCs](docs/rfcs/README.md) · [Roadmap](docs/planning/roadmap.md) · [CODEOWNERS](.github/CODEOWNERS)

This document describes who makes decisions in SONARITHM, how they are made and recorded, and how the model changes as the project grows.

## Current model

During **Stages A and B** (roadmap phases A01 to B07), SONARITHM has a **project lead** with final say on technical and community decisions. This is a BDFL-style model: it keeps decisions fast while the project has one core developer. Its obligations are to decide in public, record decisions in the repository, and explain them.

The project moves to a **maintainer council** when it has at least three maintainers, including the project lead. Each gate review from Gate B onward records whether that condition holds. The transition is a governance change (see [Changes to this document](#changes-to-this-document)).

### Principles

1. **Decisions are public and written down.** Discussion happens in issues, pull requests, discussions, ADRs and RFCs. A decision made elsewhere is recorded in the repository before it takes effect.
2. **Evidence over preference.** Defaults trace to the [design rules](docs/research/design-rules.md) and their evidence. Disagreements about sonification design are settled with data where possible.
3. **Nothing about blind and low-vision users without them.** Decisions that change how BLV people use SONARITHM are made with BLV co-design partners consulted.
4. **Sustainability.** The project avoids single points of failure in people, keys and accounts as early as it can (see [Continuity](#continuity)).

## Roles

| Role | Responsibilities | Authority |
|---|---|---|
| **Project lead** | Sets the roadmap and stage gates; final decision during Stages A and B; release manager; security contact; code-of-conduct moderator; repository administrator; one of the two Tauri updater key custodians | Final say until the maintainer council exists; then a council member |
| **Maintainers** | Review and merge PRs; triage; own areas through [CODEOWNERS](.github/CODEOWNERS); run releases when delegated; moderate under the [code of conduct](CODE_OF_CONDUCT.md) | Merge rights; approve PRs; vote on the council once it exists |
| **Architecture lead** | Owns the architecture documents ([overview](docs/architecture/01-overview.md)) and the coherence of ADRs | Approves ADRs, with the project lead during Stages A and B |
| **Research lead** | Owns the [design rules](docs/research/design-rules.md), the [findings log](docs/research-paper/findings-log.md), the research-paper track and study ethics | Approves design-rule changes, study protocols and research claims in docs and papers |
| **Accessibility lead** | Owns the [accessibility conformance plan](docs/design/accessibility-conformance-plan.md), the accessibility conformance report (ACR) and the screen-reader test matrix; coordinates co-design | Can block a merge or release that introduces a critical accessibility regression; approves A11Y rule changes with the research lead |
| **Contributors** | Anyone who files issues, reviews, tests, writes code, docs or research | Proposes changes; takes part in all public discussion |
| **BLV co-design partners** | Blind and low-vision people who co-design and evaluate SONARITHM in paid, ethics-approved rounds ([A11Y-34](docs/research/design-rules.md)) | Consulted on decisions that affect accessibility and interaction; their findings are logged and triaged; credited only with consent and in the form they choose |

One person can hold several roles. A vacant lead role is held by the project lead in the interim.

### Current role holders

| Role | Holder |
|---|---|
| Project lead | [@DuhItzAniket](https://github.com/DuhItzAniket) |
| Maintainers | [@DuhItzAniket](https://github.com/DuhItzAniket) |
| Architecture lead | Interim: project lead |
| Research lead | Interim: project lead |
| Accessibility lead | Vacant (interim: project lead). Preferably filled by a person with lived experience of assistive technology. |
| BLV co-design partners | Recruited for co-design round 1 (phase B07). Names are not published without consent. |

## Decision-making

The default is **lazy consensus**: a proposal in a PR or issue goes ahead if no maintainer objects within the review window, and an objection must give reasons and, where possible, an alternative. Larger decisions use the written processes below.

| Decision | Mechanism | Approved by (Stages A–B) | Approved by (council) | Record |
|---|---|---|---|---|
| Routine change: bug fix, feature within an accepted design, docs | Pull request | One maintainer, plus a code owner where [CODEOWNERS](.github/CODEOWNERS) requires | Same | Squash commit and changelog |
| Architecture: technology choice, new crate or top-level directory, significant dependency, change to an accepted ADR | ADR in [docs/decisions/](docs/decisions/README.md), MADR 4 format, open at least 5 business days | Project lead, consulting the architecture lead | Council, lazy consensus | ADR status `accepted` |
| Spec format, plugin API (WIT world), file formats, a new engine, other cross-cutting changes | RFC in [docs/rfcs/](docs/rfcs/README.md); 7-day final comment period | Project lead | Council, lazy consensus; majority if contested | Merged RFC, then an ADR records the outcome |
| Design rule (`SDR-##`, `A11Y-##`) | PR citing evidence ([change control](docs/research/design-rules.md)) | Research lead; the accessibility lead also approves A11Y rules | Same | design-rules.md and the findings log |
| Roadmap scope or schedule | PR editing the [roadmap](docs/planning/roadmap.md) and [status](docs/sdlc/status.md) ([roadmap §7](docs/planning/roadmap.md#7-how-to-change-this-roadmap)) | Project lead | Council | Roadmap, plus a note in the next gate review |
| Stage gate passed | Gate review in `docs/sdlc/reviews/` | Project lead | Council | Gate review document |
| Release | Release PR ([release process](docs/engineering/release-process.md)) | Release manager (project lead) | Release manager, with a second approval through the release environment | Tag, changelog, GitHub release |
| Security fix | Private advisory ([SECURITY.md](SECURITY.md)) | Project lead | Any maintainer with advisory access | Published advisory |
| Code-of-conduct enforcement | [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Moderator | Uninvolved maintainers | Private record |
| Governance change | PR to this file, open at least 14 days | Project lead | Two-thirds of maintainers | This file |

Never add a tenth engine without an ADR ([engine catalogue](docs/product/engines.md#naming-and-usage-rules)); because it is cross-cutting, it also needs an RFC.

**Interim review rule.** GitHub does not let authors approve their own pull requests. While the project lead is the only maintainer, the lead merges their own PRs after all required checks pass, through the repository ruleset's bypass list. External contributions always need the lead's approval. This exception ends when a second maintainer joins. Since 2026-10-19 the owner has also suspended required reviews for a fast-track ([git workflow](docs/engineering/git-workflow.md#review-rules)); external contributions still need the lead, who alone can merge them.

## Becoming a maintainer

**Criteria.** Sustained, high-quality contribution over about three months, in any form: code, reviews, accessibility testing, research or documentation. Sound judgement on the [architectural principles](docs/architecture/01-overview.md#2-architectural-principles) and design rules. Consistent conduct under the code of conduct. Enough availability to review regularly.

**Process.**

1. A maintainer nominates the candidate in a PR that adds them to [Current role holders](#current-role-holders) and, where relevant, to CODEOWNERS.
2. The candidate accepts in the PR.
3. During Stages A and B the project lead approves. Under the council, maintainers decide by lazy consensus over 7 days; any objection is discussed privately among maintainers first.

**Onboarding.** Two-factor authentication on GitHub is required. Access follows least privilege: write access first, administrator access only when needed. Signed commits are recommended ([git workflow](docs/engineering/git-workflow.md#commit-signing)).

**Emeritus.** A maintainer inactive for six months, or who steps down, becomes emeritus. Access is removed and the contribution remains credited. Returning is by the normal process, shortened at the maintainers' discretion.

## Conflict resolution

1. **Discuss** in the issue or PR, focusing on the problem and the evidence: benchmarks, design rules, user research and the [risk register](docs/planning/risk-register.md).
2. **Write it down.** If the disagreement is architectural or cross-cutting, open an ADR or RFC that sets out the options.
3. **Mediate.** An uninvolved maintainer, when one exists, summarises the positions and proposes a resolution.
4. **Decide.** During Stages A and B the project lead decides. Under the council, the council decides by lazy consensus, falling back to a simple majority of maintainers.
5. **Record** the decision and its reasoning in the ADR, RFC or PR.

Conduct problems are not technical disagreements; they go through the [code of conduct](CODE_OF_CONDUCT.md).

## Maintainer council (planned)

When the trigger in [Current model](#current-model) is met:

- The council consists of all active maintainers. If it grows beyond seven, it switches to an elected council of five; the election process is defined in that governance change.
- It decides by lazy consensus, falling back to a simple majority. Governance changes need two thirds.
- The project lead becomes a council member with no casting vote, unless the council decides otherwise when it forms.
- The repository moves to a GitHub organisation if it has not already. Organisation-level rulesets, team handles in CODEOWNERS and the merge queue need one.

## Continuity

- A second Tauri updater key custodian is appointed before the first updater-enabled release (phase C06) ([key custody](docs/engineering/ci-cd.md#tauri-updater-key-custody)).
- At least two people hold administrator access to the repository, the crates.io and PyPI projects, and the Zenodo integration once a second maintainer exists.
- If the project gains a legal home (a foundation, a fiscal host or a university; [open decision OD-02](docs/planning/open-decisions.md), due before C06), this document, the code-of-conduct contact and the [CRA position](SECURITY.md#eu-cyber-resilience-act) are revisited. The legal home also decides eligibility for Windows code signing ([CI/CD](docs/engineering/ci-cd.md#signing-plan-by-platform)).

## Changes to this document

Governance changes are proposed as a PR to this file, labelled `kind/docs` and `area/docs`, and kept open for at least 14 days. During Stages A and B the project lead approves. Under the council, two thirds of maintainers must approve.
