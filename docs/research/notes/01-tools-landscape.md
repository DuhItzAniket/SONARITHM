# Data-Sonification Tools Landscape: Prior Art and Competitive Analysis (as of 18 October 2026)

Method note: I checked repository status, licenses, and latest release and commit dates directly against the GitHub REST API (`gh api`), the npm registry, PyPI's JSON API and CRAN on **2026-10-18**. "Last commit" means the newest commit on the default branch. "Pushed" can be later, because it also counts pushes to other branches. Everything else comes from official docs, papers or reputable press, and is cited inline. Where I could not verify something it is marked "?" or listed under Gaps.

Status legend used throughout: **ACTIVE** = commit or release in the last 6 months; **SLOW** = last activity 6–24 months ago; **DORMANT** = no activity for more than 24 months but still available; **ARCHIVED / DISCONTINUED** = explicitly archived or paused, or the site or distribution is gone.

---

## 1. Which tools exist across commercial, open-source, academic and accessibility-focused categories, and what is their current status?

### Takeaway
The active centre of the field in 2026 is **web/JavaScript accessibility libraries**: MAIDR, Chart2Music, the Highcharts sonification module, Erie, and Umwelt/Olli. They sit alongside **Python science/astronomy toolkits** (STRAUSS, Astronify) and **general audio environments** (SuperCollider, Pd, Sonic Pi, Tone.js, pyo). Almost every **native desktop sonification application** is legacy or stale: Sonification Sandbox (Java, 2014), xSonify (Java, 2018), MathTrax (stalled around 2012), SonoUno desktop (Python/wx, last commit Jan 2024). TwoTone, a well-known no-code web tool, has been paused since mid-2023.

### Cited Findings

