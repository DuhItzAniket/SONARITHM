# Design Rules Catalogue

> **Status:** Baseline v1 (2026-10-18), derived from the kickoff research.
> **Owner:** Research lead. **Change control:** edit only through a PR that cites evidence; never renumber.

This catalogue turns the research into stable, citable rules. Code, specs, tests, ADRs and papers refer to rules by ID (for example `SDR-17`, `A11Y-14`). The full arguments and sources are in [notes/02-theory-perception.md](notes/02-theory-perception.md) §7 and [notes/03-accessibility-hci.md](notes/03-accessibility-hci.md) §7.

## How to read and use this catalogue

| Field | Meaning |
|---|---|
| **ID** | Permanent identifier. Retired rules keep their ID and are marked `RETIRED` with a reason. |
| **Strength** | `MUST` is normative (a standard, or strong convergent evidence). `SHOULD` rests on study evidence. `DEFAULT` sets a starting value that the user can override. `INFER` is an extrapolation that SONARITHM must validate itself (it is a research hypothesis, see [../research-paper/hypotheses.md](../research-paper/hypotheses.md)). |
| **Confidence** | Strong, Moderate, Weak or Expert-opinion, as judged in the research report. |
| **Owner** | The engine or layer that enforces the rule. |

Rules:

1. A rule that changes a default must state the old value and the new one, and must link the evidence: a finding (`F-###`) or an experiment (`EXP-###`).
2. A rule marked `INFER` cannot become `DEFAULT` or `SHOULD` until a SONARITHM experiment supports it.
3. ECHOMETRIC's perceptual lint (see [../architecture/07-engines.md](../architecture/07-engines.md)) emits diagnostics that carry rule IDs, so users can always trace a warning to its evidence.

---

## SDR: Sonification Design Rules

### A. Specification and reproducibility

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-01 | MUST | A serializable spec plus the data fully determines every render. The spec is exported alongside the audio. | Hermann 2008 (systematic and reproducible criteria) | Compiler, export |
| SDR-02 | MUST | All randomness (jitter, grain scheduling, noise) is seeded and declared in the spec, along with its distribution and purpose. | Hermann 2008 | Compiler, DSP |
| SDR-03 | MUST | Designs are data-agnostic and reusable. Templates ("sonecules") can be re-applied to any dataset with a compatible schema. | Hermann 2008; Reinsch & Hermann 2023 | Spec, NUMISONA |
| SDR-04 | MUST | The public vocabulary follows Enge et al.: `substrate` (time, optionally frequency), `mark` (tone, glide, grain, sample, speech, model), `channel` (pitch, loudness, timbre, pan, duration, rate). | Enge et al. 2023 | Spec |
| SDR-05 | SHOULD | SONARITHM imports and exports Erie-style declarative specs: tones, encodings, auditory legends, sequence and overlay composition. | Kim, Kim & Hullman 2024 | Spec importers |
| SDR-06 | MUST | Modes carry specific technique names: PMSon, Audification, MBS, Earcon, Auditory icon. | Hermann 2008 | Spec, UI |

### B. Pitch mapping

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-07 | DEFAULT | Pitch is the primary quantitative channel for single-series values and trends. | Dubus & Bresin 2013; Walker & Kramer 2005; Wang et al. 2022 | NUMISONA |
| SDR-08 | DEFAULT | The default pitch range is 400–1400 Hz (about MIDI 67–89). It can be expanded to 65–1480 Hz. | Fitzpatrick & Neff 2023 (one study, n=48, Moderate) | NUMISONA |
| SDR-09 | DEFAULT | Mapping is in log-frequency (semitones or cents). Mel, ERB-rate and Bark are options. | Glasberg & Moore 1990; Traunmüller 1990; Stevens et al. 1937 | NUMISONA |
| SDR-10 | MUST | Analysis mode uses continuous, unquantized pitch. Quantizing to a scale is allowed only in Presentation mode, and the UI must show the resolution trade-off. | Walker & Nees 2011 | NUMISONA, AXONOTE |
| SDR-11 | SHOULD | An optional psychophysical scaling is available: f = f_ref·(d/d_ref)^(1/m). The exponent m comes from a concept table and is labelled as guidance. | Smith & Walker 2005; Walker 2007 | NUMISONA |

