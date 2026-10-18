---
status: accepted
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0005 · Declarative spec compiled to a canonical Score IR

## Context and problem statement

How should users and programs describe a sonification, and what artifact should the system treat as authoritative for testing, rendering and export?

## Decision drivers

* Hermann's criteria: systematic, reproducible, reusable (SDR-01..03).
* A shared vocabulary with visualization grammars (SDR-04) and interop with Erie (SDR-05).
* Testability without comparing raw audio bytes across CPUs.
* Multiple consumers: real-time renderer, offline renderer, MIDI and OSC sinks, navigation, descriptions.

## Considered options

* A declarative JSON spec → compiler → Score IR (event timeline) → renderers and sinks
* An imperative API only (builder calls produce sound directly)
* A spec interpreted directly by the renderer (no intermediate representation)

## Decision outcome

Chosen option: **a declarative JSON spec compiled to a canonical Score IR** ([04](../architecture/04-sonification-spec.md), [05](../architecture/05-compiler-and-score-ir.md)). Rust and Python builders produce the same JSON. The Score IR is the snapshot-tested artifact; audio is compared within tolerance.

### Consequences

* Good: reproducibility becomes operational. Every render ships its spec and hash (ADR-0013). No surveyed tool does this.
* Good: one compiler feeds every renderer and sink. Engines become compiler capabilities (ADR-0007).
* Good: specs are reusable templates ("sonecules"). Erie and Vega-Lite importers become straightforward.
* Bad: interactive features (lens, model excitation, streams) need a live path beside the static Score. This is handled by live events and incremental compilation ([05 §6](../architecture/05-compiler-and-score-ir.md#6-incremental-and-live-compilation)).
* Bad: the spec language is a public contract and needs versioning and migrations ([04 §7](../architecture/04-sonification-spec.md#7-versioning-and-compatibility)).

### Confirmation

**RFC-0001** (opened in A03) freezes `specVersion: "0.1"`. A03 exit criteria require every documented example to validate, a regenerated schema checked in CI, and spec-parser fuzzing.
