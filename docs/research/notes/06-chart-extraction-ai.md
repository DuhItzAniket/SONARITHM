# Chart Data Extraction (Chart Digitization / Chart-to-Table) and AI/LLM Features for SONARITHM — State as of October 2026

Scope note: these notes cover methods for recovering numeric data from chart images and documents, and the AI features a sonification app could build on top. Many of the most recent numbers come from arXiv preprints posted in 2026; they are marked **[preprint]**. Leaderboard numbers from third-party aggregators are marked **[aggregator, unverified]**. Numbers I parsed from a PDF table whose layout was garbled are marked **[parsed from PDF table, verify]**.

---

## 1. Classical and semi-automatic digitizers (WebPlotDigitizer, Engauge, others): status, licensing, algorithms, accuracy

### Takeaway
WebPlotDigitizer (WPD) is still the reference semi-automatic tool, but its licensing changed. The open AGPL-3.0 code line (v4 and earlier) was paused in favour of a closed-source v5 at some point. The current upstream repository describes the "WPD frontend" as AGPL-3.0 and says the "AI Assist"/cloud parts are closed and owned by Automeris LLC. Treat WPD as a design reference, not as a dependency to vendor. Engauge Digitizer (GPL) looks dormant: its upstream GitHub URL now returns 404. Calibrated manual or semi-automatic digitizing is highly reliable in validation studies, but it is slow on complex charts.