### C. Polarity

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-12 | DEFAULT | Polarity is positive for velocity, pressure, temperature, urgency, danger, proximity and generic quantities. | Walker 2002; Walker 2007 | NUMISONA concept table |
| SDR-13 | DEFAULT | Polarity is negative for mass (strong evidence) and size (split, so flagged). | Walker 2007; Walker & Kramer 2005 | NUMISONA concept table |
| SDR-14 | MUST | Ambiguous concepts (money, attractiveness) are flagged. Blind listeners leaned negative for money. | Walker 2002; Walker & Mauney 2010 | NUMISONA, ECHOMETRIC lint |
| SDR-15 | MUST | A one-click polarity flip is always available and is persisted per user profile. | Walker & Kramer 2005 (designer intuition misleads) | UI, profiles |

### D. Context cues (auditory axes)

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-16 | DEFAULT | An x-axis tick track plays ~10 ms broadband clicks at major ticks for discrete-note marks. It is off for continuous glides. | Smith & Walker 2005 | AUDAXIS |
| SDR-17 | DEFAULT | A dynamic y-axis reference tone, in a timbre distinct from the data, sounds the series maximum while values rise and the minimum while they fall. A static first-value reference is never used. | Smith & Walker 2005 (RMS error 18.09 → 12.14) | AUDAXIS |
| SDR-18 | MUST | Spoken auditory legends and axis labels (range, units, polarity) are available before playback. Hybrid speech and sonification narratives are supported. | Kim et al. 2024; Siu et al. 2022; Holloway et al. 2022 | AUDAXIS |

### E. Timing and density

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-19 | DEFAULT | The overview rate is 2–4 points/s. More than ~8 discrete points/s triggers a lint warning. | Nees & Walker 2008; Smith & Walker 2005 | NUMISONA, ECHOMETRIC lint |
| SDR-20 | INFER | Above ~10 events/s (inter-onset interval under ~100 ms), the mark switches to glide, grain texture or audification. | London 2012; Repp 2005 (inference) | NUMISONA |
| SDR-21 | SHOULD | Volatile series are slowed or smoothed, because trend reversals hurt more than density does. | Nees & Walker 2008 | NUMISONA, ECHOMETRIC |
| SDR-22 | MUST | Every note has onset and offset ramps (~40 ms reference; never zero). | Smith & Walker 2005 | DSP |

### F. Loudness, timbre, multivariate and streams

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-23 | MUST | Loudness is never the primary channel for precise values. It stays constant while pitch carries data. | Neuhoff, Kramer & Wayand 2002 | NUMISONA, lint |
| SDR-24 | DEFAULT | ISO 226:2023 equal-loudness compensation is on for perceptual consistency. It is not claimed to improve accuracy. | ISO 226:2023; Fitzpatrick & Neff 2023 | DSP |
| SDR-25 | DEFAULT | Categorical variables map to timbre. Interval and ratio variables map to pitch, duration or rate. | Walker & Nees 2011 | NUMISONA |
| SDR-26 | INFER | Concurrent series are separated by timbre, pan and staggered onset, with at most 3–4 simultaneous streams by default. | McGookin & Brewster 2004 (the cap is inferred) | TONALENS, lint |
| SDR-27 | SHOULD | Analysis mode avoids redundant spatial encodings (pan, elevation and pitch on one variable). | Nees & Walker 2007 | NUMISONA, lint |
| SDR-28 | INFER | Simultaneous tones are kept more than ~1 ERB apart. | Plomp & Levelt 1965; Glasberg & Moore 1990 | ECHOMETRIC lint |
| SDR-29 | DEFAULT | The default timbre is harmonic and mallet-like. Pure sine is an option. A "discrete square-wave" preset serves BLV chart reading. | Nees & Walker 2007; Sharif et al. 2022 | AXONOTE instruments |
| SDR-30 | SHOULD | Space (stereo or binaural) is used for kinematic or positional variables or for stream segregation. Users are warned that generic HRTFs cause front–back confusion. | Dubus & Bresin 2013; Wenzel et al. 1993; Beadling & Vickers 2023 | AUDAXIS spatial |
| SDR-43 | INFER | Experimental, off by default (`experimental.lowConfidenceTimbre`, RFC-0006): an estimated value (`cv_estimate`, `ml_estimate`, `cloud_estimate`) plays darker than it otherwise would, by 0.25 of the brightness scale plus 0.25 × (1 − confidence), kept within [0, 1]. Pitch, timing and loudness do not change, and speech, text and the table still label every estimate (A11Y-24). | None for blind listeners: no study of extraction uncertainty conveyed in audio was found ([notes/06 §7–§8](notes/06-chart-extraction-ai.md)). The values are a starting point for H-06 / EXP-005; the step rests on A05's provisional brightness mapping (0.25 = one octave of cutoff). | NUMISONA |

