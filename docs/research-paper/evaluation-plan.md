# Evaluation plan

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [hypotheses](hypotheses.md) · [experiments](experiments/README.md) · [benchmarks log](benchmarks-log.md) · [datasets](datasets.md) · [publication plan](publication-plan.md) · [design rules](../research/design-rules.md) · [architecture overview §7](../architecture/01-overview.md) · [engines (technical)](../architecture/07-engines.md) · [accessibility conformance plan](../design/accessibility-conformance-plan.md) · [ADR-0013](../decisions/0013-determinism-and-provenance.md)

## Purpose and principles

This plan says how SONARITHM will show that its claims hold: that the engine is correct and fast, that its defaults work for real listeners, and that blind and low-vision (BLV) people can use it unaided. It follows five principles.

1. **Match the evaluation to the claim.** Toolkit papers choose evaluation strategies that fit their claims rather than defaulting to a usability study (`ledo2018evaluation`). Each claim in [contributions.md](contributions.md) names the evidence it needs.
2. **Use the field's standard tasks and metrics,** include BLV participants and a training arm (SDR-40), and set polarity for any new data concept by magnitude estimation (SDR-41).
3. **Publish stimuli, specs and analysis code** (SDR-42), so that others can rerun every study on the same harness.
4. **BLV people are paid co-designers and evaluators,** in rounds of about 10–15, on their own assistive technology (A11Y-34).
5. **Ethics approval comes before any study,** and consent materials work with screen readers ([notes/07 §7](../research/notes/07-engineering-publication.md)).

## Evaluation layers

| Layer | Question | Main evidence | Owner | Feeds |
|---|---|---|---|---|
| Technical | Is the engine correct, deterministic and fast enough? | Benchmarks, golden tests, null tests ([benchmarks log](benchmarks-log.md)) | Architecture lead | P01, P07, CL-02, CL-12 |
| Demonstration | Can the spec express and reproduce classic designs? | Replication gallery (below) | Research lead | P01, P02, CL-03, CL-04 |
| Perceptual studies | Do the defaults help listeners? | Controlled experiments EXP-001..EXP-011 (see the user-study portfolio) | Study leads | P03, P04, P06, P08 |
| Co-design and usage | Can BLV and sighted users do real tasks unaided? | Co-design rounds EXP-012, EXP-013; usage studies EXP-008, EXP-009 (see the co-design programme) | Research lead with BLV partners | P05, Gate B |
| Conformance | Does the app meet WCAG 2.2 AA via WCAG2ICT? | ACR, screen-reader matrix, Chartability audits | Accessibility lead ([conformance plan](../design/accessibility-conformance-plan.md)) | P05 (context only) |

Conformance is evidence of quality, not a research result. Papers cite it, but this plan does not duplicate it.

## Technical evaluation

### Budgets

Budgets are defined in [architecture overview §7](../architecture/01-overview.md) and recorded per release in the [benchmarks log](benchmarks-log.md).

| Attribute | Target (reference hardware) | First measured |
|---|---|---|
| Interactive response (key press → cue onset) | ≤ 50 ms p95 at 48 kHz, 256-frame buffer | A02, B03 |
| Audio robustness | 0 xruns in a 10-minute soak at 128-frame buffers | A05 |
| Offline render speed | ≥ 50× real time, 4-track 10k-event Score | A05 |
| Compile time | ≤ 200 ms for 100k rows × 4 tracks; ≤ 1 s for 1M rows | A06 |
| Determinism | Identical Score IR across OS/CPU; null-test residual ≤ −90 dBFS; LUFS Δ ≤ 0.1 | A05 |
| UI frame time | 60 fps playhead; ≤ 8 ms scripting per frame with a 1M-point series | B04 |
| Startup | Cold start to interactive ≤ 2 s | B01 |
| Footprint | Desktop installer ≤ 40 MB without ML models | C06 |

