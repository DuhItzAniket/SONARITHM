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
| [0002](0002-desktop-shell-tauri.md) | Tauri 2 as the desktop shell, Electron as fallback | accepted (provisional) | A02 prototype gate |
| [0003](0003-frontend-react-and-react-aria.md) | React 19 + React Aria Components for the UI | accepted (provisional) | A02 prototype gate |
| [0004](0004-audio-backend-and-render-graph.md) | Own render graph over a cpal-backed `AudioBackend` trait | accepted (provisional) | A02 graph spike |
| [0005](0005-declarative-spec-and-score-ir.md) | Declarative spec compiled to a canonical Score IR | accepted | RFC-0001 freezes spec v0.1 (A03) |
| [0006](0006-arrow-data-model.md) | Apache Arrow as the internal data model | accepted | — |
| [0007](0007-engines-as-pipeline-capabilities.md) | Nine engines as capabilities of one compiler | accepted | — |
| [0008](0008-offline-first-extraction-and-ai.md) | Exact-first, offline-first chart extraction and AI | accepted | — |
| [0009](0009-wasm-plugins-compile-time.md) | WebAssembly component plugins, compile time only | proposed | D05 |
| [0010](0010-licensing.md) | Licensing: MIT OR Apache-2.0 for code, CC BY 4.0 for docs and data, DCO | accepted (2026-10-18) | A01-T13 |
| [0011](0011-typed-ipc-ts-rs.md) | Typed IPC via ts-rs and JSON Schema | accepted | B01 |
| [0012](0012-privacy-and-telemetry.md) | No default telemetry; offline by default | accepted | — |
| [0013](0013-determinism-and-provenance.md) | Determinism and render provenance | accepted | A05 golden tests |

## Process

1. Copy [adr-template.md](adr-template.md) to `NNNN-short-title.md` (next free number).
2. Open a PR titled `docs(adr): NNNN short title`. Discuss there; architecture-impacting ADRs need the project lead's approval.
3. On merge, add it to the index above and link it from the documents it affects.
