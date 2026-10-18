# Hypotheses

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [track README](README.md) · [findings log](findings-log.md) · [experiments](experiments/README.md) · [evaluation plan](evaluation-plan.md) · [design rules](../research/design-rules.md) · [roadmap](../planning/roadmap.md)

Each hypothesis is a testable claim that SONARITHM's defaults or papers depend on. Most come from rules marked `INFER` in the [design rules](../research/design-rules.md), from single-study defaults that need replication with BLV listeners, or from gaps the field admits to. A hypothesis changes a rule only through an experiment (`EXP-###`) and the rule's change control.

**Lifecycle:** `proposed` → `preregistered` (OSF link recorded) → `running` → `supported` | `not supported` | `inconclusive`. A hypothesis that is no longer worth testing becomes `retired`, with a reason. IDs are never reused.

**Conventions.** Factor levels and sample sizes below are planning values, not commitments. They are fixed in each experiment's preregistration after a pilot and a power analysis ([EXP-000 template](experiments/EXP-000-template.md)). "BLV" means blind and low-vision participants. All user studies follow the [evaluation plan](evaluation-plan.md) and need ethics approval first.

## Summary

| ID | Short form | Rules | Experiment | Paper | Testable from | Status |
|---|---|---|---|---|---|---|
| H-01 | Dynamic reference tone and tick clicks reduce point-estimation error | SDR-16, SDR-17 | EXP-001 | P03 | A08 | proposed |
| H-02 | Above ~10 events/s, glides preserve trend-ID accuracy | SDR-20 | EXP-002 | P03 ext. | A08 | proposed |
| H-03 | 400–1400 Hz is non-inferior to 65–1480 Hz | SDR-08 | EXP-002 | P03 ext. | A08 | proposed |
| H-04 | Stream identification degrades beyond 3–4 streams | SDR-26 | EXP-003 | P03 ext. | D01 | proposed |
| H-05 | Graph concepts have majority polarities that differ by population | SDR-41 | EXP-004 | P04 | A08 | proposed |
| H-06 | A distinct timbre for estimated data helps BLV detection | — (P08) | EXP-005 | P08 | C02 | proposed |
| H-07 | LLM mappings agree less with empirical priors than the rule engine | — (P06) | EXP-006 | P06 | D04 | proposed |
| H-08 | Real-time and offline renders are equivalent across OS/CPU | SDR-01 | EXP-007 | P07 | A05 | proposed |
| H-09 | An ambient soundscape beats discrete alerts in dual-task monitoring | SDR-35 | EXP-008 | P05 | B06 | proposed |
| H-10 | The 15-minute tutorial cuts novices' error by at least 30% | SDR-37 | EXP-009 | P05 | B07 | proposed |
| H-11 | Concurrent tones closer than ~1 ERB are read less accurately | SDR-28 | EXP-010 | P03 ext. | A08 | proposed |
| H-12 | A comfort budget lowers annoyance without costing accuracy | SDR-39 | EXP-011 | P03 ext. | C05 | proposed |

"P03 ext." is the pacing × range × context extension of the benchmark paper, targeted at ICAD 2028 or ACM TACCESS ([publication plan](publication-plan.md)).

---

### H-01 · Dynamic reference tone and tick clicks reduce point-estimation error

- **Statement:** A dynamic y-axis reference tone combined with x-axis tick clicks reduces point-estimation RMS error compared with no context, for both BLV and sighted listeners. This replicates Smith & Walker (2005).
- **Rationale:** The dynamic reference tone cut RMS error from 18.09 to 12.14; a static opening-value reference (17.46) did not help; clicks aided time calibration for discrete tones (F-003). The evidence is one study of sighted undergraduates, and SDR-16 and SDR-17 rest on it. Walker & Nees (2011) note that context-cue configurations are underexplored. Sources: `smith2005effects`, `walker2011theory`.
- **Linked rules and findings:** SDR-16, SDR-17 · F-003
- **Operationalization:**
  - *Design:* within-subjects, context condition counterbalanced (Latin square); pre-test baseline as covariate, following Smith & Walker's ANCOVA.
  - *IV:* context = {none, x-clicks, dynamic y-reference, clicks + dynamic y-reference, static opening-value reference (negative control)}; group = {BLV, sighted}.
  - *Stimuli:* GEN-002, reproducing the published parameters: 20 tones of 0.5 s over 10 s, 40 ms ramps, sine data tones 200–1900 Hz, triangle-wave reference tone, dollar mapping $ = c·f^0.9256 with c = 0.0765.
  - *Task:* report the dollar value at a queried time point.
  - *DV:* RMS error (primary); response time, replays, confidence (secondary).
  - *Analysis:* linear mixed model on squared error with condition, group and their interaction; random intercepts for participant and stimulus.
