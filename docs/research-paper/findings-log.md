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
Cross-phase build and repository infrastructure (the coordinator) uses F-090 to F-099.
A06 (NUMISONA) uses F-080 to F-089.
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

- **Date:** 2026-10-19 · **Phase:** A02 · **Type:** engineering · **Status:** new
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

### F-090 · Debug info and incremental caches made up most of 70 GB of build output across four agent worktrees

- **Date:** 2026-10-19 · **Phase:** A05 · **Type:** engineering · **Status:** new
- **Statement:**
  - Four parallel agent worktrees held about 70 GB of `target/` output after a day of stacked rebases. The largest was 37.3 GB: 20.7 GB in `debug/deps` and 13.0 GB in `debug/incremental`. Incremental caches alone were about 25 GB across the three largest worktrees.
  - On a clean build of the workspace test binaries at commit 43bc462 (Windows, `CARGO_INCREMENTAL=0`), line-table debug info for workspace crates and none for dependencies cut the build from 982 MB to 528 MB. PDB files went from 370 MB to 135 MB and rlibs from 444 MB to 228 MB. Wall time was 33 s against 25 s, from one run each.
- **Evidence:** this PR; scratch measurement with `cargo test --workspace --no-run` into two target directories, the second with `--config 'profile.dev.debug="line-tables-only"' --config 'profile.dev.package."*".debug=false'`; robocopy size listing of the worktrees before cleanup
- **Evidence level / confidence:** EV-D / Moderate (one machine, one run per configuration)
- **Affects:** dev-environment.md "Disk usage"; root `Cargo.toml` profiles
- **Papers:** none yet