### G. Technique modes

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-31 | DEFAULT | Audification computes the speed-up k from the data band to a target audio band and shows k, duration T/k and the audible band. It resamples (no pitch-shift). The seismic preset is k ≈ 2200. | Dombois 2001; Walker & Nees 2011 (generic k computation is inferred) | CURVASONA |
| SDR-32 | DEFAULT | Granular mode for distributions and uncertainty uses 80–300 ms grains, density-weighted, with 100–1000 grains active. | Williamson & Murray-Smith 2005 | TONALENS, RESONUM |
| SDR-33 | INFER | Model-based sonification for high-dimensional and graph data is excitation-driven (data sonogram, crystallization). It is silent without interaction. | Hermann 2008; Hermann & Ritter 2002 (application to graphs is inferred) | RESONUM |
| SDR-34 | DEFAULT | The high-dimensional quick-look preset maps PCA components to harmonic partial amplitudes. | Dalziel & Lieck 2025 | TONALENS |
| SDR-35 | SHOULD | Monitoring mode uses a continuous ambient soundscape with event-driven salience changes. Urgency cues are reserved for true alarms. | Hildebrandt et al. 2016; Debashi & Vickers 2018; Edworthy et al. 1991 | SONOTRACE |

### H. Aesthetics and users

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-36 | MUST | Analysis, Presentation and Monitoring modes are explicit and recorded in the spec. | Neuhoff 2019; Hermann 2008 | Spec |
| SDR-37 | SHOULD | A built-in interactive tutorial of 10–20 minutes is provided. About 20 minutes of training halved error. | Walker & Nees 2011; Smith & Walker 2005 | Desktop app |
| SDR-38 | MUST | Users can personalize timbre, discrete or continuous marks, polarity, range and rate. BLV users are a primary design population. | Sharif et al. 2022; Walker & Mauney 2010 | Profiles |
| SDR-39 | INFER | A psychoacoustic comfort budget limits sustained sharpness and roughness over long sessions. | Fastl & Zwicker 2007; Hildebrandt et al. 2016 | ECHOMETRIC lint |

### I. Evaluation

| ID | Strength | Rule | Evidence | Owner |
|---|---|---|---|---|
| SDR-40 | MUST | Evaluations use standard tasks: point estimation (RMS), trend ID, extremum, outlier and signal detection by SNR, discrimination thresholds. They report response time, replays, NASA-TLX, SUS and pleasantness/clarity, include BLV participants, and have a training arm. | Walker & Nees 2011; Nees & Walker 2008; Tucker Brown et al. 2022; Elmquist et al. 2024 | Research track |
| SDR-41 | MUST | Any new data concept (for example graph centrality or edge weight) gets polarity and scaling from magnitude estimation before it ships as a default. | Walker 2002; Ferguson & Brewster 2018 | Research track |
| SDR-42 | MUST | Stimuli, specs and analysis code are published. Gamified remote collection may be used to reach scale. | Hermann 2008; Ziemer 2024 | Research track |

---

## A11Y: Accessibility Requirements

Requirements A11Y-01 to A11Y-34 keep the numbering of [notes/03-accessibility-hci.md](notes/03-accessibility-hci.md) §7. Standard clauses refer to WCAG 2.2 (via WCAG2ICT), Section 508 and EN 301 549. Clause numbers marked [CHECK] in the notes must be re-verified before they are quoted in a conformance report.

