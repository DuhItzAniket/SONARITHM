# Related work

> **Status:** Baseline v1 (2026-10-18). **Owner:** Research lead.
> **Related:** [references.bib](../research/references.bib) · [research report](../research/research-report.md) · [notes/01](../research/notes/01-tools-landscape.md) · [notes/02](../research/notes/02-theory-perception.md) · [notes/03](../research/notes/03-accessibility-hci.md) · [notes/06](../research/notes/06-chart-extraction-ai.md) · [contributions](contributions.md)

Paper-ready annotations, grouped by theme. Every key matches an entry in [references.bib](../research/references.bib). Statements summarize the kickoff notes; check the primary source before quoting. The "How SONARITHM differs" column states design intent. It becomes a claim only when [contributions.md](contributions.md) marks the matching claim as supported.

Entries marked *(preprint)* are not peer-reviewed. Entries whose bibliographic record is incomplete in the kickoff notes (bracketed or system-name titles in the bib file) must be completed from the primary source before submission.

## Definitions and taxonomy

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `kramer1999sonification` | Purpose-based definition: "the transformation of data relations into perceived relations in an acoustic signal". | Standard definition for introductions. | Adopts it alongside Hermann's method-based criteria. |
| `hermann2008taxonomy` | Method-based definition: objectivity, systematicity, reproducibility, reusability; closed-loop taxonomy; "data-inspired music" for art that breaks the rules. | Its criteria become SDR-01..SDR-03 and SDR-06; "structurally identical" output motivates the determinism work. | Makes the criteria operational: a serializable spec, seeded and declared randomness, a manifest with every render. |
| `walker2011theory` | Handbook chapter on theory: functions, techniques, data types, tasks, training, limiting factors. | Source for tasks, training (~20 min halves error) and channel choices. | Turns its guidance into versioned defaults with rule IDs. |
| `enge2023unified` | Unified sonification/visualization vocabulary: temporal substrate, auditory marks, auditory channels. | SDR-04: the spec's public vocabulary. | Uses the vocabulary as the API of a compiler and render engine, shared with a visual layer. |
| `enge2024star` | EuroVis 2024 state-of-the-art report classifying 57 audiovisual-display papers; finds "individual teams without many interconnections". | Frames the fragmentation problem and the audiovisual design space. | Offers shared infrastructure and an open benchmark across teams. |
| `grond2014interactive` | Listening modes × display purposes as a source of interactive-sonification guidelines. | Background for explicit modes (SDR-36). | Records the mode (Analysis, Presentation, Monitoring) in every spec. |
| `nees2019eight` | Eight components of a design theory of sonification; the field is "hindered by a lack of theory". | Motivates testable propositions and expository instantiations. | Supplies an instantiation (the toolkit) and testable propositions (H-01..H-12). |
| `lindborg2025trends` | Data Sonification Archive, 470 projects: engagement goals have grown at the expense of art. | Context for Presentation mode and public use. | Separates engagement-oriented Presentation from Analysis. |

