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

<!--
A02 (spikes) uses F-030 to F-049 so that it can append in parallel with A03, which
continues at F-021. IDs F-021 to F-029 belong to A03 entries.
A04 (data layer) uses F-070 to F-079.
Cross-phase build and repository infrastructure (the coordinator) uses F-090 to F-099.
A06 (NUMISONA) uses F-080 to F-089.
A08 (research instrumentation) uses F-110 to F-119.
A07 (CLI, Python package, first release) uses F-100 to F-109.
A05 (render engine, audio backends, export) uses F-050 to F-069.
-->

### F-019 · serde_json's default float parser can be one ULP off, which breaks full-precision round trips

- **Date:** 2026-10-19 · **Phase:** A03 · **Type:** engineering · **Status:** new
- **Statement:** With default features, `serde_json` 1.0 parsed `45.10693334729107` (written by its own shortest round-trip formatter) back as `45.106933347291076`, one ULP away, so a Score serialized and re-read was no longer byte-identical. Enabling the `float_roundtrip` feature (correct parsing at about twice the parse cost) fixed it; the A03 property test `canonical_form_round_trips_byte_for_byte` now guards it.
- **Evidence:** A03-T03 PR (proptest case committed in `crates/sonarithm-core/proptest-regressions/score/tests.txt`); serde_json feature documentation.
- **Evidence level / confidence:** EV-C / Strong (reproducible with the committed regression seed)
- **Affects:** ADR-0013, H-08 (cross-platform byte identity also needs exact parsing, not only exact formatting and `libm`)
- **Papers:** P07 (engine paper, determinism section)

### F-020 · Rust's float formatting breaks exact decimal ties upward, which RFC 8785 forbids

- **Date:** 2026-10-19 · **Phase:** A03 · **Type:** engineering · **Status:** new
- **Statement:** For the double `0x43143ff3c1cb0959` (exactly 1424953923781206.25), Rust's `{}` and `{:e}` formatting print the 17-digit tie as `1424953923781206.3`, while ECMAScript `Number.prototype.toString`, and therefore RFC 8785, require the even digit, `1424953923781206.2`. The `ryu` crate (used by serde_json) and Python's `repr` print `.2`. Spec canonicalization therefore takes its digits from `ryu`, and the RFC 8785 Appendix B number vectors are a unit test.
- **Evidence:** A03-T08 PR (`crates/sonarithm-spec/src/canonical.rs`, test `rfc_8785_appendix_b_numbers`); RFC 8785 Appendix B; ECMA-262 §6.1.6.1.20.
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** ADR-0013, H-08 (spec hashes must match those any RFC 8785 implementation computes, for example in Python or the browser)
- **Papers:** P07 (engine paper, determinism section)

