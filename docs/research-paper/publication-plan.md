# Publication plan

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead. **Deadlines last checked:** 2026-10-18.
> **Related:** [papers](papers/README.md) · [contributions](contributions.md) · [evaluation plan](evaluation-plan.md) · [roadmap](../planning/roadmap.md) · [research report, publishable gaps](../research/research-report.md#publishable-research-sits-in-the-fields-admitted-gaps) · [notes/07 §6–§9](../research/notes/07-engineering-publication.md)

**Current constraint (2026-10-19, OD-04).** The owner works alone without a faculty mentor or institutional ethics review, so no human-participant studies can be published for now. **Active:** P01 (toolkit: demonstration, replication gallery, technical benchmarks), P02 (JOSS) and P07 (engine). **Deferred** until an ethics route exists (revisit at Gate B): P03's study extension, P04, P05's usage evaluation, P06, P08 and P09. The benchmark *harness* part of P03 needs no participants and can still go ahead.

Publication deadlines are fixed points in the [roadmap](../planning/roadmap.md): scope moves to fit them, not dates. This file maps the papers `P01`..`P09` to venues and phases, and sets the policies every paper follows.

> **Re-check deadlines monthly.** Except for the passed CHI 2027 deadline and the confirmed event dates noted below, every 2027–2028 deadline here is an estimate from previous cycles, marked `[EST]`. Check each venue's official site at least monthly and update this file and the roadmap milestones in the same PR.

## Paper portfolio

| ID | Paper | Primary venue | Deadline | Roadmap phases | Depends on | Status |
|---|---|---|---|---|---|---|
| P01 | Toolkit/system paper: spec compiler, deterministic engine, replication gallery | ICAD 2027 | ~Mar 2027 [EST] | A01–A08 | Gallery, benchmarks, `v0.2.0-alpha` | outline ([P01](papers/P01-icad-2027-toolkit/outline.md)) |
| P02 | Software paper | JOSS | Rolling; earliest about 6 months after the repository goes public | B07 | Six months of public history; evidence of research use | outline ([P02](papers/P02-joss/outline.md)) |
| P03 | Open evaluation benchmark and harness | ICAD 2027 or IEEE VIS 2027 short paper | ~Mar 2027 [EST] | A08 | Generators GEN-001..004; EXP-001 if approved in time | planned |
| P03 ext. | Pacing × range × context factorial (plus stream, spacing and comfort studies) | ICAD 2028 or ACM TACCESS | ~Mar 2028 [EST] / rolling | D06 | EXP-002, EXP-003, EXP-010, EXP-011 | planned |
| P04 | Graph-concept polarity by magnitude estimation | ICAD 2028 (CHI 2028 only if EXP-004 runs early) | ~Mar 2028 [EST] | D06 | EXP-004, ethics approval | planned |
| P05 | Toolkit paper on the mature system with multi-strategy evaluation | CHI 2028 | ~early–mid Sep 2027 [EST] | C06 | Stages A–C; EXP-008, EXP-009, EXP-012, EXP-013 | planned |
| P06 | LLM-suggested vs empirical mappings | IEEE VIS 2028 or CHI 2029 | ~Mar 2028 / ~Sep 2028 [EST] | D04, D06 | EXP-006 | planned |
| P07 | Engine paper: deterministic real-time/offline rendering, latency, audification calculator | DAFx27 or JAES | ~Mar–Apr 2027 [EST] / rolling | A02, A05 (C03 for audification) | EXP-007, benchmarks log | planned |
| P08 | Accessible uncertainty in audio | ASSETS 2028 or ACM TACCESS | ~Apr 2028 [EST] / rolling | C02, D04, D06 | EXP-005 | planned |
| P09 | Expressive use: AXONOTE as an instrument | NIME 2027 | ~late Jan–Feb 2027 [EST] | C04 | AXONOTE, EXP-015 | at risk (see below) |

## Venues and deadlines

"Confirmed" means published by the venue. `[EST]` is an estimate from the previous cycle.

| Venue | Papers | Deadline | Event | Source |
|---|---|---|---|---|
| **ICAD 2027** | P01, P03 | ~Mar 2027 [EST] (ICAD 2026: full papers 20 Mar 2026, other categories 27 Mar 2026) | Athens, Greece (confirmed); dates TBA | [icad.org](https://icad.org/); [ICAD 2026](https://interactive-sonification.org/icad2026/) |
| **IEEE VIS 2027** (full and short papers) | P03 (alternative) | Abstract ~21 Mar, full paper ~31 Mar 2027 [EST] (same dates in 2024 and 2026) | Chicago, 7–12 Nov 2027 (unconfirmed listing) | [VIS 2026 CfP](https://aa.ieeevis.org/year/2026/info/call-participation/call-for-participation) |
| **DAFx27** | P07 | ~Mar–Apr 2027 [EST] | Cremona, Italy, 24–27 Aug 2027 (confirmed) | [dafx27.deib.polimi.it](https://dafx27.deib.polimi.it/) |
| **NIME 2027** | P09 | ~late Jan–Feb 2027 [EST]; call published October 2026 | Paris, 22–25 Jun 2027 (confirmed) | [nime2027.org](https://www.nime2027.org/) |
| **ACM ASSETS 2027** | none planned (see P08) | ~late Apr 2027 [EST] (ASSETS 2026: 22 Apr 2026) | ~Oct 2027 [EST] | [ASSETS 2026 CfP](https://assets26.sigaccess.org/call_for_papers.html) |
| **EuroVis 2027** | none planned (short paper or poster possible) | Full papers ~early Dec 2026 [EST] (2026 cycle: 3 Dec 2025, abstract beforehand) | Stuttgart, 31 May–4 Jun 2027 (unofficial listing) | [timeline](https://velitchko.github.io/eurovis-timeline/); [EG events](https://www.eg.org/wp/events/list/) |
| **Audio Mostly 2027** | none planned | ~Apr–May 2027 [EST, low confidence] | ~Aug–Sep 2027 [EST] | [audiomostly.com](https://audiomostly.com/2024/global-hosting/) |
| **SMC 2027** | none planned | ~Feb–Apr 2027 [EST] | TBA | [smcnetwork.org](https://smcnetwork.org/conferences.html) |
| **JOSS** | P02 | Rolling; at least six months of public history plus evidence of research use (confirmed rule) | — | [JOSS submitting](https://joss.readthedocs.io/en/latest/submitting.html) |
| **ACM CHI 2028** | P05 (P04, P06 as options) | ~early–mid Sep 2027 [EST] (CHI 2027: 10 Sep 2026, passed) | ~Apr–May 2028 [EST] | [CHI 2027 papers](https://chi2027.acm.org/authors/papers/) |
| **ICAD 2028** | P03 ext., P04 | ~Mar 2028 [EST, by pattern] | TBA | [icad.org](https://icad.org/) |
| **IEEE VIS 2028** | P06 | ~late Mar 2028 [EST, by pattern] | TBA | — |
| **ACM ASSETS 2028** | P08 | ~Apr 2028 [EST, by pattern] | TBA | — |
| **NIME 2028** | P09 (fallback) | TBA [EST] | TBA | — |
| **ACM TACCESS** | P03 ext., P08 | Rolling | — | [TACCESS](https://dl.acm.org/journal/taccess) |
| **Journal of the AES** | P07 (alternative) | Rolling | — | [AES Journal](https://www.aes.org/journal/) |
| **Organised Sound** | P09 journal option | Themed calls | — | [Organised Sound](https://www.cambridge.org/core/journals/organised-sound) |

## Sequence

| Window | Roadmap | Publication actions |
|---|---|---|
| Q4 2026 | A01–A06 | Repository public from A01 (this starts JOSS's six-month clock). Zenodo–GitHub integration enabled before the first tag. Ethics application filed by 31 Dec 2026. Go/no-go on P09 for NIME 2027 by mid-Dec 2026. |
| Q1 2027 | A07, A08 | `v0.1.0-alpha` mints the first Zenodo DOI. `v0.2.0-alpha` with harness and gallery. Submit P01 (and P03, or merge P03 into P01) to ICAD 2027, roadmap milestone 2027-03-15 (est.). P03 alternative: IEEE VIS 2027 short paper. P07 to DAFx27. |
| Q2 2027 | B01–B07 | EXP-001 and co-design round 1 (EXP-012) after approval. Submit P02 to JOSS once eligible, roadmap milestone 2027-06-01 (est.). |
| Q3 2027 | C01–C06 | Submit P05 to CHI 2028. |
| Q4 2027–2028 | D01–D06 | Study programme. P03 extension and P04 to ICAD 2028; P06 to IEEE VIS 2028 or CHI 2029; P08 to ASSETS 2028 or TACCESS; P09 to NIME 2028 if deferred. Journal extensions afterwards. |

This follows the sequence in [notes/07 §9](../research/notes/07-engineering-publication.md) and the [research report](../research/research-report.md#publishable-research-sits-in-the-fields-admitted-gaps): public repository with DOIs, ICAD 2027 debut, JOSS in mid-2027, ASSETS or VIS if studies finish in time, then CHI 2028 with a mature toolkit.

### Scheduling tensions to resolve at Gate A

These follow from the roadmap's indicative windows. They are recorded here so they are decided deliberately, not discovered late.

1. **P05 vs C06.** The CHI 2028 milestone (2027-09-10, est.) falls before C06 starts (2027-09-15). The P05 draft must be written during C01–C05, with C06 hardening after submission, or the milestone moves.
2. **P09 vs C04.** AXONOTE arrives in C04 (Aug–Sep 2027), after the estimated NIME 2027 deadline. Either P09 becomes a demo or work-in-progress built on an early prototype, or it moves to NIME 2028.
3. **P04, P06, P08 vs D06.** The research report gives 2027 windows (P04: ICAD/CHI 2028; P06: CHI 2028/VIS; P08: ASSETS 2027). The roadmap runs these studies in D06 from November 2027. The plan above uses the 2028 venues; EXP-004 can be pulled forward, because it needs only the A08 harness and ethics approval.
4. **P02 eligibility.** JOSS requires evidence of research use. If no external use exists by May 2027, the evidence must come from EXP-001 or EXP-012, which depend on ethics approval.

## Authorship and contribution policy

- **Who is an author.** Anyone who made a substantial contribution to the work (conceptualization, methodology, software, investigation, formal analysis or data curation), helped draft or critically revise the paper, approved the final version, and accepts accountability for it.
- **CRediT roles.** Every paper folder records each author's roles in `authorship.md` using the CRediT taxonomy: Conceptualization; Data curation; Formal analysis; Funding acquisition; Investigation; Methodology; Project administration; Resources; Software; Supervision; Validation; Visualization; Writing – original draft; Writing – review & editing. Papers include a CRediT statement where the venue allows.
- **Order and lead.** Author order is agreed when the outline is approved and revisited before submission. The paper lead is the corresponding author.
- **BLV co-design partners.** Partners whose contribution meets the criteria are offered co-authorship, as in Zong et al. (2022), who had a blind co-author. Others are acknowledged by name only with explicit consent, or anonymously.
- **Participants** are thanked collectively and never named.
- **Software contributors** are credited through `CITATION.cff` and the Zenodo release metadata. For JOSS, the submitting author must be a major contributor ([JOSS submitting](https://joss.readthedocs.io/en/latest/submitting.html)).
- **AI tools are never authors.** Their use is disclosed (see below).

## Artifact and reproducibility package policy

Every paper ships an artifact package so that its results can be regenerated ([notes/07 §6](../research/notes/07-engineering-publication.md)).

1. **Release DOI.** Each paper cites a specific SONARITHM release with a Zenodo DOI, minted through the Zenodo–GitHub integration and `CITATION.cff`.
2. **Package contents,** archived on Zenodo and linked from the paper:
   - specs, datasets (or generator ID, version and seeds) and render manifests;
   - stimuli and study definitions;
   - analysis code with a pinned environment;
   - de-identified data where consent allows;
   - a README with exact reproduction steps and expected outputs.
3. **Determinism.** Every figure and sound in a paper is regenerated from a committed spec; renders embed the engine version and spec hash ([ADR-0013](../decisions/0013-determinism-and-provenance.md)).
4. **Badges.** Where a venue offers ACM Artifact Review and Badging, target *Artifacts Available* and *Artifacts Evaluated – Functional* or *Reusable*. Badge names come from ACM policy v1.1 as recorded in the kickoff notes from prior knowledge; verify them at submission.
5. **Licences.** Code: MIT OR Apache-2.0. Data, stimuli and figures: CC BY 4.0, as the research report suggests for datasets and stimuli. Where a venue publishes under another licence (ICAD 2025 proceedings used CC BY-NC 4.0), the artifacts keep the project licences.
6. **Location.** Drafts and outlines live in [papers/](papers/README.md). The JOSS `paper.md` and `paper.bib` live in the root `paper/` folder, because JOSS requires the paper files in the same repository.

## Preprint policy

- Post submitted or accepted versions to arXiv (cs.HC, cross-listed to cs.SD where audio is central) when the venue's anonymity and preprint rules allow. Check those rules for each venue at submission; they were not verified in the kickoff research.
- Preregistrations go to OSF before data collection; the paper links them.
- Never post anything that contains participant data without consent for sharing.
- Cite preprints as preprints, with the arXiv identifier in [references.bib](../research/references.bib).

## AI-usage disclosure

- **JOSS** permits AI use but requires disclosure. Authors confirm human review and that core design decisions were made by humans; reviewers may not let AI determine verdicts ([notes/07 §6](../research/notes/07-engineering-publication.md)). The P02 paper has a dedicated "AI usage disclosure" section.
- **SONARITHM is built by a core developer working with AI coding agents** (roadmap planning assumption). P02 must describe this accurately, including how agent-written code is reviewed and tested.
- **For every paper:** disclose the AI tools used and their purpose (code generation, drafting, editing, analysis assistance) in the form the venue asks for. Humans verify every number, citation and claim. AI tools never generate data, participant responses, results or references. References come only from [references.bib](../research/references.bib), checked against their DOI or URL.
- Check each venue's current AI policy at submission; the kickoff research did not record policies other than JOSS's.

## Open access and article processing charges

- **ACM** has published fully open access since 1 January 2026. CHI 2027 charges an APC of $500 (ACM/SIG members) or $750 (non-members) unless the author's institution participates in ACM Open ([CHI 2027 papers](https://chi2027.acm.org/authors/papers/)). Plan for up to $750 per ACM paper (P05; P04, P06 or P08 if they go to ACM venues) until the 2028 terms are published [EST].
- **ICAD:** the 2025 proceedings were published under CC BY-NC 4.0; no APC was recorded in the kickoff notes.
- **JOSS, DAFx, NIME, IEEE VIS, TACCESS and JAES:** charges were not verified in the kickoff research. Check before submission.
- **Funding** for APCs is an open question (OQ-R2 in the [evaluation plan](evaluation-plan.md#open-questions)).

## Submission checklist (every paper)

- [ ] Claims match [contributions.md](contributions.md), at the stated evidence level.
- [ ] Every citation resolves to [references.bib](../research/references.bib); preprints are marked.
- [ ] Release DOI and artifact package ready; spec hashes listed.
- [ ] Ethics approval and preregistration referenced for any study.
- [ ] `authorship.md` with CRediT roles agreed by all authors.
- [ ] AI-usage statement written.
- [ ] Venue template, length, anonymization and accessibility requirements checked against the current call.
- [ ] The paper itself is accessible (tagged PDF, alt text, accessible tables); CHI asks authors to follow the SIGCHI accessible submission guide.
- [ ] Findings used are set to `used-in-paper` in the [findings log](findings-log.md).