## Mapping design and psychoacoustics

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `dubus2013systematic` | 495 mappings from 179 publications: pitch dominates; spatial channels serve kinematic data; only 6.1% of mappings evaluated. | Default channel choice (SDR-07, SDR-30) and the evaluation gap (F-002). | Evaluates its own defaults and publishes the harness. |
| `walker2002magnitude` | Magnitude estimation as a design tool; majority-polarity heuristic predicted 7 of 10 mappings. | Method for SDR-41 and H-05. | Applies the method to graph concepts, with BLV and sighted listeners. |
| `walker2007consistency` | Polarity and scaling stable across a direct replication (209 + 226 listeners) but specific to sound and data dimensions. | Concept table defaults (SDR-12..SDR-14). | Ships the table as flippable presets, not fixed rules. |
| `walker2010universal` | Visually impaired and sighted listeners broadly agree on polarity, with notable exceptions (dollars). | Population-specific defaults; BLV profiles. | Tests polarity with both populations for every new concept. |
| `walker2005mappings` | Designer intuition misleads: the "Bad" ensemble was fastest, the "Intuitive" among the poorest. | SDR-15 (one-click flip); motivation for H-07. | Requires evidence, not intuition, for defaults. |
| `neuhoff2002pitch` | Pitch and loudness interact; loudness increases are overestimated. | SDR-23: loudness never carries precise values. | Lints loudness-as-value mappings. |
| `fitzpatrick2023frequency` | 400–1400 Hz as good as 65–1480 Hz on line charts (n = 48); amplitude compensation had no effect. | SDR-08 default range; SDR-24 caveat. | Tests non-inferiority separately for BLV listeners (H-03). |
| `nees2008density` | Density (1–8 points/s) has a small effect; trend reversals a large one. | SDR-19, SDR-21; GEN-001. | Extends tests above 8 points/s (H-02). |
| `wang2022seeing` | BLV listeners rate pitch most intuitive; tap counts and duration most accurate. | Redundant duration and tap-count encodings (A11Y-13). | Offers them as configurable redundant channels. |
| `ferguson2018perceptual` | Polarity and scaling of roughness, noise and pitch depend on the data:sound mapping. | Psychoacoustic channels need their own priors. | Treats every channel-concept pair as needing evidence. |
| `mcgookin2004understanding` | Concurrent-earcon identification falls with number; timbre and onset differences partly help. | SDR-26 and H-04. | Measures the limit for data streams rather than earcons. |
| `nees2007listener` | Conceptual model of auditory-graph comprehension: listener, task, display. Redundant spatial coding hurt. | Organizes study variables; SDR-27. | Logs listener, task and display variables in every study. |
| `plomp1965tonal` | Pure-tone pairs within a critical band sound rough; beyond one band, consonant. | Basis of SDR-28 (INFER). | Tests the threshold for data tones (H-11). |
| `glasberg1990derivation` | ERB formulas. | Pitch units and spacing lint (SDR-09, SDR-28). | Exposes ERB-rate as a pitch unit. |
| `fastl2007psychoacoustics` | Models of sharpness, roughness, fluctuation strength and pleasantness. | Basis of the comfort budget (SDR-39, H-12). | Computes descriptors from renders to set and test thresholds. |
| `london2012hearing` | Metrical grouping floor at about 100 ms inter-onset interval. | Basis of SDR-20's ~10 events/s inference. | Tests the inference (H-02). |
| `iso226_2023` | Normal equal-loudness-level contours (2023 edition). | SDR-24 compensation for consistent salience. | Applies it for perceptual consistency without claiming accuracy gains. |

## Context cues and speech

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `smith2005effects` | Dynamic y-axis reference tone cut RMS error 18.09 → 12.14; static reference did not help; clicks aid timing for discrete tones; context and training not additive (n = 160). | SDR-16, SDR-17; GEN-002; H-01. | Replicates it with BLV listeners and makes the cues reusable spec features. |
| `siu2022audio` | Audio data narratives interleaving description and sonification; 16 screen-reader users gained more insights. | SDR-18; A11Y-23. | Links narratives to data ranges in the navigation tree. |
| `holloway2022infosonics` | Infosonics: spoken introduction and annotation plus sonification for blind adults. | Hybrid speech and sound designs. | Generates the speech parts deterministically from data (L1/L2). |
| `zhao2024speechtone` | SpeechTone: a speech-based mark extending Erie. | Speech as a mark type. | Routes speech to the user's screen reader by default and sequences it with tones (A11Y-19, A11Y-20). |
| `kim2024erie` | Erie's auditory legends and Web Speech elements. | Legends before playback (SDR-18). | Same idea in a native, exportable engine. |

