# Gate <STAGE> review: <stage name>

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead. This is the stage gate review template. In a copy, replace this block with: `> **Status:** Draft | Final (YYYY-MM-DD). **Gate:** <STAGE>. **Release:** <version>. **Decision:** Go | Conditional go | No-go.`
> **Related:** [reviews README](README.md) · [roadmap](../../planning/roadmap.md) · [Definition of Done, gate level](../definition-of-ready-and-done.md#5-definition-of-done-stage-gate) · [risk register](../../planning/risk-register.md) · [release-process](../../engineering/release-process.md)

<!--
Save as docs/sdlc/reviews/gate-<STAGE>.md.
Copy the stage-specific gate criteria verbatim from the roadmap.
-->

## Decision

| Item | Value |
|---|---|
| Stage | <A / B / C / D> · <name> |
| Gate date | <YYYY-MM-DD> |
| Release | <version from roadmap §6> |
| Decision | **Go / Conditional go / No-go** |

<Rationale in three to six lines.>

Conditions (conditional go only):

| Condition | Owner | Due | Issue |
|---|---|---|---|
| | | | |

## Go/no-go criteria

**Stage-specific criteria** (verbatim from the roadmap):

| # | Criterion | Met | Evidence |
|---|---|---|---|
| 1 | <criterion> | Yes / No / Waived | |

**Gate Definition of Done:**

- [ ] Every phase in the stage meets the phase DoD and has a final review.
- [ ] Risk register reviewed in full.
- [ ] Release cut per the release process.
- [ ] ACR published and screen-reader matrix and Chartability audit passed (stages B and C).
- [ ] Budgets measured on reference hardware; re-baselines accepted below.
- [ ] Next-stage scope confirmed; dates re-baselined in the roadmap and status.

## Phase reviews

| Phase | Review | Outcome | Open waivers |
|---|---|---|---|
| <PHASE-ID> | <link> | Done / Done with waivers | <count, links> |

## Release readiness

| Item | Status | Evidence |
|---|---|---|
| Version and tag | | |
| Crates published | | |
| Python wheels (Windows, macOS, Linux) | | |
| Desktop installers built, signed, smoke-tested (stages B and later) | | |
| Changelog and release notes | | |
| SBOM and build provenance | | |
| Known issues documented | | |
| Support policy in SECURITY.md (Gate C) | | |

## Accessibility conformance (stages B and C)

| Item | Status | Link |
|---|---|---|
| ACR version and publication | <v0 at Gate B, v1 at Gate C> | |
| Manual screen-reader matrix (A11Y-32) | Pass / Fail per cell | |
| Chartability audit (A11Y-33) | | |
| Open accessibility issues: critical / high / other | <counts> | |
| Co-design findings triaged (Gate B onward) | | |

## Performance budgets

| Attribute | Target ([01 §7](../../architecture/01-overview.md#7-quality-attributes-and-budgets)) | Measured on reference hardware | Verdict | Re-baseline proposal |
|---|---|---|---|---|
| Interactive response | ≤ 50 ms p95 at 48 kHz, 256-frame buffer | | | |
| Audio robustness | 0 xruns, 10-min soak, 128-frame buffers | | | |
| Offline render speed | ≥ 50× real time | | | |
| Compile time | ≤ 200 ms (100k × 4); ≤ 1 s (1M) | | | |
| Determinism | Identical Score IR; residual ≤ −90 dBFS; ΔLUFS ≤ 0.1 | | | |
| UI frame time (stages B and later) | 60 fps; ≤ 8 ms scripting per frame | | | |
| Startup (stages B and later) | ≤ 2 s | | | |
| Footprint (Gate C) | Installer ≤ 40 MB | | | |
| Privacy | Zero network calls without consent | | | |

## Risk register review

| Risk ID | Previous rating | New rating | Action |
|---|---|---|---|
| | | | |

New risks raised at this gate:

| Risk | Rating | Owner | Mitigation |
|---|---|---|---|
| | | | |

## Research and publications

| Item | Status | Link |
|---|---|---|
| Papers due in or after this stage (fixed points) | | |
| Findings logged in the stage | <count> | [findings log](../../research-paper/findings-log.md) |
| Hypotheses prepared, run or resolved | | |

## Next stage

**Scope confirmed:**

| Phase | Scope change from the roadmap | Reason |
|---|---|---|
| | None / <change> | |

Work moved between stages, and phases added or removed, are recorded here per [roadmap §7](../../planning/roadmap.md#7-how-to-change-this-roadmap).

**Re-baselined dates:**

| Phase | Previous window | New window | Reason |
|---|---|---|---|
| | | | |

## Sign-off

| Role | Handle | Date | Decision |
|---|---|---|---|
| Project lead | | | Go / Conditional go / No-go |
| Architecture lead | | | |
| Research lead | | | |
| Accessibility owner (stages B and C) | | | |