### F-030 · WASAPI shared mode on the development laptop ignores the requested buffer size

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** On the owner's laptop (Realtek audio, Windows 11 build 26300, 48 kHz), the shared-mode engine period is fixed at 480 frames (10 ms): the driver reports the same minimum and maximum period to `IAudioClient3`. cpal 0.18.2 called back every 480 frames for `BufferSize::Fixed(128)`, `Fixed(256)`, `Fixed(512)` and `Default`, with 1,056 frames on the first fill. A requested 128-frame `IAudioClient3` period was clamped to 480.
- **Evidence:** A02-T01 harness, `spikes/cpal-latency` ([#9](https://github.com/DuhItzAniket/SONARITHM/pull/9)), `results/2026-10-19-windows-speakers.json`; cpal 0.18.2 source (`host/wasapi/device.rs`: "The callback period is always GetDevicePeriod()")
- **Evidence level / confidence:** EV-C / Strong for this machine; other drivers may offer smaller periods
- **Affects:** PB-01, PB-02 (the 128-frame soak cannot run at 128 frames on this driver); ADR-0004; R-03
- **Papers:** P07

### F-031 · Key press to cue onset is about 123 ms p95 through cpal in WASAPI shared mode, 2.5 times PB-01

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** benchmark · **Status:** new
- **Statement:** On the development laptop, a trigger on a control thread to the cue's onset in a WASAPI loopback capture of the same endpoint took p50 116.4 ms and p95 123.1 ms (p99 123.8, max 124.6; 200 trials) through cpal 0.18.2 at the 256-frame setting. The 128, 512 and default settings were within 1 ms of that. That misses PB-01 (≤ 50 ms p95) before webview input and IPC are added. The loopback tap excludes time after the audio engine, so the acoustic latency is at least this; acoustic confirmation is pending (the built-in microphone's echo cancellation removed the cue).
- **Evidence:** [#9](https://github.com/DuhItzAniket/SONARITHM/pull/9), `spikes/cpal-latency/results/2026-10-19-windows-speakers.json`; benchmarks-log row "A02 spike (Windows)"
- **Evidence level / confidence:** EV-C / Moderate (one machine; software method)
- **Affects:** PB-01; R-03 (trigger met); ADR-0004; 06 §9
- **Papers:** P07

### F-032 · An OEM enhancement APO adds about 38 ms, 20 dB of gain and a 200 ms tail to everything the app plays

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** On the development laptop the default Realtek endpoints process the output stream with an enhancement APO (Nahimic is installed). A −45 dBFS cue reached the loopback tap at −24.4 dBFS (+20.6 dB), with a reverb-like tail of about 200 ms. Opening the same endpoint with `AUDCLNT_STREAMOPTIONS_RAW` returned the cue at −45.0 dBFS and cut the trigger-to-onset time from p50 117 ms to p50 79.2 ms (p95 83.7 ms). cpal 0.18 cannot request RAW mode (cpal #1173). The gain is applied after SONARITHM's limiter, so 06 §10's level guarantees do not hold on such systems.
- **Evidence:** [#9](https://github.com/DuhItzAniket/SONARITHM/pull/9) (`raw` mode, `src/direct.rs`); same results file
- **Evidence level / confidence:** EV-C / Moderate (one machine and driver)
- **Affects:** PB-01; 06 §10 (hearing safety), SDR-24 (loudness compensation); R-03; proposed new risk on platform audio processing
- **Papers:** P07

### F-033 · cpal's predicted playback time is about 80 ms earlier than the loopback onset on Windows

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** For the same 200 cues, cpal 0.18.2 reported `playback − callback` as 31.8 ms (p50), while the cue appeared at the loopback tap 112 ms after the callback (p50). Even in RAW mode the tap came 74 ms after the callback. If loopback timestamps are accurate, a playhead extrapolated from cpal's timestamp (06 §9) would run about 80 ms ahead of the sound on this machine. A per-device latency calibration would remove the error.
- **Evidence:** [#9](https://github.com/DuhItzAniket/SONARITHM/pull/9), results file (`reportedMs`, `pathMs`)
- **Evidence level / confidence:** EV-C / Weak (rests on the accuracy of WASAPI loopback timestamps, not cross-checked acoustically)
- **Affects:** 06 §9 (clock synchronization); B04 linked cursor; ADR-0004
- **Papers:** P07

### F-034 · The owned render graph renders the benchmark Score at about 350 times real time without allocating

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** benchmark · **Status:** new
- **Statement:** The owned spike graph renders the 60 s benchmark Score (4 tracks, 10,000 notes, 64 preallocated voices) offline on one core:
  - 349×, 361× and 352× real time at 128-, 256- and 512-frame blocks (median of 3);
  - CPU per 256-frame block: p50 14.4 µs, p99 24.0 µs.

  Through cpal for 120 s at the 480-frame WASAPI period:
  - p99 is 85.7 µs per callback (0.86% load), with no deadline misses;
  - pinned to 4 E-cores, p99 is 151.5 µs.

  `process` runs inside `assert_no_alloc` in debug builds without aborting. The output is bit-identical at block sizes 64, 256 and 1000.
- **Evidence:** [#98](https://github.com/DuhItzAniket/SONARITHM/pull/98), `spikes/render-graph/README.md`; benchmarks-log row "A02 spike (Windows)"
- **Evidence level / confidence:** EV-C / Strong (one machine)
- **Affects:** PB-02, PB-03 (A05); ADR-0004
- **Papers:** P07

### F-035 · Firewheel 0.14 matches the owned graph bit for bit at equal CPU, but adds 43 crates and allocates on the audio thread by default

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** With the same voice, gain and limiter code, Firewheel 0.14.0 produced the same samples as the owned graph (null residual −∞ over 5.76M samples). It ran at 337× real time offline against 361×, with a real-time p99 of 85.4 µs against 85.7 µs. It added 43 crates and about 28,100 lines with 81 `unsafe` occurrences, but no synthesis voices (a stated non-goal). Its default `BufferOutOfSpaceMode::AllocateOnAudioThread` allocates on the audio thread when an event buffer fills; `Panic` avoids it.
- **Evidence:** [#103](https://github.com/DuhItzAniket/SONARITHM/pull/103), `spikes/render-graph/src/fw.rs` and README comparison table; unit test `firewheel_matches_owned_graph`
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** ADR-0004 (the graph choice); R-04
- **Papers:** P07

### F-036 · A render-graph contract that hosts an engine context must split into a control half and a real-time processor

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** Firewheel's `FirewheelContext` is `!Send`, and its `update` is not real-time safe; only `FirewheelProcessor::process` may run on the audio thread. The draft contract therefore has a non-real-time control half (`prepare`, `update`, `seek`; no `Send` bound) and a `GraphProcessor` that moves to the audio thread, matching 06 §2. The owned graph fits the same shape: its whole schedule goes to the processor at prepare time, and a seek is one atomic store.
- **Evidence:** [#98](https://github.com/DuhItzAniket/SONARITHM/pull/98), [#103](https://github.com/DuhItzAniket/SONARITHM/pull/103), `spikes/render-graph/src/graph.rs`
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** 06 §3, §6 (the `RenderGraph` and `AudioBackend` sketches); A05-T02
- **Papers:** none yet

### F-037 · Miri on Windows reports undefined behaviour in std's allocator whenever a global allocator wraps `System`

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** Under Miri (nightly 1.101.0, 2026-10-17, target `x86_64-pc-windows-msvc`), the spike's Firewheel test stopped with a Stacked Borrows error, and with a Tree Borrows error. Both were raised inside std's `sys::alloc::windows` while an over-aligned buffer was freed through `assert_no_alloc`'s global allocator. A 15-line program with a pass-through `#[global_allocator]` over `System` and one 128-byte-aligned `Box` reproduces both; on the `x86_64-unknown-linux-gnu` target it runs clean. With Miri's built-in allocator the Firewheel test passes. The report is therefore not attributable to Firewheel or to the spike's code.
- **Evidence:** [#103](https://github.com/DuhItzAniket/SONARITHM/pull/103) README, Miri section; reproduction program in the [A02 worklog entry](../sdlc/worklog/2026-10-19-a02-spikes.md)
- **Evidence level / confidence:** EV-C / Moderate. TODO(verify): whether a rust-lang/rust or rust-lang/miri issue already tracks it
- **Affects:** testing-strategy (Miri runs: use the Linux target or Miri's allocator); R-04
- **Papers:** none yet

### F-038 · assert_no_alloc is BSD-1-Clause, which deny.toml does not allow

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** triaged (2026-10-19: the coordinator, under the owner's delegation, approved `allocation-counter` 0.8.1 in place of `assert_no_alloc` for A05-T10, F-054. No licence or allow-list change; allowing BSD-1-Clause later stays the owner's choice)
- **Statement:** `assert_no_alloc` 1.1.2, which the coding standards name for the real-time allocation check, is licensed BSD-1-Clause. `cargo deny check licenses` with the repository's `deny.toml` rejects it. Every other spike dependency, including Firewheel's 43 crates, passes.
- **Evidence:** [#98](https://github.com/DuhItzAniket/SONARITHM/pull/98) (DoD notes); `cargo deny check licenses` run on `spikes/render-graph` with the root `deny.toml`
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** deny.toml (an allow-list change with rationale, or another crate) before A05; ADR-0010
- **Papers:** none yet

### F-039 · Offline renders are bit-identical across Linux, Windows and macOS on x86-64 and AArch64 when the render path avoids the platform maths library

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** benchmark · **Status:** new
- **Statement:** These renders produced the same SHA-256 on four machines (Linux x86-64, Windows x86-64 and macOS AArch64 CI runners, and the owner's laptop): the 60 s benchmark Score through the owned graph and through Firewheel, with polynomial sines, plan-time `libm` and basic IEEE 754 arithmetic. The Score bytes were also identical. A variant calling `f32::sin` per sample differed by OS at −144.5 dBFS (Windows) and −150.5 dBFS (macOS) against Linux, with ΔLUFS 0.0000, inside PB-06 (≤ −90 dBFS, ≤ 0.1).
- **Evidence:** [#104](https://github.com/DuhItzAniket/SONARITHM/pull/104) and its `spike-a02-determinism` workflow run; `spikes/render-graph/README.md`
- **Evidence level / confidence:** EV-C / Strong (single-threaded, fixed blocks, no SIMD intrinsics)
- **Affects:** PB-06; ADR-0013; 06 §8; H-08
- **Papers:** P07

### F-040 · A 1 MB binary Tauri response takes about 12 ms p95 on the development laptop and 8 ms or less on CI runners

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** benchmark · **Status:** new
- **Statement:** A Tauri 2.12.4 command returning `ipc::Response` with 1 MB had these round-trip times, all inside PB-09 (≤ 30 ms):
  - development laptop, WebView2 155 (3 runs of 100): p50 10.0–10.4 ms, p95 11.5–11.8 ms;
  - the same, pinned to 4 E-cores: p95 15.7–20.7 ms;
  - CI runners (one run each): p95 7.8 ms on Windows (WebView2 154), 8 ms on macOS (WKWebView, AArch64) and 7 ms on Linux (WebKitGTK 2.52.6 under Xvfb).

  10 MB took p50 80.5–81.6 ms on the laptop.
- **Evidence:** [#101](https://github.com/DuhItzAniket/SONARITHM/pull/101), `spikes/tauri-webview/results/ipc-windows-*.json` and the `spike-a02-tauri` workflow artifacts; benchmarks-log rows "A02 spike (WebView2/WKWebView/WebKitGTK)"
- **Evidence level / confidence:** EV-C / Moderate (CI runners are virtual machines; WKWebView and WebKitGTK timers have 1 ms resolution)
- **Affects:** PB-09; ADR-0002; 09 §2
- **Papers:** P07

### F-041 · JSON costs 3.4 times as much as binary in Tauri IPC, and 30 Hz Channel messages arrive in about 2 ms

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** benchmark · **Status:** new
- **Statement:** On the development laptop (WebView2 155):
  - returning 1 MB as a JSON array took p50 34–35 ms against 10.0–10.4 ms as binary;
  - 1 KB binary Channel messages at 30 Hz arrived one-way in p50 1.8–1.9 ms and p95 2.1–2.4 ms;
  - a burst of 64 KB Channel messages moved 147–156 MB/s.

  On the macOS CI runner the 30 Hz stream arrived at only 8 messages per second (max one-way 107 ms). That is consistent with App Nap or timer coalescing of a background app, a hypothesis to check on a real Mac.
- **Evidence:** [#101](https://github.com/DuhItzAniket/SONARITHM/pull/101) results and workflow artifacts
- **Evidence level / confidence:** EV-C / Moderate (Windows); EV-D / Weak (the macOS observation)
- **Affects:** 09 §2 (no JSON on hot paths; meters on Channels); 06 §2 telemetry pump on macOS
- **Papers:** none yet

### F-042 · A strict CSP without the IPC origins silently turns Tauri's binary IPC into JSON

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** With `default-src 'self'` and no `connect-src ipc: http://ipc.localhost`, Tauri 2.12.4 on WebView2 fell back to its postMessage IPC without an error. A command returning `ipc::Response` then delivered a JSON number array instead of an `ArrayBuffer`. Adding the two origins to `connect-src` restored the binary path.
- **Evidence:** [#101](https://github.com/DuhItzAniket/SONARITHM/pull/101) (README; `tauri.conf.json`)
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** 09 §4 (CSP); B01 (a test that the binary path is in use)
- **Papers:** none yet

### F-043 · In WebView2, a moving playhead over a 1M-point waveform holds 60 fps on both GPUs, but a full 1M-point redraw every frame does not on the integrated GPU

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** benchmark · **Status:** new
- **Statement:** On a 2016 × 630 WebGL2 canvas in WebView2 155 on a 60 Hz panel, these cases ran at 60 fps on both the Intel UHD and the RTX 4050, with at most 0.6 ms of script per frame:
  - a CSS-transformed playhead over a 1M-point waveform drawn once;
  - the waveform reduced by M4 per pixel column (Intel GPU time p50 1.7 ms);
  - a 2048 × 512 scrolling spectrogram (Intel GPU time p50 1.7 ms).

  Redrawing all 1M points every frame reached 37 fps on the Intel GPU (GPU p50 26 ms) and 60 fps on the RTX 4050 (GPU p50 0.6 ms).
- **Evidence:** [#102](https://github.com/DuhItzAniket/SONARITHM/pull/102), `spikes/tauri-webview/results/webgl-windows-*.json`
- **Evidence level / confidence:** EV-C / Strong (one machine)
- **Affects:** PB-08; ADR-0002; notes/05 §6; 09 §2 (`view_get` M4 tiles)
- **Papers:** P07

### F-044 · The WebView2 switch `--force_high_performance_gpu` moves WebGL to the discrete GPU of a hybrid laptop

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
- **Statement:** On the development laptop (Intel UHD plus RTX 4050), WebView2's default WebGL renderer was the Intel GPU. Passing `--force_high_performance_gpu` through Tauri's `additional_browser_args` gave "ANGLE (NVIDIA, NVIDIA GeForce RTX 4050 Laptop GPU …)", and `--force_low_power_gpu` kept the Intel GPU. Tauri's default browser arguments must be repeated when replacing them.
- **Evidence:** [#102](https://github.com/DuhItzAniket/SONARITHM/pull/102) (renderer strings in the results)
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** B04 (an optional high-performance setting); 09 §1
- **Papers:** none yet

### F-045 · In WebView2, React Aria's Tree is a tree grid whose row names repeat the expand button label

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** accessibility · **Status:** new
- **Statement:** A UI Automation dump of the A02 prototype in WebView2 155 found the expected roles: the Tree, Toolbar, toggle button, Slider, radio group, landmarks and the `role="application"` explorer. It also found three things for the walkthroughs to check:
  - React Aria Components 1.22's Tree is exposed as a tree grid, and each row's cell is named from its whole content, including the expand button: "Collapse temps.csv, 20 rows temps.csv, 20 rows";
  - `SliderOutput` is a `status` live region;
  - React Aria adds live regions of its own next to the app's announcer.

  `UiaRaiseNotificationEvent` on the window's host provider returned success. No screen reader was run.
- **Evidence:** [#107](https://github.com/DuhItzAniket/SONARITHM/pull/107), `spikes/tauri-webview/results/a11y-uia-windows.txt`
- **Evidence level / confidence:** EV-D / Weak (an automated tree dump, not a screen-reader session)
- **Affects:** ADR-0003; OD-05; A11Y-17; 08 §4–5; B02 tree component
- **Papers:** none yet

### F-070 · CSV ingest of 1M rows × 5 columns takes about 0.51 s on the development laptop and 0.91 s on four E-cores, inside PB-05

- **Date:** 2026-10-19 · **Phase:** A04 · **Type:** benchmark · **Status:** new
- **Statement:** `sonarithm_data::read_bytes` on a 46 MB synthetic CSV (1M rows: an ISO-8601 timestamp with offset, a float, an integer, a 4-level category, a ratio), including encoding and dialect sniffing, column type inference, semantic inference and provenance normalization, took a median of 506 ms (95% CI 504–509 ms, 10 samples) on HW-01 and 912 ms (904–921 ms) on HW-01e. Without semantic inference it took 433 ms and 792 ms. PB-05 allows 1.5 s; the HW-01 result is 34% of the budget, below the 50% at which 12 §8 asks for a re-measurement, and the four-E-core stand-in is 61%.
- **Evidence:** criterion bench `pb05_csv_ingest_1m_rows_5_columns` in `crates/sonarithm-data/benches/csv_ingest.rs` (release profile, Rust 1.99.0, criterion 0.8.2, single-threaded ingest); A04 PRs; benchmarks-log row "A04"
- **Evidence level / confidence:** EV-C / Moderate (one machine, other processes running; HW-01e is a stand-in for the reference laptop, not the reference laptop)
- **Affects:** PB-05; NFR-01; 03; 10 §2
- **Papers:** P07

### F-071 · arrow-ipc 60 and calamine 0.37 panic or allocate what a malformed file declares, so the data layer validates and contains them

- **Date:** 2026-10-19 · **Phase:** A04 · **Type:** engineering · **Status:** new
- **Statement:** Short local fuzzing runs (90–300 s per target, cargo-fuzz with ASan on Windows) found four ways untrusted files stopped the process in third-party readers: arrow-ipc allocated a buffer of a block's declared body length and an LZ4 buffer's declared uncompressed size (2^58 bytes), and panicked on a validity bitmap shorter than its column (in arrow-buffer); calamine panicked in `Range::range` on a zero-width range and allocates a sheet's whole bounding box (two cells at A1 and XFD1048576 ask for about 17 billion cells). `sonarithm-data` now checks IPC framing, buffer bounds and declared uncompressed sizes against the input and the limits before decoding, streams XLSX cells and checks the bounding box against the cell limit before allocating it, and turns decoder panics into `SNR-DATA-001` errors. After the fixes, one-hour runs found nothing (`csv_read` 692,702 inputs, `xlsx_read` 5.0 million) and neither did a ten-minute `json_read` run (1.7 million). The first Linux run of `columnar_read` (the two-minute smoke run on the A04-T10 pull request) found one more, after decoding: arrow-array's `normalized_keys` asserts that a dictionary has values, and a fuzzed file's provenance column was a dictionary with none. That panic was outside the decoder containment, so `sonarithm-data` now converts dictionary keys itself and reports keys outside the values instead of clamping them. The next run found a 1.8 KB Parquet file on which parquet 60's page-header decoder spent about 12 s before failing (a libFuzzer timeout, not a crash): parquet skips Thrift fields it does not know without bounding their declared sizes, and on its page-header reader a skip past the end of input succeeds silently, so an unknown field holding a list of 268 million UUIDs looped for seconds (by the same code, read but not fuzzed, booleans in a collection are skipped as zero bytes, which allows the same in the footer). `sonarithm-data` now walks the footer and every page header with its own bounded Thrift reader before parquet decodes them, which rejects that file at once ([#146](https://github.com/DuhItzAniket/SONARITHM/issues/146)). The run after that found a file whose timestamp field had an empty time-zone string: the Arrow format treats it as no time zone and arrow-ipc writes it back as none, so the loaded table failed its own round trip; Arrow IPC and Parquet loads now remove empty time zones at any depth. The first nightly run (2026-10-20) found a file without record batches whose provenance column was a dictionary of a union without members: arrow-data panics building even an empty column of that type, again after decoding and outside the containment, so loads now check that Arrow can build an empty column of every field type. Allocation failures cannot be caught, and other formats (Parquet, ODS, XLS, XLSB) have not shown these failures yet but are not proven safe.
- **Evidence:** fuzz targets `csv_read`, `json_read`, `xlsx_read`, `columnar_read` in `crates/sonarithm-data/fuzz`; regression inputs in `crates/sonarithm-data/tests/corpus/fuzz` and the tests `fuzzer_findings_stay_fixed`, `a_dictionary_without_values_has_only_null_indices`, `a_list_of_uuids_larger_than_the_input_is_rejected`, `parquet_written_by_pyarrow_passes_the_thrift_walk`, `empty_time_zones_are_removed_at_any_depth`, `types_without_an_empty_column_are_rejected`, `declared_lengths_past_the_end_are_rejected_before_allocation`, `compressed_buffers_must_declare_sizes_within_the_limits` and `far_apart_cells_hit_the_cell_limit_instead_of_allocating_their_bounding_box`
- **Evidence level / confidence:** EV-C / Moderate (minutes of fuzzing, not hours; on Windows, libFuzzer treats even a caught panic as a crash, so panic containment is checked by unit tests and by the Linux nightly run)
- **Affects:** NFR-05; 10 §2 ("run untrusted input through fuzzed parsers"); 12 §6
- **Papers:** none yet

### F-090 · Debug info and incremental caches made up most of 70 GB of build output across four agent worktrees

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** engineering · **Status:** new
- **Statement:**
  - Four parallel agent worktrees held about 70 GB of `target/` output after a day of stacked rebases. The largest was 37.3 GB: 20.7 GB in `debug/deps` and 13.0 GB in `debug/incremental`. Incremental caches alone were about 25 GB across the three largest worktrees.
  - On a clean build of the workspace test binaries at commit 43bc462 (Windows, `CARGO_INCREMENTAL=0`), line-table debug info for workspace crates and none for dependencies cut the build from 982 MB to 528 MB. PDB files went from 370 MB to 135 MB and rlibs from 444 MB to 228 MB. Wall time was 33 s against 25 s, from one run each.
- **Evidence:** this PR; scratch measurement with `cargo test --workspace --no-run` into two target directories, the second with `--config 'profile.dev.debug="line-tables-only"' --config 'profile.dev.package."*".debug=false'`; robocopy size listing of the worktrees before cleanup
- **Evidence level / confidence:** EV-D / Moderate (one machine, one run per configuration)
- **Affects:** dev-environment.md "Disk usage"; root `Cargo.toml` profiles
- **Papers:** none yet

### F-080 · A polarity flip mirrors pitches in the pitch space's own coordinate, which equals the cents mirror only in log-frequency

- **Date:** 2026-10-19 · **Phase:** A06 · **Type:** engineering · **Status:** new
- **Statement:** NUMISONA applies negative polarity to the scale position (`u → 1 − u`) before the pitch space, so equal data steps stay equal steps in the chosen space for both polarities. A flip therefore mirrors pitches in that space's coordinate. In the default log-frequency space (SDR-09) this is exactly the cents mirror `low + high − pitch` of 05 §4.2 invariant 4; in linear-frequency, mel, ERB-rate and Bark it is not. For 400–1400 Hz in linear frequency, a pitch at a quarter of the range (650 Hz) flips to 1150 Hz, while the cents mirror would give about 860 Hz.
- **Evidence:** A06-T03 PR (`crates/engines/sonarithm-numisona/src/polarity.rs`), property test `sdr_15_polarity_flip_symmetry` (mirror in every space's coordinate; cents mirror within 1e-9 cents in log-frequency)
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** 05 §4.2 invariant 4 (clarified to "mirrors within the range in the pitch space"), SDR-09, SDR-15
- **Papers:** none yet

### F-081 · Spec v0.1 can express 8 of Erie's 15 documented encoding channels, and neither sequential composition nor intervals

- **Date:** 2026-10-19 · **Phase:** A06 · **Type:** engineering · **Status:** new
- **Statement:** Of the 15 encoding channels in Erie's documentation, NUMISONA's importer maps 8 onto spec v0.1: `time`, `pitch`, `duration`, `tapCount`, `loudness` and `pan` closely, and `tapSpeed` (as tap counts) and `detune` (as pitch in cents) approximately. The other 7 (`time2`, `repeat`, `speechBefore`, `speechAfter`, `postReverb`, `modulation`, `harmonicity`) have no v0.1 equivalent, nor do 3D pan, `roundToNote` in Analysis mode (SDR-10) or the `fold`, `density`, `quantile`, `boxplot` and `diffing` transforms. A `sequence` becomes one spec per stream, because a v0.1 spec overlays its tracks and has no sequential composition. On the 8-spec gallery subset, every imported spec validates and the report lists 19 approximated or unmapped constructs.
- **Evidence:** A06-T08 PR (`crates/engines/sonarithm-numisona/src/import/erie.rs`; `tests/erie_gallery.rs` and its snapshots); [Erie documentation](https://see-mike-out.github.io/erie-documentation), encoding and stream pages, read 2026-10-19
- **Evidence level / confidence:** EV-C / Moderate (the documentation does not state every default, such as Erie's default pitch range and band, so some mappings rely on SONARITHM's defaults)
- **Affects:** SDR-05; 04 §9; candidate spec minor versions (sequential composition, intervals, per-event speech); the broader importer in C05
- **Papers:** none yet

### F-082 · The encoder needs default ranges that no design rule fixes; three are recorded as proposals

- **Date:** 2026-10-19 · **Phase:** A06 · **Type:** engineering · **Status:** new
- **Statement:** RFC-0001 gives constant defaults for duration (0.25 s), taps (1) and loudness (−18 dB), but no range for a field mapped to them, and A11Y-13's redundant encodings need one. NUMISONA uses three proposals, each reported as `Info` when applied: duration 0.1–0.3 s (the ~100 ms event floor behind SDR-20, INFER, up to the 1/3 s inter-onset interval of the default pace, SDR-19); taps 1–8 (the channel's RFC bounds); loudness −24 to −12 dB (±6 dB around the default; SDR-23 warns against mapping loudness at all). At the default pace and duration, 8 taps last 31 ms each, below the ~100 ms floor, so the encoder warns that taps may fuse; redundant taps need a slower pace or longer notes to be countable.
- **Evidence:** A06-T04 PR (`crates/engines/sonarithm-numisona/src/encode.rs` constants `PROPOSED_DURATION_RANGE_S`, `TAPS_RANGE`, `PROPOSED_LOUDNESS_RANGE_DB`; test `sdr_20_taps_that_may_fuse_warn`); render plan splits a note into equal taps (`sonarithm-render` plan builder)
- **Evidence level / confidence:** EV-C / Weak (the values are design proposals; no listening study supports them yet)
- **Affects:** A11Y-13, SDR-19, SDR-20, SDR-23; sound-design §1 (proposals need a finding before they are settled); H-01 (redundant encodings for BLV point estimation); a candidate experiment on countable tap rates
- **Papers:** none yet

### F-083 · NUMISONA compiles 100k rows × 4 tracks in 35 ms on the development laptop and 54 ms on four E-cores, inside PB-04

- **Date:** 2026-10-19 · **Phase:** A06 · **Type:** benchmark · **Status:** new
- **Statement:** Compiling a spec with four tone tracks over 100,000 preloaded rows, through the real compiler with NUMISONA (validation, defaults, encoding, Score invariant check), takes 35.1 ms median on HW-01 and 54.3 ms on HW-01e (four E-cores), 18% and 27% of PB-04's 200 ms. A million rows take 135 ms with one track and 362 ms with four on HW-01, and 192 ms and 545 ms on HW-01e, inside the 1 s budget. Two hot-path changes before the final measurement cut the HW-01 times by about 40% (from 59, 209 and 594 ms): reading the provenance columns only when a confidence column exists, and resolving a constant timbre once per track instead of once per note.
- **Evidence:** A06-T09 PR, criterion group `pb04_compile` (`crates/engines/sonarithm-numisona/benches/compile.rs`), 20 samples per benchmark, release profile, Rust 1.99.0, criterion 0.8.2; HW-01e via `ProcessorAffinity = 0xF0000`; [benchmarks log](benchmarks-log.md) "A06"
- **Evidence level / confidence:** EV-C / Moderate (one machine; HW-01e is a stand-in for the reference laptop, 12 §8)
- **Affects:** PB-04, NFR-01; the A06 exit criterion "Compile ≤ 200 ms for 100k rows × 4 tracks"
- **Papers:** P07 (engine paper, compile performance)

### F-084 · NUMISONA calls a series volatile beyond three trend reversals, the most the cited density study used

- **Date:** 2026-10-19 · **Phase:** A06 · **Type:** engineering · **Status:** new
- **Statement:** SDR-21 asks that volatile series be slowed or smoothed, because trend reversals hurt estimates more than density (Nees & Walker 2008). The encoder counts sign changes between successive non-zero steps of the pitch field and, above three reversals, reports `SNR-ENG-012` with the count and rate and advises a `smooth` transform or a slower pace; it never alters the data. Three is the largest reversal count the research notes report for the study's stimuli ("one or three reversals"), so the exact levels need checking against the paper (TODO(verify)). Without hysteresis, noisy series exceed it easily: the DS-001-like temperature example reverses 76 times in 120 points.
- **Evidence:** A06-T09 PR (`encode::STUDIED_REVERSALS`, test `sdr_21_volatile_series_get_smoothing_advice`); research notes 02 §2 (Nees & Walker 2008)
- **Evidence level / confidence:** EV-C / Weak (the threshold is anchored on a summary of the study, not the paper itself)
- **Affects:** SDR-21; ECHOMETRIC lint (C05), which should own this check with hysteresis; H-02 and EXP-002 (reversals × density)
- **Papers:** none yet

### F-110 · Nees & Walker (2008) crossed 1, 2, 4 and 8 points/s with 0, 1, 2 and 3 reversals, but publish no generating function

- **Date:** 2026-10-19 · **Phase:** A08 · **Type:** literature · **Status:** new
- **Statement:** The study's graphs crossed four densities (1, 2, 4, 8 points/s) with four trend-reversal levels (0, 1, 2, 3), each initially rising or falling (counterbalanced). Every graph was a stock price over a 10-hour day at 1 s per hour, spanning $6–$106 and reaching both ends, sounded as 125 ms piano tones with 10 ms ramps (10.125 s per graph), without reference tones or clicks; every hour from 8 a.m. to 6 p.m. was queried in random order (Exp. 1, N = 32), and Exp. 2 asked for the local trend of each hourly interval (increased, decreased, stayed the same; N = 32). The paper gives no generating function (Fig. 1 is labelled schematic) and no point count; 10·d + 1 points (11, 21, 41, 81) is inferred from the duration and the figure. The published top frequency, 1979.5 Hz for MIDI B6, differs from equal temperament (1975.5 Hz), probably a typo. This resolves the TODO(verify) in F-084: three is the largest studied reversal count, so NUMISONA's volatility threshold of three stands.
- **Evidence:** Nees & Walker (2008), ACM TAP 5(3), Art. 13, [doi:10.1145/1402236.1402237](https://doi.org/10.1145/1402236.1402237), §2.1.3–2.1.6 and §3.1, read from the author PDF; GEN-001 module documentation (`sonarithm_echometric::harness::generators::reversals`); tests `sdr_19_gen_001_reproduces_the_studied_densities`, `sdr_21_gen_001_reproduces_the_studied_reversal_levels`
- **Evidence level / confidence:** EV-L / Strong (read from the full text); the point counts and schematic shapes are inferences (EV-D)
- **Affects:** SDR-19, SDR-20, SDR-21; F-084; GEN-001; H-02, H-03; the research notes 02 §2 summary ("one or three reversals" describes a result, not the levels)
- **Papers:** P01, P03

### F-111 · Smith & Walker (2005) used a single price series whose values are not published; its stated 200–1900 Hz is rounded

- **Date:** 2026-10-19 · **Phase:** A08 · **Type:** literature · **Status:** new
- **Statement:** Both studies sounded one series, a stock over a 10-hour day with two values per hour: $50 open, $84 maximum, $10 minimum, close equal to the open. The paper does not tabulate the values; Fig. 2 plots the hourly answers (within about ±$2). Study 1 played 20 sine tones of 0.5 s (10 s) while Study 2 glided through 21 values, so whether the 6 p.m. close sounded is ambiguous. The mapping $ = 0.0765·f^0.9256 gives 193.4 Hz for $10 and 1927.6 Hz for $84, which the paper rounds to "200–1900 Hz". The reference tone's beep rate, length and level are not published. Every hour was queried in random order (11 per test). GEN-002 therefore rebuilds the series from the stated anchors and Fig. 2, puts the $84 maximum at 10:30 and records this as a deviation.
- **Evidence:** Smith & Walker (2005), Applied Cognitive Psychology 19(8):1065–1087, [doi:10.1002/acp.1146](https://doi.org/10.1002/acp.1146), pp. 1069–1070, 1074, 1076, read from the author PDF; GEN-002 (`sonarithm_echometric::harness::generators::prices`, `FIG2_HOURLY`); test `sdr_17_gen_002_dynamic_reference_is_the_maximum_while_rising_and_the_minimum_while_falling`
- **Evidence level / confidence:** EV-L / Strong for the stated parameters; the reconstructed half-hour values are EV-D (figure-read)
- **Affects:** SDR-16, SDR-17, SDR-22; GEN-002; DS-002; the gallery item for context cues (A08-T02); H-01
- **Papers:** P01, P03

### F-112 · Tucker Brown et al. (2022) define SNR as t_d·√N_t/σ over box transits; positions are not published

- **Date:** 2026-10-19 · **Phase:** A08 · **Type:** literature · **Status:** new
- **Statement:** The light curves had 1000 points with box-shaped transits 10% of the length wide and a "fractional transit depth" of 25, Gaussian noise, and 0, 1 or 2 transits; SNR = t_d·√N_t/σ, with N_t the points in a transit, at levels 3, 5, 7, 10, 30 and 100 (18 curves, 6 without a signal). With astronify's percent depth on its default baseline of 100, N_t = 100 and σ = 250/SNR (inferred; Fig. 1 agrees). Transit positions, the noise of the zero-signal curves, seeds and the astronify version are not stated. The sonification mapped each curve's range linearly onto 440–880 Hz, one tone per point every 0.01 s with 0.2 s notes; astronify's current defaults (0.5 s notes, 100–10 000 Hz) differ.
- **Evidence:** Tucker Brown, Harrison, Zanella & Trayford (2022), MNRAS 516(4):5674–5683, [doi:10.1093/mnras/stac2590](https://doi.org/10.1093/mnras/stac2590), §2.1–2.2, read from the accepted version arXiv:2209.04465v1 (the published PDF was not reachable); GEN-003 (`sonarithm_echometric::harness::generators::transits`); test `sdr_40_gen_003_noise_follows_the_published_snr_definition`
- **Evidence level / confidence:** EV-L / Strong for the definition and levels; Moderate for the inferred units
- **Affects:** SDR-40; GEN-003; F-010; P03 benchmark task
- **Papers:** P01, P03

### F-113 · Elmquist et al. (2024) give the added noise as both "0.02 − 20" and "20% to 0.2%"; σ = 0.07 over 300 items gives the published r ≈ 0.95

- **Date:** 2026-10-19 · **Phase:** A08 · **Type:** literature · **Status:** new
- **Statement:** The Parallel Chords discrimination task asks which of axes B, C, D correlates most strongly with A; 300 items per axis, A and the target ~ N(i/300, 0.07²), the two distractors ~ N(i/300, (0.07 + Δσ)²), values outside [0, 1] redrawn, data normalized last, positive and negative correlations. The equation legend gives Δσ as "(0.02 − 20)" while the prose gives 20 levels from 20% to 0.2% (the thresholds in Table 2 read as percent), so GEN-004 takes Δσ in range units, 0.002–0.2. The level spacing and the normalization method are not stated. GEN-004 reproduces the published target strength: r = 0.94 for the target at seed 20261019 (pinned manifest), against r ≈ 0.95 in the paper.
- **Evidence:** Elmquist et al. (2024), Personal and Ubiquitous Computing 28(5):657–676, [doi:10.1007/s00779-024-01795-8](https://doi.org/10.1007/s00779-024-01795-8), §5.1–5.2, Eqs. 1–4, Table 2, read from the open-access PDF; GEN-004 (`sonarithm_echometric::harness::generators::correlation`); tests `sdr_40_gen_004_the_target_is_the_strongest_correlation`, `gen_004_manifest_is_pinned`
- **Evidence level / confidence:** EV-L / Strong for the equations; the unit of Δσ is an interpretation (Moderate)
- **Affects:** SDR-40; GEN-004; P03 benchmark task
- **Papers:** P01, P03

### F-050 · The master limiter keeps stress Scores at −1.16 dBTP or below, measured by an independent BS.1770 meter

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** engineering · **Status:** new
- **Statement:** The A05 true-peak limiter estimates inter-sample peaks with a 4× polyphase interpolator (48-tap Blackman-windowed sinc), aims 0.2 dB under the −1 dBTP ceiling, and adds 2 ms of look-ahead (101 frames of latency at 48 kHz). Two stress Scores, rendered offline at 48 kHz, were measured with `ebur128` 0.1.10 in true-peak mode:
  - 16 simultaneous band-limited square waves (A5 up to A6 + 1100 cents) at +12 dB track gain: −1.186 dBTP;
  - saw sweeps over six octaves, 8-tap notes and earcons at +12 to +18 dB: −1.162 dBTP.

  Both stay below −1 dBTP, and in both the limiter was engaged (sample peaks above 0.8). Signals below the ceiling pass bit for bit after the delay.
- **Evidence:** A05-T09 PR, `crates/sonarithm-render/tests/hearing_safety.rs` (`the_true_peak_never_exceeds_minus_1_dbtp`, printed values)
- **Evidence level / confidence:** EV-C / Moderate (two synthetic stress cases, one meter)
- **Affects:** NFR-10; 06 §10; sound-design §12 HS-02; the export loudness targets (sound-design §6.3)
- **Papers:** P07

### F-051 · In WASAPI exclusive mode, HW-01's Realtek driver signals two or three events per hardware period and queues about 100 ms

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** engineering · **Status:** new
- **Statement:** On HW-01 (Realtek(R) Audio speakers, 48 kHz), the A05 `WasapiBackend` in event-driven exclusive mode got a 144-frame buffer (3 ms minimum period), but events arrived in clusters: one gap of about 6.5 ms, then two or three events within 0.3 ms. Every event accepted a full 144-frame buffer. By the device clock (`IAudioClock`), frames written ran about 5000 frames (about 104 ms) ahead of frames played, with 0 underruns, in a 1 s silent run. When the backend skipped the extra events to keep two buffers queued, the device clock stopped and the stream stalled. So on this driver, exclusive mode may not reduce output latency at all. Only the cable test can show the real figure (ADR-0016 confirmation).
- **Evidence:** A05-T13 PR, silent test `wasapi_opens_the_default_device_exclusively_and_silently` (`buffer_frames: 144`, reported latency 104.2 ms, 0 xruns) and a temporary trace of event times and clock positions
- **Evidence level / confidence:** EV-C / Moderate (one device, one driver, a 1 s run)
- **Affects:** ADR-0016 (exclusive mode as the low-latency option); PB-01; R-03
- **Papers:** P07

### F-052 · Through the engine's command path, RAW shared mode measures 104 ms p95 key-to-onset in the loopback self-test, and the engine's prediction is 83 ms early

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** benchmark · **Status:** new
- **Statement:** The A05 latency tool sends 200 cues through `Command::Cue`, spaced 300–600 ms apart (seeded), and captures the output's WASAPI loopback. On HW-01 with `WasapiBackend`:
  - RAW shared mode (RAW in effect, 1056-frame buffer, 10 ms period, cues at −36 dBFS): 200 of 200 detected; p50 99.2 ms, p95 104.0 ms, p99 104.4 ms, max 104.6 ms; 0 xruns. A second run gave p50 98.8 ms and p95 103.7 ms.
  - Default (non-RAW) shared mode: p50 138.5 ms, p95 145.6 ms. That is about 40 ms more, consistent with the OEM APO's 38 ms (F-032).

  The calibration offset (median onset minus the host time the engine published for the block that started the cue) is 82.6 ms in RAW mode and 120.9 ms in default mode. The engine's own prediction counts only the queued buffer (about 12 ms), so playhead anchors need the calibrated offset (ADR-0016, 06 §9). These numbers include the command path (up to one 10 ms period before the next callback) and the shared buffer, so they are 15–20 ms above A02's direct RAW renderer (83.7 ms p95, F-032), which timed the tone from inside the callback. The loopback measures the software path only. PB-01 (50 ms) is not met in shared mode.
- **Evidence:** A05-T13 PR; `cargo run -p sonarithm-audio --example latency --release -- --trials 200 --level -36` (and `--mode shared`); benchmarks-log row "A05 (Windows, WASAPI loopback)"
- **Evidence level / confidence:** EV-C / Moderate (software loopback, one machine)
- **Affects:** PB-01; ADR-0016; F-033; 06 §9; R-03
- **Papers:** P07

### F-103 · The musl build of release-plz cannot render any changelog; the glibc build can

- **Date:** 2026-10-21 · **Phase:** A07 · **Type:** engineering · **Status:** new
- **Statement:** Every `release-plz / dry-run` run since the release tooling landed failed at "Preview the release PR" with "Template parse error … stack limit reached", pointing at line 1, column 1 of the changelog header. The binary that `taiki-e/install-action` installs for release-plz 0.3.172 is the x86_64 musl build. On Ubuntu, `release-plz update` fails with that build and passes with the glibc build of the same version, with or without a `header` in `[changelog]`. On Windows, 0.3.173 passes. The parser under Tera (pest 2.9) refuses to start when the free stack it measures is under 64 KiB. Stack bounds are measured through `stacker`, so the musl build most likely measures too little (not traced further). `release-plz/action` installs release-plz with cargo-binstall, which picks the glibc build, so only the dry run was affected. The dry run now downloads the glibc build and checks its SHA-256.
- **Evidence:** the A07 release-plz fix PR; `release-plz update --config release-plz.toml` run under WSL Ubuntu 24.04 (glibc 2.39) with the musl and the glibc 0.3.172 builds, each with and without the header (2 × 2: both musl runs fail, both glibc runs pass); CI run 39512839937; `pest` 2.9.4 `parser_state.rs` (`REQUIRED_STACK_SPACE`)
- **Evidence level / confidence:** EV-C / Strong for the failure and the fix; the musl mechanism is inferred
- **Affects:** `release-plz.yml`, supply-chain release tools, release-process
- **Papers:** none

### F-102 · Every Score hash changes with every release, because the header records the crate versions

- **Date:** 2026-10-21 · **Phase:** A07 · **Type:** engineering · **Status:** new
- **Statement:** The Score header lists each contributing engine with its crate version (ADR-0013), and the canonical Score hash covers the header, so bumping the workspace version from 0.0.0 to 0.1.0-alpha changed the pinned Score hash and all six review snapshots of NUMISONA's encoder tests while no sound changed. Test snapshots and pinned hashes now write engine versions as `*` before hashing; render manifests and the provenance a paper cites keep the real versions, so two releases that sound the same still have different Score hashes.
- **Evidence:** A07 PR "make Score snapshots independent of the release version": a full workspace test run at version 0.1.0-alpha failed only `encoded_scores_hash_identically_on_every_platform` and `the_example_datasets_encode_as_reviewed`; both pass at both versions after the change
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** ADR-0013, testing-strategy §4, release-process (release PRs), papers that compare Score hashes across releases
- **Papers:** none yet

### F-100 · The same rows from pyarrow, pandas and polars reach Rust with different layouts and metadata, which changed the Score hash

- **Date:** 2026-10-21 · **Phase:** A07 · **Type:** engineering · **Status:** new
- **Statement:** Four rows of two float columns and a text column handed to the Python package gave two different Score hashes: pandas 3.0.6 (through pyarrow 26) exports text as `large_string` and adds a `pandas` schema-metadata entry that records the pandas version and index, while polars 2.0.0 exports `string_view` and no metadata. The data fingerprint covers types and schema metadata (03 §2), so a pandas upgrade alone would have changed every fingerprint. `read_batches` now gives in-memory Arrow one layout for text and bytes (plain arrays up to 2 GiB per batch) and keeps only `snr.*` schema metadata; a dict, a pyarrow table and record batch, a pandas and a polars DataFrame then give one Score hash.
- **Evidence:** A07-T03 PR: `the_same_values_fingerprint_alike_from_any_library` (sonarithm-data), `test_the_same_values_compile_alike_from_every_library` (pytest, five input routes)
- **Evidence level / confidence:** EV-C / Strong (measured with the library versions named; other versions may add other metadata, which is dropped the same way)
- **Affects:** ADR-0013 (determinism), 03 §2 (fingerprints), FR-004, 11 §3
- **Papers:** none yet

### F-101 · arrow-pyarrow 60 is built on PyO3 0.29, not the 0.30 that 11 §3 names

- **Date:** 2026-10-21 · **Phase:** A07 · **Type:** engineering · **Status:** new
- **Statement:** The Arrow PyCapsule import maintained with arrow-rs (`arrow-pyarrow` 60.0.0, Apache-2.0) depends on PyO3 0.29, and `pyo3-arrow` 0.20 (PyO3 0.30) is built on arrow 59, so no published crate pairs arrow 60 with PyO3 0.30. The Python package uses PyO3 0.29.3 with `arrow-pyarrow` 60, which reads any `__arrow_c_stream__` or `__arrow_c_array__` object without `unsafe` code in SONARITHM; it moves to PyO3 0.30 when arrow-pyarrow does.
- **Evidence:** crate manifests of arrow-pyarrow 60.0.0 and pyo3-arrow 0.20.0 (crates.io, 2026-10-19); `bindings/python/Cargo.toml`
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** 11 §3, tech stack
- **Papers:** none yet

### F-053 · The A05 render graph renders the benchmark Score at 91–93× real time on HW-01, but 44× on four E-cores, below PB-03's 50×

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** benchmark · **Status:** new
- **Statement:** The DS-006 benchmark Score (`sonarithm_render::fixtures::benchmark_score`: 60 s, 4 tracks, 10,000 notes; mallet, square, sine and triangle voices, 112 voices preallocated) renders offline on one thread at 256-frame blocks, 48 kHz:
  - criterion `pb_03/render_benchmark_score_60s_256`: 656 ms, which is 91.4× real time; the plan builds in 3.05 ms;
  - `render_speed`, median of 5 runs: 92–96× on HW-01;
  - 43–46× pinned to four E-cores (HW-01e).

  The figure on HW-01 uses 54% of the budget's time, so 12 §8 requires the CPU-limited rerun, and that rerun misses 50×. The A02 spike graph reached 361× with simpler voices (F-034). The A05 voices add envelopes with decay, a biquad per filtered voice, PolyBLEP correction and a stereo-linked true-peak limiter.

  Optimizations made while measuring, in order:
  - the limiter skips its interpolator when a coefficient bound proves the true peak is under target: 66× to 84×;
  - the mallet's partials come from a Chebyshev recurrence, and its brightness is a spectral tilt instead of a biquad: 84× to 96×;
  - voices render in runs over a scratch buffer (sine-only Score 109× to 127×).

  Replacing `f64::round` did not help. With no events, the graph runs at 899×, so voices dominate at about 10 ns per voice-sample.
- **Evidence:** A05-T12 PR; `cargo bench -p sonarithm-render --bench render`; `cargo run -p sonarithm-render --example render_speed --release`, plain and under `start /affinity F0000`
- **Evidence level / confidence:** EV-C / Strong (repeated runs, one machine)
- **Affects:** PB-03 (not declared met until a reference-class machine or further optimization, for example SIMD across voices, shows ≥ 50×); 12 §8; P07
- **Papers:** P07

### F-054 · allocation-counter replaces assert_no_alloc for the audio-thread allocation check without a licence change

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** engineering · **Status:** new
- **Statement:** `allocation-counter` 0.8.1 (MIT OR Apache-2.0, allowed by `deny.toml`) counts the allocations and frees made by the current thread inside `allocation_counter::measure`. A05-T10 uses it where the coding standards named `assert_no_alloc` (BSD-1-Clause, rejected by `deny.toml`, F-038). It reports a count instead of aborting, so a test can say which step allocated. Linking the crate installs its counting global allocator, so it is a dev-dependency of `sonarithm-render` and an optional dependency behind the `count-allocations` feature of `sonarithm-audio`, never in a library's normal dependencies or a release application; `cargo xtask check-deps` enforces this (02 §3 rule 5). Control tests show that the check fails on an allocation or a free inside the measured block, and that another thread allocating throughout a measurement is not counted. Results on HW-01 (debug builds):
  - the scripted session (play, seek, loop wraps, bus and master gain, speed, cues, a plan swap, pause and resume, stop, a 300-command flood, an 8,192-frame block) makes no allocation or free in any `Processor::process` call;
  - through the null device, 3 s of commands at 2 ms intervals: no device callback allocated;
  - the `soak` example (release build with the feature): no allocating callback in 769,982 device and null-device callbacks over five 10-minute runs (F-055).

  Limit: the device check measures the engine's part of each callback (`Callback::fill`), not the backend's own code around it (cpal's or the `wasapi` crate's buffer calls).
- **Evidence:** A05-T10 PR; `cargo test -p sonarithm-render --test no_alloc`; `cargo test -p sonarithm-audio --features count-allocations --test no_alloc_device`; `cargo deny check`
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** coding-standards §11.5; testing-strategy §8 and §9.3; ci-cd `no-alloc` job; 02 §3 rule 5; F-038 (resolved without an allow-list change; coordinator approval 2026-10-19)
- **Papers:** none yet

### F-055 · Ten-minute device soaks on HW-01 show no underrun and no allocating callback; HW-01 cannot run 128-frame device buffers, and the render takes at most 14% of a 128-frame block

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** benchmark · **Status:** new
- **Statement:** The `soak` example (release build with `count-allocations`) looped the DS-006 benchmark Score for 10 minutes per configuration on HW-01 at 48 kHz. The master was at −120 dB, so the runs were silent; the render does the same work at any level.

  | Configuration | Callbacks | Underruns | Late callbacks | Allocating callbacks |
  |---|---|---|---|---|
  | WASAPI RAW shared, 480-frame period, 1056-frame buffer | 60,002 | 0 | 0 | 0 |
  | cpal 0.18.2, 128 frames requested, 480 granted | 60,001 | not reported (see below) | 0 | 0 |
  | WASAPI exclusive, the device's minimum period of 144 frames | 200,031 | 0 | 20,396 | 0 |
  | Null device, 128 frames, real-time priority (HW-01) | 224,971 | 194 missed deadlines (0.09%) | — | 0 |
  | Null device, 128 frames, real-time priority (HW-01e) | 224,977 | 153 missed deadlines (0.07%) | — | 0 |

  Per-block render time (`block_times` example, playback graph, 128-frame blocks, 22,500 blocks per run, three runs):
  - HW-01: p50 30 µs, p99 50–52 µs, p99.9 106–120 µs, max 229–382 µs, which is at most 14% of the block's 2,667 µs;
  - HW-01e (four E-cores, normal priority): p50 54 µs, p99 98–103 µs, p99.9 214–291 µs; single blocks reached 1.9, 2.2 and 9.0 ms.

  What the evidence does not show:
  - **No 128-frame device buffer on HW-01.** Shared mode runs at the engine's 480-frame period whatever is requested. Exclusive mode's minimum period is 144 frames, but the Realtek driver queues about 100 ms there (F-051), so that run does not test a 3 ms deadline either.
  - **cpal reports no output underruns on Windows.** cpal 0.18.2's WASAPI host raises `Xrun` only for input streams (the data-discontinuity flag in `process_input`), so its late-callback count is the only evidence for that run. The WASAPI backend counts underruns itself: in shared mode, an empty buffer when the engine asks for more; in exclusive mode, the device clock passing the frames written.
  - **Exclusive mode's late callbacks are the event clustering of F-051.** The driver signals two or three events per hardware period, so some gaps between fills exceed 1.5 periods while the queue never runs dry.
  - **The null device's misses are mostly its timer.** It counts an underrun when a block is ready more than one period after the device asked for it. The same paced loop with no rendering at all, at the same priority, missed 23 of 44,997 deadlines in 120 s (0.05%), and its worst wake-up was 3.4 ms late. The render's worst block on HW-01 is 382 µs. So most misses come from the OS timer's wake-up latency, and a few from the render shrinking the slack. Hardware devices wake their thread by interrupt, not by a sleep timer.
- **Evidence:** A05-T10 PR; `cargo run -p sonarithm-audio --example soak --release --features count-allocations -- --backend <backend> --seconds 600`; `cargo run -p sonarithm-render --example block_times --release`; HW-01e runs pinned with `ProcessorAffinity = 0xF0000`; the timer-only loop was a throwaway program (the null device's loop without `fill`), not committed
- **Evidence level / confidence:** EV-C / Moderate (one run per configuration, one machine, no 128-frame device buffer)
- **Affects:** PB-02 (not declared met at 128 frames: HW-01's driver offers no 128-frame period, F-030; 0 underruns at the periods it offers; TODO(verify) on a device with 128-frame periods); testing-strategy §9.3; 06 §12; the null device's underrun model
- **Papers:** P07

### F-056 · The A05 render engine's golden cases are bit-identical on Linux, Windows and macOS CI runners, including AArch64

- **Date:** 2026-10-21 · **Phase:** A05 · **Type:** benchmark · **Status:** new
- **Statement:** `cargo xtask golden --compare` compared the Windows (CI-WIN, x86-64) and macOS (CI-MAC, AArch64) renders of the six golden cases with the Linux (CI-LNX) renders. Every case was identical: peak and RMS residual at the comparison's −240 dBFS floor, ΔLUFS 0.0000. The cases are the DS-006 benchmark Score (10 s), glide sweeps, every earcon kind, taps and short notes, a limiter stress chord and an ISO 226 compensation sweep. They exercise every A05 voice, the master limiter and seeded noise. This extends F-039 from the A02 spike to the shipped render path: the polynomial sine, plan-time `libm` and basic IEEE 754 arithmetic keep renders portable.
- **Evidence:** [#178](https://github.com/DuhItzAniket/SONARITHM/pull/178), `golden-compare` job of CI run 39509526921 (step summary table); `golden` jobs on ubuntu, windows and macos in the same run
- **Evidence level / confidence:** EV-C / Strong (three OSes, two architectures, six cases)
- **Affects:** PB-06 (met); NFR-03; ADR-0013; OD-08 (hashes are enough as references)
- **Papers:** P07

### F-114 · The replication gallery reproduces every stated pitch, timing and count of four published designs, except short ramps and broadband clicks

- **Date:** 2026-10-21 · **Phase:** A08 · **Type:** engineering · **Status:** new
- **Statement:** From committed specs, SONARITHM renders 49 replication-gallery items whose stated parameters match the published designs: Smith & Walker (2005) as 20 sine tones of 0.5 s at 193.40–1927.63 Hz (the exact image of $10–$84 under $ = 0.0765·f^0.9256), 10 hour clicks and static or dynamic triangle references; Nees & Walker (2008) as 11, 21, 41 and 81 tones of 125 ms at 98.00–1975.53 Hz over 10.125 s; Tucker Brown et al. (2022) as 1000 tones of 0.2 s every 0.01 s at 440–880 Hz over 10.19 s, at SNRs within 3% of nominal; and nine imported Erie specs. Two published details cannot be reproduced with spec v0.1 and the A05 renderer: note ramps are min(40 ms, 25% of the note) and at least 5 ms (SDR-22), so Nees & Walker's 10 ms ramps become 31.25 ms; and there is no noise source, so Smith & Walker's 10 ms broadband clicks become 10 ms square-wave notes at 3 kHz. Their piano is a mallet timbre.
- **Evidence:** A08-T02 PR; `replication/*/manifests.json` (track summaries from the Score) and each group's README; `cargo xtask replication`; `sonarithm_dsp::env::note_ramp`
- **Evidence level / confidence:** EV-C / Strong for the reproduced values; the published values are EV-L (F-110 to F-112)
- **Affects:** CL-04; SDR-16, SDR-22, SDR-29; candidate spec minor version (envelope settings) and a noise or click source in the DSP; P01 §6.1
- **Papers:** P01

### F-115 · Exporting the replication gallery end to end runs at about 130× real time on one thread of the development laptop and 70× on four E-cores

- **Date:** 2026-10-21 · **Phase:** A08 · **Type:** benchmark · **Status:** new
- **Statement:** Compiling and exporting the 49 replication-gallery specs one at a time on one thread (CSV load, encoding, render at 48 kHz in 256-frame blocks, 24-bit WAV, EBU R128 and true-peak metering, manifest) took 3.37 s for 443.8 s of audio on HW-01, a median real-time factor of 131.7× (130.5–132.3 over 5 runs), and 70.9× (70.3–71.3) pinned to four E-cores. Compiles took under 1.5 ms per spec; the export dominates. Dense scores are slower: the Tucker Brown curves (1000 overlapping 0.2 s notes, about 20 sounding at once) export at 89× and 45×, the sparser Smith & Walker and Nees & Walker items at about 205× and 122×. On the code before A05's voice optimizations (#178) the same runs gave 99.5× and 55.4×.
- **Evidence:** A08-T04 PR; `cargo run --release -p xtask -- replication --timings` at `0103de8` (on `2af548f`); raw tables `docs/research-paper/benchmarks/a08-replication-timings/`; [benchmarks log](benchmarks-log.md) "A08 (replication gallery, end to end)"; Windows 11 10.0.26300, AC, Balanced plan, Rust 1.99.0, release, no compiler running
- **Evidence level / confidence:** EV-C / Moderate (one machine; HW-01e is a stand-in for the reference laptop, 12 §8)
- **Affects:** CL-12 (offline speed part), PB-03 context (F-053), P01 §6.2, P07
- **Papers:** P01, P07

### F-116 · The 49 replication-gallery renders are byte-identical on Windows, Linux and macOS (AArch64)

- **Date:** 2026-10-21 · **Phase:** A08 · **Type:** benchmark · **Status:** new
- **Statement:** For every replication-gallery spec, the 24-bit WAV export, its loudness figures and the Score hash with engine versions written as `*` are identical on CI-WIN, CI-LNX and CI-MAC: the xtask test `the_replication_gallery_renders_reproducibly` compares each with the committed record on all three runners. The null-test residual between platforms is therefore −∞ dBFS and the loudness difference zero, stronger than ADR-0013's budget (≤ −90 dBFS, LUFS Δ ≤ 0.1), across 49 specs that cover four published designs, four instruments, overlapping voices and an imported grammar. It extends the A02 spike's result (F-039) from one benchmark Score to the gallery.
- **Evidence:** A08-T02 PR's CI runs (`clippy and test` on ubuntu-latest, windows-latest and macos-latest, for example run 39508775101); `replication/*/manifests.json`. A05's six golden cases are byte-identical across the same runners too (F-056)
- **Evidence level / confidence:** EV-C / Strong (three platforms and two CPU architectures; CI virtual machines, no audio devices involved)
- **Affects:** CL-02, CL-03, ADR-0013, NFR-03; P01 §6.2
- **Papers:** P01, P07

### F-120 · Agent sessions closed 69 of the 78 Stage A phase tasks in about 55 hours, and the nine still open all wait on the owner

- **Date:** 2026-10-21 · **Phase:** Stage B planning · **Type:** engineering · **Status:** new
- **Statement:** Between 2026-10-18 19:09 and 2026-10-21 02:11 UTC (about 55 hours), agent sessions closed 69 of the 78 Stage A phase issues (milestones A01–A08); the baseline roadmap planned Stage A for November 2026 to February 2027. In the same period 89 pull requests were merged (2026-10-18 18:24 to 2026-10-21 02:40 UTC), and 63 of the commits on `main` are dated 2026-10-19. The nine open issues all wait on the owner: A02-T01 and A02-T05 to T08 (manual walkthroughs and measurements on other OSes, the gate sign-off), A05-T13 (the cable test), A07-T08 and A08-T06 (the release go-aheads) and A08-T05 (the paper submission). An issue's close time is when its PR merged, not how long the work took.
- **Evidence:** `gh issue list --state all --search "label:stage/a"` (creation and close times, milestones); `gh pr list --state merged`; `git log --format=%ad --date=short` on `main` at 6bf7dcd; the Stage B planning PR
- **Evidence level / confidence:** EV-C / Moderate (complete counts from GitHub, one project and one owner; close times measure merges, not effort)
- **Affects:** the Stage B re-estimate (roadmap §3, implementation plan §3.1); R-24, R-26, R-27; OD-15, OD-17
- **Papers:** P02 (AI-usage disclosure and development history)

### F-130 · ICU4X 2.3 and V8's Intl print the same digits for 24 shared message cases once Rust rounds half-expand from the shortest round-trip digits

- **Date:** 2026-10-21 · **Phase:** B02 · **Type:** engineering · **Status:** new
- **Statement:** With numbers formatted by ICU4X 2.3 (`icu_decimal`, `fixed_decimal` from shortest round-trip digits, half-expand rounding, at most three fraction digits by default) and plural forms chosen from the value rounded the same way, `fluent-bundle` 0.16 renders all 24 cases of the Rust–TypeScript contract exactly as `@fluent/bundle` 0.19.1 over V8's `Intl` in Node 25.9 does, including `61.005` at two fraction digits (`61.01`), `1.0004` at two digits selecting the singular (`1 degree Celsius`), Indian grouping (`12,34,567`) and the pseudo-locale. Without the pre-rounding, fluent-rs picks plural forms from the unrounded value and the minimum fraction digits only, so `1.0004` would read "1 degrees".
- **Evidence:** `crates/sonarithm-i18n/tests/contract.rs` and `packages/i18n/test/contract.test.ts` on `tests/fixtures/contract.json`, both passing on HW-01 (Windows 11, Rust 1.99.0, Node 25.9.0); the frontend CI job runs the TypeScript half on Node 24; fluent-bundle 0.16.0 `PluralOperands` from `FluentNumber` (source); PR for B02-T08 ([#207](https://github.com/DuhItzAniket/SONARITHM/issues/207))
- **Evidence level / confidence:** EV-C / Moderate (24 cases in four English locales; CLDR versions in ICU4X and V8 can drift, which the contract test would catch)
- **Affects:** NFR-09, A11Y-12, DL-P12 (one value, one formatter); 12 §5
- **Papers:** none yet
