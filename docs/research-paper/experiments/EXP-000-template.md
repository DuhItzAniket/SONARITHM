# EXP-000 · Protocol template

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [experiments registry](README.md) · [hypotheses](../hypotheses.md) · [evaluation plan](../evaluation-plan.md) · [datasets and generators](../datasets.md) · [design rules](../../research/design-rules.md) (SDR-40..SDR-42, A11Y-34)

Copy this file to `EXP-###-<slug>.md`, replace every `<…>` placeholder, and delete guidance lines in italics. A section that does not apply says "Not applicable" and why. Keep the protocol current: it is the record reviewers and readers will check.

---

## 0. Identification

| Field | Value |
|---|---|
| ID and title | `EXP-###` · <title> |
| Status | planned / drafted / approved / preregistered / piloted / running / analysed / reported / withdrawn |
| Study lead | <name or handle> |
| Protocol version | <v1, v2, …> (date of each version in the deviations log) |
| Hypotheses | <H-## list> |
| Target paper | <P##> |
| Ethics reference | <board, reference number, approval date, expiry> |
| Preregistration | <OSF URL, date posted> |
| SONARITHM release | <tag, commit, Zenodo DOI> |

## 1. Plain-language summary

*Three to five sentences a participant can understand. Reuse it in the participant information sheet.*

<summary>

## 2. Hypotheses and predictions

| ID | Confirmatory or exploratory | Prediction (direction and size) | Primary measure |
|---|---|---|---|
| <H-##> | confirmatory | <e.g., RMS error lower with dynamic reference than with no context> | <measure> |

## 3. Design

- **Type:** <within-subjects / between-subjects / mixed>.
- **Factors and levels:** <factor: levels>. *Levels are fixed here, after the pilot.*
- **Counterbalancing:** <Latin square / full permutation / randomized order>, generated with seed `<seed>`.
- **Blocks and trials:** <number of blocks, trials per cell, total trials>.
- **Session length and breaks:** <minutes; planned breaks; split across sessions if longer than about 90 minutes>.
- **Baseline:** <pre-test used as covariate, if any>.

## 4. Participants

- **Populations:** BLV and sighted, unless the hypothesis concerns only one group. *State why if a group is excluded.*
- **Inclusion criteria:** <age 18 or over; self-reported hearing status; for BLV, regular screen-reader or magnifier use; language>.
- **Exclusion criteria:** <e.g., participation in the pilot>.
- **Sample size:** <n per group>.
  - *Power analysis:* simulation-based, for the planned mixed-effects model, using the smallest effect size of interest and variance from the pilot or from a published source. Record the inputs, the target power and the script.
  - *BLV samples:* rounds of about 10–15 match field norms (median 13, `mack2021accessibility`). If that is below the power target, say how BLV results will be treated (estimation with confidence intervals, pooling across rounds, or exploratory status).
- **Reported demographics:** aggregate only (age range, vision status, screen reader, braille use, hearing profile, musical training), following Zong et al. 2022.

## 5. Recruitment

- **Channels:** <e.g., National Federation of the Blind (NFB), Program-L, blind-community collaborators, public calls, university pools>. MAIDR recruited through NFB and Program-L; Zong et al. through community collaborators and a public call ([notes/03 §6](../../research/notes/03-accessibility-hci.md)).
- **Screening:** <questionnaire; accessible format>.
- **Scheduling:** an accessible tool or plain email; flexible times.
- **Approved texts:** <link to recruitment text approved by the ethics board>.

## 6. Compensation

- **BLV participants:** about $50–85 per hour (A11Y-34), for the full scheduled time including setup.
- **Sighted participants:** a fair rate, budgeted separately ([notes/03 §6](../../research/notes/03-accessibility-hci.md)).
- **Payment method:** <accessible method>. Withdrawal does not forfeit payment for time spent.
- **Budget:** <n × hours × rate = total; funding source>.

## 7. Apparatus

| Item | Value |
|---|---|
| SONARITHM version | <tag, commit, Zenodo DOI> |
| Study runner | `sonarithm study run` <version> |
| Specs | <paths and SHA-256 spec hashes> |
| Data fingerprints | <stable data hashes> |
| Render manifests | <paths; engine version, spec hash, data fingerprint, seed, sample rate, block size> |
| Sample rate / block size | <e.g., 48 kHz / 256> |
| Playback level | Lab: calibrated to <dB SPL> at the ear, method <coupler / meter>, maximum level cap <dB SPL>. Remote: calibration procedure <tone-matching steps>; participant sets a comfortable level; level is logged. *Smith & Walker used about 60 dB SPL over headphones.* |
| Headphones | <model; closed-back preferred>. Remote participants report their model. |
| Assistive technology | <screen reader and version; braille display; magnifier> |
| Operating system and hardware | <recorded per session> |
| Network and privacy | Study mode only; no product telemetry; data leaves the machine only as the data-management section states. |

## 8. Stimuli

- **Generator:** `<GEN-###>` ([datasets.md](../datasets.md)), version <x>.
- **Parameters:** <list>. *Where the study replicates published work, list the published parameters next to ours and justify every difference.*
- **Seeds:** <seed list or seed derivation rule>.
- **Items:** <number of items per cell; total>.
- **Validation:** <checks that rendered stimuli match the intended parameters, e.g., Score IR snapshot, pitch range, rate>.
- **Manifest and licence:** <path to stimulus manifest>; stimuli released under CC BY 4.0.