| ID | Strength | Requirement (summary) | Basis | Owner |
|---|---|---|---|---|
| A11Y-01 | MUST | Every chart view is a named, programmatically identifiable object with an auto-generated summary. It is never announced as an unlabeled "graphic". | WCAG 1.1.1, 4.1.2; 508 502.3.1; Sharif et al. 2021 | AUDAXIS, UI |
| A11Y-02 | MUST | Summaries default to statistics and trends (Lundgard L2/L3). Interpretation (L4) is optional and labelled. | Lundgard & Satyanarayan 2021 | ECHOMETRIC descriptions |
| A11Y-03 | MUST | An accessible data-table view with exposed row and column headers is always available. | WCAG 1.3.1; 508 502.3.3; APG Grid | UI |
| A11Y-04 | MUST | A hierarchical, keyboard-navigable structure supports local, lateral, spatial and targeted moves. | Zong et al. 2022; Chart Reader; Data Navigator; APG Tree | AUDAXIS |
| A11Y-05 | MUST | Everything is keyboard-operable, with no traps, a logical focus order, and visible, unobscured focus. | WCAG 2.1.1, 2.1.2, 2.4.3, 2.4.7, 2.4.11 | UI |
| A11Y-06 | MUST | Single-character shortcuts can be turned off or remapped, or are chart-focus-scoped. They avoid screen-reader modifier combinations. | WCAG 2.1.4 | UI, keymap |
| A11Y-07 | SHOULD | Landmark jump keys and a "where am I" command are provided. | Chart Reader; Zong et al. 2022 | AUDAXIS |
| A11Y-08 | SHOULD | Distinct earcons mark boundaries, invalid actions, landmarks and threshold crossings, each with a text equivalent. | Chart Reader; Apple Audio Graphs; WCAG 1.1.1 | AUDAXIS |
| A11Y-09 | MUST | Overview-then-detail: one command plays the overview, and the user can pause, scrub and navigate. The current value is announced on pause. | Sharif et al. 2021; Chart Reader; MAIDR | AUDAXIS, SONOTRACE |
| A11Y-10 | MUST | Toggleable output modalities (speech/text, sonification, braille, visual, haptics) share one cursor. Settings persist per user. | MAIDR; Umwelt | AUDAXIS, UI |
| A11Y-11 | SHOULD | Selection is linked across modalities. | Umwelt | TONALENS, UI |
| A11Y-12 | MUST | Verbosity (terse and verbose) and the order of announcement parts are configurable. | Zong et al. 2022; MAIDR; Chart Reader | AUDAXIS |
| A11Y-13 | SHOULD | Default x→time and y→pitch, with redundant duration or tap-count encodings for precise values. Polarity is configurable per dimension. | Wang et al. 2022; Walker & Mauney 2010; Apple Audio Graphs | NUMISONA |
| A11Y-14 | MUST | Essential information is never carried only by pan, pitch extremes or timbre. It must survive mono output, NVDA sound split and high-frequency hearing loss. | WCAG 1.3.3; NVDA sound split; MAIDR | NUMISONA, lint |
| A11Y-15 | MUST | Sonification volume is independent of system volume. Tempo, range and instrument controls are provided. There is no autoplay longer than 3 s, and a global "stop all audio" command exists. | WCAG 1.4.2 | Audio, UI |
| A11Y-16 | MUST | Streams and announcements can be paused, stopped or hidden, and live announcements are rate-limited. | WCAG 2.2.2, 4.1.3 | SONOTRACE |
| A11Y-17 | MUST | Status is announced through platform mechanisms without stealing focus, and stale announcements are superseded. | WCAG 4.1.3; 508 502.3.14; UIA Notification; NSAccessibility | Desktop a11y bridge |
| A11Y-18 | MUST | The accessibility tree has correct role, name, value, state and bounds for every custom control and chart node. In the Tauri app the tree is exposed through the system webview (WebView2 → UIA, WKWebView → NSAccessibility, WebKitGTK → AT-SPI) using correct ARIA. AccessKit's adapters apply only to any natively rendered UI. | WCAG 4.1.2; 508 502; EN 301 549 11.5 [CHECK] | UI |
| A11Y-19 | SHOULD | Speech and braille go through the user's screen reader by default. Optional self-voicing is available. | 508 503.3; MAIDR; Chart Reader | AUDAXIS speech |
| A11Y-20 | SHOULD | Speech and sonification are sequenced, not overlapped. Any overlap keeps sonification ~20 dB below speech. Recommended NVDA settings are documented. | WCAG 1.4.7 (AAA, as guidance); Chart Reader | AUDAXIS, audio |
| A11Y-21 | SHOULD | Braille output includes a compact sparkline encoding and a persistent review line. | MAIDR | AUDAXIS |
| A11Y-22 | SHOULD | Query commands cover extrema, mean, median, mode, SD, sum, ranges and value-at-x, with optional natural-language Q&A. | VoxLens; MAIDR | ECHOMETRIC |
| A11Y-23 | SHOULD | Insights and annotations are navigable, and audio data narratives are linked to data ranges. | Chart Reader; Siu et al. 2022 | ECHOMETRIC, AUDAXIS |
| A11Y-24 | MUST | Every sonified insight has a non-audio equivalent (text, visual or braille). | EN 301 549 cl. 4 [CHECK]; 508 302.4/302.5; WCAG 1.1.1 | UI |
| A11Y-25 | MUST | Platform preferences are respected: contrast themes, text scaling, reduced motion, non-color encodings and magnifier-friendly focus. | 508 503.2; WCAG 1.4.1, 1.4.3, 1.4.4, 1.4.11 | UI |
| A11Y-26 | MUST | Drag-scrubbing and pan/zoom have single-pointer and keyboard alternatives, and targets meet the minimum size. | WCAG 2.5.7, 2.5.8 | UI |
| A11Y-27 | SHOULD | Navigation is built on a discrete-step model, so switch, braille routing keys, gamepad and voice commands can drive it. | Data Navigator; DIXTRAL; VoxLens | AUDAXIS |
| A11Y-28 | SHOULD | Function-plot STEM features include derivative and sign cues, zero and asymptote markers, and undefined and off-screen cues. | SonicFunction; AudioFunctions 2.0 | CURVASONA |
| A11Y-29 | MUST | The authoring and configuration UI is itself accessible. | EN 301 549 11.8 [CHECK]; 508 504; Umwelt | UI |
| A11Y-30 | SHOULD | An accessible built-in tutorial and context help are provided (H/F1, in a consistent location). | MAIDR; WCAG 3.2.6 | Desktop app |
| A11Y-31 | MUST | The project targets WCAG 2.2 AA via WCAG2ICT and EN 301 549 v4.1.1, documents v3.2.1 and Section 508, and publishes an ACR per release. | WCAG2ICT; ISO/IEC 40500:2025 | Release process |
| A11Y-32 | MUST | Each release is tested with NVDA, JAWS, Narrator, VoiceOver, and Orca (X11 and Wayland), including NVDA ducking and sound split, and with one braille display. | WebAIM #10 | QA |
| A11Y-33 | SHOULD | Visual mode is audited against Chartability's 14 critical tests each release and its 50 heuristics at majors. | Chartability | QA |
| A11Y-34 | MUST | BLV people are paid co-designers and evaluators: about 10–15 per round, on their own AT, compensated at about $50–85/h. | Mack et al. 2021/2022; Chart Reader; Zong et al. | Research track |

---

## Change log

| Date | Change | By |
|---|---|---|
| 2026-10-18 | Baseline created from kickoff research (SDR-01..42, A11Y-01..34). | Kickoff |
| 2026-10-18 | A11Y-18 clarified for the webview shell chosen in ADR-0002 (tree exposed via the system webview; AccessKit only for native UI). No change to the requirement itself. | Kickoff review |
| 2026-10-31 | SDR-43 added (`INFER`, experimental, off by default): low-confidence timbre for estimated values, behind `experimental.lowConfidenceTimbre` (RFC-0006, ADR-0027, C02-T09b). No default changes. Evidence to come from H-06 / EXP-005. | C02-T09b |