- **Planned experiment:** EXP-001 (planned)
- **Target paper:** P03 (replication run on the benchmark harness)
- **Testable from:** A08 (offline stimuli from the replication gallery and `sonarithm study run`); B03 for the interactive AUDAXIS version
- **Consequence for rules:** If supported, SDR-16 and SDR-17 gain SONARITHM evidence, including for BLV listeners. If the effect is absent for BLV listeners, the BLV profile defaults are reviewed.
- **Status:** proposed

### H-02 · Above ~10 events per second, continuous glides preserve trend-identification accuracy

- **Statement:** Above about 10 events per second, rendering a series as a continuous glide preserves trend-identification accuracy, whereas discrete notes lose accuracy.
- **Rationale:** Studied densities stop at 8 points per second, with only a small effect of density; trend reversals matter more. The ~100 ms metrical-grouping floor suggests a mark switch near 10 events/s, but no study has tested it (F-012). Sources: `nees2008density`, `london2012hearing`, `repp2005sensorimotor`.
- **Linked rules and findings:** SDR-20 (INFER), SDR-19, SDR-21 · F-012
- **Operationalization:**
  - *Design:* within-subjects factorial, part of the pacing × range × context study.
  - *IV:* rate (candidate levels 2, 4, 8, 12, 16, 24 events/s); mark = {discrete notes, continuous glide}; trend reversals = {0, 1, 3}.
  - *Stimuli:* GEN-001 (reversals × density, after Nees & Walker 2008).
  - *Task:* identify the overall trend; point estimation as a secondary task.
  - *DV:* trend-identification accuracy (primary); RMS error, replays.
  - *Analysis:* logistic mixed model with rate × mark × reversals; segmented regression to locate where discrete-note accuracy breaks; non-inferiority of glides against the 2–4 events/s baseline above the break.
- **Planned experiment:** EXP-002 (planned)
- **Target paper:** P03 extension
- **Testable from:** A06 (mark switching, flagged experimental) and A08 (harness)
- **Consequence for rules:** If supported, SDR-20 can leave `INFER` with an evidence-based threshold. If not, mark switching stays opt-in and the lint threshold of SDR-19 is reviewed.
- **Status:** proposed

### H-03 · A 400–1400 Hz range is non-inferior to 65–1480 Hz for BLV and sighted listeners