**A. Commercial / corporate products**
- **Highcharts sonification module** (Highsoft): a separate `sonification.js` module that adds `Chart.sonify`, `Chart.toggleSonify`, `Series.sonify` and `Point.sonify` — [Highcharts docs](https://www.highcharts.com/docs/sonification/getting-started). Highcharts' latest tag is v13.1.1; the npm `highcharts` 13.1.1 was published 2026-09-25 and the last commit was 2026-10-16. **ACTIVE** — [GitHub highcharts/highcharts](https://github.com/highcharts/highcharts), [npm](https://www.npmjs.com/package/highcharts). Highcharts is proprietary, and "a suitable Highcharts license is required for all commercial use" — [HSS README](https://github.com/highcharts/sonification-studio).
- **Highcharts Sonification Studio (HSS)**: a web app from Highsoft "designed in collaboration with Georgia Institute of Technology", at https://sonification.highcharts.com. The tool itself is BSD-3-Clause but depends on proprietary Highcharts. Latest release v1.1.1 on 2025-11-06. **SLOW** (11 months) — [GitHub highcharts/sonification-studio](https://github.com/highcharts/sonification-studio).
- **SAS Graphics Accelerator** (SAS Institute): a free Chrome extension offering "enhanced visual rendering, text descriptions, tabular data, and interactive sonification" — [SAS product page](https://support.sas.com/software/products/graphics-accelerator/index.html). The Chrome Web Store lists version 5.12.2.01223, updated January 19, 2026. **Maintained** (last update about 9 months ago) — [Chrome Web Store](https://chromewebstore.google.com/detail/sas-graphics-accelerator/ockmipfaiiahknplinepcaogdillgoko). Generally available since February 2017 — [SAS press release](https://www.sas.com/da_dk/news/press-releases/2017/february/sas-graphics-accelerator-education.html).
- **Apple Audio Graphs** (`AXChart` / `AXChartDescriptor`): an OS-level VoiceOver feature, available on iOS 15.0, iPadOS 15.0, Mac Catalyst 15.0, macOS 12.0, tvOS 15.0, watchOS 8.0 and visionOS 1.0 — [Apple docs metadata](https://developer.apple.com/documentation/accessibility/axchartdescriptor), [Audio graphs](https://developer.apple.com/documentation/accessibility/audio-graphs). **ACTIVE** (shipping OS API).
- **Desmos Audio Trace**: a free graphing-calculator feature opened with Alt+T or Option+T — [Desmos Help Center](https://help.desmos.com/hc/en-us/articles/37064105800333-Audio-Trace). **ACTIVE**.
- **Manifest Audio "Sonification Tools v2"** (with Noah Pred): commercial Max for Live devices for Ableton Live, including the new 8-voice "Data Synth". Covered by CDM on Feb 13, 2025 — [CDM](https://cdm.link/sonification-tools-for-ableton-live-v2/), [FutureMusic, Jan 2025](https://futuremusic.com/2025/01/manifest-audio-unleash-data-synth-sonification-tools/). **ACTIVE as of 2025** (I did not check the current version).
- **Microsoft**: I found no native sonification in Excel. Search returned only research prototypes such as Abu Doush et al., ASSETS '09 (haptic plus audio for Excel charts) — [search summary of ACM paper](https://www.academia.edu/28733296/Making_Microsoft_Excel_). Microsoft Research's **Chart Reader** ("web-accessibility engine for rendering accessible, interactive charts optimized for screen reader users") takes CSV data, an insights JSON and a config JSON, and does not support missing values. Its README does not mention sonification. The repo is **ARCHIVED** (last push 2026-05-09) — [GitHub microsoft/chart-reader](https://github.com/microsoft/chart-reader).
- **Google**: I found no Google Sheets, Android TalkBack or ChromeVox chart-sonification feature (see Gaps). Google's main footprint is funding TwoTone through the Google News Initiative — [TwoTone README](https://github.com/kineteklabs/twotone).
- **Yahoo SongbirdCharts**: an Android library that "allows for other apps to render accessible audio charts", used in the Yahoo Finance Android app. Apache-2.0, last commit 2021-05-06. **DORMANT** — [GitHub yahoo/SongbirdCharts](https://github.com/yahoo/SongbirdCharts).

**B. Open-source web / JavaScript accessibility libraries**
- **Chart2Music** (Julianna Langston and contributors): MIT. Last commit 2026-09-22; npm 1.20.0 (registry last modified 2025-12-03). **ACTIVE** — [GitHub](https://github.com/julianna-langston/chart2music), [npm](https://www.npmjs.com/package/chart2music), site https://www.chart2music.com.
- **MAIDR** (xability org; docs at https://maidr.ai): GPL-3.0, rewritten in TypeScript. `maidr` v4.15.0 was released 2026-10-12 and the last commit was 2026-10-17. **ACTIVE** — [GitHub xability/maidr](https://github.com/xability/maidr). Siblings: **py-maidr** v1.27.0 (2026-10-14) — [GitHub](https://github.com/xability/py-maidr); **r-maidr** v0.5.0 (2026-09-29) — [GitHub](https://github.com/xability/r-maidr); an experimental **maidr-mcp** MCP server and a **maidr-skill** agent skill (both pushed 2026-10-14) — [maidr-mcp](https://github.com/xability/maidr-mcp), [maidr-skill](https://github.com/xability/maidr-skill). The old prototype is **DEPRECATED** — [xability/maidr-legacy](https://github.com/xability/maidr-legacy).
- **Erie** (Hyeok Kim, Yea-Seul Kim, Jessica Hullman; CHI '24, DOI 10.1145/3613904.3642442): a declarative grammar for data sonification — [arXiv 2402.00156](https://arxiv.org/abs/2402.00156). `erie-web` is MIT, migrated to TypeScript; npm 1.0.1 published 2026-09-19, repo pushed 2026-10-15. **ACTIVE** — [GitHub see-mike-out/erie-web](https://github.com/see-mike-out/erie-web), [docs](https://see-mike-out.github.io/erie-documentation), [editor/gallery](https://see-mike-out.github.io/erie-editor).
- **Umwelt** (MIT Visualization Group: Zong, Pedraza Pineros, Chen, Hajas, Satyanarayan; CHI 2024): an accessible authoring environment for multimodal data representations (visualization, sonification and text) — [arXiv 2403.00106](https://arxiv.org/abs/2403.00106), [MIT VIS](https://vis.mit.edu/pubs/umwelt/). It has moved to the `umwelt-data` org, last pushed 2026-08-10, with no license file detected. **ACTIVE / SLOW** — [GitHub umwelt-data/umwelt](https://github.com/umwelt-data/umwelt), [editor](https://umwelt-data.github.io/umwelt/editor/). The old `mitvis/umwelt` last committed 2025-12-24 — [GitHub](https://github.com/mitvis/umwelt).
- **Olli** (same group): BSD-3-Clause, npm `olli` 3.1.4 (2026-07-05), pushed 2026-10-16. It is a *text-structure* (screen-reader tree) library, **not a sonifier**, but Umwelt pairs it with sonification. **ACTIVE** — [GitHub umwelt-data/olli](https://github.com/umwelt-data/olli).
- **VoxLens** (Univ. of Washington, Ather Sharif et al.; CHI 2022): BSD-3-Clause, v4.0.0 released 2024-07-18; no commits since. **DORMANT** (27 months) — [GitHub athersharif/voxlens](https://github.com/athersharif/voxlens).
- **TwoTone** (Datavized Technologies, Google News Initiative, advisor Alberto Cairo): MPL-2.0. The original repo is **ARCHIVED** (last commit 2021-12-11) — [GitHub datavized/twotone](https://github.com/datavized/twotone). A continuation at kineteklabs says: "As of mid-2023, development on TwoTone has been paused", last commit 2024-07-27, with a MIDI-out beta at https://twotone-midiout-beta.netlify.app/ — [GitHub kineteklabs/twotone](https://github.com/kineteklabs/twotone). https://twotone.io still returned HTTP 200 on 2026-10-18 (my check). **PAUSED / DORMANT**.
- **Other JS**: *audiochart* (matatk), MIT, last commit 2019-06-07, **DORMANT** — [GitHub](https://github.com/matatk/audiochart). *Data-to-Music API* (Georgia Tech Center for Music Technology), MIT, last push 2021-05-07, **DORMANT** — [GitHub GTCMT/DataToMusicAPI](https://github.com/GTCMT/DataToMusicAPI). *Listen to Wikipedia* (live generative music from the Wikipedia edit stream), 871 stars, pushed 2026-09-14 — [GitHub hatnote/listen-to-wikipedia](https://github.com/hatnote/listen-to-wikipedia). *Tone.js* (Web Audio framework), MIT, 15.1.22 (2026-07-12), **ACTIVE** — [GitHub](https://github.com/Tonejs/Tone.js).

**C. Python / R / science toolkits**
- **STRAUSS** (Trayford, Youles, Harrison, Shepherd, Bonne; JOSS 2025, 10(109):7875): Apache-2.0, PyPI 1.0.3 (2026-02-17), repo pushed 2026-09-25. **ACTIVE** — [GitHub](https://github.com/james-trayford/strauss), [JOSS](https://joss.theoj.org/papers/10.21105/joss.07875), [docs](https://strauss.readthedocs.io/).
- **Astronify** (STScI): BSD-3 per its docs (GitHub detects no license), PyPI 0.11 (2025-01-31), last commit 2025-12-03; listed as ascl:2408.005. **SLOW** — [GitHub spacetelescope/astronify](https://github.com/spacetelescope/astronify), [docs](https://astronify.readthedocs.io/en/latest/).
- **SonoUno** (CONICET, Universidad de Mendoza, EU H2020 REINFORCE; JOSS published 24 Jan 2024): desktop release 4.1.1 (2024-01-22), last commit 2024-01-25. **DORMANT-ish / SLOW** — [GitHub sonoUnoTeam/sonoUno-desktop](https://github.com/sonoUnoTeam/sonoUno-desktop), [JOSS](https://joss.theoj.org/papers/10.21105/joss.05819), [site](https://www.sonouno.org.ar/) (HTTP 200 on 2026-10-18).
- **Bielefeld University, Thomas Hermann group** (`interactive-sonification` GitHub org), all MIT: **sc3nb** 1.1.0 (PyPI 2023-08-22; last default-branch commit 2023-08-23, pushed 2025-10-24); **mesonic** 0.2.0 (2023-08-23); **sonecules** 0.1.0 (2023-08-24); **pya** 0.5.2 (2023-12-10). **DORMANT** since 2023 — [sc3nb](https://github.com/interactive-sonification/sc3nb), [mesonic](https://github.com/interactive-sonification/mesonic), [sonecules](https://github.com/interactive-sonification/sonecules), [pya](https://github.com/interactive-sonification/pya). Sonecules paper: Reinsch & Hermann, ICAD 2023, DOI 10.21785/icad2023.5580 — [Bielefeld PUB](https://pub.uni-bielefeld.de/record/2979095).
- **R `sonify`** (Stefan Siegert, Robin Williams): GPL (≥2), version 0.1-0, CRAN "Published: 2026-10-06" — [CRAN](https://cran.r-project.org/web/packages/sonify/index.html). The version number is unchanged, so this looks like a re-publication; see Gaps. **playitbyr** was removed from CRAN on 2014-09-20 because it depends on the archived 'csound' package — [CRAN](https://cran.r-project.org/web/packages/playitbyr/index.html). Its GitHub README says it "only works with Csound 6" — [GitHub statisfactions/playitbyr](https://github.com/statisfactions/playitbyr). **New: soundeR** (Matt Waite, "tidyverse-style data sonification for R", created 2026-09-23, GitHub only) — [GitHub mattwaite/soundeR](https://github.com/mattwaite/soundeR).
- **Other Python**: *audiolazy*, GPL-3.0, last PyPI release 0.6 (2016-09-29), last commit 2022-01-26, **DORMANT** — [GitHub](https://github.com/danilobellini/audiolazy). *pyo* (Python DSP), LGPL-3.0, PyPI 1.1.0 (2026-10-09), **ACTIVE** — [GitHub belangeo/pyo](https://github.com/belangeo/pyo). *audio-plot-lib* (graph sonification for screen-reader data science), MIT, 0.1.2 (2023-10-15), **DORMANT** — [GitHub hassaku/audio-plot-lib](https://github.com/hassaku/audio-plot-lib). *sonipy* ("turning scatter plots into perceptually uniform sound files"), MIT, PyPI 1.0 (2020-07-04), small commit 2026-05-13 — [GitHub lockepatton/sonipy](https://github.com/lockepatton/sonipy). *MIDITime* (time series to MIDI), MIT, last commit 2025-02-14 — [GitHub mikejcorey/miditime](https://github.com/mikejcorey/miditime). *libsoni* (sonifying music annotations), 1.1.0 (2025-03-24) — [GitHub groupmm/libsoni](https://github.com/groupmm/libsoni). *pixasonics* (image sonification toolbox), GPL-3.0, 0.1.7.1 (2026-07-29), **ACTIVE** — [GitHub balintlaczko/pixasonics](https://github.com/balintlaczko/pixasonics). *liamtoney/sonify* (seismic and infrasound audification), MIT, last commit 2026-10-09 — [GitHub](https://github.com/liamtoney/sonify). *SYSTEM Sounds sonification tutorials* (Jupyter), last commit 2024-03-21 — [GitHub](https://github.com/SYSTEMSounds/sonification-tutorials).
- **ViewCube / SoniCube** (IAA-CSIC, Rubén García-Benito; first author Adrián García Riber): real-time interactive sonification of IFS data-cube spaxel spectra. An autoencoder produces a 6-D latent vector that drives 6 oscillators of an additive synth built in CSound and controlled from Python. Announced Jan 15, 2025 — [arXiv 2412.00185](https://arxiv.org/pdf/2412.00185), [ViewCube docs](https://viewcube.readthedocs.io/en/latest/sonicube.html), [IAA news](https://archive.iaa.csic.es/en/news/data-sonification-new-way-study-universe).

**D. Legacy academic / government desktop tools**
- **xSonify** (NASA GSFC; Candey, Schertenleib, Diaz-Merced; 2005–06): Java, BSD, Java Sound MIDI output; a "prototype" as of 2008. SourceForge "Last Update" is 2018-11-16. **UNMAINTAINED** — [SourceForge](https://www.sourceforge.net/projects/xsonify/), [ASCL 1207.008](https://ascl.net/1207.008), [ICAD 2006 record](https://repository.gatech.edu/entities/publication/e11a4a26-d85d-40ef-9cd3-0d744ce6009d).
- **Georgia Tech Sonification Sandbox** (Bruce Walker's Sonification Lab): Java desktop with JavaSound/MIDI. Its last desktop update was December 2014 (Java 7); a web version was announced in 2017 — per the survey in [Casado thesis, arXiv 2305.05635](https://arxiv.org/pdf/2305.05635); lab site [sonify.psych.gatech.edu](https://sonify.psych.gatech.edu/). **LEGACY**. Its practical successor is HSS, co-designed with Georgia Tech (inference).
- **MathTrax** (NASA): graphs with descriptions and sound, where rising and falling pitch shows the dependent variable. A 2012 report says NASA "hasn't had the resources to continue to develop it", so it is not HTML5 and cannot be embedded in EPUB 3 — [Bookshare blog, 2012](https://blog.bookshare.org/2012/04/furthering-accessibility-of-graphs-at-the-international-space-apps-challenge/). Its core engine lives on as the Math Description Engine, with a Benetech fork showing "no releases" — [GitHub benetech/mde](https://github.com/benetech/mde). The NASA site (prime.jsc.nasa.gov/mathtrax) did not respond to my request on 2026-10-18. **DISCONTINUED (de facto)**.
- **SysSon** (IEM Graz, "a systematic procedure to develop sonifications", Scala/SuperCollider): **ARCHIVED**, last commit 2019-05-05 — [GitHub iem-projects/sysson](https://github.com/iem-projects/sysson).
- **play-splom** (SuperCollider + Electron sonification of multivariate scatter-plot matrices): MIT, last commit 2019-02-08, **DORMANT** — [GitHub experimentalDataAesthetics/play-splom](https://github.com/experimentalDataAesthetics/play-splom).

**E. Screen-reader ecosystem**
- **NVDA AudioChart add-on** (Tony Malykh) "allows you to play excel time series as a continuous sound". Select cells and press NVDA+A. GPL-2.0, last commit 2025-02-07. **SLOW** — [GitHub mltony/nvda-audio-chart](https://github.com/mltony/nvda-audio-chart), [NVDA add-on page](https://addons.nvda-project.org/addons/AudioChart.ar.html). The related *brailleChart* add-on renders Excel data on a braille display, not as audio — [GitHub travisroth/brailleChart](https://github.com/travisroth/brailleChart).
- **JAWS**: I found no native chart-sonification feature. JAWS users instead combine it with Desmos Audio Trace — [Perkins](https://www.perkins.org/resource/understanding-desmos-sonification-lessons/) — or with SAS Graphics Accelerator, which supports JAWS, NVDA and VoiceOver in Chrome — [SAS](https://support.sas.com/software/products/graphics-accelerator/index.html).

**F. Audio-programming environments and creative toolkits used for sonification**
- **SuperCollider**: GPL-3.0, 3.14.1 (2025-11-24), commits through 2026-10-18 — [GitHub](https://github.com/supercollider/supercollider). **Pure Data**: last commit 2026-10-08 — [GitHub](https://github.com/pure-data/pure-data). **Sonic Pi** v5.0.0 (2026-08-07) — [GitHub](https://github.com/sonic-pi-net/sonic-pi). All **ACTIVE**.
- **Loud Numbers** (Duncan Geere and Miriam Quick, a sonification studio and former podcast): Sonic Pi is "the coding environment where we make the raw forms of almost all of our sonifications" — [Loud Numbers devlog 2020](https://buttondown.com/loudnumbers/archive/devlog-2020-in-review). Their tool output is a free **VCV Rack plugin**, pushed 2026-07-19 (**ACTIVE**), plus a **norns** script (2024-01) and Sonic Pi snippets — [loudnumbers_vcv](https://github.com/loudnumbers/loudnumbers_vcv), [loudnumbers_norns](https://github.com/loudnumbers/loudnumbers_norns), [sonification-snippets](https://github.com/loudnumbers/sonification-snippets). I found no package called a "Loud Numbers toolkit" — [search summary](https://buttondown.com/loudnumbers/archive/sonification-without-coding).
- **PhET Interactive Simulations** (CU Boulder): the `tambo` library "contain[s] code to support sonification of PhET simulations". MIT, last commit 2026-04-30, **ACTIVE** — [GitHub phetsims/tambo](https://github.com/phetsims/tambo). Sonified sims include John Travoltage, Balloons and Static Electricity, Faraday's Law and Molecules and Light — [CADRE K-12](https://cadrek12.org/projects/sonified-interactive-simulations-accessible-middle-school-stem), [IDRC list](https://lists.idrc.ocadu.ca/pipermail/community/2018-May/001049.html), [PhET prototypes](https://clixplatform.tiss.edu/phet/en/accessibility/prototypes.html).
- **NASA Chandra "A Universe of Sound"** (Chandra X-ray Center with SYSTEM Sounds: Kimberly Arcand, Matt Russo, Andrew Santaguida). Since 2020 it is "the first ongoing, sustained program at NASA to 'sonify'" astronomical data, with "dozens" of sonifications. It is a **production programme, not a released tool** — [Chandra](https://chandra.si.edu/sound/index.html), [NASA](https://science.nasa.gov/open-science/astronomy-data-sonification).

**G. New tools and releases, 2023–2026**
- Erie (CHI 2024; TypeScript release 2026), Umwelt (CHI 2024), MAIDR TypeScript rewrite (repo created 2024-08-29) plus py/R/MCP — see above.
- STRAUSS JOSS paper (May 2025). **DataSonifyer** (Christian Basl and Berit Kruse) is a "free and easy-to-use sonification generator" mapping data to pitch, volume, note length and/or tempo, with audio export, at https://studio.datasonifyer.de/en. It was RJI "Tool of the month" in Oct 2024 — [RJI](https://rjionline.org/news/toolkit-overview-examples-tutorials-resources/). Manifest Audio Sonification Tools v2 (2025). ViewCube/SoniCube (2025). Loud Numbers VCV (2026). pixasonics (2025). soundeR (2026).
- **Rust projects are tiny hobby repos**: *math-sonify* is a "Real-time generative synth in Rust that turns 53 dynamical systems (Lorenz, Kuramoto, three-body, hyperchaos) into sound. Desktop app, VST3/CLAP plugin and headless WAV render, with MIDI export and 20 scales." It has 4 stars and was pushed 2026-10-04 — [GitHub Mattbusel/math-sonify](https://github.com/Mattbusel/math-sonify). *sonarplot* ("Rust spatial audio sonification of high-dimensional scatter plots") has 0 stars, pushed 2026-04-27 — [GitHub btraven00/sonarplot](https://github.com/btraven00/sonarplot). *sonify-health* is an infrastructure-monitoring sonification daemon, AGPL-3.0 — [GitHub LoganBarnett/sonify-health](https://github.com/LoganBarnett/sonify-health).
- An LLM as an interpretive layer between live data and a generative music engine (GPT-4, Sydney Opera House installation) — [SMC 2024 paper](https://smcnetwork.org/smc2024/papers/SMC2024_paper_id243.pdf).

### Inferences
- The active, well-maintained dedicated tools are almost all **web-first** (MAIDR, Chart2Music, Highcharts, Erie, Umwelt) or **Python-notebook-first** (STRAUSS). No actively maintained, dedicated, native cross-platform desktop sonification app turned up. The desktop field consists of legacy Java (Sandbox, xSonify, MathTrax), stale Python/wx (SonoUno), or plugins that need a host (VCV Rack, Ableton Live/Max for Live).
- The Bielefeld ecosystem (sc3nb/mesonic/sonecules), the only explicit "sonification architecture" with reusable designs, has gone quiet since 2023. That leaves a vacancy for a maintained, design-library-oriented engine.
- Rust is essentially unoccupied in this space: the only Rust sonification repos have between 0 and 4 stars.

### Gaps
- I could not find any Google product (Sheets, Android TalkBack, ChromeVox) with chart sonification. That may mean none exists, or that search coverage missed it.
- I did not verify Highcharts license pricing or the current Max (Cycling '74) pricing.
- I could not confirm whether the downloadable Sonification Sandbox (desktop or web) is still served; the lab page did not return content to my request.
- The R `sonify` CRAN page shows "Published: 2026-10-06" with version 0.1-0. I could not determine whether this is a re-submission after archival or a metadata refresh.
- I could not confirm the current version or price of Manifest Audio Sonification Tools, or the license of DataSonifyer.

---

## 2. Tool-by-tool profiles: maker, license, stack, inputs, mapping, interaction, real-time, export, extensibility, maintenance, strengths, limitations

### Takeaway
Accessibility libraries excel at **keyboard navigation plus speech plus simple pitch–time mapping**, but rarely export audio or support rich multivariate sound design. Science toolkits (STRAUSS) excel at **offline, multichannel, musically rich rendering**, but lack interactive screen-reader navigation. Only **Erie** offers a true declarative sonification grammar. Only **MAIDR** spans JS, Python and R and documents live/streaming updates. Only **STRAUSS** documents 5.1/7.1 and ambisonic output.

### Cited Findings

#### Profiles (dense)

**Highcharts sonification module** — https://www.highcharts.com/docs/sonification/getting-started
- *Feature list*: "Built-in lightweight synthesizer with several presets", "Speech support", "Navigation support, including features like timeline filters and scrubbing", "Use multiple tracks to layer sounds", "Mapping to musical scales", "Multi-axis & logarithmic axis support", "Context tracks / cues", "Conditional tracks and earcons", "MIDI export" — [getting started](https://www.highcharts.com/docs/sonification/getting-started).
- *Instruments*: plucked (piano, vibraphone), sustained (flute, saxophone), percussion (chop, shaker), effects (wind, wobble) — [instruments](https://highcharts.com/docs/sonification/instruments). *Speech tracks* map volume, rate and pitch, and use a `text` format string or function — [speech](https://www.highcharts.com/docs/sonification/speech).
- *Mapping*: pitch (with min/max such as c3–g6), volume, pan, noteDuration, tremolo and the gap between notes. Parameters take fixed values or data properties, with dot notation and `mapTo/min/max` ranges, and dual coding is supported. The guide recommends G4–F6 as "the most accurate range" and limiting changing parameters to about three — [mapping](https://www.highcharts.com/docs/sonification/mapping).
- *Context tracks* play at fixed intervals (e.g., a tick at each x-axis mark), which serves as an auditory axis. `sonification.order` plays series sequentially or simultaneously — [getting started](https://www.highcharts.com/docs/sonification/getting-started).
- *Limitations*: commercial license needed for commercial use ([HSS README](https://github.com/highcharts/sonification-studio)); it only sonifies Highcharts charts (inference).

**Highcharts Sonification Studio** — https://sonification.highcharts.com
- Vue + TypeScript + ParcelJS; mathJS powers a "Fill equation" feature on the data page, so it accepts functions — [GitHub README](https://github.com/highcharts/sonification-studio).
- A web app where users "enter data into a spreadsheet to create traditional visual charts" and generate non-speech audio; Georgia Tech says it lets users export results "to a useful format" — [Georgia Tech news](https://d7.cos.gatech.edu/node/4322).

**SAS Graphics Accelerator** — https://support.sas.com/software/products/graphics-accelerator/index.html
- *Sonified charts*: bar, box, bubble, heat map, histogram, line, parameterized line, pie, scatter, dendrogram and time series, including grouped charts. *Description only*: contour, band, block, mosaic, loess, regression, step, surface, waterfall, 3-D histogram, multi-layer plots and scatter-plot matrices.
- *Laboratory* imports CSV, TSV/TAB, XLSX, KML and web-page tables. Maps (KML, Google My Maps) can be sonified and navigated with a game controller.
- *Export*: graphs as PNG/HTML; tables as CSV/HTML or a SAS program. No audio export is listed.
- *Platform*: Win 10+ or macOS 10.13+, Chrome 88+, with JAWS, NVDA or VoiceOver. UI is English only — [SAS](https://support.sas.com/software/products/graphics-accelerator/index.html).
- *Mapping and interaction*: higher pitch means a higher value — [SAS 2017](https://www.sas.com/da_dk/news/press-releases/2017/february/sas-graphics-accelerator-education.html). Five listening speeds and note-by-note navigation — [search summary of educator resource](https://accessinghigherground.org/?p=38864).

**TwoTone** — https://twotone.io
- Browser-only processing ("no server-side operations or storage"), installable as a PWA. Input is tabular (spreadsheets, CSV). Columns are attached to pitch and volume. Output goes to "industry-standard formats including live MIDI output"; compositions "can be recorded and shared". The interaction model is "interaction as an instrument" — [README](https://github.com/kineteklabs/twotone).
- *Status*: paused since mid-2023 (above).

**Chart2Music** — https://www.chart2music.com
- *Chart types*: band, bar, box, candlestick, histogram, line, matrix (heatmap), pie, scatter, and arrays of these.
- *Point types*: number; x/y/y2; x/high/low; OHLC; five-number box with outliers.
- *Axes*: label, min/max, format callback, linear or log, continuous mode.
- *Keyboard*: replay; previous/next; first/last; play left/right; move between groups and statistics; jump by 1/10th; go to min/max in group or chart; move between outliers; help and options dialogs (adjust hertz levels, speed); "monitor mode"; touch gestures.
- *Dev hooks*: `onFocusCallback`, `onSelectCallback`, `.getCurrent()` — [features.md](https://github.com/julianna-langston/chart2music/blob/main/features.md).
- "Visual agnostic": it works alongside Chart.js, D3, ChartIQ, Highcharts, Recharts or Google Charts, or even a static image. Blind people are on the dev team — [README](https://github.com/julianna-langston/chart2music).
- *Recent changelog*: translations option (1.17), safer DOM (1.19), Hmong translation (1.20) — [changelog](https://github.com/julianna-langston/chart2music/blob/main/changelog.md).
- *Limitations*: web only. No audio-file export is documented in the feature list.

**Apple Audio Graphs** — https://developer.apple.com/documentation/accessibility/audio-graphs
- An audio graph "typically" represents the x-axis as time and the y-axis as pitch. A stock chart can be "a single continuous tone" and a scatter plot "individual tones" — [Apple](https://developer.apple.com/documentation/accessibility/audio-graphs).
- In Apple's 4-axis example, weight maps to time, MPG to pitch, MSRP to tone length, and country to timbre. It uses `AXDataSeriesDescriptor(isContinuous:)`, numeric and categorical axis descriptors with gridline positions, and a title and summary — [Apple](https://developer.apple.com/documentation/accessibility/representing-chart-data-as-an-audio-graph).
- Swift Charts lets VoiceOver "construct audio graphs automatically" — [Create with Swift](https://createwithswift.com/making-charts-accessible-with-swift-charts).
- *Limitations*: not supported on the iOS Simulator — [Kodeco](https://www.kodeco.com/31561694-ios-accessibility-in-swiftui-create-accessible-charts-using-audio-graphs). A developer reported gridlines (rendered as haptics) had no visible effect — [Apple Dev Forums](https://origin-devforums.apple.com/forums/profile/Desfici). It only works inside apps on Apple platforms, through VoiceOver.

**Desmos Audio Trace** — https://help.desmos.com/hc/en-us/articles/37064105800333-Audio-Trace
- H plays the graph. Stereo pans left to right as the trace moves. Negative y-values add static, and tone quality changes for positive x. V and Shift+V control volume; Alt+1…0 control speed.
- Point navigation uses arrows or J/L. I/K jump between "points of interest", U/N go to first/last, O goes to the origin, and Alt+Up/Down switches curves. Other keys speak coordinates — [Desmos keyboard shortcuts (Windows)](https://www.desmos.com/assets/docs/desmos-keyboard-shortcuts-windows.docx), [Desmos Help](https://help.desmos.com/hc/en-us/articles/37064105800333-Audio-Trace).
- *Documented learning-curve issue*: screen-reader proficiency "does not automatically mean" a student can use Audio Trace — [Perkins](https://www.perkins.org/resource/understanding-desmos-sonification-lessons/).

**MAIDR** — https://github.com/xability/maidr
- *Modes*: braille, text, sonification and review (B/T/S/R toggles); arrow-key navigation; command palette.
- *AI chat*: "requires your own API key … or a local Ollama server".
- *Live data*: `live: true` charts update via `window.maidrLive.setData()` / `appendData()` (e.g., "stock tickers, sensor dashboards, live metrics").
- *Platforms*: React component. It works with SVG (with visual highlight) or PNG/JPG (no highlight) plus a JSON data schema — [README](https://github.com/xability/maidr).
- py-maidr converts "matplotlib, seaborn, Plotly, Bokeh, plotnine, and Altair" charts — [PyPI](https://pypi.org/project/maidr/); r-maidr converts ggplot2 and base R — [GitHub](https://github.com/xability/r-maidr).
- *Limitations*: GPL-3.0 (copyleft) may deter commercial embedding (inference). I did not verify sonification mapping details (see Gaps).

**Erie** — https://github.com/see-mike-out/erie-web
- *Channels*: "time, time2, duration, tap-speed, tap-count, pitch, detune, pan, loudness, timbre", plus post-reverb, modulation index, harmonicity, speech-before/after and repeat. "Erie supports all the properties but circular pan."
- *Scales*: domain/range, polarity, scale-type (sqrt, log, exponential) and include-zero.
- *Composition*: repeat, sequence and overlay. *Tones*: periodic wave, sampling, FM/AM synths. Auditory legends — [arXiv HTML](https://arxiv.org/html/2402.00156v1), [abstract](https://arxiv.org/abs/2402.00156).
- *Player*: play, pause, resume, stop and sub-queue playback on Web Audio/Web Speech. The compiler is separate from the player, so other back ends (CSound, C++, R) "could be targeted".
- *Limitations*: "a minimum set of data transform types"; interactivity and streaming are future work; no audio-file export described — [arXiv HTML](https://arxiv.org/html/2402.00156v1). Dependencies: D3, Arquero, Moment, Vega — [README](https://github.com/see-mike-out/erie-web).

**Umwelt** — https://umwelt-data.github.io/umwelt/editor/
- Visualization, sonification and text are "coequal" and derive from a shared abstract data model. A shared query predicate links the modalities (navigating text highlights the chart and filters audio). Evaluated with 5 blind or low-vision experts — [arXiv 2403.00106](https://arxiv.org/abs/2403.00106), [MIT DSpace](https://dspace.mit.edu/handle/1721.1/155166).

**VoxLens** — https://github.com/athersharif/voxlens
- Three modes: summary, sonification, and voice-command Q&A. Reported 122% higher accuracy and 36% less interaction time; voice recognition "can be frustrating" — [UW/Route Fifty](https://www.route-fifty.com/emerging-tech/2022/06/voxlens-makes-interactive-data-more-accessible-screen-readers/368272), [ScienceDaily](https://www.sciencedaily.com/releases/2022/06/220601155513.htm).
- Supports D3, Google Charts and Chart.js; 2-D single-series, geospatial and multi-series line data; **Chrome only** — [README](https://github.com/athersharif/voxlens).
- A related ASSETS '22 study found screen-reader users prefer "distinct non-continuous responses generated using oscillators with square waveforms" — [search summary of W4A/ASSETS work](https://faculty.washington.edu/wobbrock/pubs/w4a-23.02.pdf).

**STRAUSS** — https://github.com/james-trayford/strauss
- *Classes*: Source, Score, Generator, Channels, Sonification. *Generators*: Synthesiser, Spectraliser, Sampler. Mostly parameter mapping, plus spectral audification — [docs](https://strauss.readthedocs.io/en/latest/).
- *Channels*: "mono", "stereo", "5.1", "7.1" or "custom", with an ambisonic mic type using ACN/ambiX — [channels.py](https://github.com/james-trayford/strauss/blob/main/src/strauss/channels.py). Code notes it is "awaiting better multichannel save routine" because ffmpeg can scramble channel order.
- *Dependencies* include ffmpeg-python, pychord (harmony), wavio, sounddevice and sf2utils (SoundFonts) — [PyPI](https://pypi.org/project/strauss/).
- Optional AI text-to-speech captions via coqui-TTS, supported only on Python ≤3.12 — [README](https://github.com/james-trayford/strauss).

**Astronify** — https://astronify.readthedocs.io
- Depends on astropy, scipy, matplotlib and **pyo** — [PyPI](https://pypi.org/pypi/astronify/json). Sonifies data series (light curves). Its README says it is "in the very beginning stages" — [GitHub](https://github.com/spacetelescope/astronify). Outreach material includes listening quizzes (Flares & Transits, Exoplanets) — [docs](https://astronify.readthedocs.io/en/latest/).

**SonoUno** — https://www.sonouno.org.ar/
- Python with a wxPython GUI and an Octave interface. Reads txt/csv. Can select an x-range, mark and save points of interest, apply math functions (log, square), and save the plot, marked points and CSV. Built after studying Sonification Sandbox, MathTrax and xSonify. Installation requires Octave plus per-distro wxPython wheels — [README](https://github.com/sonoUnoTeam/sonoUno-desktop).
- A web version allows exploration "without any installation" and was tested with the main screen readers — [UTN repository](https://ria.utn.edu.ar/handle/20.500.12272/10716).

**sc3nb / sonecules** — https://github.com/interactive-sonification/sc3nb
- sc3nb is a Python/Jupyter interface to scsynth (OSC, Synth/SynthDef/Buffer/Bus) and sclang, with a `Score` class "for non-realtime synthesis" — [README](https://github.com/interactive-sonification/sc3nb).
- Sonecules are "sonification designs wrapped in a concise object-oriented interface", reusable across similarly structured data. They target novices while letting experts add designs — [README](https://github.com/interactive-sonification/sonecules), [ICAD 2023](https://pub.uni-bielefeld.de/record/2979095).

**xSonify / Sonification Sandbox / MathTrax**
- *xSonify* maps to pitch, volume/loudness and rhythm; reads local files or remote databases; outputs MIDI — [search summary of ICAD 2006 / SG 2008](https://sungeosphere.org/00SGArhiv/SG_v3_No1_2008-p-42.pdf).
- *Sandbox* maps to pitch (instrument, min/max note) or volume (min/max, polarity); import, edit, visualize, sonify and export — [search summary of arXiv 2305.05635](https://arxiv.org/pdf/2305.05635).
- *MathTrax*: rising and falling pitch for the dependent variable, plus descriptions — [Bookshare](https://blog.bookshare.org/2012/04/furthering-accessibility-of-graphs-at-the-international-space-apps-challenge/).

**R `sonify`** — https://cran.r-project.org/package=sonify
- Turns "univariate data, sampled at regular or irregular intervals, into a continuous sound with time-varying frequency" as "a substitute for R's plot function". Imports tuneR — [CRAN](https://cran.r-project.org/web/packages/sonify/index.html).

**Loud Numbers VCV** — https://github.com/loudnumbers/loudnumbers_vcv
- Load a CSV (delimiter auto-detected) and pick a numeric column. Up to 10,000 values are embedded in the patch. Clock-triggered stepping, with outputs of ±5 V and 0–10 V CV, 1V/Oct pitch over a selectable octave range, and gates. Mouse cueing of datapoints. The FAQ acknowledges past CSV-load crashes — [README](https://github.com/loudnumbers/loudnumbers_vcv).

**NVDA AudioChart** — https://github.com/mltony/nvda-audio-chart
- Plays an Excel column as continuous sound (NVDA+A); pressing twice opens a calibration dialog — [NVDA add-on page](https://addons.nvda-project.org/addons/AudioChart.ar.html).

**Manifest Audio Sonification Tools v2** — https://cdm.link/sonification-tools-for-ableton-live-v2/
- *Devices*: Data MIDI (sequencer), Data Mod, Data FX, Dataforge, Typewriter (natural-language to MIDI) and Data Synth.
- *Inputs*: numbers (up to 256 chars), text, images (JPG/PNG/TIFF), MP4/MOV video or live feeds, and OSC.
- *Operations*: invert, reverse, scramble, bias; Euclidean triggers. Requires Ableton Live with Max for Live — [CDM](https://cdm.link/sonification-tools-for-ableton-live-v2/).

**PhET** — https://github.com/phetsims/tambo
- The John Travoltage multimodal sim (ASSETS 2018) used auditory icons, pitch sonification of charge and arm rotation, volume/playback-rate sonification, and speech. Some sounds were "attended to less by children". Sonified sims were tested in Chrome only — [review summary](https://a11y-paradise.onrender.com/reviews/69c495868aab5c4b101667b4), [PhET prototypes](https://clixplatform.tiss.edu/phet/en/accessibility/prototypes.html).

**Chandra Universe of Sound** — https://chandra.si.edu/sound/
- Image scans map brightness to volume and pitch, vertical position to pitch, and wavelength band to instrument family. For example, Cassiopeia A uses X-ray for piano, infrared for strings and brass, and Hubble stars for crotales — [Chandra](https://chandra.si.edu/sound/index.html).

**DataSonifyer** — https://studio.datasonifyer.de/en
- Free web generator: pitch, volume, note length and tempo mappings, with audio export — [RJI](https://rjionline.org/news/toolkit-overview-examples-tutorials-resources/).

**audio-plot-lib** — https://github.com/hassaku/audio-plot-lib
- Interactive HTML (mouse-hover sonification) or a "playable" WAV export through pydub. Requires system ffmpeg libraries — [README](https://github.com/hassaku/audio-plot-lib).

#### Feature-comparison matrix
Legend: **Y** = documented; **P** = partial or indirect; **N** = documented absent or clearly out of scope; **?** = not documented / not verified. "Desk" = runs as a native desktop app. "Offline" = scriptable, reproducible batch rendering to a file. Sources are the per-tool citations above.

| Tool | License / price | Stack | CSV/table in | Live stream | Chart-image in | Function/eqn | Multivariate (>1 sound dim) | Pitch | Pan | Timbre | Loudness | Musical scales | Polarity | Kbd nav | SR/speech | Audio file out | MIDI out | Declarative spec / API | Desk | Offline | Status (last activity) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Highcharts sonification | Proprietary (commercial license) | JS | ? | ? | N | N | Y | Y | Y | Y | Y | Y | ? | P (scrub, timeline) | Y (speech tracks) | ? | Y | API (JS options) | N | P | ACTIVE (Oct 2026) |
| Highcharts Sonif. Studio | BSD-3 + HC license | Vue/TS web | P (spreadsheet) | ? | N | Y (mathJS) | Y | Y | Y | Y | Y | Y | ? | ? | ? | ? | ? | N | N | N | SLOW (Nov 2025) |
| SAS Graphics Accelerator | Free | Chrome ext | Y (CSV/TSV/XLSX/KML) | N | N | P (param. lines) | P (grouped, heatmap) | Y | ? | ? | ? | ? | ? | Y | Y (JAWS/NVDA/VO) | N | N | N | N | N | Maintained (Jan 2026) |
| TwoTone | MPL-2.0, free | JS PWA | Y | N | N | N | Y | Y | ? | ? | Y | ? | ? | ? | ? | Y (record) | Y (live MIDI beta) | N | P (PWA) | N | PAUSED (mid-2023; commit Jul 2024) |
| Chart2Music | MIT | TS web | N (JS data) | P (monitor mode) | P (overlays image, needs data) | N | P (groups/OHLC/box) | Y (Hz range) | ? | ? | ? | ? | ? | Y (rich) | Y | ? | ? | API | N | N | ACTIVE (Sep 2026) |
| Apple Audio Graphs | OS API (free) | Swift/UIKit/SwiftUI | N | ? | N | N | Y (time, pitch, length, timbre) | Y | ? | Y | ? | ? | ? | Y (VoiceOver) | Y | ? | ? | Y (AXChartDescriptor) | Y (Apple only) | ? | ACTIVE (OS) |
| Desmos Audio Trace | Free | Web/apps | P (points) | N | N | Y | P (switch curves) | Y | Y | P (static/tone) | Y (vol) | ? | ? | Y | Y | ? | ? | N | N | N | ACTIVE |
| MAIDR (+py/R) | GPL-3.0 | TS; Python; R | P (JSON schema; plot libs) | Y | P (PNG/JPG + data) | ? | P | ? | ? | ? | ? | ? | ? | Y | Y (+braille, AI chat) | ? | ? | Y (JSON schema, React, MCP) | N | ? | ACTIVE (Oct 2026) |
| Umwelt | No license file | Web | ? | N | N | N | ? | Y | ? | ? | ? | ? | ? | Y | Y (Olli text) | ? | ? | Y (spec) | N | N | ACTIVE/SLOW (Aug 2026) |
| Erie | MIT | TS web (Web Audio/Speech) | Y (data tables) | N (future) | N | N | Y (many channels) | Y | Y | Y | Y | P (notes; no named scales) | Y | N (future) | Y (speech channels) | N | N | Y (grammar) | N | N | ACTIVE (Oct 2026) |
| VoxLens | BSD-3 | JS (Chrome) | N | N | N | N | N | Y | ? | ? | ? | ? | ? | Y | Y (voice Q&A) | N | N | config API | N | N | DORMANT (Jul 2024) |
| STRAUSS | Apache-2.0 | Python | P (pandas) | N | N | P | Y | Y | Y (5.1/7.1/ambisonic) | Y (synth/sampler/SF2) | Y | Y (pychord) | ? | N | P (TTS captions) | Y | ? | Y (Python API, presets) | N | Y | ACTIVE (Sep 2026) |
| Astronify | BSD-3 | Python (pyo) | Y (astropy tables) | N | N | N | N | Y | ? | ? | ? | ? | ? | N | N | ? | ? | API | N | Y | SLOW (Dec 2025) |
| SonoUno | Unclear (NOASSERTION) | Python/wx/Octave; web | Y (txt/csv) | N | N | P (math ops) | ? | Y | ? | ? | ? | ? | ? | P (GUI) | P (web SR-tested) | ? | ? | P (Octave) | Y | N | STALE (Jan 2024) |
| sc3nb/sonecules | MIT | Python + SuperCollider | Y (numpy/pandas) | Y (realtime SC) | N | P | Y | Y | Y | Y | Y | ? | ? | N | N | Y (NRT Score) | ? | Y (sonecule designs) | N | Y | DORMANT (2023) |
| R sonify | GPL≥2 | R/tuneR | Y (vector) | N | N | N | N | Y | ? | N | N | N | ? | N | N | ? | N | function | N | Y | CRAN 2026-10-06 |
| xSonify | BSD | Java | Y (+remote DB) | N | N | N | P | Y | ? | ? | Y | ? | ? | ? | ? | ? | Y | N | Y | ? | UNMAINTAINED (2018) |
| Sonification Sandbox | Free (academic) | Java; web (2017) | Y | N | N | N | P | Y | ? | Y (instrument) | Y | ? | Y | ? | ? | ? | P (MIDI engine) | N | Y | ? | LEGACY (desktop 2014) |
| MathTrax | Free | Desktop | ? | N | N | Y | ? | Y | ? | ? | ? | ? | ? | ? | P (descriptions + SR) | ? | ? | N | Y | ? | DISCONTINUED (stalled ~2012) |
| Loud Numbers VCV | Free (no license file) | C++ VCV Rack | Y (CSV) | P (clock/trigger) | N | N | P (one column per module) | Y (1V/Oct) | P (via patching) | P | P | P (via quantizers) | ? | N | N | P (host) | ? | modular patching | Y (Rack host) | ? | ACTIVE (Jul 2026) |
| NVDA AudioChart | GPL-2.0 | NVDA add-on | Y (Excel cells) | N | N | N | N | Y | ? | N | N | N | ? | Y | Y | N | N | N | Y (Windows) | N | SLOW (Feb 2025) |
| Manifest Sonif. Tools v2 | Commercial | Max for Live | P (numbers/text) | Y (OSC, live video) | P (image pixels) | N | Y | Y | ? | Y (Data Synth) | ? | ? | P (invert) | N | N | P (host) | Y | N | Y (Live host) | P | Released 2025 |
| DataSonifyer | Free | Web | Y | N | N | N | Y (4 params) | Y | ? | ? | Y | ? | ? | ? | ? | Y | ? | N | N | N | Featured Oct 2024 |
| PhET (tambo) | MIT lib; free sims | HTML5/TS | N | Y (sim state) | N | N | Y | Y | ? | Y (auditory icons) | Y | ? | ? | Y | Y (interactive description) | N | N | N | N | N | ACTIVE (Apr 2026) |
| Chandra Universe of Sound | Free media | Produced A/V | N | N | Y (images, bespoke) | N | Y | Y | ? | Y | Y | ? | ? | N | N | Y | ? | N | N | ? | Ongoing since 2020 |
| ViewCube/SoniCube | ? | Python + CSound | P (CALIFA cubes) | Y (interactive) | N | N | Y (6-D latent) | Y | ? | Y | ? | ? | ? | ? | ? | ? | ? | N | Y | ? | 2025 |
| math-sonify (Rust) | MIT | Rust | N | Y (realtime synth) | N | Y (dynamical systems) | ? | Y | ? | ? | ? | Y (20 scales) | ? | ? | ? | Y (headless WAV) | Y | P (VST3/CLAP) | Y | Y | ACTIVE, 4 stars |

### Inferences
- **Common baseline**: pitch plus time mapping and play/speed controls appear in every interactive tool. Keyboard navigation and screen-reader or speech output appear in every accessibility-focused tool (C2M, MAIDR, SAS, Desmos, Apple, VoxLens, Umwelt).
- **Rare capabilities**:
  - Documented audio-file export: STRAUSS, TwoTone, DataSonifyer, audio-plot-lib.
  - MIDI export: Highcharts, TwoTone (beta), xSonify, Manifest, MIDITime, math-sonify.
  - Multichannel or spatial rendering: STRAUSS only.
  - A full declarative grammar: Erie only.
  - Documented streaming: MAIDR, Chart2Music monitor mode, SuperCollider-based stacks, Manifest via OSC.
- **No tool combines** screen-reader-grade keyboard navigation, a declarative spec, live streaming, *and* reproducible offline multichannel/MIDI export. That combination is the white space.
- **No surveyed tool extracts data from raster chart images.** MAIDR and Chart2Music accept images only as a visual layer and need the data supplied separately. Pixel-level image sonification (pixasonics, Manifest, Chandra) is a different problem from chart-data recovery.

### Gaps
- MAIDR's exact sonification mappings (pitch range, pan, timbre per trace, audio export) are in `docs/CONTROLS.md` and `docs/SCHEMA.md`, which I did not read.
- I could not confirm whether Chart2Music uses stereo pan or timbre, or exports audio.
- I could not confirm HSS export formats beyond the module-level "MIDI export" feature; the third-party claims of MP3 export I found referred to a different product.
- I did not obtain Apple Audio Graphs' loudness/pan behaviour or any export capability.
- I could not confirm Astronify's output format (WAV) or interactive playback controls from fetched pages.
- I did not verify whether R `sonify` writes WAV files (via tuneR).

---

## 3. What recurring gaps exist across the landscape?

### Takeaway
Every gap the brief suggested is confirmed. There is no maintained high-performance native cross-platform desktop app. Multivariate support is weak outside Apple, Highcharts, Erie and STRAUSS. No tool ingests chart images. Live streaming is rare and web-only. The only declarative spec (Erie) is web-only, has no interaction or export, and covers minimal data transforms. Accessibility tools sound basic while musical tools lack accessibility. There is no plugin ecosystem, little Python/CLI parity beyond MAIDR, and little reproducible offline rendering outside STRAUSS and sc3nb.

### Cited Findings
- **Desktop apps are legacy or stale**:
  - Sonification Sandbox desktop last updated December 2014 (Java 7) — [arXiv 2305.05635](https://arxiv.org/pdf/2305.05635).
  - xSonify last updated on SourceForge 2018-11-16 — [SourceForge](https://www.sourceforge.net/projects/xsonify/).
  - MathTrax stalled for lack of resources (2012) — [Bookshare](https://blog.bookshare.org/2012/04/furthering-accessibility-of-graphs-at-the-international-space-apps-challenge/).
  - SonoUno desktop last committed 2024-01-25 and needs Octave plus per-distro wxPython — [GitHub](https://github.com/sonoUnoTeam/sonoUno-desktop).
  - Rust sonification repos have 0–4 stars — [math-sonify](https://github.com/Mattbusel/math-sonify), [sonarplot](https://github.com/btraven00/sonarplot).
- **Declarative spec scarcity**: Erie is described as addressing the fact that software support lagged research. Interactivity, streaming and other audio environments are listed as future work, and it supports only "a minimum set of data transform types" — [arXiv 2402.00156](https://arxiv.org/html/2402.00156v1).
- **Live streaming**: MAIDR is the only accessibility library documenting `live: true` with `appendData()` — [MAIDR README](https://github.com/xability/maidr). Erie lists streaming as future work — [arXiv](https://arxiv.org/html/2402.00156v1).
- **Platform lock-in**: VoxLens is Chrome-only ([README](https://github.com/athersharif/voxlens)). SAS GA needs Chrome 88+ ([SAS](https://support.sas.com/software/products/graphics-accelerator/index.html)). PhET sonified sims were tested only in Chrome ([PhET](https://clixplatform.tiss.edu/phet/en/accessibility/prototypes.html)). Apple Audio Graphs are Apple-only ([Apple](https://developer.apple.com/documentation/accessibility/axchartdescriptor)).
- **Chart coverage holes**: SAS GA provides description-only (no sonification) for contour, surface, regression, loess, step, waterfall, 3-D histogram, multi-layer plots and scatter-plot matrices — [SAS](https://support.sas.com/software/products/graphics-accelerator/index.html).
- **Fragile data handling**: Chart Reader "does not support missing values" — [GitHub](https://github.com/microsoft/chart-reader). Loud Numbers VCV FAQ documents CSV-load crashes — [README](https://github.com/loudnumbers/loudnumbers_vcv).
- **Packaging pain**: STRAUSS TTS works only on Python ≤3.12 — [README](https://github.com/james-trayford/strauss). Its multichannel save can scramble channel order via ffmpeg — [channels.py](https://github.com/james-trayford/strauss/blob/main/src/strauss/channels.py). audio-plot-lib needs system ffmpeg libraries — [README](https://github.com/hassaku/audio-plot-lib). playitbyr "depends on obsolete packages" — [GitHub](https://github.com/statisfactions/playitbyr).
- **Sound-design constraints in accessibility tools**: users preferred discrete square-wave tones ([UW](https://faculty.washington.edu/wobbrock/pubs/w4a-23.02.pdf)). Highcharts advises limiting to about 3 changing parameters ([Highcharts](https://www.highcharts.com/docs/sonification/mapping)). Children attended less to some PhET sounds ([review](https://a11y-paradise.onrender.com/reviews/69c495868aab5c4b101667b4)).
- **Abandonment risk**: TwoTone paused ([README](https://github.com/kineteklabs/twotone)); VoxLens has had no commits since Jul 2024 ([GitHub](https://github.com/athersharif/voxlens)); the Bielefeld stack has been quiet since 2023 ([GitHub](https://github.com/interactive-sonification/sonecules)); Microsoft Chart Reader is archived ([GitHub](https://github.com/microsoft/chart-reader)).
- **Practitioners rely on general music tools**: Loud Numbers makes "almost all" of its sonifications in Sonic Pi — [devlog](https://buttondown.com/loudnumbers/archive/devlog-2020-in-review). This suggests dedicated tools do not meet professional sound-design needs (inference).

### Inferences

#### Gaps & opportunities for SONARITHM
1. **Native, high-performance, cross-platform desktop app and library (Rust core)**: no maintained competitor. The closest alternatives are dead Java tools, stale Python/wx SonoUno, or host-dependent plugins.
2. **Declarative spec plus native renderer**: adopt or import an **Erie-compatible JSON spec**, which is MIT-licensed with a separate compiler and player and invites "other environments". SONARITHM would add what Erie lacks: interactivity, streaming, offline export and richer transforms. Also consider importers for the **MAIDR JSON schema**, **Vega-Lite** (as Olli/Umwelt do) and Apple's `AXChartDescriptor` semantics (axes, series, summary, `isContinuous`).
3. **Reproducible offline rendering and CLI**: deterministic batch rendering to WAV/FLAC/multichannel/ambisonic plus MIDI from a spec file, with CLI and Python bindings. Today this is split between STRAUSS (Python, offline, multichannel), Highcharts (MIDI) and sc3nb (NRT Score), and none of them have accessibility navigation.
4. **Live streaming sub-engine**: only MAIDR (web) documents streaming in the accessibility space. Native ingestion via OSC, WebSocket, MQTT or stdin, with "monitor mode" alerts in the style of Chart2Music, is open territory for dashboards, ops monitoring (cf. hobby daemons like sonify-health and syssonic) and science instruments.
5. **Chart-image ingestion**: no surveyed tool recovers data from raster or vector chart images. ML chart-data extraction feeding sonification would be a genuine differentiator, especially for PDFs and papers.
6. **Multivariate and model-based sonification**: model-based sonification survives mainly in the dormant Bielefeld stack. Multivariate exploration tools (play-splom) are dormant; sonarplot is tiny. SONARITHM's model-based and multivariate engines would face little maintained competition.
7. **Auditory axes and navigation as first-class features**: generalise Highcharts' "context tracks" (axis ticks), Apple's gridlines, Erie's auditory legends, and Chart2Music/Desmos navigation (min/max, outliers, points of interest) into a reusable navigation layer with screen-reader integration on Windows (NVDA/JAWS via UIA), macOS (VoiceOver) and Linux (Orca).
8. **Musicality without losing accessibility**: no tool bridges musical mapping (TwoTone, Manifest, Sonic Pi workflows, STRAUSS chords) with screen-reader-grade interaction. Offer scale/key quantisation, SoundFonts and instrument presets *and* perceptually validated accessibility presets (e.g., discrete square-wave tones, G4–F6 range).
9. **Plugin and design library ecosystem**: revive the "sonecule" idea (reusable, documented sonification designs) as a plugin registry. Optionally ship VST3/CLAP or OSC bridges, the way math-sonify ships VST3/CLAP, so SONARITHM can feed DAWs, VCV Rack and SuperCollider.
10. **Math-function sonification**: Desmos (closed, web) and HSS (mathJS fill) are the only maintained options; MathTrax is dead. A native function sonifier with points-of-interest detection (roots, extrema, inflections), as Desmos does with I/K, fills that space.
11. **Pattern analytics**: what exists is limited to navigation shortcuts (min/max/outliers in C2M, POIs in Desmos), conditional earcons (Highcharts) and LLM chat (MAIDR). Automated pattern detection (trends, anomalies, periodicity, clusters) sonified as earcons is a differentiator.
12. **Licensing position**: a permissive license (MIT/Apache) would contrast with GPL-3.0 MAIDR and proprietary Highcharts, easing commercial embedding.

### Gaps
- I found no systematic user-complaint corpora (e.g., app-store reviews, issue-tracker sentiment) for these tools. The limitations above are drawn from docs, papers and READMEs.
- I did not quantify how many of the Data Sonification Archive's projects used dedicated tools versus bespoke code.

---

## 4. Which features are table stakes versus genuine differentiators?

### Takeaway
For an accessibility-grade chart sonifier, table stakes are: x→time and y→pitch with an adjustable range; play/pause/speed; point-by-point keyboard navigation; spoken values, titles and summaries; multi-series switching; core chart types (line, bar, scatter; often box, histogram, heatmap); and a free or open license. Differentiators include a declarative grammar, live streaming, multichannel/ambisonic and MIDI export, rich multivariate channels, cross-modal linking, braille, AI Q&A, native OS integration, and cross-language parity.

### Cited Findings
- **Table stakes evidence (pitch–time mapping)**: Apple's default x→time, y→pitch ([Apple](https://developer.apple.com/documentation/accessibility/audio-graphs)); SAS "high pitch indicating a higher value" ([SAS](https://www.sas.com/da_dk/news/press-releases/2017/february/sas-graphics-accelerator-education.html)); Desmos stereo trace ([Desmos](https://help.desmos.com/hc/en-us/articles/37064105800333-Audio-Trace)); MathTrax rising and falling pitch ([Bookshare](https://blog.bookshare.org/2012/04/furthering-accessibility-of-graphs-at-the-international-space-apps-challenge/)); R sonify's time-varying frequency ([CRAN](https://cran.r-project.org/web/packages/sonify/index.html)).
- **Table stakes evidence (navigation and speed)**: Chart2Music's extensive keyboard set and speed/hertz options ([features](https://github.com/julianna-langston/chart2music/blob/main/features.md)); MAIDR's arrow-key navigation ([README](https://github.com/xability/maidr)); SAS's five speeds and note-by-note navigation ([search summary](https://accessinghigherground.org/?p=38864)); Desmos speed and volume keys ([Desmos](https://www.desmos.com/assets/docs/desmos-keyboard-shortcuts-windows.docx)).
- **Table stakes evidence (speech/summary)**: Apple's chart summary ([Apple](https://developer.apple.com/documentation/accessibility/representing-chart-data-as-an-audio-graph)); Highcharts speech tracks ([Highcharts](https://www.highcharts.com/docs/sonification/speech)); VoxLens summary mode ([Route Fifty](https://www.route-fifty.com/emerging-tech/2022/06/voxlens-makes-interactive-data-more-accessible-screen-readers/368272)).
- **Differentiator — grammar**: Erie ([arXiv](https://arxiv.org/abs/2402.00156)).
- **Differentiator — streaming**: MAIDR `live: true` ([README](https://github.com/xability/maidr)).
- **Differentiator — multichannel**: STRAUSS 5.1/7.1/ambisonic ([channels.py](https://github.com/james-trayford/strauss/blob/main/src/strauss/channels.py)).
- **Differentiator — MIDI**: Highcharts ([docs](https://www.highcharts.com/docs/sonification/getting-started)), TwoTone ([README](https://github.com/kineteklabs/twotone)).
- **Differentiator — multivariate channels**: Apple's tone length and timbre ([Apple](https://developer.apple.com/documentation/accessibility/representing-chart-data-as-an-audio-graph)); Erie's tap, detune, modulation, reverb and more ([arXiv](https://arxiv.org/html/2402.00156v1)).
- **Differentiator — cross-modal linking**: Umwelt's shared query predicate ([arXiv](https://arxiv.org/abs/2403.00106)).
- **Differentiator — braille, AI chat, Python/R parity, MCP**: MAIDR ([README](https://github.com/xability/maidr), [maidr-mcp](https://github.com/xability/maidr-mcp)).
- **Differentiator — auditory context/earcons**: Highcharts context tracks and conditional earcons ([docs](https://www.highcharts.com/docs/sonification/getting-started)).
- **Differentiator — maps with game controller**: SAS GA ([SAS](https://support.sas.com/software/products/graphics-accelerator/index.html)).
- **Differentiator — points of interest**: Desmos I/K ([Desmos](https://www.desmos.com/assets/docs/desmos-keyboard-shortcuts-windows.docx)).
- **Differentiator — reusable design library**: sonecules ([ICAD 2023](https://pub.uni-bielefeld.de/record/2979095)).

### Inferences
- For SONARITHM, a credible v1 must match the table stakes on the desktop: keyboard navigation, screen-reader output, speech, speed, multi-series switching and common chart types. It should then compete on the combination no one offers: native performance, spec plus CLI reproducibility, streaming, multichannel/MIDI export, multivariate and model-based engines, and chart-image ingestion.
- "Free/open" is table stakes. Every major non-Highcharts tool is free, and SAS GA is free despite being corporate.

### Gaps
- I did not find a published, systematic feature-coverage survey of sonification software from 2024–2026. The matrix above is my synthesis from primary sources. The Casado thesis survey (arXiv 2305.05635) covers mostly older astronomy-oriented tools.

---

## 5. Are there curated catalogues, and what do they reveal about how sonification is used in practice?

### Takeaway
The main catalogue is the **Data Sonification Archive** (sonification.design; Northeastern Center for Design with Deusto). A 2024 JAES analysis of its 445 projects classifies them by Goal, Method, User, Macro/Micro Topic and MediaMix. The **astronomy review** of about 100 projects shows public engagement (36%) outweighs research (26%), art (17%), accessibility (13%) and education (8%). Newer resource hubs (RJI Data Sonification Toolkit, Decibels) catalogue tools for journalists and beginners. Overall, sonification in practice is dominated by **outreach, journalism and art made with bespoke or general-purpose music tools**, more than by dedicated analytical software.

### Cited Findings
- **Data Sonification Archive**: "an initiative of Center for Design at Northeastern University" with the Deusto Design Research Group. Curated by Sara Lenzi and Paolo Ciuccarelli with Houjiang Liu; CC-BY-SA; provides a "Data Sonification Canvas" (Use Case, Mapping Choices, Sonification Approach, Listening Experience) — [sonification.design/about](https://sonification.design/about.html), [archive](https://sonification.design/).
- It held more than 400 projects as of a 2023 seminar — [KTH seminar](https://www.kth.se/hct/mid/calendar/seminars/the-design-of-data-sonification-1.1241095). Recent entries include "New York City Evictions Sonification" (Aura Walmer, 2024) and "Hearing black hole winds" (Trayford & Harrison, 2023) — [search summary of sonification.design](https://sonification.design/).
- **JAES analysis**: Lindborg, Caiola, Ciuccarelli, Chen and Lenzi, "Re(de)fining Sonification: Project Classification Strategies in the Data Sonification Archive", JAES 72(9):585–602, 2024, DOI 10.17743/JAES.2022.0167. It covers a corpus of **445 projects** and assesses six curatorial strategies: **Goal, Method, User, Macro Topic, Micro Topic, MediaMix**. Text-based topic modelling was "more powerful" than audio-feature clustering; predictive modelling was tentative for Goal, Method and Macro Topic — [Deusto repository](https://deustoteka.deusto.es/items/d2052f9d-ce11-4bd1-8a8a-0d88bae19217), [PoliMi open-access PDF](https://re.public.polimi.it/bitstream/11311/1281705/1/Re%28de%29fining_Sonification__Project_Classification_Strategies_in_the_Data_Sonification_Archive__doc.pdf).
- **Astronomy review**: Zanella, Harrison, Lenzi, Cooke, Damsma and Fleming, "Sonification and sound design for astronomy research, education and public engagement", *Nature Astronomy* 6:1241–1248 (2022), covering close to 100 projects. Primary objectives: public engagement 36%, research 26%, art 17%, accessibility 13%, education 8% — [arXiv 2206.13536](https://arxiv.org/pdf/2206.13542), [INAF interview](https://www.media.inaf.it/2022/08/15/sonificazione-astronomia/). (Note: the search summary cited arXiv 2206.13536 while the result link was 2206.13542; the exact arXiv ID should be double-checked.)
- **RJI Data Sonification Toolkit** (sonificationkit.com, announced Feb 27, 2025, by RJI fellow Aura Walmer): a resource hub cataloguing tools by difficulty, covering methods (parameter mapping, audification) and a workflow roadmap — [RJI announcement](https://rjionline.org/news/introducing-the-data-sonification-toolkit/). Companion exercises repo — [GitHub awalmer/Sonification-Exercises](https://github.com/awalmer/Sonification-Exercises).
- **Decibels** sonification community: website repo maintained by Loud Numbers (pushed 2026-01-29) and a "sonification-resources" list (pushed 2024-10-14) — [GitHub loudnumbers/decibels.community](https://github.com/loudnumbers/decibels.community), [GitHub Decibels-Sonification/sonification-resources](https://github.com/Decibels-Sonification/sonification-resources).
- **NASA astronomy data sonification hub** — [science.nasa.gov](https://science.nasa.gov/open-science/astronomy-data-sonification); **Audio Universe** — [audiouniverse.org](https://www.audiouniverse.org/) (linked from the STRAUSS README).
- **Practice signals**: Chandra's sustained outreach programme (since 2020) — [Chandra](https://chandra.si.edu/sound/index.html); journalism use (RJI fellowship, the NYC Evictions entry) — [RJI](https://rjionline.org/news/embracing-data-driven-audio-as-a-storytelling-method/); live/monitoring art such as Listen to Wikipedia — [GitHub](https://github.com/hatnote/listen-to-wikipedia); studio practice built on Sonic Pi — [Loud Numbers](https://buttondown.com/loudnumbers/archive/devlog-2020-in-review); workshops still running (Loud Numbers "Create Your First Sonification", 6 Nov 2026) — [TicketTailor](https://www.tickettailor.com/events/loudnumbers/2434635).

### Inferences
- Catalogued projects are mostly **communicative artefacts** (outreach pieces, journalism, art) rather than outputs of analytical tools. That fits the tooling gap: practitioners hand-build in Sonic Pi, SuperCollider or Max because no dedicated tool offers professional sound design, reproducible export and accessibility together.
- Accessibility is a stated goal for a minority of astronomy projects (13%), while dedicated accessibility *libraries* dominate the maintained-software list. SONARITHM can serve both audiences: an analysis- and accessibility-grade engine that is also expressive enough for storytellers.
- Monitoring and real-time operational sonification is underrepresented in catalogues and served only by bespoke or hobby code. That is a niche SONARITHM's streaming engine could own.

### Gaps
- I could not obtain the DSA's per-category counts (e.g., % journalism versus science versus accessibility versus monitoring) from the fetched pages. The open-access JAES PDF likely contains them but I did not read the full text.
- I did not verify the total current DSA project count as of October 2026.
- I found no catalogue of sonification *software* specifically, as opposed to projects, apart from the RJI toolkit's tool list, whose full contents I did not fetch, and the Decibels resources list.
