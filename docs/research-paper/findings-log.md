# Findings log

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [track README](README.md) · [hypotheses](hypotheses.md) · [design rules](../research/design-rules.md) · [research report](../research/research-report.md) · [references.bib](../research/references.bib)

This log is **append-only**. Add new entries at the end of the right section with the next free ID. Never delete or renumber an entry, and never rewrite its statement. The only permitted edits to an existing entry are its `Status` and `Papers` fields, and a "Superseded by" pointer. To correct a finding, log a new one that supersedes it.

## How to write an entry

Copy this template. Keep the statement to one or two sentences that someone could cite. Put numbers, sample sizes and caveats in the statement or the evidence, not in prose elsewhere.

```markdown
### F-### · <short title in sentence case>

- **Date:** YYYY-MM-DD · **Phase:** <roadmap phase, e.g. A05> · **Type:** literature | engineering | perceptual | user-study | benchmark | accessibility · **Status:** new
- **Statement:** <one or two citable sentences, with numbers and sample sizes>
- **Evidence:** <source with author, year, DOI/URL; or spec hash, render manifest, benchmark row, EXP-ID, PR, commit>
- **Evidence level / confidence:** <EV-A | EV-B | EV-C | EV-D | EV-L | EV-Lp> / <Strong | Moderate | Weak | Expert-opinion>
- **Affects:** <SDR-## / A11Y-## rules, ADR-####, H-## hypotheses, CL-## claims>
- **Papers:** <P## list, or "none yet">
- **Superseded by:** <F-###, only if status is superseded>
```

