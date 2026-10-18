# Architecture

> **Status:** Baseline v1 (2026-10-18). **Owner:** Architecture lead.
> Decisions behind this architecture are recorded as ADRs in [../decisions/](../decisions/README.md). Proposals that change contracts go through [RFCs](../rfcs/README.md).

Read the documents in order the first time. After that, use them as reference.

| # | Document | Answers |
|---|---|---|
| 01 | [Overview](01-overview.md) | What we are building, principles P1–P10, C4 context and containers, key scenarios, quality budgets, stack summary |
| 02 | [Repository and crate layout](02-repository-and-crate-layout.md) | Directory tree, crate catalogue, enforced dependency rules, feature flags, naming, versioning |
| 03 | [Data model](03-data-model.md) | DataBundle on Arrow, field semantics, the concept table, provenance and confidence, transforms, downsampling, streaming tables |
| 04 | [Sonification spec language](04-sonification-spec.md) | The declarative JSON spec (v0.1 draft): data, transforms, substrate, tracks, marks, channels, context, analysis, music, playback, output, versioning |
| 05 | [Compiler and Score IR](05-compiler-and-score-ir.md) | Pipeline stages, engine registry and capability traits, the Score IR, invariants, the RenderPlan, incremental compilation, diagnostics |
| 06 | [Audio engine](06-audio-engine.md) | Threading, render graph, real-time communication, scheduler and transport, backends, export and manifest, determinism, latency, hearing safety |
| 07 | [Engines](07-engines.md) | What each of the nine engines registers, owns and tests |
| 08 | [Interaction and accessibility](08-interaction-and-accessibility.md) | NavTree and movement verbs, cursor outputs, announcement pipeline, webview accessibility realities, modalities and profiles |
| 09 | [Desktop app](09-desktop-app.md) | Tauri host, IPC primitives and command surface, frontend architecture, security model, persistence, packaging |
| 10 | [Ingestion, extraction and AI](10-ingestion-and-extraction.md) | Source adapters, the tiered "bring any chart" pipeline, the AI assistance policy, accessibility of extraction |
| 11 | [CLI, bindings and plugins](11-bindings-cli-plugins.md) | Rust facade, CLI commands, Python package, WebAssembly, the WIT plugin model |
| 12 | [Cross-cutting concerns](12-cross-cutting-concerns.md) | Errors, logging, settings, project format, i18n, threat model, privacy, performance budgets, licensing, docs generation |

## Conventions for architecture documents

1. Diagrams are Mermaid, kept in the Markdown so they diff and review with the text.
2. Rust signatures are **sketches** until the phase that implements them. The code is the source of truth once it exists, and the doc must then link to it.
3. Every default that implements a research rule names its ID (`SDR-xx`, `A11Y-xx`).
4. When a document and an ADR disagree, the ADR wins until the document is fixed. Fix it in the same PR that is noticed.
