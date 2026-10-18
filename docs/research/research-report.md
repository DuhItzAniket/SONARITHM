# SONARITHM Can Own Sonification's Empty Native Ground

SONARITHM's opening is real and narrow. As of October 2026, **no actively maintained, native, cross-platform sonification application or engine exists**, and no tool of any kind combines screen-reader-grade navigation, a declarative mapping specification, live streaming and reproducible offline audio/MIDI export. The active field is web libraries (MAIDR, Chart2Music, Highcharts, Erie, Umwelt) and Python notebooks (STRAUSS). Desktop tools are abandoned Java or stale Python, and Rust holds only zero-to-four-star hobby repositories. The perception science is mature enough to set defaults: pitch for value and time for x, roughly 400–1400 Hz, 2–4 points per second, axis clicks plus a dynamic reference tone, and timbre and stereo to separate series. The same science warns that polarity is concept- and population-specific, that designer intuition is unreliable, that loudness distorts pitch, and that only 6.1% of published mappings were ever evaluated. Blind users want an overview first, then control, then the raw data, so the navigation layer (AUDAXIS) matters as much as the sound. The realistic product is one specification compiler and render engine with nine front ends, shipped in waves: parameter mapping, auditory axes, structured chart ingestion, streaming and export first. Model-based, multivariate and ML image-extraction features come later, because most models still score only about 18–39 RMS-F1 when extracting line charts from images. The best-fitting stack is a backend-agnostic Rust audio core (cpal, fundsp, rtrb, Arrow) under a Tauri 2.12 + React 19.2 + React Aria shell, licensed MIT OR Apache-2.0 and targeting WCAG 2.2 AA through WCAG2ICT and EN 301 549. Its strongest publishable contributions fill gaps the field admits to: an open evaluation benchmark, the first magnitude-estimation data for graph concepts, and controlled tests of LLM-suggested mappings. The natural debut is ICAD 2027 in Athens, followed by JOSS once the repository has six months of public history.

## Every active sonification tool lives in a browser or a notebook

The maintained centre of data sonification in October 2026 is a cluster of web accessibility libraries.