Phase exit criteria add more: a 1M-row CSV loads in ≤ 1.5 s (A04); a 1 kHz stream sonifies for an hour without xruns (B06); exact chart extraction has ≤ 0.5% relative error on a 50-chart corpus (C01); raster extraction has a median relative error ≤ 2% on clean charts (C02) ([roadmap](../planning/roadmap.md)).

### Determinism protocol

Determinism is tested as [ADR-0013](../decisions/0013-determinism-and-provenance.md) defines it. In summary:

1. The **Score IR** is the canonical artifact. Its hash must be identical across Windows, macOS and Linux, and across CPU architectures where CI provides them.
2. **Audio** is compared with tolerances, not byte equality, because floating-point and SIMD results differ across CPUs: null-test residual and LUFS difference against the budgets above ([notes/07 §6](../research/notes/07-engineering-publication.md)).
3. **Real-time vs offline:** the real-time path is captured at the render-graph output and compared with the offline render of the same Score (EXP-007, H-08).
4. Every render carries a manifest: engine version, spec hash, data fingerprint, seed, sample rate and block size ([architecture overview §6.3](../architecture/01-overview.md)).

### Method rules for every benchmark

- Record the machine, OS, driver or audio host, buffer size and power settings with each run.
- Report medians and the stated percentiles over repeated runs, with the run count.
- Commit the benchmark spec and script; link the benchmarks-log row to its manifest.
- A result that misses a budget, or moves by more than the run-to-run spread, is also logged as a finding (`F-###`).

## Replication gallery

The gallery re-creates classic designs from committed specs (roadmap A08). It is the "demonstration" strategy for P01 and a regression suite for the engine.

| Item | Source design | What must match | Phase |
|---|---|---|---|
| Context cues | Smith & Walker (2005): 20 tones of 0.5 s, 40 ms ramps, sine data tones 200–1900 Hz, triangle-wave dynamic reference, 10 ms clicks | Timing, pitch mapping, reference behaviour | A08 |
| Density and reversals | Nees & Walker (2008): 1, 2, 4 and 8 points/s with varying trend reversals | Rates and reversal structure | A08 |
| Transit signals | Tucker Brown et al. (2022): SNR-controlled transit-like light curves, SNR 3–100 | SNR definition and levels | A08 |
| Audification | Dombois (2001): resampling seismograms at 2,200×, no pitch shift | Speed-up factor, resulting band, duration T/k | A08 (CURVASONA voice in C03) |
| Erie examples | Erie gallery subset (Kim, Kim & Hullman 2024) | Encodings imported and rendered from the Erie spec | A06–A08 |
| Polarity presets | Walker (2007) concept table | Concept-table defaults and flip | A06 |
| Granular uncertainty | Williamson & Murray-Smith (2005): 80–300 ms grains, 100–1,000 active | Grain parameters | D01 |
| Crystallization | Hermann & Ritter (2002) | Model behaviour on a reference dataset | D02 |

**Acceptance criteria per item:** it renders from a committed spec and dataset with a manifest; its parameters match the published description, with every deviation documented next to the source; and it has a short written comparison citing the original. The gallery demonstrates expressiveness and reproducibility. It does not show perceptual equivalence to the original studies unless an experiment does.

## User-study portfolio

| Stage | Studies with participants | Technical evaluations |
|---|---|---|
| A · Foundation | EXP-014 (optional, only if ethics approval arrives in time) | EXP-007 numerical part; replication gallery |
| B · Accessible desktop | EXP-001 (after approval); EXP-012 co-design round 1 | AUDAXIS scripted task suite with NVDA, JAWS, VoiceOver, Orca |
| C · Bring any chart | EXP-008, EXP-009, EXP-013 co-design round 2, EXP-015, EXP-005 pilot; CURVASONA maths-teacher pilot (n ≥ 3, C03 exit) | Extraction corpora (C01, C02); detector precision/recall (C05) |
| D · Discover | EXP-002, EXP-003, EXP-004, EXP-005, EXP-006, EXP-010, EXP-011 (the D06 programme) | Extraction benchmark (D04) |

