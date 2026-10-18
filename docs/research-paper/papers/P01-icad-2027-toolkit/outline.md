# P01 · ICAD 2027 toolkit paper: outline

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead (paper lead to be named). **Stage:** outline.
> **Related:** [publication plan](../../publication-plan.md) · [contributions](../../contributions.md) · [evaluation plan](../../evaluation-plan.md) · [related work](../../related-work.md) · [roadmap A08](../../../planning/roadmap.md#a08--research-instrumentation-v0-first-paper) · [benchmarks log](../../benchmarks-log.md)

| Field | Value |
|---|---|
| Venue | ICAD 2027, Athens, Greece. Dates TBA. |
| Deadline | ~Mar 2027 [EST] (ICAD 2026 full papers: 20 Mar 2026). Roadmap milestone 2027-03-15 (est.). Re-check monthly. |
| Paper type | System/toolkit paper |
| Release cited | `v0.2.0-alpha` (A08), with its Zenodo DOI |
| Claims used | CL-02, CL-03, CL-04, CL-06, CL-12; CL-05 if P03 is merged into this paper ([contributions](../../contributions.md)) |
| Decision at Gate A | Submit, merge with P03, or defer with a recorded reason (A08 exit criterion) |

## Scope boundary

At the ICAD 2027 deadline only Stage A exists: the core, NUMISONA, the CLI, the Python package, export, the harness v0 and the replication gallery. The desktop app and AUDAXIS navigation arrive in Stage B. P01 therefore **must not claim** accessible navigation or BLV usability (CL-07), and it reports no perceptual results unless EXP-001 has run under ethics approval. It is a paper about reproducible, specification-driven sonification infrastructure.

## Title options

1. SONARITHM: A Deterministic, Specification-Driven Engine for Reproducible Data Sonification
2. Spec In, Score Out: Compiling Declarative Sonification Specifications for Real-Time and Offline Rendering
3. Reproducible by Construction: A Sonification Compiler That Re-creates Classic Auditory Display Designs
4. One Specification, Identical Sound: Operationalizing Hermann's Reproducibility Criterion in a Native Sonification Toolkit

## Abstract draft

*About 180 words. Numbers in brackets are placeholders filled from the benchmarks log and the gallery.*

> Sonification research has well-established perceptual results, yet software support has lagged behind them, and few published mappings were ever evaluated. Tools are mostly web libraries or notebooks, and none couples a declarative specification with reproducible offline export and live rendering. We present SONARITHM, an open-source toolkit with a Rust core that compiles a declarative specification, expressed in the substrate–mark–channel vocabulary, into a canonical Score representation. The same render graph drives real-time playback and offline export, and every render carries a manifest with the engine version, specification hash, data fingerprint and seed. Defaults follow a catalogue of evidence-backed design rules, each with an identifier that appears in compiler diagnostics. Following established strategies for evaluating toolkits, we (1) re-create classic designs, including Smith and Walker's auditory context cues, Dombois's seismic audification and examples from the Erie grammar, from committed specifications in [N] lines each on average; (2) measure interactive latency of [x] ms (p95), offline rendering at [y]× real time and cross-platform agreement within [z] dBFS; and (3) report targeted usage cases. SONARITHM is available under MIT OR Apache-2.0 with a citable release.

## Contributions

| # | Contribution | Claim | Evidence in the paper |
|---|---|---|---|
| C1 | A native compiler and render engine: declarative spec → Score IR → one render graph for real-time and offline output, with CLI and Python front ends. | CL-02, CL-12 | System description; technical benchmarks |
| C2 | Reproducibility by construction: a provenance manifest with every render, and measured determinism across operating systems. | CL-02, CL-03 | Determinism results; manifest schema |
| C3 | A replication gallery that re-creates classic designs from committed specs. | CL-04 | Gallery table with published vs reproduced parameters |
| C4 | Evidence-traceable defaults: rule IDs in defaults and diagnostics. | CL-06 | Rule catalogue excerpt; lint examples |
| C5 (optional) | The open evaluation harness v0, if P03 is merged. | CL-05 | Generators and tasks |

## Structure

1. **Introduction.** The gap between perception research and software support (`kim2024erie`); the evaluation gap (`dubus2013systematic`); abandoned tools (`budziszewski2025tools`). Contributions C1–C4.
2. **Background and related work.** Definitions and Hermann's criteria (`hermann2008taxonomy`); vocabulary (`enge2023unified`); grammars and toolkits (`kim2024erie`, `zong2024umwelt`, `trayford2025strauss`, `reinsch2023sonecules`, `cantrell2021highcharts`); capability comparison from [notes/01](../../../research/notes/01-tools-landscape.md).
3. **Design goals.** Hermann's criteria as requirements (SDR-01..SDR-06); evidence-backed defaults (SDR-07..SDR-42); separation of Analysis and Presentation modes (SDR-36).
4. **System.** Spec language and vocabulary; compiler pipeline (ingest, transform, analyze, encode, shape); Score IR; render graph and export; manifest; CLI and Python.
5. **Evidence-backed defaults.** Concept table and polarity flip; pitch range; context cues; lint diagnostics with rule IDs; which defaults are `INFER` hypotheses.
6. **Evaluation.**
   - 6.1 Demonstration: replication gallery.
   - 6.2 Technical performance: latency, real-time factor, compile time, determinism.
   - 6.3 Targeted usage: author case studies; EXP-014 walkthrough if approved in time.
7. **Discussion and limitations.** No perceptual validation yet; navigation layer in Stage B; `INFER` rules untested; landscape claims dated October 2026.
8. **Conclusion.**
- **Statements:** artifact availability, ethics (if EXP-014 ran), AI usage.

## Figures and tables

| ID | Content | Source |
|---|---|---|
| F1 | Pipeline: spec → Score IR → real-time and offline render, with the manifest | [Architecture overview §5](../../../architecture/01-overview.md) |
| F2 | A spec next to its Score timeline (audio as supplementary material) | Gallery item, Smith & Walker |
| F3 | Gallery: published parameters vs reproduced parameters | Replication gallery |
| F4 | Latency distribution and real-time factor per OS | [Benchmarks log](../../benchmarks-log.md) |
| F5 | Cross-OS null-test residuals for the golden set | Benchmarks log, EXP-007 |
| T1 | Capability comparison with existing tools (export, MIDI, streaming, spec, provenance, licence) | [Research report](../../../research/research-report.md); [notes/01](../../../research/notes/01-tools-landscape.md) |
| T2 | Lines of spec per gallery item | Replication gallery |

## Evaluation plan (Ledo et al.)

Following `ledo2018evaluation`, the evaluation matches the claims instead of defaulting to a usability study. The strategy names below follow the summary in [notes/07 §7](../../../research/notes/07-engineering-publication.md), which records them from prior knowledge; verify against the paper before submission.

| Strategy | What P01 does | Supports |
|---|---|---|
| Demonstration | Replicate prior designs: Smith & Walker context cues (GEN-002), Dombois audification at 2,200×, Nees & Walker density stimuli (GEN-001), Tucker Brown et al. transit stimuli (GEN-003), the Erie gallery subset, Walker (2007) polarity presets. Report spec length and deviations. | C3, C4 |
| Technical performance | Key-to-cue latency (p50/p95), xruns in soak, real-time factor, compile time, Score IR identity and null-test residuals across Windows, macOS and Linux. | C1, C2 |
| Usage (targeted) | Author case studies: build three new designs from scratch and report steps and time. If ethics approval arrives in time, EXP-014: a walkthrough in which about five researchers author a spec. | C1 |
| Heuristic / analytic | Check the system against Hermann's four criteria point by point. | C2 |

## Required phases

| Phase | What P01 needs from it |
|---|---|
| A01 | Public repository and CI (cross-OS matrix used for determinism); CI baseline in the benchmarks log |
| A02 | Latency and determinism spike results; render-graph decision |
| A03 | Spec v0.1, Score IR, schema (C1 description) |
| A04 | Data layer and fingerprints (manifest) |
| A05 | Render engine, export, manifest, golden-audio harness, soak and RTF benchmarks (C1, C2) |
| A06 | NUMISONA defaults with rule-ID tests; Erie importer subset (C3, C4) |
| A07 | CLI, Python package, `v0.1.0-alpha` with Zenodo DOI |
| A08 | Harness v0, replication gallery, `sonarithm study run`, technical benchmarks, `v0.2.0-alpha`; submission |

## Risks

| Risk | Mitigation |
|---|---|
| The ICAD 2027 deadline is earlier than estimated or its call differs | Monthly checks; keep a submittable draft from A06 onward |
| Gallery items deviate from the published stimuli | Document every deviation next to the source; claim expressiveness, not perceptual equivalence |
| Cross-OS determinism fails on some CPU | Report it honestly with tolerances; log a finding; fix or scope the claim |
| Reviewers expect a user study | Argue from Ledo et al.; include targeted usage; position perceptual studies as P03 and later work |
| Ethics approval arrives too late for EXP-014 | P01 does not depend on it; the usage section uses author case studies |
| Landscape claims age (F-001) | Re-verify tool status in the month of submission |
| Scope creep from Stage B features | The scope boundary above is part of the outline |

## Checklist against the ICAD template

ICAD 2026 accepted submissions through EasyChair with Office and LaTeX templates. The ICAD 2027 call has not been published. Verify each item when it is.

- [ ] Template and version for ICAD 2027 (Office or LaTeX)
- [ ] Page limit and paper category (full paper or other)
- [ ] Review model (anonymous or not) and how to cite our own repository under it
- [ ] Policy for supplementary audio and video, and where to host them
- [ ] Copyright and licence terms (the 2025 proceedings used CC BY-NC 4.0)
- [ ] Accessibility of the PDF: tagged structure, alt text for figures, accessible tables
- [ ] Release DOI and artifact package linked
- [ ] Every citation resolves to [references.bib](../../../research/references.bib), with incomplete records completed
- [ ] AI-usage statement in the form the venue requires
- [ ] Claims match [contributions.md](../../contributions.md) at submission time
