# Research

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [research-paper track](../research-paper/README.md) · [design rules](design-rules.md) · [roadmap](../planning/roadmap.md) · [AGENTS.md](../../AGENTS.md)

This folder holds the evidence base for SONARITHM: the kickoff research, the design rules derived from it, and the shared bibliography. The [research-paper track](../research-paper/README.md) builds on it to record new findings and turn them into papers.

## Contents

| Path | What it is |
|---|---|
| [research-report.md](research-report.md) | The kickoff synthesis (2026-10-18): landscape, perception science, accessibility, architecture, stack, governance and publishable gaps. Read it first. |
| [design-rules.md](design-rules.md) | Stable, citable rules `SDR-01`..`SDR-42` and `A11Y-01`..`A11Y-34`, with strength, confidence and owner. Rules marked `INFER` are hypotheses that SONARITHM must test. |
| [references.bib](references.bib) | Shared BibTeX for all documents and papers. Section 1 is copied verbatim from notes/02 §8; later sections add sources from notes/03, 06 and 07. |
| [notes/01-tools-landscape.md](notes/01-tools-landscape.md) | Prior art and competitive analysis: every relevant tool, its licence, platform and maintenance status, and the gaps across the landscape. |
| [notes/02-theory-perception.md](notes/02-theory-perception.md) | Theory, taxonomy, psychoacoustics, mapping design, evaluation methodology and open problems; the numbered design rules (§7) and the verified BibTeX list (§8). |
| [notes/03-accessibility-hci.md](notes/03-accessibility-hci.md) | BLV users' needs, navigation and interaction techniques, standards and law, platform accessibility, STEM contexts, research methods with BLV participants (§6) and the accessibility requirements (§7). |
| [notes/04-rust-core-stack.md](notes/04-rust-core-stack.md) | Evaluation of the Rust core stack: audio I/O, DSP, real-time safety, file and protocol I/O, data, ML inference, bindings, plugins and quality tooling. |
| [notes/05-desktop-frontend-stack.md](notes/05-desktop-frontend-stack.md) | Desktop shell, frontend framework, accessible component libraries, visualization performance, editors, design language and frontend testing. |
| [notes/06-chart-extraction-ai.md](notes/06-chart-extraction-ai.md) | Chart digitization and chart-to-table models, 2026 benchmarks and failure modes, local inference, exact extraction paths and LLM-assisted features. |
| [notes/07-engineering-publication.md](notes/07-engineering-publication.md) | Engineering, release and governance practice; research-software practice (§6); publication venues, toolkit evaluation and ethics (§7); the publication plan (§9). |

## How to update the research

1. **Kickoff documents are snapshots.** Never rewrite `notes/01`..`07` or `research-report.md`. If a statement is wrong or outdated, add an entry to an `## Errata` section at the bottom of that file, with the date, the correction and its source. Leave the original text in place.
2. **New research goes in a new note.** Continue the numbering in `notes/` (`08-<topic>.md`, `09-<topic>.md`, …). The first lines state the research date, the scope and the method, as the kickoff notes do. Use the same structure: takeaway, cited findings, inferences, gaps.
3. **Cite exactly.** Give author, year and DOI or URL as the source states them. Mark preprints, unverified details and estimates (`[EST]`). Never invent a citation, number, DOI or deadline.
4. **Add every new source to [references.bib](references.bib).** Append it to the right section, keep the key style described in the file header, and never change an existing key or a verified DOI.
5. **Rules change only through evidence.** A change to [design-rules.md](design-rules.md) is a PR that cites a finding (`F-###`) or an experiment (`EXP-###`) and follows the change control in that file. An `INFER` rule cannot become `DEFAULT` or `SHOULD` until a SONARITHM experiment supports it.

## How research links to the paper track

| Research artefact | Paper-track artefact |
|---|---|
| Kickoff findings in notes and the report | Background entries `F-001`.. in the [findings log](../research-paper/findings-log.md) |
| `INFER` rules and named gaps | Hypotheses `H-01`..`H-12` in [hypotheses.md](../research-paper/hypotheses.md) |
| Evaluation methods (notes/02 §5, notes/03 §6) | The [evaluation plan](../research-paper/evaluation-plan.md) and the [experiment template](../research-paper/experiments/EXP-000-template.md) |
| Venues and timing (notes/07 §7, §9) | The [publication plan](../research-paper/publication-plan.md) |
| Prior work across all notes | Paper-ready [related work](../research-paper/related-work.md), keyed to [references.bib](references.bib) |

When implementation produces new evidence, it is logged in the findings log first. Only then may it change a rule, an ADR or a paper.
