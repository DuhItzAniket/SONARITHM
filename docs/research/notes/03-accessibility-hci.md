# Accessibility and HCI for Non-Visual Data Access: BLV User Needs, Interaction Techniques, Standards, and Platform Obligations for SONARITHM

Scope note: current as of 18 October 2026. Items that are superseded or at risk of going stale are marked **[OUTDATED]** or **[CHECK]**. DOIs are given only where they were confirmed during this research (from the paper PDF, the ACM/IEEE record, or a repository listing). Where a DOI could not be confirmed, a stable URL (arXiv, author PDF, repository) is given and the DOI is marked "not verified".

---

## 1. What do empirical studies say about BLV users' experiences and needs with data visualizations?

### Takeaway
Across a decade of ASSETS/CHI/VIS/EuroVis work, the evidence agrees on a few points. Screen-reader users are badly served by standard charts: in Sharif et al. 2021 they were 61% less accurate and took 211% longer, and a third of the charts tested could not be found at all. What these users want is an overview first, then drill-down on their own terms, with a data table and a factual summary always available. The most successful systems (Chart Reader, Umwelt, MAIDR, Olli, Data Navigator, VoxLens) combine a navigable hierarchical structure, speech or text descriptions, sonification and, increasingly, braille, all driven by one cursor. Users strongly want control over verbosity, modality and pace.

### Cited Findings

**Sharif, Chintalapati, Wobbrock & Reinecke, "Understanding Screen-Reader Users' Experiences with Online Data Visualizations," ASSETS 2021. DOI 10.1145/3441852.3471202**
- Method: Study 1 was contextual interviews with 9 screen-reader users. Study 2 was a controlled experiment with 36 screen-reader users and 36 non-screen-reader users (72 total) answering multiple-choice questions about visualizations built with ChartJS, D3 and Google Charts. — [Sharif et al. 2021 (PDF)](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf); [ACM record via Unpaywall](https://unpaywall.org/10.1145%2F3441852.3471202)
- Screen-reader users extracted information 61.48% less accurately (34% vs 87% accurate answers) and spent 210.96% more time interacting with online visualizations than non-screen-reader users. — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
- 33% of the 27 sampled visualizations could not be discovered by screen readers at all. Those that could be discovered were often announced meaninglessly, as "blank", "frame", "graphic" or "object". — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
- Library mattered: Google Charts, which appends a hidden data table, gave screen-reader users the best accuracy (73%), versus D3 (17%) and ChartJS (11%). The trade-off is that sequential table navigation is slow and gives no holistic overview. — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
- Interviews found a two-stage strategy. Users first get a holistic "flyover" of trends and extrema, then drill down to compare individual points. Participants preferred tabular representations, textual representations (alt text) and braille printouts. — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
- Design recommendations:
  - R1: make visualizations discoverable and comprehensible.
  - R2: libraries should offer both a holistic view and drilled-down exploration.
  - R3: generate alternative text automatically and dynamically, personalized to the user.
  - R4: offer different modes for exploring data.
  — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)

**Sharif, Wang, Muongchan, Reinecke & Wobbrock, "VoxLens: Making Online Data Visualizations Accessible with an Interactive JavaScript Plug-In," CHI 2022. DOI 10.1145/3491102.3517431**
- System: a JavaScript plug-in (D3, chart.js, Google Charts) with three modes:
  - Question-and-Answer, using voice commands for extrema, axis labels and ranges, mean, median, mode, variance, standard deviation, sum, and the value at a given x;
  - Summary;
  - Sonification, mapping data to a musical scale.
  Keyboard modifiers are Ctrl+Shift on Windows and Option on macOS. — [VoxLens paper (PDF)](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf)
