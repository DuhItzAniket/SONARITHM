# Definition of Ready and Definition of Done

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> **Related:** [roadmap](../planning/roadmap.md) · [status](status.md) · [traceability matrix](traceability-matrix.md) · [reviews](reviews/README.md) · [retrospectives](retrospectives/README.md) · [coding-standards](../engineering/coding-standards.md) · [testing-strategy](../engineering/testing-strategy.md) · [git-workflow](../engineering/git-workflow.md) · [findings log](../research-paper/findings-log.md)

The Definition of Ready (DoR) says when work may start. The Definition of Done (DoD) says when it is finished, at three levels: task or pull request, phase, and stage gate. Both apply equally to human contributors and AI coding agents. A phase is done only when its exit criteria in the [roadmap](../planning/roadmap.md) are met and this DoD holds for its work.

## 1. Definition of Ready

An issue moves to "Ready" on the project board only when it has the following. Whoever moves it checks the list.

| Item | Required content |
|---|---|
| Problem statement | What is wrong or missing, for whom, and why it matters. Not a solution description. |
| Acceptance criteria | Observable, testable conditions. Each criterion can be checked by a test, a measurement or a named manual check. |
| Phase ID | The roadmap phase the work belongs to (`A01` … `D06`), set as a label or project field. Work outside any phase is labelled as maintenance. |
| Design or ADR link | Required if the change is architectural: a new crate or top-level directory, a cross-crate interface, a dependency-rule exception, or anything listed in [docs/decisions](../decisions/README.md). If an RFC is required ([RFC criteria](../rfcs/README.md#1-when-an-rfc-is-required)), the RFC is accepted first. |
| Accessibility impact | One of: none; UI change (which components); assistive-technology-facing change (announcements, navigation, braille); audio-only information (needs a non-audio equivalent, A11Y-24). List affected A11Y IDs. |
| Test approach | Which test types from the [testing strategy](../engineering/testing-strategy.md) will prove the acceptance criteria, including any manual check. |
| Research hook | Design-rule IDs affected (SDR or A11Y), hypotheses touched, findings to log, or benchmarks to record. "None" is a valid answer and must be written. |
| Dependencies | Blocking issues or phases identified. |

**For work delegated to an AI agent**, the issue also names the files, crates or packages in scope, the verification commands to run (for example `cargo xtask ci`), and any decision the agent must not take on its own. Agent instructions are in [AGENTS.md](../../AGENTS.md).

**Small fixes** (typos, broken links, obvious one-line bugs) need only a problem statement and an acceptance criterion.

## 2. Definition of Done: task and pull request

The PR template (`.github/PULL_REQUEST_TEMPLATE.md`) mirrors this list. A reviewer does not approve until every applicable item holds.

**Code**

- [ ] All required CI checks pass on Windows, macOS and Linux ([ci-cd](../engineering/ci-cd.md)).
- [ ] The code follows [coding-standards](../engineering/coding-standards.md); no new warnings; no `#[allow]` without a reason.
- [ ] No new `unsafe` outside the [unsafe register](../engineering/coding-standards.md#5-unsafe-code).
- [ ] New or upgraded dependencies are justified in the PR and pass `cargo deny check` ([supply-chain-and-licensing](../engineering/supply-chain-and-licensing.md)).
- [ ] Generated artifacts are regenerated and committed (JSON Schema, IPC types); CI shows no regeneration diff.

**Tests**

- [ ] Each acceptance criterion is covered by a test or a recorded manual check.
- [ ] Snapshot and golden changes are intentional, explained in the PR and, for audio, listened to ([testing-strategy §5.3](../engineering/testing-strategy.md#53-golden-file-policy)).
- [ ] No new flaky or ignored tests; any `#[ignore]` links an issue.

**Documentation**

- [ ] Public items have rustdoc (or TSDoc, or Python docstrings); examples compile.
- [ ] User-facing docs, architecture docs and the [traceability matrix](traceability-matrix.md) are updated where behaviour, structure or rule coverage changed.
- [ ] Architectural decisions are recorded as an ADR or linked to an existing one.

**Process**

- [ ] The PR title follows Conventional Commits with a crate or engine scope, commits are signed off (DCO), and the branch follows [git-workflow](../engineering/git-workflow.md).
- [ ] The PR links its issue and states the phase ID.
- [ ] Findings (`F-###`) are logged in the [findings log](../research-paper/findings-log.md) if the work produced one; measurements go to the [benchmarks log](../research-paper/benchmarks-log.md).
- [ ] At least one approving review, including a code owner where CODEOWNERS requires one. PRs authored by an AI agent are reviewed by a human maintainer. During the [fast-track](../engineering/git-workflow.md#review-rules) (owner decision, 2026-10-19), this item becomes: all required checks are green and auto-merge is enabled.

## 3. Additional Done criteria by change type

These lists add to section 2. A PR that falls into several categories meets all of them.

### 3.1 Audio and real-time code

Applies to `sonarithm-dsp`, `sonarithm-render`, `sonarithm-audio`, `sonarithm-export`, and any function marked RT-safe.

- [ ] The real-time review checklist ([coding-standards §11.6](../engineering/coding-standards.md#116-reviewing-rt-code)) is completed in the PR.
- [ ] The `no-alloc` CI run passes and exercises the changed path.
- [ ] Golden-audio tests pass, or references are updated through `cargo xtask golden --update` with a reason.
- [ ] Render and DSP benchmarks are within the regression threshold; hot-path PRs include before and after numbers ([testing-strategy §9](../engineering/testing-strategy.md#9-performance-and-soak-tests)).
- [ ] Changes to the scheduler, voice pool, plan swap or backend attach a local soak result (10 min, 128-frame buffers, 0 xruns), and are listed for the next device soak on reference hardware ([testing-strategy §9.3](../engineering/testing-strategy.md#93-soak-tests)).
- [ ] Unsafe or lock-free changes have Miri and model-test (shuttle or loom) coverage.
- [ ] Audible changes were listened to with headphones at low gain; nothing bypasses the start-up ramp or the master limiter ([06 §10](../architecture/06-audio-engine.md#10-hearing-safety-and-levels)).

### 3.2 User interface

Applies to `apps/desktop/src` and `packages/*`.

- [ ] **Keyboard**: every function is operable by keyboard, with no traps, a logical focus order and a visible, unobscured focus indicator (A11Y-05). New shortcuts are registered in the keymap and documented in [keyboard-and-navigation](../design/keyboard-and-navigation.md); single-key shortcuts are scoped or remappable (A11Y-06).
- [ ] **Screen reader**: changed components are checked manually with NVDA (Windows) or VoiceOver (macOS); new components and custom widgets are checked with both. Name, role, value and state are correct (A11Y-18).
- [ ] **axe**: no serious or critical violations in the component's Storybook stories or the affected Playwright flows.
- [ ] **Contrast**: text ≥ 4.5:1 and non-text elements and focus indicators ≥ 3:1 in the dark, light and high-contrast themes.
- [ ] **High-contrast theme and forced colours**: the component is correct in the high-contrast theme and under `forced-colors: active`; canvas layers redraw with system colours.
- [ ] **Reduced motion**: `prefers-reduced-motion` is honoured; no decorative motion.
- [ ] **Non-audio equivalents**: any information conveyed by sound is also available as text, visual or braille output (A11Y-24).
- [ ] **Strings and labels**: user-facing strings come from the message catalogue; no icon-only control without an accessible name and tooltip.
- [ ] **Stories and baselines**: a Storybook story covers each state, and visual baselines are updated intentionally.
- [ ] **Hot paths**: playhead, meters and plots do not update through React state per frame ([coding-standards §12.5](../engineering/coding-standards.md#125-hot-paths-bypass-react)).

### 3.3 Spec changes

Applies to any change to the spec types in `sonarithm-spec` or to the spec language ([04-sonification-spec](../architecture/04-sonification-spec.md)).

- [ ] The JSON Schema is regenerated with `cargo xtask codegen` (schemas and TypeScript types), and the CI schema diff passes.
- [ ] The `specVersion` bump follows [04 §7](../architecture/04-sonification-spec.md#7-versioning-and-compatibility): a minor bump for added optional fields or enum values; a major bump for renames, removals or semantic changes. A major bump has an accepted RFC. Before 1.0, a `0.x` minor bump may break, and must then ship a migration.
- [ ] Every breaking change ships a migration in the `sonarithm-spec` migration framework, tested against fixtures of the previous version, and reachable through `sonarithm spec migrate`.
- [ ] Every spec example in the docs is updated and validates in CI.
- [ ] Generated IPC types are regenerated if spec types cross the IPC boundary.
- [ ] Importers (Erie, and later Vega-Lite and MAIDR) map the change or report it as unmapped; they never drop constructs silently.
- [ ] Score snapshot changes are explained. If the snapshot format changed, `ir_version` is bumped ([02 §6](../architecture/02-repository-and-crate-layout.md#6-versioning)).

### 3.4 Research-affecting defaults

Applies to any change that sets or alters a default derived from the [design-rules catalogue](../research/design-rules.md), or adds a new perceptual default.

- [ ] The rule ID is cited in the code documentation (`/// Default per SDR-08`), in the PR description, and in the `Info` diagnostic that reports the applied default ([05 §7](../architecture/05-compiler-and-score-ir.md#7-diagnostics)).
- [ ] A changed default updates the catalogue, stating the old value, the new value and the evidence (`F-###` or `EXP-###`), per the catalogue's change rules. An `INFER` rule does not become `DEFAULT` or `SHOULD` without a supporting SONARITHM experiment.
- [ ] The finding is logged in the [findings log](../research-paper/findings-log.md).
- [ ] The conformance test named after the rule is added or updated ([testing-strategy §6](../engineering/testing-strategy.md#6-design-rule-conformance-tests)).
- [ ] The [traceability matrix](traceability-matrix.md) is updated.
- [ ] Snapshot and golden updates caused by the change cite the rule ID in their reason.

## 4. Definition of Done: phase

A phase (for example A03) is done when all of the following hold. The phase review records the evidence.

- [ ] Every exit criterion in the [roadmap](../planning/roadmap.md) is met, each with an evidence link (CI run, benchmark entry, test name, manual check record).
- [ ] All issues in the phase are closed, or deferred with a target phase and a reason.
- [ ] No open critical bug introduced by the phase, unless the phase review records an accepted exception.
- [ ] Architecture and engineering docs describe what was built; deviations from the plan are recorded as ADRs or roadmap changes ([roadmap §7](../planning/roadmap.md#7-how-to-change-this-roadmap)).
- [ ] [status.md](status.md) shows the phase as done, with the date.
- [ ] Findings are logged with `F-###` IDs; phase-end measurements are in the [benchmarks log](../research-paper/benchmarks-log.md); the phase's research hooks are satisfied or consciously deferred.
- [ ] The [traceability matrix](traceability-matrix.md) reflects every rule implemented or changed in the phase.
- [ ] The [risk register](../planning/risk-register.md) is updated for risks the phase retired, changed or raised.
- [ ] Coverage targets for crates introduced or changed in the phase are checked ([testing-strategy §16](../engineering/testing-strategy.md#16-coverage-targets)).
- [ ] The phase review is written from the [template](reviews/_phase-review-template.md), and a [retrospective](retrospectives/README.md) is held.

## 5. Definition of Done: stage gate

A stage (A, B, C, D) ends at a gate. The gate is passed when all of the following hold. Stage-specific criteria (for example Gate B's "a BLV user completes the core tasks unaided in ≥ 80% of attempts") are listed with each stage in the [roadmap](../planning/roadmap.md) and copied into the gate review.

- [ ] Every phase in the stage meets the phase DoD, and its phase review is final.
- [ ] The gate review is written from the [template](reviews/_gate-review-template.md) and records a go, conditional go or no-go decision.
- [ ] The [risk register](../planning/risk-register.md) is reviewed in full, not only the risks touched in the stage.
- [ ] The release for the gate is cut per [release-process](../engineering/release-process.md), with the version given in [roadmap §6](../planning/roadmap.md#6-release-and-version-mapping).
- [ ] For stages that ship UI (B and C): the ACR for the release is published (v0 at Gate B, v1 at Gate C) per the [accessibility conformance plan](../design/accessibility-conformance-plan.md); the manual screen-reader matrix and the Chartability audit have passed for the release candidate.
- [ ] Performance budgets are measured on reference hardware and re-baselined where the gate review accepts a change.
- [ ] The scope of the next stage is confirmed, and its indicative dates are re-baselined in the roadmap and [status.md](status.md). Publication deadlines are fixed points; scope moves to fit them.
- [ ] The project lead signs off in the gate review.

## 6. Where evidence goes

| Evidence | Location |
|---|---|
| Per-PR checklist | The PR description (from the PR template) |
| Session notes, problems, intermediate measurements | [Worklog](worklog/README.md) |
| Decisions | [ADRs](../decisions/README.md); proposals that need discussion first: [RFCs](../rfcs/README.md) |
| Research findings | [Findings log](../research-paper/findings-log.md) (`F-###`) |
| Benchmarks and measurements | [Benchmarks log](../research-paper/benchmarks-log.md) |
| Rule → implementation → test | [Traceability matrix](traceability-matrix.md) |
| Phase and gate outcomes | [Reviews](reviews/README.md) |
| Lessons and process actions | [Retrospectives](retrospectives/README.md) |
| Current state | [status.md](status.md) |
