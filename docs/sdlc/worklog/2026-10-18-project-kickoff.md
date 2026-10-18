# 2026-10-18 · Project kickoff: research and documentation baseline

> **Phase:** pre-A (kickoff) · **Author:** project lead with an AI coding agent (Claude Code) · **Type:** planning session

## Context

The project starts from an idea: make graphs and data audible, because plotted data and sound waves share structure. Names were brainstormed elsewhere. **SONARITHM** was chosen for the platform, and the other nine names became engines with their mockup icons: NUMISONA, PLOTSONA, AUDAXIS, SONOTRACE, RESONUM, AXONOTE, TONALENS, CURVASONA, ECHOMETRIC. The GitHub repository exists and is empty. The owner asked for deep research, a full architecture, an implementation plan, a staged roadmap, AGENTS.md, research-paper tracking and professional SDLC documents, with a Rust core and a professional, non-generic frontend.

## Goals for the session

1. Research the field: tools, science, accessibility, technology, chart extraction, engineering and publication practice.
2. Turn the research into an architecture, a roadmap and a plan that a small team with AI agents can execute.
3. Set up the documentation and tracking structure of a professional project.

## What was done

| Area | Output |
|---|---|
| Research (7 parallel tracks) | [docs/research/notes/01–07](../../research/notes/), synthesized into the [research report](../../research/research-report.md) |
| Rule catalogue | [design-rules.md](../../research/design-rules.md): SDR-01..42, A11Y-01..34 |
| Product | [vision](../../product/vision.md), [requirements](../../product/requirements.md) (FR/NFR), [engine catalogue](../../product/engines.md) |
| Architecture | [docs/architecture/01–12](../../architecture/README.md) |
| Decisions | [ADR-0000..0013](../../decisions/README.md) |
| Planning | [roadmap](../../planning/roadmap.md) (Stages A–D, 27 phases), [implementation plan](../../planning/implementation-plan.md), [risk register](../../planning/risk-register.md), [open decisions](../../planning/open-decisions.md) |
| Engineering, design, SDLC, research-paper track | See [docs/README.md](../../README.md) |

## Decisions made (see ADRs)

- Rust workspace core, edition 2024, core MSRV 1.88 (ADR-0001).
- Tauri 2.12 + React 19.2 + React Aria Components, **provisional until the A02 prototype gate**; Electron is the fallback (ADR-0002, ADR-0003).
- An own render graph behind `AudioBackend`/`RenderGraph` traits on cpal; Firewheel trialled (ADR-0004).
- Declarative JSON spec → canonical Score IR; determinism and render manifests (ADR-0005, ADR-0013).
- Nine engines are capabilities of one compiler, not separate subsystems (ADR-0007).
- Exact-first chart extraction; AI phrases facts and never produces them; offline by default; no telemetry (ADR-0008, ADR-0012).

## Problems and how they were handled

- **Icon clash.** AXONOTE's mockup glyph matches the platform icon, so OD-03 recommends `audio-waveform`.
- **Tagline overreach.** The evidence supports "Hear the patterns" but not yet "Discover what the graphs reveal". The tagline discipline is written into the [vision](../../product/vision.md#7-tagline-discipline).
- **Licence not chosen.** It is the owner's decision, recorded as ADR-0010 (proposed) and OD-01.

## Findings to log

The kickoff research findings are seeded as background entries in the [findings log](../../research-paper/findings-log.md).

## Next steps

1. Owner resolves OD-01 (licence), OD-02 (legal home), OD-04 (ethics route), OD-09 (docs licence) and OD-10 (capacity and funding).
2. Commit and push the baseline; create the GitHub Project board and Stage A issues (A01-T02, A01-T10).
3. Begin A01 (workspace + CI), then A02 spikes in parallel with A03.
