# Architecture Decision Records

> **Status:** Living index. **Owner:** Architecture lead. **Format:** [MADR 4](https://adr.github.io/madr/) ([template](adr-template.md)).

An ADR records **one significant decision**: its context, the options considered, the outcome and its consequences. ADRs are immutable once accepted. To change a decision, write a new ADR that supersedes the old one, and update the old one's status line only.

## When to write one

Write an ADR when a decision is hard to reverse, affects several crates or engines, changes a public contract (spec, Score IR, IPC, plugin WIT, file formats), adds a heavyweight or licence-sensitive dependency, or changes a research-backed default policy. Use an [RFC](../rfcs/README.md) first when the proposal needs open discussion. The RFC's outcome is then recorded as an ADR.

## Statuses

`proposed` → `accepted` | `rejected`; later `deprecated` or `superseded by ADR-NNNN`. An accepted ADR may carry a **Confirmation** gate (MADR 4). Until that gate passes, the decision is acted on but labelled *provisional*.

## Index

| ADR | Title | Status | Confirmation |
|---|---|---|---|
| [0000](0000-use-madr.md) | Record architecture decisions with MADR | accepted | — |
| [0001](0001-rust-workspace-core.md) | Rust workspace as the single core | accepted | — |
| [0002](0002-desktop-shell-tauri.md) | Tauri 2 as the desktop shell, Electron as fallback | accepted (provisional) | A02 prototype gate: Windows evidence in; WebKitGTK, WKWebView and screen-reader steps pending ([gate report](../sdlc/reviews/A02-prototype-gate.md#adr-decisions)). Gate U open provisionally since 2026-10-29; the Windows walkthroughs follow at a checkpoint, Linux and macOS by B03's exit (OD-15) |
| [0003](0003-frontend-react-and-react-aria.md) | React 19 + React Aria Components for the UI | accepted (provisional) | A02 prototype gate: prototype built; NVDA, Narrator, VoiceOver and Orca walkthroughs pending ([gate report](../sdlc/reviews/A02-prototype-gate.md#adr-decisions)). Components built before the Windows walkthroughs (provisional Gate U, 2026-10-29), with axe, keyboard tests and the UIA tree dump in CI; VoiceOver and Orca by B03's exit (OD-15) |
| [0004](0004-audio-backend-and-render-graph.md) | Own render graph over a cpal-backed `AudioBackend` trait | accepted | Confirmed at the A02 gate, 2026-10-19: owned graph kept, Firewheel not adopted ([gate report](../sdlc/reviews/A02-prototype-gate.md#spike-2-render-graph-owned-versus-firewheel-a02-t02-t03)); the PB-01 latency gap goes to ADR-0016 |
| [0005](0005-declarative-spec-and-score-ir.md) | Declarative spec compiled to a canonical Score IR | accepted | Spec v0.1 frozen by RFC-0001 (2026-10-19, ADR-0015) |
| [0006](0006-arrow-data-model.md) | Apache Arrow as the internal data model | accepted | — |
| [0007](0007-engines-as-pipeline-capabilities.md) | Nine engines as capabilities of one compiler | accepted | — |
| [0008](0008-offline-first-extraction-and-ai.md) | Exact-first, offline-first chart extraction and AI | accepted | — |
| [0009](0009-wasm-plugins-compile-time.md) | WebAssembly component plugins, compile time only | proposed | D05 |
| [0010](0010-licensing.md) | Licensing: MIT OR Apache-2.0 for code, CC BY 4.0 for docs and data, DCO | accepted (2026-10-18) | A01-T13 |
| [0011](0011-typed-ipc-ts-rs.md) | Typed IPC via ts-rs and JSON Schema | accepted | B01-T02: generated protocol types, the generated command → reply table and a contract test over every command; `docs / schema-diff` fails on stale output |
| [0012](0012-privacy-and-telemetry.md) | No default telemetry; offline by default | accepted | — |
| [0013](0013-determinism-and-provenance.md) | Determinism and render provenance | accepted | A05 golden tests |
| [0014](0014-capability-trait-signatures.md) | Capability traits are generic over the data type and take spec fragments | accepted (2026-10-19, coordinator under owner delegation) | A03-T04 (done), A04 instantiates `D` |
| [0015](0015-spec-v0.1-frozen.md) | Spec v0.1 frozen per RFC-0001 | accepted (2026-10-19) | A03-T06..T11 implement it |
| [0016](0016-windows-low-latency-backend.md) | A Windows low-latency audio backend and per-device latency calibration | accepted (2026-10-19, coordinator under owner delegation) | A05 cable measurement against PB-01 |
| [0017](0017-ipc-protocol-v1.md) | IPC protocol v1: one request type in the headless app core | accepted (2026-10-21, coordinator under owner delegation) | B01-T01, B01-T02, B04-T03 |
| [0018](0018-project-format-v1.md) | Project format v1: `.sonarithm` bundles per RFC-0002 | accepted (2026-10-21, coordinator under owner delegation) | B05-T04 |
| [0020](0020-profile-format-v1.md) | Profile format v1 and the layers of effective settings | accepted (2026-10-21, coordinator under owner delegation) | B03-T08 |
| [0021](0021-spec-v0.2.md) | Spec v0.2 per RFC-0004 | accepted (2026-10-26, coordinator under owner delegation) | C01-T01's code half: types, 0.2 schema, migration, doc 04, corpus and doc-example tests |
| [0022](0022-clean-room-raster-digitizer.md) | Clean-room re-implementation of the raster digitizer | accepted (2026-10-26, coordinator under owner delegation) | Accepted by the coordinator before C02-T03; every C02 raster PR lists its register sources |

## Process

1. Copy [adr-template.md](adr-template.md) to `NNNN-short-title.md` (next free number).
2. Open a PR titled `docs(adr): NNNN short title`. Discuss there; architecture-impacting ADRs need the project lead's approval.
3. On merge, add it to the index above and link it from the documents it affects.