### Cited Findings
**WebPlotDigitizer: license and status**
- The current upstream repo states: "WPD frontend is distributed under GNU AGPL v3 license (this repository)." It also says: "Automeris 'AI Assist' and other related cloud based systems are closed source and owned by Automeris LLC (owned by Ankit Rohatgi)." The repo shows about 3.2k stars and 1,089 commits, with an empty Releases section. — [automeris-io/WebPlotDigitizer](https://github.com/automeris-io/WebPlotDigitizer)
- A fork snapshot of the same repo (1,081 commits, i.e. slightly older than upstream) carries a README note that "work on this repository (v4 and earlier) has been paused in favor of v5 (closed source)". The note says issues can be filed but fixes will likely land only in v5, that PRs and other contributions are not accepted, and that details are at automeris.io/posts/. The license is still given as GNU AGPL v3. — [netopt/WebPlotDigitizer (fork)](https://github.com/netopt/WebPlotDigitizer)
- **Conflict / ambiguity:** the older README says v5 is "closed source". The newer upstream README (8 commits later) drops that note. It now says the frontend in the repo is AGPL v3 and only AI Assist/cloud systems are closed. I could not load the automeris.io announcement posts, which are JS-rendered, so I cannot confirm whether the v5 frontend was re-opened or whether the repo still holds only v4 code. — [upstream](https://github.com/automeris-io/WebPlotDigitizer); [fork](https://github.com/netopt/WebPlotDigitizer)
- The WPD docs give **version 5.2** as the citation version. They describe WPD as browser-based, combining computer vision with manual input, and accessed via automeris.io. The upstream README says to sign up on automeris.io to use it. — [WPD docs](https://automeris.io/docs/); [upstream repo](https://github.com/automeris-io/WebPlotDigitizer)
- Earlier license history (search snippet, not verified in full): WPD moved from GPL v3 to AGPL v3 at version 3.12. A 2024 blog post said ownership was moving to Automeris LLC. — search snippets referencing [Zenodo WPD record](https://zenodo.org/record/802310) and the [automeris.io blog](https://www.automeris.io/blog/index.html) (the blog was not retrievable: TLS host mismatch on www.)
- A third-party industrial blog claims WPD is "MIT Open Source". This contradicts the repository and should be disregarded. — [industrialmonitordirect.com](https://industrialmonitordirect.com/blogs/knowledgebase/webplotdigitizer-for-extracting-plot-data-from-images)

**WebPlotDigitizer: how it works (useful as a design template)**
- **Axis calibration types:**
  - 2D XY: click two points per axis, enter values, and choose linear, **log** or **date** scale.
  - Bar: two points on the value axis.
  - Polar: three points including the centre, with optional log radial scale.
  - Ternary: three corners.
  - Map/microscope: a scale bar.
  - Pixel-aligned image.
  - Circular chart recorder: five points.
  - Exponents are entered as `1.45e-10`. — [WPD docs: Digitize](https://automeris.io/docs/digitize/)
- **Masking:** box, pen and erase tools restrict the region the algorithm searches. — [WPD docs](https://automeris.io/docs/digitize/)
- **Colour selection:** pick a foreground colour (curve or markers), or use background mode, where everything except the background counts as data. A **colour-distance** threshold can be tested with a Filter button. — [WPD docs](https://automeris.io/docs/digitize/)
- **Automatic extraction algorithms:**
  - Averaging Window: the ΔX/ΔY window is matched to line thickness or marker size.
  - X Step with Interpolation: regular ΔX sampling with cubic splines and smoothing.
  - X Step.
  - Custom Independents: user-given x values.
  - Template Matching: for dense scatter plots with mixed markers.
  - Bar/histogram extraction: ΔX and ΔVal parameters.
  - Blob Detector: centroids, areas and moments, filtered by min/max diameter. — [WPD docs](https://automeris.io/docs/digitize/)
- **Manual correction:** add, adjust (arrow-key nudge) and delete points. View Data offers CSV export. — [WPD docs](https://automeris.io/docs/digitize/)

**Engauge Digitizer**
- `github.com/markummitchell/engauge-digitizer` returned **HTTP 404** when fetched in October 2026. — [fetch attempt](https://github.com/markummitchell/engauge-digitizer)
- The latest upstream release I could confirm is **12.1 (30 Nov 2019)**, via the Zenodo GitHub-release mirror. — [Zenodo 3558440](https://zenodo.org/records/3558440)
- Oracle EPEL 7 packages it as engauge-digitizer 12.1 under **GPLv2+**. — [Oracle EPEL](https://yum.oracle.com/repowatch/oraclelinux7.x86_64/ol7_developer_EPEL/4a9b4ce6d08f26df9ea77e7d8c7d4f3fb0f6e168)
- T2 Linux lists version 12.9.1 (GPL), sourced from a **fork** (github.com/akhuettel/engauge-digitizer) rather than upstream. — [T2 Linux](https://www.t2linux.com/packages/engauge-digitizer)

**Other semi-automatic tools / research systems**
- **ChartDetective (CHI 2023; Masson, Malacria, Vogel, Lank, Casiez):**
  - It works on the **vector** representation of charts in PDFs. Users select a vector chart and drag-and-drop the elements to extract, with filtering and preview.
  - Participants recovered data from CHI-paper charts in under 4 minutes, including partly or fully occluded elements.
  - Compared with raster approaches, it recovered all data, even hidden data, with **78% lower relative error**. — [HAL record](https://hal-lara.archives-ouvertes.fr/CRISTAL-LOKI/hal-04017638v1); [PDF](https://cs.uwaterloo.ca/~dmasson/pdfs/chartdetective.pdf)
- **ChartSense (Microsoft Research):** a semi-automatic tool that first classifies chart type with a deep-learning classifier and then applies per-type extraction algorithms with user interaction. — [Microsoft Research](https://www.microsoft.com/en-us/research/?p=365924)

**Accuracy of semi-automatic digitizing**
- **Drevon et al. (2017), *Behavior Modification*:** two coders extracted 3,596 data points from 168 series in 36 graphs with WPD. The authors report high intercoder reliability and validity. — [PubMed 27760807](https://www.ncbi.nlm.nih.gov/pubmed/27760807)
- **Moeyaert et al. (2016):** compared four programs including WPD. Reliability and validity depended on the individual graphs, not on the software. WPD and Ungraph received the highest usability scores. — [summary](https://behavioristbookclub.com/aba-research/assessment-research/reliability--validity--and-usability-of-data-extraction-programs-for-s/)
- **PlotDigitizer study (a different tool):** 6,846 points digitized on three computers. Intercoder and intracoder correlations were described as "nearly perfect". — [Aydin & Yassikaya, Perspectives on Behavior Science 2022](https://avesis.ankara.edu.tr/yayin/6850a7fb-53c9-40be-8454-36085c22774f/validity-and-reliability-analysis-of-the-plotdigitizer-software-program-for-data-extraction-from-single-case-graphs)
- **EpiCurveBench (2026) [preprint]:** used WPD for human ground truth. Two annotators per curve reached a mean inter-annotator ECS of **90.9%**, and digitizing one complex epidemic curve by hand "can take up to three hours". — [arXiv 2605.27195](https://arxiv.org/html/2605.27195)

### Inferences
- WPD's calibrate-then-extract pipeline (axis anchor points, then colour mask, then per-chart-type algorithm, then manual edit) is a well-tested, permissively describable workflow. SONARITHM can re-implement it in Rust from the documented behaviour without touching AGPL code. Copying or porting WPD's AGPL frontend code into a desktop app would impose AGPL obligations on the combined work. Given the v5 licensing uncertainty, avoid depending on it at all.
- Human-calibrated digitizing gives near-perfect agreement on simple charts, but ECS of about 91% between careful humans on dense real-world epicurves shows that even humans disagree on complex charts. SONARITHM should present extraction as an editable estimate, never as ground truth.

### Gaps
- Exact date and terms of the WPD v5 licence change, whether v5 is free or paid, and whether an offline desktop build still exists. The automeris.io posts and blog could not be retrieved.
- No quantitative error magnitudes (e.g. mean absolute % error vs. true values) were found for WPD's *automatic* algorithms specifically; the validation studies cover mostly manual point clicking.
- Whether Engauge development continues anywhere official, or only in forks.

---

## 2. Deep-learning chart understanding models (ChartOCR → TinyChart/ChartGemma and newer): tasks, accuracy, size, license, weights

### Takeaway
Specialized chart models fall into three generations:
- **Detection/keypoint pipelines** (2021–23): ChartOCR, ChartReader, ChartDETR, LineFormer.
- **Small OCR-free image-to-table transformers** (2022–24): Pix2Struct, MatCha, DePlot, UniChart, OneChart.
- **Chart-tuned multimodal LLMs** (2023–25): ChartLlama, ChartAssistant, ChartInstruct, TinyChart, ChartGemma, ChartMoE.

The specialized models score extremely well **in-distribution** (TinyChart reaches 95.2 RMS-F1 on ChartQA). They **collapse out-of-distribution**: TinyChart scores 28.6 RMS-F1 on World Bank charts, 18.3 on line charts and 8.5 RMS on epidemic curves. In 2026, large general VLMs beat them on realistic charts. Among small models, DePlot (282M, Apache-2.0) and OneChart (0.2B, with a built-in confidence score) remain the most practical for local, CPU-friendly first-pass extraction.

### Cited Findings
#### Model comparison table

| Model (venue/year) | Main task | Size | Reported accuracy (selected) | License / weights | Sources |
|---|---|---|---|---|---|
| **ChartOCR** (WACV 2021, Microsoft) | Keypoint detection + rule-based data extraction (bar/pie/line) | n/a | ExcelChart400k: pie 0.918, bar 0.919 (own table); PlotQA plot-to-table RNSS 81.0 / RMS-F1 60.1 (DePlot paper) | Not verified | [WACV paper](https://openaccess.thecvf.com/content/WACV2021/papers/Luo_ChartOCR_Data_Extraction_From_Charts_Images_via_a_Deep_Hybrid_WACV_2021_paper.pdf); [DePlot](https://arxiv.org/html/2212.10505v2) |
| **ChartReader** (2023) | Unified chart derendering without heuristic rules | n/a | Component detection bar/pie/line 0.95/0.95/0.97; ~22 GPU-h training vs 57 for ChartOCR | Not verified | [arXiv 2304.02173](https://arxiv.org/pdf/2304.02173) |
| **ChartDETR** (2023) | Transformer multi-shape keypoint detection | n/a | ExcelChart400k pie 0.975, bar 0.923; Adobe Synthetic F1 0.98 (vs 0.71 previous best) — authors' numbers | Not verified | [arXiv 2308.07743](https://arxiv.org/pdf/2308.07743) |
| **LineFormer** (ICDAR 2023) | Line-chart data extraction as instance segmentation (MMDetection) | n/a | AdobeSynth 97.51 (detection) / 97.02 (data extraction); LineEX 99.20 / 97.57 | Code and checkpoint on GitHub; license not verified | [GitHub](https://github.com/TheJaeLal/LineFormer); [Springer LNCS 14191](https://link.springer.com/chapter/10.1007/978-3-031-41734-4_24) |
| **Pix2Struct** (Google, 2022/23) | OCR-free screenshot-parsing pretraining (base for MatCha/DePlot) | base/large | — | Not verified in this pass | [survey](https://arxiv.org/html/2403.12027) |
| **MatCha** (Google) | Chart-to-table / chart-to-code / math pretraining | (DePlot initialized from it) | PlotQA plot-to-table RNSS 95.4 / RMS-F1 92.3 | Not verified | [DePlot paper](https://arxiv.org/html/2212.10505v2); [survey](https://arxiv.org/html/2403.12027) |
| **DePlot** (ACL Findings 2023) | Plot-to-table, then LLM reasoning | **282M** (paper); HF shows 0.3B F32 | PlotQA RNSS 97.1 / RMS-F1 94.2; OOD TaTa charts ~78 RMS-F1; 2026 re-eval: ChartQA 88.32, WB-ChartExtract 23.06 RMS-F1 | **Apache-2.0**, weights on HF (`google/deplot`) | [paper](https://arxiv.org/html/2212.10505v2); [HF](https://huggingface.co/google/deplot); [arXiv 2605.27298](https://arxiv.org/html/2605.27298) |
| **UniChart** (EMNLP 2023) | Chart pretraining (table, reasoning, summaries); Donut-initialized, 512→960 px | Not verified | — | Code at github.com/vis-nlp/unichart; license not verified | [ACL Anthology](https://aclanthology.org/2023.emnlp-main.906); [arXiv](https://arxiv.org/pdf/2305.14761) |
| **ChartLlama** (2023) | Instruction-tuned LLaVA-1.5 on synthetic charts | 13B | ChartQA avg 69.66; chart-to-table RMS-F1 90.00 | Not verified | [TinyChart Table 2](https://arxiv.org/html/2404.16635) |
| **ChartAssistant** (2024) | Chart-to-table pretraining + instruction tuning | 13B | ChartQA avg 79.90; RMS-F1 91.60 | Not verified | [TinyChart Table 2](https://arxiv.org/html/2404.16635) |
| **ChartInstruct** (2024) | Instruction tuning | 7B | ChartQA avg 66.64 | Not verified | [TinyChart Table 2](https://arxiv.org/html/2404.16635) |
| **ChartVLM / ChartX** (2024) | Chart LVLM + 18-type benchmark | 7.3B | AP@strict 71.84 (OneChart's table) | Not verified | [arXiv 2402.12185](https://arxiv.org/pdf/2402.12185); [OneChart](https://arxiv.org/html/2404.09987) |
| **OneChart** (2024) | Chart structural extraction with auxiliary number decoder + **self-consistency confidence** | **0.2B** (SAM-base + OPT-125M) | AP@strict ChartQA-SE 72.02 (81.97 after confidence filter), PlotQA-SE 34.56, ChartX-SE 44.55; 2026 re-eval: RMS-F1 35.93 ChartQA / 26.49 WB; EpiCurveBench RMS 4.4 | License not stated in paper; project page onechartt.github.io | [arXiv 2404.09987](https://arxiv.org/html/2404.09987); [2605.27298](https://arxiv.org/html/2605.27298); [2605.27195](https://arxiv.org/html/2605.27195) |
| **TinyChart** (EMNLP 2024) | Chart MLLM (TinyLLaVA: SigLIP + Phi-2) with Visual Token Merging + Program-of-Thought | **3B** | ChartQA avg 83.60 (@768); chart-to-table RMS-F1 93.78; 3.14–3.65 it/s on V100; 2026 re-eval: ChartQA RMS-F1 95.20 (fine-tuned on ChartQA train), WB 28.61 (line 18.32, area 13.02); EpiCurveBench RMS 8.5 | Code and model in mPLUG-DocOwl repo; license not stated on paper page | [arXiv 2404.16635](https://arxiv.org/html/2404.16635); [2605.27298](https://arxiv.org/html/2605.27298); [2605.27195](https://arxiv.org/html/2605.27195) |
| **ChartGemma** (2024) | PaliGemma-based chart instruction tuning (QA, summaries, fact-check; PoT prompting) | PaliGemma-class (3B base) | Authors claim SOTA on 5 benchmarks; ChartQA 80.2 per aggregator **[aggregator, unverified]** | Repo code **GPL-3.0**; weights `ahmed-masry/chartgemma` (no separate weight license stated; base PaliGemma requires accepting Google terms) | [GitHub](https://github.com/vis-nlp/ChartGemma); [arXiv 2407.04172](https://arxiv.org/abs/2407.04172v1); [benchmarklist](https://benchmarklist.com/models/chartgemma/); [PaliGemma HF](https://huggingface.co/google/paligemma-3b-pt-448-jax) |
| **ChartMoE** (ICLR 2025 oral) | MoE connector MLLM: understanding, replotting, editing, transformation | Not verified | ChartQA eval scripts in repo; score not retrieved | GitHub DataArcTech/ChartMoE | [GitHub](https://github.com/DataArcTech/ChartMoE) |
| **PaddleOCR-VL** (document parser, 2025–26) | General document parsing incl. charts | Not verified | RMS-F1 0.844 on an in-house chart benchmark, where TinyChart/OneChart score "far lower" **[preprint]** | Not verified | [arXiv 2603.24326](https://arxiv.org/pdf/2603.24326) |

#### Additional details
- **DePlot internals:** initialized from MatCha and fine-tuned on plot-to-table, with 282M parameters. It **discards colour and layout information**, so questions like "which bar is grey" fail. It is distracted by adjacent text and struggles with labels attached via arrows. It does not work on visuals without a clear textual representation. — [DePlot paper](https://arxiv.org/html/2212.10505v2)
- **DePlot usage:** the prompt is "Generate underlying data table of the figure below:" with `max_new_tokens=512`, run via `Pix2StructForConditionalGeneration`. — [HF model card](https://huggingface.co/google/deplot)
- **Parameter-count conflict:** the TinyChart and OneChart comparison tables list DePlot as **1.3B**, while the DePlot paper says 282M and HF shows about 0.3B. The 1.3B figure likely refers to a Pix2Struct-large variant; the primary source says 282M. — [TinyChart](https://arxiv.org/html/2404.16635); [OneChart](https://arxiv.org/html/2404.09987); [DePlot](https://arxiv.org/html/2212.10505v2)
- **OneChart confidence mechanism:**
  - A `<Chart>` token's hidden state feeds a 3-layer MLP that predicts up to 256 normalized numbers.
  - The self-consistency distance between the parsed text numbers and the MLP numbers, S ∈ [0,1], acts as a reliability score.
  - Filtering at δ=0.1 raises ChartQA-SE AP from 72.02 to 81.97.
  - Input is resized to 1024×1024 and encoded as 256 vision tokens. — [arXiv 2404.09987](https://arxiv.org/html/2404.09987)
- **TinyChart Program-of-Thought:** the model emits Python that an interpreter executes for numeric answers. Training used 140,584 PoT pairs, and PoT lifts ChartQA-human to 73.34. — [arXiv 2404.16635](https://arxiv.org/html/2404.16635)
- **TinyChart out of distribution:** it over-generates datapoints on OOD charts, with about 13% extra rows. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298) **[preprint]**
- **Specialized models on realistic epidemic curves:** "missed datapoints" were over half of the total loss for OneChart and TinyChart, which often extract at coarser resolution (weekly instead of daily). They also failed series-label matching more often. — [arXiv 2605.27195](https://arxiv.org/html/2605.27195) **[preprint]**
- **ChartCap (2025):** dense chart captioning designed to reduce hallucination. Per a secondary review, human evaluators preferred its captions over human ground-truth captions. — [arXiv 2508.03164](https://arxiv.org/pdf/2508.03164) **[preprint status not verified]**

### Inferences
- Sub-1B specialized models (DePlot, OneChart) are attractive for an **offline default**: small download and CPU-feasible. But 2026 evaluations show they are brittle on charts unlike their training data, and line and area charts are the weakest case, which matters most for sonification of time series. They should feed a human-review UI, not be trusted silently.
- OneChart's built-in numeric self-consistency score is the closest thing in the literature to a native per-chart **uncertainty signal**. It is worth replicating conceptually, for example by running two decoders or two models and measuring disagreement.
- LineFormer-style instance segmentation is the most directly relevant architecture for **multi-series line charts**, SONARITHM's core use case. It outputs per-line pixel keypoints, which can be mapped through a classically calibrated axis.

### Gaps
- Licenses for Pix2Struct/MatCha checkpoints, UniChart, OneChart, TinyChart, LineFormer, ChartReader and ChartDETR were not verified on their model cards or LICENSE files in this pass. This must be checked before redistribution.
- ChartGemma's exact ChartQA score from the paper, and ChartMoE's scores, were not retrieved from primary sources.
- No 2025–26 specialized *line-extraction* model newer than LineFormer was found in this pass.

---

## 3. Benchmarks and metrics; what state of the art looks like in 2026; failure modes

### Takeaway
ChartQA-style benchmarks are saturated (TinyChart 95.2 and Qwen3-VL 91.4 RMS-F1 on ChartQA tables), so they **overstate real-world accuracy**. On harder 2026 benchmarks, results drop sharply:
- **WB-ChartExtract** (World Bank data, four chart types × four plotting libraries): single-pass RMS-F1 ranges from about 23 to 88.
- **EpiCurveBench** (real public-health epidemic curves): the best model reaches only 52.3% ECS and about 20% RMS.

Line and area charts are much harder than bar charts. Value-reading errors dominate. Dense points, overlapping or stacked series, thin bars, tiny or rotated tick labels, unusual tick formats and many legend entries are the documented failure triggers.

### Cited Findings
**Benchmarks**
- **FigureQA:** 2M instances, synthetic, line/bar/pie. — [survey arXiv 2403.12027](https://arxiv.org/html/2403.12027)
- **DVQA:** 3M, synthetic bar charts. — [survey](https://arxiv.org/html/2403.12027)
- **PlotQA:** 29M, World Bank Open Data and others, line/bar/scatter. — [survey](https://arxiv.org/html/2403.12027)
- **ChartQA:** about 10K, real Statista/Pew charts, line/bar/pie. — [survey](https://arxiv.org/html/2403.12027)
- **ChartX:** 18 chart types, including line, bar, area, pie and scatter. — [survey](https://arxiv.org/html/2403.12027); [ChartX paper](https://arxiv.org/pdf/2402.12185)
- **CharXiv (NeurIPS 2024):** real arXiv charts. Human accuracy is 80.5% on reasoning questions. At release, GPT-4o scored 47.1% and the best open model, InternVL Chat V1.5, 29.2%. — [arXiv 2406.18521](https://www.arxiv.org/abs/2406.18521?context=cs)
- **CharXiv in 2026:** an aggregator reports Claude Mythos Preview at 93.2% with tools (22 May 2026) and 86.1% without tools, ahead of Claude Opus 4.7 (91% / 82.1%) **[aggregator, unverified]**. If accurate, frontier models now exceed the 2024 human baseline on CharXiv reasoning. — [BenchLM CharXiv](https://benchlm.ai/benchmarks/charxiv); [no-tools](https://www.benchlm.ai/benchmarks/charxivNoTools)
- **ChartQAPro (2025):** a harder QA benchmark. Claude 3.5 Sonnet was top in the original paper. A follow-up reports Gemini 2.0 Flash 47.15%, Claude 3.5 Sonnet 46.57% and GPT-4o 38.33%. Open-model figures in that follow-up are internally inconsistent. — [arXiv 2504.05506](https://arxiv.org/pdf/2504.05506); [arXiv 2603.06677](https://arxiv.org/pdf/2603.06677) **[preprint]**
- **ChartHal (2025):** a hallucination benchmark for chart understanding. GPT-5 scored 34.46% and o4-mini 22.79%. Questions about information **absent from or contradicting the chart** were especially likely to trigger hallucinations. — [arXiv 2509.17481](https://arxiv.org/abs/2509.17481) **[preprint]**

**Metrics**
- **RMS (Relative Mapping Similarity), introduced with DePlot:**
  - It treats a table as an unordered set of (row header, column header) → value mappings.
  - Key similarity uses normalized Levenshtein distance; value similarity uses relative distance, clipped above a threshold.
  - Matching is minimal-cost; RMS-F1 is the harmonic mean of precision and recall.
  - It is invariant to row and column permutation, and the transpose is also scored.
  - The DePlot authors' critique of the older **RNSS** (Relative Number Set Similarity): it ignores number position and non-numeric content, credits very high relative errors, and does not separate precision from recall. — [DePlot paper](https://arxiv.org/html/2212.10505v2)
- **Typical 2026 RMS-F1 settings:** header threshold τ=0.5 (normalized Levenshtein) and value threshold θ=0.1 (clipped relative error). The score is the maximum over the prediction and its transpose. RNSS and RD are also reported. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298) **[preprint]**
- **Critique of RMS for time series:** RMS treats a one-day temporal shift as a full mismatch. EpiCurveBench proposes a temporally aware **EpiCurveSimilarity (ECS)**. In one example, a value-correct extraction scored 46.5% RMS-F1 but 95.2% ECS. ChartX's **SCRM** is also reported. — [arXiv 2605.27195](https://arxiv.org/html/2605.27195) **[preprint]**
- **OneChart's AP@strict / slack / high:** average precision at different numeric tolerances, used for structural extraction. — [arXiv 2404.09987](https://arxiv.org/html/2404.09987)

**State of the art in 2026: chart-to-table (WB-ChartExtract; Berkane, Wang, Majumder; 26 May 2026) [preprint]**

| Model | ChartQA RMS-F1 | WB-ChartExtract RMS-F1 |
|---|---|---|
| Gemini 2.5 Pro | 88.23 | **87.83** |
| Claude Opus 4.6 | 87.71 | 60.99 |
| Qwen3-VL-235B-A22B (open) | 91.43 | 52.91 |
| GPT-5.1 | 84.80 | 51.26 |
| Seed 1.6 Flash | 81.62 | 35.08 |
| Llama 4 Scout (open) | 75.13 | 30.41 |
| TinyChart (3B) | 95.20 | 28.61 |
| OneChart (0.2B) | 35.93 | 26.49 |
| DePlot (282M) | 88.32 | 23.06 |
— [arXiv 2605.27298](https://arxiv.org/html/2605.27298)

- **Per chart type on WB-ChartExtract (Gemini / GPT-5.1 / Llama 4 Scout / TinyChart):**
  - Grouped bar: 95.18 / 67.91 / 48.39 / 54.23
  - Stacked bar: 89.39 / 69.67 / 31.52 / 29.28
  - **Line: 87.68 / 38.53 / 28.10 / 18.32**
  - **Area: 79.20 / 29.48 / 13.94 / 13.02**
  - The benchmark contains no pie or scatter charts. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298)
- **Self-ensembling:** sampling several outputs from the same VLM and aggregating cell-wise improves WB-ChartExtract RMS-F1 by up to about 23% relative (Seed 1.6 Flash: 35.08 → 43.17) and Qwen3-VL from 52.91 to 56.23. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298)

**State of the art in 2026: real-world epidemic curves (EpiCurveBench, n=1,000, high-effort setting) [preprint]**
- RMS / ECS by model:
  - OneChart: 4.4 / 9.4
  - TinyChart: 8.5 / 11.4
  - Qwen3-VL-235B: 18.0 / 27.5
  - GPT-5.2: 18.6 / 43.9
  - Claude Opus 4.5: 22.8 / 42.0
  - Gemini 2.5 Pro: 20.5 / **52.3**
- Gemini with code execution reached 69.4% ECS on the hand-annotated subset. The authors conclude that leading VLMs fall "well short of usable extraction quality". — [arXiv 2605.27195](https://arxiv.org/html/2605.27195)

**Documented failure modes**
- **Chart types and value reading:**
  - Area charts are hardest, because stacked or overlapping filled regions are hard to decompose.
  - Dense datapoints, multiple overlaid series and idiosyncratic labelling are named difficulties.
  - **Value errors dominate** for both strong and weak models, especially on line and area charts. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298)
- **Real-world epidemic curves:**
  - Numerical error accounts for 20–30% of the ECS loss, from difficulty reading bar heights.
  - Missed datapoints are a top-two loss for every model.
  - Inaccurate cropping with code tools produces duplicate or missing rows.
  - Listed challenges: dense datapoints, thin bars, low resolution, overlaid text, stacked bars with similar colours, and rotated or tiny axis text. — [arXiv 2605.27195](https://arxiv.org/html/2605.27195)
- **Y-axis bias (Song, Efat, Tavanapong; 27 Apr 2026) [preprint]:**
  - The FairChart2Table framework finds significant biases related to the **digit length of major tick values, the number of major ticks, the value range, and the tick format** (abbreviations like K/M/B, or scientific notation).
  - The **number of legends/entities** affects performance.
  - **Prompting the model with y-axis information significantly improves some models.**
  - Prior work (Mukhopadhyay et al., 2024) found biases tied to chart complexity, colour scheme, font size, **grid lines**, legend position, figure scale, **log-scale y-axis ticks** and x-tick orientation. — [arXiv 2604.24987](https://arxiv.org/pdf/2604.24987) (text read from PDF)
- **DePlot-era failures:** colour information is lost, adjacent text distracts the model, and labels attached via arrows confuse it. — [DePlot paper](https://arxiv.org/html/2212.10505v2)

### Inferences
- For SONARITHM, the most relevant evidence is the **line-chart** result. Outside Gemini 2.5 Pro, single-pass VLM line-chart extraction on a controlled benchmark scored only about 18–39 RMS-F1. Pure-ML line extraction should therefore be presented as a draft for correction.
- RMS-F1 is the de facto chart-to-table metric. For time series it is too harsh on small x-shifts, so SONARITHM's own regression tests should combine RMS-F1 with a curve-distance metric such as ECS-like temporal tolerance or normalized RMSE after resampling.
- Classical calibration (explicit tick-to-value mapping) directly removes the y-axis-bias failure class, and the 2026 paper shows that supplying axis information helps VLMs. This supports a **hybrid** design: calibrate axes classically or with OCR, then let the ML model find marks.
- The very large Gemini lead on WB-ChartExtract comes from a single preprint and should be re-checked before it drives vendor choice.

### Gaps
- No 2025–26 benchmark was found that quantifies accuracy specifically on **log axes**, **missing gridlines**, **scatter** or **pie** chart-to-table with modern VLMs; these are mentioned as bias factors but not separately scored in the sources retrieved.
- No official, up-to-date CharXiv leaderboard was verified; the 2026 numbers come from an aggregator.

---

## 4. General vision-language models for chart-to-table (open-weight and API): accuracy, hallucination, cost, privacy

### Takeaway
- **Open-weight:** the Qwen VL family is the strongest and most permissively licensed. Qwen2.5-VL-7B and Qwen3-VL-8B are Apache-2.0, and Qwen3-VL-235B is the best open model on 2026 chart extraction. At the small end, Florence-2 (MIT, 0.23B/0.77B) is a strong OCR component but not a chart reader, and SmolVLM (256M/500M) scores only about 56–63 on ChartQA QA. Moondream's newer versions carry non-OSI licences.
- **API:** Gemini 2.5 Pro led 2026 chart-extraction preprints. All frontier models still hallucinate on charts (GPT-5 scores 34.5% on ChartHal).
- **Privacy:** OpenAI and Anthropic API data is not used for training by default and is retained for up to about 30 days. Gemini's *free* tier may be used for training and human review.

### Cited Findings
**Open-weight models**
- **Qwen2.5-VL-7B-Instruct:**
  - License Apache-2.0; safetensors show about 8B parameters.
  - ChartQA 87.3, DocVQA 95.7, OCRBench 864.
  - For comparison: InternVL2.5-8B ChartQA 84.8; Qwen2-VL-7B 83.0.
  - The family comes in 3B, 7B and 72B. — [HF model card](https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct)
- **Qwen3-VL-8B-Instruct:** license Apache-2.0, about 9B parameters in BF16. The card links 117 quantized versions for llama.cpp, Ollama and LM Studio. — [HF model card](https://huggingface.co/Qwen/Qwen3-VL-8B-Instruct)
- **Qwen3-VL small-model scores** (Table 4, thinking / instruct) **[parsed from PDF table, verify]**:
  - ChartQA test: 2B 86.6 / 79.1; 4B 88.8 / 84.6; 8B 88.6 / 89.6.
  - CharXiv (DQ): 2B 70.1 / 62.3; 4B 83.9 / 76.2; 8B 85.9 / 83.0.
  - CharXiv (RQ): 2B 37.1 / 26.8; 4B 50.3 / 39.7; 8B 53.0 / 46.4.
  - Source: [Qwen3-VL Technical Report, arXiv 2511.21631](https://arxiv.org/pdf/2511.21631)
- **Qwen3-VL-235B-A22B:**
  - ChartQA 90.3 in both thinking and instruct modes. — [arXiv 2511.21631](https://arxiv.org/pdf/2511.21631); corroborated by [aiwiki](https://www.aiwiki.ai/wiki/qwen3_vl)
  - CharXiv-RQ about 66.1 (thinking) / 62.1 (instruct) **[parsed, verify]**.
  - The report says the thinking variant ranks second only to GPT-5-thinking and Gemini-2.5-Pro-thinking on CharXiv reasoning. — [arXiv 2511.21631](https://arxiv.org/pdf/2511.21631)
- **Florence-2:** MIT licence; base 0.23B and large 0.77B. Prompts include `<OCR>`, `<OCR_WITH_REGION>` (text with quadrilateral boxes), captioning, `<OD>` detection and grounding. No chart-specific ability is claimed. — [HF Florence-2-large](https://huggingface.co/microsoft/Florence-2-large)
- **SmolVLM (v1):** 256M scores ChartQA 55.6% and 500M scores 62.8%. GGUF builds of SmolVLM2 256M and 500M exist in the ggml-org collection. Apache-2.0 for the 500M variant is per an aggregator only. — [SmolVLM paper arXiv 2504.05299](https://arxiv.org/pdf/2504.05299); [ggml-org GGUF collection](https://huggingface.co/collections/ggml-org/smolvlm-gguf)
- **Moondream:**
  - Moondream 2 is about 1.9B parameters, Apache-2.0, with a SigLIP encoder. — [Roboflow](https://playground.roboflow.com/models/moondream-ai/moondream-2)
  - Moondream 3 Preview weights use **Business Source License 1.1**: production use is allowed except offering it as a competing paid hosted service, and it converts to Apache-2.0 two years after release. — [LibreMoondream3 mirror](https://huggingface.co/LibreYOLO/LibreMoondream3)
  - Moondream 3.1 (9B-A2B) uses the **Moondream Model License 1.0**, which allows commercial use, modification and redistribution inside products but not hosting general-purpose Moondream as a service. — [moondream.ai license](https://moondream.ai/licenses/model/1.0); [orcarouter blog](https://www.orcarouter.ai/ko/blog/moondream-3-1-9b-a2b-vs-ui-venus-2-9b)
- **PaliGemma:** weights require accepting Google's usage terms. One configuration reports ChartQA relaxed accuracy of 57.08 (human) and 71.36 (aug); the column-to-resolution mapping is unclear. — [HF PaliGemma](https://huggingface.co/google/paligemma-3b-pt-448-jax); [model card mirror](https://ai.google.dev/gemma/docs/paligemma/model-card?hl=vi)
- **Phi-4-multimodal-instruct:**
  - The official ONNX build is **MIT** with int4 GPU builds for CUDA and DirectML. A CPU build is "not yet provided"; users must generate their own.
  - ChartQA claims conflict: a community MLX conversion reports 81.4 for the FP16 reference (on a 100-sample subset), while a blog reports 69. — [HF Phi-4-multimodal-instruct-onnx](https://huggingface.co/microsoft/Phi-4-multimodal-instruct-onnx); [MLX conversion](https://huggingface.co/Ferox-AI/Phi-4-multimodal-instruct-mlx-4bit); [wbolt blog](https://wbolt.com/tw/microsoft-phi-4-multimodal.html)
- **InternVL 2.5/3 (1B–14B):** supported in llama.cpp. — [llama.cpp multimodal docs](https://github.com/ggml-org/llama.cpp/blob/master/docs/multimodal.md). InternVL2.5-8B scores ChartQA 84.8. — [Qwen2.5-VL card](https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct)

**API models: accuracy and cost**
- **WB-ChartExtract (2026) [preprint]:** Gemini 2.5 Pro 87.83, Claude Opus 4.6 60.99, GPT-5.1 51.26 RMS-F1. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298)
- **EpiCurveBench [preprint]:**
  - ECS: Gemini 2.5 Pro 52.3, GPT-5.2 43.9, Claude Opus 4.5 42.0.
  - Reported costs for the high-effort runs: Qwen3-VL $1.98, Gemini $15.75, GPT-5.2 $16.96, Claude $29.60. — [arXiv 2605.27195](https://arxiv.org/html/2605.27195)
- **Early 2024 ChartQA averages:** GPT-4V 78.5, Gemini Ultra 80.8, Qwen-VL-Max 79.8. — [TinyChart Table 2](https://arxiv.org/html/2404.16635)

**Hallucination risk**
- **ChartHal:** GPT-5 scores 34.46% and o4-mini 22.79%. Questions about missing or contradicting information trigger hallucinations. — [arXiv 2509.17481](https://arxiv.org/abs/2509.17481)
- **CHOCOLATE:** builds a typology of factual errors in generated chart captions. Even GPT-4V frequently produces factually inaccurate captions. The authors propose the **ChartVE** visual-entailment checker and the C2TFEC correction framework. — [arXiv 2312.10160](https://arxiv.org/html/2312.10160v1)
- **"How Do VLMs Behave When Blind or Misled?" (2026):** tests VLMs with captions that are about 70% correct but contain 2–3 subtle errors in values, trends or rankings. This mirrors the risk that a blind user cannot verify a generated description. — [arXiv 2608.13267](https://arxiv.org/pdf/2608.13267) **[preprint]**

**Privacy and data-use policies (API)**
- **OpenAI API:**
  - Data is "not used to train or improve OpenAI models" unless the customer opts in.
  - Abuse-monitoring logs are kept for up to 30 days.
  - Zero Data Retention requires approval.
  - Images sent to `/v1/responses` and `/v1/chat/completions` are scanned for CSAM, and flagged images are retained even under ZDR. — [OpenAI "Your data"](https://developers.openai.com/api/docs/guides/your-data)
- **Anthropic API:**
  - Inputs and outputs are deleted from the backend "within 30 days".
  - Exceptions: ZDR agreements, the Files API, usage-policy enforcement (flagged content up to 2 years, classifier scores up to 7 years), legal requirements, and "Covered Models" with separate retention requirements. — [Anthropic Privacy Center](https://privacy.claude.com/en/articles/7996866-how-long-do-you-store-my-organization-s-data)
  - Secondary sources say the Commercial Terms bar training on customer content. — [char.com summary](https://char.com/blog/anthropic-data-retention-policy)
  - **Conflict:** one secondary source claims API retention dropped to 7 days in September 2025, which contradicts the Privacy Center's 30 days. — [anarlog.so](https://anarlog.so/blog/anthropic-data-retention-policy)
- **Google Gemini API:**
  - Unpaid services: prompts and responses may be used to improve Google products and ML, and human reviewers may read them (disconnected from the account). Users are told not to submit sensitive data.
  - Paid services: data is not used for product improvement.
  - Google AI Studio counts as free even for paid accounts.
  - Sources are the 2024 archived terms plus commentary; the live terms were not verified. — [Gemini API terms archive](https://ai.google.dev/gemini-api/terms-archive/terms_05_02_24); [Simon Willison](https://simonwillison.net/2024/Oct/17/gemini-terms-of-service)

### Inferences
- For a local, licence-clean default VLM, **Qwen2.5-VL-3B/7B or Qwen3-VL-2B/4B/8B** (Apache-2.0 for the sizes verified) offer the best accuracy per byte. Florence-2 is best used as a permissive **OCR/tick-label reader** inside a classical pipeline rather than as a chart-to-table model.
- Cloud APIs are clearly more accurate on hard charts. They must be **opt-in**, with a clear disclosure that the image leaves the device. Recommend paid-tier endpoints only, because Gemini's free tier allows training and human review.
- Because hallucination is common even in frontier models, any LLM-produced numbers should be treated as **claims to be verified against the extracted table**, never as the table itself.

### Gaps
- Licences for Qwen2.5-VL-3B and -72B, the Qwen3-VL MoE variants, InternVL3 and PaliGemma 2 were not verified on their own cards.
- No apples-to-apples chart-to-table RMS-F1 numbers were found for small local VLMs (Qwen-VL 2B–8B, SmolVLM, Florence-2, Moondream, Phi-4-MM) on realistic line charts.
- Per-image API pricing was not collected; only the EpiCurveBench run totals are available.

---

## 5. Local inference feasibility for a Rust desktop app (ONNX/`ort`, candle, llama.cpp, quantization, sizes, latency, licensing)

### Takeaway
Two practical local paths exist:
- **llama.cpp's libmtmd** for chat-style VLMs. It covers GGUF builds of Qwen2/2.5-VL, Gemma 3/4, SmolVLM, InternVL, Pixtral, Moondream2 and Llama 4, and offers an OpenAI-compatible `llama-server`.
- **ONNX Runtime via the Rust `ort` crate** for small encoder-decoder or detection models: DePlot/Pix2Struct, Florence-2, LineFormer-style segmenters and OCR.

candle is pure Rust and permissively licensed, but its README lists few VLMs (Moondream, BLIP, TrOCR, CLIP). Redistribution is easiest with Apache-2.0/MIT models (DePlot, Qwen2.5-VL-7B, Qwen3-VL-8B, Florence-2, Phi-4-MM). Gemma-terms, BSL and GPL-coded models need care.

### Cited Findings
- **llama.cpp multimodal (libmtmd):**
  - Supports image, audio and video input through `llama-cli`, `llama-server` (OpenAI-compatible `/chat/completions`) and `llama-mtmd-cli`.
  - Vision needs a separate **mmproj** projector file, which is offloaded to GPU by default.
  - Pre-quantized GGUFs (mostly Q4_K_M) exist for:
    - Gemma 3 (4B/12B/27B) and Gemma 4 (E2B/E4B/26B-A4B/31B)
    - SmolVLM/SmolVLM2 (256M/500M/2.2B)
    - Pixtral 12B
    - Qwen2-VL 2B/7B and **Qwen2.5-VL 3B/7B/32B/72B**
    - Mistral Small 3.1
    - InternVL 2.5/3 (1B–14B)
    - Llama 4 Scout
    - Moondream2 (2025-04-14)
  - Some models "may require large context window" (e.g. `-c 8192`). The page lists no quality benchmarks and does not list Qwen3-VL. — [llama.cpp multimodal.md](https://github.com/ggml-org/llama.cpp/blob/master/docs/multimodal.md)
- The **Qwen3-VL-8B** card links quantized builds for "llama.cpp, Ollama, LM Studio". — [HF](https://huggingface.co/Qwen/Qwen3-VL-8B-Instruct)
- **`ort` (Rust ONNX Runtime wrapper):**
  - The latest version seen is **2.0.0-rc.13**, wrapping ONNX Runtime 1.28; I found no stable 2.0 release. — [docs.rs ort 2.0.0-rc.13](https://docs.rs/crate/ort/2.0.0-rc.13/source/README.md)
  - Execution-provider feature flags reported by a downstream project: cuda, tensorrt, directml, coreml, rocm, openvino, onednn, xnnpack, qnn. It warns that without enabling the feature, ort "may silently fall-back to CPU". — [fbilhaut/orp](https://github.com/fbilhaut/orp)
  - Under the old 1.x docs, only CUDA/TensorRT providers were in the default download; DirectML and CoreML needed a custom build. This may have changed in 2.x. — [docs.rs ort 1.14.1](https://docs.rs/crate/ort/1.14.1)
- **candle:**
  - Dual Apache-2.0/MIT licence.
  - Backends: optimized CPU (MKL/Accelerate), CUDA (cuDNN, NCCL) and WASM.
  - Supports llama.cpp quantized types, but its quantized examples are LLMs.
  - Vision examples in the README: Moondream, BLIP, TrOCR, CLIP, Segment Anything, YOLOv3/v8, DINOv2 and others. PaliGemma, Florence-2, Qwen2-VL and SmolVLM are not mentioned in the README. — [huggingface/candle](https://github.com/huggingface/candle)
- **Phi-4-multimodal:** the official ONNX build ships int4 (RTN, block 32) for CUDA and DirectML only; CPU must be self-built. MIT licence. — [HF Phi-4-multimodal-instruct-onnx](https://huggingface.co/microsoft/Phi-4-multimodal-instruct-onnx)
- **DePlot:** a Transformers `Pix2Struct` model of about 0.3B F32 parameters, Apache-2.0. Conversion from T5X checkpoints is supported. — [HF google/deplot](https://huggingface.co/google/deplot)
- **Model footprints and latency (as published):**
  - OneChart is 0.2B, with 1024×1024 input. — [arXiv 2404.09987](https://arxiv.org/html/2404.09987)
  - TinyChart (3B) runs at 3.14–3.65 it/s on a V100 at batch size 1, versus 1.47–1.94 it/s for 7–13B chart MLLMs. — [arXiv 2404.16635](https://arxiv.org/html/2404.16635)
  - Florence-2 is 0.23B/0.77B. — [HF](https://huggingface.co/microsoft/Florence-2-large)
  - SmolVLM is 256M/500M. — [arXiv 2504.05299](https://arxiv.org/pdf/2504.05299)
- **LineFormer** is built on MMDetection and tested on PyTorch 1.13.1 / CUDA 11.7. An ONNX export path is not documented in the README summary. — [GitHub](https://github.com/TheJaeLal/LineFormer)
- **ChartGemma:** offers 4-bit bitsandbytes loading. Its fine-tuning Colab supports LoRA/QLoRA on a 24 GB GPU. Code is GPL-3.0. — [GitHub](https://github.com/vis-nlp/ChartGemma)
- **Licence summary from cards:**
  - DePlot: Apache-2.0. — [HF](https://huggingface.co/google/deplot)
  - Qwen2.5-VL-7B: Apache-2.0. — [HF](https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct)
  - Qwen3-VL-8B: Apache-2.0. — [HF](https://huggingface.co/Qwen/Qwen3-VL-8B-Instruct)
  - Florence-2: MIT. — [HF](https://huggingface.co/microsoft/Florence-2-large)
  - Phi-4-MM ONNX: MIT. — [HF](https://huggingface.co/microsoft/Phi-4-multimodal-instruct-onnx)
  - PaliGemma: Google usage terms must be accepted. — [HF](https://huggingface.co/google/paligemma-3b-pt-448-jax)
  - Moondream 3 Preview: BSL 1.1. — [mirror](https://huggingface.co/LibreYOLO/LibreMoondream3)
  - ChartGemma code: GPL-3.0. — [GitHub](https://github.com/vis-nlp/ChartGemma)

### Inferences
- **Recommended runtime split:**
  - `ort` for small fixed-graph models: OCR (Florence-2 or a dedicated OCR model), a line or mark segmenter, and DePlot/OneChart, ideally int8-quantized.
  - llama.cpp, linked via FFI or run as a sidecar `llama-server` bound to localhost, for an optional "local VLM" tier with Qwen2.5-VL-3B/7B GGUF Q4_K_M.
  - This keeps the default install small and makes the multi-GB VLM an optional download.
- **Approximate download sizes** (my arithmetic, not sourced):
  - DePlot at 0.3B parameters × 4 bytes is about 1.1 GB in F32, roughly 300 MB at int8.
  - Florence-2-base at 0.23B is about 0.5 GB in FP16.
  - A 7B VLM at Q4_K_M is roughly 4–5 GB plus a projector of several hundred MB.
  - These should be confirmed against actual files.
- Watch `ort`'s silent CPU fallback: SONARITHM should surface which execution provider actually loaded, because latency estimates shown to users depend on it.
- To stay redistributable inside an OSS app, prefer Apache-2.0/MIT weights. Make Gemma-terms or BSL models user-downloaded with licence acceptance, not bundled. Avoid linking GPL-3.0 code unless SONARITHM's own licence is compatible.

### Gaps
- No measured CPU latency numbers were found for DePlot, Florence-2, Qwen2.5-VL-3B or Qwen3-VL on consumer laptops (as opposed to the V100 numbers above).
- I did not verify which Pix2Struct, Florence-2 or Qwen-VL models have maintained ONNX exports (e.g. Optimum or onnx-community repos), or whether llama.cpp now officially supports Qwen3-VL (the docs page did not list it).
- No exact GGUF file sizes were collected.

---

## 6. Exact (non-ML) extraction paths to prefer when available (vector PDF, SVG, web chart JSON, Office chart XML)

### Takeaway
Whenever the source is not a raster image, exact or near-exact data is usually recoverable without ML:
- **Office files (XLSX/PPTX/DOCX):** charts store cached series values in `c:numCache`, often with an embedded workbook.
- **Web charts:** expose data in live JS objects. D3 binds data to `__data__`, Vega exposes `view.data(name)`, and Highcharts has the `Highcharts.charts` array.
- **Vector PDFs and SVGs:** encode marks as path geometry that can be inverted through axis calibration. ChartDetective showed this gives 78% lower error than raster digitizing.

These paths should run first, and their outputs should be tagged as exact.

### Cited Findings
**Office Open XML charts (DOCX/PPTX/XLSX)**
- In chart XML, series values live in `c:val`, which contains either a reference (`c:numRef`) or a literal (`c:numLit`). — [Open XML SDK: Values](https://learn.microsoft.com/en-us/previous-versions/office/developer/office-2010/cc880640(v=office.14))
- `c:numRef` pairs a formula (`c:f`) with a `c:numCache` of the last values used. — [NumberReference](https://learn.microsoft.com/pt-br/previous-versions/office/developer/office-2010/cc880521(v=office.14))
- `numCache` stores a format code, a point count (`c:ptCount`) and numeric points (`c:pt`), i.e. the most recent data shown for a series. — [NumberingCache](https://learn.microsoft.com/en-us/previous-versions/office/developer/office-2010/ee880031(v=office.14))
- DOCX/PPTX charts carry both cached values and an **embedded spreadsheet**. PowerTools' ChartUpdater updates both. — [OpenXML Developer](https://openxmldeveloper.org/blog/b/openxmldeveloper/archive/2014/12/09/update-cached-data-and-embedded-xlsx-for-charts-in-docx-pptx.html)
- Category (x-axis) string values also need extracting. Some tools, such as Okapi, historically missed chart text in PPTX/XLSX. — [Okapi issue 503](https://gitlab.com/okapiframework/okapi/-/issues/503)

**Web charts**
- **D3:** "When data is assigned to an element, it is stored in the property `__data__`". `selection.datum` gets or sets bound data. — [d3-selection: Joining](https://d3js.org/d3-selection/joining)
- **Vega/Vega-Lite:** `view.data(name)` "returns the data set with the given name" as a live array; copy it before modifying. `view.getState({data: predicate})` serializes chosen datasets. `toSVG`/`toCanvas` return images, not data. — [Vega View API](https://vega.github.io/vega/docs/api/view/)
- **Highcharts:** `Highcharts.charts` is a static array of the chart objects on the page, with destroyed charts left as `undefined`. Each `Chart` exposes `series` (all current series), `options` (merged configuration) and `userOptions` (the original configuration). — [Highcharts class reference](https://api.highcharts.com/class-reference/Highcharts#.charts); [Chart class](https://api.highcharts.com/class-reference/Highcharts.Chart)
- **LLMs on SVG/Vega-Lite:** a York University thesis used LLMs to deconstruct and restyle Vega-Lite/SVG charts, reporting accurate deconstruction of 93.4% of charts in its evaluation. — [YorkSpace](https://yorkspace.library.yorku.ca/signposting/describedby/7fa5cd01-3189-4127-b10a-35080ede06f1)

**Vector PDF charts**
- ChartDetective extracts data from vector charts in PDFs, recovers occluded data, and achieves 78% lower relative error than raster approaches. — [HAL](https://hal-lara.archives-ouvertes.fr/CRISTAL-LOKI/hal-04017638v1)
- PDF path operators (moveto/lineto/curveto) can be read from content streams and converted to path objects. — [Apryse/PDFTron community](https://community.apryse.com/t/how-can-i-extract-vector-shapes-from-pdf-as-net-graphicspath-objects/571)
- Locating paths is hard because the parser must track the graphics state; grouping subpaths into higher-level objects helps. — [PSU DocEng 2015 figure extraction](https://clgiles.ist.psu.edu:443/pubs/doceng2015-src-figextraction.pdf)
- PyMuPDF documents extracting vector graphics from PDFs in Python. — [PyMuPDF on Medium](https://medium.com/@pymupdf/extracting-and-creating-vector-graphics-in-a-pdf-using-python-4c38820e2da8)

### Inferences
- **Priority order for "bring any chart":**
  1. Structured source: XLSX/PPTX/DOCX chart XML, CSV, or embedded JSON specs.
  2. Live web chart object (D3 `__data__`, Vega `view.data`, Highcharts `series`, Plotly's trace data).
  3. Vector geometry (SVG elements, PDF paths), inverted through axis calibration, with tick labels read as text.
  4. Raster only as a fallback.
- **Office XML:** `c:numCache` gives exact plotted values even when the source workbook is missing. When the embedded workbook exists, it may contain more precision or extra rows.
- **Vector paths:** they yield exact *geometry*, but mapping to values still requires axis calibration. Tick labels are usually real text objects in vector PDFs and SVGs, so calibration can often be automatic and exact. These results should be labelled "exact geometry, calibrated", which is a different confidence class from raster estimates.
- **Plotly:** charts embed trace JSON in the page (passed to `Plotly.newPlot`). This is widely known but **not sourced in this pass**, so treat it as an inference to verify.

### Gaps
- No Rust-specific crates were evaluated for PDF path extraction (e.g. lopdf or pdfium bindings) or SVG parsing (e.g. usvg).
- No published accuracy figures were found for fully automatic vector-PDF chart extraction (ChartDetective is interactive).
- Plotly and ECharts data-access APIs were not verified against their docs.

---

## 7. LLM-assisted features for sonification: descriptions, Q&A, mapping suggestions; risks and mitigations

### Takeaway
The Lundgard & Satyanarayan four-level model gives a principled structure for generated descriptions:
- Blind readers most valued **Level 2 (statistics)** and **Level 3 (trends/patterns)**.
- 63% of blind readers wanted no editorializing.
- The authors recommend automating Levels 1–2.

That maps cleanly onto deterministic computation from the extracted table (extrema, slopes, correlations) with an optional LLM for phrasing Level 3. Grounding is essential, because VLMs hallucinate chart facts frequently. The evidence supports:
- table-first ("DePlot-style") pipelines;
- program-of-thought or code execution for numeric answers;
- self-consistency or ensembling as an uncertainty signal;
- verification checkers.

### Cited Findings
**The four-level model**
- Lundgard & Satyanarayan (IEEE VIS 2021 / TVCG 2022) built the model from a grounded-theory analysis of 2,147 sentences. — [arXiv 2110.04406](https://arxiv.org/pdf/2110.04406.pdf); [IEEE VIS page](https://content.ieeevis.org/year/2021/paper_v-full-1401.html)
- **The levels:**
  - **L1** elemental and encoded properties: chart type, title, axes, encodings.
  - **L2** statistical concepts and relations: extrema, outliers, correlations, comparisons.
  - **L3** perceptual and cognitive phenomena: overall trends, complex patterns, exceptions.
  - **L4** contextual and domain-specific insights.
  — [vis.mit.edu accessible HTML](https://vis.mit.edu/pubs/vis-text-model/)
- **User study:** 30 blind and 90 sighted readers ranked descriptions for 15 visualizations.
  - Blind readers ranked **L2 and L3 most useful** and L1 and L4 least.
  - Sighted readers ranked L3 and L4 most useful.
  - 63% of blind readers said descriptions should avoid subjective interpretation and editorializing.
  - 40% of blind readers said the best descriptions combine levels.
  - L1 rankings were bimodal. — [vis.mit.edu](https://vis.mit.edu/pubs/vis-text-model/)
- **Recommendations:**
  - Prioritize trends and statistics for blind readers.
  - Keep L4 optional.
  - Let readers customize and combine levels.
  - Automate L1–L2 (e.g. from chart specs or ARIA), with humans authoring L3. L4 is currently a human task.
  - Evaluate with disabled readers. — [vis.mit.edu](https://vis.mit.edu/pubs/vis-text-model/)

**Grounding patterns**
- **Table first (DePlot):** convert plot to table, then prompt an LLM. One-shot DePlot+LLM gave a 24.0% improvement over a fine-tuned SOTA on human-written ChartQA questions. — [DePlot abstract](https://arxiv.org/abs/2212.10505)
- **Program-of-Thought (TinyChart):** the model writes Python that an interpreter executes for numeric answers, which improves ChartQA-human accuracy. — [arXiv 2404.16635](https://arxiv.org/html/2404.16635)
- **Built-in confidence (OneChart):** self-consistency between the text decoder and an auxiliary numeric head gives a reliability score. Filtering low-confidence outputs raises AP by about 10 points on ChartQA-SE. — [arXiv 2404.09987](https://arxiv.org/html/2404.09987)
- **Self-ensembling as an uncertainty signal:** repeated sampling plus cell-wise aggregation improves accuracy. The per-chart uncertainty (relative MAD across samples) **correlates with error**: Spearman ρ ≈ −0.30 to −0.37 against RMS-F1 on WB-ChartExtract. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298) **[preprint]**
- **Axis hints:** prompting with y-axis information significantly improves chart-to-table for some models. — [arXiv 2604.24987](https://arxiv.org/pdf/2604.24987) **[preprint]**

**Hallucination evidence and checkers**
- ChartHal found severe hallucination even in GPT-5 (34.46%), especially for questions about information absent from or contradicting the chart. — [arXiv 2509.17481](https://arxiv.org/abs/2509.17481)
- CHOCOLATE found frequent factual errors in GPT-4V chart captions and proposed the ChartVE entailment checker and a correction framework. — [arXiv 2312.10160](https://arxiv.org/html/2312.10160v1)
- Subtle caption errors (2–3 wrong values, trends or rankings in an otherwise correct caption) are a studied attack and robustness case. — [arXiv 2608.13267](https://arxiv.org/pdf/2608.13267) **[preprint]**

### Inferences
- **Descriptions:**
  - Generate L1 deterministically from the chart model: axes, units, series names, ranges.
  - Generate L2 deterministically from the extracted table: min/max with positions, mean, monotonic runs, crossings between series, outliers, correlation.
  - Use an LLM only to *phrase* L3 from a precomputed fact list, with a constraint that every number in the output must appear in the fact list. Verify by post-hoc string and number matching.
  - Offer L4 only on request, labelled as interpretation.
  - This follows the paper's automate-L1/L2 recommendation and blind readers' dislike of editorializing.
- **Natural-language Q&A:** answer by having a local LLM write a small query or code over the extracted table (PoT style), executed in a sandbox, and return the answer together with the cells it used. Never let the VLM answer from pixels alone when a table exists.
- **Sonification-mapping suggestions:** a rule engine over data characteristics is safer and fully offline. Examples: series count leads to timbre or stereo separation; value range and sign lead to pitch range and a diverging reference tone; monotonic or periodic structure leads to tempo or rhythm choices; log-scaled source axes lead to log pitch mapping. An LLM can optionally explain or rank the rules' suggestions. No published evaluation of LLM-suggested sonification mappings was found (see Gaps).
- **Uncertainty to the user:** reuse disagreement signals (OneChart-style, self-ensembling, or classical-vs-ML disagreement) to flag cells. In audio, low-confidence regions could be rendered distinctly, for example with a softer timbre or an earcon, so blind users get an accessible uncertainty display.
- **Privacy:** the default is fully local. Any cloud call needs explicit per-chart opt-in, a disclosure of the vendor's retention policy (about 30 days for OpenAI and Anthropic APIs; Gemini free tier used for training), and must never be used silently for screenshots, which may contain personal data.

### Gaps
- No peer-reviewed study was found evaluating **LLM-generated sonification mapping suggestions**, or LLM descriptions *paired with sonification*, with blind users.
- No study was found that measures blind or low-vision users' trust or error-detection when given AI-extracted chart data with uncertainty cues.
- The Lundgard paper's later sections (beyond 100k characters) were not read; additional design implications may exist there.

---

## 8. Recommended staged "bring any chart" pipeline and uncertainty representation

### Takeaway
Build a cascade that always prefers the most exact source:
1. Structured or embedded data (Office XML caches, web chart objects, SVG/D3 bound data, Vega datasets).
2. Vector geometry (PDF/SVG paths) with automatic axis calibration from text ticks.
3. Classical raster digitizing, WPD-style: calibrate, mask, extract, with human correction.
4. Local ML: a small specialized model or segmenter, then an optional local Qwen-VL.
5. Opt-in cloud VLM.

Every value should carry provenance and an uncertainty estimate, so that sonification and descriptions can expose confidence. The evidence base for this order is ChartDetective's vector advantage, the collapse of specialized models on out-of-distribution line and area charts, VLM hallucination rates, and the gains from axis hints and ensembling.

### Cited Findings (evidence underpinning each stage)
- **Stage 1 (structured):** Office charts store the last plotted values in `c:numCache` (`c:ptCount`, `c:pt`) under `c:numRef` with a formula, or as literal `c:numLit`. — [Microsoft Open XML docs](https://learn.microsoft.com/en-us/previous-versions/office/developer/office-2010/ee880031(v=office.14)); [Values](https://learn.microsoft.com/en-us/previous-versions/office/developer/office-2010/cc880640(v=office.14))
- **Stage 1 (web):** D3 `__data__`, Vega `view.data(name)`, and `Highcharts.charts[i].series` / `options`. — [D3](https://d3js.org/d3-selection/joining); [Vega](https://vega.github.io/vega/docs/api/view/); [Highcharts](https://api.highcharts.com/class-reference/Highcharts#.charts)
- **Stage 2 (vector):** vector-based extraction recovered hidden data with 78% lower relative error than raster tools, in under 4 minutes per chart. — [ChartDetective](https://hal-lara.archives-ouvertes.fr/CRISTAL-LOKI/hal-04017638v1)
- **Stage 3 (classical raster):** WPD's documented workflow combines axis calibration (linear, log, date, polar, ternary, bar), colour-distance masks and averaging-window, X-step-with-spline, template-matching, bar and blob algorithms with manual add, adjust and delete. — [WPD docs](https://automeris.io/docs/digitize/)
- **Stage 3 accuracy:** human-calibrated digitizing shows high reliability. — [Drevon et al. 2017](https://www.ncbi.nlm.nih.gov/pubmed/27760807)
- **Stage 4 (local ML), small models:** in-distribution accuracy is high but out-of-distribution accuracy is poor (TinyChart line 18.32, area 13.02 RMS-F1 on WB-ChartExtract; DePlot 23.06 overall). — [arXiv 2605.27298](https://arxiv.org/html/2605.27298)
- **Stage 4, line charts:** LineFormer reaches about 97 data-extraction scores on synthetic line benchmarks. — [GitHub](https://github.com/TheJaeLal/LineFormer)
- **Stage 4, local VLMs:** Qwen2.5-VL-7B scores ChartQA 87.3 (Apache-2.0). — [HF](https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct). Qwen3-VL-235B is the best open model on WB-ChartExtract at 52.91. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298)
- **Stage 5 (cloud):** Gemini 2.5 Pro 87.83 RMS-F1 on WB-ChartExtract — [arXiv 2605.27298](https://arxiv.org/html/2605.27298). Even the best cloud configuration is only 52.3% ECS on real epidemic curves — [arXiv 2605.27195](https://arxiv.org/html/2605.27195). Vendor retention policies: [OpenAI](https://developers.openai.com/api/docs/guides/your-data); [Anthropic](https://privacy.claude.com/en/articles/7996866-how-long-do-you-store-my-organization-s-data); [Gemini terms](https://ai.google.dev/gemini-api/terms-archive/terms_05_02_24)
- **Uncertainty signals available in the literature:**
  - OneChart's self-consistency score S ∈ [0,1], with δ=0.1 filtering. — [arXiv 2404.09987](https://arxiv.org/html/2404.09987)
  - Ensemble dispersion (relative MAD), which correlates with error. — [arXiv 2605.27298](https://arxiv.org/html/2605.27298)
  - Axis-information prompting, which improves accuracy. — [arXiv 2604.24987](https://arxiv.org/pdf/2604.24987)

### Inferences (proposed design — these are recommendations, not sourced facts)

**Pipeline stages**
- **Stage 0, ingest and classify:** detect the input type (XLSX/PPTX/DOCX, PDF, SVG, HTML/URL, raster). For PDFs, check whether the chart region contains vector paths and text or only an embedded image. For HTML, check for known chart libraries.
- **Stage 1, exact parsers (no ML, offline):**
  - OOXML: unzip, read `xl|ppt|word/charts/chart*.xml`, take series `c:tx` name, `c:cat` and `c:val` (numCache or numLit), and prefer the embedded workbook if present.
  - Web: run in the embedded webview or a headless page with user consent, and read D3 `__data__`, Vega `view.data`, Highcharts `series[].options.data`/`points`, and Plotly traces.
  - SVG: parse marks and text, and read `__data__` only if live.
  - Output carries confidence class `exact`.
- **Stage 2, vector geometry:**
  - Parse PDF or SVG paths and text runs.
  - Auto-calibrate axes from numeric tick text plus tick-mark positions, fitting linear, log or date scales and reporting the residual.
  - Classify marks (polylines become line series, rects become bars, repeated glyphs become scatter markers, arcs become pie slices) and invert through the calibration.
  - Output carries confidence class `exact_geometry_calibrated`, with calibration residual stored.
- **Stage 3, classical raster with human-in-the-loop:**
  - Re-implement the WPD-style workflow in Rust, without AGPL code: 2–4 point axis calibration (assisted by OCR of tick labels using Florence-2 OCR_WITH_REGION or another permissive OCR), colour picking and masking, averaging-window or X-step extraction for lines, bar detection and blob detection.
  - Always show an overlay of extracted points on the image for correction.
  - Output carries confidence class `estimated_cv`, with per-point pixel-quantization error (half a pixel mapped through the calibration) and colour-match distance.
- **Stage 4, local ML assist (opt-in download):**
  - A small specialized model (DePlot or OneChart class via `ort`) or a line-instance segmenter proposes series. Its proposals are re-projected through the Stage-3 calibration, using the model for *mark finding* and classical calibration for *value mapping*.
  - Optionally, a local Qwen2.5-VL-3B/7B or Qwen3-VL-4B/8B (GGUF via llama.cpp) proposes a table, prompted with the extracted axis info.
  - Run k samples or two models and use disagreement as uncertainty.
  - Output carries confidence class `estimated_ml`.
- **Stage 5, opt-in cloud:**
  - Per-chart explicit consent, a vendor-policy notice, paid-tier endpoints only, and API keys supplied by the user.
  - Cloud output is treated as another proposal, reconciled against Stage 3/4 results and shown with disagreement highlighting.

**Data model for uncertainty (Rust structs, conceptual)**
- `Dataset { source: SourceRef, series: Vec<Series>, axes: Vec<AxisCalibration>, extraction_log: Vec<Step> }`
- `AxisCalibration { kind: Linear|Log{base}|Date|Category|Polar, anchors: Vec<(pixel, value, from_text: bool)>, fit_residual, method }`
- `Series { name, name_confidence, points: Vec<DataPoint>, style_hint (colour, dash, marker) }`
- `DataPoint { x: Value, y: Value, y_interval: Option<(lo, hi)>, confidence: f32 (0–1), provenance: Exact|VectorCalibrated|CvEstimated|MlEstimated{model, version}|CloudEstimated{vendor, model}|UserEdited, pixel: Option<(f32, f32)>, flags: {interpolated, occluded, ensemble_disagreement, out_of_axis_range} }`
- Rules:
  - Never drop provenance. User edits override but keep history.
  - Sonification and LLM descriptions read `confidence` and `provenance`. For example, descriptions only state numbers from `Exact`, `VectorCalibrated` or `UserEdited` points without hedging; others are hedged ("approximately").
- **Metrics for SONARITHM's own test suite:** RMS-F1 for table structure, plus resampled curve error for time series, because RMS over-penalizes x-shifts.

### Gaps
- No published end-to-end system was found that combines exact parsers, classical calibration and VLMs with per-cell uncertainty. The design above is a synthesis, not a validated reference architecture.
- No user-study evidence was found on how to best convey extraction uncertainty in *audio* to blind users.
- Calibration of confidence scores (whether a 0.8 confidence really means about 80% within tolerance) is unstudied for chart extraction. Only rank correlations were reported (ρ ≈ −0.3 to −0.37).

## Errata

Dated corrections. The text above is kept as the kickoff snapshot ([research README](../README.md)).

- **2026-10-18, link check.** The ChartDetective PDF cited in §1 (`cs.uwaterloo.ca/~dmasson/pdfs/chartdetective.pdf`) returned HTTP 404. Use the HAL record given on the same line: <https://hal-lara.archives-ouvertes.fr/CRISTAL-LOKI/hal-04017638v1>.
- **2026-10-26, C01-T06 format check.** §6 marks the Plotly and Highcharts data formats as unverified. Checked for PLOTSONA's static readers: Plotly's scatter reference (<https://plotly.com/javascript/reference/scatter/>) documents `x`, `y`, `x0`/`dx`, `mode`, `name` and `fill` as used; plotly.py 6.7.0 writes NumPy arrays as typed arrays (`{"dtype", "bdata"}`, base64, little-endian) and places style prototypes under `layout.template.data` (F-162). Highcharts' `series.line.data` reference (<https://api.highcharts.com/highcharts/series.line.data>) documents three forms: plain y values (x from categories, or from `pointStart` and `pointInterval`), `[x, y]` pairs (a string first item names the point), and point objects. The §6 approach of reading live objects (`Highcharts.charts`, `view.data`) is not used: PLOTSONA reads the options statically (OD-25).
- The Engauge Digitizer GitHub link in §1 is a dead link on purpose: it records the 404 itself. The automeris.io blog link in §1 was already noted as unreachable (TLS host mismatch). Both are excluded from the CI link check.