- Participants and results: 21 screen-reader users in task-based experiments. Accuracy of information extraction improved by 122% and interaction time fell by 36% compared with not using VoxLens. Follow-up interviews were held with 6 participants. — [VoxLens paper](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf). Discrepancies: the UW press release says 22 participants, and press coverage notes the comparison was against participants from a previous study rather than a concurrent control group. — [UW News](https://washington.edu/news/2022/06/01/voxlens-adding-one-line-of-code-can-make-some-interactive-visualizations-accessible-to-screen-reader-users)
- Qualitative finding: first-time sonification users were initially unsure what pitch height meant but quickly learned to "visualize" the trend. Sonification gave holistic trend information. — [VoxLens paper](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf)

**Lundgard & Satyanarayan, "Accessible Visualization via Natural Language Descriptions: A Four-Level Model of Semantic Content," IEEE TVCG 28(1), VIS 2021. DOI 10.1109/TVCG.2021.3114770**
- Method: grounded-theory analysis of 2,147 sentences describing charts, followed by a mixed-methods evaluation with 30 blind and 90 sighted readers, who produced 3,600 ranked descriptions. — [Lundgard & Satyanarayan (arXiv 2110.04406)](https://arxiv.org/pdf/2110.04406)
- The four levels:
  - L1, elemental and encoded properties: chart type, title, axes, encodings.
  - L2, statistical concepts and relations: descriptive statistics, extrema, outliers, correlations, point-wise comparisons.
  - L3, perceptual and cognitive phenomena: complex trends, patterns, exceptions.
  - L4, contextual and domain-specific insights.
  The corpus distribution was L1 9.1%, L2 35.1%, L3 42.9%, L4 12.9%. — [Lundgard & Satyanarayan](https://arxiv.org/pdf/2110.04406)
- Findings: both groups preferred mid-level content (L2 and L3), but they diverged sharply at the extremes. Blind readers ranked L4 (author interpretation, context, editorializing) among the least useful. They wanted plain facts and room to interpret the numbers themselves. The authors conclude that descriptions for blind readers should convey trends and statistics rather than only low-level design elements or high-level insights. — [Lundgard & Satyanarayan](https://arxiv.org/pdf/2110.04406)

**Zong, Lee, Lundgard, Jang, Hajas & Satyanarayan, "Rich Screen Reader Experiences for Accessible Data Visualization," Computer Graphics Forum 41(3), EuroVis 2022.** [arXiv 2205.04917](https://arxiv.org/pdf/2205.04917); DOI not verified.
- Method: 6-month co-design with a blind co-author (Hajas), producing 15 prototypes. Evaluation was 90-minute Zoom sessions with 13 BLV participants (7 totally blind, 4 with light perception, 2 low vision; 7 Windows/Chrome, 6 Mac/Safari) using Cooperative Usability Testing. Participants were paid $50. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
- Three design dimensions:
  - **Structure**: the form (list, table, tree) and the entities, organized by information granularity.
  - **Navigation**:
    - *structural*: local moves up and down levels and between siblings, plus lateral moves between equivalent nodes in adjacent facets;
    - *spatial*: following screen x/y directions;
    - *targeted*: jumping to a chosen location without traversing a path.
  - **Description**: content, composition (ordering of utterances) and verbosity.
  — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
- Findings: the designs helped users conceptualize data spatially, attend selectively at different granularities, and feel control and agency. Redundant, near-identical utterances while traversing a tree slowed users down. The blind co-author described Apple's Data Comprehension readout as all-or-nothing ("play-or-stop") when the points of interest sit in the middle of the data. Affording a sense of the structure's boundaries reduces cognitive load. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
- JAWS exposes high, medium and low verbosity levels, which the authors cite as a model for user-controlled verbosity. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)

**Olli (Blanco, Zong & Satyanarayan), IEEE VIS 2022 poster**
- An open-source library that converts charts into a keyboard-navigable ARIA tree view, which screen readers narrate as users move with the arrow keys. It instantiates the structure, navigation and description dimensions above. A toolkit-agnostic adapter pattern supports Vega and Vega-Lite among others. npm packages are `olli` and `olli-adapters`. — [Olli project page](https://vis.mit.edu/pubs/olli); [olli-adapters README](https://cdn.jsdelivr.net/npm/olli-adapters@2.0.2/README.md)

**Siu, Kim, O'Modhrain & Follmer, "Supporting Accessible Data Visualization Through Audio Data Narratives," CHI 2022.** DOI not verified.
- Method: two co-design workshops with screen-reader users produced design principles for the structure, content and duration of an audio data narrative. A dynamic-programming approach then generates narratives automatically from time-series data, combining textual description with sonification. The evaluation involved 16 screen-reader users. — [Stanford HCI listing](https://hci.stanford.edu/publications/paper.php?id=473); [NSF PAR record](https://par.nsf.gov/biblio/10379155)
- Findings: with audio narratives, users gained significantly more insights, and reported that the narratives helped them extract and understand information from both the sonification and the description. — [Stanford HCI listing](https://hci.stanford.edu/publications/paper.php?id=473)

**Holloway, Goncu, Ilsar, Butler & Marriott, "Infosonics: Accessible Infographics for People who are Blind using Sonification and Voice," CHI 2022, Article 480. DOI 10.1145/3491102.3517465**
- An interactive approach that combines spoken introduction and annotation with non-speech sonification. The study found that it supported understanding and the formation of a mental image of the data. — [Enge et al. STAR summary (arXiv 2402.16558)](https://arxiv.org/pdf/2402.16558); [datouch.uma.es reference listing](https://datouch.uma.es/referencias-2/)

**Wang, Jung & Kim, "Seeing Through Sounds: Mapping Auditory Dimensions to Data and Charts for People with Visual Impairments," Computer Graphics Forum 41(3):71–83, EuroVis 2022. DOI 10.1111/cgf.14523** (derived from Eurographics handle 10.1111/cgf14523)
- Method: an exploratory experiment in which BLV participants rated how intuitive and how accurate auditory channels (for example pitch and volume) were across data types and chart types.
- Findings:
  - Pitch was rated the most intuitive channel.
  - Number of taps and sound length gave the most accurate decoding of values.
  - Perception of individual data points did not carry over directly to whole charts, because participants drew on visual concepts of charts.
  — [Eurographics Digital Library](https://diglib.eg.org/handle/10.1111/cgf14523)

**Thompson, Martinez, Sarikaya, Cutrell & Lee, "Chart Reader: Accessible Visualization Experiences Designed with Screen Reader Users," CHI 2023. DOI 10.1145/3544548.3581186**
- Method: "parallel co-design", meaning one-on-one sessions with each of 10 BLV screen-reader users, three one-hour sessions each over five months, with prototype iterations v1 to v3 between sessions. — [Chart Reader paper (Microsoft Research PDF)](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- Five design dimensions: Zong et al.'s structure, navigation and description, plus non-speech audio (sonification and earcons) and focus (filtering, highlighting, selection). — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- Structure: a high-level summary that also tells the user how to start, then five branches: Data Insights (grouped by insight type), X-Axis (bins), Y-Axis (bins), Data Points, and Filters (per series). Users can jump from an insight directly to the associated data. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- Interaction features:
  - Shift+Enter plays a sonification of the series, with pitch encoding value.
  - Shift plus a navigation key moves while sonifying.
  - Data-value announcements are interwoven with the sonification, and the last value is announced on pause.
  - Spatial (stereo) sonification of series.
  - A "bonk" boundary earcon at the edge of the structure and a "drip" sound for an invalid sonify action.
  - Semantically meaningful jump keys (X for X-Axis, F for Filters) as an escape hatch for users who are lost.
  - Document mode and series filtering.
  — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- Findings:
  - Reducing verbosity got a mixed reception: it made navigation faster, but users sometimes still wanted full data and still did not know where they could go. The boundary tone was added to address the second problem.
  - Comparing across series needed explicit support beyond adjacent navigation.
  - Autonomy was the overarching value.
  - The design goal is an engine that works without a specific screen reader, plug-in or extra hardware.
  — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- Methods recommendation: partners engaged less with data they did not care about, so future studies should let participants bring their own data or topics. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)

**Seo, Xia, Lee, McCurry & Yam, "MAIDR: Making Statistical Visualizations Accessible with Multimodal Data Representation," CHI 2024. DOI 10.1145/3613904.3642730**
- System: covers bar plots, heat maps, box plots and scatter plots with a smooth-line layer. It has four modalities that share one cursor and are all off by default:
  - Braille (B): shape or trend on a single-line refreshable display using 8-dot cells.
  - Text (T): speech through the screen reader, with Verbose and Terse modes.
  - Sonification (S): x mapped to stereo pan, y mapped to pitch.
  - Review (R): a persistent literary-braille overlay of the focused value.
  Controls: Tab focuses the plot, B/T/S/R toggle modalities, arrow keys move, PageUp/PageDown switch layers, and H opens help and settings (braille cell count, volume, and so on). — [MAIDR (arXiv 2403.00717)](https://arxiv.org/html/2403.00717v1)
- Braille encodings: bar heights in four levels (dot chords 78, 36, 25, 14). Box plots use outlier dot 2, whiskers 25, IQR 123456, and median 456/123. Heat-map rows are separated by dots 1256. — [MAIDR](https://arxiv.org/html/2403.00717v1)
- Participants: 11 totally blind people, recruited via NFB and Program-L from 176 respondents. Mean age 35.7 (range 18–57). 7 used JAWS, 3 NVDA, 1 Orca. Sessions were 2.5–3 hours over Zoom. — [MAIDR](https://arxiv.org/html/2403.00717v1)
- Findings:
  - SUS scores: bar 81.4, heat map 75.5, box 74, scatter 70.3.
  - Most participants worked whole-to-part: sonification or braille for an overview, then text for detail. One worked part-to-whole.
  - Preferences tracked ability. A participant with high-frequency hearing loss relied on braille and text, while one with strong relative pitch favored sonification.
  - Braille was best for box plots, helpful for heat maps, and least useful for bar and scatter plots.
  - Persistent review helped users read at their own pace.
  - Users needed training in both the tactile encoding and the statistical concept.
  — [MAIDR](https://arxiv.org/html/2403.00717v1)
- Follow-ups:
  - "MAIDR Meets AI" (ASSETS 2024) added an LLM chat interface and was studied with 8 BLV participants.
  - The `maidr` JavaScript library is maintained (legacy archived), and py-maidr brings the approach to matplotlib and seaborn.
  — [MAIDR search summary / Microsoft Research page](https://www.microsoft.com/en-us/research/publication/maidr-making-statistical-visualizations-accessible-with-multimodal-data-representation/); [maidr README](https://cdn.jsdelivr.net/npm/maidr@4.6.0/README.md)

**Zong, Pedraza Pineros, Chen, Hajas & Satyanarayan, "Umwelt: Accessible Structured Editing of Multi-Modal Data Representations," CHI 2024. DOI 10.1145/3613904.3641996**
- System: an authoring environment in which visualization, sonification and textual description are coequal, all derived from one shared abstract data model with no modality privileged. Heuristics generate default multimodal representations from the dataset's functional relationships. A shared query predicate links the modalities, so navigating the text highlights the visualization and filters the sonification. Sonification can be played sequentially with a play button or selected interactively with a dropdown and slider. — [Umwelt (PDF)](https://vis.mit.edu/pubs/umwelt.pdf); [arXiv 2403.00106](https://arxiv.org/abs/2403.00106v1)
- Method: about a year of co-design led by Zong and Hajas (a blind researcher), followed by remote evaluation with 5 expert BLV participants, paid $250 for 3 hours. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
- Findings: the modalities had complementary affordances: more options, modality-specific strengths, and easy toggling between overview and detail. Sonification settings were easy to customize, but participants found it harder to predict how the viewer would change after editor edits. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)

**Chart2Music (Julianna Langston, Sean Mealin and contributors)**
- An MIT-licensed library that turns charts into sound and keyboard-explorable experiences, independent of rendering library (Chart.js, D3, images). The maintainers state that blind people are involved in design, development and testing. ChartIQ has integrated it into its product, and S&P Global described it in a 2025 blog post. — [chart2music on npm](https://www.npmjs.com/package/chart2music); [Chart2Music README](https://cdn.jsdelivr.net/npm/chart2music@1.20.0/README.md); [ChartIQ API docs](https://documentation.chartiq.com/WebComponents.Chart2Music.html); [S&P Global blog, 17 Apr 2025](https://www.spglobal.com/market-intelligence/en/news-insights/research/hearing-the-market-chart2music-makes-charts-accessible-for-the-visually-impaired)
- No peer-reviewed evaluation paper was found. — [search result summary of the above sources](https://www.npmjs.com/package/chart2music)

**Elavsky, Bennett & Moritz, "How Accessible is my Visualization? Evaluating Visualization Accessibility with Chartability," Computer Graphics Forum 41(3), EuroVis 2022. DOI 10.1111/cgf.14522** (derived from Eurographics handle 10.1111/cgf14522)
- Heuristics organized under 7 principles. Four are POUR (Perceivable, Operable, Understandable, Robust) and three extend Robust: Compromising, Assistive, Flexible. There are 50 heuristics in total and a quick shortlist of 14 critical tests. Coverage spans visual, motor, vestibular, neurological and cognitive accessibility. Novice auditors became more confident and found auditing easier with it. — [Chartability site](https://chartability.fizz.studio/); [Eurographics DL](https://diglib.eg.org/handle/10.1111/cgf14522); [EU Data Visualisation Guide](https://data.europa.eu/apps/data-visualisation-guide/intro-to-chartability)

**Elavsky, Nadolskis & Moritz, "Data Navigator: An Accessibility-Centered Data Navigation Toolkit," IEEE VIS 2023 / TVCG.** [arXiv 2308.08475](https://arxiv.org/pdf/2308.08475); DOI not verified.
- A dynamic node-edge graph structure that lets developers build navigable lists, trees, graphs, flows and spatial structures, loosely coupled from the visual encoding. It supports screen reader, keyboard, speech, gesture and fabricated assistive inputs. Building on a discrete, sequential, keyboard-like base gives switch, sip-and-puff, on-screen keyboard and many braille-display users access by default. The paper presents 3 case examples, including navigation over raster images. — [Data Navigator](https://arxiv.org/pdf/2308.08475)

**Newer work, 2023–2026**
- **Skeleton** (Elavsky, Nnadozie, Nadolskis, Carrington, Moritz), accepted at IEEE VIS 2026 (to appear November 2026). It is a direct-manipulation authoring tool for non-visual navigation graphs: an Inspector that draws navigation graphs as node-link diagrams, a Dimensions API, a scaffolding engine, label templates with screen-reader preview, and a traversal testing mode. In an in-situ study, 8 practitioners came to treat accessibility as a design problem rather than a compliance task. — [arXiv 2607.14579](https://arxiv.org/abs/2607.14579)
- **SpeechTone** (Zhao, Lu, Nacenta), ASSETS 2024. A speech-based mark that extends the Erie sonification grammar by mapping data to voice pitch, speed, voice and speech content. It is demonstrated through 3 examples, with no user study. — [arXiv 2408.06942](https://arxiv.org/abs/2408.06942)
- **DIXTRAL** (Kamath, Zeidieh, Seo), 2025. Browser-based non-visual access to 3D surface and point data, co-designed with 2 blind researchers and 1 sighted researcher. Synchronized auditory, visual and text feedback improved structure discovery and orientation, supported by keyboard and gamepad navigation. These are preliminary findings from a very small sample. — [arXiv 2508.08554](https://arxiv.org/abs/2508.08554)
- **"They Aren't Built For Me"** (Khalaila, Harrison, Kim, Cashman), 2024 preprint. A Cleveland and McGill replication with swell-form tactile charts and 11 visually impaired participants. They read faster than in earlier replications, with similar or better accuracy, but participants want encodings designed specifically for them. — [arXiv 2410.08438](https://arxiv.org/abs/2410.08438)
- **Reinders, Zaib, Butler, Lee, Zukerman, Qu, Marriott**, VIS 2024 Accessible Data Visualization workshop poster. Ongoing co-design of a system that combines refreshable tactile displays (RTDs) with a conversational agent, using tactile graphics plus speech to proactively help BLV users with data analysis. No results yet. — [arXiv 2506.23443](https://arxiv.org/abs/2506.23443)
- **Hoque, Ehtesham-Ul-Haque, Elmqvist & Billah, "Accessible Data Representation with Natural Sound,"** CHI 2023. Uses natural sounds rather than synthetic tones. — [Penn State Pure record](https://pure.psu.edu/en/publications/accessible-data-representation-with-natural-sound/)
- **Enge et al., "Open Your Ears and Take a Look: A State-of-the-Art Report on the Integration of Sonification and Visualization,"** 2024. A survey useful for the sighted-analyst side of SONARITHM. — [arXiv 2402.16558](https://arxiv.org/pdf/2402.16558)
- Additional leads found but not reviewed in depth:
  - ChartA11y, accessible touch charts with blind smartphone users. — [arXiv 2410.20545](https://arxiv.org/html/2410.20545v1)
  - "Customization is Key: Reconfigurable Content Tokens for Accessible Data Visualizations". — [arXiv 2307.08773](https://arxiv.org/pdf/2307.08773)
  - Low-vision on-cursor visual context. — [arXiv 2603.02498](https://arxiv.org/pdf/2603.02498)
  - Experience-based co-design of a multimodal 3D visualization tool. — [arXiv 2604.09426](https://arxiv.org/pdf/2604.09426)
  - QUARTZ. — [arXiv 2608.11364](https://arxiv.org/pdf/2608.11364)

### Inferences
- The consistent pattern is **overview → navigate → detail on demand, under user control**. Sonification is valued mainly for fast holistic trend perception (VoxLens, Chart Reader, MAIDR). Speech, text, tables and braille serve exact values and comparisons.
- Individual differences are large, by hearing profile, braille literacy, statistical background, and screen reader in use (MAIDR, Chart Reader). SONARITHM therefore cannot ship one "best" mapping. Configurability and per-user profiles are core features, not extras.
- Descriptions SONARITHM generates should default to L2 and L3 content (statistics, extrema, trends), keep L4 interpretation separate and optional, and never replace access to the data itself (Lundgard; Sharif).
- Most evidence comes from web contexts (ARIA trees in browsers). The interaction concepts transfer to a native desktop app, but the plumbing differs (see Section 4).

### Gaps
- Infosonics: participant count and quantitative outcomes were not retrieved (the full text was not accessed).
- Wang et al. 2022: the sample size and exact channel ranking table were not retrieved (Eurographics DL returned 403).
- Siu et al. 2022 and Zong et al. 2022: DOIs could not be confirmed from primary pages during this session.
- Chart2Music: no peer-reviewed evaluation was located. Its exact key map was not verified.
- Kim et al. (2021 design-space survey; 2023 chart question answering for BLV users) and Jung et al. (2021 alt text): known to exist from prior knowledge but not verified in this session, so they are omitted from the findings.

---

## 2. Navigation and interaction techniques: what works, for which tasks?

### Takeaway
The strongest evidence supports several techniques together:
- a **hierarchical, keyboard-navigable structure**, with structural, lateral, spatial and targeted moves, landmarks and boundary cues;
- **sonification for overview** (pitch for y, time for x), with play, pause and scrub, paired with **speech for exact values** and **data tables for lookup**;
- **braille for persistent, self-paced reading** of shape and values.

Earcons for boundaries and landmarks are well liked. Haptics and tactile graphics are promising, but desktop evidence is thin. Task fit matters: trends and shape suit sonification and braille, exact values and comparisons suit speech and tables, and specific lookups suit targeted navigation or Q&A.

### Cited Findings
- **Hierarchical/tree navigation.**
  - Tree structures support granularity from overview to datum. Local, lateral, spatial and targeted navigation reduce the mental-map burden, and conveying boundaries reduces cognitive load. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
  - Chart Reader's five-branch hierarchy (Insights, X, Y, Data Points, Filters), topped by a summary that explains how to begin, was the converged design after three co-design rounds. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
  - Olli renders an ARIA tree navigated with the arrow keys. — [Olli](https://vis.mit.edu/pubs/olli)
- **Keyboard models.**
  - MAIDR: arrow keys for x and y, single-letter modality toggles (B/T/S/R), PageUp/PageDown for layers, H for help. — [MAIDR](https://arxiv.org/html/2403.00717v1)
  - Chart Reader: Shift+Enter plays, Shift+arrow sonifies while moving, and semantic letter keys jump to landmarks (X, F). — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
  - VoxLens: modifier chords (Ctrl+Shift on Windows, Option on macOS), presumably chosen to avoid screen-reader conflicts. — [VoxLens](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf)
  - A keyboard-like, discrete sequential base automatically extends access to switches and braille-display input. — [Data Navigator](https://arxiv.org/pdf/2308.08475)
- **Scrubbing and continuous control.**
  - Umwelt's audio units support sequential playback (a play button) and interactive selection with a dropdown and slider. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
  - Chart Reader announces the last value when playback pauses and lets users move while sonifying. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
  - DIXTRAL used keyboard and gamepad to navigate continuous 3D fields. — [arXiv 2508.08554](https://arxiv.org/abs/2508.08554)
- **Overview-then-detail.**
  - Screen-reader users first seek a holistic flyover, then drill down. — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
  - Most MAIDR participants used sonification or braille for the overview and text for details. — [MAIDR](https://arxiv.org/html/2403.00717v1)
  - Umwelt participants valued toggling between overview and detail across modalities. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
- **Summary + sonification + speech combinations.**
  - Audio data narratives, which interleave description and sonification with a designed structure and duration, significantly increased insights (16 screen-reader users). — [Siu et al. 2022](https://hci.stanford.edu/publications/paper.php?id=473)
  - Infosonics: a spoken introduction and annotations plus sonification supported building a mental image. — [Enge et al. STAR](https://arxiv.org/pdf/2402.16558)
  - VoxLens's Summary, Q&A and Sonification modes produced +122% accuracy (21 users). — [VoxLens](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf)
  - Chart Reader interweaves value announcements with tones. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- **Earcons for landmarks and boundaries.**
  - Chart Reader added a distinct noise earcon at boundaries ("bonk") after partners struggled to know where they could go. A "drip" signals invalid actions. Earcons also mark points of interest such as line intersections. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
  - Apple Audio Graphs: gridlines are reportedly rendered as haptic feedback during playback and interaction (per the WWDC21 session as quoted on the Apple forums; one developer could not observe an effect). — [Apple Developer Forums thread](https://developer.apple.com/forums/thread/713568)
- **Auditory mapping choices.**
  - Pitch is the most intuitive channel, while taps and duration decode values most accurately. — [Wang et al. 2022](https://diglib.eg.org/handle/10.1111/cgf14523)
  - Blind and visually impaired listeners broadly agreed with sighted listeners on polarity, with notable exceptions and some differences in scaling slopes, so designers must consider their intended users' vision status. — [Walker & Mauney 2010, ACM TACCESS 2(3), DOI 10.1145/1714458.1714459](https://wirelessrerc.gatech.edu/node/341.html)
  - A secondary summary reports 45 BLV listeners vs 435 sighted, with polarity r = 0.818; this was not verified against the primary text. — [a11y-paradise summary](https://a11y-paradise.onrender.com/reviews/69cdc9269c2cf36d2aa44e9b)
  - Apple Audio Graphs maps x to time, y to pitch, an additional numeric axis to tone length, and a categorical axis to timbre. — [Apple: Representing chart data as an audio graph](https://developer.apple.com/documentation/accessibility/representing-chart-data-as-an-audio-graph.md)
  - MAIDR maps x to stereo pan and y to pitch. — [MAIDR](https://arxiv.org/html/2403.00717v1)
- **Braille display output.**
  - MAIDR's single-line 8-dot encodings worked best for box plots and heat maps and least well for bars and scatter. Persistent "review" braille was valued over transient screen-reader flash messages. The authors suggest exploring multi-line displays. — [MAIDR](https://arxiv.org/html/2403.00717v1)
- **Tactile graphics and refreshable tactile displays.**
  - Swell-form tactile charts gave fast, accurate reading in a graphical-perception replication with 11 participants, but users want purpose-built encodings. — [Khalaila et al. 2024](https://arxiv.org/abs/2410.08438)
  - RTDs combined with a conversational agent are under co-design. — [Reinders et al.](https://arxiv.org/abs/2506.23443)
  - Braille printouts remain a preferred method for some screen-reader users. — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
- **Natural-language Q&A and LLMs.**
  - VoxLens's voice Q&A covered extrema, statistics and point lookup. — [VoxLens](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf)
  - MAIDR Meets AI (ASSETS 2024) added LLM chat, studied with 8 BLV users. — [MAIDR Microsoft Research page](https://www.microsoft.com/en-us/research/publication/maidr-making-statistical-visualizations-accessible-with-multimodal-data-representation/)
- **Linked multimodal views.**
  - In Umwelt, a shared selection across text, sonification and visualization supports both independent BLV analysis and mixed-ability work. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)

### Inferences
- Map techniques to tasks:
  - trend, shape and anomaly detection → sonification and braille sparkline;
  - exact value, extremum and statistic → speech or Q&A;
  - lookup and comparison of many values → data table/grid;
  - orientation → summary, landmarks, earcons and "where am I".
  These mappings are drawn across Sharif 2021, MAIDR, Chart Reader and Wang 2022.
- For a graph/network sonifier, Data Navigator's arbitrary node-edge navigation and Zong's lateral/targeted moves are the closest validated analogues (for example "follow edge", "jump to node by name"). No BLV study of network sonification was found in this research.
- Stereo panning (MAIDR, Chart Reader) is fragile in practice. NVDA's sound split can assign apps to a single channel (Section 4), and hearing asymmetry is common, so panning should be redundant, never the sole carrier of information.

### Gaps
- No controlled head-to-head study was found comparing hierarchical navigation, scrubbing and Q&A on the same tasks.
- Evidence on vibrotactile and haptic output for desktop data (beyond Apple's gridline haptics on mobile) is thin. No rigorous desktop studies were found in this session.
- No BLV evaluation of node-link/network sonification was located.

---

## 3. Which standards and laws apply to a desktop data tool?

### Takeaway
For a cross-platform desktop app, the practical baseline as of October 2026 has five parts:
1. **WCAG 2.2 Level AA applied to non-web software via W3C's WCAG2ICT Note.** WCAG 2.2 has been ISO/IEC 40500:2025 since October 2025.
2. **EN 301 549 clause 11 (software) plus functional performance statements.** v3.2.1 is the legal reference today. v4.1.1, aligned to WCAG 2.2, was published on 2 September 2026 and is awaiting citation in the Official Journal.
3. **Section 508 Chapter 5** for US federal procurement. **[OUTDATED]** It still incorporates WCAG 2.0 AA.
4. **The European Accessibility Act.** It applies from 28 June 2025, but a free, standalone data-sonification app is likely outside its enumerated product and service categories unless it is part of an in-scope service.
5. **WCAG 3.0.** Still a Working Draft (10 September 2026) and not citable as a requirement.

WAI-ARIA Graphics roles and the APG patterns are web technologies, but they provide the semantic vocabulary to mirror in native accessibility trees.

### Cited Findings
- **WCAG 2.2 and ISO.** WCAG 2.2 was approved as ISO/IEC 40500:2025, announced by W3C on 21 October 2025. A standards-store listing gives an approval date of 24 September 2025. — [W3C WAI news](https://www.w3.org/WAI/news/2025-10-21/wcag22-iso); [ISO listing (Bahrain store)](https://bsmd.moic.gov.bh/store/standards/iso:pub:std:IS:91029/ISO-IEC%2040500:2025)
- **WCAG 2.2 success criteria most relevant to SONARITHM** (normative text in the [WCAG 2.2 Recommendation](https://www.w3.org/TR/WCAG22/)):
  - 1.1.1 Non-text Content (A): text alternatives for the graph, and for sonified output where it conveys information.
  - 1.3.1 Info and Relationships (A): data-table and axis relationships must be programmatic.
  - 1.3.3 Sensory Characteristics (A): instructions must not rely solely on sound, pitch or position.
  - 1.4.1 Use of Color (A).
  - 1.4.2 Audio Control (A): audio that plays automatically for more than 3 seconds needs pause/stop or a volume control independent of system volume.
  - 1.4.3 / 1.4.11 Contrast (AA).
  - 1.4.7 Low or No Background Audio (AAA): background sound at least 20 dB below foreground speech. This is useful guidance when sonification overlaps speech.
  - 2.1.1 Keyboard (A), 2.1.2 No Keyboard Trap (A).
  - 2.1.4 Character Key Shortcuts (A): single-key shortcuts must be able to be turned off or remapped, or be active only on focus. This directly affects MAIDR- and Chart Reader-style letter keys.
  - 2.2.1 Timing Adjustable (A).
  - 2.2.2 Pause, Stop, Hide (A): for auto-updating or streaming data displays.
  - 2.3.1 Three Flashes (A).
  - 2.4.3 Focus Order (A), 2.4.7 Focus Visible (AA), 2.4.11 Focus Not Obscured (Minimum) (AA, new in 2.2).
  - 2.5.7 Dragging Movements (AA, new in 2.2): drag-to-scrub needs a single-pointer alternative.
  - 2.5.8 Target Size (Minimum) (AA, new in 2.2).
  - 3.2.6 Consistent Help (A, new in 2.2).
  - 3.3.8 Accessible Authentication (AA).
  - 4.1.2 Name, Role, Value (A).
  - 4.1.3 Status Messages (AA).
  - 4.1.1 Parsing is obsolete and removed in 2.2.
  — [W3C WCAG 2.2](https://www.w3.org/TR/WCAG22/)
- **WCAG2ICT.** Published as a W3C Group Note on 8 October 2024 ("Guidance on Applying WCAG 2 to Non-Web Information and Communications Technologies"). It covers WCAG 2.0, 2.1 and 2.2 success criteria for non-web documents and software. It is informative, not normative. The task force has moved on to which criteria should apply to particular technologies. — [W3C news](https://www.w3.org/WAI/news/2024-10-08/wcag2ict); [US Access Board](https://www.access-board.gov/news/2024/10/08/wcag2ict-published-as-w3c-group-note/); [WCAG2ICT Note](https://www.w3.org/TR/wcag2ict-22/)
- **WAI-ARIA Graphics Module 1.0**, W3C Recommendation, 2 October 2018.
  - `graphics-document`: the root of a structured graphic whose visual layout conveys meaning; charts, maps and diagrams are the examples.
  - `graphics-object`: a semantically meaningful sub-component that can nest and does not create a new navigation context.
  - `graphics-symbol`: a simple meaningful glyph, possibly part of a chart, whose children are presentational.
  The spec says future work would add richer annotation for charts and maps. **[OUTDATED/CHECK]** No chart-specific roles have emerged as far as found. — [W3C Graphics-ARIA 1.0](https://www.w3.org/TR/graphics-aria-1.0/)
- **ARIA Authoring Practices.** Useful patterns to mirror natively:
  - Tree View, for hierarchical chart navigation as in Olli. — [APG Tree View](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/)
  - Grid, for data tables. — [APG Grid](https://www.w3.org/WAI/ARIA/apg/patterns/grid/)
  - Slider, for scrub and playhead. — [APG Slider](https://www.w3.org/WAI/ARIA/apg/patterns/slider/)
  - Toolbar, for modality toggles. — [APG Toolbar](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/)
  Zong et al. note that alt text, data tables and the current ARIA spec cannot by themselves deliver rich chart experiences. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
- **Section 508 (Revised Standards, 36 CFR 1194).**
  - E207.2 requires software UI and content to conform to WCAG 2.0 Level A and AA. Non-web software is exempt from SC 2.4.1, 2.4.5, 3.2.3 and 3.2.4 and from Conformance Requirement 3.
  - Chapter 5:
    - 502.1: software shall interoperate with AT.
    - 502.2.2: software shall not disrupt platform accessibility features.
    - 502.3.1: object role, state, properties, boundary, name and description must be programmatically determinable.
    - 502.3.14: event notification.
    - 503.2: applications must honour platform user preferences (color, contrast, font, focus cursor).
    - 503.3: an alternative UI that functions as AT must use platform accessibility services.
  — [US Access Board ICT Standards](https://www.access-board.gov/ict/)
  - **[OUTDATED]** The Section 508 baseline is still WCAG 2.0. — [Access Board](https://www.access-board.gov/ict/)
- **US ADA Title II.** The DOJ's 2024 rule requires WCAG 2.1 AA for state and local government web content and mobile apps. An interim final rule of 20 April 2026 extended compliance to 26 April 2027 for entities serving 50,000 people or more, and to 26 April 2028 for smaller entities. The technical standard is unchanged. HHS issued a parallel Section 504 interim final rule on 7 May 2026. — [Duane Morris alert](https://www.duanemorris.com/alerts/doj_extends_ada_title_ii_digital_accessibility_deadlines_one_year_0426.html); [Univ. of Arizona](https://accessibility.arizona.edu/news/doj-issues-interim-final-rule-title-ii-digital-accessibility-compliance-dates); [Level Access](https://www.levelaccess.com/blog/the-ada-title-ii-extension-what-it-doesnt-change/)
- **EN 301 549.**
  - v4.1.1 was formally adopted on 24 August 2026 and published by ETSI/CEN/CENELEC on 2 September 2026. It moves from WCAG 2.1 to WCAG 2.2 A/AA and adds annexes mapping to the EAA.
  - Its main legal effect (presumption of conformity under the EAA) begins only once it is cited in the Official Journal.
  - As of mid-October 2026, sources say it is not yet cited, so v3.2.1 remains the reference. Expected citation is late 2026 (Level Access says November 2026); this is unconfirmed.
  — [Davis Wright Tremaine, Sept 2026](https://www.dwt.com/insights/2026/09/european-accessibility-act-ict-standards-update); [Pivotal Accessibility](https://www.pivotalaccessibility.com/2026/09/en-301-549-v4-1-1-is-published-what-it-means-for-digital-accessibility/); [Level Access](https://www.levelaccess.com/compliance-overview/european-accessibility-act-eaa/)
  - The current v3.2.1 text is at [ETSI EN 301 549 V3.2.1 (PDF)](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf). The relevant clauses for SONARITHM:
    - clause 4 functional performance statements, including use without vision, with limited vision, without hearing and with limited hearing;
    - clause 5 generic requirements;
    - clause 11 non-web software: WCAG-derived 11.1–11.4, 11.5 interoperability with AT, 11.6 documented accessibility usage, 11.7 user preferences, 11.8 authoring tools;
    - clause 12 documentation and support.
    These clause numbers are from prior knowledge of v3.2.1 and were not re-verified in this session. **[CHECK]**
- **European Accessibility Act (Directive (EU) 2019/882).**
  - Products in scope (Art. 2(1)(a)) include consumer general-purpose computer hardware systems *and their operating systems*. Recital 25 lists PCs, including desktops, notebooks, smartphones and tablets. — [legislation.gov.uk copy of Directive 2019/882, Chapter I](https://www.legislation.gov.uk/eudr/2019/882/chapter/I)
  - Services in scope (Art. 2(2)) include electronic communications, access to audiovisual media, passenger transport elements, consumer banking, e-books *and dedicated software*, and e-commerce. — [accessible.org scope summary](https://accessible.org/eaa-scope-products-services/); [lawplayer Art. 2](https://lawplayer.com/eu/article/32019L0882/art_2)
  - Obligations apply from 28 June 2025. — [Karl Groves on EAA and B2B](https://karlgroves.com/the-european-accessibility-act-and-b2b-software-what-internal-platforms-must-comply-with/)
  - Microenterprises (fewer than 10 employees and under €2M turnover) providing services are exempt. — [karlgroves.com](https://karlgroves.com/?p=3390)
  - Recital 11 excludes products and services provided by public-sector bodies only to people acting in a professional capacity. — [karlgroves.com](https://karlgroves.com/the-european-accessibility-act-and-b2b-software-what-internal-platforms-must-comply-with/)
  - Note: EUR-Lex itself blocked automated retrieval, so the Article 2(2) wording is from secondary sources. **[CHECK]**
- **WCAG 3.0.**
  - The latest W3C Working Draft is dated 10 September 2026. It contains only requirements at "Developing" status and must not be cited other than as a work in progress.
  - Its structure: guidelines are outcome statements; requirements are labelled core or supplemental; assertions are statements about an organization's practices; methods are technology-specific. Sections carry Placeholder, Exploratory, Developing, Refining or Mature status.
  — [W3C WCAG 3.0 WD](https://www.w3.org/TR/wcag-3.0/)
  - The schedule targets a Candidate Recommendation around Q4 2027, with a final Recommendation not expected before 2028. These figures come from secondary sources, so they need checking. — [AbilityNet](https://abilitynet.org.uk/resources/digital-accessibility/what-expect-wcag-30-web-content-accessibility-guidelines); [ADA Compliance Pros](https://www.adacompliancepros.com/blog/wcag-3-draft-status-procurement)

### Inferences
- **EAA applicability to SONARITHM:**
  - A free, open-source, standalone desktop analysis and sonification tool does not fit the enumerated product categories, which cover hardware and OSs, not applications. Nor does it fit the services list (it is not e-commerce, banking, e-books, communications, AV access or transport).
  - If it is distributed non-commercially, it is probably not an economic operator's "product or service placed on the market" at all.
  - It **would** be implicated if it were embedded in an in-scope service, for example an e-book reading app for math textbooks, a banking dashboard, or a consumer e-commerce analytics feature.
  - Public-sector procurement in the EU separately references EN 301 549, so universities and agencies may still require conformance claims. Note: the public-procurement linkage is from prior knowledge and was not verified this session.
  - This is legal analysis, not legal advice. The project should get advice if it is commercialized.
- The robust strategy is to target **EN 301 549 v4.1.1 / WCAG 2.2 AA (via WCAG2ICT)**. That satisfies Section 508 (WCAG 2.0 AA is a subset in practice), EN 301 549 v3.2.1 (WCAG 2.1), and likely EAA procurement expectations. The project should publish an accessibility conformance report.
- Because SONARITHM is audio-first, the "usage without hearing" functional performance statements (EN 301 549 clause 4; Section 508 302.4/302.5) matter as much as the "without vision" ones. Every sonified insight needs a visual, text or braille equivalent for Deaf, hard-of-hearing and DeafBlind users.

### Gaps
- The exact EAA Article 2(2) and Article 4(5) text could not be retrieved from EUR-Lex (blocked). The wording above relies on secondary sources.
- No primary confirmation was found of the date EN 301 549 v4.1.1 will be cited in the Official Journal, or of any changes to clause 11 in v4.1.1.
- Whether the Access Board plans to update Section 508 to WCAG 2.1 or 2.2 was not found.
- WCAG2ICT's specific interpretation of new 2.2 criteria (for example 3.2.6 Consistent Help and 2.4.11) for desktop software was not reviewed in detail.

---

## 4. Platform accessibility: exposing a Rust desktop UI, screen-reader behavior, and audio coexistence

### Takeaway
Each OS has its own accessibility API: UI Automation on Windows, NSAccessibility on macOS, AT-SPI2 over D-Bus on Linux (with a Wayland-native successor, Newton, in development). **AccessKit** is the de facto Rust abstraction over all three, plus Android and iOS. SONARITHM must expose a real accessibility tree (roles, names, values, states, events) and announce dynamic changes through notification APIs:
- UIA Notification events on Windows 10 1709 and later;
- NSAccessibility `announcementRequested` with a priority on macOS;
- live regions via AccessKit.

The hardest problem is audio coexistence. Screen readers can duck other apps' audio, and NVDA can split itself and other apps into separate stereo channels. Sonification design must tolerate both. Apple's Audio Graphs (AXChart) is the best reference design for a standardized chart descriptor.

### Cited Findings
- **AccessKit** (Rust):
  - Architecture: a core crate defines the accessibility tree schema and the action-handling trait, and platform adapters expose it.
  - Adapters: Windows (UI Automation), macOS (NSAccessibility in AppKit), Linux/Unix (AT-SPI D-Bus via the pure-Rust zbus), Android, and iOS (UIAccessibility). A web/canvas adapter is only planned.
  - Status: adapters are at rough feature parity but do not yet support all UI element types or schema properties, nor rich text or hypertext. The winit adapter is recommended for winit apps. The design suits immediate-mode toolkits if they provide stable IDs per element.
  — [AccessKit GitHub](https://github.com/AccessKit/accesskit)
  - AccessKit's schema includes a `Live` property, used by downstream crates for live-region politeness. — [oxiui-accessibility README](https://docs.rs/crate/oxiui-accessibility/0.2.0/source/README.md)
- **Windows UIA notifications.** `UiaRaiseNotificationEvent(provider, NotificationKind, NotificationProcessing, displayString, activityId)` lets a provider ask AT to announce a string. The minimum is Windows 10 version 1709. — [Microsoft Learn: UiaRaiseNotificationEvent](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationcoreapi/nf-uiautomationcoreapi-uiaraisenotificationevent)
- **macOS announcements.**
  - `NSAccessibility.Notification.announcementRequested` is presented via speech and/or braille when VoiceOver is on and otherwise does nothing. It requires a localized `announcement` string in `userInfo` plus an optional `priority`. — [Apple docs](https://developer.apple.com/documentation/appkit/nsaccessibility-swift.struct/notification/announcementrequested.md)
  - Developers report VoiceOver sometimes skipping queued announcements, and macOS has no completion callback (unlike iOS). — [Apple Developer Forums](https://developer.apple.com/forums/thread/709501)
  - Chromium source notes that announcements from inactive windows were not spoken on Mojave. — [Chromium ax_platform_node_mac.mm](https://chromium.googlesource.com/chromium/src/+/913eaefa55beab765aa1d99da170db474d8c63ca/ui/accessibility/platform/ax_platform_node_mac.mm)
- **Linux.**
  - Newton is an experimental, Wayland-native replacement for AT-SPI in which apps push accessibility information. It is more efficient and compatible with Wayland and Flatpak. Apps use a new Wayland protocol to Mutter, and Orca talks to Mutter over D-Bus.
  - Orca was modernized to use AT-SPI directly via GObject Introspection, without pyatspi.
  - Orca is reported basically usable on Wayland with some GTK 4 apps, but GNOME Shell still uses AT-SPI. **[CHECK]** The date of the Newton status is unclear.
  — [GNOME a11y blog](https://blogs.gnome.org/a11y/); [GNOME STF 2024 report](https://blogs.gnome.org/tbernard/?p=10486); [OSnews](https://www.osnews.com/story/139996/update-on-newton-the-wayland-native-accessibility-project/)
- **Screen-reader market.** WebAIM Survey #10 (December 2023 to January 2024, 1,539 responses) found primary desktop screen readers to be JAWS 40.5%, NVDA 37.7% and VoiceOver 9.7%. NVDA overtook JAWS in overall usage. The sample was uncontrolled. **[CHECK]** A newer survey may exist. — [WebAIM Survey #10](https://webaim.org/projects/screenreadersurvey10/); [WebAIM blog](https://webaim.org/blog/screen-reader-user-survey-10-results/)
- Study samples mirror this mix: MAIDR had 7 JAWS, 3 NVDA and 1 Orca users. — [MAIDR](https://arxiv.org/html/2403.00717v1)
- **Audio ducking.** NVDA offers three modes: no ducking, duck when outputting speech and sounds, and always duck. Ducking requires Windows 8 or later and an installed (not portable) NVDA. — [NVDA audioDucking source](https://softwarepublico.gov.br/gitlab/vlibras/nvda/blob/master/source/audioDucking.py); [NVDA issue #16052](https://github.com/nvaccess/nvda/issues/16052)
- **NVDA sound split.** NVDA can route its own speech to one stereo channel and other applications' audio to the other. It is configured in Audio settings (NVDA+Ctrl+U) and cycled with NVDA+Alt+S. Sound split and ducking interact. — [NV Access In-Process, 31 May 2024](https://www.nvaccess.org/?p=30731); [NVDA issue #16052](https://github.com/nvaccess/nvda/issues/16052)
- **Screen-reader modes.** In Chart Reader's web scenario, the screen reader switches to application mode when entering the chart, and the chart prompts the user to press Enter to start. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- Chart Reader's goal was an experience that does not require a specific screen reader, extension or extra hardware. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
- **Apple Audio Graphs as a reference design.**
  - Developers adopt `AXChart` and set `accessibilityChartDescriptor` to an `AXChartDescriptor`. It contains:
    - title and summary;
    - x and y axes plus `additionalAxes`, typed as `AXNumericDataAxisDescriptor` (range, `gridlinePositions`, `valueDescriptionProvider` for unit-formatted strings) or `AXCategoricalDataAxisDescriptor` (`categoryOrder`);
    - `AXDataSeriesDescriptor`, with name, `isContinuous` and `AXDataPoint` entries (x, y, additional values, label).
  - VoiceOver renders it with x as time, y as pitch, an extra numeric axis as tone length and a categorical axis as timbre.
  — [Apple: Representing chart data as an audio graph](https://developer.apple.com/documentation/accessibility/representing-chart-data-as-an-audio-graph.md); [WWDC21 session 10122](https://developer.apple.com/la/videos/play/wwdc2021/10122/)
  - The `AXChart` protocol arrived with iOS 15. — [react-native-ax-chart README](https://cdn.jsdelivr.net/npm/react-native-ax-chart@0.3.0/README.md)
  - Apple's earlier Data Comprehension readout was criticized as all-or-nothing by a blind researcher. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)

### Inferences
- **Architecture.** SONARITHM's UI layer should build on a toolkit that integrates AccessKit (or use AccessKit directly with winit). Every custom-drawn chart element that is navigable must be a node with a stable ID, role, name, value and state:
  - tree/treeitem for the hierarchy;
  - grid/row/cell for tables;
  - slider for the playhead;
  - toggle buttons for modalities;
  - graphics-document, graphics-object and graphics-symbol equivalents where AccessKit offers them. **[CHECK]** Their availability in the current AccessKit Role enum was not verified.
- **Speech strategy.** Default to the user's screen reader for speech and braille, through focus changes, value-change events and notifications. This preserves the user's voice, rate, verbosity and braille-display routing. MAIDR's text mode works this way, and it is the approach implied by Section 508 502.3 and EN 301 549 11.5. Offer optional self-voicing (for example via OS TTS) for users without a screen reader, such as sighted STEM learners or kiosks.
- **Timing.** Avoid speaking and sonifying at the same instant. Sequence them as Chart Reader does: announce the value on pause and interleave. Where overlap is unavoidable, keep sonification well below speech level; WCAG 1.4.7's 20 dB guideline is a reasonable target. Document NVDA's ducking and sound-split settings, and test with each.
- **Rate limiting.** Use the "most recent" style of notification processing for rapid navigation, so a backlog of announcements does not queue behind the cursor. Throttle live-data announcements to meet WCAG 2.2.2 and 4.1.3.
- **Shortcuts.** Keyboard shortcuts must avoid screen-reader modifier combinations (NVDA key, JAWS key, VoiceOver's Ctrl+Option, Orca key). Single-letter commands should be active only when the chart has focus, or be remappable (WCAG 2.1.4). VoxLens used modifier chords for this reason.

### Gaps
- The `NotificationProcessing` enum semantics and actual JAWS/NVDA/Narrator handling of UIA notifications were not verified in this session.
- No primary documentation was found for JAWS or Narrator audio-ducking behavior (Narrator's "lower the volume of other apps" setting is from prior knowledge and unverified).
- AccessKit's per-platform live-region behavior and graphics roles were not verified.
- No study was found that quantifies how screen-reader speech and concurrent sonification interfere with comprehension.

---

## 5. STEM education and other contexts: math sonification, astronomy outreach, eyes-busy monitoring

### Takeaway
Math function sonification for blind students has a long but small-sample evidence base:
- SonicFunction had 14 participants;
- AudioFunctions had 7 participants and outperformed tactile paper;
- newer 2024–2026 projects such as SONAIRGRAPH and AudioFunctions 2.0 add cues for undefined and off-screen values.

Astronomy has embraced sonification for public engagement at scale: about 100 projects, and a NASA study of 3,184 participants reporting self-reported learning gains. Evidence for sighted, eyes-busy monitoring (continuous soundscapes improving peripheral monitoring without hurting the primary task) is promising but rests on few studies.

### Cited Findings
- **SonicFunction** (Grond, Drossard, Hermann, ICAD 2010). It combines discrete and continuous sonification of function values, conveys the derivative, and signals the sign of f(x) via timbre. It was evaluated with 14 partially sighted and blind students. — [Bielefeld record](https://pub.uni-bielefeld.de/record/2017190); [Georgia Tech repository](https://repository.gatech.edu/entities/publication/a856465a-6dc9-4d61-a369-26ffa775ff48)
- **AudioFunctions** (Taibbi et al., 2014). An iPad app with three sonification techniques. In an evaluation with 7 blind participants, they understood graphs better than with tactile paper or existing software. — [Springer chapter listing](https://www.springerprofessional.de/audiofunctions-eyes-free-exploration-of-mathematical-functions-o/2191074)
- **AudioFunctions 2.0** (Spagnuolo et al., 2026 poster, ICTMT17). Adds auditory cues for undefined and off-screen function values, so multiple features can be perceived simultaneously without conflict. — [poster PDF](https://univda.iris.cineca.it/retrieve/59b6347a-cb21-4176-8235-e9e2fb51c5ef/Spagnuolo%20et%20al.%20%282026%29_poster_ICTMT17.pdf)
- **SONAIRGRAPH** (Erasmus+, from September 2024). Sonification for accessible and inclusive representation of graphs in education, to be tested with higher-education students across countries. — [Univ. Valle d'Aosta project page](https://www.univda.it/progetti-di-ricerca/erasmus-sonairgraph-sonification-for-accessible-and-inclusive-representation-of-graphs-in-education/); [INDIRE 2024 poster](https://www.erasmusplus.it/wp-content/uploads/2024/11/SONAIRGRAPH_Poster-evento-INDIRE-2024_UniTo.pdf)
- **Astronomy.**
  - Zanella et al., Nature Astronomy 6:1241–1248 (2022), DOI 10.1038/s41550-022-01721-z, reviewed nearly 100 sound-based astronomy projects. — [Deusto repository record](https://portalinvestigacion.deusto.es/documentos/672fc6cd6d4b2a057b4bf81f)
  - Primary objectives of those projects: public engagement 36%, research 26%, art 17%, accessibility 13%, education 8%. — [Sky & Telescope summary](https://skyandtelescope.org/astronomy-news/how-astronomers-are-rendering-the-universe-in-sound/)
  - Arcand et al., "A Universe of Sound," Frontiers in Communication 9 (2024), DOI 10.3389/fcomm.2024.1288896, sonified NASA (Chandra) data for three objects and surveyed 3,184 sighted and BLV participants. They found significant self-reported learning gains and positive experience, and argue that multisensory data can build trust and access. — [arXiv 2403.18082](https://arxiv.org/pdf/2403.18082); [Frontiers](https://www.frontiersin.org/journals/communication/articles/10.3389/fcomm.2024.1288896/pdf)
- **Eyes-busy monitoring (sighted users).**
  - Hildebrandt, Hermann & Rinderle-Ma (IJHCS 2016) ran a within-subjects study with n=18 comparing visual only, visual plus auditory alerts, and visual plus a continuous forest soundscape. Monitoring performance was highest with continuous sonification, and the primary arithmetic task was not significantly impaired. — [Univie eprint](https://eprints.cs.univie.ac.at/4755)
  - A follow-up with business-process data found that visualization plus alerts plus continuous sonification beat visualization alone. — [Univie eprint 5345](https://eprints-dev5.cs.univie.ac.at/5345)
  - A 3D-printing dual-task study found minor attentional pull. — [HAL](https://hal.archives-ouvertes.fr/hal-03618071)
  - A 2023 study found less ecologically coherent soundscapes were detected less accurately. — [arXiv 2302.03957 (ar5iv)](https://ar5iv.org/html/2302.03957)

### Inferences
- For STEM learners, SONARITHM should support:
  - function plots with derivative, sign and zero-crossing cues (SonicFunction);
  - explicit cues for undefined, asymptote and off-screen regions (AudioFunctions 2.0);
  - axis and gridline ticks as earcons (Apple, Chart Reader);
  - exportable audio for lessons.
- Astronomy outreach shows that sighted audiences also value sonification. A shared, linked multimodal view (Umwelt) supports mixed-ability classrooms.
- For sighted analysts in monitoring mode, continuous ambient or ecological soundscapes with event earcons are better supported than alerts alone, but this rests on n=18 and needs SONARITHM-specific validation.

### Gaps
- No controlled classroom studies with school-age blind students using function sonification were found.
- Desmos, SAS Graphics Accelerator, Highcharts Sonification Studio and MathTrax were not reviewed in this session.
- No independent replication of Hildebrandt et al. 2016 was found.

---

## 6. Research methods with BLV participants: co-design, recruitment, remote studies, ethics, compensation

### Takeaway
Leading work uses **long-term co-design with blind co-authors or design partners**: Zong et al. 2022 (6 months, 15 prototypes), Umwelt (about a year), and Chart Reader (10 partners over 5 months). Evaluations are then run remotely over Zoom with participants' own assistive technology. Samples are typically around 10–16, matching the field median of 13. Recruitment runs through blind organizations (NFB, Program-L) and community networks. Reported compensation ranges from about $15/hour (2021) to about $83/hour (Umwelt 2024). Access must be planned in advance and adjusted as needed, across communication, materials, space and time.

### Cited Findings
- **Field norms.** Mack et al. (CHI 2021) surveyed 836 CHI and ASSETS accessibility papers (1994–2019) and coded 506 from 2010–2019. Over 43% of papers in the past decade focused on BLV people, and the median sample size for disabled or older participant groups was 13. — [Mack et al. 2021 (arXiv 2101.04271)](https://arxiv.org/pdf/2101.04271)
- **Access in methods.** Mack et al., "Anticipate and Adjust: Cultivating Access in Human-Centered Methods," CHI 2022 (Best Paper Honorable Mention), interviewed 17 accessibility experts (13 researchers and 4 community organizers). They recommend anticipating access needs while adjusting flexibly, across communication, materials, space and time at every study stage. For example, a blind researcher ran diary studies via text and voice memos. — [NSF PAR full text](https://par.nsf.gov/servlets/purl/10354105); [UW HCDE news](https://hcde.washington.edu/index.php/news/article/2022-05-06/hcde-chi-2022)
- **Co-design models.**
  - Zong et al. 2022: a blind co-author, weekly hour-long video meetings over 6 months, 15 prototypes, and memos by the blind co-designer. Additional participants were recruited for evaluation to broaden perspectives. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
  - Chart Reader: one-on-one "parallel co-design" with 10 partners, chosen over group sessions. Implementing features between sessions was a time constraint. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
  - Umwelt: co-design led by a blind researcher over about a year. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
  - Siu et al.: two co-design workshops. — [Siu et al.](https://hci.stanford.edu/publications/paper.php?id=473)
- **Recruitment.**
  - MAIDR recruited via NFB and Program-L, with 176 respondents yielding 11 participants. — [MAIDR](https://arxiv.org/html/2403.00717v1)
  - Zong et al. recruited through blind-community collaborators and a public call on Twitter. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
- **Remote study practice.**
  - Zoom sessions were used by Zong et al. (90 min), MAIDR (2.5–3 h) and Umwelt (3 h). — [Zong](https://arxiv.org/pdf/2205.04917); [MAIDR](https://arxiv.org/html/2403.00717v1); [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
  - In remote in-home studies with 12 blind participants, a first-person smart-glasses view fully captured 58.7% of interactions vs 3.7% for a third-person laptop camera. Problems included smart glasses lacking screen-reader support, battery drain and connectivity. — [Lee et al., W4A 2022 (arXiv 2203.04282)](https://arxiv.org/pdf/2203.04282)
  - Remote studies let participants use AT already configured to their preferences. — [Bigham et al., WebinSitu (ASSETS 2007)](https://faculty.washington.edu/wobbrock/pubs/assets-07.04.pdf)
- **Compensation (reported).**
  - Sharif et al. 2021: $15 Amazon gift card for 45–60 min of interview. In the experiment, screen-reader users received $15 for 30–45 min and non-screen-reader users $10 for 10–20 min, scaled by average completion time. — [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
  - Zong et al. 2022: $50 for 90 min. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
  - Umwelt 2024: $250 for 3 h. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
- **Ethics and reporting.** Zong et al. report participant demographics only in aggregate, following ethnographic practice, to protect privacy and avoid reducing participants to their demographics. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917)
- **Study design pitfalls.**
  - Participants engaged less with uninteresting data, so let them bring their own. — [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
  - Participants needed training in tactile encodings and statistical concepts, and statistical background affected learning speed. — [MAIDR](https://arxiv.org/html/2403.00717v1)
  - Practitioner guidance for remote sessions: pre-test materials for screen-reader compatibility and zoom to 200%. — [User Interviews blog](https://www.userinterviews.com/blog/remote-interviews-with-participants-with-visual-disabilities)

### Inferences
- SONARITHM should recruit at least one blind or low-vision core contributor or paid design partner from the outset, and run parallel co-design (one-on-one) iterations on each major interaction feature.
- Evaluation samples of about 10–15 BLV participants match field norms. Include a mix of JAWS, NVDA, VoiceOver and Orca users, braille readers, people with hearing differences, and congenitally vs adventitiously blind people.
- Budget compensation at roughly $50–85/hour, in line with recent MIT studies, and pay non-screen-reader comparison participants fairly but separately.
- Consent forms, surveys and scheduling tools must themselves be screen-reader accessible. Sessions should run on the participant's own configured AT.

### Gaps
- No field-wide published norm or guideline on compensation rates for BLV participants was found; the figures are per-study.
- No IRB-specific guidance unique to BLV participants was found in this session.

---

## 7. Synthesis: Accessibility requirements for SONARITHM

### Takeaway
The requirements below turn the findings above into testable obligations. Each one is traced to an empirical study or a standards clause. "MUST" items are grounded in normative standards or strong, convergent evidence. "SHOULD" items rest on study evidence.

### Accessibility requirements for SONARITHM

1. **MUST expose every chart/graph view as a programmatically identifiable, named object with a concise auto-generated summary** (title, chart type, axes and ranges, series count, plus key statistics and trends). It must never be announced as an unlabeled "graphic". — WCAG 2.2 SC 1.1.1, 4.1.2; Section 508 502.3.1; [Sharif et al. 2021, R1/R3](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf)
2. **MUST make summaries default to statistical and trend content (Lundgard L2/L3).** Domain interpretation (L4) must be optional and clearly separated, so users can draw their own conclusions. — [Lundgard & Satyanarayan 2021](https://arxiv.org/pdf/2110.04406)
3. **MUST provide an always-available, accessible data-table view** of the underlying data with row and column headers exposed through the platform table/grid APIs. — WCAG 1.3.1; Section 508 502.3.3; [Sharif et al. 2021](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf); [APG Grid](https://www.w3.org/WAI/ARIA/apg/patterns/grid/)
4. **MUST provide a hierarchical, keyboard-navigable structure** (dataset → series/axes/insights/filters → bins/segments → data points; for networks, node → neighbors/edges). It must support local, lateral (across series or facets), spatial (by x/y) and targeted (jump-to/search) navigation. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917); [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf); [Data Navigator](https://arxiv.org/pdf/2308.08475); [APG Tree View](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/)
5. **MUST make all functionality keyboard-operable with no keyboard traps, a logical focus order, and visible, unobscured focus.** — WCAG 2.1.1, 2.1.2, 2.4.3, 2.4.7, 2.4.11
6. **MUST allow single-character shortcuts** (for example landmark jumps such as X or Y, or modality toggles such as B/T/S) **to be turned off or remapped, or restrict them to when the chart has focus.** Default bindings must avoid screen-reader modifier combinations. — WCAG 2.1.4; [MAIDR key model](https://arxiv.org/html/2403.00717v1); [VoxLens modifiers](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf)
7. **SHOULD provide landmark jump keys and a "where am I" command** that reports position, level and neighbors. — [Chart Reader semantic shortcuts](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf); [Zong et al. 2022 targeted navigation](https://arxiv.org/pdf/2205.04917)
8. **SHOULD use distinct earcons for boundaries, invalid actions, landmarks and gridline or threshold crossings.** They must be clearly different from data tones, and each must have a text or visual equivalent. — [Chart Reader "bonk"/"drip"](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf); [Apple Audio Graphs gridlines](https://developer.apple.com/forums/thread/713568); WCAG 1.1.1
9. **MUST support overview-then-detail.** One command plays the whole series or selection, users can pause or scrub and navigate while sonifying, and the current value is announced on pause. — [Sharif et al. 2021 "flyover"](https://wildlab.cs.washington.edu/Publications_files/Sharif_ASSETS2021.pdf); [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf); [MAIDR whole-to-part](https://arxiv.org/html/2403.00717v1)
10. **MUST offer independently toggleable output modalities sharing one cursor**: speech/text, sonification, braille, visual highlight, and optional haptics. Settings must persist per user. — [MAIDR BTS+R](https://arxiv.org/html/2403.00717v1); [Umwelt coequal modalities](https://vis.mit.edu/pubs/umwelt.pdf)
11. **SHOULD link selection across modalities.** Navigating the text or structure highlights the visual and filters the sonification, and vice versa, to support mixed-ability collaboration. — [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
12. **MUST provide verbosity control** (at least terse and verbose) **and user-configurable announcement composition and order**, removing redundant context while moving within a series. — [Zong et al. 2022](https://arxiv.org/pdf/2205.04917); [MAIDR Verbose/Terse](https://arxiv.org/html/2403.00717v1); [Chart Reader verbosity findings](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
13. **SHOULD default to time for x and pitch for y, with redundant duration or tap-count encodings available for precise value judgment.** Polarity and scaling should be configurable per data dimension, and defaults should be validated with BLV listeners. — [Wang et al. 2022](https://diglib.eg.org/handle/10.1111/cgf14523); [Walker & Mauney 2010](https://wirelessrerc.gatech.edu/node/341.html); [Apple Audio Graphs mapping](https://developer.apple.com/documentation/accessibility/representing-chart-data-as-an-audio-graph.md)
14. **MUST NOT carry essential information only in stereo panning, pitch extremes or timbre.** It must survive mono output, NVDA sound split, and high-frequency hearing loss, with a configurable pitch range. — WCAG 1.3.3; [NVDA sound split](https://www.nvaccess.org/?p=30731); [MAIDR participant with hearing loss](https://arxiv.org/html/2403.00717v1)
15. **MUST provide user controls for sonification volume independent of system volume, plus tempo/duration, pitch range and instrument.** There must be no autoplay longer than 3 seconds without a stop mechanism, and a global "stop all audio" command. — WCAG 1.4.2; [Umwelt sonification settings](https://vis.mit.edu/pubs/umwelt.pdf); [MAIDR settings](https://arxiv.org/html/2403.00717v1)
16. **MUST let users pause, stop or hide streaming or auto-updating data views and their announcements, and rate-limit live announcements.** — WCAG 2.2.2, 4.1.3
17. **MUST announce dynamic status through platform mechanisms**, not focus-stealing: UIA Notification events on Windows (10 1709 and later), NSAccessibility `announcementRequested` with a priority on macOS, and AccessKit live regions on Linux. Rapid navigation should supersede stale queued announcements. — WCAG 4.1.3; Section 508 502.3.14; [UiaRaiseNotificationEvent](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationcoreapi/nf-uiautomationcoreapi-uiaraisenotificationevent); [Apple announcementRequested](https://developer.apple.com/documentation/appkit/nsaccessibility-swift.struct/notification/announcementrequested.md); [AccessKit](https://github.com/AccessKit/accesskit)
18. **MUST implement the accessibility tree via the platform APIs**, using AccessKit's UIA, NSAccessibility and AT-SPI adapters, with correct role, name, value, state and boundary for every custom-drawn control and chart node, and must not disrupt platform accessibility features. — WCAG 4.1.2; Section 508 502.1, 502.2.2, 502.3.1; EN 301 549 clause 11.5 [CHECK]; [AccessKit](https://github.com/AccessKit/accesskit)
19. **SHOULD route speech and braille through the user's screen reader by default**, preserving their voice, rate and braille routing, **and offer optional self-voicing for users without a screen reader.** — Section 508 503.3; [MAIDR text via screen reader](https://arxiv.org/html/2403.00717v1); [Chart Reader screen-reader-agnostic goal](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf)
20. **SHOULD sequence speech and sonification to avoid overlap** (interleave; speak on pause). Where overlap occurs, keep sonification about 20 dB below speech, and document recommended NVDA ducking and sound-split settings. — WCAG 1.4.7 (AAA, as guidance); [Chart Reader interleaving](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf); [NVDA audio ducking](https://softwarepublico.gov.br/gitlab/vlibras/nvda/blob/master/source/audioDucking.py)
21. **SHOULD provide braille-oriented output**: a compact braille "sparkline" encoding of series shape and box plots, and a persistent review line showing the focused value, sized to the user's braille cell count. — [MAIDR braille and Review modes](https://arxiv.org/html/2403.00717v1)
22. **SHOULD provide query commands for extrema, mean, median, mode, standard deviation, sum, axis ranges and value-at-x**, with optional natural-language Q&A. — [VoxLens Q&A](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf); [MAIDR Meets AI](https://www.microsoft.com/en-us/research/publication/maidr-making-statistical-visualizations-accessible-with-multimodal-data-representation/)
23. **SHOULD support navigable insights and annotations, plus audio data narratives** (description interleaved with sonification), each linked to its data range. — [Chart Reader insights](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf); [Siu et al. 2022](https://hci.stanford.edu/publications/paper.php?id=473); [Infosonics via Enge et al.](https://arxiv.org/pdf/2402.16558)
24. **MUST provide non-audio equivalents for all sonified information** (text, visual or braille) so Deaf, hard-of-hearing and DeafBlind users are not excluded by the audio-first design. — EN 301 549 clause 4 FPS "usage without hearing" [CHECK]; Section 508 302.4/302.5; WCAG 1.1.1
25. **MUST respect platform user preferences and low-vision needs**: high-contrast and contrast themes, OS text scaling, reduced motion, sufficient contrast, non-color encodings, and magnifier-friendly focus tracking. — Section 508 503.2; WCAG 1.4.1, 1.4.3, 1.4.4, 1.4.11; [Chartability](https://chartability.fizz.studio/)
26. **MUST give drag-based scrubbing and pan/zoom a single-pointer and keyboard alternative**, and pointer targets of at least the WCAG minimum size. — WCAG 2.5.7, 2.5.8
27. **SHOULD support discrete sequential input beyond the keyboard** (switch access, braille-display routing keys, gamepad, speech commands) by building navigation on a discrete-step model. — [Data Navigator](https://arxiv.org/pdf/2308.08475); [DIXTRAL gamepad](https://arxiv.org/abs/2508.08554); [VoxLens voice](https://wildlab.cs.washington.edu/Publications_files/Sharif_CHI2022.pdf)
28. **SHOULD include STEM features for function plots**: derivative and sign cues, zero-crossing and asymptote markers, explicit cues for undefined and off-screen values, and tick earcons. — [SonicFunction](https://pub.uni-bielefeld.de/record/2017190); [AudioFunctions 2.0](https://univda.iris.cineca.it/retrieve/59b6347a-cb21-4176-8235-e9e2fb51c5ef/Spagnuolo%20et%20al.%20%282026%29_poster_ICTMT17.pdf)
29. **MUST make the authoring and configuration UI itself accessible**, so BLV users can create and edit mappings, descriptions and sonifications, not just consume them. — EN 301 549 clause 11.8 (authoring tools) [CHECK]; Section 508 504; [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)
30. **SHOULD ship a built-in, accessible tutorial and context help** (consistent location, H/F1) that teaches both the audio and braille encodings and the underlying statistical concepts. Help and documentation must be accessible. — [MAIDR training needs](https://arxiv.org/html/2403.00717v1); WCAG 3.2.6; Section 508 Chapter 6
31. **MUST target WCAG 2.2 AA conformance via WCAG2ICT and EN 301 549 v4.1.1**, while documenting conformance to v3.2.1 (WCAG 2.1) until v4.1.1 is cited, and to Section 508 (WCAG 2.0 AA). The project should publish an accessibility conformance report per release. WCAG 3.0 is tracked but not claimed. — [WCAG2ICT](https://www.w3.org/TR/wcag2ict-22/); [ISO/IEC 40500:2025](https://www.w3.org/WAI/news/2025-10-21/wcag22-iso); [DWT on EN 301 549 v4.1.1](https://www.dwt.com/insights/2026/09/european-accessibility-act-ict-standards-update); [Access Board ICT](https://www.access-board.gov/ict/); [WCAG 3.0 WD](https://www.w3.org/TR/wcag-3.0/)
32. **MUST test each release with NVDA, JAWS and Narrator on Windows, VoiceOver on macOS, and Orca on Linux (both X11/AT-SPI and Wayland)**, including NVDA ducking and sound-split configurations and at least one refreshable braille display. — [WebAIM Survey #10](https://webaim.org/projects/screenreadersurvey10/); [GNOME Newton/Orca status](https://blogs.gnome.org/a11y/); [MAIDR participant screen readers](https://arxiv.org/html/2403.00717v1)
33. **SHOULD audit the visual (sighted and low-vision) mode with Chartability's 14 critical tests each release**, and the full set of 50 heuristics at major versions. — [Chartability](https://chartability.fizz.studio/)
34. **MUST involve BLV people as paid co-designers and evaluators throughout**:
    - a parallel co-design cadence, with about 10–15 participants per evaluation round;
    - remote sessions on participants' own AT;
    - participants may bring their own data;
    - compensation of about $50–85/hour;
    - demographics reported in aggregate;
    - access planned in advance and adjusted as needed.
    — [Mack et al. 2022](https://par.nsf.gov/servlets/purl/10354105); [Mack et al. 2021](https://arxiv.org/pdf/2101.04271); [Chart Reader](https://microsoft.com/en-us/research/uploads/prod/2023/04/ChartReader-CHI2023.pdf); [Zong et al. 2022](https://arxiv.org/pdf/2205.04917); [Umwelt](https://vis.mit.edu/pubs/umwelt.pdf)

### Gaps
- Requirements 24 and 29 cite EN 301 549 clause numbers from prior knowledge of v3.2.1. Re-verify them against the v3.2.1 and v4.1.1 texts before quoting them in a conformance report.
- No empirical evidence specific to network or node-link sonification was found to ground the network-navigation parts of requirement 4. They are extrapolated from Data Navigator's graph model.
