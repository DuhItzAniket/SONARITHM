# Experiments

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [protocol template](EXP-000-template.md) · [hypotheses](../hypotheses.md) · [evaluation plan](../evaluation-plan.md) · [datasets and generators](../datasets.md) · [publication plan](../publication-plan.md)

An experiment (`EXP-###`) is any study with participants, or any technical evaluation whose result feeds a hypothesis or a paper. Every experiment starts from [EXP-000-template.md](EXP-000-template.md). `EXP-000` is the template and is never run.

## Files

- **Protocol:** `experiments/EXP-###-<slug>.md`, copied from the template. It is the single source of truth for the study.
- **Materials (optional folder):** `experiments/EXP-###-<slug>/` for consent texts, questionnaires, stimulus manifests and pointers to analysis code.
- **Not in the repository:** raw participant data, contact details, consent records and recordings. These are stored as the protocol's data-management section specifies.

## Lifecycle

| Status | Gate to reach it |
|---|---|
| `planned` | Listed below with its hypotheses and target paper. |
| `drafted` | Protocol complete from the template; reviewed by the research lead and at least one BLV co-design partner where BLV participants are involved. |
| `approved` | Ethics approval recorded (reference and date). Technical evaluations without participants skip this gate. |
| `preregistered` | OSF preregistration posted before data collection; link in the protocol. |
| `piloted` | Pilot run; the power analysis and margins are updated; changes are logged as deviations. |
| `running` | Data collection under way; deviations logged as they occur. |
| `analysed` | Preregistered analysis complete; exploratory analyses labelled. |
| `reported` | Results summary filled, findings logged (`F-###`), hypotheses updated, artifacts archived. |
| `withdrawn` | Stopped, with a reason. The ID is not reused. |

## Planned experiments

Phase columns refer to the [roadmap](../../planning/roadmap.md). "Run" is the earliest window in which the study can run once the software and ethics approval exist; it is re-planned at each gate.

| ID | Title | Hypotheses | Kind | Participants | Prepare / run | Paper | Status |
|---|---|---|---|---|---|---|---|
| EXP-001 | Context-cue replication (Smith & Walker 2005) | H-01 | Controlled, within-subjects | BLV + sighted | A08 / Stage B, after approval | P03 | planned |
| EXP-002 | Pacing × range × context factorial | H-02, H-03; extends H-01 | Controlled, within-subjects factorial | BLV + sighted | A08 / D06 | P03 ext. | planned |
| EXP-003 | Concurrent-stream limit | H-04 | Controlled, within-subjects | BLV + sighted | D01 / D01–D06 | P03 ext. | planned |
| EXP-004 | Graph-concept magnitude estimation | H-05 | Magnitude estimation | BLV + sighted | A08 / D06 | P04 | planned |
| EXP-005 | Accessible uncertainty in audio | H-06 | Co-design, then task study | BLV (primary) + sighted | C02 / D04–D06 | P08 | planned |
| EXP-006 | LLM vs rule-engine vs empirical mappings | H-07 | Part A without participants; Part B task study | BLV + sighted (Part B) | D04 / D06 | P06 | planned |
| EXP-007 | Real-time vs offline render equivalence | H-08 | Technical benchmark; ABX listening test | Small listener panel (ABX only) | A02 / A05 | P07 | planned |
| EXP-008 | Dual-task monitoring (Hildebrandt et al. 2016 replication) | H-09 | Controlled, within-subjects | Sighted + BLV variant | B06 / Stage C | P05 | planned |
| EXP-009 | Tutorial effect | H-10 | Pre/post with control group | Novice BLV + sighted | B07 / Stage C | P05 | planned |
| EXP-010 | Critical-band spacing of concurrent tones | H-11 | Controlled, within-subjects | BLV + sighted | A08 / D06 | P03 ext. | planned |
| EXP-011 | Comfort budget over long sessions | H-12 | Exploratory, then confirmatory | BLV + sighted | C05 / D06 | P03 ext. | planned |
| EXP-012 | Co-design round 1 | — (formative; A11Y-34) | Parallel co-design | 10–15 BLV | B07 | P05; evidence for P02 | planned |
| EXP-013 | Co-design round 2 | — (formative; A11Y-34) | Parallel co-design | 10–15 BLV | C06 | P05 | planned |
| EXP-014 | Targeted usage walkthrough for the toolkit paper | — (Ledo usage strategy) | Walkthrough of spec authoring | About 5 researchers [EST] | A08 | P01 | planned; runs only if approval arrives in time |
| EXP-015 | Practitioner study of AXONOTE as an instrument | — | Demo and practitioner study | Musicians and sound designers | C04 | P09 | planned |

EXP-002 absorbs the full factorial version of H-01 so that pacing, range and context are tested together, as the research report proposes. EXP-001 is the smaller, earlier replication.

## Rules for every experiment

1. **Ethics before data.** No participant data is collected before approval (see the [evaluation plan](../evaluation-plan.md#ethics-and-irb-timeline)).
2. **Preregister confirmatory analyses** on OSF. Anything not preregistered is reported as exploratory.
3. **Reproducible stimuli.** Every stimulus comes from a registered generator (`GEN-###`) or a committed spec, with a seed and a render manifest.
4. **Accessible materials** and paid BLV participation, as A11Y-34 requires.
5. **Log as you go.** Deviations go into the protocol during the study. Results become findings (`F-###`) at `reported`.
