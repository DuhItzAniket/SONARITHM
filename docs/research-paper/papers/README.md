# Papers

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [publication plan](../publication-plan.md) · [contributions](../contributions.md) · [related work](../related-work.md) · [references.bib](../../research/references.bib) · [findings log](../findings-log.md)

Each paper `P##` gets a folder here once its outline starts. The [publication plan](../publication-plan.md) owns venues, deadlines and policies; this folder holds the papers themselves.

## Index

| ID | Paper | Venue | Folder | Stage |
|---|---|---|---|---|
| P01 | Toolkit/system paper | ICAD 2027 | [P01-icad-2027-toolkit](P01-icad-2027-toolkit/outline.md) | outline |
| P02 | Software paper | JOSS | [P02-joss](P02-joss/outline.md) | outline |
| P03 | Open evaluation benchmark and harness | ICAD 2027 or IEEE VIS 2027 short | — | planned |
| P04 | Graph-concept polarity | ICAD 2028 | — | planned |
| P05 | Mature toolkit paper | CHI 2028 | — | planned |
| P06 | LLM-suggested vs empirical mappings | IEEE VIS 2028 or CHI 2029 | — | planned |
| P07 | Engine paper | DAFx27 or JAES | — | planned |
| P08 | Accessible uncertainty in audio | ASSETS 2028 or TACCESS | — | planned |
| P09 | AXONOTE as an instrument | NIME 2027 (at risk) | — | planned |

## Folder convention

Name each folder `P##-<venue>-<year>-<slug>/` (for example `P01-icad-2027-toolkit/`). Once a paper is drafted, the folder holds:

| File | Content |
|---|---|
| `outline.md` | Title options, abstract draft, contributions, structure, figures, evaluation, dependencies, risks, venue checklist. Written first. |
| `src/` | Manuscript sources in the venue's template (LaTeX or Markdown). Citations come only from [references.bib](../../research/references.bib). |
| `authorship.md` | Author list, order, and CRediT roles agreed by all authors. |
| `artifact.md` | The artifact package manifest: release DOI, specs and hashes, stimuli, analysis code, data, reproduction steps. |
| `ai-usage.md` | The AI-usage statement and the record behind it. |
| `submission.md` | Venue requirements checked against the current call, submission ID, decision, a summary of reviews and the response. Confidential review text is not committed. |

**Exception: JOSS.** JOSS requires the paper in the software repository, so P02's `paper.md` and `paper.bib` live in the root `paper/` folder ([repository layout](../../architecture/02-repository-and-crate-layout.md)). `P02-joss/` keeps the outline and records.

## Workflow

| Stage | Exit condition |
|---|---|
| `outline` | Outline approved by the research lead; claims checked against [contributions.md](../contributions.md); author order agreed. |
| `draft` | Complete draft; every number traced to a finding, benchmark row or experiment. |
| `internal review` | One reviewer for claims and evidence, one for writing. Papers about accessibility also get review from a BLV co-design partner. |
| `submitted` | Submission checklist in the [publication plan](../publication-plan.md#submission-checklist-every-paper) complete; findings used set to `used-in-paper`. |
| `revision` | Response to reviews; changes logged in `submission.md`. |
| `accepted` / `rejected` | Decision recorded. A rejected paper's next venue is chosen at the next phase review. |
| `published` | Camera-ready submitted; artifact package archived with a DOI; preprint posted where allowed. |