Details, hypotheses and target papers are in the [experiments registry](experiments/README.md).

**Timing caveat for P03.** EXP-001 cannot run before ethics approval. If approval comes after the spring 2027 deadlines, the 2027 version of P03 presents the harness with technical validation only, and the human replication moves to the P03 extension. This choice is recorded at Gate A.

## BLV co-design programme

### Model

- **Parallel co-design:** one-on-one sessions with each partner, iterating prototypes between sessions, as in Chart Reader (10 partners, three one-hour sessions each over five months) (`thompson2023chartreader`).
- **Standing partners:** at least one BLV core contributor or paid design partner from the start, following the inference in [notes/03 §6](../research/notes/03-accessibility-hci.md). Partners are paid (A11Y-34).
- **Rounds:** about 10–15 participants per evaluation round, matching the field median of 13 (`mack2021accessibility`).
- **Remote, on the participant's own assistive technology,** which preserves their configured settings (`bigham2007webinsitu`).
- **Bring your own data:** partners engaged less with data they did not care about, so participants may use their own data or topics (Chart Reader).
- **Mix of participants:** JAWS, NVDA, VoiceOver and Orca users; braille readers; people with hearing differences; congenitally and adventitiously blind people ([notes/03 §6](../research/notes/03-accessibility-hci.md)).
- **Access:** anticipate needs in advance and adjust during the study across communication, materials, space and time (`mack2022anticipate`).
- **Reporting:** demographics in aggregate only (`zong2022rich`).

### Schedule

| Round | Phase | Focus | Exit evidence |
|---|---|---|---|
| Standing sessions | From B03 | AUDAXIS navigation, earcons, verbosity; SONOTRACE monitoring; spec editor | Findings (type `accessibility` or `user-study`, level EV-D) |
| Round 1 (EXP-012) | B07 | Install, open data, explore, configure, export, unaided | Gate B: a BLV user completes the core tasks unaided in ≥ 80% of attempts |
| Round 2 (EXP-013) | C06 | PLOTSONA correction workflow (including the accessible point-entry alternative), CURVASONA, AXONOTE, descriptions | Findings feed P05 and v1.0 fixes |
| Uncertainty co-design | C02–D04 | Candidate audio renderings of estimated data (EXP-005 phase 1) | Shortlist for the H-06 task study |

### Compensation and budget

About $50–85 per hour (A11Y-34). For planning, a round of 15 participants with two 90-minute sessions each costs 45 hours × $50–85 = $2,250–3,825 [EST], before platform fees and the sighted comparison groups of controlled studies. The funding source is an open question (OQ-R2).

## Ethics and IRB timeline

