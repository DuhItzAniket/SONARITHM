# Requirements Specification

> **Status:** Baseline v1 (2026-10-18). **Owner:** Project lead.
> **Traceability:** [sdlc/traceability-matrix.md](../sdlc/traceability-matrix.md). **Rules:** [design-rules.md](../research/design-rules.md). **Phases:** [roadmap.md](../planning/roadmap.md).

**Conventions.**
- **Priority** uses MoSCoW: **M**ust, **S**hould, **C**ould, **W**on't (this release).
- **Release** is the version expected to deliver the requirement: `α` = 0.1–0.2, `β` = 0.5, `GA` = 1.0, `D` = Stage D.
- **Basis** cites design rules, accessibility requirements, ADRs or performance budgets (PB-xx in [12 §8](../architecture/12-cross-cutting-concerns.md#8-performance-budgets)).
- IDs are permanent. Retired requirements are struck through with a reason, never renumbered.

## 1. Functional requirements

### 1.1 Data and ingestion

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-001 | Load CSV, TSV, JSON, NDJSON, Parquet, Arrow IPC, XLSX and ODS files | M | A04 | α | — |
| FR-002 | Infer field type, role and concept; report every inference as a diagnostic | M | A04 | α | A11Y-01, SDR-12 |
| FR-003 | Transforms: filter, sort, aggregate, bin, smooth, resample, derive, normalize | M | A04 | α | SDR-21 |
| FR-004 | Accept pyarrow, pandas and polars data from Python without copying where possible | M | A07 | α | ADR-0006 |
| FR-005 | Connect to WebSocket, OSC, serial and MQTT streams; pause, stop and hide streams | M | B06 | β | A11Y-16 |
| FR-006 | Extract chart data exactly from Office chart XML, Vega/Vega-Lite/Plotly/Highcharts JSON, SVG and vector PDF | M | C01 | GA | ADR-0008 |
| FR-007 | Digitize raster charts with axis calibration and an accessible human-correction workflow | S | C02 | GA | ADR-0008 |
| FR-008 | Carry per-value provenance class and confidence through transforms, sound, speech and table | M | A04, C01 | α | P6, H-06 |
| FR-009 | Sample functions from math expressions over a domain, with adaptive sampling | M | C03 | GA | A11Y-28 |
| FR-010 | Local ML-assisted extraction; cloud extraction only with per-chart opt-in | C | D04 | D | ADR-0008, ADR-0012 |

### 1.2 Spec language and compilation

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-020 | A declarative JSON spec with a generated JSON Schema, semantic validation and located diagnostics | M | A03 | α | SDR-01, SDR-04, ADR-0005 |
| FR-021 | Explicit `analysis`, `presentation` and `monitoring` modes recorded in the spec | M | A03 | α | SDR-36 |
| FR-022 | Declared, seeded randomness; identical Score for identical inputs | M | A03 | α | SDR-02, ADR-0013 |
| FR-023 | Spec versioning with explicit migrations | M | A03, A07 | α | — |
| FR-024 | Import Erie specs (subset in A06, broader in C05) | S | A06 | α | SDR-05 |
| FR-025 | Reusable, data-agnostic templates | M | A06 | α | SDR-03 |
| FR-026 | Report every applied default as an `Info` diagnostic naming its rule ID | M | A06 | α | P7 |
| FR-027 | Import Vega-Lite and MAIDR data/encodings into a default spec | S | C01 | GA | Positioning |

### 1.3 Mapping (NUMISONA)

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-030 | Channels: pitch, duration, taps, loudness, pan, timbre, brightness, rate | M | A06 | α | SDR-07, SDR-25 |
| FR-031 | Default pitch 400–1400 Hz in log-frequency; mel, ERB and Bark spaces selectable | M | A06 | α | SDR-08, SDR-09 |
| FR-032 | Concept-based polarity defaults; a one-key flip persisted per profile; ambiguous concepts flagged | M | A06, B03 | α | SDR-12..15 |
| FR-033 | Psychophysical scaling option with labelled exponents | S | A06 | α | SDR-11 |
| FR-034 | Redundant duration and tap-count encodings for precise reading | S | A06 | α | A11Y-13 |
| FR-035 | Density-aware switching from discrete tones to glide or texture (experimental) | C | A06 | α | SDR-20, H-02 |

### 1.4 Context, navigation and output modalities (AUDAXIS)

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-040 | x-axis tick track and dynamic y-axis reference tone | M | B03 | β | SDR-16, SDR-17 |
| FR-041 | Spoken auditory legend (title, ranges, units, polarity) before playback | M | B03 | β | SDR-18 |
| FR-042 | Hierarchical navigation with local, lateral, spatial and targeted verbs and "where am I" | M | B03 | β | A11Y-04, A11Y-07 |
| FR-043 | Earcons for boundaries, invalid actions, landmarks and thresholds, each with a text equivalent | M | B03 | β | A11Y-08 |
| FR-044 | Announcements through the user's screen reader by default, optional self-voicing, configurable verbosity and part order, supersede policy | M | B03 | β | A11Y-12, A11Y-17, A11Y-19 |
| FR-045 | Braille review line and compact braille sparkline | S | B03 | β | A11Y-21 |
| FR-046 | An always-available accessible data table with headers and provenance | M | B04 | β | A11Y-03 |
| FR-047 | Independently toggleable modalities sharing one cursor | M | B03 | β | A11Y-10 |
| FR-048 | Stereo placement (B03); binaural HRTF and multichannel output (D03) | M / C | B03, D03 | β / D | SDR-30 |
| FR-049 | Speech and sonification sequenced, not overlapped; ducking when self-voicing | S | B03 | β | A11Y-20 |

### 1.5 Transport and streams (SONOTRACE)

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-050 | Play overview, pause, stop all, seek, scrub, loop and speed control | M | A05, B01 | α | A11Y-09, A11Y-15 |
| FR-051 | Announce the current value on pause | M | B03 | β | A11Y-09 |
| FR-052 | Monitoring mode: ambient soundscape, salience on detected events, rate-limited announcements | S | B06 | β | SDR-35, A11Y-16 |

### 1.6 Functions and audification (CURVASONA)

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-060 | Cues for zeros, extrema, inflections, asymptotes, undefined and off-screen regions | M | C03 | GA | A11Y-28 |
| FR-061 | Audification with a speed-up calculator and presets | S | C03 | GA | SDR-31 |

### 1.7 Music and output (AXONOTE)

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-070 | Scale and tuning quantization in presentation mode only (rejected in analysis mode) | S | C04 | GA | SDR-10 |
| FR-071 | Instrument library including the BLV discrete square-wave preset | M | A05, C04 | α | SDR-29 |
| FR-072 | Standard MIDI File export; live MIDI and OSC output | S | C04 | GA | — |

### 1.8 Analytics, descriptions and research (ECHOMETRIC)

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-080 | Detect extrema, outliers, change points, periodicity and trend segments as navigable insights | M | C05 | GA | A11Y-23 |
| FR-081 | Deterministic L1/L2 descriptions; interpretation (L4) never by default | M | C05 | GA | A11Y-02 |
| FR-082 | Query commands: min, max, mean, median, mode, SD, sum, range, value-at-x | M | C05 | GA | A11Y-22 |
| FR-083 | Perceptual lint with diagnostics tagged by rule ID | M | A07 (stub), C05 | α / GA | SDR-14, -19, -23, -26..28, -39 |
| FR-084 | Evaluation harness: stimulus generators, replication gallery, study runner with structured logs | M | A08 | α | SDR-40, SDR-42 |

### 1.9 Multivariate and model-based (TONALENS, RESONUM)

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-090 | Interactive lens with linked selection across visual, audio and text | C | D01 | D | A11Y-11 |
| FR-091 | Multi-stream separation by timbre, pan and onset, with a default stream cap | S | D01 | D | SDR-26, H-04 |
| FR-092 | Granular density textures and a PCA → partials quick-look | C | D01 | D | SDR-32, SDR-34 |
| FR-093 | Model-based sonification (data sonogram, crystallization, modal) excited by interaction | C | D02 | D | SDR-33 |

### 1.10 Export and reproducibility

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-100 | WAV and FLAC export with a mandatory render manifest | M | A05 | α | SDR-01, ADR-0013 |
| FR-101 | WebVTT cue sidecar for speech and insight cues | S | A05, B03 | α | A11Y-24 |
| FR-102 | Stem export per bus | C | C04 | GA | — |

### 1.11 Desktop application

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-110 | Project bundles, autosave and crash recovery | M | B05 | β | — |
| FR-111 | Undo/redo for every edit | M | B05 | β | — |
| FR-112 | Spec editor with schema validation and live diagnostics | M | B05 | β | — |
| FR-113 | Mapping inspector grouped by engine, round-tripping to the spec | M | B05 | β | — |
| FR-114 | User profiles and shipped presets | M | B03 | β | SDR-38 |
| FR-115 | Built-in interactive tutorial (≈15 minutes) and context help | M | B07 | β | SDR-37, A11Y-30 |
| FR-116 | Full keyboard operation, command palette, remappable and disableable shortcuts | M | B02 | β | A11Y-05, A11Y-06 |
| FR-117 | Dark, light and high-contrast themes; forced-colors support; reduced motion | M | B02 | β | A11Y-25 |
| FR-118 | Sonification volume independent of system volume; a global "stop all audio" | M | B01 | β | A11Y-15 |
| FR-119 | Non-drag alternatives for scrubbing and pan/zoom | M | B04 | β | A11Y-26 |

### 1.12 Developer interfaces

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-120 | `sonarithm` CLI (render, play, quick, validate, lint, inspect, schema, migrate, devices) | M | A07 | α | — |
| FR-121 | Python package with notebook playback and typed stubs | M | A07 | α | — |
| FR-122 | WebAssembly/npm package with a WebAudio renderer | C | D05 | D | — |
| FR-123 | Sandboxed engine-plugin API (WIT) | C | D05 | D | ADR-0009 |

### 1.13 AI assistance

| ID | Requirement | Pri | Phase | Release | Basis |
|---|---|---|---|---|---|
| FR-130 | LLM-phrased L3 descriptions from a verified fact list, with numeric verification and labelling | C | D04 | D | ADR-0008 |
| FR-131 | Natural-language Q&A answered by generated queries over the table | C | D04 | D | ADR-0008 |
| FR-132 | A rule-based mapping suggester (C05), with optional LLM explanation (D04) | S / C | C05, D04 | GA / D | P7 |

## 2. Non-functional requirements

| ID | Category | Requirement | Measure | Phase |
|---|---|---|---|---|
| NFR-01 | Performance | Meet budgets PB-01..PB-13 on reference hardware | Benchmarks in CI and at release | A02 → C06 |
| NFR-02 | Accessibility | WCAG 2.2 AA via WCAG2ICT; EN 301 549 and Section 508 documented; an ACR per UI release | ACR; manual screen-reader matrix (A11Y-31, A11Y-32) | B02 → every release |
| NFR-03 | Determinism | A byte-identical Score; audio within tolerance across OSes | Golden suite (PB-06) | A05 |
| NFR-04 | Privacy | No network access without explicit consent; no telemetry | Network-deny test | B01 |
| NFR-05 | Security | Threat-model mitigations; fuzzed parsers; signed updates; OpenSSF Scorecard ≥ 7 by GA | Scorecard, fuzzing CI, security review | A01 → C06 |
| NFR-06 | Portability | Windows 10/11 (x86-64, ARM64 later); macOS (Apple Silicon and Intel, minimum set by Tauri 2.12, confirmed in A02); Linux x86-64 (X11 and Wayland, PipeWire) | CI matrix; release smoke tests | A01 → C06 |
| NFR-07 | Licensing | Default builds contain only permissively licensed dependencies; licence notices shipped | cargo-deny, npm licence check, About view | A01 |
| NFR-08 | Maintainability | Public APIs documented; ADRs for significant decisions; coverage targets per [testing strategy](../engineering/testing-strategy.md) | CI doc and coverage checks | A03 → |
| NFR-09 | Internationalization | All UI and announcement strings externalized; locale-aware numbers and units | Pseudo-locale CI check | B02 |
| NFR-10 | Hearing safety | Start-up gain ramp, true-peak limiter, no full-scale onsets | Unit and golden tests | A05 |
| NFR-11 | Reliability | Autosave; recovery offered after a crash; no data loss on device change | Fault-injection tests | B05 |
| NFR-12 | Usability | Install to first sound in ≤ 5 minutes; BLV task completion ≥ 80% at gates | Observed sessions; co-design rounds | A07, B07 |
| NFR-13 | Research integrity | Defaults traceable to rules; findings logged; studies preregistered; materials open | Findings log; OSF; artifact packages | A08 → |
| NFR-14 | Sustainability | Bus factor ≥ 2 for core crates by GA; dependency health reviewed at each gate | Gate review | Gate C |

## 3. Constraints and assumptions

| ID | Type | Statement |
|---|---|---|
| CON-01 | Constraint | Core implementation language is Rust ([ADR-0001](../decisions/0001-rust-workspace-core.md)) |
| CON-02 | Constraint | No copyleft dependencies in default features ([ADR-0010](../decisions/0010-licensing.md)) |
| CON-03 | Constraint | Offline-first; no telemetry ([ADR-0012](../decisions/0012-privacy-and-telemetry.md)) |
| ASM-01 | Assumption | Capacity: one core developer with AI coding agents (≈1.5–2 FTE effective) until Stage C |
| ASM-02 | Assumption | Paid BLV co-designers (A11Y-34) can be recruited from Stage B (budget needed: OD-10 in [open decisions](../planning/open-decisions.md)) |
| ASM-03 | Assumption | An institution can provide ethics review for user studies (see open decision OD-04) |