- **Statement:** The default 400–1400 Hz pitch range is non-inferior to the wide 65–1480 Hz range on line-chart tasks, for BLV and for sighted listeners separately.
- **Rationale:** One study (n = 48, sighted) found the narrow range as good as the wide one; amplitude compensation had no effect (F-004). SDR-08 depends on it, and individual hearing differences (for example high-frequency hearing loss, reported in MAIDR's sample) are large. Sources: `fitzpatrick2023frequency`, `seo2024maidr`.
- **Linked rules and findings:** SDR-08, SDR-24 · F-004
- **Operationalization:**
  - *Design:* within-subjects, part of the pacing × range × context study; range order counterbalanced.
  - *IV:* range = {400–1400 Hz, 65–1480 Hz}; group = {BLV, sighted}; ISO 226 compensation = {on, off} as an exploratory factor.
  - *Task:* trend identification and point estimation on line charts.
  - *DV:* trend accuracy and RMS error (primary); perceived difficulty (secondary). Self-reported hearing profile is recorded.
  - *Analysis:* non-inferiority test per group with a margin fixed at preregistration from pilot variance.
- **Planned experiment:** EXP-002 (planned)
- **Target paper:** P03 extension
- **Testable from:** A06 (pitch-range control) and A08
- **Consequence for rules:** If non-inferiority holds in both groups, SDR-08's confidence rises. If it fails for BLV listeners, the BLV profile gets a different default range.
- **Status:** proposed

### H-04 · Concurrent-series identification degrades beyond 3–4 streams despite separation cues

- **Statement:** Identification of concurrent data series degrades beyond 3–4 simultaneous streams, even when the streams are separated by timbre, stereo position and staggered onset.
- **Rationale:** Concurrent-earcon identification falls as their number grows, and timbre and onset differences only partly restore it. No upper bound for data streams is known, so SDR-26's cap is inferred (F-015). Stereo cannot carry information alone (A11Y-14). Sources: `mcgookin2004understanding`, `bregman1990auditory`.
- **Linked rules and findings:** SDR-26 (INFER), SDR-27, A11Y-14 · F-015
- **Operationalization:**
  - *Design:* within-subjects.
  - *IV:* stream count = {1..6}; separation = {timbre, timbre + pan, timbre + pan + onset stagger}; output = {stereo, mono} to check A11Y-14.
  - *Task:* identify the trend of a cued target series, or which series changed.
  - *DV:* identification accuracy (primary); response time.
  - *Analysis:* logistic mixed model; test the drop between 3–4 and 5–6 streams against a preregistered accuracy criterion.
- **Planned experiment:** EXP-003 (planned)
- **Target paper:** P03 extension
- **Testable from:** D01 (TONALENS multi-stream scheduler; the roadmap's D01 exit signal names this experiment). Pilot stimuli can be rendered earlier with multi-track NUMISONA specs.
- **Consequence for rules:** SDR-26's cap is confirmed, moved, or replaced by a lint that depends on the separation cues used.
- **Status:** proposed

### H-05 · Graph concepts have majority polarities, and they differ between BLV and sighted listeners

- **Statement:** For each of node degree, centrality, edge weight and path length, a majority polarity exists for pitch. For at least one of these concepts, the majority polarity or the scaling differs between BLV and sighted listeners.
- **Rationale:** No polarity data exist for graph concepts (F-008). Polarity is concept- and population-specific, and BLV listeners differed on "number of dollars" (F-011). SDR-41 forbids shipping a new concept's default without magnitude estimation. Sources: `walker2002magnitude`, `walker2007consistency`, `walker2010universal`, `ferguson2018perceptual`.
- **Linked rules and findings:** SDR-41, SDR-12..SDR-15, SDR-33 · F-008, F-011
- **Operationalization:**
  - *Design:* Walker-style modulus-free magnitude estimation; concept blocks counterbalanced.
  - *IV:* data concept = {degree, centrality, edge weight, path length}, plus one known concept (for example velocity) as a validity anchor; sound dimension = {frequency; tempo as optional second dimension}; group = {BLV, sighted}.
  - *Procedure:* a short, accessible explanation of each graph concept with a comprehension check before its block.
  - *DV:* per-participant log–log slope: sign (polarity) and magnitude (scaling).
  - *Analysis:* majority-polarity rule per concept with a preregistered criterion; group difference in polarity by logistic regression per concept; slopes by mixed model. The anchor concept must reproduce Walker's published polarity.
- **Planned experiment:** EXP-004 (planned)
- **Target paper:** P04
- **Testable from:** A08 (magnitude-estimation task in the study runner); scheduled in D06
- **Consequence for rules:** Results populate the concept table for graph concepts. A concept without a clear majority ships flagged, like money (SDR-14).
- **Status:** proposed

### H-06 · A distinct timbre for estimated data improves BLV detection of estimates without hurting trend accuracy

- **Statement:** Rendering low-confidence (estimated) data points in a distinct timbre improves BLV listeners' detection of estimated regions, without reducing trend-identification accuracy.
- **Rationale:** ML line-chart extraction is weak (F-006), so PLOTSONA labels every estimate. Rendering low-confidence regions distinctly is "promising but untested with blind users", and no study measures BLV trust or error detection with uncertainty cues. Sources: research report ("PLOTSONA must prefer exact data and label every estimate"); [notes/06 §7–§8](../research/notes/06-chart-extraction-ai.md); `onechart2024`.
- **Linked rules and findings:** A11Y-14, A11Y-24 (non-audio equivalents must remain) · F-006
- **Operationalization:**
  - *Design:* co-design rounds first, to choose candidate renderings; then a within-subjects task study.
  - *IV:* uncertainty rendering = {none (speech hedging only), timbre change, earcon marker}; proportion of estimated points.
  - *Stimuli:* GEN-008 (series with injected estimated regions and provenance classes).
  - *Task:* trend identification, then "where was the data estimated?".
  - *DV:* detection sensitivity (d′) for estimated regions (primary); trend accuracy (non-inferiority); trust calibration (confidence vs correctness); NASA-TLX.
  - *Analysis:* mixed models; non-inferiority on trend accuracy with a preregistered margin.
- **Planned experiment:** EXP-005 (planned)
- **Target paper:** P08
- **Testable from:** C02 (per-point provenance and optional timbre experiment); full study in D04/D06
- **Consequence for rules:** If supported, a new rule for uncertainty rendering is proposed. If not, uncertainty stays in speech, text and the table only.
- **Status:** proposed

### H-07 · LLM-suggested mappings agree less with empirical priors than the rule engine and yield lower accuracy

- **Statement:** Mappings suggested by large language models agree less with empirical polarity priors than mappings from SONARITHM's transparent rule engine, and sonifications built from them yield lower task accuracy.
- **Rationale:** No peer-reviewed evaluation of LLM-suggested mappings exists (F-009). Designer intuition is unreliable (F-011), and LLM outputs may reproduce "intuitive" but ineffective mappings. The research report recommends a rule engine with an LLM only ranking or explaining. Sources: `hommel2025ai`, `walker2005mappings`, `walker2007consistency`.
- **Linked rules and findings:** SDR-12..SDR-15, SDR-25, SDR-41 · F-009, F-011
- **Operationalization:**
  - *Part A (agreement):* for a fixed set of datasets and concepts with known priors (Walker 2007, plus H-05 results when available), collect mappings from several LLMs (pinned model IDs, fixed prompts and sampling settings, repeated samples) and from the rule engine. DV: agreement with majority polarity; rule-conformant channel choice (for example categorical → timbre).
  - *Part B (performance):* within-subjects task study on sonifications built from each mapping source. DV: trend accuracy, RMS error, response time.
  - *Analysis:* agreement proportions with confidence intervals; mixed models for Part B.
- **Planned experiment:** EXP-006 (planned)
- **Target paper:** P06
- **Testable from:** D04 (AI assist and local LLM sidecar); the rule engine exists from A06. Part A can start earlier with external models, because it needs no participants.
- **Consequence for rules:** Informs whether any LLM-driven suggestion may be enabled by default. It may not change polarity defaults by itself.
- **Status:** proposed

### H-08 · Real-time and offline renders of the same Score are equivalent across OS and CPU

- **Statement:** Real-time and offline renders of the same Score are numerically within tolerance and perceptually indistinguishable, across operating systems and CPUs.
- **Rationale:** Reproducibility is a definitional criterion of sonification (Hermann 2008: "structurally identical" output) and SONARITHM's main differentiator. Floating-point and SIMD results differ across CPUs, so equivalence must be tested with tolerances. Sources: `hermann2008taxonomy`; [notes/07 §6](../research/notes/07-engineering-publication.md); [ADR-0013](../decisions/0013-determinism-and-provenance.md).
- **Linked rules and findings:** SDR-01, SDR-02 · (engineering findings expected from A02)
- **Operationalization:**
  - *Numerical:* identical Score IR hashes across OS/CPU; null-test residual and loudness difference against the budgets in [architecture overview §7](../architecture/01-overview.md) (null-test residual ≤ −90 dBFS, LUFS Δ ≤ 0.1). Real-time output is captured at the render-graph output, before the device.
  - *Perceptual:* ABX test on pairs (offline vs real-time capture; OS A vs OS B). DV: proportion correct.
  - *Analysis:* equivalence test against chance with preregistered bounds.
- **Planned experiment:** EXP-007 (planned; numerical part needs no participants)
- **Target paper:** P07
- **Testable from:** A02 (determinism spike) and A05 (golden-audio harness)
- **Consequence for rules:** Supports the reproducibility claims (CL-02). A failure becomes a blocking engineering finding.
- **Status:** proposed

### H-09 · An ambient soundscape beats discrete alerts in dual-task monitoring

- **Statement:** In a dual task, SONARITHM's Monitoring-mode ambient soundscape yields higher anomaly detection at lower NASA-TLX workload than discrete alerts. This replicates Hildebrandt et al. (2016).
- **Rationale:** One within-subjects study (n = 18) and SoNSTAR support continuous soundscapes; no independent replication exists (F-014). Sources: `hildebrandt2016continuous`, `debashi2018sonification`, `hart1988nasatlx`.
- **Linked rules and findings:** SDR-35 · F-014
- **Operationalization:**
  - *Design:* within-subjects, condition order counterbalanced.
  - *IV:* monitoring display = {visual only, visual + discrete alerts, visual + ambient soundscape}. A BLV variant compares audio alerts with the soundscape, with no visual condition.
  - *Tasks:* primary arithmetic task; secondary monitoring of a simulated process stream (GEN-009).
  - *DV:* anomaly hit rate and detection latency (primary); false alarms, primary-task performance, NASA-TLX, helpfulness and pleasantness ratings.
  - *Analysis:* mixed models; primary-task non-inferiority across conditions.
- **Planned experiment:** EXP-008 (planned)
- **Target paper:** P05 (as the usage evidence for SONOTRACE), or a standalone paper decided at triage
- **Testable from:** B06 (SONOTRACE Monitoring mode)
- **Consequence for rules:** SDR-35 gains an independent replication, or is downgraded.
- **Status:** proposed

### H-10 · The built-in 15-minute tutorial reduces novices' point-estimation error by at least 30%

- **Statement:** Completing SONARITHM's built-in 15-minute interactive tutorial reduces novices' point-estimation RMS error by at least 30% relative to their pre-test.
- **Rationale:** About 20 minutes of training can halve point-estimation error (F-013), and training and context effects were comparable but not additive (F-003). The tutorial is a product feature (SDR-37, A11Y-30), so its effect should be measured. Sources: `walker2011theory`, `smith2005effects`.
- **Linked rules and findings:** SDR-37, A11Y-30 · F-003, F-013
- **Operationalization:**
  - *Design:* pre-test / post-test with a control group that spends equal time on a non-training activity (between-subjects factor).
  - *IV:* tutorial = {tutorial, control}; group = {BLV, sighted}. Context cues stay at defaults; context on/off is exploratory, given the non-additivity result.
  - *DV:* relative change in RMS error from pre-test to post-test (primary); response time, confidence, SUS for the tutorial.
  - *Analysis:* mixed model with the pre-test as covariate. The claim is supported if the tutorial group improves significantly more than control and the lower confidence bound of its relative reduction is at least 30%.
- **Planned experiment:** EXP-009 (planned)
- **Target paper:** P05
- **Testable from:** B07 (tutorial delivered)
- **Consequence for rules:** SDR-37 gains direct evidence for the shipped tutorial; the tutorial is revised if the effect is small.
- **Status:** proposed

### H-11 · Concurrent tones closer than about one ERB are read less accurately

- **Statement:** When two data tones sound simultaneously, separations below about one ERB reduce comparison accuracy and increase rated roughness, and accuracy plateaus at separations of about one ERB or more.
- **Rationale:** SDR-28 extrapolates a perceptual-consonance result for pure-tone pairs to data sonification, with no validation (F-016). Sources: `plomp1965tonal`, `glasberg1990derivation`, `moore2012properties`.
- **Linked rules and findings:** SDR-28 (INFER), SDR-09 · F-016
- **Operationalization:**
  - *Design:* within-subjects.
  - *IV:* frequency separation (candidate levels 0.25, 0.5, 1, 2, 4 ERB); register (three positions within the default range); timbre = {sine, default harmonic}.
  - *Stimuli:* GEN-007.
  - *Task:* report which of two simultaneous series is higher, or which one moved.
  - *DV:* accuracy (primary); roughness rating; response time.
  - *Analysis:* logistic mixed model; segmented regression to estimate where accuracy plateaus.
- **Planned experiment:** EXP-010 (planned)
- **Target paper:** P03 extension
- **Testable from:** A08 (stimuli from multi-track specs); the corresponding lint lands in C05
- **Consequence for rules:** SDR-28 leaves `INFER` with a measured threshold, or its threshold changes.
- **Status:** proposed

### H-12 · A psychoacoustic comfort budget lowers annoyance without costing accuracy

- **Statement:** Over a sustained session, a sound design constrained by a comfort budget (caps on sustained sharpness and roughness) yields lower annoyance and fatigue ratings than an unconstrained design with the same information content, with non-inferior task accuracy.
- **Rationale:** SDR-39 is an inference. No validated comfort thresholds or longitudinal fatigue studies exist (F-016). Monitoring and long analysis sessions are core use cases. Sources: `fastl2007psychoacoustics`, `hildebrandt2016continuous`.
- **Linked rules and findings:** SDR-39 (INFER), SDR-35 · F-016
- **Operationalization:**
  - *Stage 1 (exploratory):* measure sharpness and roughness of candidate designs from renders and collect annoyance ratings to propose thresholds.
  - *Stage 2 (confirmatory):* within-subjects, two sessions on different days in counterbalanced order (candidate length 45–60 minutes); IV budget = {on, off}; time-on-task block.
  - *DV:* annoyance and fatigue ratings per block (primary); accuracy across blocks (non-inferiority), voluntary volume changes, NASA-TLX.
  - *Analysis:* mixed models with block as a covariate.
- **Planned experiment:** EXP-011 (planned)
- **Target paper:** P03 extension, or a standalone paper decided at triage
- **Testable from:** C05 (comfort lint); Monitoring mode from B06 provides the long-session use case
- **Consequence for rules:** SDR-39 gains measured thresholds and can leave `INFER`, or is retired.
- **Status:** proposed

---

## Candidate hypotheses (not yet numbered)

Triage may promote these to `H-13` onward.

| Source | Candidate |
|---|---|
| F-018 | Sequencing speech and sonification (A11Y-20) improves value comprehension over overlap at about 20 dB separation. |
| F-008 | MBS-style excitation (a data-sonogram wavefront from a selected node) conveys graph distance (SDR-33). |
| Research report (gaps) | Listener fatigue over weeks of use (longitudinal). |
