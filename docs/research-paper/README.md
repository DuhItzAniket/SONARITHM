# Research-paper track

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [research index](../research/README.md) · [research report](../research/research-report.md) · [design rules](../research/design-rules.md) · [roadmap](../planning/roadmap.md) · [implementation plan](../planning/implementation-plan.md) · [status](../sdlc/status.md) · [AGENTS.md](../../AGENTS.md)

## Purpose

This track is SONARITHM's lab notebook. While the software is being built, the team records what it learns, especially anything measured, surprising or decided by evidence. It then turns those records into hypotheses, experiments and papers.

The track exists because evaluation is the field's weakest link. Only 6.1% of published mapping occurrences were ever assessed ([Dubus & Bresin 2013](https://doi.org/10.1371/journal.pone.0082491)). The research report concludes that SONARITHM's most valuable output is "evaluated defaults and a benchmark that makes everyone's claims testable, including its own" ([research report, Conclusion](../research/research-report.md#conclusion)). The track is how the project keeps that promise.

## How the track works

### Day to day: log findings while implementing

Anyone implementing a phase, human or agent, adds an entry to the [findings log](findings-log.md) when one of these happens:

| Trigger | Examples |
|---|---|
| **Something was measured** | A spike's latency, a soak result, a real-time factor, a null-test residual, a compile time. Each benchmark row also goes into the [benchmarks log](benchmarks-log.md). |
| **Something was surprising** | A default sounds wrong on real data; a screen reader drops announcements; two OSes render audibly different output; a co-design partner works in an unexpected way. |
| **A decision rested on evidence** | An ADR, a rule change or a default chosen because of a paper, a measurement or a study result. |
| **A source was wrong or outdated** | A kickoff claim no longer holds (a tool was revived, a deadline was announced, a crate changed licence). |

A finding is a short, citable statement plus its evidence. It is not a status update. If in doubt, log it; triage is cheap.

### Weekly: triage

The research lead reviews every finding with status `new` (target: 30 minutes a week) and gives each one an outcome:

| Outcome | Action |
|---|---|
| No action | Status `triaged`; the entry stays as a record. |
| New or revised hypothesis | Add or update an entry in [hypotheses.md](hypotheses.md); link both ways. |
| Rule change | Open a PR against [design-rules.md](../research/design-rules.md) that cites the finding. `INFER` rules need an experiment, not just a finding. |
| Architectural decision | Raise or amend an ADR in [docs/decisions](../decisions/). |
| Paper material | Add the P-ID to the finding and note it in the paper's outline. |
| Correction | Log a new finding that supersedes the old one; set the old one to `superseded`. |

### At every phase review and stage gate

1. Update [contributions.md](contributions.md): move each claim's evidence status forward or back.
2. Update [publication-plan.md](publication-plan.md): re-check deadlines, confirm or defer each paper due in the next stage, and record deferrals with a reason.
3. Add the phase's benchmark row to the [benchmarks log](benchmarks-log.md).
4. Confirm that the phase's "research hooks" in the [roadmap](../planning/roadmap.md) are met, and record any that are not in [status.md](../sdlc/status.md).

### Monthly

Re-check every venue deadline marked `[EST]` against the official site (see the [publication plan](publication-plan.md)).

## ID conventions

IDs are permanent. They are never renumbered or reused. A retired item keeps its ID and is marked as such.

| ID | Meaning | Defined in |
|---|---|---|
| `F-###` | Finding (append-only) | [findings-log.md](findings-log.md) |
| `H-##` | Hypothesis | [hypotheses.md](hypotheses.md) |
| `EXP-###` | Experiment or study protocol. `EXP-000` is the template. | [experiments/](experiments/README.md) |
| `P##` | Paper (`P01`..`P09` at baseline) | [publication-plan.md](publication-plan.md), [papers/](papers/README.md) |
| `CL-##` | Claimed contribution | [contributions.md](contributions.md) |
| `DS-###` / `GEN-###` | Dataset / stimulus generator | [datasets.md](datasets.md) |
| `SDR-##` / `A11Y-##` | Design rule / accessibility requirement (owned by the research folder) | [design-rules.md](../research/design-rules.md) |

Cross-reference IDs in code comments, test names, commit messages, PR descriptions and ECHOMETRIC lint diagnostics, so that every default can be traced to its evidence.

## File map

| File | Purpose |
|---|---|
| [findings-log.md](findings-log.md) | Append-only log of findings: kickoff background, then implementation findings |
| [hypotheses.md](hypotheses.md) | `H-01`..`H-12`: statement, rationale, operationalization, experiment, paper, status |
| [experiments/README.md](experiments/README.md) | Experiment registry and lifecycle |
| [experiments/EXP-000-template.md](experiments/EXP-000-template.md) | Protocol template for every study |
| [evaluation-plan.md](evaluation-plan.md) | Overall evaluation strategy: technical, replication, studies, co-design, ethics, infrastructure |
| [publication-plan.md](publication-plan.md) | Venues, deadlines, sequence, authorship, artifacts, preprints, AI disclosure, open access |
| [contributions.md](contributions.md) | Claims, the evidence each needs, and current status |
| [related-work.md](related-work.md) | Paper-ready annotated related work, keyed to [references.bib](../research/references.bib) |
| [datasets.md](datasets.md) | Registry of datasets and stimulus generators |
| [benchmark-report-a08.md](benchmark-report-a08.md) | A08's technical benchmark report for P01 and P07: latency, rendering speed and determinism against the budgets |
| [benchmarks-log.md](benchmarks-log.md) | Engineering metrics per release and phase, against budgets |
| [papers/](papers/README.md) | One folder per paper, starting with an outline |

## Roles

| Role | Responsibilities |
|---|---|
| **Research lead** | Owns this track. Runs weekly triage, keeps hypotheses and contributions honest, owns the publication plan, signs off on preregistrations. Until a separate research lead is appointed, the project lead (@DuhItzAniket) holds this role. |
| **Study lead** (per experiment) | Writes the protocol from the template, obtains ethics approval, runs the study, files deviations, writes the results summary. |
| **Paper lead** (per paper) | Owns the outline and draft, the authorship agreement and the artifact package. |
| **Engineering owners** | Produce benchmark numbers with manifests (architecture lead for engine metrics, frontend owner for UI metrics). |
| **BLV co-design partners** | Paid partners in design and evaluation (A11Y-34). They shape protocols and materials, not only answer them. |
| **Contributors and AI agents** | Log findings and cite rule IDs as they work. See [AGENTS.md](../../AGENTS.md) for their duties toward this track. AI agents may draft entries and analyses; humans review them and make every research decision. |

## Quality bar

### Evidence levels

Every finding and claim carries one of these levels. Papers state the level of each claim they make.

| Level | Meaning |
|---|---|
| **EV-A** | Controlled study, preregistered, ethics-approved, with open materials and data |
| **EV-B** | Controlled study that is not preregistered, or a pilot below its planned sample size |
| **EV-C** | Reproducible engineering measurement: committed spec, render manifest (engine version, spec hash, data fingerprint, seed) and script |
| **EV-D** | Qualitative observation: a co-design note, a single session, an informal listening check |
| **EV-L** | Peer-reviewed literature. **EV-Lp** marks a preprint. |

Confidence uses the vocabulary of the [design rules](../research/design-rules.md): Strong, Moderate, Weak, Expert-opinion.

### Standards for SONARITHM's own evidence

- **Preregister** every confirmatory study on OSF before data collection. Mark analyses that were not preregistered as exploratory.
- **Ethics first.** No study with participants starts without approval (see the [evaluation plan](evaluation-plan.md)).
- **BLV participants are included** wherever the claim concerns them, and they are paid (SDR-40, A11Y-34).
- **Open materials.** Publish stimuli, specs, render manifests and analysis code (SDR-42). Publish de-identified data when consent allows.
- **Reproducible stimuli.** Every stimulus is rendered from a committed spec and seed with a manifest, following [ADR-0013](../decisions/0013-determinism-and-provenance.md).
- **No claim outruns its evidence.** [contributions.md](contributions.md) is the gate: a paper may only claim what that file marks as supported, at the stated level.
- **Exact citations.** Cite as the source states it. Mark preprints and estimates `[EST]`. Never invent a citation, number, DOI or deadline.

## Links to the rest of the project

- **Design rules.** `INFER` rules (SDR-20, SDR-26, SDR-28, SDR-33, SDR-39) map to hypotheses; changes to any rule need an `F-###` or `EXP-###` ([design-rules.md](../research/design-rules.md)).
- **Roadmap.** Each phase lists research hooks; A08, B07, C06 and D06 carry papers P01/P03, P02, P05 and the study programme ([roadmap](../planning/roadmap.md)).
- **Engines.** The ECHOMETRIC evaluation harness is the study infrastructure ([architecture/07-engines.md](../architecture/07-engines.md)).
- **Accessibility.** Conformance evidence (ACR, screen-reader matrix) is maintained in the [accessibility conformance plan](../design/accessibility-conformance-plan.md); this track uses it but does not own it.
- **AGENTS.md.** Duties of contributors and agents toward this track are listed in [AGENTS.md](../../AGENTS.md).
