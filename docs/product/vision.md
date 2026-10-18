# Product Vision

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> **Evidence:** [research report](../research/research-report.md). **Requirements:** [requirements.md](requirements.md). **Engines:** [engines.md](engines.md).

## 1. One-line description

**SONARITHM is an interactive, reproducible framework for exploring graphs, charts and multivariate data through structured sound.**

*Hear the patterns.*

## 2. The problem

1. **Charts are the default interface to data, and they exclude people.** In a controlled study, screen-reader users extracted information from online charts 61% less accurately and took 211% longer. A third of the charts sampled could not be found by screen readers at all (Sharif et al., ASSETS 2021).
2. **Sight is not always available or best.** Monitoring tasks, eyes-busy work, classrooms and outreach all benefit from an auditory channel. The evidence is strongest for trends, overviews and peripheral monitoring (Hildebrandt et al. 2016; Debashi & Vickers 2018).
3. **The tools are fragmented and fragile.** As of October 2026, no maintained native cross-platform sonification application exists. The desktop tools are abandoned (Sonification Sandbox 2014, xSonify 2018, MathTrax ≈2012), and the active tools are web libraries or notebooks. No tool combines screen-reader-grade navigation, a declarative spec, live streaming, chart-image ingestion and reproducible export ([notes/01](../research/notes/01-tools-landscape.md)).
4. **The science is under-used and under-tested.** Evidence-backed defaults exist (pitch for value, 400–1400 Hz, 2–4 points/s, dynamic reference tones). Yet only 6.1% of published mappings were ever evaluated (Dubus & Bresin 2013), and designer intuition is a poor guide (Walker & Kramer 2005).

## 3. The vision

A person who receives a chart, a spreadsheet, a function or a live data feed can **open it, hear its shape in seconds, explore it point by point with the keyboard, ask it questions, and export a reproducible sonification**, whether or not they can see the screen.

A researcher can **specify a sonification declaratively, render identical stimuli on any machine, and run a study**, citing a DOI and a spec hash.

## 4. Positioning

SONARITHM is **trustworthy sonification infrastructure**: one deterministic core that powers a desktop application, a CLI, a Python package and (later) the web. It complements the web accessibility libraries (MAIDR, Chart2Music, Highcharts, Erie, Umwelt) rather than competing with them. It imports their formats and provides what they lack: native performance, reproducible export, streaming, chart extraction, and maintained model-based and multivariate engines.

| We are | We are not |
|---|---|
| A sonification framework with a professional desktop app | A DAW, a synthesizer or a music-production tool |
| Evidence-backed, with defaults traceable to research | A source of "sublime" data music without analytic grounding |
| Accessible by construction (BLV users are a primary population) | An accessibility overlay bolted onto a visual tool |
| Offline-first and private | A cloud service |
| Reproducible: every render has a spec and a manifest | A black box |
| Complementary infrastructure that imports Erie, Vega-Lite and MAIDR | A walled garden |

## 5. Who it is for

| Persona | Context | Jobs to be done | Key engines |
|---|---|---|---|
| **BLV analyst or student** (screen-reader user) | Receives charts and spreadsheets at work or school | Get an overview fast; find extremes and trends; check exact values; work independently of sighted help | AUDAXIS, NUMISONA, PLOTSONA, ECHOMETRIC |
| **Scientist or analyst** (sighted) | Large time series, multivariate data, monitoring | Notice patterns and anomalies in parallel with visuals; monitor streams eyes-free | SONOTRACE, TONALENS, ECHOMETRIC |
| **Educator / STEM learner** | Maths and physics classrooms, including BLV learners | Hear functions, slopes, zeros and asymptotes; teach graphing inclusively | CURVASONA, AUDAXIS |
| **Sonification / HCI researcher** | Designing and evaluating auditory displays | Specify designs precisely, generate stimuli, run studies, publish reproducibly | ECHOMETRIC harness, NUMISONA, RESONUM |
| **Developer / data scientist** | Python notebooks, pipelines | Add sonification to analyses and products with a few lines | CLI, Python, facade |
| **Journalist / sound designer / communicator** | Data stories, outreach | Make sonifications that are both pleasant and honest; export audio and MIDI | AXONOTE, NUMISONA |

## 6. Product principles

1. **Hear first, then explore, then read** (overview → control → data).
2. **Every default has a reason**, and every reason has a citation.
3. **Never hide uncertainty.** Estimates sound, read and look like estimates.
4. **The keyboard is the primary input.** The mouse is a convenience.
5. **Instrument, not toy.** The UI and sound are calibrated, precise and calm.
6. **Reproducible or it didn't happen.** A spec and a manifest accompany every render.
7. **Analysis and presentation are different goals.** Modes keep them honest (Neuhoff 2019).

## 7. Tagline discipline

"**Hear the patterns**" is supported by evidence for trends, overviews and monitoring. "**Discover what the graphs reveal**" is an aspiration. In studies, subtle signals at low signal-to-noise ratio went undetected by ear (Tucker Brown et al. 2022), and no evidence exists yet for sonifying network graphs. Public materials therefore use *Hear the patterns* as the primary tagline. The longer form is used only where it is framed as a research goal that SONARITHM's studies (P03–P08) are designed to test.

## 8. Success measures

SONARITHM collects **no telemetry** ([ADR-0012](../decisions/0012-privacy-and-telemetry.md)). Success is measured through studies, releases and public signals:

| Measure | Target | When |
|---|---|---|
| BLV participants complete core tasks unaided (co-design round) | ≥ 80% of attempts | Gate B, Gate C |
| SUS score in co-design rounds (BLV and sighted) | ≥ 75 | Gate C |
| Time from install to first sound (new users, observed) | ≤ 5 minutes | A07, B07 |
| Reproducibility of the replication gallery across OSes | 100% Score-identical | Every release |
| Accessibility conformance | WCAG 2.2 AA ACR published, no critical open issues | Every UI release |
| Research output | ICAD 2027 submission; JOSS acceptance; ≥ 2 studies by Gate D | Per publication plan |
| Adoption signals | PyPI and crates.io downloads, citations of the Zenodo DOI, external contributors | Reviewed at gates |
| Sustainability | ≥ 2 active maintainers by Gate C; no critical dependency unmaintained for more than 6 months | Gate C |

## 9. Non-goals (for 1.0)

- Hosting VST/CLAP plugins or being a plugin itself (licensing and scope).
- Cloud accounts, collaboration servers or SaaS.
- Sonifying arbitrary network graphs as a *default* feature before the graph-concept polarity study (H-05) reports.
- Automatic interpretation (L4 descriptions) without an explicit user request.
- Mobile apps (Tauri mobile is possible later; not planned before 2.0).