## 9. Procedure

| Step | Content | Minutes |
|---|---|---|
| 1 | Setup check: audio, headphones, assistive technology, level calibration | <n> |
| 2 | Consent (accessible format; verbal or audio option) | <n> |
| 3 | Background questionnaire (vision, hearing, AT, musical training, statistics background) | <n> |
| 4 | Standardized training block (fixed content and length; same for all conditions) | <n> |
| 5 | Practice trials with feedback | <n> |
| 6 | Main blocks, with breaks | <n> |
| 7 | Post-task questionnaires (NASA-TLX, SUS, ratings) | <n> |
| 8 | Debrief and payment | <n> |

## 10. Measures

*Keep the measures that apply; define each one precisely.*

| Measure | Definition | Primary or secondary |
|---|---|---|
| RMS error | Root mean square of (response − true value) per participant × condition, in data units | <…> |
| Accuracy | Proportion correct (trend identification, extremum, outlier, signal count) | <…> |
| Sensitivity | d′ from hits and false alarms, where detection is the task | <…> |
| Response time | Stimulus offset (or onset; state which) to response, from the runner's monotonic clock | <…> |
| Replays | Number of times the stimulus was played | <…> |
| Magnitude-estimation slope | Per-participant log–log slope: sign = polarity, magnitude = scaling | <…> |
| NASA-TLX | Workload (`hart1988nasatlx`); raw or weighted, state which | <…> |
| SUS | Usability (`brooke1996sus`) | <…> |
| Pleasantness, clarity, confidence | Rating scales as in Sharif et al. 2022 (`sharif2022userfriendly`) | <…> |

## 11. Analysis plan

- **Primary model:** <mixed-effects model; formula; family (Gaussian, binomial, …); random effects for participant and stimulus>.
- **Contrasts:** <planned contrasts>.
- **Margins:** <non-inferiority or equivalence margins and their justification>, if used.
- **Multiple comparisons:** <correction method>.
- **Exclusions:** <preregistered exclusion criteria for participants and trials>.
- **Missing data:** <handling>.
- **Exploratory analyses:** <list; reported separately and labelled exploratory>.
- **Software:** <language, packages, versions>; analysis code in the repository and archived with the paper.
- **Preregistration:** posted on OSF before data collection: <URL>.

## 12. Accessibility of study materials

- [ ] Consent form, information sheet and questionnaires available as screen-reader-accessible HTML or tagged PDF and as plain text; verbal or audio consent offered.
- [ ] Materials pre-tested with at least two screen readers and at 200% zoom.
- [ ] Study runner fully keyboard-operable and tested with NVDA, JAWS, VoiceOver and Orca where participants use them.
- [ ] No instruction relies on sight; no response requires pointing.
- [ ] Flexible scheduling, breaks on request, extra time without penalty.
- [ ] Access needs asked for in advance and adjusted during the study (`mack2022anticipate`); adjustments recorded in the deviations log.

## 13. Ethics

- **Board and reference:** <board; reference number; approval and expiry dates>.
- **Risk level:** <e.g., minimal risk>.
- **Hearing safety:** calibrated playback levels with a stated maximum; participants can stop audio at any time.
- **Withdrawal:** at any time, without giving a reason; data handling on withdrawal: <rule>.
- **Amendments:** <list with dates and approval>.

## 14. Data management

- **Collected:** <trial logs, questionnaires, notes; whether audio or video is recorded and why>.
- **Pseudonymization:** random participant codes; the key file is stored separately, encrypted, and destroyed at the end of the retention period.
- **Storage and access:** <location, encryption, who has access>.
- **Retention:** <period required by the approving board>.
- **Sharing:** if consent allows, de-identified trial-level data and a codebook under CC BY 4.0; analysis code under MIT OR Apache-2.0. Free-text answers are checked for identifying details before release. Recordings are never published.

## 15. Deviations log

| Date | Deviation | Reason | Impact on analysis | Approved by |
|---|---|---|---|---|
| | | | | |

## 16. Results summary

*Fill in at `analysed`, finalize at `reported`.*

- **Sample achieved:** <n per group; exclusions>.
- **Primary outcomes:** <effect sizes with 95% confidence intervals>.
- **Hypothesis verdicts:** <H-##: supported / not supported / inconclusive>.
- **Exploratory observations:** <labelled>.
- **Artifacts:** <release DOI; OSF data and code links>.

## 17. Links to findings and rules

- **Findings logged:** <F-### list>.
- **Rule changes proposed:** <SDR/A11Y IDs and PR links>.
- **Paper sections:** <P## and section>.

## 18. Ready-to-run checklist

- [ ] Ethics approval recorded (section 0).
- [ ] Preregistration posted (section 11).
- [ ] Pilot complete; power analysis and margins updated.
- [ ] Stimuli rendered from committed specs with manifests; spec hashes listed.
- [ ] Accessibility checklist complete (section 12).
- [ ] Data-management arrangements in place (section 14).
- [ ] Compensation budget approved (section 6).