Ethics approval must precede every study with participants. CHI, for example, requires that human-subjects research meets the authors' local ethics-review requirements, and ACM's 2021 policy on research involving human participants applies ([notes/07 §7](../research/notes/07-engineering-publication.md)). Starting applications by December 2026 keeps spring 2027 deadlines realistic ([research report](../research/research-report.md#publishable-research-sits-in-the-fields-admitted-gaps)).

| When | Step |
|---|---|
| By end of Nov 2026 | Decide which ethics board reviews SONARITHM studies (open question OQ-R1). Draft an umbrella protocol covering minimal-risk listening tasks, magnitude estimation, remote sessions and co-design. Prepare accessible consent templates. |
| By 31 Dec 2026 | File the umbrella application (roadmap A08 research hook). |
| Q1 2027 [EST] | Approval; review time depends on the board. EXP-014 and EXP-001 wait for it. |
| Per experiment | File an amendment or a new application with the protocol from the [template](experiments/EXP-000-template.md). Record the reference in the protocol. |
| Every year | Renew approval before expiry; record continuing-review dates in each protocol. |

**Open question.** The project lead's institutional affiliation is not recorded in the project documents. Which board can review SONARITHM's studies therefore depends on a decision (OQ-R1): the lead's own institution if there is one, an academic collaborator acting as principal investigator under their board, or an independent ethics review service. This must be settled in November 2026 to file in December.

## Metrics catalogue

| Metric | Definition | Used by | Source |
|---|---|---|---|
| RMS error | Root mean square of response minus true value in a point-estimation task | H-01, H-02, H-03, H-10 | `smith2005effects`; `nees2008density` |
| Trend-identification accuracy | Proportion of correct trend judgements | H-02, H-03, H-06, H-07 | `nees2008density` |
| Detection rate and d′ | Hits and false alarms for signals, anomalies or estimated regions, by SNR where relevant | H-06, H-09 | `tuckerbrown2022evaluating` |
| Discrimination sensitivity | Choice of the strongest of three correlations, or a threshold | Benchmark task (GEN-004) | `elmquist2024parallel` |
| Response time; replays | Time to respond; number of plays | All perceptual studies | `nees2008density` |
| Magnitude-estimation slope | Log–log slope: sign = polarity, magnitude = scaling | H-05 | `walker2002magnitude` |
| Workload | NASA-TLX | H-06, H-09, H-12 | `hart1988nasatlx` |
| Usability | SUS | EXP-012, EXP-013, H-10 | `brooke1996sus` |
| Pleasantness, clarity, confidence | Rating scales | Co-design, H-12 | `sharif2022userfriendly` |
| Perceived difficulty | Rating | H-03 | `fitzpatrick2023frequency` |
| Enjoyability, engagement | Rating; user-engagement scale | P09, Presentation mode | `elmquist2024parallel`; `middleton2023datatomusic` |
| Insight count | Insights reported in open exploration | Co-design | `siu2022audio` |
| Unaided task completion | Share of core tasks completed without help | Gate B | [Roadmap](../planning/roadmap.md) |
| Psychoacoustic descriptors | Sharpness, roughness; ERB separation | H-11, H-12 | `fastl2007psychoacoustics`; `glasberg1990derivation` |
| Extraction accuracy | RMS-F1 for table structure; resampled curve error for time series; relative error | C01, C02, D04 | `deplot2023`; [notes/06 §3](../research/notes/06-chart-extraction-ai.md) |
| Engine metrics | Latency p50/p95, xruns, real-time factor, null-test residual, LUFS Δ, compile time, startup, installer size, UI frame time | H-08, P07 | [Architecture overview §7](../architecture/01-overview.md) |

No sonification-specific standardized questionnaire comparable to SUS was found ([notes/02 §5](../research/notes/02-theory-perception.md)). Developing one is out of scope for now.

## Study infrastructure

### ECHOMETRIC harness

The ECHOMETRIC evaluation harness supplies stimulus generators, standard tasks and logging ([engines catalogue](../product/engines.md); technical contract in [architecture/07-engines.md](../architecture/07-engines.md)). Version 0 ships in A08 with the generators GEN-001..GEN-004 ([datasets](datasets.md)) and the replication gallery.

### `sonarithm study run`

The experiment-runner CLI from A08 reads a study definition (conditions, blocks, counterbalancing seed, stimulus manifests), presents trials, and writes structured logs. Requirements:

- Keyboard-only operation and screen-reader compatibility from the first version; it is a research instrument for BLV participants, not only for sighted ones.
- Deterministic trial order from a recorded seed.
- Stimuli rendered from committed specs; the manifest of every stimulus is referenced in the log.
- A study mode that is separate from product behaviour. Research logging is study-specific, covered by ethics approval and consent, and never part of general product analytics ([notes/07 §8](../research/notes/07-engineering-publication.md); architecture principle P8).

### Logging schema (proposed v0)

One NDJSON record per event, plus one session manifest. This is a proposal for A08; the normative schema will live with the harness in [architecture/07-engines.md](../architecture/07-engines.md).

| Field | Content |
|---|---|
| `exp_id`, `protocol_version` | Experiment and protocol version |
| `session_id`, `participant_code` | Random codes; never names or contact details |
| `block`, `trial`, `condition` | Position and factor levels |
| `stimulus_id`, `spec_hash`, `data_fingerprint`, `seed`, `manifest_ref` | Exact stimulus provenance |
| `engine_version` | SONARITHM version and commit |
| `event` | `trial_start`, `play`, `replay`, `pause`, `response`, `rating`, `trial_end`, `note` |
| `t_mono_ms` | Monotonic time since session start |
| `response`, `correct`, `true_value` | Response data and scoring inputs |
| `playback_level`, `output_device`, `headphone_model` | Audio context (calibrated level in the lab; logged setting remotely) |
| `assistive_tech` | Screen reader and version, braille display, magnifier, as reported |

### Calibration

- **Lab:** playback level calibrated at the ear with a stated method and a maximum cap. Smith & Walker used about 60 dB SPL over headphones.
- **Remote:** absolute level cannot be guaranteed. Use a calibration-tone procedure, let participants set a comfortable level, log the setting and headphone model, and treat device variation as a threat to validity (see below).

## Threats to validity

| Threat | Example | Mitigation |
|---|---|---|
| Order, learning and fatigue | Performance improves across blocks | Counterbalancing; practice trials; breaks; block as a model term |
| Designer bias | Authors evaluating their own tool | Preregistration; standard tasks; replication of published designs; external replication via open materials |
| Population | Most prior samples are sighted students; polarity data are mostly from US undergraduates | Mixed BLV and sighted samples; recruit beyond universities; report hearing, AT and musical background |
| Small BLV samples | Rounds of 10–15 limit power | Simulation-based power analysis; estimation with confidence intervals; pooling across rounds where preregistered |
| Remote equipment | Headphones, levels and environments vary | Calibration procedure; logged devices; device as a covariate; lab subsample where possible |
| Screen-reader interaction | NVDA ducking and sound split change what participants hear | Record AT settings; document recommended settings; test both configurations |
| Construct validity | RMS error depends on the mapped range and data units | Normalize errors; report raw and normalized values; use published stimuli |
| Ecological validity | Lab pure tones may mislead for real environments | Bring-your-own-data sessions; field-like monitoring tasks (EXP-008) |
| LLM drift | Model updates change suggestions (H-07) | Pin model IDs and dates; store prompts and outputs; include local models |
| Multiple comparisons | Factorial designs yield many tests | Preregistered primary contrasts; correction for secondary tests |

## Data management plan summary

Each protocol's data-management section is binding for that study. Across the project:

- **Pseudonymization:** random participant codes; the key file is stored separately and encrypted; no names or contact details in logs.
- **Storage:** raw data in access-controlled, encrypted storage outside the public repository, retained for the period the approving board requires.
- **Sharing:** with consent, de-identified trial-level data and codebooks under CC BY 4.0 (OSF or Zenodo); analysis code under MIT OR Apache-2.0; stimuli and generators as listed in [datasets.md](datasets.md). Recordings are never published. Demographics are reported in aggregate.
- **Provenance:** every result links to the SONARITHM release DOI, spec hashes and render manifests.
- **Principles:** FAIR principles for research software (`barker2022introducing`); every paper ships an artifact package ([publication plan](publication-plan.md)).
- **Jurisdiction:** data-protection law depends on where participants and the approving board are; the board's requirements govern.

## Open questions

| ID | Question | Needed by |
|---|---|---|
| OQ-R1 | Which ethics board reviews SONARITHM studies? The project lead's institutional affiliation is unknown. | End of Nov 2026 |
| OQ-R2 | How are participant compensation and article processing charges funded? | Before EXP-001 and EXP-012 |
| OQ-R3 | Who provides independent statistical review of preregistrations? | Before the first preregistration |
| OQ-R4 | Can recruitment calls be posted through NFB, Program-L and community partners, and on what terms? | Before EXP-001 |
| OQ-R5 | What is the reference hardware for benchmarks? (Shared with the architecture lead.) | A02 |
| OQ-R6 | Does the ICAD 2027 call expect user evaluation for system papers? | When the call is published |