## Accessible visualization and navigation

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `sharif2021understanding` | Screen-reader users 61% less accurate, 211% more time; a third of charts undiscoverable; "flyover" then drill-down. | Baseline problem (F-005); A11Y-01, A11Y-03, A11Y-09. | Builds the overview-then-detail flow into a native app. |
| `lundgard2022accessible` | Four-level model of chart descriptions; blind readers value L2/L3 and dislike editorializing. | A11Y-02; description pipeline. | Computes L1/L2 deterministically; L3 phrased only from verified facts. |
| `zong2022rich` | Structure, navigation and description as design dimensions; six-month co-design with a blind co-author. | A11Y-04, A11Y-12; co-design model. | Implements the dimensions over sound as well as speech. |
| `blanco2022olli` | Olli: charts as keyboard-navigable ARIA trees. | Tree navigation pattern. | Native accessibility tree via the webview, plus sonification on the same cursor. |
| `thompson2023chartreader` | Chart Reader: parallel co-design with 10 users; five-branch hierarchy; boundary and invalid-action earcons; autonomy as the core value. | A11Y-07, A11Y-08; co-design method. | Screen-reader-agnostic engine with export and streaming; Chart Reader is archived. |
| `seo2024maidr` | MAIDR: braille, text, sonification and review modes on one cursor; SUS 70–81 with 11 blind users. | A11Y-10, A11Y-21. | Permissive licence, native rendering, audio export, deterministic specs. |
| `zong2024umwelt` | Umwelt: visualization, sonification and text as coequal views of one data model. | A11Y-11 linked selection. | Adds native rendering, export and reproducible specs. |
| `elavsky2023datanavigator` | Data Navigator: node-edge navigation structures for many input modalities. | A11Y-27; closest analogue for network navigation (F-008). | Applies node-edge navigation to sonified networks, pending evidence. |
| `elavsky2026skeleton` | Skeleton: authoring tool for non-visual navigation graphs *(accepted at IEEE VIS 2026)*. | Authoring of navigation structures. | Generates the navigation tree from the spec. |
| `elavsky2022chartability` | Chartability: 50 heuristics and 14 critical tests for visualization accessibility. | A11Y-33 audits. | Used as an audit tool, not as research evidence. |
| `sharif2022voxlens` | VoxLens: Q&A, summary and sonification modes; +122% accuracy against a non-concurrent comparison group. | A11Y-22 query commands. | Concurrent controls in its own studies. |
| `sharif2022userfriendly` | Screen-reader users prefer discrete square-wave tones and want personalization. | SDR-29 BLV preset; SDR-38. | Ships the preset and per-user profiles. |
| `kim2021accessible` | Design space of accessible visualization. | Framing for accessibility papers. | — |
| `langston_chart2music` | Chart2Music: the richest keyboard model in the field; blind developers on the team; no peer-reviewed evaluation. | Keyboard model reference. | Desktop, CLI and Python, with export and evaluation. |
| `apple_audiographs` | Apple Audio Graphs: standard chart descriptor; x→time, y→pitch, extra axis→duration, category→timbre. | Mapping conventions; importer target. | Cross-platform, not tied to VoiceOver. |
| `khalaila2024arent` | Tactile charts read fast and accurately, but users want purpose-built encodings *(preprint)*. | Supports encodings designed for BLV users. | — |
| `kamath2025dixtral` | DIXTRAL: non-visual access to 3D surface and point data, gamepad navigation *(preprint, very small sample)*. | Discrete-step input beyond keyboard (A11Y-27). | — |

## Declarative grammars and toolkits

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `kim2024erie` | Erie: the only declarative sonification grammar; compiles to Web Audio; replicates prior prototypes; interaction and streaming are future work. | SDR-05 importer; gallery items; F-007. | Native and offline rendering, interaction, streaming, audio and MIDI export, provenance manifests. |
| `zong2024umwelt` | Shared data model across modalities. | Spec design. | Spec compiles to a Score IR shared by audio, navigation and visuals. |
| `trayford2023strauss` | STRAUSS: flexible Python sonification package. | Offline multichannel rendering reference. | Interactive, accessible and deterministic across front ends. |
| `trayford2025strauss` | STRAUSS JOSS paper. | Model for P02. | — |
| `reinsch2023sonecules` | Sonecules: reusable "design-once-use-many" sonification designs. | SDR-03 templates. | Maintained successor role for reusable designs; the Bielefeld stack is quiet. |
| `born2025pyagen` | Pya AGen audio generators for Python. | Related toolkit. | — |
| `cantrell2021highcharts` | Highcharts Sonification Studio: online, open-source, accessible tool on proprietary Highcharts. | Context tracks and MIDI export as prior art. | Permissive licence; not tied to one charting library. |
| `peng2023siren` | Siren: creative, extensible web sonification. | Web toolkit landscape. | Native core with web build later. |
| `poret2023ossia` | Sonification cell in Ossia Score. | Generic toolkit approach. | — |
| `laczko2025pixasonics` | Pixasonics: image sonification toolbox for Python. | Related toolkit. | — |
| `kantan2024mixing` | Multitrack-mixer metaphor for PMSon design (prototype, not user-tested). | Inspector design ideas. | — |
| `budziszewski2025tools` | History of sonification tools since 1990; sustainability is a central challenge; ML agenda. | Motivates one shared compiler (ADR on engines as capabilities). | Designed against feature sprawl and abandonment. |

