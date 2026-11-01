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
- **Affects:** B04-T02: the `gpu` setting (`highPerformance` adds this switch to wry's default WebView2 arguments; the host retries without it if WebView2 refuses, 09 §7, 12 §3); 09 §1
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

### F-046 · NVDA 2026.3 speaks the managed live regions as designed but nothing from the host's UI Automation notifications

- **Date:** 2026-10-29 · **Phase:** A02 · **Type:** accessibility · **Status:** new
- **Statement:** An agent ran the A02 NVDA walkthrough (steps 1–19) on HW-01 with NVDA 2026.3 and WebView2 156. It read the Speech Viewer, and nobody listened. What it found:
  - **Live-region route.** Each announcement was spoken once. Ten updates sent within 1 ms gave only "Rapid update 10 of 10", and ten chart steps over 314 ms gave only the last point (supersede, A11Y-17). The assertive alert was spoken.
  - **Native route.** `UiaRaiseNotificationEvent` on the top-level window's host provider returned success for every call, but NVDA spoke none of them. With both routes on, the text was spoken once.
  - **Tree.** React Aria's Tree is announced as a "table", with "not selected" on every row. The doubled row name of F-045 appears in browse mode only.
  - **Chart explorer.** The `role="application"` explorer gives focus mode and stale-point dropping, but NVDA does not speak the role.

  One likely cause for the silent native route: NVDA drops notifications whose sender is not a native UIA element. TODO(verify), for example with NVDA's debug log or a different sender element.
- **Evidence:** `spikes/tauri-webview/results/a11y-windows-nvda.json`; [gate report, Spike 5](../sdlc/reviews/A02-prototype-gate.md#nvda-agent-driven-speech-viewer-transcript)
- **Evidence level / confidence:** EV-C / Moderate (one screen reader, one machine; a Speech Viewer transcript with no timing, no listener)
- **Affects:** OD-05; ADR-0003; A11Y-17; 08 §4–5; B02 tree component; B03 announcer
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

### F-131 · The generated OKLCH tokens meet every design-language contrast pair in four themes; two light signals stay below 4.5:1 for text

- **Date:** 2026-10-25 · **Phase:** B02 · **Type:** accessibility · **Status:** new
- **Statement:** All 156 pairs of design-language §5.3 pass WCAG 2.2: 39 pairs in each of the dark, light, high-contrast dark and high-contrast light themes, computed from the 8-bit sRGB colours a browser renders for the emitted OKLCH values. The lowest text ratio is 5.42:1 (light `text-3` on `surface-2`). The lowest non-text ratio is 3.03:1 (dark `border-strong` on `surface-3`, at the limit, as the baseline arithmetic predicted). No high-contrast pair is lower than its standard-theme pair. Seven recipe colours (light accent and focus, high-contrast dark warning and error, high-contrast light accent, warning and ok) were outside sRGB and were gamut-mapped with the CSS Color 4 algorithm, losing up to 0.022 chroma (high-contrast light accent, 0.095 to 0.0731). Light `signal-warning` (3.55–4.44:1) and light `signal-ok` on `surface-2` and `surface-3` (3.89–4.31:1) pass 3:1 as marks but not 4.5:1 as text; the baseline predicted only the warning case.
- **Evidence:** `packages/ui/src/generated/tokens.json` (the pair list with ratios) and `packages/ui/test/tokens.test.ts`, which recomputes every ratio from the emitted CSS; the OKLab conversion matches the published OKLCH of the sRGB primaries to 1e-4 (`packages/ui/test/color.test.ts`); Node 25.9 on HW-01 and Node 24 in CI; PR for B02-T01 ([#200](https://github.com/DuhItzAniket/SONARITHM/issues/200))
- **Evidence level / confidence:** EV-C / Strong for the ratios (deterministic arithmetic on the rendered colours); not a perceptual validation, which is B02-T09's design review and the owner's forced-colours checks per webview
- **Affects:** A11Y-25, FR-117, DL-X07; design-language §5.2–§5.3 (signal text uses `text-1` with an icon for warning and success in the light theme)
- **Papers:** none yet

### F-132 · IBM Plex needs no tnum feature: its default figures are tabular, the same width in every bundled weight, and every weight has a slashed zero

- **Date:** 2026-10-25 · **Phase:** B02 · **Type:** engineering · **Status:** new
- **Statement:** IBM Plex Sans 3.005 and IBM Plex Mono 2.005 (WOFF2, weights 400, 500 and 600) have no `tnum` or `pnum` feature in GSUB. Their default digits are already tabular: each of 0–9 is 600 font units wide in all six files, so digits keep the same width across weights too. Every file has `zero` (slashed zero; Sans's default zero is plain, Mono's is dotted) and `lnum`. The `onum` feature also exists, so oldstyle figures stay out of the CSS. Rendered in Chromium with the bundled files at 18 px, `1111.11`, `8888.88` and `0000.00` have identical widths with default and with tabular figures in every weight. The system sans-serif fallback measured 59.7 and 65.1 px for the first two, so bundling is what keeps readouts from jittering.
- **Evidence:** `packages/ui/test/fonts.test.ts` (`dl_x10 every weight has tabular figures and a slashed zero`, reading GSUB, cmap and hmtx from each WOFF2 with `packages/ui/tokens/woff2.ts`); the font test page `packages/ui/font-test/index.html` measured in the Claude desktop app's Chromium pane on HW-01; PR for B02-T02 ([#201](https://github.com/DuhItzAniket/SONARITHM/issues/201))
- **Evidence level / confidence:** EV-C / Strong (direct reading of the font tables; one rendering engine checked, so WebKit renders are the owner's check with the forced-colours item)
- **Affects:** DL-P04, DL-X10, A11Y-25; design-language §7.1 and §14 (closes the Plex `tnum` and `zero` gap of research notes 05 §8)
- **Papers:** none yet

### F-133 · IBM's Plex npm packages send install telemetry, and pnpm 12 fails the install when their script is blocked

- **Date:** 2026-10-25 · **Phase:** B02 · **Type:** engineering · **Status:** new
- **Statement:** `@ibm/plex-sans` 1.1.0 and `@ibm/plex-mono` 2.5.0 depend on `@ibm/telemetry-js` and run `ibmtelemetry --config=telemetry.yml` as a `postinstall` script, which reports to `https://www-api.ibm.com/ibm-telemetry/v1/metrics`. pnpm 12.14 blocked the script, but `pnpm install` then exited with `ERR_PNPM_IGNORED_BUILDS`, which would fail CI. SONARITHM therefore bundles the six WOFF2 files, unmodified, with their upstream SHA-256 recorded and tested, and does not depend on the packages. Updates use `npm pack`, which runs no scripts.
- **Evidence:** the packages' `package.json` (`scripts.postinstall`, `dependencies`) and `telemetry.yml` as installed on 2026-10-25; the pnpm 12.14.0 install output; `packages/ui/fonts/README.md`
- **Evidence level / confidence:** EV-C / Strong (observed directly)
- **Affects:** ADR-0012 (no telemetry without consent), AGENTS rule 9; the supply-chain policy's npm checks (an install-script review belongs next to the licence check)
- **Papers:** none yet

### F-134 · Exact per-run voice loops render the benchmark Score at 60× real time on four E-cores, so PB-03 is met with bit-identical output

- **Date:** 2026-10-25 · **Phase:** B (A05 follow-up, #185) · **Type:** benchmark · **Status:** new
- **Statement:** Timing each DSP primitive on HW-01e put the DS-006 render's cost in three places:
  - the polynomial sine (`sine_turns`, about 9 ns per call on an E-core), which the mallet calls twice per sample and every attack and release ramp once;
  - the biquad's chain of dependent operations (6.7 ns per sample);
  - per-sample dispatch in the oscillator and the envelope.

  Three changes compute the same IEEE operations per sample, so the output is unchanged:
  - `sine_turns_unit`: the same polynomial with a one-comparison reduction for turns in [0, 1.5), the range of oscillator phases and envelope ramps;
  - the oscillator and the mallet compute a chunk of phases first, then the waveform for all of them, which the compiler can vectorize; the envelope works a stage at a time;
  - filtered voices generate and filter in one loop, so the oscillator's arithmetic overlaps the filter's chain.

  The 60 s benchmark render hashes to the A05 value at 256- and 61-frame blocks. Each A/B alternated the A05 and new binaries on the same machine, which another build session was using at the time:
  - `render_speed` on HW-01e: 40–41× → 59–64× (three rounds);
  - criterion `pb_03/render_benchmark_score_60s_256` on HW-01e: 1.51–1.55 s → 0.99 s (39–40× → 60×);
  - HW-01: 72× → 107× (`render_speed`) and 974 → 689 ms (criterion).

  Per-primitive costs on an E-core: mallet partials 22.7 → 14.9 ns per sample, envelope ramp 8.0 → 4.7 ns, square oscillator with biquad and gain 11.9 → 7.1 ns. No SIMD intrinsics or `unsafe` were needed, so the render graph's design is unchanged.
- **Evidence:** `crates/sonarithm-render/tests/render_identity.rs` (`the_benchmark_render_is_bit_identical_to_the_a05_graph`: FNV-1a 4290137234021000538 over the 60 s render); the bit-for-bit unit tests `sine_turns_unit_matches_sine_turns_bit_for_bit`, `fills_match_the_per_sample_oscillator_bit_for_bit`, `apply_matches_next_level_bit_for_bit` and `the_mallet_fill_matches_its_per_sample_path_bit_for_bit`; `render_speed` and the criterion bench under `start /affinity F0000`; PR for [#185](https://github.com/DuhItzAniket/SONARITHM/issues/185)
- **Evidence level / confidence:** EV-C / Moderate (one machine, under a background load that slowed both sides of each A/B: the A05 binary measured 40–41× here against 43–46× in A05's runs)
- **Affects:** PB-03 (met on HW-01e; the A05 waiver is closed), 12 §8, F-053, ADR-0013 (unchanged output), NFR-01
- **Papers:** P07

### F-140 · A05's `Player` cannot move between threads, so the app core owns the thread that drives it

- **Date:** 2026-10-21 · **Phase:** B01 · **Type:** engineering · **Status:** new
- **Statement:** `sonarithm_audio::Stream` keeps the device stream as a `Box<dyn Any>` without `Send`, because cpal streams are not `Send` on every OS, so `Player<B>` is not `Send` either. A host cannot keep the player inside shared state (Tauri's managed state needs `Send + Sync`). `sonarithm-app` therefore creates the player on its own control thread, which ticks it every 10 ms and answers transport commands over a channel; `App` itself is `Send` and holds only the channel. The same thread follows the playhead, stops at the end of the Score, and turns device changes into events, so a host never polls. ADR-0017 §1 records the design.
- **Evidence:** `crates/sonarithm-audio/src/backend.rs` (`Stream::_keep`); `crates/sonarithm-app/src/transport.rs`; the B01-T01 scenarios on the null device (`device_loss_pauses_at_the_same_position`, `playback_stops_at_the_end_and_reports_it`)
- **Evidence level / confidence:** EV-C / Strong (a compile-time property, checked by the scenarios)
- **Affects:** ADR-0017 §1; 09 §1; B01-T03 (the Tauri host keeps `App` behind one lock)
- **Papers:** none yet

### F-141 · Reading an embedded Parquet member in place cuts the peak memory of opening it by 1.6 times the member's size

- **Date:** 2026-10-21 · **Phase:** B05 · **Type:** engineering · **Status:** new
- **Statement:** RFC-0002's condition B asks that a project's embedded data not be held twice in memory. Reading a 200,000-row, two-column Parquet member (2.5 MB, Snappy, values that do not compress) in place from the stored ZIP member peaks at 9.0 MB of heap; reading the member into a buffer and decoding that peaks at 12.9 MB, 3.9 MB more, because the buffered path holds the member and a copy of it while decoding. The in-place path reads only the footer and each page header with positional reads (the structural check of #146 now streams page headers instead of loading each column chunk), so the default 2 GiB data limit stands.
- **Evidence:** `crates/sonarithm-app/tests/project_memory.rs` (`rfc_0002_condition_b_embedded_data_is_read_in_place`, `allocation_counter::measure` peaks) on HW-01 (Windows 11, Rust 1.99.0, debug build); `sonarithm_data::read_parquet_at`, `columnar::check_pages`
- **Evidence level / confidence:** EV-C / Moderate (one data shape on one machine; the saving grows with the member's size, the decoded table dominates either way)
- **Affects:** RFC-0002 §6, ADR-0018; NFR-11; the data limits of A04 (`Limits::DEFAULT`)
- **Papers:** none yet

### F-142 · Registering AUDAXIS changes every Score's header, so the app's Scores differ from the CLI's until AUDAXIS is a default engine

- **Date:** 2026-10-21 · **Phase:** B03 · **Type:** engineering · **Status:** new
- **Statement:** The navigation tree that announcements describe exists only when AUDAXIS is registered with the compiler. The facade registers only NUMISONA by default, so B03-T07 adds an opt-in (`Builder::with_navigation`) that the app core uses. With AUDAXIS registered, a spec without `context` still compiles to a different Score: the header lists `audaxis` among the engines and `spatial` among the shapers (stereo placement, B03-T11), so the Score hash changes, while the data events stay the same. The same spec therefore hashes differently in the desktop app and in the CLI or Python until B03 makes AUDAXIS a default engine of the facade.
- **Evidence:** `crates/sonarithm/tests/facade.rs` (`navigation_builds_the_tree_beside_the_score`); the two Score headers compared on HW-01
- **Evidence level / confidence:** EV-C / Strong (deterministic)
- **Affects:** ADR-0013 (provenance: a render manifest names the Score hash); 05 §8; B03 (making AUDAXIS a default engine), A07 parity between the CLI, Python and the app
- **Papers:** none yet

### F-143 · On the monitoring bed, event salience shows as a spectral-centroid rise of 39 %, 51 % and 97 % at constant level

- **Date:** 2026-10-25 · **Phase:** B06 · **Type:** engineering · **Status:** new
- **Statement:** The B06-T04 bed signals events by brightness, not level (sound-design §10.2, SDR-35). In the golden render of `monitoring-soundscape` (a 10 s triangle bed at A3 with events of low, medium and high salience at 1.5 s, 4 s and 7 s), the per-second spectral centroid is 272 Hz in the first second, rises to 378 Hz, 412 Hz and 538 Hz in the seconds of the three events (39 %, 51 % and 97 % above the first second), and falls back to 284–320 Hz between them, while the bed's gain stays at 0 dB throughout. Whether these steps are noticeable, and calm enough, in a dual task is H-09's question (EXP-008), not this measurement's.
- **Evidence:** `tests/golden/monitoring-soundscape/case.json` (reference rendered on HW-01, identical at block sizes 64, 256 and 1024); `crates/engines/sonarithm-sonotrace/src/monitor/tests.rs` (`sdr_35_salience_changes_brightness_not_level`)
- **Evidence level / confidence:** EV-C / Strong for the acoustic measurement; none yet for perception
- **Affects:** SDR-35, H-09, sound-design §10, HP-11 (listening check)
- **Papers:** none yet

### F-144 · A 1 kHz stream played live for an hour keeps memory flat at about 26 MiB and loses no rows; a full window compiles in 8.2 ms

- **Date:** 2026-10-25 · **Phase:** B06 · **Type:** engineering · **Status:** new
- **Statement:** The B06-T07 soak plays a synthetic 1 kHz stream live through the app core for an hour (3,600 s): a `tail` stream of a file written in 10 ms bursts, a monitoring spec with a glide over the newest 10 s, compiled about once a second and swapped in while playing on the timer-driven null device. Resident memory rises while the stream fills its 100,000 rows and then stays flat: peak 26.1 MiB from 900 s to 1,800 s and 26.2 MiB after (PB-11's bound: 10 % + 8 MiB). 3,601,204 rows were written and all were received, none invalid. After the warm-up the timeline was at most 0.56 s behind the data and the playhead at most 2.04 s behind the timeline, playing throughout. Compiling a full 10 s window (10,000 rows) takes 8.2 ms median and 10.0 ms at most, so whole-window encoding keeps up with room to spare (B06-T03's open question). The null device counted 148 late blocks over the hour; they are timer misses of a software device under other load (F-055 saw 153–194 per 10 min at 128 frames), not PB-11's device underruns, which need a real device (owner step).
- **Evidence:** `cargo run -p sonarithm-app --example stream_soak --release -- --seconds 3600 --strict` on HW-01 (Windows 11, Rust 1.99.0), release build; three full `cargo xtask ci` runs of other branches ran during the hour, and the late blocks cluster in them (up to 54 per 5 minutes, against 0 to 4 when the machine was otherwise idle); the samples every 10 s and the verdict in [benchmarks/b06-stream-soak](benchmarks/b06-stream-soak/); the benchmarks log row "B06 (1-hour stream)"
- **Evidence level / confidence:** EV-C / Moderate (one run on one machine, the null device; the device run is pending)
- **Affects:** PB-11, NFR-01, FR-005; 07 SONOTRACE (no `encode_incremental`); B06-T07
- **Papers:** none yet

### F-150 · Stage B's Rust lane closed 21 of its 23 tasks on two calendar days, about four to five tasks per session-day

- **Date:** 2026-10-26 · **Phase:** Stage C planning · **Type:** engineering · **Status:** new
- **Statement:** After the Stage B planning PR merged on 2026-10-21, agent sessions closed 21 of the 23 Lane R issues: 13 between 03:59 and 07:19 UTC on 2026-10-21 and 8 between 15:07 and 20:31 UTC on 2026-10-25, with two or three sessions working at once (the worklogs of 2026-10-21 and 2026-10-25: AUDAXIS with the catalogues, the app core with SONOTRACE, then the resumed Rust lane beside design tokens and a render-speed session). The two open Lane R tasks are B03-T08 (profiles) and B06-T07, a one-hour stream soak that waits until the shared laptop is free. None of the 37 Lane U tasks has started, because Gate U (HP-01, HP-04) is still closed. As with F-120, a close time is when the task's PR merged, not how long the work took.
- **Evidence:** `gh issue list --state closed --label stage/b` (close times; #185 excluded as a carry-over issue); `git log` on `main` at 35fa8d4; the worklogs of those days; the Stage C planning PR
- **Evidence level / confidence:** EV-C / Moderate (complete counts from GitHub; merges, not effort; sessions overlapped)
- **Affects:** the Stage C re-estimate (roadmap §4, implementation plan §4.1); R-24, R-27, R-30; OD-20
- **Papers:** P02 (development history)

### F-160 · No pure-Rust PDF parser on crates.io fitted an engine crate's constraints, so PLOTSONA reads PDF objects itself

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** engineering · **Status:** new
- **Statement:** Of the two maintained pure-Rust PDF parsers checked for PLOTSONA's vector-PDF tier on 2026-10-26, `lopdf` 0.46.0 (MIT, MSRV 1.88) has non-optional `aes`, `cbc`, `md-5`, `rand`, `getrandom` and `log` dependencies, mostly for decryption, and `getrandom` needs an extra feature on `wasm32-unknown-unknown`; `hayro-syntax` 0.8.0 (`Apache-2.0 OR MIT`, few dependencies) declares MSRV 1.92, above the core MSRV of 1.88. Neither fits an engine crate that must build for wasm32 at MSRV 1.88 and refuses encrypted files anyway, so C01-T08 reads PDF objects, object streams and `FlateDecode` itself, on `flate2`, with its own budgets and fuzz target.
- **Evidence:** crates.io API (`/api/v1/crates/lopdf/0.46.0/dependencies`, `/api/v1/crates/hayro-syntax`), 2026-10-26; the C01-T02 table in [supply-chain-and-licensing.md](../engineering/supply-chain-and-licensing.md#c01-plotsonas-parsers-and-corpus-tools-c01-t02-checked-2026-10-26)
- **Evidence level / confidence:** EV-C / Strong (declared metadata, checked directly)
- **Affects:** C01-T08, R-29 (untrusted chart parsers); revisit when the core MSRV reaches 1.92 (hayro-syntax would then replace the own reader)
- **Papers:** none yet

### F-161 · Chart writers cache categories with a different type from their cells, so a chart cache is not a reliable type signal

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** engineering · **Status:** new
- **Statement:** In charts written by XlsxWriter 3.3.0, text categories that look numeric (the strings `"0"` to `"19"`) are cached as numbers (`c:numCache`, format `General`) while the cells hold text, and date categories on a date axis (`c:dateAx`) are cached as serial numbers with format `General` rather than a date format. PLOTSONA therefore compares a cache with its workbook by value, not by storage type, and treats numeric categories on a date axis as dates; with both rules, all 22 Office charts of the C01 corpus extract with zero error and no stale-cache warning.
- **Evidence:** `crates/engines/sonarithm-plotsona/tests/corpus/office/xlsx-area.xlsx` and `xlsx-date-line.xlsx` (`xl/charts/chart1.xml` against `xl/worksheets/sheet1.xml`); `office::tests::corpus_charts_extract_with_zero_error`; the C01-T05 PR
- **Evidence level / confidence:** EV-C / Strong for these files; other writers (Excel, LibreOffice) not yet checked, TODO(verify) with real Office files in C01-T11
- **Affects:** C01-T05, C01-T11, DS-007
- **Papers:** none yet

### F-162 · plotly.py 6 writes NumPy data as base64 typed arrays and ships prototype traces in every figure's template

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** engineering · **Status:** new
- **Statement:** `fig.to_json()` in plotly 6.7.0 encodes NumPy arrays as `{"dtype": "f8", "bdata": "<base64>"}` (little-endian; `i4`, `f4` and the other widths likewise) instead of JSON arrays, and the default template puts prototype traces under `layout.template.data` (about 10 KB per figure). A static reader that expects JSON arrays reads no values from such figures, and one that scans for any `data` array reads the prototypes as series. PLOTSONA decodes typed arrays and reads only the figure's top-level `data`; all 7 Plotly charts of the C01 corpus, one with typed arrays, extract with zero error.
- **Evidence:** plotly.py 6.7.0 output (`go.Figure([go.Scatter(x=np.arange(4), …)]).to_json()`, 2026-10-26); `crates/engines/sonarithm-plotsona/tests/corpus/chartjson/plotly-scatter.json`; `chartjson::tests::corpus_charts_extract_with_zero_error`, `plotly_templates_are_never_read_as_traces`; the C01-T06 PR
- **Evidence level / confidence:** EV-C / Strong for plotly.py 6.7.0; other Plotly writers (R, Julia, plotly.js's own export) not checked, TODO(verify)
- **Affects:** C01-T06, C01-T11, DS-007, notes/06 §6 (errata)
- **Papers:** none yet

### F-163 · Calibrated SVG extraction reads Vega-Lite, Plotly and D3 renders within 0.15 % of the largest value, limited by half-pixel tick offsets

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** benchmark · **Status:** new
- **Statement:** On 12 SVG renders of DS-001 to DS-004 (line, bar, scatter, area and pie charts; Vega-Lite through vl-convert 1.9.0, Plotly through Kaleido 1.5.0, D3 7.9.0 in jsdom), PLOTSONA's SVG tier recovered every series with a worst absolute error of 0.147 % of the chart's largest value (Vega-Lite area); Plotly renders were within 0.002 %, bars and pies within 0.0003 %. The Vega-Lite and D3 errors, about 0.1 %, are a constant half-pixel shift: both draw tick lines offset by 0.5 units for crisp edges (d3-axis's offset when the device pixel ratio is 1), and labels are snapped to those lines. All are well inside the C01 target of 0.5 %.
- **Evidence:** `svg::tests::corpus_charts_extract_within_half_a_percent` (prints per-chart errors); `crates/engines/sonarithm-plotsona/tests/corpus/svg/`; the C01-T07 PR
- **Evidence level / confidence:** EV-C / Strong for these renders; renders at other sizes, other libraries and hand-made SVGs not measured; benchmarks-log row pending with C01-T11's harness
- **Affects:** C01-T07, C01-T11, DS-007, 10 §3 (tier 2 "near-exact")
- **Papers:** none yet

### F-164 · Vega's SVG export labels each mark with its data values for assistive technology

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** accessibility · **Status:** new
- **Statement:** SVGs exported by Vega 6 through vl-convert 1.9.0 carry an `aria-label` on every data mark, such as `"month: Jan; rain: 99.2; region: north"`, with the values as Vega formats them, beside `role="graphics-symbol"`. A chart file can therefore hold its values twice: in geometry (tier 2) and in text written for screen readers. PLOTSONA's SVG tier reads only the geometry; reading such labels would be a separate, exact-but-formatted path (values rounded by the format, field names from the spec), which needs a decision before use.
- **Evidence:** `crates/engines/sonarithm-plotsona/tests/corpus/svg/vl-line.svg` and the other Vega-Lite SVGs
- **Evidence level / confidence:** EV-C / Strong for vl-convert 1.9.0; other Vega renderers not checked
- **Affects:** C01-T07; a possible later extractor (open question, not filed as an OD yet); A11Y research on chart exports
- **Papers:** none yet

### F-165 · Chart PDF exporters split words into separate text showings, so labels must be rejoined before they can be read

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** engineering · **Status:** new
- **Statement:** In the 9 corpus PDFs, vl-convert 1.9.0 (through svg2pdf) writes every glyph of a tick label in its own `BT … ET` block with its own text matrix, and Kaleido 1.5.0 (Chrome's Skia) splits labels at kerning pairs (`<0029> Tj 6.6 0 Td <00480045> Tj` for "Feb"). Read showing by showing, "apple" becomes five one-letter labels and "Feb" becomes "F" and "eb", so no axis or legend is found. Joining a showing to the previous one when it starts within 0.6 em after it on the same baseline (0.2 em across) restores every label, and with it all 9 charts, within 0.147 % of the largest value, the same accuracy as the SVG renders of the same charts (F-163).
- **Evidence:** `crates/engines/sonarithm-plotsona/tests/corpus/pdf/`; `pdf::tests::corpus_charts_extract_within_half_a_percent`; the C01-T08 PR
- **Evidence level / confidence:** EV-C / Strong for these two exporters; other PDF producers (matplotlib, Office, LaTeX) not yet checked, TODO(verify) in C01-T11
- **Affects:** C01-T08, C01-T11, C02 (OCR-free tick reading in vector PDFs)
- **Papers:** none yet

### F-168 · A chart page's `Date.UTC` with an out-of-range argument overflowed the literal reader

- **Date:** 2026-10-27 · **Phase:** C01 · **Type:** engineering (security) · **Status:** new
- **Statement:** The nightly `html_page` fuzz target (run 40289971584) found a 645-byte page whose Highcharts options call `Date.UTC(1013, -5666…6)` with a 77-digit month. PLOTSONA's script-free literal reader computed the time in `i64` and panicked on overflow (a debug-build abort; release builds would have wrapped to a wrong date). JavaScript itself gives `NaN` for any time beyond ±8.64 × 10¹⁵ ms (its `TimeClip`, about ±275,000 years). The reader now rejects arguments and carried years past that range, sums in `i128`, and gives `null` (a gap) for an out-of-range time, as a chart drawn by JavaScript would show nothing there. The crash input is committed as the fuzz seed `seed-regression-date-utc-overflow.html` and replayed by a unit test.
- **Evidence:** `chartjson::literal::tests::dates_outside_javascripts_range_are_gaps_not_overflows`, `chartjson::tests::fuzz_regressions_read_without_panicking`; the fix PR
- **Evidence level / confidence:** EV-C / Strong for this path; other arithmetic on parsed numbers in the extractors is bounded by the point and work budgets, and the nightly fuzz targets keep running on it
- **Affects:** C01-T06, NFR-05 (untrusted input), 12 §6
- **Papers:** none yet

### F-169 · Fuzzing found two decoders that trusted what the input declared: SVG arc sizes and Parquet page-header wire types

- **Date:** 2026-10-27, extended 2026-10-29 · **Phase:** C01 · **Type:** engineering (security) · **Status:** new
- **Statement:** Two cases, each fixed by checking the input before a third-party decoder sees it:
  - **SVG arcs.** The `svg_chart` fuzz target found Plotly pie slices whose arcs were enormous: once a radius of about 2.4 × 10⁴⁸ (2.4 GB allocated, over libFuzzer's 2,048 MB limit), and once a normal radius with an end point about 10³⁵ away, which SVG enlarges the radius to reach (over the 10 s timeout). `svgtypes`' simplifying path parser converts an arc to cubic Béziers eagerly, and the segment count grows with the arc's size. PLOTSONA now scans each path first, tracking the current point through relative commands, and refuses an arc whose radius or either end lies beyond 10⁹ units, or is not a number (`SNR-ENG-307`).
  - **Parquet page headers.** The `columnar_read` target found a 2 KB file that kept parquet busy for about 1.7 s in a release build (over the timeout with sanitizers). One data page header sent field 6 (`index_page_header`, a struct) with a boolean wire type. The data layer's Thrift walk (#146, F-071) skipped it as a one-byte boolean, but parquet reads a known field by its ID whatever the wire type, so it parsed the following bytes as a struct and ran into a huge collection. The walk now checks the declared wire type of every field parquet.thrift gives a page header and the structs inside it (data page v1 and v2, dictionary page, statistics), and refuses a mismatch as malformed, in 0.2 ms.
  - Each input is a committed `seed-regression-*` fuzz seed, replayed by a test (`chartjson::tests::fuzz_regressions_read_without_panicking` in PLOTSONA, `fuzz_regressions` in `sonarithm-data`).
- **Evidence:** the tests above; `source::thrift::tests::page_header_fields_must_carry_their_declared_wire_types`; nightly run 40845510574; the fix PR
- **Evidence level / confidence:** EV-C / Strong for these paths. The Parquet footer is walked without a schema, so a wire-type mismatch there would still reach parquet; no fuzz input has shown one yet, TODO(verify) with a schema for `FileMetaData`. Other `svgtypes` conversions (smooth curves, quadratics) emit one segment each and are bounded by the work budget
- **Affects:** C01-T07, A04 (Parquet source), NFR-05 (untrusted input), 12 §6; possible upstream reports to `svgtypes` (a lazy or bounded arc conversion) and `parquet` (type-checked field reads)
- **Papers:** none yet

### F-170 · Tauri 2.12 cannot turn off WebView2's browser accelerator keys, although wry can

- **Date:** 2026-10-29 · **Phase:** B01 · **Type:** engineering (accessibility) · **Status:** new
- **Statement:** WebView2 enables browser accelerator keys by default: F5 and Ctrl+R reload, Ctrl+F finds, F7 offers caret browsing, Alt+Left goes back, and F12 opens developer tools when they are enabled. Each of these would take the UI away from a keyboard or screen-reader user (keyboard §12.3). wry 0.57 has `with_browser_accelerator_keys(false)`, but Tauri 2.12.4's `WebviewWindowBuilder` and its runtime do not pass the option through, so the host sets `ICoreWebView2Settings3::AreBrowserAcceleratorKeysEnabled` itself through `with_webview` and reads it back. That needs `unsafe` COM calls ([ADR-0025](../decisions/0025-unsafe-register-desktop-host.md)). Tauri's `zoom_hotkeys_enabled` is already off by default. A page-level guard that calls `preventDefault` covers WKWebView and WebKitGTK too, but on its own it cannot be trusted for keys WebView2 handles before the page sees them.
- **Evidence:** source read of `tauri` 2.12.4, `tauri-runtime-wry` 2.12.2 and `wry` 0.57.0 (no `browser_accelerator_keys` outside wry); the `desktop` CI job's smoke report (`browserAccelerators`)
- **Evidence level / confidence:** EV-C / Strong for the API gap; which keys a page can block in WebView2 is not measured, TODO(verify) during the NVDA walkthrough
- **Affects:** B01-T03, KB-04, A11Y-05, keyboard §12.3, ADR-0025
### F-166 · MAIDR's data schema states no version and leaves stacked bars undocumented

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** engineering · **Status:** new
- **Statement:** MAIDR's published data schema (`docs/SCHEMA.md` in `xability/maidr`, main branch, read 2026-10-26) gives the shape of `data` for bar, line, step, point, box, heat, hist, dodged_bar, pie, smooth and candlestick layers but states no schema version; the package was the TypeScript rewrite (4.x), and the README's example writes axis labels as strings (`axes: { x: 'Quarter' }`) where the schema writes objects (`{ x: { label } }`). The `stacked_bar` and `stacked_normalized_bar` layers are listed as stable without a data example; their selector grid suggests the `dodged_bar` shape (one list per series), which the C01-T10 importer assumes. A MAIDR document therefore cannot say which schema it follows, so an importer must accept both label forms and flag the version it was written against.
- **Evidence:** `sonarithm_plotsona::import::maidr` module docs; `crates/engines/sonarithm-plotsona/tests/import/maidr/`; the C01-T10 PR
- **Evidence level / confidence:** EV-C / Moderate: one reading of the documentation; no MAIDR-produced file was checked, TODO(verify) against files MAIDR's Python binding writes
- **Affects:** C01-T10, FR-027, 04 §9
- **Papers:** none yet

### F-167 · All 67 corpus charts extract with the right class, within 0.15 % and an RMS-F1 of at least 0.998

- **Date:** 2026-10-26 · **Phase:** C01 · **Type:** benchmark · **Status:** new
- **Statement:** The C01-T11 harness scores DS-007, 67 charts and 1,778 values: 22 Office (XLSX, PPTX, DOCX), 24 chart-JSON (Vega-Lite, Vega, Plotly, Highcharts, three in HTML pages), 12 SVG and 9 vector PDF. Every chart has the right provenance class (`exact` for Office and chart JSON, `vector` for SVG and PDF). The exact tiers have zero error and an RMS-F1 of 1. The vector tiers' worst error is 0.147 % of the chart's largest value (Vega-Lite area, in SVG and in PDF), the lowest RMS-F1 0.9986 and the worst resampled curve error 0.206 %. All are inside the C01 bar of 0.5 % (notes/06 §8). The vector error is the half-pixel tick offset of F-163; the curve error is larger because interpolation between points carries it over the whole span.
- **Evidence:** `crates/engines/sonarithm-plotsona/src/accuracy.rs` and its snapshot (the per-chart table); the CI job `chart-accuracy` uploads the same report; the C01-T11 PR
- **Evidence level / confidence:** EV-C / Strong for this corpus. The corpus is generated by the project from four synthetic datasets, so it measures the extractors on well-formed files from seven writers; charts from other writers, real-world files and hand-edited charts are not measured. RMS-F1 uses exact key matching (stricter than `DePlot`'s edit distance), TODO(verify) against its reference implementation before comparing numbers across papers
- **Affects:** C01 exit, FR-006, DS-007, F-163, 10 §3; the raster tier's baseline (C02)
- **Papers:** none yet

### F-171 · ECMA-402 has no unit for hertz or decibels, so React Aria's unit formatting cannot carry SONARITHM's units

- **Date:** 2026-10-30 · **Phase:** B02 · **Type:** engineering (accessibility) · **Status:** new
- **Statement:** `Intl.NumberFormat` with `{ style: "unit", unit: "hertz" }` throws `RangeError: Invalid unit argument` in Chromium (Playwright 1.64). ECMA-402's sanctioned unit list has seconds, percent, degrees Celsius and lengths, but not hertz, decibels, cents or semitones, which are SONARITHM's most common units. React Aria Components formats a slider's and a number field's value, including `aria-valuetext`, only through `formatOptions`. So its `Slider` and `NumberField` cannot say "440 Hz" or "−12.0 dB", and DL-P12 (one shared formatter for display and speech) cannot hold with them. B02-T05 builds the Slider from React Aria's hooks (`useSliderState` with a formatter backed by the `unit-*` Fluent messages). The NumberField stays on React Aria Components for Intl units for now, and React Aria spells seconds "secs" where `Intl` writes "sec" (#389).
- **Evidence:** `packages/ui/src/components/Slider.stories.tsx` (`aria-valuetext` "440 Hz" from `unit-hertz`); the failing run before the change (RangeError); ECMA-402 sanctioned units (TODO(verify): cite the edition's table)
- **Evidence level / confidence:** EV-C / Strong
- **Affects:** B02-T05, DL-P12, A11Y-12, #389
- **Papers:** none yet

### F-172 · React Aria's Tree is always a tree grid, and its Button drops `aria-hidden`

- **Date:** 2026-10-30 · **Phase:** B02 · **Type:** accessibility · **Status:** new
- **Statement:** React Aria Components 1.22's `useTree` sets `role="treegrid"` with no option for `role="tree"`, which is why NVDA reads the dataset tree as a table (#383, F-045). The doubled row name came from the expand button, labelled "Expand" plus the row's ID, inside the row's only cell. That button is outside the Tab order (`excludeFromTabOrder`), and keys expand the row itself. React Aria's `Button` passes only known DOM props, so `aria-hidden` set on it is dropped; a wrapping `<span aria-hidden="true">` does hide it. axe accepts this because the hidden button is not tabbable. After the change, each row's cell is named by its title alone (story test).
- **Evidence:** `packages/ui/src/components/Tree.stories.tsx` (cell names, no exposed button); `react-aria` 3.53.1 `useTree`; issue #383
- **Evidence level / confidence:** EV-C / Strong for the DOM; whether NVDA and Narrator accept the tree grid as a tree is for the B02-T10 walkthrough
- **Affects:** B02-T05, #383, ADR-0003, A11Y-18
- **Papers:** none yet

### F-173 · Under colour-vision deficiency the signal colours collapse, but the indicators keep 3:1

- **Date:** 2026-10-30 · **Phase:** B02 · **Type:** accessibility · **Status:** new
- **Statement:** The B02 design review simulated protanopia, deuteranopia and tritanopia (Machado, Oliveira and Fernandes 2009, severity 1.0, on linear sRGB) and achromatopsia (relative luminance) on the rendered tokens of all four themes. `focus`, `accent` and `border-strong` keep at least 3:1 against `surface-0..3` for every deficiency; the lowest is 3.03:1, dark `border-strong` on `surface-3`, the same as without simulation. The signal colours do not stay apart: the closest pair falls to an OKLab distance of 0.006 (dark, achromatopsia, warning and success), 0.018 (high contrast light, achromatopsia, error and success) and 0.039 (light, deuteranopia, error and success), against about 0.02 for a just-noticeable difference. No token change can fix this for three saturated hues of similar lightness, so the existing rule holds: a signal colour never carries meaning without text or an icon (DL-X08, §5.3). Every status view in later phases must pass that check in the review record.
- **Evidence:** `packages/ui/tokens/cvd.ts`; `packages/ui/test/cvd.test.ts` (`a11y_25 indicators keep 3 to 1 under every cvd type`, `dl_x08 some signal pairs collapse under cvd so signals need text`); [B02 design review](../sdlc/reviews/B02-design-review.md) (the per-theme table)
- **Evidence level / confidence:** EV-C / Moderate. A simulation, not a test with CVD viewers; the severity-1.0 matrices model full dichromacy, so anomalous trichromats see more difference than shown. Matrix values TODO(verify) against the paper's published table
- **Affects:** B02-T09, DL-X08, A11Y-25, design-language §5.3; B04 series colours (§6)

### F-180 · Raster line charts read through hand-entered anchors within 0.15 % median error at 1x and 2x

- **Date:** 2026-10-30 · **Phase:** C02 · **Type:** benchmark · **Status:** new
- **Statement:** The raster line extractor (C02-T03) reads the line charts of DS-007's raster part: Vega-Lite, Plotly and D3 renders at 1x and 2x (vl-convert 1.9.0, resvg, anti-aliased), three series each, 18 series in all. The calibration is entered the way a person would click it: the outermost value ticks and every category label, taken from the SVG's axes and scaled to the image. The median error per series is 0.05 % to 0.15 % of the chart's largest value, and 2x is no worse than 1x. The C02 bar is 2 % (notes/06 §8). Of the 216 values, one category point was not found (plotly-line at 2x). The error is close to F-163's vector error, so on clean renders the anchors' half-pixel placement, not the line search, sets the limit.
- **Evidence:** `crates/engines/sonarithm-plotsona/src/raster/extract/tests.rs` (`raster_line_charts_read_within_two_percent`, which prints each series' median); the C02-T03 PR (#395)
- **Evidence level / confidence:** EV-C / Moderate. These are clean, anti-aliased renders of synthetic data with exact anchors; JPEG artefacts, scans, thick or dashed lines, crossing series of one colour and anchors clicked by people are not measured. C02-T08 adds those and the CI job
- **Affects:** C02 exit, FR-007, FR-008, DS-007, F-163, 10 §3
- **Papers:** none yet

### F-181 · A palette colour can sit inside the default colour distance of a chart's grey axis line

- **Date:** 2026-10-30 · **Phase:** C02 · **Type:** engineering · **Status:** new
- **Statement:** Vega's first category colour, `#4c78a8` (76, 120, 168), lies 69.9 from its axis-domain grey `#888` (136, 136, 136) in Euclidean 8-bit RGB. With a fixed foreground threshold of 70, the axis line counted as the blue series, and X step followed the axis (median error 45 %). The other series were unaffected. SONARITHM now lowers each automatically found series colour's threshold to half its distance to the nearest grey and to the nearest other series colour (32.5 for that blue); the series then reads at 0.13 %. A fixed colour-distance threshold is not safe when colours are picked automatically. A threshold the user sets for a colour they picked is unchanged.
- **Evidence:** `raster::colour::separating_threshold`; the failing run before the change (vl-line@1x, series 3); Vega's default palette and axis colours (TODO(verify): cite the Vega config defaults page)
- **Evidence level / confidence:** EV-C / Strong for this case
- **Affects:** C02-T03, C02-T04 (bars use the same colours), 10 §3
- **Papers:** none yet

### F-182 · Raster bars and scatter markers read within 0.11 % with every marker counted

- **Date:** 2026-10-30 · **Phase:** C02 · **Type:** benchmark · **Status:** new
- **Statement:** On DS-007's raster part (vl-convert 1.9.0 renders at 1x and 2x, anchors entered on the outermost ticks and category labels), C02-T04 classifies every bar and scatter render correctly. The bar charts (Vega-Lite, Plotly, D3; five categories) read with a median error of 0 % to 0.11 % of the largest value, the worst bar 0.11 %. The scatter plots (60 points each) report exactly 60 markers in all six renders, with each truth point's nearest marker at a median of at most 0.07 % of the x span and 0.11 % of the y span. Before fragments were joined, Vega-Lite's hollow circles gave 62 markers: anti-aliasing broke one ring into three arcs. As with lines (F-180), the anchors' half-pixel placement, not the detector, sets the error on clean renders.
- **Evidence:** `crates/engines/sonarithm-plotsona/src/raster/extract/tests.rs` (`raster_bar_charts_read_within_two_percent`, `raster_scatter_charts_report_their_markers`, which print each render's figures); `raster/marks/tests.rs`; the C02-T04 PR
- **Evidence level / confidence:** EV-C / Moderate. Clean renders of synthetic data, well-separated markers and five bars; overlapping markers in dense scatters, stacked and grouped bars, horizontal bars, JPEG artefacts and anchors clicked by people are not measured (C02-T08)
- **Affects:** C02 exit, FR-007, FR-008, DS-007, F-180, 10 §3

### F-174 · Okabe–Ito keeps 3:1 and stays apart under CVD only if light themes darken its warm hues to browns

- **Date:** 2026-10-30 · **Phase:** B04 · **Type:** accessibility · **Status:** new
- **Statement:** Taken as published, four Okabe–Ito colours fall below 3:1 on the light surfaces (orange, sky blue, yellow and reddish purple; §6), and black fails on dark. Darkening each colour only to 3:1 makes the light palette's lightness so uniform that, under deuteranopia, series 4 (yellow) and 6 (vermillion) come within ΔE_OK 0.011 of each other, about half a just-noticeable difference. The derivation in `packages/ui/tokens/series.ts` changes only OKLCH lightness, and darkens one member of the closest pair until every pair is at least 0.03 apart under protanopia, deuteranopia and tritanopia. The result:
  - Closest pairs: light 0.031, high contrast light 0.031, dark 0.075, high contrast dark 0.051.
  - The light theme's orange, yellow and vermillion become #6C4200, #443E00 and #762300: they lose the identity their names suggest.
  - 0.04 is not reachable on light: the loop pushes colours to black.
  - Achromatopsia is not reachable at all for eight hues at 3:1, so the dash pattern, marker and label of every series carry it (§6).
- **Evidence:** `packages/ui/test/series.test.ts` (`dl_x08 the series stay apart under every dichromacy`, `a11y_25 the series meet 3 to 1 …`, `the recipe's series are what the derivation gives`); `node packages/ui/tokens/series-print.ts`; the cvd.ts simulation of F-173
- **Evidence level / confidence:** EV-C / Moderate. A simulation, not a test with viewers; the 0.03 threshold (1.5 just-noticeable differences) is a proposal, flagged INFER in the code
- **Affects:** B04-T01, design-language §6, DL-X08, A11Y-25; legends and direct labels (B04), a lighter light-theme plot background if the browns are judged unacceptable
- **Papers:** none yet

### F-175 · The Windows network-deny harness blamed a test binary for the probe's attempts, through a reused process ID

- **Date:** 2026-10-30 · **Phase:** B01 · **Type:** engineering · **Status:** new
- **Statement:** On #406's CI, `network-deny (windows-latest)` reported that `speech-c7bd1538ecf07f1c.exe` made 2 connection attempts (#409). The harness's own trace shows otherwise. Both events name the application `network_probe-3f7721675b72fccf.exe`, both go to 192.0.2.1:9 (the probe's TEST-NET-1 target), and both carry PID 6916. That is the self-test probe's deliberate pair, made one step earlier as PID 6916. Windows then gave PID 6916 to the `speech` test binary, and the harness matches Security-log events by PID alone, over a window that starts 60 s before the run. The speech path made no connection: its tests use a recording speaker, not the OS voice. The harness now counts an event only when its application file name matches the executable as well.
- **Evidence:** job 124105949271 of run 41068449405 (the attempt lines with the probe's path); `xtask/src/network_deny.rs` `wfp_events_of_an_earlier_process_with_the_same_pid_do_not_count`
- **Evidence level / confidence:** EV-C / Strong for this run: the application path and the destination identify the probe. Whether the OS voice (SAPI or OneCore) would connect when self-voicing really speaks is not covered by any test, because no test drives the OS speaker; TODO(verify) with a real voice under the harness
- **Affects:** NFR-04, ADR-0012, #409; testing-strategy §10; macOS's sandbox log is also read by PID (sandbox_lines), with the same reuse risk
- **Papers:** none yet

### F-183 · Clean raster line and bar charts read within 0.14 % median error at every resolution, but JPEG artefacts break automatic series colours

- **Date:** 2026-10-30 · **Phase:** C02 · **Type:** benchmark · **Status:** new
- **Statement:** The C02-T08 harness scores the raster line and bar charts of DS-007 (Vega-Lite, Plotly and D3; three line charts of 36 values and three bar charts of 5) rendered by vl-convert 1.9.0 at 1x, 1.5x and 2x, and at 1x without anti-aliasing, 24 clean PNG renders in all, read through anchors placed on the SVG's ticks. Every clean render reads as `cv_estimate` with a confidence; the worst median error is 0.14 % of the chart's largest value (crisp D3 line) and the lowest RMS-F1 0.994, well inside the C02 bar of 2 %. The same renders re-encoded as JPEG fare differently: the bar charts keep their accuracy at quality 90 and 60 except D3's (whose thin bars lose most categories), but line charts often fail at quality 90 and mostly fail at 60 (no series, or one or two of three). Chroma subsampling and ringing spread each line colour over many nearby shades, so automatic series-colour detection (C02-T03) splits or drops series. Plotly's line charts have one point per series near 8.6 % off at every clean resolution, where the line meets a marker of the same colour; the median is unaffected.
- **Evidence:** `crates/engines/sonarithm-plotsona/src/accuracy/raster.rs` (`the_raster_corpus_meets_the_c02_accuracy_bar`, which writes the per-render table); the CI artefact `chart-accuracy` (`chart-accuracy-raster.md`); the C02-T08 PR
- **Evidence level / confidence:** EV-C / Strong for clean renders of this corpus; Moderate for JPEG (two qualities from one encoder, the `image` crate's). Scans, photographs, thick or dashed lines, overlapping series of one colour and anchors clicked by people are not measured
- **Affects:** C02 exit (FR-007, SDR-42), DS-007, F-180, F-182, F-181 (colour thresholds), 10 §3; a follow-up for JPEG robustness (colour clustering before series detection, or a user-picked colour per series as WebPlotDigitizer's documentation describes)
- **Papers:** none yet

### F-184 · Series of estimates were announced like exact data, and the lowest confidence is named relative to each series

- **Date:** 2026-10-30 · **Phase:** C02 · **Type:** engineering (accessibility) · **Status:** new
- **Statement:** Auditing announcements for C02-T09 found that AUDAXIS labelled each estimated *point* at both verbosities (03 §4 rule 3), but a *series* or *segment* made of estimates was announced with its count and range and no qualifier, so a listener moving by series heard estimated data as if it were exact. Series and segments now carry the estimate mix: terse "estimated" or "partly estimated"; verbose "N of M points estimated from a chart image, median confidence …". Where a series is least certain is named relative to the series, not against a fixed cut-off: the window of max(3, n/10) consecutive estimated points with the lowest median confidence, said only when that median is below the series' median ("lowest confidence around March, median 0.41"). The extraction confidence is uncalibrated (notes/06 §8, C02-T05), so an absolute threshold such as 0.5 would claim more than the number means; OD-07 records that P08/H-06 must decide one. The facade's text description names the same stretch by time. The keyboard guide's terse point example began with "About", which would have put estimate wording into the value part, outside the Provenance part a listener can move (A11Y-12); the example now ends in "Estimated." as the composer says.
- **Evidence:** `crates/engines/sonarithm-audaxis/tests/announce.rs` (`a11y_02_series_and_segments_of_estimates_say_so`); `sonarithm_core::lowest_confidence_stretch` and its tests; keyboard-and-navigation §8.2; the C02-T09 PR
- **Evidence level / confidence:** EV-C / Strong for the gap; the stretch rule is a design choice awaiting P08
- **Affects:** C02-T09, A11Y-02, A11Y-12, A11Y-24, FR-008, OD-07, H-06
- **Papers:** none yet

### F-201 · A hole on a pole's steep slope was taken for a jump, because the sides were compared with a fixed tolerance

- **Date:** 2026-11-01 · **Phase:** C03 · **Type:** engineering (robustness) · **Status:** new
- **Statement:** The property test `poles_and_holes_are_never_missed` (CI on #438, Windows job 124726369301) found a hole at b = 2.0748585424618367 only 0.0042 from a pole at a = 2.0790137126487966, on a 3-sample grid. The scan reported it as a jump, with left −240.66171264649452 and right −240.66634618052169. The scan classifies a candidate by its one-sided limits, read at 10^-2 to 10^-8 grid steps, and called the sides different when they disagreed by more than a relative 1e-6. Beside a pole the slope is about 1/0.004² ≈ 60 000, so the two sides of a smooth point differ by about the slope times the probe distance, 2e-5 relative here, and the fixed tolerance failed. Now the sides also count as equal when their signed difference D(h), extrapolated to h = 0 through the two closest probes (a decade apart, D₀ = (10·D₆ − D₅)/9), is 0 to a relative 1e-6. To first order D(h) = D₀ + C·h, with D₀ = 0 at a hole and the jump at a jump. A first version judged the gap's size instead (fivefold smaller over two decades). Lane U's review showed that this drops a real jump: for `floor(x) + 1/(x − 2.0001)` at 101 samples, the gaps were 15, 0.6 and 0.84, because the slope term dwarfed the jump before the gap settled at it. A rule that only asks the gap to halve each decade fails too, with the pole 5e-5 away (gaps of about 63, 5.4 and 0.36). The extrapolated D₀ gives −1 in both cases. ADR-0028 is unchanged.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/scan.rs` (`closing`, `classify`); the tests `f_201_a_hole_beside_a_pole_is_not_a_jump` (the counterexample, pinned), `f_201_a_jump_beside_a_pole_is_still_a_jump` and `f_201_a_jump_on_a_steep_slope_stays_a_jump` (the review's repro, the 5e-5 case and a wide domain)
- **Evidence level / confidence:** EV-D / Strong (a reproduced property-test failure, with regression tests)
- **Affects:** C03-T02 (the scan), the function cues and the critical-point list

### F-200 · A function's glide is paced by row, so the rows CURVASONA inserts bend its timeline against x

- **Date:** 2026-11-01 · **Phase:** C03 · **Type:** engineering (timing, determinism) · **Status:** new
- **Statement:** After `main` (with #429) was merged into C03-T07, the CLI snapshot `quick_function` changed from cues at "1.689 to 6.303 seconds" to "1.691 to 6.301 seconds". It changed identically on HW-01 (Windows 11), on CI Windows and on CI macOS arm64, so it is not a platform difference. Applying #429 alone to the pre-merge branch reproduces it on one machine. The cause: the function spec's `pace: { duration: 8 s }` makes NUMISONA space the glide's rows evenly in time (8/1003 s, about 8 ms each, for the cubic's 1000 samples plus three critical-point rows), by row position, not by x. Every row the function source inserts at a critical point shifts the rest of the timeline by one step, so #429, which changed how many rows are inserted (F-194), moved the cues by about 2 ms. Each cue still sounds exactly when the glide reaches its point (tested bit for bit), but against x the cubic's cues sit 0.3 to 8 ms from (x − min)/(max − min) · T (the zero at √3: 6.301 s rather than 6.309 s). With `secondsPerUnit` pacing on `x`, every cue lands on its x-linear time exactly, but NUMISONA then ends the glide one unit late (9.333 s for 8 s), so that switch waits for a look at NUMISONA.
- **Evidence:** CI jobs 124721139173 (macOS) and 124721139250 (Windows); the tests `f_200_the_cubic_cue_times_are_pinned` and `f_200_each_earcon_sounds_at_its_rows_glide_breakpoint` in `crates/sonarithm/tests/functions.rs`; a local probe of the glide's breakpoints against x
- **Evidence level / confidence:** EV-D / Strong (measured on three platforms and reproduced by bisection)
- **Affects:** C03-T07 (`Function::spec`, `quick --function`), C03-T06 (the 40-curve gallery), the 04 §6.3 example; follow-ups for NUMISONA's `secondsPerUnit` tail and for x-linear function time
- **Papers:** none yet

### F-199 · CURVASONA's canonical form put a negative divisor in parentheses, which made it deeper than its source

- **Date:** 2026-11-01 · **Phase:** C03 · **Type:** engineering (robustness) · **Status:** new
- **Statement:** The fuzz target `expr_parse` (CI job 124637048464, on #429) found an expression that parses but whose canonical form does not: `4-x/` followed by 32 unary minuses and `4-1`. The parser counts one level of nesting per unary minus, so the input sits exactly at the depth limit of 32. The printer wrapped any negative divisor in parentheses (`x/(-4)`), although ADR-0028's grammar reads a divisor as a unary expression, so `x/-4` already means `x/(-4)`; the added parentheses were a 33rd level, and the canonical form failed with `SNR-ENG-402`. A divisor now prints with the multiplier's rule, so `x/-4` stays `x/-4` and the printer adds parentheses only where the grammar needs them, which the source text must then also have had. ADR-0028's grammar is unchanged; only the canonical spelling of a negative divisor changes.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/expr.rs` (`Node::print`); the test `f_199_a_divisor_at_the_depth_limit_reads_back`; the corpus seed `seed-regression-divisor-depth.txt`, replayed by `tests/fuzz_regressions.rs` on every run
- **Evidence level / confidence:** EV-D / Strong (a reproduced fuzz failure, with a regression test)
- **Affects:** C03-T01 (the expression facade), ADR-0028's canonical form

### F-198 · An unboxed `Expr` was 19 KB, and sampling a function overflowed the CLI's 1 MB main-thread stack in debug builds

- **Date:** 2026-11-01 · **Phase:** C03 · **Type:** engineering (robustness) · **Status:** new
- **Statement:** exmex's compiled form (`FlatEx<f64>`) keeps its buffers inline, so CURVASONA's `Expr` was 19,296 bytes and `Calculus` (f, f′, f″) 57,888 bytes. A debug build copies these values through many frames. Sampling `x^3 - 3*x` needed between 768 KB and 1 MB of stack, and the facade's `read_function` between 1 and 1.5 MB, with no recursion involved. That overflowed the 1 MB main thread on Windows, where the CLI runs: both new CLI tests crashed with `STATUS_STACK_OVERFLOW`. Python calls run on the caller's thread, so they were exposed the same way. Boxing the compiled form makes `Expr` 136 bytes, and `read_function` then runs in 512 KB but not in 256 KB. Measured on HW-01 (Windows 11, debug build); release builds use less stack and were not measured.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/expr.rs` (`compiled: Box<Compiled>`); the test `a_function_reads_and_compiles_on_a_1_mb_stack`; stack probes at 256 KB to 4 MB during C03-T07
- **Evidence level / confidence:** EV-D / Strong (measured, one machine)
- **Affects:** C03-T07, every host that runs CURVASONA on a main thread (CLI, Python, the app's command thread)
- **Papers:** none yet

### F-197 · A registered context provider is recorded in every Score, so the facade adds CURVASONA only where a spec uses it

- **Date:** 2026-11-01 · **Phase:** C03 · **Type:** engineering (determinism, diagnostics) · **Status:** new
- **Statement:** The compiler's context stage records every registered context provider as a contributing engine, whether or not it adds a track, and it warns that context cues need AUDAXIS (`SNR-COMP-023`) only when no provider is registered. CURVASONA provides the `curvasona.cues` context track, so registering it for every compile in the facade (C03-T07) would have put `curvasona` into every Score header, changing every Score hash and golden snapshot, and would have hidden `SNR-COMP-023` from every CLI and Python compile that has no AUDAXIS. The facade therefore adds CURVASONA only to compiles whose spec uses it: a `function` source, an `audify` track, or `critical-points` in `analysis.detect` (its manifest's spec keys). Found by reading `pipeline.rs`, not measured; a compiler-level fix (record a provider only when it adds a track, and warn when no provider handles `context`) would change the app's Score headers and is left as a follow-up.
- **Evidence:** `crates/sonarithm-compiler/src/pipeline.rs` (`add_context`); `crates/sonarithm/src/api.rs` (`uses_curvasona`); the test `specs_that_do_not_use_curvasona_compile_without_it`
- **Evidence level / confidence:** EV-D / Strong (code reading, with a test)
- **Affects:** C03-T07, the facade's default engines, every later engine with a context provider

- **Papers:** none yet

### F-196 · An audification without `k` or `band` needs a target band, and no study gives one

- **Date:** 2026-11-01 · **Phase:** C03 · **Type:** engineering (sound default) · **Status:** new
- **Statement:** RFC-0001 lets `tracks[].audify` be omitted ("`k` computed from the data band"), and SDR-31 computes k "from the data band to a target audio band", but neither names the target. C03-T05 needs one, so it maps the top of the series' band (fs/2) to 4 kHz. This follows ED-06's reasoning (keep the main energy below about 4 kHz, which survives high-frequency hearing loss and small speakers, A11Y-14); it is not measured. It is recorded as SDR-44 (`INFER`, experimental) until a listening study (HP-21 or a later experiment) tests it against alternatives such as 2 kHz or 8 kHz.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/audify.rs` (`DEFAULT_TOP_HZ`) and the test `sdr_44_infer_the_default_k_puts_the_top_of_the_band_at_4_khz`; design-rules SDR-44
- **Evidence level / confidence:** EV-D / Weak (an extrapolation)
- **Affects:** SDR-31, SDR-44, C03-T05
- **Papers:** none yet

### F-195 · Audification resamples with a deterministic windowed-sinc, because rubato's FFT path is not bit-identical across CPUs

- **Date:** 2026-11-01 · **Phase:** C03 · **Type:** engineering (determinism) · **Status:** new
- **Statement:** RFC-0007 §3 planned to resample audified series with rubato at plan time. Golden audio relies on renders being bit-identical on Windows, Linux and macOS (F-039, ADR-0013), and rubato's fixed-ratio resamplers use FFTs through realfft and rustfft, which choose AVX, SSE or NEON code at run time, so their rounding can differ between x86 and ARM machines (TODO(verify) by rendering the same plan on both). `sonarithm_render::resample` is a windowed-sinc interpolator (Blackman window, 16 taps a side at unity, widened to low-pass at the output's Nyquist when downsampling, at most 4096 a side), computed with `libm` and IEEE arithmetic only, normalized per output sample and clamped to [−1, 1]. It is slower than an FFT resampler (about 2·16/min(1, ratio) multiply-adds per output sample), acceptable for at most 2^22 samples on a worker thread. rubato stays licence-checked (C03-T01) should a deterministic mode be needed later. The audifier voice then reads the buffer with linear interpolation at the transport speed, so speed changes and seeks need no new resampling on the audio thread.
- **Measured (2026-11-01, HW-01, Windows 11, debug build, local):** a 440 Hz tone resampled from 8 kHz, 96 kHz and 44.1 kHz to 48 kHz measured 440.000, 439.000 and 439.000 Hz by rising zero crossings over one second (the method resolves 1 Hz); a 20 kHz tone at 96 kHz resampled to 16 kHz kept a peak of 6.0e-7 of 0.5 (about −118 dB). Linux and macOS values: TODO(verify) from CI.
- **Evidence:** `crates/sonarithm-render/src/resample.rs` and its tests (`sdr_31_resampling_keeps_the_tone`, `downsampling_removes_what_the_output_cannot_hold`); `crates/sonarithm-render/src/graph/tests.rs` (`sdr_31_an_audified_sine_plays_the_expected_tone`); the C03-T05 PR
- **Evidence level / confidence:** EV-D / Moderate (the SIMD rounding difference is inferred from rustfft's design, not measured)
- **Affects:** RFC-0007 §3, 06 (audifier row), C03-T05, C03-T06
- **Papers:** none yet

### F-194 · A zero of a guard exactly on a grid point hid a second zero in the next grid step

- **Date:** 2026-10-31 · **Phase:** C03 · **Type:** engineering (correctness) · **Status:** new
- **Statement:** The property test `poles_and_holes_are_never_missed` failed on an unrelated PR's CI run (41224538690): with a pole at x = 0 and a hole at b in (0, 4), on the coarsest grid -4, 0, 4, the scan found the pole and missed the hole. The guard `x(x - b)` is exactly 0 at the grid point 0 and positive at -4 and 4, so the second zero has no sign change on the grid, and the dip search skipped grid points where the guard is 0. The zero finder of `critical` shared the gap (`x*(x - 1)` on the same grid). Both intervals beside an exact zero are now searched like a dip. Searching beside an exact zero then found some features a float step from a grid point, so the source added near-duplicate rows: extra breakpoints in a glide (the `a_parabola_score_snapshot` diff, which was refused rather than accepted) and an inflection hidden from the analyzer's recompute by two rows at almost the same x. Rows within 1e-9 of the domain width are now one row, keeping a pole or hole, else the grid point; duplicate features are merged within each kind; and a run of undefined samples within a float step of a pole or hole counts as that point. Earlier random runs passed because an exact zero on the grid needs a pole at a grid point, which proptest's shrinking towards 0 finds and uniform sampling rarely does.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/scan/tests.rs` (`a_zero_on_the_grid_does_not_hide_a_second_one_beside_it`), `src/critical/tests.rs` (`a_zero_on_the_grid_does_not_hide_its_neighbour`); `scan::dips`
- **Evidence level / confidence:** EV-C / Strong (reproduced)
- **Affects:** C03-T02, C03-T03 (fixed in this PR)

### F-193 · NUMISONA's glide `src` ranges include their last row, against `RowRange`'s one-past-the-end contract

- **Date:** 2026-10-31 · **Phase:** C03 · **Type:** engineering (correctness) · **Status:** new
- **Statement:** C03-T04's cue track found no cue times at all: it expected one glide breakpoint per row of the glide's `src` range, but a glide over rows 0 to 120 has 121 breakpoints with `src` 0..120. `sonarithm_core::RowRange` documents `end` as "one past the last row", so NUMISONA's glide ranges stop one row short (the next glide, after an empty row at a pole, starts at 121). CURVASONA's cue track now accepts both forms. Other readers of glide `src` (AUDAXIS's point placement, linked selection in the app, the scrub preview) may assume the documented contract and miss the last row of each glide. Fixing NUMISONA changes every Score snapshot with glides, so it needs its own PR with the snapshot updates; it is outside C03.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/cues.rs` (`row_times`); a compile of `1/(x^2 - 4)` with 401 samples gives glides with `src` 0..120, 121..280, 281..401 and 121, 160, 121 breakpoints; `crates/sonarithm-core/src/ids.rs` (`RowRange::end`)
- **Evidence level / confidence:** EV-C / Strong (reproduced)
- **Affects:** NUMISONA's glide encoder (A06), every reader of `Glide::src`, C03-T04

- **Papers:** none yet

### F-192 · exmex's symbolic derivatives overflow the stack on allowed input, so CURVASONA differentiates on its own tree

- **Date:** 2026-10-31 · **Phase:** C03 · **Type:** engineering (security, correctness) · **Status:** new
- **Statement:** C03-T03's tests aborted with a stack overflow (Windows, debug build, 2 MB test threads). On a 2 MB thread, exmex 0.21.0's `partial` overflowed on the first derivative of `x*(x*(…(x)…))` nested eight deep, well inside CURVASONA's limits (depth 32, 512 nodes), and on the second derivative of `exp(-x^2) * cos(3*x)`, whose first derivative is only 89 characters long. A stack overflow aborts the process and cannot be caught, so an expression in a spec or a project could crash the app or the CLI. C03-T01's code had this path on `main`, but nothing shipped reached it (the crate is unpublished and not in the facade yet). exmex's parsing and evaluation of the lowered form stayed safe up to 1600 nested parentheses on a 1 MB thread. CURVASONA now computes derivatives itself on its parse tree, with simplification and a node budget charged while the derivative is built (8192 nodes; input depth 128, result depth 256), and keeps exmex for evaluation only. A bonus: `abs`, `floor`, `ceil`, `round`, `trunc`, `cbrt` and `atan2` now have symbolic derivatives, so only `min` and `max` fall back to numerical ones. The first version of CURVASONA's own derivative also overflowed a 1 MB thread on a 256-factor product (quadratic growth before the size check), which the budget charged during construction fixed.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/expr/tests.rs` (`deep_derivatives_run_on_a_small_stack`, on a 1 MB thread); the fuzz target `expr_parse` now takes second derivatives; the C03-T03 PR
- **Evidence level / confidence:** EV-C / Strong (reproduced on a fixed stack size)
- **Affects:** C03-T01 (fixed in C03-T03), 07 CURVASONA, supply-chain-and-licensing (exmex row), 12 §6
- **Papers:** none yet
- **Draft upstream issue (for the owner to file; not posted):**

  > **Title:** `partial` overflows the stack on shallow expressions
  >
  > With exmex 0.21.0 (`partial` feature), `FlatEx::<f64>::parse("x*(x*(x*(x*(x*(x*(x*(x*(x))))))))")?.partial(0)` overflows a 2 MB thread stack (Windows, debug build), as does taking `partial(0)` twice of `exp(-x^2)*cos(3*x)`, whose first derivative unparses to 89 characters. Parsing and evaluating the same expressions work on a 1 MB stack, even with 1600 nested parentheses. A stack overflow aborts the process, so an application that differentiates user input cannot recover. A depth limit, or an iterative `to_deepex`/`partial`, would avoid it. A reproduction runs `partial` inside `std::thread::Builder::new().stack_size(2 << 20)`.

### F-191 · Insight labels are read as data names, so CURVASONA's critical-point labels are English

- **Date:** 2026-10-31 · **Phase:** C03 · **Type:** engineering (accessibility, localization) · **Status:** new
- **Statement:** AUDAXIS announces an insight as the catalogue word "Insight" followed by the annotation's label, which the composer treats like a series name: data, read as written (announce.ftl header). That fits labels that come from the data, but CURVASONA's labels name kinds of point ("local maximum", "vertical asymptote", "hole"), so in another locale a listener would hear the word "Insight" translated and the kind of point in English. C03-T03 keeps the labels as short, stable English phrases and adds no catalogue messages, because the core `Annotation` has no field for a message ID and adding one changes a core type every engine shares. Mapping analyzer labels to catalogue messages (for CURVASONA's and ECHOMETRIC's insights alike) belongs with the B07 locale work or an RFC for annotation kinds.
- **Evidence:** `crates/engines/sonarithm-audaxis/src/announce/compose.rs` (the `NavKind::Insight` arm); `crates/sonarithm-i18n/locales/en-US/announce.ftl` (header); `sonarithm_curvasona::critical::FeatureKind::label`
- **Evidence level / confidence:** EV-C / Strong (source)
- **Affects:** C03-T03, C05 (ECHOMETRIC insights), B07 (locales), A11Y-24
- **Papers:** none yet

### F-190 · exmex alone reads -2^2 as 4, so CURVASONA parses expressions itself

- **Date:** 2026-10-31 · **Phase:** C03 · **Type:** engineering (correctness, security) · **Status:** new
- **Statement:** Reading exmex 0.21.0 for C03-T01 showed three differences from school maths that a blind maths teacher would hear as wrong answers: unary operators bind tighter than every binary one, so `-2^2` evaluates to 4 and `-x^2` is a parabola opening upwards; `signum(0)` is 1 (Rust's `f64::signum`); and `log` is the natural logarithm, where many school books mean base 10. exmex also has no symbolic derivative for `abs`, `floor`, `min`, `max`, `cbrt` or `atan2`, and its number syntax has no exponents. CURVASONA therefore reads the expression with its own parser (ordinary precedence, `^` right-associative, a closed set of names, length, depth and node limits) and hands exmex only a form with every operation in parentheses and numbers it can read, so exmex evaluates and differentiates but never decides what the text means. `log` is refused with a hint, `sign` is left out, and `dy` falls back to central differences where exmex has no derivative (`SNR-ENG-405`). The facade also keeps exmex, a single-maintainer crate (tech-stack: Trial), replaceable without changing the language.
- **Evidence:** `crates/engines/sonarithm-curvasona/src/expr/tests.rs` (`exmex_alone_reads_minus_two_squared_as_four`, `precedence_follows_ordinary_maths`, `canonical_form_round_trips_and_lowering_keeps_meaning`); exmex 0.21.0 `src/lib.rs` ("unary operators always have higher priority") and `src/operators.rs`
- **Evidence level / confidence:** EV-C / Strong (source and tests)
- **Affects:** C03-T01, 07 CURVASONA, A11Y-28, FR-009, the teacher pilot's tasks (C03-T09)
- **Papers:** none yet

### F-176 · Two CI-only test flakes came from React Aria acting a frame or an event later than the tests assumed

- **Date:** 2026-10-31 · **Phase:** B01, B02 · **Type:** engineering (testing) · **Status:** new
- **Statement:** (1) React Aria's `FocusScope` returns the focus to the opener in the animation frame after a dialog unmounts, so an E2E test that read the focused element right after the dialog was hidden failed on WebKit in CI (run 41158576905), though it passed locally. (2) After Escape closes a tooltip through React Aria's document listener, the trigger still counts as focused, because that listener stops the event before the trigger sees it. Any pointer entering the trigger then opens a new tooltip at once. A local experiment confirmed that one `pointerenter` after Escape reopens it. The `Icon Only` story checked for "no tooltip" and failed in every theme across five CI runs on 2026-10-30 and 2026-10-31, though Escape had worked. Where the stray pointer event comes from on the runner is not known.
- **Evidence:** `apps/desktop/e2e/audio.spec.ts` (`fr_118_preferences_choose_the_device_and_set_the_volume_by_keyboard`, which now polls); `packages/ui/src/components/Button.stories.tsx` (`IconOnly`, which now waits for the tooltip's removal); React Aria 3.53.1 `FocusScope` and `useTooltipTrigger`
- **Evidence level / confidence:** EV-C / Strong for the mechanisms (source and the local experiment); Weak for the pointer event's origin in CI, TODO(verify) if the story fails again
- **Affects:** testing-strategy §12 (browser tests): read focus and overlay state with polling or observers, not one read after an overlay closes
- **Papers:** none yet

### F-177 · React Aria's Table cannot say a row's place in a million, so the row number is content

- **Date:** 2026-10-31 · **Phase:** B04 · **Type:** engineering (accessibility) · **Status:** new
- **Statement:** React Aria Components' Table builds a collection node for every row and sets `aria-rowcount` and `aria-rowindex` from that collection (react-aria 3.53.1, `useTable` and `useTableRow`), with no override. A million-row table cannot be one collection, and patching the attributes would be hand-rolled ARIA that React Aria rewrites. The data table holds windows of 1,000 rows from `table_page` and puts each row's number in the whole table in a row-header column, so screen readers announce the true position as content while `aria-rowcount` counts the window. A visible and politely announced range ("Rows 1,001 to 2,000 of 1,000,000") and whole-table moves (window edges, Primary+Home/End, "Go to row…") keep the user's place.
- **Evidence:** `apps/desktop/e2e/table.spec.ts` (Chromium and WebKit); `apps/desktop/test/table-paging.test.ts`; the coordinator's decision of 2026-10-31
- **Evidence level / confidence:** EV-C / Strong for the constraint. How NVDA, JAWS, VoiceOver and Orca read a window's row count next to the row-number header is TODO(verify) in the owner's walkthroughs (HP-07, HP-08)
- **Affects:** B04-T05, A11Y-03, A11Y-24, FR-046, keyboard-and-navigation §5.6, 09 §3; B04-T06 (the focused row as the shared cursor)
- **Papers:** none yet

### F-178 · Two meanings of "latency calibration" in the code: a replacement and an added offset

- **Date:** 2026-10-31 · **Phase:** B04 · **Type:** engineering · **Status:** new
- **Statement:** ADR-0016 says the calibration is used for the playhead "instead of trusting the backend's reported latency", and ADR-0017's `outputLatencyS` takes the calibrated latency in place of the backend's. A05's `sonarithm_audio::LatencyCalibration`, which no code uses, instead stores an offset *added* to the engine's predicted time. B04-T04 stores the measured total output latency per device (`audio.calibratedLatencyS`) and uses it as a replacement. A loopback self-test (A05, Windows) must therefore report the total latency, the key-press-to-onset value of ADR-0016's confirmation step 3, not the difference from cpal's estimate.
- **Evidence:** `crates/sonarithm-audio/src/calibration.rs` (`heard_at` adds the offset); `crates/sonarithm-app/src/transport.rs` (`anchor`); ADR-0016 §Decision
- **Evidence level / confidence:** EV-C / Strong (source)
- **Affects:** ADR-0016 confirmation step 3; a future loopback self-test that fills the setting; `LatencyCalibration` (to retire or re-document when the self-test lands)

### F-185 · The render manifest refuses unknown fields, so even an optional field needs a new manifest version

- **Date:** 2026-10-31 · **Phase:** C02 · **Type:** engineering · **Status:** new
- **Statement:** RFC-0006's first draft added `experimental` to the render manifest as an "additive" field and kept manifest version 1, on the assumption that readers ignore fields they do not know. Checking the code before acceptance showed otherwise. Every manifest type is `deny_unknown_fields`, and `MANIFEST_VERSION` is documented to change with any change to the serialized form. So a reader built before the change would refuse a manifest carrying the field. The manifest became version 2 (RFC-0006 Q6). The field is left out when no experiment is on, so such a manifest still matches the version 1 schema once its version number is set to 1, and the version 2 reader still reads version 1 manifests. The same strictness applies to every other strict format (spec, project, correction file): an RFC that calls a field "additive" should name the reader behaviour it relies on.
- **Evidence:** `crates/sonarithm-export/src/manifest.rs` (`deny_unknown_fields`, `MANIFEST_VERSION`); `crates/sonarithm-export/tests/manifest.rs` (`manifest_version_2_records_experiments_and_version_1_still_reads`); RFC-0006 §4 and Q6
- **Evidence level / confidence:** EV-C / Strong (code and test)
- **Affects:** ADR-0013, RFC-0006, ADR-0027, 06 §7; RFC template ("Affects" should name reader strictness)
- **Papers:** none yet

### F-210 · A POSIX "C" locale stopped the desktop app before its code ran

- **Date:** 2026-11-01 · **Phase:** B04 · **Type:** engineering (portability) · **Status:** new
- **Statement:** On CI's Linux runner (`LANG=C`), the desktop smoke run hung until the step limit. The smoke watchdog (#436) then reported "page error before the smoke run started: RangeError: invalid language tag: C". WebKitGTK reports the process locale as `navigator.language`, and uPlot 1.6.32 runs `new Intl.NumberFormat(navigator.language)` at module scope (`dist/uPlot.esm.js:442`). "C" is not a BCP 47 tag, so loading uPlot threw, and with it the whole module graph, before any SONARITHM code ran. Any Linux user with `LANG=C` or `POSIX` would hit it; Windows and macOS report valid tags.
- **Fix:** `normalizeLocale` (`@sonarithm/i18n`) maps C and POSIX (with any `.encoding` or `@modifier`) and anything `Intl.getCanonicalLocales` rejects to the fallback locale, and turns POSIX names such as `en_US.UTF-8` into tags. `createLocalizer` uses it. The desktop's first import (`apps/desktop/src/locale.ts`) replaces an invalid `navigator.language` and `languages` before any other module runs.
- **Evidence:**
  - `packages/i18n/test/locale.test.ts`
  - `apps/desktop/test/locale.test.ts`
  - `apps/desktop/test/uplot-locale.test.ts`, which loads uPlot with `navigator.language` "C": a `RangeError` without the shim, a clean load with it.
  - `apps/desktop/e2e/locale.spec.ts`, which reproduces CI's error without the fix and passes with it, in Chromium and WebKit.
  - #436's smoke-run report.
- **Evidence level / confidence:** EV-C / Strong (reproduced and fixed). The WebKitGTK behaviour is inferred from the CI report; TODO(verify) on a desktop Linux with `LANG=C` (HP-02).
- **Upstream:** uPlot's module-scope `Intl.NumberFormat(navigator.language)` (1.6.32, `uPlot.esm.js:442`) could be reported upstream: guard the tag, or create the formatter lazily. Whether to open a public issue is the owner's decision.
- **Affects:** B04-T06, any library that reads `navigator.language` at import; 12 §5 (locales); B07-T07 (the locale setting must pass through `normalizeLocale`)

### F-179 · uPlot's drag selection missed in WebKit because it positions from a cached box

- **Date:** 2026-11-01 · **Phase:** B04 · **Type:** engineering · **Status:** new
- **Statement:** B04-T07's first drag-to-zoom used uPlot 1.6.32's cursor drag selection. It zoomed in Chromium, but in Playwright's WebKit a drag across the plot selected nothing. uPlot computes the selection from a bounding box it caches and refreshes only on its own resize, scroll and enter events; after a layout shift the cached box no longer matched the plot. The drag is now SONARITHM's own: pointer events on uPlot's overlay, positioned from `getBoundingClientRect()` at each event, with pointer capture. That works in Chromium and WebKit. uPlot's cursor stays off, which also keeps the hover readout out (DL-X11).
- **Evidence:** `apps/desktop/e2e/plot.spec.ts` (`a11y_26_a_drag_zooms_and_does_not_move_the_cursor`, Chromium and WebKit); `packages/viz/src/Plot.tsx`
- **Evidence level / confidence:** EV-C / Moderate. The cached-box cause is inferred from uPlot's documented behaviour and the fix; WebKitGTK and WKWebView inside Tauri are untested (TODO(verify), B04-T08)
- **Affects:** B04-T07, A11Y-26; any later uPlot interaction (B04-T08 traces in three webviews)
- **Papers:** none yet

### F-221 · Voice steals came from playback speed and double seeks, not from short notes at 1×, and a steal clicked by up to 0.29

- **Date:** 2026-11-01 · **Phase:** B04 (render, A05 code) · **Type:** engineering (real-time audio) · **Status:** new
- **Statement:** #443 (from F-211) inferred two ways a playback voice pool could overflow and a stolen voice click. Reproduced on main:
  - **Short notes.** The first path does not happen at 1×: the planner already stretches every note to two ramps, and the pool is sized from those lengths. It happens at higher speed. At 4×, a note plays four times shorter while its 5 ms ramps do not, so 2 ms notes 3 ms apart stole 12 voices.
  - **Double seek.** A second seek 1 ms into a seek crossfade stole one voice. Replacing the voice outright stepped the output by 0.293, over twice the sine's own largest step.
- **The fix:**
  - **Sizing:** pools count each voice's sounding length at the fastest supported speed, at least two ramps × 4, so no Score steals at any speed. Each track is capped at 256 voices of polyphony (pool ≤ 512 × 520 bytes ≈ 266 KB), the most events a seek restarts.
  - **Steal choice:** a voice already fading out goes first.
  - **De-click:** a steal adds the stolen voice's last output to the track's bus and fades it to zero over 5 ms. That needs no extra voice and no allocation.
- **Result:** both reproductions pass. A forced steal (a pool of one voice) and a pathological track (0.5 ms notes every 0.05 ms, about 800 deep at 4×, past the cap) steal without stepping the output. The golden-audio cases and the replication gallery are unchanged at 1×.
- **Evidence:** `crates/sonarithm-render/src/graph/tests.rs` (`issue_443_short_notes_are_not_stolen`, `issue_443_a_seek_inside_a_crossfade_does_not_click`, `issue_443_a_stolen_voice_fades_instead_of_clicking`, `issue_443_the_polyphony_cap_holds_and_steals_stay_smooth`); `crates/sonarithm-render/tests/no_alloc.rs` (`rt_voice_steals_never_allocate_on_the_audio_thread`); `cargo xtask golden` (every case identical) and `cargo xtask replication` (86 files match)
- **Evidence level / confidence:** EV-C / Strong for the reproductions and the fix (tests); the de-click's audibility was not judged by ear, only by the size of the output step
- **Affects:** F-211, #443, 05 §5 (pool sizing), 06 §3 (voices), PB-03 (bigger pools for dense short-note tracks: allocation and the render loop scan every voice of a pool for activity, so a capped 512-voice pool costs a scan of 512 flags per segment; not measured)
- **Papers:** none yet

### F-220 · A glide paced by `secondsPerUnit` held its last value for a whole time-field unit, not one point's spacing

- **Date:** 2026-11-01 · **Phase:** C03 (found), A06 code · **Type:** engineering · **Status:** new
- **Statement:** NUMISONA holds a glide's last value for one step between points, so the last value is heard (A06-T04, #151). Under `pointsPerSecond` and `duration` that step is the spacing, 1/rate or duration/n. Under `secondsPerUnit` the encoder used one unit of the time field instead, which equals the spacing only when points sit one unit apart. A CURVASONA function sampled 1003 times over x = −3…3 at 4/3 s per unit therefore ended at 9.333 s, a whole unit (1.333 s) after its last point at 8 s (#440). The hold is now the mean spacing of the points in time-field units times the seconds per unit (one unit for a single point), so that glide ends about 8 ms after its last point (computed, 6/1002 × 4/3 s). The spacing is taken in units before scaling, so data one unit apart keeps a hold of exactly one unit. Nothing in the replication gallery, the golden-audio cases or the Score snapshots changed. Notes were never affected: a note's length is its `duration` channel. No doc or ADR had defined a one-unit hold; doc 07 (NUMISONA) now states the rule for every pace.
- **Evidence:** `crates/engines/sonarithm-numisona/tests/encode.rs` (`glides_hold_one_point_spacing_under_seconds_per_unit`); `cargo xtask replication` (all 86 generated files match) and `cargo xtask golden` (every case identical) after the fix; #440
- **Evidence level / confidence:** EV-C / Strong (code and tests)
- **Affects:** 07 NUMISONA (timing), RFC-0001 §5 pace (`secondsPerUnit`), #441 (CURVASONA's function specs on x-linear time, which this unblocks)
- **Papers:** none yet

### F-212 · PB-08's script time is over budget on every CI webview, by 14 times under Xvfb

- **Date:** 2026-11-01 · **Phase:** B04 · **Type:** performance · **Status:** new
- **Statement:** The desktop smoke check `pb-08-dense-layer` redraws a 1M-point dense line in the app's own webview. In #439's CI runs at `e1f0fedf`, no webview met the 8 ms per-frame script budget:
  - WebView2: p95 15.6 ms debug, 13.7 ms release.
  - WKWebView: p95 26 ms debug, 21.0 ms release.
  - WebKitGTK under Xvfb: p95 110 ms debug, 87 ms release.
  The playhead held 59–64 fps everywhere.
- **Reading:**
  - The runners are shared and have no GPU, so these are upper bounds.
  - Xvfb in particular draws WebGL in software even though `backend` reports webgl2. It is not a WebKitGTK figure.
  - The playhead is unaffected because it moves by transform, not by redrawing.
  - Whether a real GPU meets the budget is the HW-01 run (owner, #129) and, for Linux, HP-02. Until then PB-08 stays unconfirmed, not failed.
- **Evidence:** the `smoke.json` printed in the job logs ([debug](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649561), [release](https://github.com/DuhItzAniket/SONARITHM/actions/runs/41267649558)), benchmarks-log rows "B04 (B04-T08)".
- **Evidence level / confidence:** EV-C / Moderate. Measured, but on runners unlike the target hardware.
- **Affects:** PB-08, B04-T08, B04-T02 (dense layer); 09 §3
- **Papers:** none yet
