# Claimed contributions

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead. **Review:** at every phase review and stage gate.
> **Related:** [publication plan](publication-plan.md) · [evaluation plan](evaluation-plan.md) · [hypotheses](hypotheses.md) · [findings log](findings-log.md) · [research report, Conclusion](../research/research-report.md#conclusion) · [engines](../product/engines.md)

This file is the gate between what SONARITHM intends and what it may claim. A paper, README, release note or talk may state a claim only at the evidence level this file records for it. At baseline almost nothing is supported by SONARITHM's own evidence, because nothing has been built or tested yet. That is the honest starting point.

**Status vocabulary.** `aspiration` (no supporting evidence; must not be claimed), `literature-only` (the problem or gap is documented, SONARITHM's answer is not yet shown), `in progress` (evidence is being produced), `supported` (evidence at the required level exists, with links), `refuted` (evidence contradicts the claim; reword or drop it). Evidence levels are defined in the [track README](README.md#evidence-levels).

## Claims

| ID | Claim | Evidence required | Current status | Papers |
|---|---|---|---|---|
| CL-01 | SONARITHM is the first actively maintained, native, cross-platform sonification application and engine. | Landscape review re-verified at submission (F-001); released, installable builds on three OSes; maintenance record. | literature-only: the gap is documented; the software does not yet exist | P01, P02, P05 |
| CL-02 | One deterministic core renders the same Score identically on desktop, CLI, Python and (later) web; audio agrees within tolerance across OS and CPU, and between real-time and offline paths. | EV-C: identical Score IR hashes across the CI matrix; null-test residual ≤ −90 dBFS and LUFS Δ ≤ 0.1; EXP-007 (H-08), including the ABX test. | aspiration until A02 and A05 report | P01, P07 |
| CL-03 | Every render carries a spec and provenance manifest, which makes Hermann's reproducibility criterion operational. | EV-C: schema and manifest shipped; any gallery item re-rendered from its manifest reproduces the Score; comparison with surveyed tools (F-001, F-007). | literature-only: design specified in ADR-0013; not implemented | P01, P02 |
| CL-04 | The spec reproduces classic designs (Smith & Walker context cues, Dombois audification, Erie examples) from committed specs. | EV-C: replication-gallery items meet the acceptance criteria in the [evaluation plan](evaluation-plan.md#replication-gallery), with documented deviations. | in progress (A08-T02): Smith & Walker, Nees & Walker, Tucker Brown and Erie items render reproducibly from committed specs with their records and documented deviations (F-114); Dombois needs the audification mark (C03). Claim it only for the items that exist, and as reproduction of stimuli, not of results | P01 |
| CL-05 | SONARITHM provides an open evaluation benchmark: parametric stimulus generators, standard tasks and logged metrics. | EV-C: GEN-001..004 released and validated against published stimulus descriptions; tasks documented; at least one study (EXP-001) run on the harness. | aspiration until A08; the gap is documented (F-002) | P03 |
| CL-06 | Defaults are evidence-backed and traceable: each default cites a rule ID, and the perceptual lint reports rule IDs. | Design-rules catalogue (exists); unit tests reference rule IDs (A06 exit); lint diagnostics carry IDs (C05). Note: "evidence-backed" means "backed by literature" for most rules and "hypothesis" for `INFER` rules. | literature-only: the catalogue exists; the code does not | P01, P05 |
| CL-07 | BLV screen-reader users can install, explore, configure and export without sighted help. | EV-D/EV-B: co-design round 1 meets Gate B (≥ 80% unaided completion of core tasks); scripted task suite passes with NVDA, JAWS, VoiceOver and Orca; ACR published. | aspiration until B07 | P05 |
| CL-08 | Chart ingestion prefers exact data and labels every estimate with its provenance and confidence. | EV-C: C01 exact corpus ≤ 0.5% relative error with correct provenance class; C02 raster corpus median ≤ 2%; every estimate labelled in description and table (tests). | aspiration until C01–C02 | P05, P08 |
| CL-09 | SONARITHM provides the first magnitude-estimation polarity data for graph concepts. | EV-A: EXP-004 (H-05), preregistered, BLV and sighted samples; prior-art search re-run before submission (F-008). | literature-only: the gap is documented | P04 |
| CL-10 | SONARITHM provides the first controlled evaluation of LLM-suggested sonification mappings. | EV-A: EXP-006 (H-07); prior-art search re-run before submission (F-009). | literature-only: the gap is documented | P06 |
| CL-11 | A distinct audio rendering of estimated data helps BLV users detect estimates without hurting trend reading. | EV-A: EXP-005 (H-06), after co-design. | aspiration; the research report calls it "promising but untested with blind users" | P08 |
| CL-12 | The engine meets interactive latency and offline-speed budgets (≤ 50 ms p95 key-to-cue; ≥ 50× real time; 0 xruns in a 10-minute soak). | EV-C: benchmarks-log rows on reference hardware, with manifests and method notes. | aspiration until A02/A05 | P07, P01 |
| CL-13 | Monitoring mode supports anomaly detection with lower workload than discrete alerts. | EV-A: EXP-008 (H-09). Until then only the literature (F-014) may be cited, as literature. | literature-only | P05 |
| CL-14 | The built-in tutorial substantially reduces novices' point-estimation error. | EV-A: EXP-009 (H-10). | literature-only (F-013 is about training in general, not this tutorial) | P05 |
| CL-15 | "Hear the patterns": listeners can perceive trends, overviews and changes in monitored streams with SONARITHM. | Literature supports the general claim for trends, overviews and monitoring (F-003, F-012, F-014). SONARITHM-specific support needs EXP-001, EXP-002 and EXP-008. | literature-only; usable as a tagline, not as a research claim | All |
| CL-16 | "Discover what the graphs reveal": sonification reveals patterns users would otherwise miss. | Studies showing discovery by ear, including at moderate SNR and with BLV users, compared with visual baselines. Current literature points the other way at low SNR (F-010). | aspiration; must not appear as a claim in any abstract, paper or release note | none until evidence exists |
| CL-17 | SONARITHM makes networks (node-link graphs) accessible by sound. | No polarity data (F-008) and no BLV study of network sonification exist. Needs EXP-004 plus a BLV usage study of network navigation. | aspiration | none until evidence exists |
| CL-18 | Musical presentation (AXONOTE) improves engagement without misrepresenting data. | Evidence is mixed: aesthetic gains, accuracy costs (research report). Presentation mode is labelled and unquantized pitch stays in Analysis mode (SDR-10). P09 reports qualitative practitioner evidence only. | aspiration | P09 (qualitative only) |

## What SONARITHM does not claim

These follow directly from the research and the design rules. Papers must not imply them.

- That equal-loudness compensation improves accuracy (SDR-24; Fitzpatrick & Neff 2023 found no effect).
- That any `INFER` rule (SDR-20, SDR-26, SDR-28, SDR-33, SDR-39) is established. These are hypotheses H-02, H-04, H-11, H-12 and a candidate for SDR-33.
- That sonification replaces visual inspection. The evidence supports it as a complement ([research report](../research/research-report.md), "Monitoring works; 'discovery' is the weakest-supported promise").
- That ML chart extraction is accurate. It is presented as an editable estimate (F-006).
- That pitch-time mapping, basic keyboard navigation, LLM chat or a free licence are differentiators. The research report lists them as table stakes.

## Tagline discipline

From the [research report, Conclusion](../research/research-report.md#conclusion) and the [engine catalogue](../product/engines.md):

- **Use** "Hear the patterns." The evidence supports hearing trends, overviews and changes in monitored streams, and shows that audio added to visuals raises engagement.
- **Do not assert** "Discover what the graphs reveal." Use it only as an aspiration, framed as a complement to looking, until SONARITHM's own studies support it. Subtle signals go undetected by ear without training, and expert visual skill does not transfer to listening.
- **If "graphs" means networks,** say plainly that there is no polarity data and no blind-user evidence yet, and that SONARITHM's studies aim to produce it.

## Defensible differentiators, in order

From the research report: one deterministic core across desktop, CLI, Python and WebAssembly (CL-02); a spec and provenance with every render (CL-03); chart ingestion that prefers exact data and labels estimates (CL-08); accessible streaming and monitoring (CL-07, CL-13); maintained model-based and multivariate engines where alternatives are dormant (Stage D; no claim yet); and a perceptual lint and benchmark harness (CL-05, CL-06).

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline: CL-01..CL-18 recorded; no claim yet supported by SONARITHM's own evidence. | Research lead |
