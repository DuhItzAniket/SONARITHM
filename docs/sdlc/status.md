# Project Status

> **Last updated:** 2026-10-18 · **Current stage:** pre-A (documentation baseline) · **Next gate:** Gate A (alpha), target Feb 2027
> **Owner:** Project lead. **Update rule:** every PR that changes a task's or phase's state updates this file ([sdlc/README](README.md)).

## 1. Summary

| Item | State |
|---|---|
| Overall health | **Green**: planning complete; no code yet |
| Current focus | Owner decisions OD-01, OD-02, OD-04, OD-09, OD-10, then A01 bootstrap |
| Latest release | none |
| Blocking decisions | OD-01 licence (blocks A01-T13 and any public code) |
| Top risks | R-08 scope creep (20), R-09 bus factor (20), R-02 screen readers in the webview (15) |

## 2. Phase board

Status values: `Not started` · `In progress` · `In review` · `Done` · `Blocked` · `Deferred`.

| Stage | Phase | Name | Status | Progress | Notes |
|---|---|---|---|---|---|
| — | — | Kickoff: research and documentation baseline | **Done** | 100% | 2026-10-18; see [worklog](worklog/2026-10-18-project-kickoff.md) |
| A | A01 | Repository bootstrap and governance | Not started | 0% | Needs OD-01 |
| A | A02 | Technical spikes and prototype gate | Not started | 0% | |
| A | A03 | Core types, Spec v0.1, Score IR | Not started | 0% | RFC-0001 to open |
| A | A04 | Data layer | Not started | 0% | |
| A | A05 | Render engine, audio backend, export | Not started | 0% | |
| A | A06 | NUMISONA v1 | Not started | 0% | |
| A | A07 | CLI, Python, v0.1.0-alpha | Not started | 0% | |
| A | A08 | Research instrumentation v0, ICAD 2027 | Not started | 0% | Ethics application filed by the research lane, RL-T02 (Dec 2026) |
| B | B01 | Desktop shell, IPC, app core | Not started | 0% | |
| B | B02 | Design system and application frame | Not started | 0% | |
| B | B03 | AUDAXIS v1 | Not started | 0% | |
| B | B04 | Visualization and linked cursor | Not started | 0% | |
| B | B05 | Inspector, spec editor, projects | Not started | 0% | |
| B | B06 | SONOTRACE v1 | Not started | 0% | |
| B | B07 | Conformance, tutorial, co-design, v0.5.0-beta | Not started | 0% | JOSS P02 |
| C | C01 | PLOTSONA exact ingestion | Not started | 0% | |
| C | C02 | PLOTSONA raster digitizer | Not started | 0% | |
| C | C03 | CURVASONA | Not started | 0% | |
| C | C04 | AXONOTE | Not started | 0% | |
| C | C05 | ECHOMETRIC v1 | Not started | 0% | CHI 2028 P05 drafted and submitted (est. Sep 2027) |
| C | C06 | Hardening and v1.0 GA | Not started | 0% | P05 artifact package |
| D | D01 | TONALENS | Not started | 0% | |
| D | D02 | RESONUM | Not started | 0% | |
| D | D03 | Spatial and multichannel | Not started | 0% | |
| D | D04 | ML extraction and AI assist | Not started | 0% | |
| D | D05 | Plugins and WebAssembly | Not started | 0% | |
| D | D06 | Research studies programme | Not started | 0% | |

## 3. Current iteration

| Iteration | Dates | Goal | Tasks |
|---|---|---|---|
| I-00 | 2026-10-18 → 2026-11-01 | Owner decisions + A01 start | A01-T01, A01-T02, A01-T10 |

## 4. Engine maturity

| Engine | Maturity | Next milestone |
|---|---|---|
| NUMISONA | Designed | A06 |
| AUDAXIS | Designed | B03 |
| SONOTRACE | Designed | B06 |
| PLOTSONA | Designed | C01 |
| CURVASONA | Designed | C03 |
| AXONOTE | Designed | C04 |
| ECHOMETRIC | Designed | A08 (harness), C05 |
| TONALENS | Designed | D01 |
| RESONUM | Designed | D02 |

Maturity scale: `Designed` → `Prototype` → `Alpha` → `Beta` → `Stable` → `Research-grade` (for engines whose defaults await studies).

## 5. Research track snapshot

| Item | State |
|---|---|
| Findings logged | Background findings from the kickoff research ([findings-log](../research-paper/findings-log.md)) |
| Hypotheses | H-01..H-12 proposed ([hypotheses](../research-paper/hypotheses.md)) |
| Next paper | P01, ICAD 2027 (deadline est. Mar 2027) |
| Ethics | Not started (OD-04) |

## 6. Change history

| Date | Change |
|---|---|
| 2026-10-18 | Status board created at the documentation baseline |