## Audification, model-based and granular sonification

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `dombois2001using` | Seismic audification at 2,200× by resampling; near quakes click, distant ones rumble. | SDR-31 audification calculator; gallery item. | Computes the speed-up from a target band and shows k, T/k and the band. |
| `speeth1961seismometer` | Early seismometer audification. | Historical basis for audification. | — |
| `hermann2002crystallization` | Crystallization sonification of high-dimensional data. | RESONUM model (SDR-33). | Maintained, excitation-driven models with reusable templates. |
| `hermann2011model` | Model-based sonification handbook chapter. | RESONUM foundations. | — |
| `hermann1999listen` | Data sonograms and particle trajectory sonification. | RESONUM models. | — |
| `williamson2005sonification` | Granular synthesis for probabilistic feedback: 80–300 ms grains, 100–1,000 active. | SDR-32; granular voice. | Granular marks in the same spec and engine. |
| `dalziel2025harmonic` | PCA components mapped to harmonic partials. | SDR-34 quick-look preset. | — |
| `linke2024somson` | SOMson: sonification of Kohonen maps. | Multivariate exploration. | — |
| `helmuth2024pulsars` | Pulsar parameters mapped to granular synthesis. | Granular parameter mapping. | — |

## Monitoring

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `hildebrandt2016continuous` | Continuous soundscape beat visual-only and alerts in peripheral monitoring (n = 18) without hurting the primary task. | SDR-35; H-09 replication. | Monitoring mode as a reusable spec preset. |
| `debashi2018sonification` | SoNSTAR network monitoring: higher situational awareness, lower NASA-TLX. | SDR-35. | — |
| `watson2004sonification` | Respiratory sonification supports eyes-free monitoring and time-sharing. | Clinical precedent. | — |
| `anderson2009sonification` | Sonification design for complex work domains: dimensions and distractors. | Monitoring design. | — |
| `gaver1991effective` | ARKola: auditory icons in a complex system. | Classic peripheral-awareness precedent. | — |
| `mynatt1998designing` | Audio Aura: background awareness. | Ambient design precedent. | — |
| `vickers2011process` | Handbook chapter on process monitoring. | Background. | — |
| `edworthy1991improving` | Perceived urgency rises with pitch and rate. | Reserve urgency cues for true alarms (SDR-35). | — |

