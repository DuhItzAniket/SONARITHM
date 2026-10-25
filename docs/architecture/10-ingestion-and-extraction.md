# 10 · Ingestion, Chart Extraction and AI Assistance

> **Status:** Baseline v1 (2026-10-18). File ingestion A04; streams B06; exact chart extraction C01; raster C02; ML and AI D04.
> **Decision:** [ADR-0008 Offline-first extraction and AI](../decisions/0008-offline-first-extraction-and-ai.md). **Evidence:** [notes/06-chart-extraction-ai.md](../research/notes/06-chart-extraction-ai.md).

## 1. Principle: exact before estimated, and label every estimate

Many "chart images" are not really images. Office charts carry their values, web charts carry their data objects, and vector PDFs carry exact geometry. Machine-learning extraction of line charts from raster images is still weak: most models score about 18–39 RMS-F1 on the 2026 WB-ChartExtract preprint. So SONARITHM **tries exact paths first, falls back step by step, and records the provenance class of every value** ([03 §4](03-data-model.md#4-provenance-and-confidence)).

## 2. Source adapters

| Source | Crate / engine | Phase | Notes |
|---|---|---|---|
| CSV, TSV | `sonarithm-data` (csv) | A04 | Delimiter, encoding and header sniffing; locale-aware decimals (opt-in) |
| JSON, NDJSON | `sonarithm-data` | A04 | Records or columns; JSON pointer to the array |
| Parquet, Arrow IPC | `sonarithm-data` (arrow, parquet) | A04 | Preserves `snr.` metadata |
| XLSX, ODS | `sonarithm-data` (calamine) | A04 | Sheet and range selection; formulas take cached values |
| Python objects | `sonarithm-py` | A07 | pyarrow, pandas or polars via the Arrow C Data Interface |
| Functions | CURVASONA | C03 | Expression + domain; adaptive sampling |
| Streams | SONOTRACE / `sonarithm-stream` | B06 | WebSocket, MQTT, serial, OSC-in, file tail |
| Charts | PLOTSONA | C01–C02, D04 | §3 |
| SQL | Later (`sqlx`, feature-gated) | post-1.0 | Read-only queries |

All sources have **size and time budgets** (configurable). They report what they inferred as diagnostics, and run untrusted input through fuzzed parsers.

## 3. The "bring any chart" pipeline (PLOTSONA)

```mermaid
flowchart TB
  IN[Input: file, paste, or drag] --> SNIFF{Container?}
  SNIFF -->|XLSX/PPTX/DOCX| T1A[Tier 1: Office chart XML<br/>c:numCache / c:numLit / embedded workbook]
  SNIFF -->|HTML / JSON| T1B[Tier 1: Vega, Vega-Lite,<br/>Plotly, Highcharts specs]
  SNIFF -->|SVG| T2A[Tier 2: SVG marks + tick labels<br/>→ calibration]
  SNIFF -->|PDF| T2B[Tier 2: vector drawing operators<br/>→ axis detection → calibration]
  SNIFF -->|PNG/JPG/screenshot| T3[Tier 3: raster CV<br/>axis calibration, colour masks,<br/>line/bar/point extraction]
  T2B -->|no vector content| T3
  T3 --> HUMAN[Human correction loop<br/>accessible point entry and editing]
  T3 -.optional, D04.-> T4[Tier 4: local ML<br/>tract/ort models, local VLM sidecar]
  T4 --> HUMAN
  T4 -.explicit opt-in per chart.-> T5[Tier 5: cloud VLM<br/>paid tiers only]
  T5 --> HUMAN
  T1A & T1B --> OUT[(DataBundle<br/>provenance = exact)]
  T2A & T2B --> OUT2[(DataBundle<br/>provenance = vector)]
  HUMAN --> OUT3[(DataBundle<br/>cv/ml/cloud_estimate + confidence,<br/>user_edited where corrected)]
```

| Tier | Method | Provenance | Expected quality | Phase |
|---|---|---|---|---|
| 1 | Read the data the chart was built from (Office XML caches; chart JSON) | `exact` | Lossless | C01 |
| 2 | Geometry + calibration from vector graphics (SVG, PDF operators). ChartDetective reported 78% lower error than raster digitizing | `vector` | Near-exact | C01 |
| 3 | Classical CV, re-implementing WebPlotDigitizer's *documented* workflow (calibration, colour masks, averaging window, X-step, bar and blob extraction). **WPD code is never copied** (AGPL) | `cv_estimate` + confidence | Good on clean charts; C02 target median ≤ 2% | C02 |
| 4 | Local ML: mark detection + classical calibration (better than end-to-end chart-to-table); self-consistency sampling for confidence (OneChart-style) | `ml_estimate` + confidence | Variable; benchmarked before release | D04 |
| 5 | Cloud VLM, **opt-in per chart**, paid API tiers only (no-training terms) | `cloud_estimate` + confidence | Best models strong but hallucination-prone | D04 |

### 3.1 Calibration and validation checks

- Axis calibration supports linear, log and date scales, plus polar and ternary later. Tick labels are read by OCR (Florence-2 class models in D04, classical OCR before that) **or entered by the user**. The point-entry path is fully keyboard-accessible.
- **Sanity checks** run before acceptance: monotonic tick spacing, label/scale consistency (log vs linear detection), series count against the legend, and points within the plot bounds. Failures become diagnostics, never silent fixes.
- **As built (C01-T04).** `sonarithm_plotsona::calibrate` fits a least-squares line from the scale's transform of the anchor values (identity for linear and date axes, `log10` for log axes) to drawing positions. The residual is the root-mean-square distance of the anchors from that line as a share of their position span; above 0.5 % it is reported (`SNR-ENG-325`), and values read through the axis get a confidence of 1 − 10 × residual (a convention until the C01 corpus checks it). Tick labels become anchors through `calibrate::ticks`: numbers with separators, the Unicode minus, SI suffixes (d3-format's `k`, `M`, `G`, `m`, `µ`…, and `B`), exponents and superscript powers of ten; ISO and month-name dates, and d3 multi-scale ticks where month names continue the last year tick. A label it cannot read is skipped and reported (`SNR-ENG-326`), never guessed. The checks are `check_monotonic` (an error, `SNR-ENG-322`), `check_scale` (a log axis read as linear or the reverse, from three anchors: `SNR-ENG-323`), `Calibration::check_bounds` (`SNR-ENG-324`) and the residual. `fit_auto` picks log over linear only when its residual is ten times smaller. Category axes read the nearest label within half the gap to its neighbours. Codes `SNR-ENG-320` to `326`.
- **Human in the loop.** Estimated data opens in a correction view: a table of extracted points with confidence, and an overlay on the image. The workflow is usable by keyboard and screen reader (navigate points, hear them, edit values). Corrected values become `user_edited`.

## 4. AI assistance policy

AI features **phrase facts; they do not produce them** ([research report §LLM features](../research/research-report.md#llm-features-should-phrase-facts-not-produce-them)).

| Feature | Design | Guardrails | Phase |
|---|---|---|---|
| Descriptions L1/L2 | Computed deterministically by ECHOMETRIC | No LLM involved | C05 |
| Description L3 (trends) | An optional LLM phrases a **precomputed fact list** | Every number in the output is checked against the fact list. Unverifiable numbers are removed, and the output is labelled "AI-phrased" | D04 |
| Description L4 (interpretation) | Only on explicit request | Labelled as interpretation. Never in default summaries (A11Y-02) | D04 |
| Natural-language Q&A | The model writes a small query (restricted DSL) over the table; ECHOMETRIC executes it | Answers cite the query and the rows used. The model never reads pixels for numbers | D04 |
| Mapping suggestions | A transparent rule engine (design rules + concept table) proposes; an LLM may only rank or explain | Suggestions show the rules they rest on | D04 (rules engine from A06) |
| Chart extraction | Tiers 4–5 above | Provenance and confidence on every value | D04 |

**Runtime and licensing.** Local inference runs through tract (default, wasm-capable) or ort (opt-in, `load-dynamic`). Local VLMs run in a **llama.cpp sidecar** process, isolated from the app. Only models with Apache-2.0 or MIT weights are offered (for example Qwen2.5-VL-7B, Qwen3-VL-8B, Florence-2, DePlot). **No bundling of GPL code (ChartGemma) or BSL weights (Moondream 3).** Models are never bundled in the installer. They are downloaded on explicit request with checksums ([supply-chain policy](../engineering/supply-chain-and-licensing.md)).

**Privacy.** Cloud calls require per-action consent showing the provider, what is sent and its retention terms. The default is local-only. API keys live in the OS keychain, never in project files ([ADR-0012](../decisions/0012-privacy-and-telemetry.md)).

## 5. Accessibility of extraction

Extraction is often needed most by blind users, who received an image without data. So:

1. The extraction result is announced with its provenance mix ("146 points: 0 exact, 146 estimated, median confidence 0.82").
2. Estimated values carry an estimate qualifier in announcements at every verbosity level.
3. The correction workflow is fully keyboard- and screen-reader-operable, though some corrections are realistically easier with sighted assistance. The UI supports collaborative correction (export and import of a correction file).
4. Rendering uncertainty in sound (H-06) is experimental until the P08 study reports.