| Field | Rule |
|---|---|
| Type | `literature` (a published source), `engineering` (an implementation fact or decision), `perceptual` (a listening observation or result), `user-study` (a result from an `EXP-###`), `benchmark` (a measured metric; also add a [benchmarks-log](benchmarks-log.md) row), `accessibility` (screen reader, braille or conformance behaviour). |
| Status | `new` (not yet triaged) → `triaged` (outcome recorded) → `used-in-paper` (cited in a submitted paper). `superseded` when a later finding replaces it. |
| Evidence levels | Defined in the [track README](README.md#evidence-levels). |

---

## Background (kickoff research)

These entries record the kickoff research of 2026-10-18 as citable findings. All are `literature`. They were triaged into the [design rules](../research/design-rules.md) and the [hypotheses](hypotheses.md) at baseline. Re-verify any landscape or "no study exists" statement before quoting it in a paper, because such statements age.

### F-001 · No maintained native cross-platform sonification engine exists

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** As of October 2026, no actively maintained, native, cross-platform sonification application or engine exists. The maintained tools are web libraries (MAIDR, Chart2Music, Highcharts, Erie, Umwelt) and Python notebooks (STRAUSS). Desktop tools last moved in December 2014 (Sonification Sandbox), 2018 (xSonify) and January 2024 (SonoUno desktop), and Rust projects have 0–4 GitHub stars. No surveyed tool combines screen-reader-grade navigation, a declarative mapping specification, live streaming and reproducible offline audio/MIDI export.
- **Evidence:** [notes/01 §1 and §3](../research/notes/01-tools-landscape.md); [research report, opening sections](../research/research-report.md). Repository status checked against the GitHub API on 2026-10-18.
- **Evidence level / confidence:** EV-L (landscape survey) / Moderate (a snapshot that ages)
- **Affects:** CL-01; positioning in all system papers
- **Papers:** P01, P02, P05

### F-002 · Only 6.1% of published mappings were evaluated

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** Of 495 mapping occurrences in 179 publications from 60 projects, only 30 (6.1%) carried any assessment, and only 7 of 60 projects assessed at least one mapping.
- **Evidence:** Dubus & Bresin (2013), *PLoS ONE* 8(12):e82491, [doi:10.1371/journal.pone.0082491](https://doi.org/10.1371/journal.pone.0082491) (`dubus2013systematic`)
- **Evidence level / confidence:** EV-L / Strong
- **Affects:** SDR-40, SDR-42; CL-05; motivation for H-01..H-12
- **Papers:** P01, P03, P05

### F-003 · A dynamic y-axis reference tone cut point-estimation error from 18.09 to 12.14

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** In a study of 160 listeners estimating dollar values, a dynamic y-axis reference tone (maximum while values rose, minimum while they fell, in a distinct timbre) cut RMS error from 18.09 (no y-context) to 12.14. A static reference at the opening value (RMS 17.46) did not help significantly. Ten-millisecond x-axis clicks aided time calibration (RMS ≈ 14.63 with clicks) but did not help with continuous stimuli. Context and training effects were "comparable, but not additive".
- **Evidence:** Smith & Walker (2005), *Applied Cognitive Psychology* 19(8):1065–1087, [doi:10.1002/acp.1146](https://doi.org/10.1002/acp.1146) (`smith2005effects`); [notes/02 §2g](../research/notes/02-theory-perception.md)
- **Evidence level / confidence:** EV-L / Moderate (one study, sighted undergraduates)
- **Affects:** SDR-16, SDR-17, SDR-37; H-01, H-10; CL-04
- **Papers:** P01 (replication gallery), P03

### F-004 · A 400–1400 Hz range performed as well as 65–1480 Hz

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** On line-chart tasks, a narrow 400–1400 Hz pitch range performed as well as a wide 65–1480 Hz range (three groups of 16, n = 48). Basic amplitude compensation on the wide range had no significant effect on performance or perceived difficulty.
- **Evidence:** Fitzpatrick & Neff (2023), *ICAD 2023*, [doi:10.21785/icad2023.1904](https://doi.org/10.21785/icad2023.1904) (`fitzpatrick2023frequency`)
- **Evidence level / confidence:** EV-L / Moderate (one study, n = 48)
- **Affects:** SDR-08, SDR-24; H-03
- **Papers:** P03

### F-005 · Screen-reader users were 61% less accurate and spent 211% more time on online charts

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** In a controlled study of 36 screen-reader and 36 non-screen-reader users, screen-reader users extracted information 61.48% less accurately (34% vs 87% correct) and spent 210.96% more time. A third of 27 sampled visualizations could not be found by screen readers at all. Accuracy depended on the library: Google Charts 73%, D3 17%, ChartJS 11%.
- **Evidence:** Sharif et al. (2021), *ASSETS '21*, [doi:10.1145/3441852.3471202](https://doi.org/10.1145/3441852.3471202) (`sharif2021understanding`); [notes/03 §1](../research/notes/03-accessibility-hci.md)
- **Evidence level / confidence:** EV-L / Strong
- **Affects:** A11Y-01, A11Y-03, A11Y-09; CL-07
- **Papers:** P05, P08

### F-006 · ML line-chart extraction still scores about 18–39 RMS-F1 for most models

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** On the 2026 WB-ChartExtract benchmark (a preprint), single-pass line-chart RMS-F1 was 87.68 for Gemini 2.5 Pro, 38.53 for GPT-5.1, 28.10 for Llama 4 Scout and 18.32 for TinyChart, which scores 95.20 on ChartQA tables. On real epidemic curves (EpiCurveBench, a preprint), the best model reached 52.3% ECS against 90.9% agreement between careful human annotators.
- **Evidence:** [arXiv 2605.27298](https://arxiv.org/abs/2605.27298) (`berkane2026wbchartextract`); [arXiv 2605.27195](https://arxiv.org/abs/2605.27195) (`epicurvebench2026`); [notes/06 §3](../research/notes/06-chart-extraction-ai.md)
- **Evidence level / confidence:** EV-Lp / Moderate (preprints; the Gemini lead comes from a single source)
- **Affects:** Architecture principle P6 (exact before estimated); PLOTSONA provenance classes; H-06; CL-08
- **Papers:** P05, P08

### F-007 · Erie is the only declarative sonification grammar, and it lacks interaction, streaming and export

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** Erie is the only declarative sonification grammar found. It compiles to Web Audio and Web Speech, lists interactive sonification, streaming and other audio environments as future work, supports "a minimum set of data transform types", and documents no audio export.
- **Evidence:** Kim, Kim & Hullman (2024), *CHI '24*, [doi:10.1145/3613904.3642442](https://doi.org/10.1145/3613904.3642442) (`kim2024erie`); [notes/01 §3](../research/notes/01-tools-landscape.md); [notes/02 §6](../research/notes/02-theory-perception.md)
- **Evidence level / confidence:** EV-L / Strong
- **Affects:** SDR-04, SDR-05; CL-03
- **Papers:** P01, P02, P05

### F-008 · No polarity data exist for graph concepts, and no BLV study of network sonification was found

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** No published magnitude-estimation (polarity or scaling) data were found for graph concepts such as node degree, centrality, edge weight or path length, and no blind-user evaluation of node-link or network sonification was found. Data Navigator's node-edge navigation model is the closest validated analogue.
- **Evidence:** [notes/02 §7, Inferences](../research/notes/02-theory-perception.md); [notes/03 §2, Gaps](../research/notes/03-accessibility-hci.md); [research report](../research/research-report.md), section "Blind users want an overview, then control, then the data itself"; `elavsky2023datanavigator`
- **Evidence level / confidence:** EV-L (absence of evidence in a structured search) / Moderate
- **Affects:** SDR-33, SDR-41; H-05; CL-09, CL-16, CL-17
- **Papers:** P04

### F-009 · No peer-reviewed evaluation of LLM-suggested sonification mappings exists

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** No peer-reviewed controlled evaluation of LLM-generated sonification mappings was found for 2023–2026. AI's "application in sonification remains limited".
- **Evidence:** Hommel & Hermann (2025), *ICAD 2025*, [doi:10.21785/icad2025.037](https://doi.org/10.21785/icad2025.037) (`hommel2025ai`); [notes/02 §6, Gaps](../research/notes/02-theory-perception.md); [notes/06 §7, Gaps](../research/notes/06-chart-extraction-ai.md)
- **Evidence level / confidence:** EV-L (absence of evidence) / Moderate
- **Affects:** H-07; CL-10
- **Papers:** P06

### F-010 · Detection by ear depends on signal-to-noise ratio, and expert visual skill did not transfer

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** With 192 participants counting transit-like signals in light curves, experts and non-experts reached 85–100% correct with sonification at SNR 30–100. At SNR 7–10, experts did about 2–3× better with visuals but no better than non-experts with sonification. At SNR 3–5, performance with sonification was at chance.
- **Evidence:** Tucker Brown, Harrison, Zanella & Trayford (2022), *MNRAS* 516(4):5674–5683, [doi:10.1093/mnras/stac2590](https://doi.org/10.1093/mnras/stac2590) (`tuckerbrown2022evaluating`)
- **Evidence level / confidence:** EV-L / Moderate
- **Affects:** CL-16 (tagline discipline); SDR-37; GEN-003
- **Papers:** P03, P05

### F-011 · Polarity is concept- and population-specific, and designer intuition misleads

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** Pitch polarities differ by data concept (positive/none/negative counts: velocity 18/1/0, mass 2/0/17, size 9/0/10, number of dollars 9/2/8; n = 19 per block). Sighted listeners used an abstract "higher means more" model for dollars, while visually impaired listeners leaned negative. In a process-control task the "Bad" mapping ensemble produced the fastest performance and the "Intuitive" one was among the poorest.
- **Evidence:** Walker (2007), [doi:10.1002/acp.1291](https://doi.org/10.1002/acp.1291) (`walker2007consistency`); Walker & Mauney (2010), [doi:10.1145/1714458.1714459](https://doi.org/10.1145/1714458.1714459) (`walker2010universal`); Walker & Kramer (2005), [doi:10.1145/1101530.1101534](https://doi.org/10.1145/1101530.1101534) (`walker2005mappings`)
- **Evidence level / confidence:** EV-L / Moderate (mostly US undergraduates; small BLV samples)
- **Affects:** SDR-12..SDR-15, SDR-41; H-05, H-07
- **Papers:** P04, P06

### F-012 · Data density up to 8 points per second has only a small effect; nothing is known above that

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** Density from 1 to 8 points per second had "a small, but significant, effect" on point-estimation error, while the number of trend reversals had large effects. No study measured discrete-event limits for auditory graphs above 8 points per second. Metrical grouping has a floor of about 100 ms between onsets, so the ~10 events/s switch in SDR-20 is an untested inference.
- **Evidence:** Nees & Walker (2008), [doi:10.1145/1402236.1402237](https://doi.org/10.1145/1402236.1402237) (`nees2008density`); London (2012), [doi:10.1093/acprof:oso/9780199744374.001.0001](https://doi.org/10.1093/acprof:oso/9780199744374.001.0001) (`london2012hearing`)
- **Evidence level / confidence:** EV-L / Moderate (density); Weak (the switch threshold)
- **Affects:** SDR-19, SDR-20, SDR-21; H-02; GEN-001
- **Papers:** P03

### F-013 · About 20 minutes of training can halve point-estimation error

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** "A brief training period (around 20 min) can reduce performance error by 50% on a point estimation sonification task."
- **Evidence:** Walker & Nees (2011), *The Sonification Handbook* ch. 2, citing Walker & Nees (2005b), [URL](https://sonification.de/handbook/chapters/chapter2/) (`walker2011theory`)
- **Evidence level / confidence:** EV-L (secondary citation of the 2005b study) / Moderate
- **Affects:** SDR-37, A11Y-30; H-10
- **Papers:** P05

### F-014 · Continuous soundscapes support peripheral monitoring, on thin evidence

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** In a within-subjects study (n = 18), a continuous forest soundscape gave higher process-monitoring performance than visual-only or visual-plus-alert conditions, without significantly impairing the primary arithmetic task. SoNSTAR raised situational awareness with lower NASA-TLX workload than visual techniques. No independent replication of the 2016 study was found.
- **Evidence:** Hildebrandt, Hermann & Rinderle-Ma (2016), [doi:10.1016/j.ijhcs.2016.06.002](https://doi.org/10.1016/j.ijhcs.2016.06.002) (`hildebrandt2016continuous`); Debashi & Vickers (2018), [doi:10.1371/journal.pone.0195948](https://doi.org/10.1371/journal.pone.0195948) (`debashi2018sonification`); [notes/03 §5, Gaps](../research/notes/03-accessibility-hci.md)
- **Evidence level / confidence:** EV-L / Moderate (single main study)
- **Affects:** SDR-35; H-09
- **Papers:** P05

### F-015 · No empirical limit on concurrent data streams is known

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** Identification of concurrent earcons falls as their number grows, and timbre and onset differences only partly restore it. No empirical upper bound on concurrent streams in data sonification (as opposed to earcons) was found, so the 3–4 stream cap in SDR-26 is an inference.
- **Evidence:** McGookin & Brewster (2004), [doi:10.1145/1024083.1024087](https://doi.org/10.1145/1024083.1024087) (`mcgookin2004understanding`); [notes/02 §7, Gaps](../research/notes/02-theory-perception.md)
- **Evidence level / confidence:** EV-L / Moderate (earcons); Weak (the cap)
- **Affects:** SDR-26; H-04
- **Papers:** P03 (extension)

### F-016 · Critical-band spacing and comfort budgets lack validated thresholds

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** The rule to keep simultaneous tones more than about one critical band apart is extrapolated from pure-tone consonance data (Plomp & Levelt 1965, not re-read in full during the kickoff). No validated psychoacoustic comfort thresholds for long sonification sessions and no longitudinal listener-fatigue study were found.
- **Evidence:** `plomp1965tonal`, `glasberg1990derivation`, `fastl2007psychoacoustics`; [notes/02 §2d, §3 Gaps, §7 Gaps](../research/notes/02-theory-perception.md)
- **Evidence level / confidence:** EV-L (absence of evidence) / Weak
- **Affects:** SDR-28, SDR-39; H-11, H-12
- **Papers:** P03 (extension)

### F-017 · BLV study norms: about 13 participants, remote on own AT, paid $50–250 per session

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** The median sample size for disabled or older participant groups in 506 accessibility papers (2010–2019) was 13. Leading BLV studies ran remotely on participants' own assistive technology and paid $50 for 90 minutes (Zong et al. 2022) to $250 for 3 hours (Umwelt 2024). Gamified remote evaluation of sonification gave results comparable to the lab.
- **Evidence:** `mack2021accessibility` ([arXiv 2101.04271](https://arxiv.org/abs/2101.04271)); `zong2022rich`; `zong2024umwelt`; Ziemer (2024), [doi:10.21785/icad2024.007](https://doi.org/10.21785/icad2024.007) (`ziemer2024curat`); [notes/03 §6](../research/notes/03-accessibility-hci.md)
- **Evidence level / confidence:** EV-L / Moderate
- **Affects:** A11Y-34, SDR-42; all user-study protocols
- **Papers:** P03, P04, P05, P08

### F-018 · Interference between screen-reader speech and sonification is unmeasured

- **Date:** 2026-10-18 · **Phase:** Kickoff · **Type:** literature · **Status:** triaged
- **Statement:** No study was found that quantifies how screen-reader speech and concurrent sonification interfere with comprehension. A11Y-20's sequencing and ~20 dB separation rest on WCAG 1.4.7 guidance and Chart Reader's design, not on measurement.
- **Evidence:** [notes/03 §4, Gaps](../research/notes/03-accessibility-hci.md); [research report, "Publishable research sits in the field's admitted gaps"](../research/research-report.md#publishable-research-sits-in-the-fields-admitted-gaps)
- **Evidence level / confidence:** EV-L (absence of evidence) / Moderate
- **Affects:** A11Y-20; candidate hypothesis (not yet assigned; triage at B03)
- **Papers:** none yet

---

## Implementation findings

<!--
Append implementation findings below this comment, starting at F-019.
Use the template above. One entry per finding; link the PR, commit, render
manifest, benchmark row or EXP-### that carries the evidence. Set Status to
"new"; the research lead changes it at weekly triage.
-->

### F-019 · serde_json's default float parser can be one ULP off, which breaks full-precision round trips

- **Date:** 2026-10-19 · **Phase:** A03 · **Type:** engineering · **Status:** new
- **Statement:** With default features, `serde_json` 1.0 parsed `45.10693334729107` (written by its own shortest round-trip formatter) back as `45.106933347291076`, one ULP away, so a Score serialized and re-read was no longer byte-identical. Enabling the `float_roundtrip` feature (correct parsing at about twice the parse cost) fixed it; the A03 property test `canonical_form_round_trips_byte_for_byte` now guards it.
- **Evidence:** A03-T03 PR (proptest case committed in `crates/sonarithm-core/proptest-regressions/score/tests.txt`); serde_json feature documentation.
- **Evidence level / confidence:** EV-C / Strong (reproducible with the committed regression seed)
- **Affects:** ADR-0013, H-08 (cross-platform byte identity also needs exact parsing, not only exact formatting and `libm`)
- **Papers:** P07 (engine paper, determinism section)