## Chart extraction and AI

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `masson2023chartdetective` | Vector-PDF extraction with 78% lower relative error than raster approaches, under 4 minutes per chart. | Exact-before-estimated ingestion (C01). | Automatic calibration from tick text; provenance per point. |
| `rohatgi_webplotdigitizer` | Calibrate, mask, extract, correct: the reference semi-automatic workflow. | C02 workflow, re-implemented from documentation (AGPL code not used). | Accessible point entry and provenance labels. |
| `drevon2017webplotdigitizer` | High intercoder reliability for manual WebPlotDigitizer extraction. | Human-calibrated digitizing is reliable. | — |
| `deplot2023` | Plot-to-table then LLM; RMS-F1 metric introduced. | Metric for C01/C02/D04; table-first Q&A. | Table-first, verified answers. |
| `onechart2024` | Small model with a self-consistency confidence score; filtering raised precision 72.02 → 81.97. | Uncertainty signal for H-06. | Uses disagreement as per-point confidence. |
| `tinychart2024` | Program-of-Thought for numeric answers; high in-distribution scores. | Q&A via generated queries. | — |
| `lineformer2023` | Line-chart extraction as instance segmentation. | Candidate mark finder for line charts. | ML finds marks; classical calibration maps values. |
| `berkane2026wbchartextract` | Line-chart RMS-F1 18–39 for most models; self-ensembling helps *(preprint)*. | F-006; ML output is a draft. | Labels every ML estimate. |
| `epicurvebench2026` | Best model 52.3% ECS vs 90.9% human agreement on epidemic curves *(preprint)*. | Realistic difficulty; temporal metric. | Combines RMS-F1 with curve-distance metrics. |
| `song2026fairchart2table` | Y-axis tick-format biases; axis hints help *(preprint)*. | Supports classical axis calibration first. | Calibrates axes before ML. |
| `charthal2025` | Severe chart hallucination even in frontier models *(preprint)*. | LLM numbers are claims to verify. | Checks every generated number against the table. |
| `chocolate2023` | Typology of factual errors in chart captions; entailment checker. | Description verification. | — |
| `charxiv2024` | Real arXiv charts; human 80.5% on reasoning questions. | Benchmark context. | — |
| `qwen3vl2025` | Open-weight vision-language model family, Apache-2.0 sizes. | Local model candidates for D04. | Local by default; cloud only per chart and opt-in. |
| `vlmsblind2026` | Behaviour of VLMs given subtly wrong captions *(preprint)*. | Risk that blind users cannot verify descriptions. | Deterministic L1/L2; verified L3. |
| `hommel2025ai` | AI in sonification "remains limited"; prototype on sc3nb/sonecules. | F-009; H-07. | Controlled comparison of LLM and rule-engine mappings. |
| `lundgard2022accessible` | Automate L1–L2; keep L4 optional. | Description pipeline. | — |

## Evaluation methodology

| Key | Contribution | Relevance to SONARITHM | How SONARITHM differs |
|---|---|---|---|
| `bonebright2011evaluation` | Handbook chapter on evaluating auditory displays. | Methods background. | — |
| `seica2020contrasts` | Evaluation practice in Audio Mostly and ICAD papers. | Evaluation gap. | Open harness and standard tasks. |
| `ziemer2024curat` | Gamified remote evaluation is reliable; interaction improves precision. | Remote study design. | — |
| `tuckerbrown2022evaluating` | Signal detection by SNR, experts vs non-experts, 192 participants; chance at SNR 3–5. | GEN-003; tagline discipline (F-010). | Reuses the protocol as a benchmark task. |
| `elmquist2024parallel` | Correlation discrimination (n = 35): visualization most sensitive; combining sonification with visualization raised enjoyability. | GEN-004 task; audio as a complement. | — |
| `zanella2022sonification` | Nearly 100 astronomy sonification projects; efficacy testing "extremely limited". | Evaluation gap. | — |
| `misdariis2023audible` | Audible Universe workshop on designing and evaluating astronomy sonification tools. | Community methods. | — |
| `hart1988nasatlx` | NASA-TLX workload. | Metric. | — |
| `brooke1996sus` | SUS usability scale. | Metric. | — |
| `ledo2018evaluation` | Analysis of 68 toolkit papers: match evaluation strategies to claims. | P01 and P05 evaluation design. | Demonstration (gallery), technical benchmarks and targeted usage. |
| `mack2021accessibility` | Accessibility research norms: median sample of 13 for disabled participant groups *(record incomplete)*. | Sample planning (A11Y-34). | — |
| `mack2022anticipate` | Anticipate and adjust access across communication, materials, space and time. | Study accessibility. | — |
| `neuhoff2019doomed` | Adoption failed; mixing art and science is a root cause; pick a goal. | Modes; tagline discipline. | Explicit modes recorded in every spec. |
| `supper2014sublime` | Public appeal of sonification rests on promises of sublime experience. | Caution against overclaiming. | Claims gated by [contributions.md](contributions.md). |