**MAIDR** (GPL-3.0) shipped v4.15.0 on 12 October 2026. It has spread into Python, where py-maidr converts matplotlib, seaborn, Plotly, Bokeh, plotnine and Altair charts, and into R. It also has an experimental MCP server and live-updating charts through `appendData()` ([GitHub xability/maidr](https://github.com/xability/maidr), [PyPI](https://pypi.org/project/maidr/)).

**Chart2Music** (MIT) has the richest keyboard model in the field: jump by tenths, go to minimum or maximum, step between outliers, and a "monitor mode". It runs on top of Chart.js, D3, Highcharts or even a static image, and counts blind people on its development team ([features.md](https://github.com/julianna-langston/chart2music/blob/main/features.md), [README](https://github.com/julianna-langston/chart2music)). ChartIQ has integrated it commercially ([ChartIQ docs](https://documentation.chartiq.com/WebComponents.Chart2Music.html)).

The proprietary **Highcharts sonification module** contributes context tracks that act as auditory axes, conditional earcons, speech tracks and MIDI export. Commercial use requires a commercial licence ([Highcharts docs](https://www.highcharts.com/docs/sonification/getting-started), [HSS README](https://github.com/highcharts/sonification-studio)).

Two CHI 2024 research systems define the frontier of specification. **Erie** is the only declarative sonification grammar, with channels for time, duration, tap speed, pitch, detune, pan, loudness and timbre ([arXiv 2402.00156](https://arxiv.org/abs/2402.00156)). **Umwelt** treats visualization, sonification and text as coequal views of one data model ([arXiv 2403.00106](https://arxiv.org/abs/2403.00106)).

Outside the browser, **STRAUSS** (Apache-2.0, JOSS 2025) is the only tool documenting 5.1, 7.1 and ambisonic output ([channels.py](https://github.com/james-trayford/strauss/blob/main/src/strauss/channels.py), [JOSS](https://joss.theoj.org/papers/10.21105/joss.07875)). At the platform level, Apple's Audio Graphs (`AXChartDescriptor`) sonify charts inside the operating system, Desmos Audio Trace does so inside a graphing calculator, and the free SAS Graphics Accelerator does so as a Chrome extension ([Apple](https://developer.apple.com/documentation/accessibility/audio-graphs), [Desmos](https://help.desmos.com/hc/en-us/articles/37064105800333-Audio-Trace), [SAS](https://support.sas.com/software/products/graphics-accelerator/index.html)).

Native desktop software, the category SONARITHM targets, is a graveyard. The Georgia Tech Sonification Sandbox last updated its desktop build in **December 2014** on Java 7 ([Casado thesis](https://arxiv.org/pdf/2305.05635)); NASA's xSonify last moved in 2018 ([SourceForge](https://www.sourceforge.net/projects/xsonify/)); MathTrax stalled around 2012 for lack of resources ([Bookshare](https://blog.bookshare.org/2012/04/furthering-accessibility-of-graphs-at-the-international-space-apps-challenge/)); and SonoUno's desktop app last committed in January 2024 and still needs Octave plus per-distribution wxPython wheels ([GitHub](https://github.com/sonoUnoTeam/sonoUno-desktop)).

Abandonment is the field's recurring failure. TwoTone has been paused since mid-2023 ([README](https://github.com/kineteklabs/twotone)), and VoxLens has had no commits since July 2024 ([GitHub](https://github.com/athersharif/voxlens)). Bielefeld's sc3nb/sonecules stack, the only explicit library of reusable sonification designs, has been quiet since 2023 ([sonecules](https://github.com/interactive-sonification/sonecules)). Microsoft's Chart Reader is archived ([GitHub](https://github.com/microsoft/chart-reader)).

Rust is essentially empty. The closest projects, math-sonify and sonarplot, have **4 and 0 GitHub stars** ([math-sonify](https://github.com/Mattbusel/math-sonify), [sonarplot](https://github.com/btraven00/sonarplot)). The only related crate on docs.rs, data2sound, writes raw bytes to WAV rather than doing perceptual sonification ([docs.rs](https://docs.rs/data2sound)).

| Tool | Licence | Platform | Best at | Missing for SONARITHM's goals | Last activity |
|---|---|---|---|---|---|
| MAIDR | GPL-3.0 | TypeScript web; Python; R; MCP | Braille, text, sonification and review modes on one cursor; live data; LLM chat | Copyleft licence; browser-only rendering; no documented audio export | Oct 2026 |
| Chart2Music | MIT | TypeScript web | Deepest keyboard navigation; works with any visual library | Web only; no documented audio export; no peer-reviewed evaluation | Sep 2026 |
| Highcharts sonification | Proprietary | JavaScript | Context tracks, earcons, speech tracks, MIDI export | Commercial licence; sonifies only Highcharts charts | Oct 2026 |
| Erie | MIT | TypeScript web | Only declarative sonification grammar | Interactivity and streaming listed as future work; no audio export | Oct 2026 |
| Umwelt | No licence file | Web | Coequal visual, audio and text views with linked selection | Unclear licence; no export | Aug 2026 |
| STRAUSS | Apache-2.0 | Python | Offline multichannel and ambisonic rendering, harmony | No interactive or screen-reader navigation | Sep 2026 |
| Apple Audio Graphs | OS API | Apple platforms | Standard chart descriptor; four-axis mapping | Apple and VoiceOver only | Shipping |
| SAS Graphics Accelerator | Free | Chrome extension | CSV, XLSX and KML import; map navigation by game controller | No audio export; many chart types description-only | Jan 2026 |
| sc3nb / sonecules | MIT | Python + SuperCollider | Reusable designs; non-realtime scores | Dormant; no accessibility layer | 2023 |
| SonoUno | Unclear | Python/wx desktop; web | Desktop exploration, points of interest | Stale; heavy install | Jan 2024 |

No tool offers the combination SONARITHM is aiming for. Each relevant capability exists in only one or a handful of tools:

| Capability | Where it exists today |
|---|---|
| Documented audio-file export | STRAUSS, TwoTone, DataSonifyer and audio-plot-lib |
| MIDI export | Highcharts, TwoTone's beta, xSonify, Manifest and math-sonify |
| Multichannel rendering | STRAUSS alone |
| Live streaming in an accessibility tool | MAIDR's `live: true` and Chart2Music's monitor mode only |
| Data recovery from raster chart images | None. MAIDR and Chart2Music accept images only as a visual layer, with the data supplied separately |

Sources: [STRAUSS](https://github.com/james-trayford/strauss), [TwoTone](https://github.com/kineteklabs/twotone), [Highcharts](https://www.highcharts.com/docs/sonification/getting-started), [MAIDR README](https://github.com/xability/maidr), [Chart2Music README](https://github.com/julianna-langston/chart2music).

Accessibility-first tools sound basic. Highcharts recommends G4–F6 as "the most accurate range" and changing no more than about three parameters at once ([Highcharts mapping](https://www.highcharts.com/docs/sonification/mapping)). Expressive tools fail the other way: Manifest Audio's Sonification Tools v2 require Ableton Live with Max for Live and offer no screen-reader support ([CDM](https://cdm.link/sonification-tools-for-ableton-live-v2/)).

Practice confirms that dedicated software has not met practitioners' needs. A 2024 JAES analysis classified **445 projects** in the Data Sonification Archive ([Deusto repository](https://deustoteka.deusto.es/items/d2052f9d-ce11-4bd1-8a8a-0d88bae19217)), and a 2025 follow-up covering 470 projects found that "engagement" goals have grown at the expense of "art", with a large effect size ([Lindborg et al. 2025](https://doi.org/10.21785/icad2025.017)). Among about 100 astronomy projects, primary aims split into public engagement (36%), research (26%), art (17%), accessibility (13%) and education (8%) ([Sky & Telescope summary of Zanella et al.](https://skyandtelescope.org/astronomy-news/how-astronomers-are-rendering-the-universe-in-sound/)). The studio Loud Numbers makes "almost all" of its sonifications in Sonic Pi ([Loud Numbers devlog](https://buttondown.com/loudnumbers/archive/devlog-2020-in-review)). SONARITHM's real competitors are therefore general music tools on the storytelling side and web libraries on the accessibility side.

The strongest counter-argument to a desktop-first project is that users live in browsers and notebooks, and MAIDR's spread across JavaScript, Python, R and MCP shows that momentum. The answer is to make SONARITHM complementary infrastructure rather than an island. The same Rust core should compile to WebAssembly, since cpal now has first-class WebAudio and AudioWorklet hosts ([cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)), and should also ship as Python wheels. It should import the formats web tools already use: Erie specs, Vega-Lite, MAIDR's JSON schema and Apple's `AXChartDescriptor` semantics. Erie keeps its compiler separate from its player, and its authors say other back ends "could be targeted" ([Erie paper](https://arxiv.org/html/2402.00156v1)). A permissive licence completes the positioning against GPL-3.0 MAIDR and proprietary Highcharts.

## Perception research fixes the defaults but not the polarities

### Hermann's four criteria become engineering requirements

The field works with two definitions. Kramer (1999) defines sonification by purpose: "the transformation of data relations into perceived relations in an acoustic signal for the purposes of facilitating communication or interpretation." Hermann (2008) defines it by method. A technique qualifies only if the sound reflects **objective** properties of the data, the transformation is **systematic** (precisely defined), the result is **reproducible** (the same data and interactions give structurally identical sound), and the technique is **reusable** with different data ([Hermann 2008](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)).

Hermann allows randomness, provided "it is important to declare where and what random elements are used". He requires "structurally identical" rather than sample-identical output. Data-driven art that breaks these rules, he argues, is better called "data-inspired music".

Enge et al. align the vocabulary with visualization theory. Time is the **temporal substrate**, sounds are **auditory marks**, and sound attributes are **auditory channels** ([Enge et al. 2023](https://doi.org/10.1007/s00779-023-01720-5)). A EuroVis 2024 state-of-the-art report used this vocabulary to classify 57 audiovisual-display papers, and found "individual teams without many interconnections" ([Enge et al. 2024](https://doi.org/10.1111/cgf.15114)).

For SONARITHM's core, these definitions become concrete requirements. Every render exports a serializable mapping spec; all randomness is seeded and declared; designs are data-agnostic and reusable, in the spirit of Bielefeld's "sonecules" ([Reinsch & Hermann 2023](https://doi.org/10.21785/icad2023.5580)); modes carry specific technique names (PMSon, audification, MBS), as Hermann recommends; and the API speaks in substrate, mark and channel, so a visual layer and an Erie-style spec share one vocabulary.

### Pitch carries value, but polarity must be a switch

Pitch dominates the field. A review of 179 publications from 60 projects found 495 mapping occurrences, and pitch was used significantly more than any of the other 29 auditory dimensions ([Dubus & Bresin 2013](https://doi.org/10.1371/journal.pone.0082491)). Trend in a sonification "closely parallels the notion of melodic contour" ([Walker & Nees 2011](https://sonification.de/handbook/chapters/chapter2/)). Tempo and onset changes register more slowly than pitch or loudness changes ([Walker & Kramer 2005](https://doi.org/10.1145/1101530.1101534)).

Blind and low-vision listeners add a nuance. They rated pitch the most intuitive channel, but tap counts and sound length gave the most accurate decoding of values ([Wang, Jung & Kim 2022](https://doi.org/10.1111/cgf.14523)). SONARITHM should keep pitch as the default and offer duration or tap-count as a redundant channel for precise reading.

Polarity, meaning whether higher values sound higher, is where intuition fails. In Walker's magnitude-estimation studies, listeners assigned numbers to what a sound represents. Perceptual pitch judgments were stable across 209 + 226 listeners in a direct replication (pitch:frequency slope 0.89, 95% CI 0.71–1.07), but the mapping from data concept to sound varied by dimension ([Walker 2002](https://doi.org/10.1037/1076-898X.8.4.211), [Walker 2007](https://doi.org/10.1002/acp.1291)). Population matters too. For "number of dollars", sighted listeners used an abstract "higher means more" model, while visually impaired listeners pictured a bag of bills landing with a "deep thud" and preferred negative polarity ([Walker & Mauney 2010](https://doi.org/10.1145/1714458.1714459)). Designer intuition misleads as well. In a process-control study the "Bad" mapping ensemble produced the fastest performance and the "Intuitive" one was among the poorest, leading the authors to warn that mappings "that seem intuitive to a sound designer may actually result in less effective performance" ([Walker & Kramer 2005](https://doi.org/10.1145/1101530.1101534)).

| Data concept | Pitch responses +/none/− (Walker 2007, n=19 per block) | Reading | SONARITHM default |
|---|---|---|---|
| Velocity | 18 / 1 / 0 | Strong positive | Positive |
| Pressure, urgency | 13/3/3, 14/3/2 | Positive | Positive |
| Temperature | 11 / 2 / 6 | Majority positive | Positive, flippable |
| Mass | 2 / 0 / 17 | Strong negative | Negative |
| Size | 9 / 0 / 10 | Split | Negative, flagged |
| Number of dollars | 9 / 2 / 8 | Split; blind listeners lean negative | Flagged, recommend testing |
| Attractiveness | 3 / 8 / 8 | Ambiguous | Avoid pitch |

Walker and Nees caution that exponent values "should be treated as guidance" ([Walker & Nees 2011](https://sonification.de/handbook/chapters/chapter2/)), and psychoacoustic channels behave the same way: for roughness and noise, polarity and scaling "are dependent on the data:sound mapping" ([Ferguson & Brewster 2018](https://doi.org/10.1145/3173574.3174185)). The engineering consequences are polarity presets drawn from the table above, a one-click polarity flip saved per user profile, and an optional psychophysical scaling, f = f_ref·(d/d_ref)^(1/m), labelled as a starting point rather than a law.

### Context cues, range and pacing matter as much as mapping

Smith and Walker's study of 160 listeners estimating dollar values is the most actionable single result for AUDAXIS ([Smith & Walker 2005](https://doi.org/10.1002/acp.1146)). Short 10 ms x-axis clicks aided time calibration but did not help with continuous tones. A **dynamic y-axis reference tone** sounded the maximum while prices rose and the minimum while they fell, in a timbre distinct from the data tones. It cut RMS error from **18.09 to 12.14**, whereas a static reference tone at the opening value (RMS 17.46) did not help significantly. Untrained listeners given y-axis context performed as well as trained ones, so the benefits of context and training were "comparable, but not additive".

About 20 minutes of training can cut point-estimation error by 50% ([Walker & Nees 2011](https://sonification.de/handbook/chapters/chapter2/)). A built-in 10–20 minute tutorial is therefore a feature, not documentation.

On range, a narrow **400–1400 Hz** band performed as well as a wide 65–1480 Hz band on line-chart tasks, in a study of three groups of 16. Basic amplitude compensation had no significant effect ([Fitzpatrick & Neff 2023](https://doi.org/10.21785/icad2023.1904)).

On pacing, data density from 1 to 8 points per second had only "a small, but significant, effect" on error, while the number of trend reversals had large effects ([Nees & Walker 2008](https://doi.org/10.1145/1402236.1402237)). Metrical grouping has a floor of about 100 ms between onsets ([London 2012](https://doi.org/10.1093/acprof:oso/9780199744374.001.0001)). Combined with the 8-points-per-second ceiling of the evidence, this suggests switching mark type above roughly 10 events per second, to a continuous glide, granular texture or audification. That threshold is an untested inference. Onset ramps of about 40 ms prevent clicks ([Smith & Walker 2005](https://doi.org/10.1002/acp.1146)).

### Loudness, timbre and space are supporting channels

Loudness should not carry precise values. Pitch and loudness interact: "changes in loudness can influence judgments of pitch change", and loudness increases are judged larger than equal decreases. These effects distorted the data in a stock-market sonification ([Neuhoff, Kramer & Wayand 2002](https://doi.org/10.1037/1076-898X.8.1.17)). Equal-loudness compensation using the ISO 226:2023 contours ([ISO](https://www.iso.org/standard/83117.html)) is still worth enabling by default, so that different pitches sound equally salient. SONARITHM should not claim it improves accuracy.

The remaining channels have clear jobs. Categorical variables belong on timbre ([Walker & Nees 2011](https://sonification.de/handbook/chapters/chapter2/)). Concurrent series should be separated with timbre and stereo position, because pitch is "a less controllable and less reliable attribute for manipulating segregation" while it is busy carrying data. Identifying concurrent earcons gets harder as their number rises, and timbre and onset differences only partly restore it ([McGookin & Brewster 2004](https://doi.org/10.1145/1024083.1024087)); a cap of three to four streams is an inference, not a measured limit. Redundant spatial coding backfires: adding panning and elevation on top of frequency produced "exceptionally poor performance" ([Nees & Walker 2007](https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf)). Spatial dimensions suit motion, since they are used "almost exclusively" for kinematic quantities ([Dubus & Bresin 2013](https://doi.org/10.1371/journal.pone.0082491)). Binaural rendering is a supplement. Generic HRTFs (head-related transfer functions) increase front–back confusions ([Wenzel et al. 1993](https://doi.org/10.1121/1.407089)), and binaural ambisonics worked best "as an extra factor supplementing" other parameters, mainly for musically trained listeners ([Beadling & Vickers 2023](https://doi.org/10.21785/icad2023.3814)).

### Model-based sonification, audification and granular synthesis cover what mapping cannot

**Model-based sonification (MBS)** turns data into an instrument, "while the playing is left to the user". The model makes no sound until someone excites it ([Hermann 2008](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)). One example is crystallization sonification, which grows a "crystal" through high-dimensional data from a seed point to convey clustering and intrinsic dimensionality ([Hermann & Ritter 2002](https://www.icad.org/Proceedings/2002/HermannRitter2002.pdf)). A simpler quick-look method maps PCA components to harmonic partial amplitudes ([Dalziel & Lieck 2025](https://doi.org/10.21785/icad2025.019)).

**Audification** plays dense time series directly as sound. Dombois resampled seismograms at **2,200×**, so near quakes sound like "sharp clicks" and distant ones like "deep rumbles" ([Dombois 2001](https://www.icad.org/Proceedings/2001/Dombois2001.pdf)). Audified seismic events have been categorized "with accuracies of over 90%" ([Walker & Nees 2011](https://sonification.de/handbook/chapters/chapter2/)).

**Granular synthesis** renders distributions and uncertainty ([Williamson & Murray-Smith 2005](https://doi.org/10.1109/MMUL.2005.37)). Grains of 80–300 ms are drawn according to a probability distribution, with 100–1,000 active at once. The result sounds "incoherent when ambiguity is high" and resolves as certainty rises.

These techniques give RESONUM, CURVASONA and TONALENS their scientific footing.

### Monitoring works; "discovery" is the weakest-supported promise

Continuous sonification has its best evidence in peripheral monitoring. In a within-subjects study with 18 participants, a continuous forest soundscape beat visual-only and visual-plus-alert conditions for process monitoring, without significantly hurting the main task ([Hildebrandt, Hermann & Rinderle-Ma 2016](https://doi.org/10.1016/j.ijhcs.2016.06.002)). The SoNSTAR network monitor raised situational awareness with lower NASA-TLX workload than visual techniques ([Debashi & Vickers 2018](https://doi.org/10.1371/journal.pone.0195948)).

Discovery is a different story. In a study of 192 participants counting transit-like signals in light curves, results depended on signal-to-noise ratio (SNR) ([Tucker Brown et al. 2022](https://doi.org/10.1093/mnras/stac2590)):

| SNR | Result with sonification |
|---|---|
| 30–100 | Experts and non-experts alike counted 85–100% correctly |
| 7–10 | Experts did about 2–3× better with visuals, but no better than non-experts with sonification |
| 3–5 | Chance performance |

In a correlation-discrimination task, visualization gave the best sensitivity, while adding sonification "resulted in increased enjoyability" ([Elmquist et al. 2024](https://doi.org/10.1007/s00779-024-01795-8)).

The field's own critics point the same way. Neuhoff observes that "widespread adoption of sonification to display complex data has largely failed to materialize", blames the "indiscriminate mingling of art and science", and recommends designs that "explicitly strive to meet either artistic or scientific goals" ([Neuhoff 2019](https://doi.org/10.21785/icad2019.069)). Supper shows that sonification's public appeal rests on promises of sublime experience ([Supper 2014](https://doi.org/10.1177/0306312713496875)). Screen-reader users, meanwhile, prefer "distinct non-continuous responses generated using oscillators with square waveforms" and want to personalize the sound ([Sharif, Wang & Muongchan 2022](https://doi.org/10.1145/3517428.3550360)).

For SONARITHM, this argues for separate **Analysis** and **Presentation** modes, with the mode recorded in every spec. It also bears on the tagline. "Hear the patterns" is well supported for trends, overviews and monitoring. "Discover what the graphs reveal" should be framed as a complement to looking, not a replacement, until SONARITHM's own studies show otherwise.

| Design decision | Evidence-backed default | Key source | Confidence |
|---|---|---|---|
| Primary value channel | Pitch on a log-frequency (semitone) scale | [Dubus & Bresin](https://doi.org/10.1371/journal.pone.0082491) | Strong |
| Pitch range | About 400–1400 Hz, expandable to 65–1480 Hz | [Fitzpatrick & Neff](https://doi.org/10.21785/icad2023.1904) | Moderate (one study, n=48) |
| Polarity | Positive for generic quantities, negative for mass; flag money and size | [Walker 2007](https://doi.org/10.1002/acp.1291) | Moderate (mostly US undergraduates) |
| Overview rate | 2–4 points/s; warn above about 8 | [Nees & Walker 2008](https://doi.org/10.1145/1402236.1402237) | Moderate |
| Dense data | Switch to glide, granular texture or audification above about 10 events/s | [London 2012](https://doi.org/10.1093/acprof:oso/9780199744374.001.0001) (inference) | Weak |
| X-axis cue | Short clicks at ticks, for discrete notes only | [Smith & Walker](https://doi.org/10.1002/acp.1146) | Moderate |
| Y-axis cue | Dynamic max/min reference tone in a distinct timbre | [Smith & Walker](https://doi.org/10.1002/acp.1146) | Moderate |
| Loudness | Constant while pitch carries data; ISO 226 compensation for consistent salience | [Neuhoff et al. 2002](https://doi.org/10.1037/1076-898X.8.1.17) | Strong |
| Concurrent series | Timbre + pan + staggered onset, about 3–4 streams | [McGookin & Brewster](https://doi.org/10.1145/1024083.1024087) | Moderate; cap inferred |
| Training | Built-in 10–20 minute interactive tutorial | [Walker & Nees 2011](https://sonification.de/handbook/chapters/chapter2/) | Moderate |
| Modes | Separate Analysis and Presentation, recorded in the spec | [Neuhoff 2019](https://doi.org/10.21785/icad2019.069) | Expert opinion |

## Blind users want an overview, then control, then the data itself

The baseline is poor. In a controlled study comparing 36 screen-reader users with 36 non-screen-reader users, screen-reader users extracted information **61% less accurately** (34% versus 87% correct) and spent **211% more time** ([Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)). **A third of the 27 sampled visualizations could not be found by screen readers at all.** Library choice mattered: Google Charts, which appends a hidden data table, reached 73% accuracy, against 17% for D3 and 11% for ChartJS. Interviews revealed a two-stage strategy, a holistic "flyover" of trends and extremes followed by drill-down into points.

Descriptions should match what blind readers value. They ranked statistical (L2) and trend (L3) content most useful, and 63% said descriptions should avoid editorializing so they can interpret the numbers themselves ([Lundgard & Satyanarayan 2021](https://arxiv.org/pdf/2110.04406), [MIT VIS](https://vis.mit.edu/pubs/vis-text-model/)).

The systems that work share a pattern. **Zong et al.** co-designed with a blind co-author for six months. They defined three dimensions (structure; structural, spatial and targeted navigation; and description) and found that conveying the structure's boundaries reduced cognitive load ([Zong et al. 2022](https://arxiv.org/pdf/2205.04917)). **Chart Reader** came from parallel co-design with 10 screen-reader users over five months. It added a five-branch hierarchy, a "bonk" earcon at boundaries and a "drip" for invalid actions, landmark jump keys, and value announcements interleaved with tones; autonomy was the overarching value ([Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)). **MAIDR** lets users toggle braille, text, sonification and review modes on one cursor ([MAIDR paper](https://arxiv.org/html/2403.00717v1)). Its 11 blind participants gave SUS scores of 81.4 for bar plots, 75.5 for heat maps, 74 for box plots and 70.3 for scatter plots. Most worked whole-to-part, with sonification or braille for the overview and text for detail, and preferences tracked ability: a participant with high-frequency hearing loss relied on braille and text.

**Umwelt**'s coequal modalities let users switch between overview and detail ([Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)). **VoxLens** reported 122% higher accuracy and 36% less interaction time, though its comparison group was participants from an earlier study rather than a concurrent control ([VoxLens](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf), [UW News](https://washington.edu/news/2022/06/01/voxlens-adding-one-line-of-code-can-make-some-interactive-visualizations-accessible-to-screen-reader-users)). **Audio data narratives**, which interleave description and sonification, helped 16 screen-reader users gain significantly more insights ([Siu et al. 2022](https://doi.org/10.1145/3491102.3517678)).

Together these define AUDAXIS's core: a hierarchical keyboard structure with local, lateral, spatial and targeted moves; a single "play overview" command; the current value announced on pause; independently toggleable modalities with adjustable verbosity; an always-available data table; and per-user profiles, because differences in hearing, braille literacy and statistical background are too large for any one "best" mapping.

Native desktop plumbing differs from the web. AccessKit gives Rust an abstraction over Windows UI Automation, macOS NSAccessibility and Linux AT-SPI ([AccessKit](https://github.com/AccessKit/accesskit)). Announcements need UIA notification events, available from Windows 10 version 1709 ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationcoreapi/nf-uiautomationcoreapi-uiaraisenotificationevent)), and `announcementRequested` on macOS, which developers report VoiceOver sometimes skips ([Apple docs](https://developer.apple.com/documentation/appkit/nsaccessibility-swift.struct/notification/announcementrequested.md)). The primary desktop screen readers are JAWS (40.5%), NVDA (37.7%) and VoiceOver (9.7%) ([WebAIM Survey #10](https://webaim.org/projects/screenreadersurvey10/)), so Windows support is non-negotiable. NVDA can duck other applications' audio and can split its speech and other apps onto separate stereo channels ([NV Access](https://www.nvaccess.org/?p=30731), [NVDA issue #16052](https://github.com/nvaccess/nvda/issues/16052)), so stereo panning must never carry information alone.

Speech should go through the user's own screen reader by default and be sequenced with the tones, not overlapped. Where overlap is unavoidable, keep sonification well below speech; WCAG 1.4.7's 20 dB guidance is a reasonable target. Single-letter shortcuts must be remappable, switchable off, or active only when the chart has focus, per WCAG 2.1.4 ([WCAG 2.2](https://www.w3.org/TR/WCAG22/)).

An audio-first tool has a second accessibility audience. Deaf and hard-of-hearing users need a visual, text or braille equivalent for every sonified insight. Section 508's functional performance criteria for use without hearing make that explicit ([Access Board](https://www.access-board.gov/ict/)).

Education is a natural market with a small evidence base:

| Project | Evidence |
|---|---|
| SonicFunction | Combined discrete and continuous tones with derivative and sign cues; tested with 14 blind and partially sighted students ([Bielefeld](https://pub.uni-bielefeld.de/record/2017190)) |
| AudioFunctions | 7 blind participants understood graphs better than with tactile paper ([Springer](https://www.springerprofessional.de/audiofunctions-eyes-free-exploration-of-mathematical-functions-o/2191074)) |
| AudioFunctions 2.0 | Adds cues for undefined and off-screen values ([poster](https://univda.iris.cineca.it/retrieve/59b6347a-cb21-4176-8235-e9e2fb51c5ef/Spagnuolo%20et%20al.%20%282026%29_poster_ICTMT17.pdf)) |
| NASA "A Universe of Sound" | Significant self-reported learning gains among 3,184 sighted and blind or low-vision participants ([Arcand et al. 2024](https://arxiv.org/pdf/2403.18082)) |

One gap matters for the tagline. **No blind-user evaluation of node-link or network sonification was found**, and graph concepts such as centrality or edge weight have no published polarity data. Data Navigator's node-edge navigation model is the closest validated analogue ([Data Navigator](https://arxiv.org/pdf/2308.08475)). If "graphs" means networks, SONARITHM is entering uncharted territory.

## Nine engines should compile through one specification, shipped in three waves

Hermann's reusability criterion and Erie's separation of compiler from player point to the same architecture. Every sub-engine should feed one intermediate representation (substrate, marks, channels), which compiles to an event timeline rendered by one engine.

Two warnings reinforce this. Firewheel's author cites "more feature creep than what the author originally anticipated" as one reason for handing that audio engine to Bevy ([Firewheel README](https://github.com/BillyDM/Firewheel)). A 2025 history of sonification tools since 1990 identifies sustainability as a central challenge ([Budziszewski & Groß-Vogt 2025](https://doi.org/10.21785/icad2025.071)).

The nine names should therefore be product surfaces over a shared set of crates, not nine codebases. Each engine contributes one of three things: a front end that turns a source into a spec, a mark type, or an analysis pass.

| Sub-engine | Role in the shared pipeline | Closest prior art | Evidence base | Realistic first scope | Wave |
|---|---|---|---|---|---|
| NUMISONA | Spec compiler from data to channels | Erie, Highcharts, STRAUSS | Strong (Walker; Smith & Walker; Fitzpatrick & Neff) | Pitch, time, duration, timbre and pan channels; polarity presets with flip; log, mel and ERB scales; psychophysical exponent option; Erie-spec import | 1 |
| AUDAXIS | Navigation, context tracks, spatial output | Chart Reader, MAIDR, Olli, Apple `AXChart` | Strong HCI evidence; weak for HRTF | Hierarchical keyboard tree, tick clicks, dynamic reference tone, earcons, stereo; SOFA binaural later | 1 (HRTF in 3) |
| SONOTRACE | Time series and live ingest | MAIDR live mode, Chart2Music monitor mode | Moderate (Hildebrandt n=18; SoNSTAR) | CSV and Parquet series; WebSocket, MQTT, serial and OSC; rate-limited announcements; ambient monitoring preset | 1–2 |
| PLOTSONA | Chart ingestion front end | SAS GA (no image input), ChartDetective, WebPlotDigitizer | Strong for exact and vector sources; weak for ML on line charts | Office XML caches, live web-chart objects, SVG and vector PDF with auto-calibration; WPD-style raster with human correction | 2 (ML in 3) |
| CURVASONA | Function plots and audification | Desmos Audio Trace, HSS mathJS, SonicFunction | Small-sample studies; Dombois | Expressions with derivative, sign and zero cues; points of interest; asymptote and undefined cues; audification speed-up calculator | 2 |
| AXONOTE | Musical quantization, MIDI and OSC output | Highcharts MIDI, TwoTone, Manifest, STRAUSS | Mixed: aesthetic gains, accuracy costs | Scale quantization in Presentation mode only; SMF export; live MIDI and OSC to DAWs and VCV Rack | 2 |
| TONALENS | Multivariate exploration lens | Apple four-axis audio graphs, Erie, play-splom (dormant) | Interference evidence; stream cap unproven | Linked multimodal lens; timbre + pan + onset streams; PCA-to-partials quick-look; granular density | 3 |
| RESONUM | Model-based sonification | sc3nb/sonecules (dormant) | Thin, mostly one lab | Data-sonogram and crystallization models that sound only on interaction | 3 |
| ECHOMETRIC | Pattern analytics and perceptual metrics | Chart2Music outliers, Desmos points of interest, Highcharts conditional earcons | Weak; no validated comfort thresholds | Outlier, changepoint and periodicity detection as earcons; mapping "lint" for rate, critical-band spacing and loudness; evaluation harness | 2–3 |

**Wave 1** is the minimum credible product. On the desktop, it must match the landscape's table stakes: keyboard navigation, screen-reader output, speech, speed control, switching between series, and line, bar, scatter, box, histogram and heat-map charts, all under a free licence. It must also add what nobody else offers: a reproducible spec, a CLI, a Python wheel, and WAV, FLAC and MIDI export.

**Wave 2** brings the desktop app to general availability, with exact chart ingestion, function sonification and musical output.

**Wave 3** carries the research-heavy engines: multivariate, model-based, ML extraction and HRTF. Their defaults should not be trusted until SONARITHM's own studies back them.

SONARITHM's genuine differentiators, roughly in order of defensibility, are one deterministic core powering desktop, CLI, Python and WebAssembly; a spec plus provenance exported with every render, which makes Hermann's reproducibility operational in a way no surveyed tool does; chart ingestion that prefers exact data and labels every estimate; accessible streaming and monitoring; maintained model-based and multivariate engines where the alternatives are dormant; and a perceptual lint and benchmark harness. Pitch-time mapping, basic keyboard navigation, LLM chat (which MAIDR already has) and a free licence are table stakes, not differentiators.

### PLOTSONA must prefer exact data and label every estimate

Many "chart images" are not really images. Office charts store plotted values in `c:numCache` alongside an embedded workbook ([Microsoft Open XML](https://learn.microsoft.com/en-us/previous-versions/office/developer/office-2010/ee880031(v=office.14))). Web charts expose their data through D3's `__data__` ([D3](https://d3js.org/d3-selection/joining)), Vega's `view.data(name)` ([Vega](https://vega.github.io/vega/docs/api/view/)) and the `Highcharts.charts` array ([Highcharts API](https://api.highcharts.com/class-reference/Highcharts#.charts)). For vector PDFs, ChartDetective recovered data, including hidden data, with **78% lower relative error** than raster approaches, in under four minutes per chart ([ChartDetective](https://hal-lara.archives-ouvertes.fr/CRISTAL-LOKI/hal-04017638v1)).

For true rasters, WebPlotDigitizer's documented workflow is the template: calibrate axes, mask by colour, run chart-type-specific extraction, then correct by hand ([WPD docs](https://automeris.io/docs/digitize/)). SONARITHM should re-implement this from the documented behaviour rather than port WPD's code. The current repository describes the frontend as AGPL-3.0 and "AI Assist" as closed-source, and an older note calls v5 closed ([WPD upstream](https://github.com/automeris-io/WebPlotDigitizer), [fork](https://github.com/netopt/WebPlotDigitizer)).

Machine learning is far weaker than ChartQA-style leaderboards suggest. On the 2026 WB-ChartExtract benchmark, a preprint, line-chart RMS-F1 scores were ([arXiv 2605.27298](https://arxiv.org/html/2605.27298)):

| Model | Line-chart RMS-F1 |
|---|---|
| Gemini 2.5 Pro | **87.68** |
| GPT-5.1 | 38.53 |
| Llama 4 Scout | 28.10 |
| TinyChart (3B) | 18.32 |

TinyChart's 18.32 compares with the 95.2 it scores on ChartQA tables. On real epidemic curves (EpiCurveBench), the best model reached only **52.3%** on the benchmark's similarity score, against 90.9% agreement between careful human annotators ([arXiv 2605.27195](https://arxiv.org/html/2605.27195)). Hallucination is endemic: GPT-5 scored 34.46% on ChartHal ([arXiv 2509.17481](https://arxiv.org/abs/2509.17481)).

Three findings point to a hybrid design. Biases tied to y-axis tick formats are significant, and telling models about the y-axis improves some of them ([arXiv 2604.24987](https://arxiv.org/pdf/2604.24987)). Self-ensembling improves results by up to about 23% relative, and disagreement across samples correlates with error at ρ ≈ −0.30 to −0.37 ([arXiv 2605.27298](https://arxiv.org/html/2605.27298)). OneChart's self-consistency score lifts precision from 72.02 to 81.97 when low-confidence outputs are filtered out ([OneChart](https://arxiv.org/html/2404.09987)).

The design that follows: calibrate axes classically, let ML find marks, and map pixels to values through the calibration. Every point carries a provenance class: exact, vector-calibrated, CV-estimated, ML-estimated, cloud-estimated or user-edited. Descriptions state numbers without hedging only when they are exact or user-edited. Rendering low-confidence regions with a distinct timbre is promising but untested with blind users.

### LLM features should phrase facts, not produce them

Lundgard and Satyanarayan recommend automating L1 (encoded properties) and L2 (statistics), and leaving L4 (interpretation) to humans ([MIT VIS](https://vis.mit.edu/pubs/vis-text-model/)). SONARITHM's description pipeline should follow suit. It should compute L1 and L2 deterministically from the data, let an LLM phrase L3 from a precomputed fact list and then check every number in its output against that list, and offer L4 only on request, labelled as interpretation.

Questions about the data should be answered by having a model write a small query over the extracted table, never by reading pixels. TinyChart's Program-of-Thought approach improved numeric answers this way ([TinyChart](https://arxiv.org/html/2404.16635)). DePlot's chart-to-table step paired with an LLM gained 24.0% on human-written ChartQA questions ([DePlot](https://arxiv.org/abs/2212.10505)).

Mapping suggestions should come from a transparent rule engine, with an LLM optionally ranking or explaining them. AI in sonification "remains limited" ([Hommel & Hermann 2025](https://doi.org/10.21785/icad2025.037)), and no peer-reviewed evaluation of LLM-suggested mappings exists.

Privacy requires local processing by default, because cloud retention terms vary:

| Provider | Retention and training terms |
|---|---|
| OpenAI API | No training on customer data by default; abuse-monitoring logs kept up to 30 days ([OpenAI](https://developers.openai.com/api/docs/guides/your-data)) |
| Anthropic API | Inputs and outputs deleted within 30 days, with exceptions ([Anthropic](https://privacy.claude.com/en/articles/7996866-how-long-do-you-store-my-organization-s-data)) |
| Gemini API, unpaid tier | Prompts may be used for product improvement and human review ([Gemini terms](https://ai.google.dev/gemini-api/terms-archive/terms_05_02_24)) |

Cloud extraction must therefore be explicit, opted into per chart, and limited to paid tiers.

Local models with clean licences include Qwen2.5-VL-7B (Apache-2.0, ChartQA 87.3), Qwen3-VL-8B (Apache-2.0), Florence-2 (MIT) as a tick-label OCR reader, and DePlot (Apache-2.0, 282M parameters) ([Qwen2.5-VL](https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct), [Qwen3-VL](https://huggingface.co/Qwen/Qwen3-VL-8B-Instruct), [Florence-2](https://huggingface.co/microsoft/Florence-2-large), [DePlot](https://huggingface.co/google/deplot)).

SONARITHM should not bundle ChartGemma, whose code is GPL-3.0 ([GitHub](https://github.com/vis-nlp/ChartGemma)), or Moondream 3 Preview, whose weights use the Business Source License 1.1 ([mirror](https://huggingface.co/LibreYOLO/LibreMoondream3)).

## A Rust engine under a webview shell is the accessible stack

### The audio core should own its graph and treat devices as plug-ins

**cpal 0.18.2** (16 August 2026) is the de facto Rust audio I/O layer, with 8.5 million downloads in 90 days ([crates.io](https://crates.io/crates/cpal)). It added native PipeWire and PulseAudio hosts, latency-aware timestamps and xrun reporting ([cpal CHANGELOG](https://github.com/RustAudio/cpal/blob/HEAD/CHANGELOG.md)). It still lacks WASAPI and CoreAudio exclusive mode ([cpal #106](https://github.com/RustAudio/cpal/issues/106)), and 0.19 will bring breaking changes.

SONARITHM should hide cpal behind its own `AudioBackend` trait and own a backend-agnostic render graph. The same `process()` function then drives both real-time playback and offline export, which runs in a plain loop at a fixed block size. That is what makes renders reproducible.

**Firewheel 0.14** is the closest reusable graph. It compiles a DAG into a schedule, passes events over a real-time-safe channel, and keeps seconds, sample and musical clocks ([Firewheel design doc](https://github.com/BillyDM/Firewheel/blob/HEAD/DESIGN_DOC.md)). It is pre-1.0, though, and a Bevy-side audit found "nontrivial UB that must be resolved" ([Bevy audit](https://hackmd.io/@bevy/rkwFscXObx)). It should be trialled behind the trait, not built on.

The rest of the core:

| Need | Choice | Notes |
|---|---|---|
| Synthesis | **fundsp 0.23** | Preallocation, seeded noise, PolyBLEP oscillators and FIR resampling ([fundsp](https://github.com/SamiPerttu/fundsp)). Glicol, knyst and oddio are stale and synthizer-rs is archived, so avoid them. |
| Real-time path | **rtrb**, **triple_buffer**, **basedrop** | rtrb provides wait-free ring buffers; triple_buffer (MPL-2.0) carries meters; basedrop keeps deallocation off the audio thread ([rtrb](https://crates.io/crates/rtrb), [triple_buffer](https://crates.io/crates/triple_buffer), [basedrop](https://github.com/micahrj/basedrop)). Debug builds wrap the callback in `assert_no_alloc`. |
| Granular, wavetable and modal synthesis | Custom nodes | No maintained crates exist, so these become small in-house nodes. |
| Spatial audio | **sofar 0.4.0** | Pure-Rust SOFA HRTF rendering under a permissive licence ([sofar](https://github.com/andreiltd/sofar)) |

### Data, export and scripting choices are mostly licence decisions

Apache **Arrow 60** should be the internal data model, giving zero-copy exchange with Python and Polars ([crates.io](https://crates.io/crates/arrow)). Polars itself should stay optional, because its versioning policy says Rust API breaking changes "are not deprecated first" ([Polars](https://docs.pola.rs/development/versioning/)).

| Job | Crate | Notes |
|---|---|---|
| WAV export | **hound** | Last release 2023 but stable ([hound](https://crates.io/crates/hound)) |
| FLAC export | **flacenc** | Pure Rust ([flacenc](https://crates.io/crates/flacenc)) |
| MIDI files | **midly** | ([midly](https://crates.io/crates/midly)) |
| Decoding | **symphonia** | MPL-2.0 ([Symphonia](https://github.com/pdeljanov/Symphonia)) |
| Live MIDI | **midir** | ([midir](https://crates.io/crates/midir)) |
| OSC | **rosc** | ([rosc](https://crates.io/crates/rosc)) |
| MP3 export | mp3lame-encoder | LGPL-3.0, so optional feature only ([crates.io](https://crates.io/crates/mp3lame-encoder)) |

For expressions, **evalexpr has been AGPL-3.0 since 12.0.0** ([evalexpr CHANGELOG](https://github.com/ISibboI/evalexpr/blob/HEAD/CHANGELOG.md)), which rules it out. **exmex** (MIT/Apache) does symbolic differentiation, which suits CURVASONA's derivative cues. It has a single maintainer, so wrap it behind a facade ([exmex](https://crates.io/crates/exmex)). **rhai** handles sandboxed user scripts ([rhai](https://crates.io/crates/rhai)).

ECHOMETRIC's analytics can use **statrs** and Grafana's **augurs** ([augurs](https://github.com/grafana/augurs)). Avoid the GPL-3.0 `anomaly_detection` crate ([crates.io](https://crates.io/crates/anomaly_detection)).

The **tts** crate routes speech to NVDA or SAPI via Tolk, but its last release was in July 2024 ([tts-rs](https://github.com/ndarilek/tts-rs)). It speaks through the OS engine, so speech cannot be mixed into offline exports. Exports should carry a sidecar cue track instead.

Third-party engines fit **wasmtime 49 + WIT**, now that WASI 0.3 shipped in June 2026 ([WASI P3](https://wasi.dev/releases/wasi-p3)). Plugins should run only at compile time, never in the audio callback. Wasmtime's MSRV of 1.96 belongs in a leaf crate ([wasmtime](https://crates.io/crates/wasmtime)).

A cargo-deny licence allowlist catches every one of these traps automatically.

### Tauri wins the shell decision on accessibility, with Electron as the hedge

The deciding criterion is screen-reader quality on three operating systems, and native Rust GUI toolkits are not there yet. egui 0.35 turns AccessKit on by default but lacks live regions; Slint exposes text inputs incompletely and is licensed GPLv3 or commercial; iced has no AccessKit support in stable releases; and the released gpui has no accessibility support at all ([GoldStrikeArch survey](https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/data/stack-rows.md), [Slint licensing](https://slint.dev/pricing)). AccessKit itself has a "bus factor 2 and lapsed grant funding".

A webview shell inherits the browser engines' mature accessibility trees and the whole ARIA pattern ecosystem. **Tauri 2.12** (26 September 2026, MSRV 1.90) keeps the Rust core in-process. It adds capability-scoped permissions and signed updates whose verification cannot be disabled ([Tauri 2.12](https://v2.tauri.app/blog/tauri-2.12/), [updater](https://v2.tauri.app/plugin/updater/)).

Tauri carries two risks. First, its accessibility is inherited rather than committed. Its accessibility tracking issue has been open since 2019, and a blind developer calls it "about as accessible as the underlying webview" ([Tauri #207](https://github.com/tauri-apps/tauri/issues/207)). Second, Linux rendering is weak. One user reported about 40 fps in WebKitGTK versus about 240 fps after porting to Electron ([discussion #8524](https://github.com/orgs/tauri-apps/discussions/8524)), and NVIDIA systems need DMABUF workarounds ([Tauri Linux graphics](https://v2.tauri.app/develop/debug/linux-graphics/)). **Electron 44** (Chromium M152) is the hedge ([Electron schedule](https://releases.electronjs.org/schedule)). A two-week prototype gate should therefore measure Channel throughput on all three operating systems, WebGL2 spectrogram and million-point waveform frame rates in WebKitGTK, and NVDA, VoiceOver and Orca walkthroughs of a React Aria tree, slider and toolbar before the project commits. If Linux fails, swap the shell and keep the frontend.

Inter-process communication must avoid Tauri's JSON events on hot paths. Events "are not designed for low latency or high throughput", while channels are "fast and deliver ordered data" ([Tauri docs](https://v2.tauri.app/develop/calling-frontend/)). Binary data should travel as `ipc::Response` buffers ([Tauri docs](https://v2.tauri.app/develop/calling-rust/)); one informal test found 10 MB took about 5 ms on macOS but about 200 ms on Windows ([discussion #11915](https://github.com/orgs/tauri-apps/discussions/11915)). In practice the playhead should travel as an audio-clock anchor that the frontend extrapolates in its animation loop, meter values should be batched into binary frames, and downsampling should happen in Rust with M4 or LTTB ([M4](https://www.vldb.org/pvldb/vol7/p797-jugel.pdf), [LTTB](https://skemman.is/handle/1946/15343)).

### React Aria is the decisive frontend dependency

React 19.2 with **React Aria Components** is the only mainstream headless stack with production Tree, Virtualizer, expandable Table rows, GridList, accessible drag-and-drop, Slider and Toolbar ([React 19.2](https://react.dev/blog/2025/10/01/react-19-2), [RAC 2025-03-05](https://react-spectrum.adobe.com/releases/2025-03-05.html), [RAC v1.17](https://react-aria.adobe.com/releases/v1-17-0)). Base UI has no Tree or grid ([Base UI](https://base-ui.com/react/overview/releases)), and Solid 2.0 is still a release candidate ([Solid](https://github.com/solidjs/solid/releases)).

The rest of the frontend follows from performance and accessibility. **uPlot** renders 166,650 points in 25 ms from about 50 KB but struggles beyond about 100,000 points in view ([uPlot](https://github.com/leeoniya/uPlot)). Dense views and spectrograms therefore need a custom WebGL2 layer, with WebGPU optional because Linux lacks it. The spec editor should be CodeMirror 6 driven by the schemars-generated JSON Schema. Transport state belongs in Zustand plus XState v5. Theming should use CSS custom-property tokens generated in OKLCH that honour `forced-colors` and `prefers-contrast`. Fixed regions with F6 cycling beat free docking, because dockview's spatial keyboard navigation and keyboard docking are Enterprise-only ([dockview licence](https://dockview.dev/docs/overview/licence)).

Ableton Live 12 shows a dense music tool can be screen-reader accessible when tested with VoiceOver, NVDA, Narrator and JAWS ([Ableton](https://help.ableton.com/hc/en-us/articles/11550373507868)). The visual identity should read as a calibrated instrument: IBM Plex Sans and Mono, tabular numerals, neutral dark grays and one accent. It should avoid the indigo gradients and default-template look that mark generic AI-generated UI ([Linear](https://linear.app/now/how-we-redesigned-the-linear-ui), [prg.sh](https://prg.sh/ramblings/Why-Your-AI-Keeps-Building-the-Same-Purple-Gradient-Website)).

### Bindings and workspace layout

**Bindings.** Python uses PyO3 0.30.1, which supports Python 3.15, with maturin 1.15 and abi3 wheels ([pyo3](https://crates.io/crates/pyo3), [maturin](https://crates.io/crates/maturin)). wasm-bindgen 0.2.129 now lives in its own GitHub organization after the rustwasm organization was sunset ([Inside Rust](https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org)). Given wasm-pack's maintainer churn, drive `wasm-bindgen-cli` directly.

**Workspace.** Follow tokenizers and ruff: a core crate for the spec, compiler and timeline; sibling crates for DSP, engine, the cpal backend, export, data, digitization, plugins and the CLI; and Python bindings, wasm bindings and the desktop app alongside them.

**Toolchain.** Use edition 2024 and a core MSRV around 1.88, matching Arrow and AccessKit, while Rust stable is 1.99.1 ([Rust releases](https://github.com/rust-lang/rust/releases)). Generate TypeScript types with ts-rs, since specta remains at 2.0.0-rc.25 ([specta](https://crates.io/crates/specta)).

| Layer | Recommendation (Oct 2026) | Key reason | Main risk / fallback |
|---|---|---|---|
| Audio I/O | cpal 0.18.2 behind own trait; ASIO opt-in | De facto standard; native PipeWire | No exclusive mode; 0.19 breaking changes |
| Engine/graph | Own backend-agnostic graph; Firewheel 0.14 on trial | Same code path for real-time and offline export | Firewheel pre-1.0 with audited UB |
| DSP | fundsp 0.23 + custom granular and wavetable nodes; rubato; rustfft | Maintained, preallocating, seeded noise | No maintained granular crate |
| Real-time communication | rtrb, triple_buffer, basedrop, atomic_float | Wait-free, allocation-free audio thread | basedrop is small and quiet |
| Data | Arrow 60, csv, calamine, jiff; Polars optional | Zero-copy to Python and JS | Polars breaks without deprecation |
| Export/decode | hound, flacenc, midly; symphonia | Pure Rust, deterministic | MP3 and Opus need C libraries; LAME is LGPL |
| Expressions/scripts | exmex; rhai | Permissive; derivatives; sandboxed | evalexpr is AGPL; exmex bus factor |
| ML | tract default; ort opt-in; llama.cpp sidecar | Small default install | ort still 2.0.0-rc.14; model licences |
| Shell | Tauri 2.12 | Webview accessibility, in-process Rust | WebKitGTK on Linux; Electron fallback |
| UI | React 19.2 + React Aria Components; Zustand + XState | Only headless library with Tree, Table, GridList, drag-and-drop | More assembly than styled kits |
| Plots | uPlot + custom WebGL2; M4/LTTB in Rust | Fast; universal across webviews | WebGL needed above 100,000 in-view points |
| Bindings | PyO3 0.30 + maturin; wasm-bindgen | Python and browser parity | wasm-pack maintainer churn |
| Plugins | wasmtime 49 + WIT, compile-time only | Typed, sandboxed, language-agnostic | MSRV 1.96; not real-time-safe |

ML runtime sources: [ort releases](https://github.com/pykeio/ort/releases), [tract-onnx](https://crates.io/crates/tract-onnx), [llama.cpp multimodal](https://github.com/ggml-org/llama.cpp/blob/master/docs/multimodal.md).

## Industry-grade means automated governance and audited accessibility from day one

### Process and release automation

The 2026 Rust-ecosystem default is trunk-based development with squash merges and Conventional Commits; GitFlow's own author no longer recommends GitFlow for continuously delivered software ([nvie.com](https://nvie.com/posts/a-successful-git-branching-model/)). **release-plz** opens release PRs, writes Keep-a-Changelog entries with git-cliff, runs cargo-semver-checks and tags each crate ([release-plz](https://release-plz.dev/docs)). Crates publish through crates.io Trusted Publishing over OIDC, launched July 2025, with the first release published manually ([Rust blog](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07)).

Decisions belong in MADR 4 ADRs under `docs/decisions/` ([MADR](https://adr.github.io/madr/)). Cross-cutting changes such as the spec format or the plugin WIT world go through Rust-style RFCs ([rust-lang/rfcs](https://github.com/rust-lang/rfcs)), and docs follow Diátaxis ([diataxis.fr](https://diataxis.fr/)). Agent instructions belong in **AGENTS.md**, a format stewarded by the Agentic AI Foundation under the Linux Foundation ([agents.md](https://agents.md/)). Claude Code reads it natively from v2.1.277, and its docs advise a `CLAUDE.md` that imports `@AGENTS.md` rather than a symlink where Windows contributors are involved ([Claude Code docs](https://code.claude.com/docs/en/memory)).

uv's release workflow is the model to copy: manual dispatch behind a protected-environment approval, actions pinned to commit SHAs, top-level `permissions: {}` and build-provenance attestations ([uv release.yml](https://github.com/astral-sh/uv/blob/main/.github/workflows/release.yml)).

### CI/CD and signing

A GitHub Actions matrix (Ubuntu, Windows, macOS) should run formatting, clippy with `-D warnings`, nextest, cargo-deny, semver-checks, an MSRV job, a wasm32 build of the core and a Python wheel smoke test on every PR.

Releases split by artifact. Libraries go through release-plz. The desktop app goes through `tauri-action`, which generates the updater's `latest.json` ([Tauri updater](https://v2.tauri.app/plugin/updater/)). Wheels go through `maturin-action`. dist v0.33.0 added Azure Artifact Signing for CLI installers ([cargo-dist releases](https://github.com/axodotdev/cargo-dist/releases)), but downstream teams question its maintenance ([vig-os issue](https://github.com/vig-os/devkit/issues/1774)), so treat it as a generator you can eject from.

Signing costs money and paperwork:

| Platform | Mechanism | Notes |
|---|---|---|
| Windows | Azure Artifact Signing (GA January 2026) | From $9.99/month for 5,000 signatures. Available to individuals in the US and Canada and to organizations in the EU and UK ([DevClass](https://devclass.com/2026/01/14/code-signing-windows-apps-may-be-easier-and-more-secure-with-new-azure-artifact-service/)). EV certificates no longer grant instant SmartScreen reputation ([Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/)). |
| macOS | Developer ID certificate plus notarization | Apple Developer Program costs US$99/year ([Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/)) |
| Linux | GPG-signed AppImages | The signature "is not verified automatically" ([Tauri Linux signing](https://v2.tauri.app/distribute/sign/linux/)) |

The Tauri updater's private key should have two custodians, because losing it means existing installs can never be updated.

### Testing a sound engine

Testing must treat the compiled **event timeline**, not audio bytes, as the canonical artifact. On the Rust side that means insta snapshots of the timeline and golden-audio comparisons with tolerances (RMS error and LUFS via ebur128), because floating-point and SIMD results differ across CPUs ([ebur128](https://github.com/sdroege/ebur128)). It also means proptest on spec-to-timeline invariants, cargo-fuzz on the parser and importers, and Miri plus shuttle on lock-free code ([shuttle](https://crates.io/crates/shuttle)).

On the frontend it means Vitest 4 browser mode with screenshot assertions ([Vitest 4](https://main.vitest.dev/blog/vitest-4)), Storybook 10 with its axe accessibility addon ([Storybook](https://storybook.js.org/docs/writing-tests/integrations/vitest-addon)) and Playwright with Tauri IPC mocking ([Tauri mocking](https://v2.tauri.app/develop/tests/mocking/)). Real-app tests need WebdriverIO's Tauri service, because the official `tauri-driver` does not support macOS ([Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/)).

Automation cannot judge what a screen reader announces. Every release needs a manual pass with NVDA, JAWS and Narrator on Windows, VoiceOver on macOS, Orca on X11 and Wayland, and a refreshable braille display.

### Security, supply chain and the Cyber Resilience Act

A credible baseline starts with OpenSSF Scorecard, which rates Dangerous-Workflow as critical and Token-Permissions and Signed-Releases as high ([Scorecard checks](https://github.com/ossf/scorecard/blob/main/docs/checks.md)). It adds the OSPS Baseline's 41 security requirements across three maturity levels ([OSPS Baseline](https://baseline.openssf.org/)), with the matching Best Practices baseline badges ([OpenSSF](https://openssf.org/blog/2026/02/25/getting-an-openssf-baseline-badge-with-the-best-practices-badge)).

Releases should carry SLSA v1.2 provenance, which added a Source Track ([SLSA](https://slsa.dev/blog/2025/11/announce-slsa-v1.2)). They should also be immutable releases, GA since 28 October 2025; because that setting is all-or-nothing, nightlies belong elsewhere ([GitHub changelog](https://github.blog/changelog/2025-10-28-immutable-releases-are-now-generally-available/), [community discussion](https://github.com/orgs/community/discussions/178351)). CycloneDX SBOMs, cargo-auditable binaries and a cargo-deny allowlist close the dependency loop; the allowlist denies AGPL and GPL, flags LGPL and allows file-level MPL. Secrets and disclosure need repository-level push protection, which is not on by default ([GitHub Docs](https://docs.github.com/en/code-security/secret-scanning/introduction/about-push-protection)), plus private vulnerability reporting and a SECURITY.md.

The EU Cyber Resilience Act's manufacturer reporting duties began **11 September 2026**, and the Act applies in full from **11 December 2027** ([noze.it](https://www.noze.it/en/insights/cyber-resilience-act-sbom/)). A non-commercial academic project is generally not a "manufacturer". A legal person that systematically supports FOSS intended for commercial use becomes a lighter-touch "steward", required to maintain a cybersecurity policy and cooperate with authorities ([OpenSSF CRA one-pager](https://policy.openssf.org/CRA/stewards-one-pager.html)). Doing the steward basics now costs little, and companies embedding SONARITHM will ask for SBOMs and a security contact.

### Licensing and contribution

`MIT OR Apache-2.0` is the Rust norm, recommended by the Rust API Guidelines for maximum compatibility ([Rust API Guidelines](https://github.com/rust-lang/api-guidelines)). Apache-2.0 contributes a patent grant. MIT is retained because Apache-2.0 is incompatible with GPLv2 ([Rust internals](https://internals.rust-lang.org/t/rationale-of-apache-dual-licensing/8952)).

The rest of the licensing setup is conventional: a DCO rather than a CLA, REUSE/SPDX headers ([REUSE](https://reuse.software/spec/)), OFL texts for bundled fonts, Lucide's ISC notice plus Feather's MIT notice ([Lucide](https://lucide.dev/license)), and Contributor Covenant 3.0, released July 2025 with a restorative enforcement model ([Ethical Source](https://ethicalsource.dev/blog/contributor-covenant-3)). SONARITHM should avoid VST3 support through nih-plug, whose VST3 bindings carry GPLv3 obligations ([nih-plug](https://github.com/robbert-vdh/nih-plug)). Example datasets and stimuli should be licensed separately, for instance under CC BY 4.0.

### Accessibility compliance is a conformance programme

The practical target is **WCAG 2.2 AA**, which became ISO/IEC 40500:2025 in October 2025 ([W3C](https://www.w3.org/WAI/news/2025-10-21/wcag22-iso)). It applies to desktop software through the WCAG2ICT Note ([WCAG2ICT](https://www.w3.org/TR/wcag2ict-22/)). The surrounding rules:

| Standard or law | Status as of October 2026 | Relevance to SONARITHM |
|---|---|---|
| EN 301 549 | v4.1.1 (WCAG 2.2) published 2 September 2026 but not yet cited in the Official Journal, so v3.2.1 (WCAG 2.1) remains the legal reference ([DWT](https://www.dwt.com/insights/2026/09/european-accessibility-act-ict-standards-update)) | Universities procuring against it will ask for conformance claims |
| Section 508 | Still incorporates WCAG 2.0 AA, plus Chapter 5 requirements for interoperating with assistive technology and honouring platform preferences ([Access Board](https://www.access-board.gov/ict/)) | US federal procurement |
| European Accessibility Act | Applies since 28 June 2025; product categories cover hardware and operating systems ([Directive 2019/882](https://www.legislation.gov.uk/eudr/2019/882/chapter/I)) | A free standalone sonification app is probably outside scope (legal analysis, not advice) |
| ADA Title II | WCAG 2.1 AA for state and local government, with deadlines extended to April 2027 and April 2028 ([Duane Morris](https://www.duanemorris.com/alerts/doj_extends_ada_title_ii_digital_accessibility_deadlines_one_year_0426.html)) | Matters when public universities deploy SONARITHM |
| WCAG 3.0 | Still a Working Draft ([W3C](https://www.w3.org/TR/wcag-3.0/)) | Not citable |

Some criteria bite especially hard on a sonification tool ([WCAG 2.2](https://www.w3.org/TR/WCAG22/)):

| Criterion | What it requires here |
|---|---|
| 1.4.2 Audio Control | A volume control independent of system volume |
| 2.1.4 Character Key Shortcuts | Single-key shortcuts that can be switched off or remapped |
| 2.2.2 Pause, Stop, Hide | A way to pause streaming displays |
| 4.1.3 Status Messages | Status announced without moving focus |
| 1.3.3 Sensory Characteristics | Instructions that don't rely on pitch or position alone |
| 2.5.7 Dragging Movements | A non-drag alternative to scrubbing |

The project should publish an accessibility conformance report each release. It should also audit visual mode against Chartability's 14 critical tests each release and its 50 heuristics at major versions ([Chartability](https://chartability.fizz.studio/)).

Conformance also needs people. Leading teams co-design over the long term with blind partners; Chart Reader used one-on-one "parallel co-design" with 10 partners. Compensation in recent studies ranged from $50 for 90 minutes ([Zong et al.](https://arxiv.org/pdf/2205.04917)) to $250 for three hours ([Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)). Samples of about 10–15 match the median of 13 in accessibility papers ([Mack et al. 2021](https://arxiv.org/pdf/2101.04271)), and access needs should be anticipated and adjusted for at every study stage ([Mack et al. 2022](https://par.nsf.gov/servlets/purl/10354105)).

Telemetry should follow Go's model: collected locally by default and uploaded only on opt-in ([Go telemetry](https://go.dev/doc/telemetry)). EU guidance extends consent requirements well beyond cookies, to device-side tracking generally ([EDPB 2/2023](https://www.edpb.europa.eu/documents/guideline/guidelines-22023-on-technical-scope-of-art-53-of-eprivacy-directive_en)).

## Publishable research sits in the field's admitted gaps

Evaluation is the field's weakest link, by its own account. Only **30 of 495 (6.1%)** published mapping occurrences carried any assessment, and only 7 of 60 projects assessed even one mapping ([Dubus & Bresin 2013](https://doi.org/10.1371/journal.pone.0082491)). Studies mostly use sighted students, so "performance ceilings for sonifications remain[] speculative" ([Walker & Nees 2011](https://sonification.de/handbook/chapters/chapter2/)). Efficacy testing in astronomy is "extremely limited" ([Zanella et al. 2022](https://doi.org/10.1038/s41550-022-01721-z)), research is "hindered by a lack of theory" ([Nees 2019](https://doi.org/10.21785/icad2019.048)), and software support is "not commensurate" with research on effective design ([Kim, Kim & Hullman 2024](https://doi.org/10.1145/3613904.3642442)).

This research also found no work on data rates above 8 points per second, empirical limits on concurrent streams, validated comfort thresholds, longitudinal fatigue, measured interference between screen-reader speech and sonification, polarity data for graph concepts, blind-user studies of network sonification, or peer-reviewed evaluation of LLM-suggested mappings.

Each gap is a paper that SONARITHM's infrastructure makes cheap to run.

| Paper | Contribution | Method | Best venue | Earliest window |
|---|---|---|---|---|
| System/toolkit | Native spec compiler, deterministic engine, accessible navigation | Ledo et al. strategies: replicate classic designs (Smith & Walker stimuli, Dombois audification, Erie gallery); latency and real-time-factor benchmarks; targeted usage study | ICAD 2027 (Athens); CHI 2028 | ~Mar 2027 [est.]; ~Sep 2027 [est.] |
| Software paper | Citable, DOI-backed software record | JOSS review | JOSS | ≥6 months after repo goes public |
| Benchmark harness | Open stimulus generators, standard tasks, logged metrics | Generators for reversals × density, SNR-controlled transits, correlation triplets | ICAD 2027; IEEE VIS 2027 short paper | ~Mar 2027 [est.] |
| Pacing, range and context study | Evidence beyond 8 points/s and on cue combinations | Factorial within-subjects study, blind and sighted participants, RMS error and trend accuracy | ICAD 2028; ACM TACCESS | Late 2027 |
| Graph-concept polarity | First magnitude-estimation data for degree, centrality, edge weight, path length | Walker-style modulus-free magnitude estimation; majority-polarity rule | ICAD; CHI 2028 | 2027 |
| LLM versus empirical mappings | Controlled test of LLM-suggested mappings against Walker priors | Generate mappings, then compare task performance and magnitude-estimation agreement | CHI 2028; IEEE VIS | 2027–28 |
| Accessible uncertainty | How blind users read extraction confidence in audio | Co-design, then a trust and error-detection task study | ASSETS 2027; TACCESS | ~Apr 2027 [est.] |
| Engine paper | Deterministic real-time/offline rendering, granular density, audification calculator | Technical benchmarks | DAFx27 (Cremona, 24–27 Aug 2027); JAES | Spring 2027 |
| Expressive use | AXONOTE as an instrument; MIDI and OSC bridges | Demo plus practitioner study | NIME 2027 (Paris, 22–25 Jun 2027) | CfP published Oct 2026 |

Venue sources: [icad.org](https://icad.org/), [JOSS submitting](https://joss.readthedocs.io/en/latest/submitting.html), [CHI 2027](https://chi2027.acm.org/authors/papers/), [ASSETS 2026 CfP](https://assets26.sigaccess.org/call_for_papers.html), [IEEE VIS 2026 CfP](https://aa.ieeevis.org/year/2026/info/call-participation/call-for-participation), [DAFx27](https://dafx27.deib.polimi.it/), [NIME 2027](https://www.nime2027.org/).

Timing is set by past cycles, since ICAD has not published 2027 dates:

| Venue | Last known deadline | Implication |
|---|---|---|
| ICAD | Full papers 20 March 2026 ([ICAD 2026](https://interactive-sonification.org/icad2026/)) | ICAD 2027 in Athens likely around March 2027 |
| CHI 2027 | 10 September 2026, now passed | CHI 2028 is the realistic target |
| IEEE VIS 2026 | Abstracts 21 March, full papers 31 March | Same window likely in 2027 |
| ASSETS 2026 | 22 April | Same window likely in 2027 |

JOSS requires "at least six months of public history" and evidence of research impact. Its papers run 750–1,750 words, with "State of the field" and "Research impact statement" sections and an AI-usage disclosure ([JOSS paper format](https://joss.readthedocs.io/en/latest/paper.html)).

The resulting sequence runs from a public repository with Zenodo DOIs in Q4 2026, to an ICAD 2027 debut, to JOSS in mid-2027, to ASSETS or VIS 2027 if studies finish in time, and finally to CHI 2028 with a mature toolkit.

Toolkit papers should follow Ledo et al., whose analysis of 68 toolkit papers argues for choosing evaluation strategies that match the paper's claims rather than defaulting to a usability study ([Ledo et al. 2018](https://discovery.ucl.ac.uk/id/eprint/10053385/)).

Study methods are well established:

| Element | Established options |
|---|---|
| Tasks | Point estimation scored by RMS error ([Smith & Walker](https://doi.org/10.1002/acp.1146)); trend identification ([Nees & Walker 2008](https://doi.org/10.1145/1402236.1402237)); signal counting across SNR levels ([Tucker Brown et al.](https://doi.org/10.1093/mnras/stac2590)); discrimination thresholds ([Elmquist et al.](https://doi.org/10.1007/s00779-024-01795-8)); dual-task monitoring ([Hildebrandt et al.](https://doi.org/10.1016/j.ijhcs.2016.06.002)); insight counts with blind users ([Siu et al.](https://doi.org/10.1145/3491102.3517678)) |
| Design-phase method | Magnitude estimation ([Walker 2002](https://doi.org/10.1037/1076-898X.8.4.211)) |
| Metrics | NASA-TLX, SUS, and pleasantness, clarity and confidence ratings ([Sharif et al. 2022](https://doi.org/10.1145/3517428.3550360)) |

A defensible default design is within-subjects with counterbalanced mapping order, a pre-test baseline as covariate and a short standardized training block. It should mix blind and sighted samples, since polarities differ between them ([Walker & Mauney 2010](https://doi.org/10.1145/1714458.1714459)), and participants should be free to bring their own data.

Gamified remote evaluation has proved "a reliable means for the remote evaluation of sonification", and interaction "tremendously improves the precision" ([Ziemer 2024](https://doi.org/10.21785/icad2024.007)).

Ethics approval must come before any study. CHI requires authors to meet their local ethics-review requirements ([CHI 2027](https://chi2027.acm.org/authors/papers/)). Consent materials must work with screen readers, and playback levels must be calibrated and stated. Starting ethics applications by December 2026 keeps spring 2027 deadlines realistic.

Reproducibility comes almost free if designed in. A `CITATION.cff` file plus the Zenodo–GitHub integration gives every release a DOI ([CFF](https://citation-file-format.github.io/)). Every render embeds the engine version and spec hash, and each paper ships an artifact package that can qualify for ACM's artifact badges.

Under ACM's now fully open-access model, CHI 2027 charges a $500 or $750 APC unless the author's institution participates in ACM Open ([CHI 2027](https://chi2027.acm.org/authors/papers/)).

## Conclusion

SONARITHM's opportunity is not "sonification software"; web libraries already serve that well. The vacancy is **trustworthy sonification infrastructure**: a reproducible, provenance-carrying, accessible specification that renders identically on a desktop, in a CLI, in Python and in a browser. The field's own diagnosis (adoption failed, evaluation is rare, tools get abandoned) makes this the higher-leverage bet. The most valuable thing SONARITHM can ship is not a ninth engine but evaluated defaults and a benchmark that makes everyone's claims testable, including its own. The nine-engine identity helps as a conceptual map over one compiler. It becomes a liability if it turns into nine codebases competing for a small maintainer pool, the pattern that has killed most tools in this field.

The tagline needs the same discipline. The evidence supports hearing trends, overviews and changes in monitored streams, and shows that audio added to visuals raises engagement. It also shows that subtle signals go undetected by ear without training, and that expert visual skill does not transfer to listening. "Discover what the graphs reveal" is therefore something SONARITHM has to earn through the studies above, not assert. If "graphs" includes networks, there is no polarity data and no blind-user evidence at all. Whoever produces that evidence first will set the defaults for a subfield that does not yet exist, and SONARITHM's spec, engine and harness make it unusually well placed to do so.
