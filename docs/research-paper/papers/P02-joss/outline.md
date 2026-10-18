# P02 · JOSS software paper: outline

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead (paper lead to be named). **Stage:** outline.
> **Related:** [publication plan](../../publication-plan.md) · [contributions](../../contributions.md) · [related work](../../related-work.md) · [roadmap B07](../../../planning/roadmap.md#b07--accessibility-conformance-tutorial-and-co-design-round-1) · [notes/07 §6](../../../research/notes/07-engineering-publication.md) · [repository layout](../../../architecture/02-repository-and-crate-layout.md)

| Field | Value |
|---|---|
| Venue | Journal of Open Source Software (JOSS), rolling submission |
| Target | B07, roadmap milestone 2027-06-01 (est.) |
| Earliest eligible | About six months after the repository goes public. If it goes public in A01 (November 2026), that is about May 2027. |
| Manuscript location | Root `paper/paper.md` and `paper/paper.bib`, as JOSS requires the paper files in the software repository |
| Length | 750–1,750 words |
| Release cited | The release current at submission (expected `v0.5.0-beta`); a new tagged release is archived for a DOI after acceptance |

## Eligibility checklist

From [JOSS submitting](https://joss.readthedocs.io/en/latest/submitting.html) and [JOSS paper format](https://joss.readthedocs.io/en/latest/paper.html), as recorded in [notes/07 §6](../../../research/notes/07-engineering-publication.md).

- [ ] Research software with clear research impact; feature-complete and packaged to community standards (not a minor utility, thin API client or single-function package).
- [ ] At least six months of public history with releases, public issues and pull requests ("A repo dump is not a history").
- [ ] Commit history that passes JOSS's automated checks on commit distribution and burst commits. Development with AI coding agents can produce bursty history; keep a steady cadence of reviewed PRs from A01 onward.
- [ ] Evidence of research use (see "Research impact statement" below).
- [x] OSI-approved licence: MIT OR Apache-2.0 (ADR-0010, accepted 2026-10-18).
- [ ] Issue tracker open without registration.
- [ ] Paper files in the same repository.
- [ ] The submitting author is a major contributor.
- [ ] AI use disclosed; authors confirm human review and human core design decisions.
- [ ] Capacity to respond to reviewers within two weeks and finish changes within four to six weeks.
- [ ] After acceptance: tagged release and archive (Zenodo) for a DOI.

## Sections and word budget

| Section | Words (target) | Content |
|---|---|---|
| Summary | 150–200 | What SONARITHM is for a non-specialist: turning data, graphs and charts into structured, navigable sound from one reproducible specification; desktop app, CLI, Python package and Rust crates on one deterministic core. |
| Statement of need | 250–300 | Researchers, educators and BLV users need sonifications that are reproducible, accessible and exportable. Tools are web libraries or notebooks; desktop tools are abandoned (`budziszewski2025tools`; F-001). Evaluation is rare (`dubus2013systematic`) and software support "not commensurate" with research (`kim2024erie`). Target users and research uses. |
| State of the field | 250–300 | MAIDR (`seo2024maidr`), Chart2Music (`langston_chart2music`), Highcharts Sonification Studio (`cantrell2021highcharts`), Erie (`kim2024erie`), Umwelt (`zong2024umwelt`), STRAUSS (`trayford2025strauss`), sonecules (`reinsch2023sonecules`). **Build vs contribute:** why a new project rather than contributing upstream (licence: GPL-3.0 MAIDR and proprietary Highcharts; platform: browser-only rendering; architecture: no shared real-time and offline path, no provenance manifest; accessibility: STRAUSS has no navigation). Interoperability instead of competition: Erie import, planned MAIDR and Vega-Lite importers. |
| Software design | 300–350 | Spec → Score IR → one render graph; engines as capabilities in one pipeline (not nine programs); manifest and determinism; real-time safety; bindings; trade-offs (webview shell for screen-reader quality; backend traits around volatile dependencies). |
| Research impact statement | 150–250 | Evidence of use: the replication gallery; studies run on the harness (EXP-001 and co-design round 1, if completed); P01 if accepted; any external research use, cited. Only verifiable facts. |
| AI usage disclosure | 80–150 | How AI coding agents were used in development and writing; how code was reviewed and tested; confirmation that humans made the core design decisions and reviewed all text. |
| Acknowledgements | ≤ 50 | BLV co-design partners (with consent), contributors, funding (if any). |
| References | — | Full venue names, from [references.bib](../../../research/references.bib). |
| **Total** | **≈ 1,200–1,600** | Within the 750–1,750 limit. |

## Key references

`hermann2008taxonomy`, `enge2023unified`, `kim2024erie`, `zong2024umwelt`, `seo2024maidr`, `langston_chart2music`, `cantrell2021highcharts`, `trayford2025strauss`, `reinsch2023sonecules`, `dubus2013systematic`, `budziszewski2025tools`, `smith2005effects`, `sharif2021understanding`.

## Evidence needed before submission

| Need | Source | Status |
|---|---|---|
| Six months of public history | Repository from A01 | pending |
| Releases with DOIs | `v0.1.0-alpha` (A07), `v0.2.0-alpha` (A08), `v0.5.0-beta` (Gate B) | pending |
| Research use | EXP-001, EXP-012, external users | pending; depends on ethics approval |
| Documentation and tests to community standards | Docs site, CI, coverage of the public API | pending |
| Statement-of-need facts re-verified | F-001 and the landscape table re-checked in the submission month | pending |

## Risks

| Risk | Mitigation |
|---|---|
| No evidence of research use by May–June 2027 | Prioritize EXP-001 after approval; invite early research users from the ICAD community; otherwise defer and record why at Gate B |
| Bursty commit history flagged by JOSS checks | Regular, reviewed PRs; avoid large squashed dumps |
| Scope looks like several projects | Present one platform with engines as capabilities, as the engine catalogue does |
| AI-usage concerns | A clear, specific disclosure; human review recorded in PRs |
