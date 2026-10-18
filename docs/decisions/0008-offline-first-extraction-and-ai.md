---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0008 · Exact-first, offline-first chart extraction and AI

## Context and problem statement

"Bring any chart" is a headline capability (PLOTSONA), and AI features (descriptions, Q&A, mapping suggestions) are expected in 2026. How do we extract data from charts and use AI without misleading users, especially blind users who cannot check the image?

## Decision drivers

* Accuracy honesty. On the 2026 WB-ChartExtract preprint, most models score about 18–39 RMS-F1 on line charts, and the best model reaches only 52.3% on real epidemic curves ([notes/06](../research/notes/06-chart-extraction-ai.md)).
* Hallucination risk: GPT-5 scores 34.46% on ChartHal.
* Privacy: offline by default (ADR-0012).
* Licences of models and code (GPL ChartGemma, BSL Moondream 3, AGPL WebPlotDigitizer frontend).

## Considered options

* An end-to-end VLM (cloud or local) as the primary extractor
* **A tiered pipeline:** exact containers → vector geometry → classical CV with human correction → local ML → opt-in cloud, with per-value provenance
* No chart extraction (users must supply data)

## Decision outcome

Chosen option: **the tiered, exact-first pipeline with per-value provenance and confidence** ([10](../architecture/10-ingestion-and-extraction.md)). AI phrases facts and never produces them:

* L1/L2 descriptions are deterministic.
* An LLM may phrase L3 from a verified fact list, and its numbers are checked.
* Q&A works by generating queries over the table.
* Mapping suggestions come from a transparent rule engine.

Cloud use requires per-action consent and paid-tier providers. Only Apache-2.0 or MIT model weights are offered, and WebPlotDigitizer is re-implemented from its documentation, never copied.

### Consequences

* Good: exact data whenever it exists (Office XML, chart JSON, vector PDF; ChartDetective reported 78% lower error than raster digitizing).
* Good: every estimate stays labelled through sound, speech and the table (P6).
* Bad: more engineering than "send the image to a VLM". Raster-only charts still need human correction.
* Bad: local models are large downloads. They are never bundled, and are fetched only on explicit request.
